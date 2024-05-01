use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::validation::{BOMChangeEventValidator, ValidationError, Validator};

use super::{BOMChangeEvent, Component};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BOM {
    pub id: Uuid,
    pub name: String,
    pub version: i32,
    pub description: Option<String>,
    pub components: Vec<(Component, i32)>,
}

impl Default for BOM {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            version: 0,
            description: None,
            components: Vec::new(),
        }
    }
}

impl TryFrom<&Vec<BOMChangeEvent>> for BOM {
    type Error = ValidationError;

    fn try_from(value: &Vec<BOMChangeEvent>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ValidationError(
                "Unable to construct BOM without input".to_string(),
            ));
        }

        // Check if events include a NameChanged event
        if !value
            .iter()
            .any(|event| matches!(event, BOMChangeEvent::NameChanged(_)))
        {
            return Err(ValidationError(
                "Creating a BOM without name is not allowed. Please provide a name".to_string(),
            ));
        }

        let mut bom = BOM::default();
        for event in value {
            bom.apply_change(event, BOMChangeEventValidator)?;
        }
        Ok(bom)
    }
}

impl BOM {
    pub fn apply_change<T: Validator<BOMChangeEvent>>(
        &mut self,
        event: &BOMChangeEvent,
        validator: T,
    ) -> Result<(), ValidationError> {
        validator.validate(event)?;
        match event {
            BOMChangeEvent::NameChanged(name) => {
                self.name = name.clone();
            }
            BOMChangeEvent::DescriptionChanged(description) => {
                self.description = Some(description.clone());
            }
            BOMChangeEvent::ComponentAdded(component, qty) => {
                self.components.push((component.clone(), *qty));
            }
            BOMChangeEvent::ComponentRemoved(component) => {
                self.components.retain(|(c, _)| c.id != component.id);
            }
            BOMChangeEvent::ComponentUpdated(id, qty) => {
                self.components.iter_mut().for_each(|(c, q)| {
                    if c.id == *id {
                        *q = *qty;
                    }
                });
            }
        }
        Ok(())
    }

    pub fn increment_version(&mut self) {
        self.version += 1;
    }
}

#[cfg(test)]
mod tests {
    use mockall::{mock, predicate::*};

    use super::*;
    use crate::domain::{BOMChangeEvent, Component, Price};

    mock! {
        pub BOMChangeEventValidator {}
        impl Validator<BOMChangeEvent> for BOMChangeEventValidator {
            fn validate(&self, event: &BOMChangeEvent) -> Result<(), ValidationError>;
        }
    }

    fn create_test_component() -> Component {
        Component {
            id: Uuid::new_v4(),
            name: "Test Component".to_string(),
            description: Some("Test Description".to_string()),
            part_number: "123456".to_string(),
            supplier: "Test Supplier".to_string(),
            price: Price {
                value: 10.0,
                currency: "USD".to_string(),
            },
        }
    }

    fn setup_test_bom() -> BOM {
        let mut bom = BOM::default();
        bom.name = "Test BOM".to_string();
        bom.version = 1;
        bom.description = Some("Test Description".to_string());
        bom.components.push((create_test_component(), 1));
        bom
    }

    #[test]
    fn test_apply_change_name_changed() {
        let mut bom = setup_test_bom();
        let mut mock_validator = MockBOMChangeEventValidator::new();

        mock_validator
            .expect_validate()
            .times(1)
            .returning(|_| Ok(()));

        let event = BOMChangeEvent::NameChanged("New Name".to_string());
        let _ = bom.apply_change(&event, mock_validator);

        assert_eq!(bom.name, "New Name");
    }

    #[test]
    fn test_apply_change_name_changed_with_invalid_name() {
        let mut bom = setup_test_bom();
        let mut mock_validator = MockBOMChangeEventValidator::new();

        mock_validator
            .expect_validate()
            .times(1)
            .returning(|_| Err(ValidationError("Invalid name input".to_string())));

        let event = BOMChangeEvent::NameChanged("".to_string());
        let _ = bom.apply_change(&event, mock_validator);

        assert_eq!(bom.name, "Test BOM");
    }

    #[test]
    fn test_apply_change_description_changed() {
        let mut bom = setup_test_bom();
        let mut mock_validator = MockBOMChangeEventValidator::new();

        mock_validator
            .expect_validate()
            .times(1)
            .returning(|_| Ok(()));

        let event = BOMChangeEvent::DescriptionChanged("New Description".to_string());
        let _ = bom.apply_change(&event, mock_validator);

        assert_eq!(bom.description, Some("New Description".to_string()));
    }

    #[test]
    fn test_apply_change_description_changed_with_invalid_description() {
        let mut bom = setup_test_bom();
        let mut mock_validator = MockBOMChangeEventValidator::new();

        mock_validator
            .expect_validate()
            .times(1)
            .returning(|_| Err(ValidationError("Invalid description input".to_string())));

        let event = BOMChangeEvent::DescriptionChanged("New Description".to_string());
        let _ = bom.apply_change(&event, mock_validator);

        assert_eq!(bom.description, Some("Test Description".to_string()));
    }

