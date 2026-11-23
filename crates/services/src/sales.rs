use std::collections::HashSet;
use std::sync::Arc;

use pos_domain::{Client, Product, Sale};
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
        let mut products = self.products_of(&sale)?;
        for (product, quantity) in &mut products {
            if !allow_negative && *quantity > product.stock {
                return rule(format!(
                    "'{}' solo tiene {} unidades",
                    product.name, product.stock
                ));
            }
            product.stock = (product.stock - *quantity).max(0);
        }
        sale.calculate_totals(tax_rate);
        self.store.sales().upsert(&sale)?;
        for (product, _) in &products {
            self.store.products().upsert(product)?;
        }
        Ok(sale)
    }

    fn products_of(&self, sale: &Sale) -> Result<Vec<(Product, i64)>> {
        let mut products = Vec::new();
        for item in &sale.items {
            if let Some(product) = self.store.products().find(&item.product_id)? {
                products.push((product, item.quantity));
            }
        }
        Ok(products)
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
