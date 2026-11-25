use eframe::egui::{Context, Key, ScrollArea, Ui};
use pos_services::report::Overview;
use pos_services::{format, print, Period};
use rust_decimal::prelude::ToPrimitive;

use super::view::View;
use crate::components::chart::{bars, Bar};
use crate::components::table::{left, right, table, Column, Row};
use crate::components::{button, card, loading, title};
use crate::env::Env;
use crate::input;
use crate::task::Task;
use crate::theme;

const TOP: [Column; 3] = [
    left("Producto", 240.0),
    right("Unidades", 100.0),
    right("Ingresos", 130.0),
];

const SELLERS: [Column; 3] = [
    left("Vendedor", 240.0),
    right("Ventas", 100.0),
    right("Total", 130.0),
];

pub struct ReportsView {
    period: Period,
    task: Option<Task<Overview>>,
    overview: Option<Overview>,
}

impl ReportsView {
    pub fn new(ctx: &Context, env: &Env) -> Self {
        let period = Period::default();
        Self {
            period,
            task: Some(fetch(ctx, env, period)),
            overview: None,
        }
    }

    fn periods(&mut self, ui: &mut Ui) -> bool {
        let before = self.period;
        ui.horizontal(|ui| {
            for period in Period::ALL {
                ui.selectable_value(&mut self.period, period, period.label());
            }
        });
        before != self.period
    }
}

impl View for ReportsView {
    fn show(&mut self, ui: &mut Ui, env: &mut Env) {
        if let Some(outcome) = Task::poll(&mut self.task) {
            self.overview = env.check(outcome);
        }
        let ctx = ui.ctx().clone();
        let free = !input::modal_open(&ctx);
        let mut refresh = free && input::pressed(&ctx, Key::F5);
        let mut print = free && input::command(&ctx, Key::P);
        title::title_bar(ui, "Informes", |ui| {
            print |= button::accent(ui, "Imprimir");
            refresh |= button::plain(ui, "Actualizar");
        });
        refresh |= self.periods(ui);
        if refresh {
            self.task = Some(fetch(&ctx, env, self.period));
        }
        if self.task.is_some() {
            loading::loading(ui, "Cargando informes...");
            return;
        }
        let Some(overview) = &self.overview else {
            return;
        };
        if print {
            let document = print::overview_report(overview, &env.config);
            env.print(document, format!("Periodo: {}", self.period.label()));
        }
        ScrollArea::vertical().show(ui, |ui| render(ui, overview));
    }
}

fn fetch(ctx: &Context, env: &Env, period: Period) -> Task<Overview> {
    let services = env.services.clone();
    Task::spawn(ctx, move || services.reports.overview(10, period))
}

fn render(ui: &mut Ui, overview: &Overview) {
    let colors = theme::palette();
    title::heading(ui, "Resumen de Ventas");
    card::stats(
        ui,
        &[
            (
                "Total Ventas",
                format::amount(overview.sales.total),
                colors.accent,
            ),
            (
                "Nro. Ventas",
                overview.sales.count.to_string(),
                colors.success,
            ),
            (
                "Promedio",
                format::amount(overview.sales.average),
                colors.purple,
            ),
            (
                "Utilidad estimada",
                format::amount(overview.profit),
                colors.warning,
            ),
        ],
    );
    title::heading(ui, "Ventas por dia");
    if overview.sales.count == 0 {
        title::hint(ui, "No hay ventas en el periodo");
    } else {
        card::surface(ui, |ui| bars(ui, &daily(overview), 150.0));
    }
    title::heading(ui, "Estado del Inventario");
    card::stats(
        ui,
        &[
            (
                "Productos",
                overview.inventory.products.to_string(),
                colors.accent,
            ),
            (
                "Stock Bajo",
                overview.inventory.low_stock.len().to_string(),
                colors.danger,
            ),
            (
                "Valor Inventario",
                format::amount(overview.inventory.value),
                colors.success,
            ),
        ],
    );
    title::heading(ui, "Productos Mas Vendidos");
    let top = overview
        .top
        .iter()
        .map(|item| ranked(&item.name, item.quantity.to_string(), item.revenue))
        .collect();
    section(ui, "top", &TOP, top);
    title::heading(ui, "Ventas por vendedor");
    let sellers = overview
        .sellers
        .iter()
        .map(|item| ranked(&item.name, item.count.to_string(), item.total))
        .collect();
    section(ui, "sellers", &SELLERS, sellers);
}

pub fn daily(overview: &Overview) -> Vec<Bar> {
    let step = overview.daily.len().div_ceil(15);
    overview
        .daily
        .iter()
        .enumerate()
        .map(|(index, day)| Bar {
            label: if index % step == 0 {
                day.date.get(5..).unwrap_or_default().to_owned()
            } else {
                String::new()
            },
            value: day.total.to_f32().unwrap_or_default(),
            detail: format!(
                "{}: {} ({} ventas)",
                day.date,
                format::amount(day.total),
                day.count
            ),
        })
        .collect()
}

fn ranked(name: &str, count: String, total: rust_decimal::Decimal) -> Row {
    Row::new(vec![name.to_owned(), count, format::amount(total)])
}

fn section(ui: &mut Ui, id: &str, columns: &[Column], rows: Vec<Row>) {
    if rows.is_empty() {
        title::hint(ui, "No hay ventas en el periodo");
        return;
    }
    let height = 34.0 * rows.len().clamp(1, 6) as f32 + 40.0;
    ui.allocate_ui([ui.available_width(), height].into(), |ui| {
        table(ui, id, columns, &rows, None, height - 34.0);
    });
}
