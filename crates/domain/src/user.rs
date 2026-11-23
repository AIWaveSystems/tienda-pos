use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{clock, new_id, now, password, permission, Entity, Permission, Role};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub full_name: String,
    pub role: Role,
    #[serde(deserialize_with = "permission::lenient")]
    pub permissions: Vec<Permission>,
    pub is_default: bool,
    pub active: bool,
    #[serde(with = "clock::optional")]
    pub last_login: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Default for User {
    fn default() -> Self {
        let moment = now();
        Self {
            id: new_id(),
            username: String::new(),
            password_hash: String::new(),
            full_name: String::new(),
            role: Role::default(),
            permissions: Vec::new(),
            is_default: true,
            active: true,
            last_login: None,
            created_at: moment,
            updated_at: moment,
        }
    }
}

impl User {
    pub fn set_password(&mut self, plain: &str) {
        self.password_hash = password::hash(plain);
        self.updated_at = now();
    }

    pub fn check_password(&self, plain: &str) -> bool {
        password::verify(plain, &self.password_hash)
    }

    pub fn is_admin(&self) -> bool {
        self.role == Role::Admin
    }

    pub fn effective_permissions(&self) -> Vec<Permission> {
        if self.role.is_locked() {
            return self.role.permissions();
        }
        Permission::normalize(self.permissions.iter().copied())
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        self.effective_permissions().contains(&permission)
    }
}

impl Entity for User {
    const COLLECTION: &'static str = "users";

    fn id(&self) -> &str {
        &self.id
    }
}
