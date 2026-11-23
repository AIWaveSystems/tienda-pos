use eframe::egui::Context;
use pos_domain::User;
use pos_services::AuthService;

use crate::components::dialog::{Dialog, DialogEvent};
use crate::components::{field, title};
use crate::env::Env;

pub struct ProfileDialog {
    user: User,
    current: String,
    new: String,
    confirmation: String,
    error: String,
}

impl ProfileDialog {
    pub fn open(env: &Env) -> Option<Self> {
        Some(Self {
            user: env.services.auth.current()?,
            current: String::new(),
            new: String::new(),
            confirmation: String::new(),
            error: String::new(),
        })
    }

    pub fn show(&mut self, ctx: &Context, env: &mut Env) -> bool {
        match Dialog::form("Mi Perfil", 440.0, &self.error).show(ctx, |ui| {
            title::hint(ui, &format!("Rol: {}", self.user.role.label()));
            field::text(ui, "Nombre completo", &mut self.user.full_name);
            field::text(ui, "Usuario", &mut self.user.username);
            ui.separator();
            title::heading(ui, "Cambiar contrasena");
            title::hint(ui, "Deje los campos vacios si no desea cambiarla.");
            field::password(ui, "Contrasena actual", &mut self.current);
            field::password(ui, "Nueva contrasena", &mut self.new);
            field::password(ui, "Confirmar nueva contrasena", &mut self.confirmation);
        }) {
            DialogEvent::None => true,
            DialogEvent::Cancel => false,
            DialogEvent::Accept => match self.save(env) {
                Ok(()) => {
                    env.info(
                        "Perfil actualizado",
                        "Sus datos se guardaron correctamente.",
                    );
                    false
                }
                Err(failure) => {
                    self.error = failure;
                    true
                }
            },
        }
    }

    fn save(&self, env: &Env) -> Result<(), String> {
        let auth = &env.services.auth;
        let changing = [&self.current, &self.new, &self.confirmation]
            .into_iter()
            .any(|value| !value.is_empty());
        if changing {
            self.validate()?;
        }
        auth.update_profile(&self.user.id, &self.user.full_name, &self.user.username)
            .map_err(|failure| failure.to_string())?;
        if changing {
            auth.change_password(&self.user.id, &self.current, &self.new)
                .map_err(|failure| failure.to_string())?;
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        if self.current.is_empty() {
            return Err("Ingrese su contrasena actual para cambiarla".to_owned());
        }
        if self.new != self.confirmation {
            return Err("Las contrasenas nuevas no coinciden".to_owned());
        }
        AuthService::validate_password(&self.new).map_err(|failure| failure.to_string())?;
        if !self.user.check_password(&self.current) {
            return Err("La contrasena actual es incorrecta".to_owned());
        }
        if self.current == self.new {
            return Err("La nueva contrasena debe ser diferente a la actual".to_owned());
        }
        Ok(())
    }
}
