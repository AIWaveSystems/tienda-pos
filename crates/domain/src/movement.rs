use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{new_id, now, Entity, Sale};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MovementKind {
    Venta,
    Anulacion,
    #[default]
    #[serde(other)]
    Ajuste,
}

impl MovementKind {
    pub const ALL: [Self; 3] = [Self::Venta, Self::Anulacion, Self::Ajuste];

    pub fn key(self) -> &'static str {
        match self {
            Self::Venta => "venta",
            Self::Anulacion => "anulacion",
            Self::Ajuste => "ajuste",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Venta => "Venta",
            Self::Anulacion => "Anulacion",
            Self::Ajuste => "Ajuste",
        }
    }

    pub fn from_key(key: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|kind| kind.key() == key)
            .unwrap_or_default()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StockMovement {
    pub id: String,
    pub product_id: String,
    pub product_name: String,
    pub kind: MovementKind,
    pub quantity: i64,
    pub reference: String,
    pub author: String,
    pub created_at: NaiveDateTime,
}

impl Default for StockMovement {
    fn default() -> Self {
        Self {
            id: new_id(),
            product_id: String::new(),
            product_name: String::new(),
            kind: MovementKind::default(),
            quantity: 0,
            reference: String::new(),
            author: String::new(),
            created_at: now(),
        }
    }
}

impl StockMovement {
    pub fn of_sale(sale: &Sale, kind: MovementKind, author: &str) -> Vec<Self> {
        let direction = if kind == MovementKind::Venta { -1 } else { 1 };
        sale.items
            .iter()
            .map(|item| Self {
                product_id: item.product_id.clone(),
                product_name: item.product_name.clone(),
                kind,
                quantity: direction * item.quantity,
                reference: sale.id.clone(),
                author: author.to_owned(),
                ..Self::default()
            })
            .collect()
    }
}

impl Entity for StockMovement {
    const COLLECTION: &'static str = "stock_movements";

    fn id(&self) -> &str {
        &self.id
    }
}
