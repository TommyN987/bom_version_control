use actix_web::{error::BlockingError, ResponseError};
use diesel::{r2d2::Error as R2D2Error, result::Error as DieselError};

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Not Found: {0}")]
    NotFound(String),
}

impl ResponseError for ApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ApiError::Unexpected(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BadRequest(_) => actix_web::http::StatusCode::BAD_REQUEST,
            ApiError::NotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
        }
    }
}

impl From<DieselError> for ApiError {
    fn from(value: DieselError) -> Self {
        match value {
            DieselError::NotFound => Self::NotFound("Resource not found".into()),
            _ => Self::Unexpected(value.into()),
        }
    }
}

impl From<R2D2Error> for ApiError {
    fn from(value: R2D2Error) -> Self {
        Self::Unexpected(value.into())
    }
}

impl From<BlockingError> for ApiError {
    fn from(value: BlockingError) -> Self {
        Self::Unexpected(value.into())
    }
}
