use eframe::egui::{Context, Ui};
use pos_domain::StockMovement;
use pos_services::{format, inventory, print};

use super::view::View;
use crate::components::list::{List, ListAction};
use crate::components::table::{left, right, Column, Row};
use crate::components::{button, title};
use crate::env::Env;
use crate::task::Task;
use crate::theme;

const COLUMNS: [Column; 6] = [
    left("Producto", 180.0),
    left("Fecha", 140.0),
    left("Tipo", 100.0),
    right("Cantidad", 90.0),
    left("Referencia", 160.0),
    left("Autor", 130.0),
];

pub struct MovementsView {
    list: List<StockMovement>,
}

impl MovementsView {
    pub fn new(ctx: &Context, env: &Env) -> Self {
        let mut view = Self {
            list: List::new("movements"),
        };
        view.refresh(ctx, env);
        view
    }

    fn refresh(&mut self, ctx: &Context, env: &Env) {
        let services = env.services.clone();
        self.list
            .load(Task::spawn(ctx, move || services.inventory.movements()));
    }

    fn print(&self, env: &mut Env) {
        let rows: Vec<Vec<String>> = self
            .list
            .items
            .iter()
            .filter(|item| inventory::movement_matches(item, &self.list.query))
            .map(|item| cells(item).cells)
            .collect();
        let detail = format!("{} registro(s)", rows.len());
        let listing = print::Listing {
            title: "Movimientos de inventario".to_owned(),
            columns: COLUMNS
                .iter()
                .map(|column| print::Column {
                    label: column.title,
                    numeric: column.right,
                })
                .collect(),
            rows,
            ..print::Listing::default()
        };
        env.print(print::table(&listing, &env.config), detail);
    }
}

impl View for MovementsView {
    fn show(&mut self, ui: &mut Ui, env: &mut Env) {
        self.list.poll(env);
        title::title_bar(ui, "Movimientos", |_| {});
        self.list.search(
            ui,
            "Buscar por producto, tipo, referencia o autor",
            f32::INFINITY,
        );
        let items = std::mem::take(&mut self.list.items);
        let matches: Vec<&StockMovement> = items
            .iter()
            .filter(|item| inventory::movement_matches(item, &self.list.query))
            .collect();
        let mut action = self.list.show(ui, &COLUMNS, &matches, cells, 90.0);
        let total = matches.len();
        self.list.items = items;
        ui.horizontal(|ui| {
            if button::plain(ui, "Imprimir") {
                action = Some(ListAction::Print);
            }
            title::hint(ui, &format!("{total} movimiento(s)"));
        });
        match action {
            Some(ListAction::Refresh) => self.refresh(ui.ctx(), env),
            Some(ListAction::Print) => self.print(env),
            _ => {}
        }
    }
}

fn cells(item: &StockMovement) -> Row {
    let color = if item.quantity < 0 {
        theme::DANGER
    } else {
        theme::SUCCESS
    };
    Row {
        cells: vec![
            item.product_name.clone(),
            format::minute(item.created_at),
            item.kind.label().to_owned(),
            format!("{:+}", item.quantity),
            item.reference.clone(),
            item.author.clone(),
        ],
        color: Some(color),
    }
}
