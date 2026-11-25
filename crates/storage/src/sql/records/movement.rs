use pos_domain::{MovementKind, StockMovement};

use crate::sql::schema::{column, Column, Index, SqlRecord, Table};
use crate::sql::value::{Kind, Row, SqlValue};
use crate::Result;

const COLUMNS: &[Column] = &[
    column("id", Kind::Id),
    column("product_id", Kind::Id),
    column("product_name", Kind::Short),
    column("kind", Kind::Short),
    column("quantity", Kind::Int),
    column("reference", Kind::Short),
    column("author", Kind::Short),
    column("created_at", Kind::Moment),
];

impl SqlRecord for StockMovement {
    const TABLE: Table = Table {
        name: "pos_stock_movements",
        columns: COLUMNS,
        primary_key: "id",
        parent: None,
        indexes: &[
            Index {
                columns: "product_id",
                unique: false,
            },
            Index {
                columns: "created_at",
                unique: false,
            },
        ],
    };

    fn values(&self) -> Vec<SqlValue> {
        vec![
            (&self.id).into(),
            (&self.product_id).into(),
            (&self.product_name).into(),
            self.kind.key().into(),
            self.quantity.into(),
            (&self.reference).into(),
            (&self.author).into(),
            self.created_at.into(),
        ]
    }

    fn from_row(row: &mut Row) -> Result<Self> {
        Ok(Self {
            id: row.text()?,
            product_id: row.text()?,
            product_name: row.text()?,
            kind: MovementKind::from_key(&row.text()?),
            quantity: row.int()?,
            reference: row.text()?,
            author: row.text()?,
            created_at: row.moment()?,
        })
    }
}
