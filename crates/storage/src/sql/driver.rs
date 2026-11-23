use super::dialect::Dialect;
use super::value::{Kind, Row, SqlValue};
use crate::{Result, StorageError};

pub struct Statement {
    pub sql: String,
    pub values: Vec<SqlValue>,
    pub required: Option<String>,
}

impl Statement {
    pub fn new(sql: impl Into<String>, values: Vec<SqlValue>) -> Self {
        Self {
            sql: sql.into(),
            values,
            required: None,
        }
    }

    pub fn requiring(mut self, name: &str) -> Self {
        self.required = Some(name.to_owned());
        self
    }

    pub fn verify(&self, affected: u64) -> Result<()> {
        match &self.required {
            Some(name) if affected == 0 => Err(StorageError::Stock(name.clone())),
            _ => Ok(()),
        }
    }
}

pub trait SqlDriver: Send + Sync {
    fn dialect(&self) -> Dialect;

    fn query(&self, statement: &Statement, kinds: &[Kind]) -> Result<Vec<Row>>;

    fn transaction(&self, statements: &[Statement]) -> Result<Vec<u64>>;

    fn migrate(&self, statements: &[String]) -> Result<()>;

    fn close(&self);
}

pub fn failure(error: sqlx::Error) -> StorageError {
    match error {
        sqlx::Error::Database(detail) if detail.is_unique_violation() => {
            StorageError::Conflict(detail.to_string())
        }
        sqlx::Error::ColumnDecode { .. } | sqlx::Error::Decode(_) => {
            StorageError::Data(error.to_string())
        }
        other => StorageError::Connection(other.to_string()),
    }
}
