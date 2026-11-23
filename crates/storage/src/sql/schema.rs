use super::value::{Kind, Row, SqlValue};
use crate::Result;
use pos_domain::Entity;

#[derive(Clone, Copy)]
pub struct Column {
    pub name: &'static str,
    pub kind: Kind,
}

pub const fn column(name: &'static str, kind: Kind) -> Column {
    Column { name, kind }
}

#[derive(Clone, Copy)]
pub struct Index {
    pub columns: &'static str,
    pub unique: bool,
}

pub struct Table {
    pub name: &'static str,
    pub columns: &'static [Column],
    pub primary_key: &'static str,
    pub parent: Option<&'static str>,
    pub indexes: &'static [Index],
}

impl Table {
    pub fn column_list(&self) -> String {
        self.columns
            .iter()
            .map(|column| column.name)
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn kinds(&self) -> Vec<Kind> {
        self.columns.iter().map(|column| column.kind).collect()
    }
}

pub trait SqlRecord: Entity {
    const TABLE: Table;
    const CHILD: Option<Table> = None;

    fn values(&self) -> Vec<SqlValue>;

    fn from_row(row: &mut Row) -> Result<Self>;

    fn child_values(&self) -> Vec<Vec<SqlValue>> {
        Vec::new()
    }

    fn attach_child(&mut self, _row: &mut Row) -> Result<()> {
        Ok(())
    }
}
