use std::fs;

use serde_json::{Map, Value};

use crate::{files, sanitize, AppConfig, ConfigError, Paths};

#[derive(Clone, Debug)]
pub struct ConfigStore {
    paths: Paths,
}

impl ConfigStore {
    pub fn new(paths: Paths) -> Self {
        Self { paths }
    }

    pub fn paths(&self) -> &Paths {
        &self.paths
    }

    pub fn load(&self) -> Result<AppConfig, ConfigError> {
        Ok(sanitize(&self.read()?))
    }

    pub fn save(&self, config: &AppConfig) -> Result<(), ConfigError> {
        let mut stored = self.read()?;
        merge(&mut stored, serde_json::to_value(config)?);
        let content = serde_json::to_vec_pretty(&stored)?;
        files::write_atomic(&self.paths.config_file(), &content)?;
        Ok(())
    }

    pub fn reset(&self) -> Result<AppConfig, ConfigError> {
        let path = self.paths.config_file();
        if path.exists() {
            files::quarantine(&path)?;
        }
        let config = AppConfig::default();
        self.save(&config)?;
        Ok(config)
    }

    fn read(&self) -> Result<Value, ConfigError> {
        let path = self.paths.config_file();
        if !path.exists() {
            return Ok(Value::Object(Map::new()));
        }
        let content = fs::read(&path)?;
        match serde_json::from_slice::<Value>(&content) {
            Ok(value) if value.is_object() => Ok(value),
            _ => {
                files::quarantine(&path)?;
                Ok(Value::Object(Map::new()))
            }
        }
    }
}

fn merge(base: &mut Value, incoming: Value) {
    match (base, incoming) {
        (Value::Object(current), Value::Object(changes)) => {
            for (key, value) in changes {
                merge(current.entry(key).or_insert(Value::Null), value);
            }
        }
        (slot, value) => *slot = value,
    }
}
