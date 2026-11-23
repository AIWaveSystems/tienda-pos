use std::sync::Arc;

use pos_config::{AppConfig, BackendKind, CredentialStore, Credentials, Paths};

use crate::cloud::{app_for, DriveEndpoints, Dropbox, DropboxEndpoints, GoogleDrive};
use crate::mongo::{MongoConnection, MongoStore};
use crate::sql::{SqlConnection, SqlStore};
use crate::{DocumentStore, MirroredStore, Result, StorageError, Store};

pub struct StoreFactory {
    paths: Paths,
    credentials: CredentialStore,
}

impl StoreFactory {
    pub fn new(paths: Paths, credentials: CredentialStore) -> Self {
        Self { paths, credentials }
    }

    pub fn build(&self, config: &AppConfig) -> Result<Arc<dyn Store>> {
        let backend = config.storage.backend;
        if backend == BackendKind::LocalJson {
            return self.local();
        }
        let primary = self.primary(config)?;
        let mirror = DocumentStore::local(&self.paths.data_dir)?;
        Ok(Arc::new(MirroredStore::new(primary, mirror)))
    }

    pub fn local(&self) -> Result<Arc<dyn Store>> {
        Ok(Arc::new(DocumentStore::local(&self.paths.data_dir)?))
    }

    pub fn primary(&self, config: &AppConfig) -> Result<Arc<dyn Store>> {
        if config.storage.backend == BackendKind::LocalJson {
            return self.local();
        }
        let credentials = self.credentials.load(config.storage.backend)?;
        Self::connect(config, &credentials)
    }

    pub fn connect(config: &AppConfig, credentials: &Credentials) -> Result<Arc<dyn Store>> {
        let storage = &config.storage;
        let password = &credentials.password;
        Ok(match storage.backend {
            BackendKind::LocalJson => {
                return Err(StorageError::Config(
                    "El almacenamiento local no necesita conexion.".to_owned(),
                ))
            }
            BackendKind::Mysql => Arc::new(SqlStore::mysql(&SqlConnection::new(
                &storage.mysql,
                password,
            ))?),
            BackendKind::Postgresql => Arc::new(SqlStore::postgres(&SqlConnection::new(
                &storage.postgresql,
                password,
            ))?),
            BackendKind::Mongodb => Arc::new(MongoStore::connect(&MongoConnection::new(
                &storage.mongodb,
                password,
            ))?),
            BackendKind::GoogleDrive => {
                let token = refresh_token(BackendKind::GoogleDrive, credentials)?;
                let app = app_for(BackendKind::GoogleDrive)?;
                Arc::new(DocumentStore::new(Arc::new(GoogleDrive::connect(
                    app,
                    token,
                    DriveEndpoints::default(),
                )?)))
            }
            BackendKind::Dropbox => {
                let token = refresh_token(BackendKind::Dropbox, credentials)?;
                let app = app_for(BackendKind::Dropbox)?;
                Arc::new(DocumentStore::new(Arc::new(Dropbox::connect(
                    app,
                    token,
                    DropboxEndpoints::default(),
                )?)))
            }
        })
    }
}

fn refresh_token(backend: BackendKind, credentials: &Credentials) -> Result<&str> {
    if credentials.refresh_token.is_empty() {
        return Err(StorageError::Config(format!(
            "No hay ninguna cuenta de {} conectada. \
             Conectese desde Configuracion > Almacenamiento.",
            backend.label()
        )));
    }
    Ok(&credentials.refresh_token)
}
