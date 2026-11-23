use eframe::egui::Context;
use pos_domain::SaleItem;
use pos_services::format;

use super::cart::Cart;
use crate::components::dialog::{Dialog, DialogEvent};
use crate::components::{field, title};

pub struct QuantityDialog {
    index: usize,
    item: SaleItem,
    limit: Option<i64>,
    value: String,
    error: String,
}

impl QuantityDialog {
    pub fn new(index: usize, item: SaleItem, limit: Option<i64>) -> Self {
        Self {
            index,
            value: item.quantity.to_string(),
            item,
            limit,
            error: String::new(),
        }
    }

    pub fn show(&mut self, ctx: &Context, cart: &mut Cart) -> bool {
        let dialog = Dialog {
            title: &self.item.product_name,
            width: 360.0,
            accept: "Aceptar",
            cancel: "Cancelar",
            error: &self.error,
        };
        let event = dialog.show(ctx, |ui| {
            let mut detail = format!("Precio unitario: {}", format::amount(self.item.unit_price));
            if let Some(limit) = self.limit {
                detail.push_str(&format!("   |   Stock disponible: {limit}"));
            }
            title::hint(ui, &detail);
            field::text(ui, "Cantidad", &mut self.value);
            title::hint(ui, "Escriba 0 para quitar el producto del carrito.");
        });
        match event {
            DialogEvent::None => false,
            DialogEvent::Cancel => true,
            DialogEvent::Accept => match self.apply(cart) {
                Ok(()) => true,
                Err(message) => {
                    self.error = message;
                    false
                }
            },
        }
    }

    fn apply(&self, cart: &mut Cart) -> Result<(), String> {
        let quantity: i64 = self
            .value
            .trim()
            .parse()
            .map_err(|_| "Escriba un numero entero valido".to_owned())?;
        if quantity < 0 {
            return Err("La cantidad no puede ser negativa".to_owned());
        }
        cart.set_quantity(self.index, quantity, self.limit)
    }
}
