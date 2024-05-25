use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{error::DomainError, validation::Validator};

use super::{BOMChangeEvent, CountedComponent};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BOM {
    pub id: Uuid,
    pub name: String,
    pub version: i32,
    pub description: Option<String>,
    pub components: Vec<CountedComponent>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for BOM {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            version: 0,
            description: None,
            components: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl BOM {
    pub fn apply_change<T: Validator<BOMChangeEvent>>(
        &mut self,
        event: &BOMChangeEvent,
        validator: T,
    ) -> Result<(), DomainError> {
        validator.validate(event)?;
        match event {
            BOMChangeEvent::NameChanged(name) => {
                self.name.clone_from(name);
            }
            BOMChangeEvent::DescriptionChanged(description) => {
                self.description = Some(description.clone());
            }
            BOMChangeEvent::ComponentAdded(component, qty) => {
                self.components
                    .push(CountedComponent::new(component.clone(), *qty));
            }
            BOMChangeEvent::ComponentRemoved(component) => {
                self.components.retain(|cc| cc.component.id != component.id);
            }
            BOMChangeEvent::ComponentUpdated(id, qty) => {
                self.components.iter_mut().for_each(|cc| {
                    if cc.component.id == *id {
                        cc.quantity = *qty;
                    }
                });
            }
        }
        Ok(())
    }

    pub fn increment_version(&mut self) {
        self.version += 1;
    }

    pub fn clean_for_revert(&mut self) {
        self.description = None;
        self.components.clear();
    }
}

#[cfg(test)]
mod tests {
    use mockall::{mock, predicate::*};

    use super::*;
    use crate::domain::{
        newtypes::new_bom::{self, NewBOM},
        BOMChangeEvent, Component, Price,
    };

    mock! {
        pub BOMChangeEventValidator {}
        impl Validator<BOMChangeEvent> for BOMChangeEventValidator {
            fn validate(&self, event: &BOMChangeEvent) -> Result<(), DomainError>;
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
        BOM {
            name: "Test BOM".to_string(),
            version: 1,
            description: Some("Test Description".to_string()),
            components: vec![CountedComponent::new(create_test_component(), 1)],
            ..Default::default()
        }
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

        mock_validator.expect_validate().times(1).returning(|_| {
            Err(DomainError::ValidationError(
                "Invalid name input".to_string(),
            ))
        });

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

        mock_validator.expect_validate().times(1).returning(|_| {
            Err(DomainError::ValidationError(
                "Invalid description input".to_string(),
            ))
        });

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
        assert_eq!(bom.components[1].component, component);
        assert_eq!(bom.components[1].quantity, 1);
    }

    #[test]
    fn test_apply_change_component_added_with_invalid_quantity() {
        let mut bom = setup_test_bom();
        let mut mock_validator = MockBOMChangeEventValidator::new();

        mock_validator.expect_validate().times(1).returning(|_| {
            Err(DomainError::ValidationError(
                "Invalid quantity input".to_string(),
            ))
        });

        let component = create_test_component();
        let event = BOMChangeEvent::ComponentAdded(component.clone(), 0);
        let _ = bom.apply_change(&event, mock_validator);

        assert_eq!(bom.components.len(), 1);
    }

    #[test]
    fn test_apply_change_component_removed() {
        let mut bom = setup_test_bom();
        let mut mock_validator = MockBOMChangeEventValidator::new();

        let component = bom.components[0].component.clone();

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

        let component = bom.components[0].component.clone();

        mock_validator
            .expect_validate()
            .times(1)
            .returning(|_| Ok(()));

        let event = BOMChangeEvent::ComponentUpdated(component.id, 2);
        let _ = bom.apply_change(&event, mock_validator);

        assert_eq!(bom.components[0].quantity, 2);
    }

    #[test]
    fn test_apply_change_component_updated_with_invalid_quantity() {
        let mut bom = setup_test_bom();
        let mut mock_validator = MockBOMChangeEventValidator::new();

        let component = bom.components[0].component.clone();

        mock_validator.expect_validate().times(1).returning(|_| {
            Err(DomainError::ValidationError(
                "Invalid quantity input".to_string(),
            ))
        });

        let event = BOMChangeEvent::ComponentUpdated(component.id, 0);
        let _ = bom.apply_change(&event, mock_validator);

        assert_eq!(bom.components[0].quantity, 1);
    }

    #[test]
    fn test_try_from_bom() {
        let component = create_test_component();
        let events = Box::new(vec![
            BOMChangeEvent::NameChanged("Test BOM".to_string()),
            BOMChangeEvent::DescriptionChanged("Test Description".to_string()),
            BOMChangeEvent::ComponentAdded(component.clone(), 1),
        ]);

        let new_bom = new_bom::NewBOM { events };

        let bom = BOM::try_from(&new_bom).unwrap();

        assert_eq!(bom.name, "Test BOM");
        assert_eq!(bom.description, Some("Test Description".to_string()));
        assert_eq!(bom.components.len(), 1);
        assert_eq!(bom.components[0].component, component);
        assert_eq!(bom.components[0].quantity, 1);
    }

    #[test]
    fn test_try_from_bom_with_empty_events() {
        let events = Box::new(vec![]);

        let new_bom = new_bom::NewBOM { events };

        let result = BOM::try_from(&new_bom);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            DomainError::ValidationError("Unable to construct BOM without input".to_string())
        );
    }

    #[test]
    fn test_try_from_bom_without_name_changed_event() {
        let component = create_test_component();
        let events = Box::new(vec![
            BOMChangeEvent::DescriptionChanged("Test Description".to_string()),
            BOMChangeEvent::ComponentAdded(component.clone(), 1),
        ]);

        let new_bom = new_bom::NewBOM { events };

        let result = BOM::try_from(&new_bom);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            DomainError::ValidationError(
                "Creating a BOM without name is not allowed. Please provide a name".to_string()
            )
        );
    }

    #[test]
    fn test_try_from_bom_with_many_components() {
        let components: Vec<(Component, i32)> =
            (1..=5).map(|i| (create_test_component(), i)).collect();

        let events: Vec<BOMChangeEvent> = vec![
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

        let new_bom = NewBOM {
            events: Box::new(events),
        };

        let bom = BOM::try_from(&new_bom).unwrap();

        assert_eq!(bom.components.len(), 5);
        for (i, counted_component) in bom.components.iter().enumerate() {
            assert_eq!(counted_component.quantity, (i as i32 + 1));
            assert_eq!(counted_component.component, components[i].0);
        }
    }
}
