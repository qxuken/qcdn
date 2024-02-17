use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DatabaseError {
    #[error("Pool setup error: {0}")]
    PoolSetupError(String),
    #[error("Pool connection error: {0}")]
    PoolConnectionError(String),
    #[error("Error during query: {0}")]
    QueryError(String),
    #[error("Record not found {0}")]
    NotFound(String),
    #[error("Requirements is not set: {0}")]
    PreconditionError(String),
    #[error("Database error: {0}")]
    Other(String),
}

impl DatabaseError {
    pub fn err<S>(self) -> Result<S, Self> {
        Err(self)
    }
}

impl From<diesel::result::Error> for DatabaseError {
    fn from(value: diesel::result::Error) -> Self {
        match value {
            diesel::result::Error::NotFound => Self::NotFound("".to_string()),
            v => Self::QueryError(v.to_string()),
        }
    }
}

impl From<Box<dyn Error + Send + Sync>> for DatabaseError {
    fn from(value: Box<dyn Error + Send + Sync>) -> Self {
        Self::Other(value.to_string())
    }
}
