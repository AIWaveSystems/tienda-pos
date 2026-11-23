use std::sync::{Arc, Mutex, PoisonError};

use pos_config::{AppConfig, ConfigStore, CredentialStore};
use pos_services::Services;
use pos_storage::{Store, StoreFactory};

#[derive(Clone, Default)]
pub struct Progress(Arc<Mutex<(String, f32)>>);

impl Progress {
    pub fn set(&self, message: impl Into<String>, fraction: f32) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = (message.into(), fraction);
    }

    pub fn get(&self) -> (String, f32) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

pub struct Booted {
    pub services: Arc<Services>,
    pub config: AppConfig,
    pub warning: Option<String>,
}

pub fn boot(
    store: &ConfigStore,
    credentials: &CredentialStore,
    progress: &Progress,
) -> Result<Booted, String> {
    progress.set("Cargando configuracion...", 0.15);
    let config = store.load().map_err(|failure| failure.to_string())?;
    let label = config.storage.backend.label();
    progress.set(format!("Conectando a {label}..."), 0.35);
    let factory = StoreFactory::new(store.paths().clone(), credentials.clone());
    let (data, warning) = match open(&factory, &config) {
        Ok(data) => (data, None),
        Err(failure) => {
            let local = factory.local().map_err(|error| error.to_string())?;
            let text = format!(
                "No se pudo conectar con el almacenamiento configurado ({label}):\n{failure}\n\n\
                 Se usara el almacenamiento local (JSON) para esta sesion. Puede corregir la \
                 configuracion en Configuracion > Almacenamiento."
            );
            (local, Some(text))
        }
    };
    progress.set("Iniciando servicios...", 0.60);
    let services = Services::new(data);
    progress.set("Verificando usuarios...", 0.85);
    services
        .auth
        .ensure_default_user()
        .map_err(|failure| failure.to_string())?;
    progress.set("Listo", 1.0);
    Ok(Booted {
        services: Arc::new(services),
        config,
        warning,
    })
}

fn open(factory: &StoreFactory, config: &AppConfig) -> pos_storage::Result<Arc<dyn Store>> {
    let data = factory.build(config)?;
    data.ping()?;
    Ok(data)
}
