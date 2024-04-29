use std::fmt::Display;

use mockall::automock;
use unicode_segmentation::UnicodeSegmentation;

use super::BOMChangeEvent;

#[derive(thiserror::Error, Debug)]
pub struct ValidationError(pub String);

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[automock]
pub trait Validator<T> {
    fn validate(&self, data: &T) -> Result<(), ValidationError>;
}

pub struct BOMChangeEventValidator;

impl Validator<BOMChangeEvent> for BOMChangeEventValidator {
    fn validate(&self, event: &BOMChangeEvent) -> Result<(), ValidationError> {
        match event {
            BOMChangeEvent::NameChanged(name) => {
                if self.is_valid_string(name) {
                    Ok(())
                } else {
                    Err(ValidationError("Invalid name input".to_string()))
                }
            }
            BOMChangeEvent::DescriptionChanged(description) => {
                if self.is_valid_string(description) {
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

impl BOMChangeEventValidator {
    fn is_valid_string(&self, s: &str) -> bool {
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        !s.trim().is_empty() && s.graphemes(true).count() < 255 && !contains_forbidden_characters
    }
}
