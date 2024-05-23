use super::aliases::DieselError;
use r2d2;

#[derive(Debug)]
pub enum DatabaseError {
    DieselError(diesel::result::Error),
    R2D2Error(r2d2::Error),
}

impl From<DieselError> for DatabaseError {
    fn from(error: DieselError) -> Self {
        Self::DieselError(error)
    }
}

impl From<r2d2::Error> for DatabaseError {
    fn from(error: r2d2::Error) -> Self {
        Self::R2D2Error(error)
    }
}
