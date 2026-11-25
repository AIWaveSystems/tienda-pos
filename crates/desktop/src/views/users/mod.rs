mod form;
mod reset;

use eframe::egui::{Context, Ui};
use pos_domain::User;

use super::view::View;
use crate::components::confirm::Confirm;
use crate::components::list::{List, ListAction};
use crate::components::table::{left, right, Column, Row};
use crate::components::{button, title};
use crate::env::Env;
use crate::task::Task;
use crate::theme;
use form::UserForm;
use reset::ResetDialog;

const COLUMNS: [Column; 5] = [
    left("Usuario", 140.0),
    left("Nombre", 200.0),
    left("Rol", 130.0),
    right("Permisos", 80.0),
    left("Estado", 120.0),
];

pub struct UsersView {
    list: List<User>,
    form: Option<UserForm>,
    reset: Option<ResetDialog>,
    deleting: Option<(Confirm, String)>,
}

impl UsersView {
    pub fn new(ctx: &Context, env: &Env) -> Self {
        let mut view = Self {
            list: List::new("users"),
            form: None,
            reset: None,
            deleting: None,
        };
        view.refresh(ctx, env);
        view
    }

    fn refresh(&mut self, ctx: &Context, env: &Env) {
        let services = env.services.clone();
        self.list
            .load(Task::spawn(ctx, move || services.auth.list()));
    }

    fn perform(&mut self, action: ListAction, ctx: &Context, env: &mut Env) {
        match action {
            ListAction::Refresh => self.refresh(ctx, env),
            ListAction::Print => {}
            ListAction::Open => self.form = self.list.require(env).map(UserForm::edit),
            ListAction::Delete => {
                self.deleting = self.list.require(env).map(|user| {
                    let text = format!(
                        "Se eliminara la cuenta '{}'.\nEsta accion no se puede deshacer.",
                        user.username
                    );
                    (Confirm::new("Eliminar usuario", text), user.id)
                });
            }
        }
    }

    fn toggle(&mut self, ctx: &Context, env: &mut Env) {
        if let Some(user) = self.list.require(env) {
            let outcome = env.services.auth.set_active(&user.id, !user.active);
            if env.check(outcome).is_some() {
                self.refresh(ctx, env);
            }
        }
    }

    fn dialogs(&mut self, ctx: &Context, env: &mut Env) {
        if let Some(form) = &mut self.form {
            if let Some(saved) = form.show(ctx, env) {
                self.form = None;
                if saved {
                    self.refresh(ctx, env);
                }
            }
        }
        if let Some(reset) = &mut self.reset {
            if !reset.show(ctx, env) {
                self.reset = None;
            }
        }
        let Some((confirm, id)) = &self.deleting else {
            return;
        };
        if let Some(accepted) = confirm.show(ctx) {
            if accepted {
                let outcome = env.services.auth.delete_user(id);
                env.check(outcome);
                self.refresh(ctx, env);
            }
            self.deleting = None;
        }
    }
}

impl View for UsersView {
    fn show(&mut self, ui: &mut Ui, env: &mut Env) {
        self.list.poll(env);
        title::title_bar(ui, "Usuarios", |ui| {
            if button::accent(ui, "+ Nuevo Usuario") {
                self.form = Some(UserForm::create());
            }
        });
        self.list.search(ui, "Buscar", f32::INFINITY);
        let current = env
            .services
            .auth
            .current()
            .map(|user| user.id)
            .unwrap_or_default();
        let query = self.list.query.trim().to_lowercase();
        let items = std::mem::take(&mut self.list.items);
        let matches: Vec<&User> = items.iter().filter(|user| found(user, &query)).collect();
        let row = |user: &User| cells(user, &current);
        let mut action = self.list.show(ui, &COLUMNS, &matches, row, 90.0);
        let total = matches.len();
        self.list.items = items;
        let ctx = ui.ctx().clone();
        ui.horizontal(|ui| {
            if button::plain(ui, "Editar") {
                action = Some(ListAction::Open);
            }
            if button::plain(ui, "Activar / Desactivar") {
                self.toggle(&ctx, env);
            }
            if button::plain(ui, "Restablecer contrasena") {
                self.reset = self.list.require(env).map(ResetDialog::new);
            }
            if button::danger(ui, "Eliminar") {
                action = Some(ListAction::Delete);
            }
            title::hint(ui, &format!("{total} usuario(s)"));
        });
        if let Some(action) = action {
            self.perform(action, &ctx, env);
        }
        self.dialogs(&ctx, env);
    }
}

fn found(user: &User, query: &str) -> bool {
    user.username.to_lowercase().contains(query)
        || user.full_name.to_lowercase().contains(query)
        || user.role.label().to_lowercase().contains(query)
}

fn cells(user: &User, current: &str) -> Row {
    let (state, color) = if user.id == current {
        ("Sesion actual", Some(theme::palette().success))
    } else if user.active {
        ("Activo", None)
    } else {
        ("Inactivo", Some(theme::palette().subtle))
    };
    Row {
        cells: vec![
            user.username.clone(),
            user.full_name.clone(),
            user.role.label().to_owned(),
            user.effective_permissions().len().to_string(),
            state.to_owned(),
        ],
        color,
    }
}
