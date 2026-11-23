use std::sync::Arc;

use pos_domain::{now, Client};
use pos_storage::Store;

use crate::error::rule;
use crate::Result;

pub struct ClientService {
    store: Arc<dyn Store>,
}

impl ClientService {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self { store }
    }

    pub fn list(&self, include_inactive: bool) -> Result<Vec<Client>> {
        let mut items = self.store.clients().all()?;
        items.retain(|item| include_inactive || item.active);
        Ok(items)
    }

    pub fn save(&self, mut item: Client) -> Result<Client> {
        for field in [
            &mut item.name,
            &mut item.email,
            &mut item.phone,
            &mut item.address,
            &mut item.tax_id,
            &mut item.notes,
        ] {
            *field = field.trim().to_owned();
        }
        if item.name.is_empty() {
            return rule("El nombre es obligatorio");
        }
        if !item.email.is_empty() && !valid_email(&item.email) {
            return rule("El email no tiene un formato valido");
        }
        if !item.tax_id.is_empty() {
            let others = self.list(true)?;
            if let Some(other) = others
                .iter()
                .find(|other| other.id != item.id && other.tax_id == item.tax_id)
            {
                return rule(format!("El NIT ya pertenece a '{}'", other.name));
            }
        }
        item.updated_at = now();
        self.store.clients().upsert(&item)?;
        Ok(item)
    }

    pub fn delete(&self, id: &str) -> Result<bool> {
        Ok(self.store.clients().delete(id)?)
    }

    pub fn count(&self) -> Result<usize> {
        Ok(self.store.clients().count()?)
    }
}

pub fn matches(item: &Client, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    item.name.to_lowercase().contains(&query) || matches_contact(item, &query)
}

pub fn matches_contact(item: &Client, lowered: &str) -> bool {
    [&item.email, &item.phone, &item.tax_id]
        .into_iter()
        .any(|field| field.to_lowercase().contains(lowered))
}

fn valid_email(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    let Some((host, ending)) = domain.rsplit_once('.') else {
        return false;
    };
    !local.is_empty()
        && !host.is_empty()
        && !domain.contains('@')
        && !email.contains(char::is_whitespace)
        && ending.len() >= 2
        && ending.chars().all(|letter| letter.is_ascii_alphabetic())
}
