use std::fs;
use std::path::PathBuf;

use pos_config::{files, BackendKind};

use super::{FileBackend, Snapshot};
use crate::Result;

pub struct LocalFiles {
    folder: PathBuf,
}

impl LocalFiles {
    pub fn new(folder: impl Into<PathBuf>) -> Result<Self> {
        let folder = folder.into();
        fs::create_dir_all(&folder)?;
        Ok(Self { folder })
    }

    fn path(&self, name: &str) -> PathBuf {
        self.folder.join(name)
    }
}

impl FileBackend for LocalFiles {
    fn kind(&self) -> BackendKind {
        BackendKind::LocalJson
    }

    fn read(&self, name: &str) -> Result<Option<Snapshot>> {
        let path = self.path(name);
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(Snapshot {
            content: fs::read(path)?,
            revision: String::new(),
        }))
    }

    fn write(&self, name: &str, content: &[u8], _expected: Option<&str>) -> Result<()> {
        files::write_atomic(&self.path(name), content)?;
        Ok(())
    }

    fn quarantine(&self, name: &str) -> Result<bool> {
        files::quarantine(&self.path(name))?;
        Ok(true)
    }

    fn ping(&self) -> Result<()> {
        fs::create_dir_all(&self.folder)?;
        Ok(())
    }
}
