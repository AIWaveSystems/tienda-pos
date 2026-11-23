use pos_domain::Client;

use crate::sql::schema::{column, Column, Index, SqlRecord, Table};
use crate::sql::value::{Kind, Row, SqlValue};
use crate::Result;

const COLUMNS: &[Column] = &[
    column("id", Kind::Id),
    column("name", Kind::Short),
    column("email", Kind::Short),
    column("phone", Kind::Short),
    column("address", Kind::Long),
    column("tax_id", Kind::Short),
    column("notes", Kind::Long),
    column("active", Kind::Bool),
    column("created_at", Kind::Moment),
    column("updated_at", Kind::Moment),
];

impl SqlRecord for Client {
    const TABLE: Table = Table {
        name: "pos_clients",
        columns: COLUMNS,
        primary_key: "id",
        parent: None,
        indexes: &[Index {
            columns: "tax_id",
            unique: false,
        }],
    };

    fn values(&self) -> Vec<SqlValue> {
        vec![
            (&self.id).into(),
            (&self.name).into(),
            (&self.email).into(),
            (&self.phone).into(),
            (&self.address).into(),
            (&self.tax_id).into(),
            (&self.notes).into(),
            self.active.into(),
            self.created_at.into(),
            self.updated_at.into(),
        ]
    }

    fn from_row(row: &mut Row) -> Result<Self> {
        Ok(Self {
            id: row.text()?,
            name: row.text()?,
            email: row.text()?,
            phone: row.text()?,
            address: row.text()?,
            tax_id: row.text()?,
            notes: row.text()?,
            active: row.bool()?,
            created_at: row.moment()?,
            updated_at: row.moment()?,
        })
    }
}
