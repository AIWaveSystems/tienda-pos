use std::env;
use std::path::PathBuf;

const FOLDER: &str = "VentasPOSRust";
const UNIX_FOLDER: &str = "ventas-pos-rust";
const HOME_VARIABLE: &str = "VENTAS_POS_HOME";

#[derive(Clone, Debug, PartialEq)]
pub struct Paths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl Paths {
    pub fn rooted(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self {
            config_dir: root.join("config"),
            data_dir: root.join("data"),
        }
    }

    pub fn detect() -> Self {
        if let Some(root) = env::var_os(HOME_VARIABLE).filter(|value| !value.is_empty()) {
            return Self::rooted(root);
        }
        let app = app_dir();
        if app.join("portable.txt").exists() {
            return Self::rooted(app);
        }
        Self {
            config_dir: system_dir("APPDATA", ".config"),
            data_dir: system_dir("LOCALAPPDATA", ".local/share"),
        }
    }

    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("config.json")
    }

    pub fn credentials_file(&self) -> PathBuf {
        self.config_dir.join("storage_credentials.json")
    }
}

pub fn app_dir() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(PathBuf::from))
        .unwrap_or_default()
}

fn system_dir(variable: &str, fallback: &str) -> PathBuf {
    match env::var_os(variable) {
        Some(base) => PathBuf::from(base).join(FOLDER),
        None => env::home_dir()
            .unwrap_or_default()
            .join(fallback)
            .join(UNIX_FOLDER),
    }
}
