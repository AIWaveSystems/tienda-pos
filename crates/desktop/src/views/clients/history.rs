use eframe::egui::Context;
use pos_domain::{Client, Sale};
use pos_services::report::SalesSummary;
use pos_services::{format, report, Result};

use crate::components::dialog::{Dialog, DialogEvent};
use crate::components::table::{left, right, table, Column, Row};
use crate::components::{card, loading, title};
use crate::env::Env;
use crate::task::Task;
use crate::theme;

const COLUMNS: [Column; 4] = [
    left("Venta", 110.0),
    left("Fecha", 150.0),
    right("Items", 70.0),
    right("Total", 120.0),
];

pub struct ClientHistory {
    client: Client,
    task: Option<Task<Vec<Sale>>>,
    sales: Vec<Sale>,
}

impl ClientHistory {
    pub fn new(ctx: &Context, env: &Env, client: Client) -> Self {
        let services = env.services.clone();
        let id = client.id.clone();
        let task = Task::spawn(ctx, move || -> Result<Vec<Sale>> {
            let mut sales = services.sales.list()?;
            sales.retain(|sale| sale.client_id == id && !sale.is_voided());
            sales.sort_by_key(|sale| std::cmp::Reverse(sale.created_at));
            Ok(sales)
        });
        Self {
            client,
            task: Some(task),
            sales: Vec::new(),
        }
    }

    pub fn show(&mut self, ctx: &Context, env: &mut Env) -> bool {
        if let Some(outcome) = Task::poll(&mut self.task) {
            self.sales = env.check(outcome).unwrap_or_default();
        }
        let name = format!("Compras de {}", self.client.name);
        let dialog = Dialog {
            title: &name,
            width: 560.0,
            accept: "Aceptar",
            cancel: "Cerrar",
            error: "",
        };
        let event = dialog.show(ctx, |ui| {
            if self.task.is_some() {
                loading::loading(ui, "Cargando compras...");
                return;
            }
            summary(ui, &report::summarize(&self.sales));
            if self.sales.is_empty() {
                title::hint(ui, "El cliente no tiene compras");
                return;
            }
            let rows: Vec<Row> = self.sales.iter().map(cells).collect();
            ui.allocate_ui([ui.available_width(), 260.0].into(), |ui| {
                table(ui, "client_sales", &COLUMNS, &rows, None, 226.0);
            });
        });
        event == DialogEvent::None
    }
}

fn summary(ui: &mut eframe::egui::Ui, summary: &SalesSummary) {
    let colors = theme::palette();
    card::stats(
        ui,
        &[
            ("Compras", summary.count.to_string(), colors.accent),
            ("Total", format::amount(summary.total), colors.success),
            ("Promedio", format::amount(summary.average), colors.purple),
        ],
    );
}

fn cells(sale: &Sale) -> Row {
    Row::new(vec![
        sale.short_id().to_owned(),
        format::minute(sale.created_at),
        sale.items.len().to_string(),
        format::amount(sale.total),
    ])
}
