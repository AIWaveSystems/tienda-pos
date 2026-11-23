use std::fs;
use std::sync::Arc;

use pos_config::{
    AppConfig, BackendKind, ConfigError, ConfigStore, CredentialStore, Credentials, KeyProvider,
    Paths, PrinterFormat,
};
use rust_decimal::Decimal;
use serde_json::{json, Value};

struct FixedKey(u8);

impl KeyProvider for FixedKey {
    fn key(&self) -> Result<[u8; 32], ConfigError> {
        Ok([self.0; 32])
    }
}

fn workspace() -> (tempfile::TempDir, Paths) {
    let folder = tempfile::tempdir().unwrap();
    let paths = Paths::rooted(folder.path());
    fs::create_dir_all(&paths.config_dir).unwrap();
    (folder, paths)
}

#[test]
fn defaults_match_the_original() {
    let config = AppConfig::default();
    assert_eq!(config.general.tax_rate, Decimal::new(19, 2));
    assert_eq!(config.general.currency, "COP");
    assert_eq!(config.storage.postgresql.port, 5432);
    assert_eq!(config.sales.printer_format, PrinterFormat::A4);
}

#[test]
fn invalid_values_are_sanitized_and_unknown_keys_survive() {
    let (_folder, paths) = workspace();
    let original = json!({
        "general": {"tax_rate": 7, "currency": "", "future": 1},
        "ui": {"window_width": "nada"},
        "storage": {"backend": "otro"},
        "plugin": {"a": true}
    });
    fs::write(paths.config_file(), original.to_string()).unwrap();
    let store = ConfigStore::new(paths.clone());
    let config = store.load().unwrap();
    assert_eq!(config.general.tax_rate, Decimal::ONE);
    assert_eq!(config.general.currency, "COP");
    assert_eq!(config.ui.window_width, 1000);
    assert_eq!(config.storage.backend, BackendKind::LocalJson);
    store.save(&config).unwrap();
    let saved: Value = serde_json::from_slice(&fs::read(paths.config_file()).unwrap()).unwrap();
    assert_eq!(saved["plugin"]["a"], true);
    assert_eq!(saved["general"]["future"], 1);
    assert_eq!(store.load().unwrap(), config);
}

#[test]
fn corrupt_file_is_set_aside() {
    let (_folder, paths) = workspace();
    fs::write(paths.config_file(), "{roto").unwrap();
    let config = ConfigStore::new(paths.clone()).load().unwrap();
    assert_eq!(config, AppConfig::default());
    assert!(paths.config_dir.join("config.json.bak").exists());
}

#[test]
fn credentials_are_encrypted_at_rest() {
    let (_folder, paths) = workspace();
    let store = CredentialStore::with_keys(&paths, Arc::new(FixedKey(1)));
    let data = Credentials {
        password: "valor-de-prueba".into(),
        ..Credentials::default()
    };
    store.save(BackendKind::Mysql, &data).unwrap();
    let raw = fs::read_to_string(paths.credentials_file()).unwrap();
    assert!(!raw.contains("valor-de-prueba"));
    assert!(store.load(BackendKind::Mysql).unwrap() == data);
    let other = CredentialStore::with_keys(&paths, Arc::new(FixedKey(2)));
    assert!(other.load(BackendKind::Mysql).is_err());
    store.clear(BackendKind::Mysql).unwrap();
    assert!(store.load(BackendKind::Mysql).unwrap() == Credentials::default());
}
