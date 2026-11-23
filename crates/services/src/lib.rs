mod error;
pub use error::{Result, ServiceError};
mod auth;
pub mod format;
pub use auth::{AuthService, UserDraft};
