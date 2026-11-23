use eframe::egui::{Context, Key, ScrollArea, Ui};
use pos_services::report::Overview;
use pos_services::{format, print};

use super::view::View;
use crate::components::table::{left, right, table, Column, Row};
use crate::components::{button, card, loading, title};
use crate::env::Env;
use crate::task::Task;
use crate::{input, theme};

const TOP: [Column; 3] = [
    left("Producto", 240.0),
    right("Unidades", 100.0),
    right("Ingresos", 130.0),
];

const DAILY: [Column; 3] = [
    left("Fecha", 160.0),
    right("Ventas", 100.0),
    right("Total", 130.0),
];

pub struct ReportsView {
    task: Option<Task<Overview>>,
    printing: Option<Task<Overview>>,
    overview: Option<Overview>,
}

impl ReportsView {
    pub fn new(ctx: &Context, env: &Env) -> Self {
        Self {
            task: Some(fetch(ctx, env, 14)),
            printing: None,
            overview: None,
        }
    }

    fn poll(&mut self, env: &mut Env) {
        if let Some(outcome) = Task::poll(&mut self.task) {
            self.overview = env.check(outcome);
        }
        if let Some(outcome) = Task::poll(&mut self.printing) {
            if let Some(overview) = env.check(outcome) {
                env.print(
                    print::overview_report(&overview, &env.config),
                    "Ventas, inventario y productos",
                );
            }
        }
    }
}

impl View for ReportsView {
    fn show(&mut self, ui: &mut Ui, env: &mut Env) {
        self.poll(env);
        let ctx = ui.ctx().clone();
        let free = !input::modal_open(&ctx);
        let mut refresh = free && input::pressed(&ctx, Key::F5);
        let mut print = free && input::command(&ctx, Key::P);
        title::title_bar(ui, "Informes", |ui| {
            print |= button::accent(ui, "Imprimir");
            refresh |= button::plain(ui, "Actualizar");
        });
        if refresh {
            self.task = Some(fetch(&ctx, env, 14));
        }
        if print && self.printing.is_none() {
            self.printing = Some(fetch(&ctx, env, 30));
        }
        if self.task.is_some() {
            loading::loading(ui, "Cargando informes...");
        } else if let Some(overview) = &self.overview {
            ScrollArea::vertical().show(ui, |ui| render(ui, overview));
        }
    }
}

fn fetch(ctx: &Context, env: &Env, days: i64) -> Task<Overview> {
    let services = env.services.clone();
    Task::spawn(ctx, move || services.reports.overview(10, days))
}

fn render(ui: &mut Ui, overview: &Overview) {
    title::heading(ui, "Resumen de Ventas");
    card::stats(
        ui,
        &[
            (
                "Total Ventas",
                format::amount(overview.sales.total),
                theme::ACCENT,
            ),
            (
                "Nro. Ventas",
                overview.sales.count.to_string(),
                theme::SUCCESS,
            ),
            (
                "Promedio",
                format::amount(overview.sales.average),
                theme::PURPLE,
            ),
        ],
    );
    title::heading(ui, "Estado del Inventario");
    card::stats(
        ui,
        &[
            (
                "Productos",
                overview.inventory.products.to_string(),
                theme::ACCENT,
            ),
            (
                "Stock Bajo",
                overview.inventory.low_stock.len().to_string(),
                theme::DANGER,
            ),
            (
                "Valor Inventario",
                format::amount(overview.inventory.value),
                theme::SUCCESS,
            ),
        ],
    );
    title::heading(ui, "Productos Mas Vendidos");
    let top = overview
        .top
        .iter()
        .map(|item| {
            Row::new(vec![
                item.name.clone(),
                item.quantity.to_string(),
                format::amount(item.revenue),
            ])
        })
        .collect();
    section(ui, "top", &TOP, top, "No hay ventas registradas");
    title::heading(ui, "Ventas Diarias (ultimos 14 dias)");
    let daily = overview
        .daily
        .iter()
        .filter(|day| day.count > 0)
        .map(|day| {
            Row::new(vec![
                day.date.clone(),
                day.count.to_string(),
                format::amount(day.total),
            ])
        })
        .collect();
    section(
        ui,
        "daily",
        &DAILY,
        daily,
        "No hay ventas en los ultimos 14 dias",
    );
}

fn section(ui: &mut Ui, id: &str, columns: &[Column], rows: Vec<Row>, empty: &str) {
    if rows.is_empty() {
        title::hint(ui, empty);
        return;
    }
    let height = 34.0 * rows.len().clamp(1, 6) as f32 + 40.0;
    ui.allocate_ui([ui.available_width(), height].into(), |ui| {
        table(ui, id, columns, &rows, None, height - 34.0);
    });
}
