mod error;
pub use error::{Result, StorageError};
mod repository;
pub use repository::Repository;
mod store;
pub use store::Store;
pub mod document;
pub use document::{DocumentStore, FileBackend};
