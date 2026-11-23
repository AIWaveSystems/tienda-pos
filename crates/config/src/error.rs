use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("No se pudo acceder a los archivos de configuracion: {0}")]
    Io(#[from] std::io::Error),
    #[error("La configuracion no tiene un formato valido: {0}")]
    Format(#[from] serde_json::Error),
    #[error("{0}")]
    Credentials(String),
}
