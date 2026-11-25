use eframe::egui::{Context, RichText, ScrollArea, Ui};
use pos_domain::Product;
use pos_services::report::Overview;
use pos_services::{format, Period, Result};

use super::reports::daily;
use super::view::View;
use crate::components::chart::bars;
use crate::components::{card, loading, title};
use crate::env::Env;
use crate::task::Task;
use crate::theme;

struct Summary {
    today: Overview,
    recent: Overview,
    clients: usize,
}

pub struct DashboardView {
    task: Option<Task<Summary>>,
    summary: Option<Summary>,
    failure: String,
}

impl DashboardView {
    pub fn new(ctx: &Context, env: &Env) -> Self {
        let services = env.services.clone();
        let task = Task::spawn(ctx, move || -> Result<Summary> {
            Ok(Summary {
                today: services.reports.overview(5, Period::Today)?,
                recent: services.reports.overview(5, Period::Fortnight)?,
                clients: services.clients.count()?,
            })
        });
        Self {
            task: Some(task),
            summary: None,
            failure: String::new(),
        }
    }
}

impl View for DashboardView {
    fn show(&mut self, ui: &mut Ui, env: &mut Env) {
        match Task::poll(&mut self.task) {
            Some(Ok(summary)) => self.summary = Some(summary),
            Some(Err(failure)) => self.failure = failure,
            None => {}
        }
        title::title(
            ui,
            "Dashboard",
            &format!("Resumen de {}", env.config.business_name()),
        );
        if self.task.is_some() {
            loading::loading(ui, "Cargando resumen...");
        }
        title::status(ui, &self.failure, theme::palette().danger);
        if let Some(summary) = &self.summary {
            ScrollArea::vertical().show(ui, |ui| render(ui, summary, env));
        }
    }
}

fn render(ui: &mut Ui, summary: &Summary, env: &Env) {
    let colors = theme::palette();
    let currency = &env.config.general.currency;
    card::stats(
        ui,
        &[
            (
                "Ventas de hoy",
                summary.today.sales.count.to_string(),
                colors.warning,
            ),
            (
                "Ingresos de hoy",
                format::money(summary.today.sales.total, currency),
                colors.success,
            ),
            (
                "Ingresos 14 dias",
                format::money(summary.recent.sales.total, currency),
                colors.purple,
            ),
            (
                "Productos / Clientes",
                format!("{} / {}", summary.today.inventory.products, summary.clients),
                colors.accent,
            ),
        ],
    );
    title::heading(ui, "Ultimos 14 dias");
    if summary.recent.sales.count == 0 {
        title::hint(ui, "Aun no hay ventas en este periodo");
    } else {
        card::surface(ui, |ui| bars(ui, &daily(&summary.recent), 150.0));
    }
    if !summary.recent.top.is_empty() {
        title::heading(ui, "Mas vendidos");
        card::surface(ui, |ui| {
            for item in &summary.recent.top {
                ui.label(format!(
                    "{}  -  {} unidades, {}",
                    item.name,
                    item.quantity,
                    format::amount(item.revenue)
                ));
            }
        });
    }
    if env.config.inventory.low_stock_on_dashboard {
        low_stock(ui, &summary.today.inventory.low_stock);
    }
}

fn low_stock(ui: &mut Ui, products: &[Product]) {
    if products.is_empty() {
        return;
    }
    title::heading(ui, &format!("Stock bajo ({})", products.len()));
    card::surface(ui, |ui| {
        for product in products.iter().take(6) {
            ui.label(
                RichText::new(format!(
                    "{}  -  {} unidades (minimo {})",
                    product.name, product.stock, product.min_stock
                ))
                .color(theme::palette().warning),
            );
        }
        if products.len() > 6 {
            title::hint(ui, &format!("y {} producto(s) mas", products.len() - 6));
        }
    });
}
