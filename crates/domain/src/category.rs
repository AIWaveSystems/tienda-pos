use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{new_id, now, Entity};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub description: String,
    pub active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Default for Category {
    fn default() -> Self {
        let moment = now();
        Self {
            id: new_id(),
            name: String::new(),
            description: String::new(),
            active: true,
            created_at: moment,
            updated_at: moment,
        }
    }
}

impl Entity for Category {
    const COLLECTION: &'static str = "categories";

    fn id(&self) -> &str {
        &self.id
    }
}
