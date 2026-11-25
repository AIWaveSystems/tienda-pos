use std::sync::Arc;

use pos_domain::{now, Product, StockMovement};
use pos_storage::{SaleChange, StockOperation, Store};
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
        if let Some(current) = others.iter().find(|other| other.id == item.id) {
            item.stock = current.stock;
        }
        item.updated_at = now();
        self.store.products().upsert(&item)?;
        Ok(item)
    }

    pub fn adjust_stock(
        &self,
        product: &Product,
        delta: i64,
        reason: &str,
        author: &str,
    ) -> Result<()> {
        if delta == 0 {
            return rule("Ingrese una cantidad distinta de cero");
        }
        if reason.trim().is_empty() {
            return rule("Escriba el motivo del ajuste");
        }
        let movement = StockMovement {
            product_id: product.id.clone(),
            product_name: product.name.clone(),
            quantity: delta,
            reference: reason.trim().to_owned(),
            author: author.to_owned(),
            ..StockMovement::default()
        };
        self.store.apply_stock(&StockOperation {
            sale: SaleChange::None,
            movements: &[movement],
            allow_negative: false,
        })?;
        Ok(())
    }

    pub fn movements(&self) -> Result<Vec<StockMovement>> {
        let mut items = self.store.movements().all()?;
        items.sort_by_key(|item| std::cmp::Reverse(item.created_at));
        Ok(items)
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

pub fn movement_matches(item: &StockMovement, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    [
        item.product_name.as_str(),
        item.kind.label(),
        item.reference.as_str(),
        item.author.as_str(),
    ]
    .into_iter()
    .any(|field| field.to_lowercase().contains(&query))
}
