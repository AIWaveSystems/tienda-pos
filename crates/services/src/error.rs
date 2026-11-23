use pos_storage::StorageError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("{0}")]
    Rule(String),
    #[error(transparent)]
    Storage(#[from] StorageError),
}

pub type Result<T> = std::result::Result<T, ServiceError>;

pub(crate) fn rule<T>(message: impl Into<String>) -> Result<T> {
    Err(ServiceError::Rule(message.into()))
}
