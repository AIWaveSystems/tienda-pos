mod form;
mod history;

use eframe::egui::{Context, Ui};
use pos_domain::Client;
use pos_services::{client, print};

use super::view::View;
use crate::components::confirm::Confirm;
use crate::components::list::{List, ListAction};
use crate::components::table::{left, Column, Row};
use crate::components::{button, title};
use crate::env::Env;
use crate::task::Task;
use form::ClientForm;
use history::ClientHistory;

const COLUMNS: [Column; 4] = [
    left("Nombre", 180.0),
    left("Email", 190.0),
    left("Telefono", 120.0),
    left("NIT / ID Fiscal", 130.0),
];

pub struct ClientsView {
    list: List<Client>,
    form: Option<ClientForm>,
    history: Option<ClientHistory>,
    deleting: Option<(Confirm, String)>,
}

impl ClientsView {
    pub fn new(ctx: &Context, env: &Env) -> Self {
        let mut view = Self {
            list: List::new("clients"),
            form: None,
            history: None,
            deleting: None,
        };
        view.refresh(ctx, env);
        view
    }

    fn refresh(&mut self, ctx: &Context, env: &Env) {
        let services = env.services.clone();
        self.list
            .load(Task::spawn(ctx, move || services.clients.list(false)));
    }

    fn perform(&mut self, action: ListAction, ctx: &Context, env: &mut Env) {
        match action {
            ListAction::Refresh => self.refresh(ctx, env),
            ListAction::Print => self.print(env),
            ListAction::Open => self.form = self.list.require(env).map(ClientForm::edit),
            ListAction::Delete => {
                self.deleting = self.list.require(env).map(|item| {
                    let text = format!(
                        "¿Esta seguro de eliminar a '{}'?\nEsta accion no se puede deshacer.",
                        item.name
                    );
                    (Confirm::new("Eliminar cliente", text), item.id)
                });
            }
        }
    }

    fn print(&self, env: &mut Env) {
        let rows: Vec<Vec<String>> = self
            .list
            .items
            .iter()
            .filter(|item| client::matches(item, &self.list.query))
            .map(|item| cells(item).cells)
            .collect();
        let detail = format!("{} registro(s)", rows.len());
        let listing = print::Listing {
            title: "Listado de Clientes".to_owned(),
            columns: COLUMNS
                .iter()
                .map(|column| print::Column::text(column.title))
                .collect(),
            rows,
            ..print::Listing::default()
        };
        env.print(print::table(&listing, &env.config), detail);
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
        if let Some(history) = &mut self.history {
            if !history.show(ctx, env) {
                self.history = None;
            }
        }
        let Some((confirm, id)) = &self.deleting else {
            return;
        };
        if let Some(accepted) = confirm.show(ctx) {
            if accepted {
                let outcome = env.services.clients.delete(id);
                env.check(outcome);
                self.refresh(ctx, env);
            }
            self.deleting = None;
        }
    }
}

impl View for ClientsView {
    fn show(&mut self, ui: &mut Ui, env: &mut Env) {
        self.list.poll(env);
        title::title_bar(ui, "Clientes", |ui| {
            if button::accent(ui, "+ Nuevo Cliente") {
                self.form = Some(ClientForm::create());
            }
        });
        self.list.search(ui, "Buscar", f32::INFINITY);
        let items = std::mem::take(&mut self.list.items);
        let matches: Vec<&Client> = items
            .iter()
            .filter(|item| client::matches(item, &self.list.query))
            .collect();
        let mut action = self.list.show(ui, &COLUMNS, &matches, cells, 90.0);
        let total = matches.len();
        self.list.items = items;
        ui.horizontal(|ui| {
            for (text, choice) in [
                ("Editar", ListAction::Open),
                ("Eliminar", ListAction::Delete),
                ("Imprimir", ListAction::Print),
            ] {
                if button::plain(ui, text) {
                    action = Some(choice);
                }
            }
            if button::plain(ui, "Compras") {
                self.history = self
                    .list
                    .require(env)
                    .map(|client| ClientHistory::new(ui.ctx(), env, client));
            }
            title::hint(ui, &format!("{total} cliente(s)"));
        });
        if let Some(action) = action {
            self.perform(action, ui.ctx(), env);
        }
        self.dialogs(&ui.ctx().clone(), env);
    }
}

fn cells(item: &Client) -> Row {
    Row::new(vec![
        item.name.clone(),
        item.email.clone(),
        item.phone.clone(),
        item.tax_id.clone(),
    ])
}
