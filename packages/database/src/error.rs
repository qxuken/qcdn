use thiserror::Error;

#[derive(Error, Debug)]
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
}

impl DatabaseError {
    pub fn as_err<S>(self) -> Result<S, Self> {
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
