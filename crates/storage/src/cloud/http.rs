use std::io::Read;
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use serde_json::Value;
use ureq::{Agent, AgentBuilder, Request, Response};

use crate::{Result, StorageError};

pub struct Http {
    agent: Agent,
    provider: &'static str,
}

impl Http {
    pub fn new(provider: &'static str) -> Self {
        Self {
            agent: AgentBuilder::new().timeout(Duration::from_secs(20)).build(),
            provider,
        }
    }

    pub fn request(&self, method: &str, url: &str) -> Request {
        self.agent.request(method, url)
    }

    pub fn post_form(&self, url: &str, fields: &[(&str, &str)]) -> Result<Value> {
        let response = self
            .agent
            .post(url)
            .send_form(fields)
            .map_err(|error| self.failure(error))?;
        read_json(response)
    }

    pub fn failure(&self, error: ureq::Error) -> StorageError {
        match error {
            ureq::Error::Status(code, response) => {
                let detail = response.into_string().unwrap_or_default();
                StorageError::Connection(format!("Error de {} ({code}): {detail}", self.provider))
            }
            ureq::Error::Transport(detail) => StorageError::Connection(format!(
                "No se pudo conectar con {}: {detail}",
                self.provider
            )),
        }
    }
}

pub fn read_json(response: Response) -> Result<Value> {
    Ok(serde_json::from_slice(&read_bytes(response)?)?)
}

pub fn read_bytes(response: Response) -> Result<Vec<u8>> {
    let mut content = Vec::new();
    response.into_reader().read_to_end(&mut content)?;
    Ok(content)
}

pub fn text<'a>(value: &'a Value, field: &str) -> Result<&'a str> {
    value[field]
        .as_str()
        .ok_or_else(|| StorageError::Data(format!("la respuesta no incluye {field}")))
}

#[derive(Default)]
pub struct AccessToken(Mutex<Option<String>>);

impl AccessToken {
    pub fn get(&self, refresh: impl FnOnce() -> Result<String>) -> Result<String> {
        let mut slot = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(token) = slot.as_ref() {
            return Ok(token.clone());
        }
        let token = refresh()?;
        *slot = Some(token.clone());
        Ok(token)
    }

    pub fn forget(&self) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = None;
    }
}
