use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{new_id, now, Entity};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub price: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub cost: Decimal,
    pub stock: i64,
    pub min_stock: i64,
    pub category_id: String,
    pub barcode: String,
    pub active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Default for Product {
    fn default() -> Self {
        let moment = now();
        Self {
            id: new_id(),
            name: String::new(),
            description: String::new(),
            price: Decimal::ZERO,
            cost: Decimal::ZERO,
            stock: 0,
            min_stock: 0,
            category_id: String::new(),
            barcode: String::new(),
            active: true,
            created_at: moment,
            updated_at: moment,
        }
    }
}

impl Product {
    pub fn low_stock(&self) -> bool {
        self.stock <= self.min_stock
    }

    pub fn profit_margin(&self) -> Decimal {
        margin(self.price, self.cost)
    }

    pub fn inventory_value(&self) -> Decimal {
        Decimal::from(self.stock) * self.cost
    }

    pub fn has_barcode(&self, code: &str) -> bool {
        let code = code.trim();
        !code.is_empty() && self.barcode.trim() == code
    }
}

pub fn margin(price: Decimal, cost: Decimal) -> Decimal {
    if cost <= Decimal::ZERO {
        return Decimal::ZERO;
    }
    (price - cost) / cost * Decimal::ONE_HUNDRED
}

impl Entity for Product {
    const COLLECTION: &'static str = "products";

    fn id(&self) -> &str {
        &self.id
    }
}
