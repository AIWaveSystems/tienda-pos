mod google_drive;
mod http;
pub mod oauth;
pub use google_drive::{DriveEndpoints, GoogleDrive};
mod dropbox;
pub use dropbox::{Dropbox, DropboxEndpoints};
mod account;
pub use account::{app_for, CloudAccount};
