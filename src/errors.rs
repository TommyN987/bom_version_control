use actix_web::{error::BlockingError, ResponseError};
use derive_more::Display;
use diesel::{r2d2::Error as R2D2Error, result::Error as DieselError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, PartialEq, Serialize, Deserialize)]
#[display(fmt = "ApiError: {}", detail)]
pub struct ApiError {
    pub status_code: u16,
    pub detail: String,
}

impl ApiError {
    pub fn new_internal(detail: String) -> Self {
        Self {
            status_code: 500,
            detail: format!("Internal server error: {}", detail),
        }
    }
}

impl From<DieselError> for ApiError {
    fn from(value: DieselError) -> Self {
        ApiError::new_internal(value.to_string())
    }
}

impl From<R2D2Error> for ApiError {
    fn from(value: R2D2Error) -> Self {
        ApiError::new_internal(value.to_string())
    }
}

impl From<BlockingError> for ApiError {
    fn from(value: BlockingError) -> Self {
        ApiError::new_internal(value.to_string())
    }
}

impl ResponseError for ApiError {}
