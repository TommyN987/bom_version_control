use crate::{domain::error::DomainError, infrastructure::error::DatabaseError};

#[derive(Debug)]
pub enum ServiceError {
    DomainError(DomainError),
    DatabaseError(DatabaseError),
    InvalidData(String),
}

impl From<DomainError> for ServiceError {
    fn from(error: DomainError) -> Self {
        Self::DomainError(error)
    }
}

impl From<DatabaseError> for ServiceError {
    fn from(error: DatabaseError) -> Self {
        Self::DatabaseError(error)
    }
}
