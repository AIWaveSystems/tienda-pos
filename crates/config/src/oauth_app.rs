use std::collections::HashMap;
use std::env;

use crate::{paths, BackendKind};

#[derive(Clone)]
pub struct OAuthApp {
    pub client_id: String,
    pub client_secret: String,
}

impl OAuthApp {
    pub fn load(backend: BackendKind) -> Option<Self> {
        let prefix = match backend {
            BackendKind::GoogleDrive => "GOOGLE_DRIVE",
            BackendKind::Dropbox => "DROPBOX",
            _ => return None,
        };
        let file = env_file();
        Some(Self {
            client_id: value(&file, &format!("{prefix}_CLIENT_ID"))?,
            client_secret: value(&file, &format!("{prefix}_CLIENT_SECRET"))?,
        })
    }
}

fn env_file() -> HashMap<String, String> {
    let beside_app = paths::app_dir().join(".env");
    let path = if beside_app.exists() {
        beside_app
    } else {
        ".env".into()
    };
    dotenvy::from_path_iter(path)
        .map(|entries| entries.filter_map(Result::ok).collect())
        .unwrap_or_default()
}

fn value(file: &HashMap<String, String>, name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .or_else(|| file.get(name).cloned())
        .filter(|content| !content.is_empty())
}
