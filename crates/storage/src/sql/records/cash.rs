use pos_domain::{CashEntry, CashKind, PaymentMethod};

use crate::sql::schema::{column, Column, Index, SqlRecord, Table};
use crate::sql::value::{Kind, Row, SqlValue};
use crate::{Result, StorageError};

const COLUMNS: &[Column] = &[
    column("id", Kind::Id),
    column("kind", Kind::Short),
    column("concept", Kind::Short),
    column("category", Kind::Short),
    column("amount", Kind::Money),
    column("payment_method", Kind::Short),
    column("author", Kind::Short),
    column("voided_at", Kind::OptionalMoment),
    column("created_at", Kind::Moment),
];

impl SqlRecord for CashEntry {
    const TABLE: Table = Table {
        name: "pos_cash_entries",
        columns: COLUMNS,
        primary_key: "id",
        parent: None,
        indexes: &[Index {
            columns: "created_at",
            unique: false,
        }],
    };

    fn values(&self) -> Vec<SqlValue> {
        vec![
            (&self.id).into(),
            self.kind.key().into(),
            (&self.concept).into(),
            (&self.category).into(),
            self.amount.into(),
            self.payment_method.key().into(),
            (&self.author).into(),
            self.voided_at.into(),
            self.created_at.into(),
        ]
    }

    fn from_row(row: &mut Row) -> Result<Self> {
        Ok(Self {
            id: row.text()?,
            kind: kind(&row.text()?)?,
            concept: row.text()?,
            category: row.text()?,
            amount: row.money()?,
            payment_method: PaymentMethod::from_key(&row.text()?).unwrap_or(PaymentMethod::Otro),
            author: row.text()?,
            voided_at: row.optional_moment()?,
            created_at: row.moment()?,
        })
    }
}

fn kind(key: &str) -> Result<CashKind> {
    CashKind::from_key(key)
        .ok_or_else(|| StorageError::Data(format!("tipo de movimiento de caja '{key}'")))
}
