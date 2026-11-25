use eframe::egui::{Context, Grid, RichText, Ui};
use pos_domain::Sale;
use pos_services::{format, print};

use crate::components::card;
use crate::components::dialog::{Dialog, DialogEvent};
use crate::components::table::{left, right, table, Column, Row};
use crate::env::Env;
use crate::theme;

const COLUMNS: [Column; 4] = [
    left("Producto", 170.0),
    right("Precio U.", 90.0),
    right("Cant.", 60.0),
    right("Subtotal", 100.0),
];

pub struct DetailDialog {
    sale: Sale,
}

impl DetailDialog {
    pub fn new(sale: Sale) -> Self {
        Self { sale }
    }

    pub fn show(&self, ctx: &Context, env: &mut Env) -> bool {
        let title = format!("Venta #{}", self.sale.short_id());
        let dialog = Dialog {
            title: &title,
            width: 500.0,
            accept: "Imprimir comprobante",
            cancel: "Cerrar",
            error: "",
        };
        match dialog.show(ctx, |ui| {
            self.facts(ui);
            self.items(ui);
            self.totals(ui);
        }) {
            DialogEvent::None => true,
            DialogEvent::Cancel => false,
            DialogEvent::Accept => {
                let detail = format!("Venta del {}", self.sale.date());
                env.print(print::receipt(&self.sale, &env.config), detail);
                true
            }
        }
    }

    fn facts(&self, ui: &mut Ui) {
        Grid::new("sale_facts").num_columns(2).show(ui, |ui| {
            for (label, value) in [
                ("Fecha:", format::minute(self.sale.created_at)),
                ("Cliente:", self.sale.client_name.clone()),
                (
                    "Metodo de pago:",
                    self.sale.payment_method.label().to_owned(),
                ),
                ("Vendedor:", self.sale.seller.clone()),
                ("Estado:", self.state()),
            ] {
                ui.label(RichText::new(label).strong().color(theme::palette().muted));
                ui.label(value);
                ui.end_row();
            }
        });
    }

    fn state(&self) -> String {
        match self.sale.voided_at {
            Some(moment) => format!("Anulada el {}", format::minute(moment)),
            None => "Activa".to_owned(),
        }
    }

    fn items(&self, ui: &mut Ui) {
        let rows: Vec<Row> = self
            .sale
            .items
            .iter()
            .map(|item| {
                Row::new(vec![
                    item.product_name.clone(),
                    format::amount(item.unit_price),
                    item.quantity.to_string(),
                    format::amount(item.subtotal),
                ])
            })
            .collect();
        let height = 34.0 * rows.len().clamp(1, 6) as f32 + 40.0;
        ui.allocate_ui([ui.available_width(), height].into(), |ui| {
            table(ui, "sale_items", &COLUMNS, &rows, None, height - 34.0);
        });
    }

    fn totals(&self, ui: &mut Ui) {
        card::surface(ui, |ui| {
            Grid::new("sale_totals").num_columns(2).show(ui, |ui| {
                for (label, value, color) in [
                    ("Subtotal:", self.sale.subtotal, theme::palette().muted),
                    ("Impuesto:", self.sale.tax, theme::palette().muted),
                    ("Descuento:", -self.sale.discount, theme::palette().danger),
                    ("Total:", self.sale.total, theme::palette().success),
                ] {
                    ui.label(label);
                    ui.label(RichText::new(format::amount(value)).color(color));
                    ui.end_row();
                }
            });
        });
    }
}
