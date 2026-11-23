use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use aes_gcm::aead::{Aead, AeadCore, OsRng};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::prelude::{Engine, BASE64_STANDARD};
use serde::{Deserialize, Serialize};

use crate::keychain::SystemKeychain;
use crate::secret::KeyProvider;
use crate::{files, BackendKind, ConfigError, Paths};

const NONCE_LENGTH: usize = 12;

#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Credentials {
    pub password: String,
    pub refresh_token: String,
    pub account_email: String,
}

#[derive(Clone)]
pub struct CredentialStore {
    path: PathBuf,
    keys: Arc<dyn KeyProvider>,
}

impl CredentialStore {
    pub fn new(paths: &Paths) -> Self {
        Self::with_keys(paths, Arc::new(SystemKeychain))
    }

    pub fn with_keys(paths: &Paths, keys: Arc<dyn KeyProvider>) -> Self {
        Self {
            path: paths.credentials_file(),
            keys,
        }
    }

    pub fn save(&self, backend: BackendKind, data: &Credentials) -> Result<(), ConfigError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let sealed = self
            .cipher()?
            .encrypt(&nonce, serde_json::to_vec(data)?.as_slice())
            .map_err(|_| damaged(backend))?;
        let mut packed = nonce.to_vec();
        packed.extend(sealed);
        let mut entries = self.read()?;
        entries.insert(backend.key().to_owned(), BASE64_STANDARD.encode(packed));
        self.write(&entries)
    }

    pub fn load(&self, backend: BackendKind) -> Result<Credentials, ConfigError> {
        let entries = self.read()?;
        let Some(token) = entries.get(backend.key()) else {
            return Ok(Credentials::default());
        };
        let packed = BASE64_STANDARD
            .decode(token)
            .map_err(|_| damaged(backend))?;
        if packed.len() <= NONCE_LENGTH {
            return Err(damaged(backend));
        }
        let (nonce, sealed) = packed.split_at(NONCE_LENGTH);
        let plain = self
            .cipher()?
            .decrypt(Nonce::from_slice(nonce), sealed)
            .map_err(|_| damaged(backend))?;
        Ok(serde_json::from_slice(&plain)?)
    }

    pub fn clear(&self, backend: BackendKind) -> Result<(), ConfigError> {
        let mut entries = self.read()?;
        if entries.remove(backend.key()).is_some() {
            self.write(&entries)?;
        }
        Ok(())
    }

    fn cipher(&self) -> Result<Aes256Gcm, ConfigError> {
        Ok(Aes256Gcm::new(&self.keys.key()?.into()))
    }

    fn read(&self) -> Result<BTreeMap<String, String>, ConfigError> {
        if !self.path.exists() {
            return Ok(BTreeMap::new());
        }
        Ok(serde_json::from_slice(&fs::read(&self.path)?)?)
    }

    fn write(&self, entries: &BTreeMap<String, String>) -> Result<(), ConfigError> {
        files::write_atomic(&self.path, &serde_json::to_vec_pretty(entries)?)?;
        Ok(())
    }
}

fn damaged(backend: BackendKind) -> ConfigError {
    ConfigError::Credentials(format!(
        "No se pudieron descifrar las credenciales de {}.",
        backend.label()
    ))
}
