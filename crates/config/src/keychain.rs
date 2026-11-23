use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::OsRng;
use base64::prelude::{Engine, BASE64_STANDARD};
use keyring::{Entry, Error};

use crate::secret::KeyProvider;
use crate::ConfigError;

const SERVICE: &str = "VentasPOSRust";
const ACCOUNT: &str = "storage-encryption-key";

pub struct SystemKeychain;

impl KeyProvider for SystemKeychain {
    fn key(&self) -> Result<[u8; 32], ConfigError> {
        let entry = Entry::new(SERVICE, ACCOUNT).map_err(unavailable)?;
        match entry.get_password() {
            Ok(text) => decode(&text),
            Err(Error::NoEntry) => create(&entry),
            Err(failure) => Err(unavailable(failure)),
        }
    }
}

fn create(entry: &Entry) -> Result<[u8; 32], ConfigError> {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    entry
        .set_password(&BASE64_STANDARD.encode(key))
        .map_err(unavailable)?;
    Ok(key)
}

fn decode(text: &str) -> Result<[u8; 32], ConfigError> {
    BASE64_STANDARD
        .decode(text)
        .ok()
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| {
            ConfigError::Credentials(
                "La clave de cifrado guardada en el sistema es invalida.".into(),
            )
        })
}

fn unavailable(failure: Error) -> ConfigError {
    ConfigError::Credentials(format!(
        "No se pudo acceder al almacen de credenciales del sistema operativo: {failure}"
    ))
}
