#[derive(Debug, PartialEq)]
pub enum DomainError {
    ConversionError(String),
    ValidationError(String),
}
