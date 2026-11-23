use std::sync::Arc;

use pos_config::{AppConfig, ConfigStore, CredentialStore};
use pos_domain::Permission;
use pos_services::print::Document;
use pos_services::Services;

use crate::components::notice::Notice;
use crate::components::print_dialog::PrintDialog;

pub struct Env {
    pub services: Arc<Services>,
    pub config: AppConfig,
    pub store: ConfigStore,
    pub credentials: CredentialStore,
    pub notices: Vec<Notice>,
    pub printing: Option<PrintDialog>,
}

impl Env {
    pub fn can(&self, permission: Permission) -> bool {
        self.services.auth.can(permission)
    }

    pub fn info(&mut self, title: &str, text: impl Into<String>) {
        self.notices.push(Notice::info(title, text));
    }

    pub fn warn(&mut self, title: &str, text: impl Into<String>) {
        self.notices.push(Notice::warning(title, text));
    }

    pub fn print(&mut self, document: Document, detail: impl Into<String>) {
        self.printing = Some(PrintDialog::new(document, detail));
    }

    pub fn check<T, E: ToString>(&mut self, outcome: Result<T, E>) -> Option<T> {
        match outcome {
            Ok(value) => Some(value),
            Err(failure) => {
                self.warn("No se pudo completar", failure.to_string());
                None
            }
        }
    }
}
