use pos_config::BackendKind;

use crate::Result;

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub content: Vec<u8>,
    pub revision: String,
}

pub trait FileBackend: Send + Sync {
    fn kind(&self) -> BackendKind;

    fn read(&self, name: &str) -> Result<Option<Snapshot>>;

    fn write(&self, name: &str, content: &[u8], expected: Option<&str>) -> Result<()>;

    fn quarantine(&self, _name: &str) -> Result<bool> {
        Ok(false)
    }

    fn ping(&self) -> Result<()>;
}
