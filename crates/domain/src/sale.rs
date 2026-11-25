use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{clock, new_id, now, Entity, PaymentMethod};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SaleItem {
    pub product_id: String,
    pub product_name: String,
    pub quantity: i64,
    #[serde(with = "rust_decimal::serde::float")]
    pub unit_price: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub subtotal: Decimal,
}

impl SaleItem {
    pub fn new(product_id: &str, product_name: &str, quantity: i64, unit_price: Decimal) -> Self {
        let mut item = Self {
            product_id: product_id.to_owned(),
            product_name: product_name.to_owned(),
            quantity,
            unit_price,
            subtotal: Decimal::ZERO,
        };
        item.calculate();
        item
    }

    pub fn calculate(&mut self) -> Decimal {
        self.subtotal = Decimal::from(self.quantity) * self.unit_price;
        self.subtotal
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Sale {
    pub id: String,
    pub client_id: String,
    pub client_name: String,
    pub items: Vec<SaleItem>,
    #[serde(with = "rust_decimal::serde::float")]
    pub subtotal: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub tax: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub discount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total: Decimal,
    pub payment_method: PaymentMethod,
    pub notes: String,
    pub seller: String,
    #[serde(with = "clock::optional")]
    pub voided_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

impl Default for Sale {
    fn default() -> Self {
        Self {
            id: new_id(),
            client_id: String::new(),
            client_name: String::new(),
            items: Vec::new(),
            subtotal: Decimal::ZERO,
            tax: Decimal::ZERO,
            discount: Decimal::ZERO,
            total: Decimal::ZERO,
            payment_method: PaymentMethod::default(),
            notes: String::new(),
            seller: String::new(),
            voided_at: None,
            created_at: now(),
        }
    }
}

impl Sale {
    pub fn calculate_totals(&mut self, tax_rate: Decimal) {
        self.subtotal = self.items.iter_mut().map(SaleItem::calculate).sum();
        self.tax = (self.subtotal * tax_rate).round_dp(4);
        self.total = self.subtotal + self.tax - self.discount;
    }

    pub fn is_voided(&self) -> bool {
        self.voided_at.is_some()
    }

    pub fn short_id(&self) -> &str {
        self.id.get(..8).unwrap_or(&self.id)
    }

    pub fn date(&self) -> String {
        self.created_at.format("%Y-%m-%d").to_string()
    }
}

impl Entity for Sale {
    const COLLECTION: &'static str = "sales";

    fn id(&self) -> &str {
        &self.id
    }
}
