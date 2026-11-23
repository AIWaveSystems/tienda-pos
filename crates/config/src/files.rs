use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn write_atomic(path: &Path, content: &[u8]) -> io::Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let mut file = tempfile::Builder::new()
        .suffix(".tmp")
        .tempfile_in(parent)?;
    file.write_all(content)?;
    file.as_file().sync_all()?;
    file.persist(path).map_err(|failure| failure.error)?;
    Ok(())
}

pub fn quarantine(path: &Path) -> io::Result<PathBuf> {
    let mut name = path.as_os_str().to_owned();
    name.push(".bak");
    let backup = PathBuf::from(name);
    fs::rename(path, &backup)?;
    Ok(backup)
}
