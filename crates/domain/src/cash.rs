use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{clock, new_id, now, Entity, PaymentMethod};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CashKind {
    Entrada,
    #[default]
    Gasto,
}

impl CashKind {
    pub const ALL: [Self; 2] = [Self::Entrada, Self::Gasto];

    pub fn key(self) -> &'static str {
        match self {
            Self::Entrada => "entrada",
            Self::Gasto => "gasto",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Entrada => "Entrada",
            Self::Gasto => "Gasto",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.key() == key)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CashEntry {
    pub id: String,
    pub kind: CashKind,
    pub concept: String,
    pub category: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
    pub payment_method: PaymentMethod,
    pub author: String,
    #[serde(with = "clock::optional")]
    pub voided_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

impl Default for CashEntry {
    fn default() -> Self {
        Self {
            id: new_id(),
            kind: CashKind::default(),
            concept: String::new(),
            category: String::new(),
            amount: Decimal::ZERO,
            payment_method: PaymentMethod::default(),
            author: String::new(),
            voided_at: None,
            created_at: now(),
        }
    }
}

impl CashEntry {
    pub fn is_voided(&self) -> bool {
        self.voided_at.is_some()
    }

    pub fn signed(&self) -> Decimal {
        match self.kind {
            CashKind::Entrada => self.amount,
            CashKind::Gasto => -self.amount,
        }
    }
}

impl Entity for CashEntry {
    const COLLECTION: &'static str = "cash_entries";

    fn id(&self) -> &str {
        &self.id
    }
}
