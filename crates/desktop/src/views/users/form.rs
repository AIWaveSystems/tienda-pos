use eframe::egui::{Context, Ui};
use pos_domain::{Permission, Role, User};
use pos_services::{AuthService, UserDraft};

use crate::components::dialog::{Dialog, DialogEvent};
use crate::components::{card, field, select, title};
use crate::env::Env;

pub struct UserForm {
    id: Option<String>,
    draft: UserDraft,
    role: usize,
    password: String,
    confirmation: String,
    error: String,
}

impl UserForm {
    pub fn create() -> Self {
        Self::build(
            None,
            UserDraft {
                username: String::new(),
                full_name: String::new(),
                role: Role::Vendedor,
                permissions: Role::Vendedor.permissions(),
                active: true,
            },
        )
    }

    pub fn edit(user: User) -> Self {
        let permissions = user.effective_permissions();
        Self::build(
            Some(user.id),
            UserDraft {
                username: user.username,
                full_name: user.full_name,
                role: user.role,
                permissions,
                active: user.active,
            },
        )
    }

    fn build(id: Option<String>, draft: UserDraft) -> Self {
        Self {
            id,
            role: Role::ALL
                .iter()
                .position(|role| *role == draft.role)
                .unwrap_or_default(),
            draft,
            password: String::new(),
            confirmation: String::new(),
            error: String::new(),
        }
    }

    pub fn show(&mut self, ctx: &Context, env: &Env) -> Option<bool> {
        let name = if self.id.is_some() {
            "Editar Usuario"
        } else {
            "Nuevo Usuario"
        };
        let error = self.error.clone();
        match Dialog::form(name, 500.0, &error).show(ctx, |ui| self.fields(ui)) {
            DialogEvent::None => None,
            DialogEvent::Cancel => Some(false),
            DialogEvent::Accept => match self.save(env) {
                Ok(()) => Some(true),
                Err(failure) => {
                    self.error = failure;
                    None
                }
            },
        }
    }

    fn fields(&mut self, ui: &mut Ui) {
        field::text(ui, "Nombre completo *", &mut self.draft.full_name);
        field::text(ui, "Usuario *", &mut self.draft.username);
        let label = if self.id.is_some() {
            "Nueva contrasena (dejar vacio para no cambiarla)"
        } else {
            "Contrasena *"
        };
        field::password(ui, label, &mut self.password);
        field::password(ui, "Confirmar contrasena", &mut self.confirmation);
        ui.separator();
        let roles: Vec<String> = Role::ALL
            .iter()
            .map(|role| role.label().to_owned())
            .collect();
        if select::labelled(ui, "Rol", &roles, &mut self.role) {
            self.change_role();
        }
        title::hint(ui, self.draft.role.description());
        ui.checkbox(
            &mut self.draft.active,
            "Usuario activo (puede iniciar sesion)",
        );
        title::heading(ui, "Permisos de acceso");
        ui.add_enabled_ui(!self.draft.role.is_locked(), |ui| {
            for permission in Permission::ALL {
                self.permission(ui, permission);
            }
        });
    }

    fn permission(&mut self, ui: &mut Ui, permission: Permission) {
        let mut granted = self.draft.permissions.contains(&permission);
        if card::check(
            ui,
            &mut granted,
            permission.label(),
            permission.description(),
        ) {
            self.draft.permissions.retain(|item| *item != permission);
            if granted {
                self.draft.permissions.push(permission);
            }
        }
    }

    fn change_role(&mut self) {
        self.draft.role = Role::ALL[self.role];
        if self.draft.role.is_locked() {
            self.draft.permissions = self.draft.role.permissions();
        } else if self.draft.permissions.is_empty() {
            self.draft.permissions.push(Permission::Dashboard);
        }
    }

    fn save(&self, env: &Env) -> Result<(), String> {
        if self.password != self.confirmation {
            return Err("Las contrasenas no coinciden".to_owned());
        }
        let auth = &env.services.auth;
        let Some(id) = &self.id else {
            return auth
                .create_user(&self.draft, &self.password)
                .map(|_| ())
                .map_err(|failure| failure.to_string());
        };
        if !self.password.is_empty() {
            AuthService::validate_password(&self.password)
                .map_err(|failure| failure.to_string())?;
        }
        auth.update_user(id, &self.draft)
            .map_err(|failure| failure.to_string())?;
        if self.password.is_empty() {
            return Ok(());
        }
        auth.reset_password(id, &self.password)
            .map_err(|failure| failure.to_string())
    }
}
