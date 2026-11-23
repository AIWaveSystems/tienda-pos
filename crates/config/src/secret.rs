use crate::ConfigError;

pub trait KeyProvider: Send + Sync {
    fn key(&self) -> Result<[u8; 32], ConfigError>;
}
