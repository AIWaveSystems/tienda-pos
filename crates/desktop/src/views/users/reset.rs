use eframe::egui::Context;
use pos_domain::User;

use crate::components::dialog::{Dialog, DialogEvent};
use crate::components::{field, title};
use crate::env::Env;

pub struct ResetDialog {
    user: User,
    new: String,
    confirmation: String,
    error: String,
}

impl ResetDialog {
    pub fn new(user: User) -> Self {
        Self {
            user,
            new: String::new(),
            confirmation: String::new(),
            error: String::new(),
        }
    }

    pub fn show(&mut self, ctx: &Context, env: &mut Env) -> bool {
        match Dialog::form("Restablecer contrasena", 400.0, &self.error).show(ctx, |ui| {
            title::hint(ui, &format!("Cuenta: {}", self.user.username));
            field::password(ui, "Nueva contrasena", &mut self.new);
            field::password(ui, "Confirmar contrasena", &mut self.confirmation);
        }) {
            DialogEvent::None => true,
            DialogEvent::Cancel => false,
            DialogEvent::Accept => self.save(env),
        }
    }

    fn save(&mut self, env: &mut Env) -> bool {
        if self.new != self.confirmation {
            self.error = "Las contrasenas no coinciden".to_owned();
            return true;
        }
        if let Err(failure) = env.services.auth.reset_password(&self.user.id, &self.new) {
            self.error = failure.to_string();
            return true;
        }
        env.toast(format!(
            "Contrasena de '{}' actualizada.",
            self.user.username
        ));
        false
    }
}
