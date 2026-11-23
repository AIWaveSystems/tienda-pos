use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{new_id, now, Entity};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Client {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub tax_id: String,
    pub notes: String,
    pub active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Default for Client {
    fn default() -> Self {
        let moment = now();
        Self {
            id: new_id(),
            name: String::new(),
            email: String::new(),
            phone: String::new(),
            address: String::new(),
            tax_id: String::new(),
            notes: String::new(),
            active: true,
            created_at: moment,
            updated_at: moment,
        }
    }
}

impl Entity for Client {
    const COLLECTION: &'static str = "clients";

    fn id(&self) -> &str {
        &self.id
    }
}
