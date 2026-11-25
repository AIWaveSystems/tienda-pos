use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("{0}")]
    Connection(String),
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Conflict(String),
    #[error("No hay existencias suficientes de '{0}'")]
    Stock(String),
    #[error("La venta ya fue anulada")]
    AlreadyVoided,
    #[error("Los datos almacenados no son validos: {0}")]
    Data(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

impl From<std::io::Error> for StorageError {
    fn from(failure: std::io::Error) -> Self {
        Self::Connection(failure.to_string())
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(failure: serde_json::Error) -> Self {
        Self::Data(failure.to_string())
    }
}

impl From<pos_config::ConfigError> for StorageError {
    fn from(failure: pos_config::ConfigError) -> Self {
        Self::Config(failure.to_string())
    }
}
