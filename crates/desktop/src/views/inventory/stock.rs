use eframe::egui::Context;
use pos_domain::Product;

use crate::components::dialog::{Dialog, DialogEvent};
use crate::components::field;
use crate::env::Env;

pub struct StockDialog {
    product: Product,
    delta: String,
    error: String,
}

impl StockDialog {
    pub fn new(product: Product) -> Self {
        Self {
            product,
            delta: "0".to_owned(),
            error: String::new(),
        }
    }

    pub fn show(&mut self, ctx: &Context, env: &Env) -> Option<bool> {
        match Dialog::form("Ajustar Stock", 360.0, &self.error).show(ctx, |ui| {
            ui.label(format!(
                "{}: {} unidades",
                self.product.name, self.product.stock
            ));
            field::text(ui, "Cantidad (+/-)", &mut self.delta);
        }) {
            DialogEvent::None => None,
            DialogEvent::Cancel => Some(false),
            DialogEvent::Accept => self.save(env),
        }
    }

    fn save(&mut self, env: &Env) -> Option<bool> {
        let Ok(delta) = self.delta.trim().parse::<i64>() else {
            self.error = "Ingrese un numero entero".to_owned();
            return None;
        };
        match env.services.inventory.adjust_stock(&self.product.id, delta) {
            Ok(_) => Some(true),
            Err(failure) => {
                self.error = failure.to_string();
                None
            }
        }
    }
}
