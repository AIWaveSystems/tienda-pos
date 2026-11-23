use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use pos_domain::now;

use super::Document;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PrintOutcome {
    Sent,
    OpenedInBrowser,
}

pub fn save(document: &Document, destination: &Path) -> io::Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(destination, &document.html)
}

pub fn preview(document: &Document) -> io::Result<()> {
    open::that(temporary(document)?)
}

pub fn print(document: &Document, printer: &str) -> io::Result<PrintOutcome> {
    let path = temporary(document)?;
    if send(&path, printer) {
        return Ok(PrintOutcome::Sent);
    }
    open::that(path)?;
    Ok(PrintOutcome::OpenedInBrowser)
}

fn temporary(document: &Document) -> io::Result<PathBuf> {
    let stamp = now().format("%Y%m%d_%H%M%S");
    let path = env::temp_dir()
        .join("VentasPOSRust")
        .join(format!("{}_{stamp}.html", document.file_name()));
    save(document, &path)?;
    Ok(path)
}

#[cfg(windows)]
fn send(path: &Path, printer: &str) -> bool {
    let script = "$name = '\"' + $env:POS_PRINTER + '\"'; \
                  Start-Process -FilePath $env:POS_DOCUMENT -Verb PrintTo \
                  -ArgumentList $name -ErrorAction Stop";
    succeeds(
        Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .env("POS_DOCUMENT", path)
            .env("POS_PRINTER", printer),
    )
}

#[cfg(not(windows))]
fn send(path: &Path, printer: &str) -> bool {
    succeeds(Command::new("lp").args(["-d", printer]).arg(path))
        || succeeds(Command::new("lpr").args(["-P", printer]).arg(path))
}

fn succeeds(command: &mut Command) -> bool {
    hidden(command)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[cfg(windows)]
pub fn hidden(command: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW)
}

#[cfg(not(windows))]
pub fn hidden(command: &mut Command) -> &mut Command {
    command
}
