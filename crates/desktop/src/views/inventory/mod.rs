mod form;
mod stock;

use std::collections::HashMap;

use eframe::egui::{Context, Ui};
use pos_domain::{Category, Product};
use pos_services::{format, inventory, print};

use super::view::View;
use crate::components::confirm::Confirm;
use crate::components::list::{List, ListAction};
use crate::components::table::{left, right, Column, Row};
use crate::components::{button, title};
use crate::env::Env;
use crate::task::Task;
use crate::theme;
use form::ProductForm;
use stock::StockDialog;

const COLUMNS: [Column; 6] = [
    left("Producto", 180.0),
    left("Categoria", 130.0),
    right("Precio", 110.0),
    right("Stock", 70.0),
    right("Stock Min.", 80.0),
    left("Codigo", 120.0),
];

pub struct InventoryView {
    list: List<Product>,
    categories: Vec<Category>,
    loading: Option<Task<Vec<Category>>>,
    form: Option<ProductForm>,
    stock: Option<StockDialog>,
    deleting: Option<(Confirm, String)>,
}

impl InventoryView {
    pub fn new(ctx: &Context, env: &Env) -> Self {
        let mut view = Self {
            list: List::new("products"),
            categories: Vec::new(),
            loading: None,
            form: None,
            stock: None,
            deleting: None,
        };
        view.refresh(ctx, env);
        view
    }

    fn refresh(&mut self, ctx: &Context, env: &Env) {
        let services = env.services.clone();
        self.list
            .load(Task::spawn(ctx, move || services.inventory.list(false)));
        let services = env.services.clone();
        self.loading = Some(Task::spawn(ctx, move || services.categories.list(true)));
    }

    fn names(&self) -> HashMap<String, String> {
        self.categories
            .iter()
            .map(|category| (category.id.clone(), category.name.clone()))
            .collect()
    }

    fn perform(&mut self, action: ListAction, ctx: &Context, env: &mut Env) {
        match action {
            ListAction::Refresh => self.refresh(ctx, env),
            ListAction::Print => self.print(env),
            ListAction::Open => {
                self.form = self
                    .list
                    .require(env)
                    .map(|item| ProductForm::edit(item, &self.categories));
            }
            ListAction::Delete => {
                self.deleting = self.list.require(env).map(|item| {
                    let text = format!(
                        "¿Esta seguro de eliminar '{}'?\nEsta accion no se puede deshacer.",
                        item.name
                    );
                    (Confirm::new("Eliminar producto", text), item.id)
                });
            }
        }
    }

    fn print(&self, env: &mut Env) {
        let names = self.names();
        let found = inventory::search(&self.list.items, &self.list.query, |id| name(&names, id));
        let value = found.iter().map(|item| item.inventory_value()).sum();
        let low = found.iter().filter(|item| item.low_stock()).count();
        let listing = print::Listing {
            title: "Listado de Inventario".to_owned(),
            columns: COLUMNS
                .iter()
                .map(|column| print::Column {
                    label: column.title,
                    numeric: column.right,
                })
                .collect(),
            rows: found.iter().map(|item| cells(item, &names).cells).collect(),
            summary: vec![
                ("Productos".to_owned(), found.len().to_string()),
                ("Stock bajo".to_owned(), low.to_string()),
                ("Valor inventario".to_owned(), format::amount(value)),
            ],
            ..print::Listing::default()
        };
        let detail = format!("{} registro(s)", found.len());
        env.print(print::table(&listing, &env.config), detail);
    }

    fn dialogs(&mut self, ctx: &Context, env: &mut Env) {
        let mut changed = false;
        if let Some(form) = &mut self.form {
            if let Some(saved) = form.show(ctx, env) {
                self.form = None;
                changed |= saved;
            }
        }
        if let Some(stock) = &mut self.stock {
            if let Some(saved) = stock.show(ctx, env) {
                self.stock = None;
                changed |= saved;
            }
        }
        if let Some((confirm, id)) = &self.deleting {
            if let Some(accepted) = confirm.show(ctx) {
                if accepted {
                    let outcome = env.services.inventory.delete(id);
                    env.check(outcome);
                    changed = true;
                }
                self.deleting = None;
            }
        }
        if changed {
            self.refresh(ctx, env);
        }
    }
}

impl View for InventoryView {
    fn show(&mut self, ui: &mut Ui, env: &mut Env) {
        self.list.poll(env);
        if let Some(outcome) = Task::poll(&mut self.loading) {
            self.categories = env.check(outcome).unwrap_or_default();
        }
        title::title_bar(ui, "Inventario", |ui| {
            if button::accent(ui, "+ Nuevo Producto") {
                let minimum = env.config.inventory.min_stock_alert;
                self.form = Some(ProductForm::create(minimum, &self.categories));
            }
        });
        self.list.search(ui, "Buscar", f32::INFINITY);
        let names = self.names();
        let items = std::mem::take(&mut self.list.items);
        let matches = inventory::search(&items, &self.list.query, |id| name(&names, id));
        let row = |item: &Product| cells(item, &names);
        let mut action = self.list.show(ui, &COLUMNS, &matches, row, 90.0);
        let total = matches.len();
        self.list.items = items;
        ui.horizontal(|ui| {
            if button::plain(ui, "Editar") {
                action = Some(ListAction::Open);
            }
            if button::plain(ui, "Ajustar Stock") {
                self.stock = self.list.require(env).map(StockDialog::new);
            }
            if button::plain(ui, "Eliminar") {
                action = Some(ListAction::Delete);
            }
            if button::plain(ui, "Imprimir") {
                action = Some(ListAction::Print);
            }
            title::hint(ui, &format!("{total} producto(s)"));
        });
        if let Some(action) = action {
            self.perform(action, ui.ctx(), env);
        }
        self.dialogs(&ui.ctx().clone(), env);
    }
}

fn name(names: &HashMap<String, String>, id: &str) -> String {
    names.get(id).cloned().unwrap_or_default()
}

fn cells(item: &Product, names: &HashMap<String, String>) -> Row {
    Row {
        cells: vec![
            item.name.clone(),
            name(names, &item.category_id),
            format::amount(item.price),
            item.stock.to_string(),
            item.min_stock.to_string(),
            item.barcode.clone(),
        ],
        color: item.low_stock().then_some(theme::palette().warning),
    }
}
