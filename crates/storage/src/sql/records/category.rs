use pos_domain::Category;

use crate::sql::schema::{column, Column, SqlRecord, Table};
use crate::sql::value::{Kind, Row, SqlValue};
use crate::Result;

const COLUMNS: &[Column] = &[
    column("id", Kind::Id),
    column("name", Kind::Short),
    column("description", Kind::Long),
    column("active", Kind::Bool),
    column("created_at", Kind::Moment),
    column("updated_at", Kind::Moment),
];

impl SqlRecord for Category {
    const TABLE: Table = Table {
        name: "pos_categories",
        columns: COLUMNS,
        primary_key: "id",
        parent: None,
        indexes: &[],
    };

    fn values(&self) -> Vec<SqlValue> {
        vec![
            (&self.id).into(),
            (&self.name).into(),
            (&self.description).into(),
            self.active.into(),
            self.created_at.into(),
            self.updated_at.into(),
        ]
    }

    fn from_row(row: &mut Row) -> Result<Self> {
        Ok(Self {
            id: row.text()?,
            name: row.text()?,
            description: row.text()?,
            active: row.bool()?,
            created_at: row.moment()?,
            updated_at: row.moment()?,
        })
    }
}
