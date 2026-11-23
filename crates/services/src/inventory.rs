use std::sync::Arc;

use pos_domain::{now, Product};
use pos_storage::Store;
use rust_decimal::Decimal;

use crate::error::rule;
use crate::Result;

pub struct InventoryService {
    store: Arc<dyn Store>,
}

impl InventoryService {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self { store }
    }

    pub fn list(&self, include_inactive: bool) -> Result<Vec<Product>> {
        let mut items = self.store.products().all()?;
        items.retain(|item| include_inactive || item.active);
        Ok(items)
    }

    pub fn find(&self, id: &str) -> Result<Option<Product>> {
        Ok(self.store.products().find(id)?)
    }

    pub fn save(&self, mut item: Product) -> Result<Product> {
        for field in [&mut item.name, &mut item.description, &mut item.barcode] {
            *field = field.trim().to_owned();
        }
        if item.name.is_empty() {
            return rule("El nombre es obligatorio");
        }
        if item.price <= Decimal::ZERO {
            return rule("El precio de venta debe ser mayor que cero");
        }
        if item.cost < Decimal::ZERO {
            return rule("El costo no puede ser negativo");
        }
        let others = self.list(true)?;
        if let Some(other) = others
            .iter()
            .find(|other| other.id != item.id && other.has_barcode(&item.barcode))
        {
            return rule(format!("El codigo ya pertenece a '{}'", other.name));
        }
        item.updated_at = now();
        self.store.products().upsert(&item)?;
        Ok(item)
    }

    pub fn adjust_stock(&self, id: &str, delta: i64) -> Result<Option<Product>> {
        let Some(mut product) = self.find(id)? else {
            return Ok(None);
        };
        product.stock = (product.stock + delta).max(0);
        product.updated_at = now();
        self.store.products().upsert(&product)?;
        Ok(Some(product))
    }

    pub fn delete(&self, id: &str) -> Result<bool> {
        Ok(self.store.products().delete(id)?)
    }
}

pub fn search<'a>(
    products: &'a [Product],
    query: &str,
    category_name: impl Fn(&str) -> String,
) -> Vec<&'a Product> {
    let lowered = query.trim().to_lowercase();
    let mut found: Vec<&Product> = products
        .iter()
        .filter(|product| {
            [&product.name, &product.description, &product.barcode]
                .into_iter()
                .any(|field| field.to_lowercase().contains(&lowered))
                || category_name(&product.category_id)
                    .to_lowercase()
                    .contains(&lowered)
        })
        .collect();
    found.sort_by_key(|product| !product.has_barcode(query));
    found
}
