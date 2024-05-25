use actix_web::{error::BlockingError, ResponseError};
use diesel::result::Error as DieselError;

use crate::{
    domain::error::DomainError, infrastructure::error::DatabaseError, services::error::ServiceError,
};

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Internal Server Error: {0}")]
    Unexpected(String),
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Not Found: {0}")]
    NotFound(String),
}

impl From<BlockingError> for ApiError {
    fn from(_: BlockingError) -> Self {
        Self::Unexpected("Blocking operation was cancelled".to_string())
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ApiError::Unexpected(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BadRequest(_) => actix_web::http::StatusCode::BAD_REQUEST,
            ApiError::NotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        actix_web::HttpResponse::build(self.status_code())
            .json(serde_json::json!({ "error": self.to_string() }))
    }
}

impl From<ServiceError> for ApiError {
    fn from(value: ServiceError) -> Self {
        match value {
            ServiceError::DomainError(error) => match error {
                DomainError::ValidationError(message) => Self::BadRequest(message),
                DomainError::ConversionError(message) => Self::Unexpected(message),
            },
            ServiceError::DatabaseError(error) => match error {
                DatabaseError::DieselError(error) => match error {
                    DieselError::NotFound => Self::NotFound("Resource not found".into()),
                    _ => Self::Unexpected(error.to_string()),
                },
                DatabaseError::R2D2Error(error) => Self::Unexpected(error.to_string()),
            },
            ServiceError::InvalidData(message) => Self::BadRequest(message),
        }
    }
}
