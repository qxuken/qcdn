use std::error::Error;
use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug, Clone)]
pub enum DatabaseError {
    #[error("Pool setup error: {0}")]
    PoolSetupError(String),
    #[error("Database migration error: {0}")]
    MigrationError(String),
    #[error("Pool connection error: {0}")]
    PoolConnectionError(String),
    #[error("Error during query: {0}")]
    QueryError(String),
    #[error("Record not found {0}")]
    NotFound(String),
    #[error("Requirements is not met: {0}")]
    PreconditionError(String),
    #[error("Database error: {0}")]
    Other(String),
}

impl DatabaseError {
    pub fn err<S>(self) -> Result<S, Self> {
        Err(self)
    }
}

impl From<DatabaseError> for Status {
    fn from(value: DatabaseError) -> Self {
        match value {
            DatabaseError::NotFound(_) => Self::not_found(value.to_string()),
            DatabaseError::PreconditionError(_) => Self::failed_precondition(value.to_string()),
            _ => Self::internal(value.to_string()),
        }
    }
}

impl From<sqlx::Error> for DatabaseError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NotFound("".to_string()),
            v => Self::QueryError(v.to_string()),
        }
    }
}

impl From<Box<dyn Error + Send + Sync>> for DatabaseError {
    fn from(value: Box<dyn Error + Send + Sync>) -> Self {
        Self::Other(value.to_string())
    }
}
