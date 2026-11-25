use pos_domain::{PaymentMethod, Sale, SaleItem};

use crate::sql::schema::{column, Column, Index, SqlRecord, Table};
use crate::sql::value::{Kind, Row, SqlValue};
use crate::Result;

const COLUMNS: &[Column] = &[
    column("id", Kind::Id),
    column("client_id", Kind::Id),
    column("client_name", Kind::Short),
    column("subtotal", Kind::Money),
    column("tax", Kind::Money),
    column("discount", Kind::Money),
    column("total", Kind::Money),
    column("payment_method", Kind::Short),
    column("notes", Kind::Long),
    column("created_at", Kind::Moment),
    column("seller", Kind::Short),
    column("voided_at", Kind::OptionalMoment),
];

const ITEM_COLUMNS: &[Column] = &[
    column("sale_id", Kind::Id),
    column("position", Kind::Int),
    column("product_id", Kind::Id),
    column("product_name", Kind::Short),
    column("quantity", Kind::Int),
    column("unit_price", Kind::Money),
    column("subtotal", Kind::Money),
];

impl SqlRecord for Sale {
    const TABLE: Table = Table {
        name: "pos_sales",
        columns: COLUMNS,
        primary_key: "id",
        parent: None,
        indexes: &[
            Index {
                columns: "created_at",
                unique: false,
            },
            Index {
                columns: "client_id",
                unique: false,
            },
        ],
    };

    const CHILD: Option<Table> = Some(Table {
        name: "pos_sale_items",
        columns: ITEM_COLUMNS,
        primary_key: "sale_id, position",
        parent: Some("pos_sales"),
        indexes: &[Index {
            columns: "product_id",
            unique: false,
        }],
    });

    fn values(&self) -> Vec<SqlValue> {
        vec![
            (&self.id).into(),
            (&self.client_id).into(),
            (&self.client_name).into(),
            self.subtotal.into(),
            self.tax.into(),
            self.discount.into(),
            self.total.into(),
            self.payment_method.key().into(),
            (&self.notes).into(),
            self.created_at.into(),
            (&self.seller).into(),
            self.voided_at.into(),
        ]
    }

    fn from_row(row: &mut Row) -> Result<Self> {
        Ok(Self {
            id: row.text()?,
            client_id: row.text()?,
            client_name: row.text()?,
            items: Vec::new(),
            subtotal: row.money()?,
            tax: row.money()?,
            discount: row.money()?,
            total: row.money()?,
            payment_method: PaymentMethod::from_key(&row.text()?).unwrap_or(PaymentMethod::Otro),
            notes: row.text()?,
            created_at: row.moment()?,
            seller: row.text()?,
            voided_at: row.optional_moment()?,
        })
    }

    fn child_values(&self) -> Vec<Vec<SqlValue>> {
        self.items
            .iter()
            .enumerate()
            .map(|(position, item)| {
                vec![
                    (&self.id).into(),
                    (position as i64).into(),
                    (&item.product_id).into(),
                    (&item.product_name).into(),
                    item.quantity.into(),
                    item.unit_price.into(),
                    item.subtotal.into(),
                ]
            })
            .collect()
    }

    fn attach_child(&mut self, row: &mut Row) -> Result<()> {
        row.int()?;
        self.items.push(SaleItem {
            product_id: row.text()?,
            product_name: row.text()?,
            quantity: row.int()?,
            unit_price: row.money()?,
            subtotal: row.money()?,
        });
        Ok(())
    }
}
