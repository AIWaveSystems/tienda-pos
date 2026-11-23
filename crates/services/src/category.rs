use std::sync::Arc;

use pos_domain::{now, Category};
use pos_storage::Store;

use crate::error::rule;
use crate::Result;

pub struct CategoryService {
    store: Arc<dyn Store>,
}

impl CategoryService {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self { store }
    }

    pub fn list(&self, include_inactive: bool) -> Result<Vec<Category>> {
        let mut items = self.store.categories().all()?;
        items.retain(|item| include_inactive || item.active);
        Ok(items)
    }

    pub fn save(&self, mut item: Category) -> Result<Category> {
        item.name = item.name.trim().to_owned();
        item.description = item.description.trim().to_owned();
        if item.name.is_empty() {
            return rule("El nombre es obligatorio");
        }
        let lowered = item.name.to_lowercase();
        let others = self.list(true)?;
        if let Some(other) = others
            .iter()
            .find(|other| other.id != item.id && other.name.to_lowercase() == lowered)
        {
            return rule(format!("Ya existe una categoria llamada '{}'", other.name));
        }
        item.updated_at = now();
        self.store.categories().upsert(&item)?;
        Ok(item)
    }

    pub fn delete(&self, id: &str) -> Result<bool> {
        Ok(self.store.categories().delete(id)?)
    }
}

pub fn matches(item: &Category, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    item.name.to_lowercase().contains(&query) || item.description.to_lowercase().contains(&query)
}
