use chrono::NaiveDateTime;
use rust_decimal::Decimal;

use crate::{Result, StorageError};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Id,
    Short,
    Long,
    Int,
    Bool,
    Money,
    Moment,
    OptionalMoment,
}

#[derive(Clone, Debug)]
pub enum SqlValue {
    Text(String),
    Int(i64),
    Bool(bool),
    Money(Decimal),
    Moment(Option<NaiveDateTime>),
}

impl From<&str> for SqlValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<&String> for SqlValue {
    fn from(value: &String) -> Self {
        Self::Text(value.clone())
    }
}

impl From<i64> for SqlValue {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<bool> for SqlValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<Decimal> for SqlValue {
    fn from(value: Decimal) -> Self {
        Self::Money(value)
    }
}

impl From<NaiveDateTime> for SqlValue {
    fn from(value: NaiveDateTime) -> Self {
        Self::Moment(Some(value))
    }
}

impl From<Option<NaiveDateTime>> for SqlValue {
    fn from(value: Option<NaiveDateTime>) -> Self {
        Self::Moment(value)
    }
}

pub struct Row(std::vec::IntoIter<SqlValue>);

impl Row {
    pub fn new(values: Vec<SqlValue>) -> Self {
        Self(values.into_iter())
    }

    pub fn text(&mut self) -> Result<String> {
        match self.0.next() {
            Some(SqlValue::Text(value)) => Ok(value),
            _ => Err(mismatch("texto")),
        }
    }

    pub fn int(&mut self) -> Result<i64> {
        match self.0.next() {
            Some(SqlValue::Int(value)) => Ok(value),
            _ => Err(mismatch("entero")),
        }
    }

    pub fn bool(&mut self) -> Result<bool> {
        match self.0.next() {
            Some(SqlValue::Bool(value)) => Ok(value),
            _ => Err(mismatch("logico")),
        }
    }

    pub fn money(&mut self) -> Result<Decimal> {
        match self.0.next() {
            Some(SqlValue::Money(value)) => Ok(value),
            _ => Err(mismatch("decimal")),
        }
    }

    pub fn optional_moment(&mut self) -> Result<Option<NaiveDateTime>> {
        match self.0.next() {
            Some(SqlValue::Moment(value)) => Ok(value),
            _ => Err(mismatch("fecha")),
        }
    }

    pub fn moment(&mut self) -> Result<NaiveDateTime> {
        self.optional_moment()?.ok_or_else(|| mismatch("fecha"))
    }
}

fn mismatch(expected: &str) -> StorageError {
    StorageError::Data(format!("se esperaba una columna de tipo {expected}"))
}
