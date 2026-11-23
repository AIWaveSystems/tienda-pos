use std::collections::HashSet;
use std::sync::Arc;

use pos_domain::{Client, Sale};
use pos_storage::Store;
use rust_decimal::Decimal;

use crate::client::matches_contact;
use crate::error::rule;
use crate::Result;

pub struct SalesService {
    store: Arc<dyn Store>,
}

impl SalesService {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self { store }
    }

    pub fn list(&self) -> Result<Vec<Sale>> {
        Ok(self.store.sales().all()?)
    }

    pub fn register(
        &self,
        mut sale: Sale,
        tax_rate: Decimal,
        allow_negative: bool,
    ) -> Result<Sale> {
        if sale.items.is_empty() {
            return rule("Agregue productos antes de confirmar");
        }
        sale.calculate_totals(tax_rate);
        self.store.register_sale(&sale, allow_negative)?;
        Ok(sale)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SaleFilter {
    pub query: String,
    pub from: String,
    pub to: String,
}

pub fn filter<'a>(sales: &'a [Sale], clients: &[Client], filter: &SaleFilter) -> Vec<&'a Sale> {
    let query = filter.query.trim().to_lowercase();
    let (from, to) = (filter.from.trim(), filter.to.trim());
    let owners: HashSet<&str> = clients
        .iter()
        .filter(|client| !query.is_empty() && matches_contact(client, &query))
        .map(|client| client.id.as_str())
        .collect();
    sales
        .iter()
        .filter(|sale| {
            let date = sale.date();
            (from.is_empty() || date.as_str() >= from)
                && (to.is_empty() || date.as_str() <= to)
                && (query.is_empty()
                    || sale.id.contains(&query)
                    || sale.client_name.to_lowercase().contains(&query)
                    || sale.payment_method.key().contains(&query)
                    || sale.notes.to_lowercase().contains(&query)
                    || owners.contains(sale.client_id.as_str()))
        })
        .collect()
}
