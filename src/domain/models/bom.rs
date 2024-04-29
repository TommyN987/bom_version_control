use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::domain::ValidationError;
use crate::domain::{BOMChangeEventValidator, Validator};

use super::{BOMChangeEvent, Component};

#[derive(Debug, Serialize, Deserialize)]
pub struct BOM {
    pub id: Uuid,
    pub name: String,
    pub version: i32,
    pub description: Option<String>,
    pub components: Vec<(Component, i32)>,
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

impl TryFrom<&Vec<BOMChangeEvent>> for BOM {
    type Error = ValidationError;

    fn try_from(value: &Vec<BOMChangeEvent>) -> Result<Self, Self::Error> {
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
    use super::*;
    use crate::domain::{BOMChangeEvent, Component, MockValidator, Price};

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
        let mut mock_validator = MockValidator::<BOMChangeEvent>::new();

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
        let mut mock_validator = MockValidator::<BOMChangeEvent>::new();

        mock_validator
            .expect_validate()
            .times(1)
            .returning(|_| Err(ValidationError("Invalid name input".to_string())));

        let event = BOMChangeEvent::NameChanged("".to_string());
        let result = bom.apply_change(&event, mock_validator);

        assert!(result.is_err());
        assert_eq!(bom.name, "Test BOM");
    }
}
