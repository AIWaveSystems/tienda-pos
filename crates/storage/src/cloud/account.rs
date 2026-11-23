use pos_config::{BackendKind, OAuthApp};

use super::dropbox::{Dropbox, DropboxEndpoints};
use super::google_drive::{DriveEndpoints, GoogleDrive};
use crate::{Result, StorageError};

pub struct CloudAccount {
    pub refresh_token: String,
    pub email: String,
}

impl CloudAccount {
    pub fn authorize(backend: BackendKind) -> Result<Self> {
        let app = app_for(backend)?;
        match backend {
            BackendKind::GoogleDrive => GoogleDrive::authorize(&app, &DriveEndpoints::default()),
            _ => Dropbox::authorize(&app, &DropboxEndpoints::default()),
        }
    }
}

pub fn app_for(backend: BackendKind) -> Result<OAuthApp> {
    OAuthApp::load(backend).ok_or_else(|| {
        StorageError::Config(format!(
            "Faltan las credenciales fijas de {} en el archivo .env.",
            backend.label()
        ))
    })
}
