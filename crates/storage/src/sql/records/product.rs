use pos_domain::Product;

use crate::sql::schema::{column, Column, Index, SqlRecord, Table};
use crate::sql::value::{Kind, Row, SqlValue};
use crate::Result;

const COLUMNS: &[Column] = &[
    column("id", Kind::Id),
    column("name", Kind::Short),
    column("description", Kind::Long),
    column("price", Kind::Money),
    column("cost", Kind::Money),
    column("stock", Kind::Int),
    column("min_stock", Kind::Int),
    column("category_id", Kind::Id),
    column("barcode", Kind::Short),
    column("active", Kind::Bool),
    column("created_at", Kind::Moment),
    column("updated_at", Kind::Moment),
];

impl SqlRecord for Product {
    const TABLE: Table = Table {
        name: "pos_products",
        columns: COLUMNS,
        primary_key: "id",
        parent: None,
        indexes: &[
            Index {
                columns: "barcode",
                unique: false,
            },
            Index {
                columns: "category_id",
                unique: false,
            },
        ],
    };

    fn values(&self) -> Vec<SqlValue> {
        vec![
            (&self.id).into(),
            (&self.name).into(),
            (&self.description).into(),
            self.price.into(),
            self.cost.into(),
            self.stock.into(),
            self.min_stock.into(),
            (&self.category_id).into(),
            (&self.barcode).into(),
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
            price: row.money()?,
            cost: row.money()?,
            stock: row.int()?,
            min_stock: row.int()?,
            category_id: row.text()?,
            barcode: row.text()?,
            active: row.bool()?,
            created_at: row.moment()?,
            updated_at: row.moment()?,
        })
    }
}
