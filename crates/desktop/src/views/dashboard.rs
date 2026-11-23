use eframe::egui::{Context, RichText, Ui};
use pos_domain::{Product, Sale};
use pos_services::{format, report, Result};

use super::view::View;
use crate::components::{card, loading, title};
use crate::env::Env;
use crate::task::Task;
use crate::theme;

struct Summary {
    products: Vec<Product>,
    sales: Vec<Sale>,
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
                products: services.inventory.list(false)?,
                sales: services.sales.list()?,
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
        title::status(ui, &self.failure, theme::DANGER);
        if let Some(summary) = &self.summary {
            render(ui, summary, env);
        }
    }
}

fn render(ui: &mut Ui, summary: &Summary, env: &Env) {
    let revenue = report::summarize(&summary.sales).total;
    card::stats(
        ui,
        &[
            (
                "Productos",
                summary.products.len().to_string(),
                theme::ACCENT,
            ),
            ("Clientes", summary.clients.to_string(), theme::SUCCESS),
            ("Ventas", summary.sales.len().to_string(), theme::WARNING),
            (
                "Ingresos",
                format::money(revenue, &env.config.general.currency),
                theme::PURPLE,
            ),
        ],
    );
    if !env.config.inventory.low_stock_on_dashboard {
        return;
    }
    let low: Vec<&Product> = summary.products.iter().filter(|p| p.low_stock()).collect();
    if low.is_empty() {
        return;
    }
    ui.add_space(12.0);
    card::surface(ui, |ui| {
        ui.label(
            RichText::new(format!("Stock bajo ({})", low.len()))
                .strong()
                .color(theme::WARNING),
        );
        for product in low.iter().take(6) {
            ui.label(format!(
                "{}  -  {} unidades (minimo {})",
                product.name, product.stock, product.min_stock
            ));
        }
        if low.len() > 6 {
            title::hint(ui, &format!("y {} producto(s) mas", low.len() - 6));
        }
    });
}
