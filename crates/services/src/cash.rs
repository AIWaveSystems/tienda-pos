use std::sync::Arc;

use pos_domain::{now, CashEntry, CashKind};
use pos_storage::Store;
use rust_decimal::Decimal;

use crate::error::rule;
use crate::Result;

pub struct CashService {
    store: Arc<dyn Store>,
}

impl CashService {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self { store }
    }

    pub fn list(&self) -> Result<Vec<CashEntry>> {
        let mut items = self.store.cash().all()?;
        items.sort_by_key(|item| std::cmp::Reverse(item.created_at));
        Ok(items)
    }

    pub fn register(&self, mut entry: CashEntry) -> Result<CashEntry> {
        entry.concept = entry.concept.trim().to_owned();
        entry.category = entry.category.trim().to_owned();
        if entry.concept.is_empty() {
            return rule("El concepto es obligatorio");
        }
        if entry.amount <= Decimal::ZERO {
            return rule("El valor debe ser mayor que cero");
        }
        entry.created_at = now();
        self.store.cash().upsert(&entry)?;
        Ok(entry)
    }

    pub fn void(&self, id: &str) -> Result<()> {
        let Some(mut entry) = self.store.cash().find(id)? else {
            return rule("El movimiento de caja no existe");
        };
        if entry.is_voided() {
            return rule("El movimiento ya fue anulado");
        }
        entry.voided_at = Some(now());
        self.store.cash().upsert(&entry)?;
        Ok(())
    }
}

pub fn total(entries: &[CashEntry], kind: CashKind) -> Decimal {
    entries
        .iter()
        .filter(|entry| entry.kind == kind && !entry.is_voided())
        .map(|entry| entry.amount)
        .sum()
}

pub fn matches(entry: &CashEntry, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    [
        entry.concept.as_str(),
        entry.category.as_str(),
        entry.author.as_str(),
        entry.kind.label(),
        entry.payment_method.label(),
    ]
    .into_iter()
    .any(|field| field.to_lowercase().contains(&query))
}
