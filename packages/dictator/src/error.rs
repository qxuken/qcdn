use qcdn_database::DatabaseError;
use std::error::Error;
use thiserror::Error;
use tonic::Status;

pub type Result<T, E = Report> = color_eyre::Result<T, E>;
pub struct Report(color_eyre::Report);

impl std::fmt::Debug for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> From<E> for Report
where
    E: Into<color_eyre::Report>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl From<Report> for Status {
    fn from(value: Report) -> Self {
        let err = value.0;
        let err_string = format!("{err:?}");

        tracing::error!("{err_string}");

        if let Some(err) = err.downcast_ref::<AppError>() {
            return err.into();
        }

        Self::internal("Internal server error")
    }
}

#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("Not found")]
    NotFound,
    #[error("Requirements is not met: {0}")]
    PreconditionError(String),
    #[error("Data corruption: {0}")]
    DataCorruption(String),
    #[error("Internal server error {0}")]
    Other(String),
}

impl From<AppError> for Status {
    fn from(value: AppError) -> Self {
        (&value).into()
    }
}

impl From<&AppError> for Status {
    fn from(value: &AppError) -> Self {
        match value {
            AppError::NotFound => Self::not_found(value.to_string()),
            AppError::PreconditionError(_) => Self::failed_precondition(value.to_string()),
            AppError::DataCorruption(e) => Self::data_loss(e.to_string()),
            _ => Self::internal(value.to_string()),
        }
    }
}

impl From<DatabaseError> for AppError {
    fn from(value: DatabaseError) -> Self {
        match value {
            DatabaseError::NotFound(_e) => Self::NotFound,
            DatabaseError::PreconditionError(e) => Self::PreconditionError(e),
            _ => Self::Other(value.to_string()),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::Other(value.to_string())
    }
}

impl From<Box<dyn Error + Send + Sync>> for AppError {
    fn from(value: Box<dyn Error + Send + Sync>) -> Self {
        Self::Other(value.to_string())
    }
}
