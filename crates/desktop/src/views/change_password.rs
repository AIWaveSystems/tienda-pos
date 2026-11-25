use eframe::egui::{Key, RichText, Ui};

use crate::components::{button, field, title};
use crate::env::Env;
use crate::theme;

pub struct ChangePasswordView {
    user_id: String,
    current: String,
    new: String,
    confirmation: String,
    error: String,
}

impl ChangePasswordView {
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            current: String::new(),
            new: String::new(),
            confirmation: String::new(),
            error: String::new(),
        }
    }

    pub fn show(&mut self, ui: &mut Ui, env: &mut Env) -> bool {
        let mut done = false;
        ui.add_space(ui.available_height() * 0.12);
        ui.vertical_centered(|ui| {
            ui.set_max_width(380.0);
            ui.label(
                RichText::new("Cambio de contrasena requerido")
                    .size(20.0)
                    .strong()
                    .color(theme::palette().warning),
            );
            title::hint(
                ui,
                "Esta usando la contrasena predeterminada del sistema. \
                 Definala de nuevo para continuar.",
            );
            ui.add_space(12.0);
            ui.vertical(|ui| {
                field::password(ui, "Contrasena actual", &mut self.current);
                field::password(ui, "Nueva contrasena", &mut self.new);
                let last = field::password(ui, "Confirmar contrasena", &mut self.confirmation);
                let submitted =
                    last.lost_focus() && ui.input(|state| state.key_pressed(Key::Enter));
                ui.add_space(8.0);
                if button::wide_accent(ui, "Guardar contrasena") || submitted {
                    done = self.submit(env);
                }
                title::status(ui, &self.error, theme::palette().danger);
            });
        });
        done
    }

    fn submit(&mut self, env: &mut Env) -> bool {
        if self.current.is_empty() || self.new.is_empty() || self.confirmation.is_empty() {
            self.error = "Complete todos los campos".to_owned();
            return false;
        }
        if self.new != self.confirmation {
            self.error = "Las contrasenas no coinciden".to_owned();
            return false;
        }
        let outcome = env
            .services
            .auth
            .change_password(&self.user_id, &self.current, &self.new);
        if let Err(failure) = outcome {
            self.error = failure.to_string();
            return false;
        }
        env.toast("Su contrasena se guardo correctamente.");
        true
    }
}
