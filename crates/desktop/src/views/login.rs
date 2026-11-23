use eframe::egui::{Key, RichText, Ui};
use pos_domain::User;

use crate::components::{button, field, title};
use crate::env::Env;
use crate::task::Task;
use crate::{input, theme};

#[derive(Default)]
pub struct LoginView {
    username: String,
    password: String,
    error: String,
    checking: Option<Task<Option<User>>>,
    focused: bool,
}

impl LoginView {
    pub fn show(&mut self, ui: &mut Ui, env: &mut Env) -> Option<User> {
        let mut user = None;
        match Task::poll(&mut self.checking) {
            Some(Ok(Some(found))) => user = Some(found),
            Some(Ok(None)) => {
                self.error = "Usuario o contrasena incorrectos".to_owned();
                self.password.clear();
            }
            Some(Err(failure)) => self.error = failure,
            None => {}
        }
        ui.add_space(ui.available_height() * 0.16);
        ui.vertical_centered(|ui| {
            ui.set_max_width(340.0);
            ui.label(
                RichText::new("Ventas POS")
                    .size(30.0)
                    .strong()
                    .color(theme::ACCENT),
            );
            title::hint(ui, "Sistema de punto de venta e inventario");
            ui.add_space(18.0);
            ui.vertical(|ui| self.form(ui, env));
        });
        user
    }

    fn form(&mut self, ui: &mut Ui, env: &Env) {
        let username = field::text(ui, "Usuario", &mut self.username);
        let password = field::password(ui, "Contrasena", &mut self.password);
        if !std::mem::replace(&mut self.focused, true) {
            username.request_focus();
        }
        let entered = ui.input(|state| state.key_pressed(Key::Enter));
        if username.lost_focus() && entered {
            password.request_focus();
        }
        let submitted = password.lost_focus() && entered;
        ui.add_space(8.0);
        let idle = self.checking.is_none();
        let clicked = ui
            .add_enabled_ui(idle, |ui| button::wide_accent(ui, "Ingresar"))
            .inner;
        if idle && !input::modal_open(ui.ctx()) && (clicked || submitted) {
            self.submit(ui, env);
        }
        title::status(ui, &self.error, theme::DANGER);
    }

    fn submit(&mut self, ui: &Ui, env: &Env) {
        if self.username.trim().is_empty() || self.password.is_empty() {
            self.error = "Ingrese usuario y contrasena".to_owned();
            return;
        }
        self.error.clear();
        let services = env.services.clone();
        let (username, password) = (self.username.clone(), self.password.clone());
        self.checking = Some(Task::spawn(ui.ctx(), move || {
            services.auth.authenticate(&username, &password)
        }));
    }
}
