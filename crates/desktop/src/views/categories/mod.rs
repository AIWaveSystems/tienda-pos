mod form;

use eframe::egui::{Context, Ui};
use pos_domain::Category;
use pos_services::{category, print};

use super::view::View;
use crate::components::confirm::Confirm;
use crate::components::list::{List, ListAction};
use crate::components::table::{left, Column, Row};
use crate::components::{button, title};
use crate::env::Env;
use crate::task::Task;
use form::CategoryForm;

const COLUMNS: [Column; 3] = [
    left("Nombre", 180.0),
    left("Descripcion", 320.0),
    left("Estado", 90.0),
];

pub struct CategoriesView {
    list: List<Category>,
    form: Option<CategoryForm>,
    deleting: Option<(Confirm, String)>,
}

impl CategoriesView {
    pub fn new(ctx: &Context, env: &Env) -> Self {
        let mut view = Self {
            list: List::new("categories"),
            form: None,
            deleting: None,
        };
        view.refresh(ctx, env);
        view
    }

    fn refresh(&mut self, ctx: &Context, env: &Env) {
        let services = env.services.clone();
        self.list
            .load(Task::spawn(ctx, move || services.categories.list(true)));
    }

    fn perform(&mut self, action: ListAction, ctx: &Context, env: &mut Env) {
        match action {
            ListAction::Refresh => self.refresh(ctx, env),
            ListAction::Print => self.print(env),
            ListAction::Open => {
                self.form = self.list.require(env).map(CategoryForm::edit);
            }
            ListAction::Delete => {
                self.deleting = self.list.require(env).map(|item| {
                    let text = format!(
                        "¿Esta seguro de eliminar '{}'?\nEsta accion no se puede deshacer.",
                        item.name
                    );
                    (Confirm::new("Eliminar categoria", text), item.id)
                });
            }
        }
    }

    fn print(&self, env: &mut Env) {
        let rows: Vec<Vec<String>> = self
            .list
            .items
            .iter()
            .filter(|item| category::matches(item, &self.list.query))
            .map(|item| cells(item).cells)
            .collect();
        let detail = format!("{} registro(s)", rows.len());
        let listing = print::Listing {
            title: "Listado de Categorias".to_owned(),
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
        let Some((confirm, id)) = &self.deleting else {
            return;
        };
        if let Some(accepted) = confirm.show(ctx) {
            if accepted {
                let outcome = env.services.categories.delete(id);
                env.check(outcome);
                self.refresh(ctx, env);
            }
            self.deleting = None;
        }
    }
}

impl View for CategoriesView {
    fn show(&mut self, ui: &mut Ui, env: &mut Env) {
        self.list.poll(env);
        let mut action = None;
        title::title_bar(ui, "Categorias", |ui| {
            if button::accent(ui, "+ Nueva Categoria") {
                self.form = Some(CategoryForm::create());
            }
        });
        self.list.search(ui, "Buscar", f32::INFINITY);
        let items = std::mem::take(&mut self.list.items);
        let matches: Vec<&Category> = items
            .iter()
            .filter(|item| category::matches(item, &self.list.query))
            .collect();
        action = self
            .list
            .show(ui, &COLUMNS, &matches, cells, 90.0)
            .or(action);
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
            title::hint(ui, &format!("{total} categoria(s)"));
        });
        if let Some(action) = action {
            self.perform(action, ui.ctx(), env);
        }
        self.dialogs(&ui.ctx().clone(), env);
    }
}

fn cells(item: &Category) -> Row {
    let state = if item.active { "Activa" } else { "Inactiva" };
    Row::new(vec![
        item.name.clone(),
        item.description.clone(),
        state.to_owned(),
    ])
}
