use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use pos_domain::{now, password, Permission, Role, User};
use pos_storage::Store;

use crate::error::rule;
use crate::Result;

pub const DEFAULT_USERNAME: &str = "admin";
pub const DEFAULT_PASSWORD: &str = "admin123";
const MIN_PASSWORD_LENGTH: usize = 6;
const MIN_USERNAME_LENGTH: usize = 3;
const NEEDS_ADMIN: &str = "Debe existir al menos un administrador activo";

#[derive(Clone, Debug)]
pub struct UserDraft {
    pub username: String,
    pub full_name: String,
    pub role: Role,
    pub permissions: Vec<Permission>,
    pub active: bool,
}

pub struct AuthService {
    store: Arc<dyn Store>,
    current: Mutex<Option<User>>,
}

impl AuthService {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self {
            store,
            current: Mutex::new(None),
        }
    }

    pub fn ensure_default_user(&self) -> Result<User> {
        let users = self.store.users().all()?;
        if let Some(user) = users.into_iter().find(|u| u.username == DEFAULT_USERNAME) {
            return Ok(user);
        }
        let mut user = User {
            username: DEFAULT_USERNAME.to_owned(),
            full_name: "Administrador".to_owned(),
            role: Role::Admin,
            permissions: Permission::ALL.to_vec(),
            ..User::default()
        };
        user.set_password(DEFAULT_PASSWORD);
        self.store.users().upsert(&user)?;
        Ok(user)
    }

    pub fn authenticate(&self, username: &str, plain: &str) -> Result<Option<User>> {
        let target = username.trim().to_lowercase();
        let users = self.store.users().all()?;
        let Some(mut user) = users
            .into_iter()
            .find(|user| user.active && user.username.to_lowercase() == target)
        else {
            return Ok(None);
        };
        if !user.check_password(plain) {
            return Ok(None);
        }
        if password::needs_rehash(&user.password_hash) {
            user.set_password(plain);
        }
        user.last_login = Some(now());
        self.store.users().upsert(&user)?;
        *self.session() = Some(user.clone());
        Ok(Some(user))
    }

    pub fn validate_password(plain: &str) -> Result<()> {
        if plain.is_empty() {
            return rule("La contrasena es obligatoria");
        }
        if plain.chars().count() < MIN_PASSWORD_LENGTH {
            return rule(format!(
                "La contrasena debe tener al menos {MIN_PASSWORD_LENGTH} caracteres"
            ));
        }
        Ok(())
    }

    pub fn create_user(&self, draft: &UserDraft, plain: &str) -> Result<User> {
        self.validate_username(&draft.username, "")?;
        Self::validate_password(plain)?;
        let mut user = User {
            is_default: false,
            ..User::default()
        };
        apply(&mut user, draft)?;
        user.set_password(plain);
        self.store.users().upsert(&user)?;
        Ok(user)
    }

    pub fn update_user(&self, id: &str, draft: &UserDraft) -> Result<User> {
        let mut user = self.require(id)?;
        self.validate_username(&draft.username, id)?;
        let demoted = draft.role != Role::Admin || !draft.active;
        if user.is_admin() && user.active && demoted && self.active_admins()? <= 1 {
            return rule(NEEDS_ADMIN);
        }
        apply(&mut user, draft)?;
        user.updated_at = now();
        self.persist(user)
    }

    pub fn delete_user(&self, id: &str) -> Result<()> {
        let user = self.require(id)?;
        if self.is_current(id) {
            return rule("No puede eliminar el usuario con el que inicio sesion");
        }
        if user.is_admin() && user.active && self.active_admins()? <= 1 {
            return rule(NEEDS_ADMIN);
        }
        self.store.users().delete(id)?;
        Ok(())
    }

    pub fn set_active(&self, id: &str, active: bool) -> Result<()> {
        let mut user = self.require(id)?;
        if self.is_current(id) && !active {
            return rule("No puede desactivar el usuario con el que inicio sesion");
        }
        if user.is_admin() && user.active && !active && self.active_admins()? <= 1 {
            return rule(NEEDS_ADMIN);
        }
        user.active = active;
        user.updated_at = now();
        self.persist(user)?;
        Ok(())
    }

    pub fn change_password(&self, id: &str, old: &str, new: &str) -> Result<()> {
        let user = self.require(id)?;
        if !user.check_password(old) {
            return rule("La contrasena actual es incorrecta");
        }
        Self::validate_password(new)?;
        if old == new {
            return rule("La nueva contrasena debe ser diferente a la actual");
        }
        self.store_password(user, new)
    }

    pub fn reset_password(&self, id: &str, new: &str) -> Result<()> {
        let user = self.require(id)?;
        Self::validate_password(new)?;
        self.store_password(user, new)
    }

    pub fn update_profile(&self, id: &str, full_name: &str, username: &str) -> Result<User> {
        let mut user = self.require(id)?;
        user.full_name = required_name(full_name)?;
        self.validate_username(username, id)?;
        user.username = username.trim().to_owned();
        user.updated_at = now();
        self.persist(user)
    }

    pub fn list(&self) -> Result<Vec<User>> {
        let mut users = self.store.users().all()?;
        users.sort_by_key(|user| user.username.to_lowercase());
        Ok(users)
    }

    pub fn current(&self) -> Option<User> {
        self.session().clone()
    }

    pub fn logout(&self) {
        *self.session() = None;
    }

    pub fn can(&self, permission: Permission) -> bool {
        self.session()
            .as_ref()
            .is_some_and(|user| user.has_permission(permission))
    }

    fn validate_username(&self, username: &str, exclude: &str) -> Result<()> {
        let name = username.trim();
        if name.is_empty() {
            return rule("El usuario es obligatorio");
        }
        if name.chars().count() < MIN_USERNAME_LENGTH {
            return rule(format!(
                "El usuario debe tener al menos {MIN_USERNAME_LENGTH} caracteres"
            ));
        }
        if name.contains(char::is_whitespace) {
            return rule("El usuario no puede contener espacios");
        }
        let lowered = name.to_lowercase();
        let taken = self
            .store
            .users()
            .all()?
            .iter()
            .any(|user| user.id != exclude && user.username.to_lowercase() == lowered);
        if taken {
            return rule("Ese nombre de usuario ya esta en uso");
        }
        Ok(())
    }

    fn require(&self, id: &str) -> Result<User> {
        match self.store.users().find(id)? {
            Some(user) => Ok(user),
            None => rule("El usuario no existe"),
        }
    }

    fn store_password(&self, mut user: User, plain: &str) -> Result<()> {
        user.set_password(plain);
        user.is_default = false;
        self.persist(user)?;
        Ok(())
    }

    fn persist(&self, user: User) -> Result<User> {
        self.store.users().upsert(&user)?;
        if self.is_current(&user.id) {
            *self.session() = Some(user.clone());
        }
        Ok(user)
    }

    fn active_admins(&self) -> Result<usize> {
        let users = self.store.users().all()?;
        Ok(users.iter().filter(|u| u.is_admin() && u.active).count())
    }

    fn is_current(&self, id: &str) -> bool {
        self.session().as_ref().is_some_and(|user| user.id == id)
    }

    fn session(&self) -> MutexGuard<'_, Option<User>> {
        self.current.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn apply(user: &mut User, draft: &UserDraft) -> Result<()> {
    let permissions = if draft.role.is_locked() {
        draft.role.permissions()
    } else {
        Permission::normalize(draft.permissions.iter().copied())
    };
    user.full_name = required_name(&draft.full_name)?;
    if permissions.is_empty() {
        return rule("Seleccione al menos un permiso");
    }
    user.username = draft.username.trim().to_owned();
    user.role = draft.role;
    user.permissions = permissions;
    user.active = draft.active;
    Ok(())
}

fn required_name(full_name: &str) -> Result<String> {
    match full_name.trim() {
        "" => rule("El nombre completo es obligatorio"),
        name => Ok(name.to_owned()),
    }
}