    #[test]
    fn test_apply_change_component_added() {
        let mut bom = setup_test_bom();
        let mut mock_validator = MockBOMChangeEventValidator::new();

        mock_validator
            .expect_validate()
            .times(1)
            .returning(|_| Ok(()));

        let component = create_test_component();
        let event = BOMChangeEvent::ComponentAdded(component.clone(), 1);
        let _ = bom.apply_change(&event, mock_validator);

        assert_eq!(bom.components.len(), 2);
        assert_eq!(bom.components[1].0, component);
        assert_eq!(bom.components[1].1, 1);
    }

    #[test]
    fn test_apply_change_component_added_with_invalid_quantity() {
        let mut bom = setup_test_bom();
        let mut mock_validator = MockBOMChangeEventValidator::new();

        mock_validator
            .expect_validate()
            .times(1)
            .returning(|_| Err(ValidationError("Invalid quantity input".to_string())));

        let component = create_test_component();
        let event = BOMChangeEvent::ComponentAdded(component.clone(), 0);
        let _ = bom.apply_change(&event, mock_validator);

        assert_eq!(bom.components.len(), 1);
    }

    #[test]
    fn test_apply_change_component_removed() {
        let mut bom = setup_test_bom();
        let mut mock_validator = MockBOMChangeEventValidator::new();

        let component = bom.components[0].0.clone();

        mock_validator
            .expect_validate()
            .times(1)
            .returning(|_| Ok(()));

        let event = BOMChangeEvent::ComponentRemoved(component.clone());
        let _ = bom.apply_change(&event, mock_validator);

        assert!(bom.components.is_empty());
    }

    #[test]
    fn test_apply_change_component_updated() {
        let mut bom = setup_test_bom();
        let mut mock_validator = MockBOMChangeEventValidator::new();

        let component = bom.components[0].0.clone();

        mock_validator
            .expect_validate()
            .times(1)
            .returning(|_| Ok(()));

        let event = BOMChangeEvent::ComponentUpdated(component.id, 2);
        let _ = bom.apply_change(&event, mock_validator);

        assert_eq!(bom.components[0].1, 2);
    }

    #[test]
    fn test_apply_change_component_updated_with_invalid_quantity() {
        let mut bom = setup_test_bom();
        let mut mock_validator = MockBOMChangeEventValidator::new();

        let component = bom.components[0].0.clone();

        mock_validator
            .expect_validate()
            .times(1)
            .returning(|_| Err(ValidationError("Invalid quantity input".to_string())));

        let event = BOMChangeEvent::ComponentUpdated(component.id, 0);
        let _ = bom.apply_change(&event, mock_validator);

        assert_eq!(bom.components[0].1, 1);
    }

    #[test]
    fn test_try_from_bom() {
        let component = create_test_component();
        let events = vec![
            BOMChangeEvent::NameChanged("Test BOM".to_string()),
            BOMChangeEvent::DescriptionChanged("Test Description".to_string()),
            BOMChangeEvent::ComponentAdded(component.clone(), 1),
        ];

        let bom = BOM::try_from(&events).unwrap();

        assert_eq!(bom.name, "Test BOM");
        assert_eq!(bom.description, Some("Test Description".to_string()));
        assert_eq!(bom.components.len(), 1);
        assert_eq!(bom.components[0].0, component);
        assert_eq!(bom.components[0].1, 1);
    }

    #[test]
    fn test_try_from_bom_with_empty_events() {
        let events = vec![];

        let result = BOM::try_from(&events);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().0,
            "Unable to construct BOM without input"
        );
    }

    #[test]
    fn test_try_from_bom_without_name_changed_event() {
        let component = create_test_component();
        let events = vec![
            BOMChangeEvent::DescriptionChanged("Test Description".to_string()),
            BOMChangeEvent::ComponentAdded(component.clone(), 1),
        ];

        let result = BOM::try_from(&events);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().0,
            "Creating a BOM without name is not allowed. Please provide a name"
        );
    }

    #[test]
    fn test_try_from_bom_with_many_components() {
        let components: Vec<(Component, i32)> =
            (1..=5).map(|i| (create_test_component(), i)).collect();

        let events = vec![
            BOMChangeEvent::NameChanged("Test BOM".to_string()),
            BOMChangeEvent::DescriptionChanged("Test Description".to_string()),
        ]
        .into_iter()
        .chain(
            components
                .iter()
                .map(|(c, q)| BOMChangeEvent::ComponentAdded(c.clone(), *q)),
        )
        .collect();

        let bom = BOM::try_from(&events).unwrap();

        assert_eq!(bom.components.len(), 5);
        for (i, (component, qty)) in bom.components.iter().enumerate() {
            assert_eq!(qty, &(i as i32 + 1));
            assert_eq!(component, &components[i].0);
        }
    }
}
