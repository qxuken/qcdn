use axum::{http::StatusCode, response::IntoResponse};
use qcdn_database::DatabaseError;
use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("Not found")]
    NotFound,
    #[error("Internal server error")]
    Other,
}

impl From<DatabaseError> for AppError {
    fn from(value: DatabaseError) -> Self {
        match value {
            DatabaseError::NotFound(_e) => Self::NotFound,
            _ => Self::Other,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::NotFound => (StatusCode::NOT_FOUND, self.to_string()).into_response(),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response(),
        }
    }
}

impl From<Box<dyn Error + Send + Sync>> for AppError {
    fn from(_value: Box<dyn Error + Send + Sync>) -> Self {
        Self::Other
    }
}
