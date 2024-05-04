use std::fmt::Display;

use unicode_segmentation::UnicodeSegmentation;

use super::BOMChangeEvent;

#[derive(thiserror::Error, Debug, PartialEq)]
pub struct ValidationError(pub String);

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait Validator<T> {
    fn validate(&self, data: &T) -> Result<(), ValidationError>;
}

pub struct BOMChangeEventValidator;

impl Validator<BOMChangeEvent> for BOMChangeEventValidator {
    fn validate(&self, event: &BOMChangeEvent) -> Result<(), ValidationError> {
        match event {
            BOMChangeEvent::NameChanged(name) => {
                if is_valid_string(name) {
                    Ok(())
                } else {
                    Err(ValidationError("Invalid name input".to_string()))
                }
            }
            BOMChangeEvent::DescriptionChanged(description) => {
                if is_valid_string(description) {
                    Ok(())
                } else {
                    Err(ValidationError("Invalid description input".to_string()))
                }
            }
            BOMChangeEvent::ComponentAdded(_, qty) => {
                if *qty > 0 {
                    Ok(())
                } else {
                    Err(ValidationError(
                        "Quantity must be greater than 0".to_string(),
                    ))
                }
            }
            BOMChangeEvent::ComponentUpdated(_, qty) => {
                if *qty > 0 {
                    Ok(())
                } else {
                    Err(ValidationError(
                        "Quantity must be greater than 0".to_string(),
                    ))
                }
            }
            BOMChangeEvent::ComponentRemoved(_) => Ok(()),
        }
    }
}

fn is_valid_string(s: &str) -> bool {
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

    !s.trim().is_empty() && s.graphemes(true).count() < 255 && !contains_forbidden_characters
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::domain::Component;

    use super::*;

    fn create_test_component() -> Component {
        Component {
            id: Uuid::new_v4(),
            name: "Test Component".to_string(),
            description: Some("Test Description".to_string()),
            part_number: "123456".to_string(),
            supplier: "Test Supplier".to_string(),
            price: crate::domain::Price {
                value: 10.0,
                currency: "USD".to_string(),
            },
        }
    }

    #[test]
    fn test_is_valid_string() {
        assert!(is_valid_string("valid string"));
        assert!(!is_valid_string("invalid string/"));
        assert!(!is_valid_string("invalid string("));
        assert!(!is_valid_string("invalid string)"));
        assert!(!is_valid_string("invalid string\""));
        assert!(!is_valid_string("invalid string<"));
        assert!(!is_valid_string("invalid string>"));
        assert!(!is_valid_string("invalid string\\"));
        assert!(!is_valid_string("invalid string{"));
        assert!(!is_valid_string("invalid string}"));
        assert!(!is_valid_string("iiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiii"));
        assert!(!is_valid_string(" "));
        assert!(!is_valid_string(""));
    }

    #[test]
    fn test_validate_name() {
        let validator = BOMChangeEventValidator;
        let name = "valid name".to_string();
        let invalid_name = "invalid name/".to_string();

        assert_eq!(
            validator.validate(&BOMChangeEvent::NameChanged(name)),
            Ok(())
        );
        assert_eq!(
            validator.validate(&BOMChangeEvent::NameChanged(invalid_name)),
            Err(ValidationError("Invalid name input".to_string()))
        );
    }

    #[test]
    fn test_validate_description() {
        let validator = BOMChangeEventValidator;
        let description = "valid description".to_string();
        let invalid_description = "invalid description/".to_string();

        assert_eq!(
            validator.validate(&BOMChangeEvent::DescriptionChanged(description)),
            Ok(())
        );
        assert_eq!(
            validator.validate(&BOMChangeEvent::DescriptionChanged(invalid_description)),
            Err(ValidationError("Invalid description input".to_string()))
        );
    }

    #[test]
    fn test_validate_component_added() {
        let validator = BOMChangeEventValidator;
        let qty = 1;

        let test_component = create_test_component();

        assert_eq!(
            validator.validate(&BOMChangeEvent::ComponentAdded(test_component, qty)),
            Ok(())
        );
    }

    #[test]
    fn test_validate_component_with_invalid_quantity() {
        let validator = BOMChangeEventValidator;
        let qty = 0;

        let test_component: Component = create_test_component();

        assert_eq!(
            validator.validate(&BOMChangeEvent::ComponentAdded(test_component, qty)),
            Err(ValidationError(
                "Quantity must be greater than 0".to_string()
            ))
        );
    }

    #[test]
    fn test_validate_component_updated() {
        let validator = BOMChangeEventValidator;
        let qty = 1;

        assert_eq!(
            validator.validate(&BOMChangeEvent::ComponentUpdated(Uuid::new_v4(), qty)),
            Ok(())
        );
    }

    #[test]
    fn test_validate_component_updated_with_invalid_quantity() {
        let validator = BOMChangeEventValidator;
        let qty = 0;

        assert_eq!(
            validator.validate(&BOMChangeEvent::ComponentUpdated(Uuid::new_v4(), qty)),
            Err(ValidationError(
                "Quantity must be greater than 0".to_string()
            ))
        );
    }

    #[test]
    fn test_validate_component_removed() {
        let validator = BOMChangeEventValidator;
        let test_component = create_test_component();

        assert_eq!(
            validator.validate(&BOMChangeEvent::ComponentRemoved(test_component)),
            Ok(())
        );
    }
}
