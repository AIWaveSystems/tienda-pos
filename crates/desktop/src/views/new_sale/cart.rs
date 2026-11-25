use eframe::egui::{Align, Button, Frame, Label, Layout, Margin, RichText, Sense, Ui};
use pos_domain::{Product, SaleItem};
use pos_services::format;

use crate::components::title;
use crate::theme;

pub enum CartEvent {
    Select(usize),
    Step(usize, i64),
    Edit(usize),
    Remove(usize),
}

#[derive(Default)]
pub struct Cart {
    pub items: Vec<SaleItem>,
}

impl Cart {
    pub fn units(&self) -> i64 {
        self.items.iter().map(|item| item.quantity).sum()
    }

    pub fn add(&mut self, product: &Product, allow_negative: bool) -> Result<usize, String> {
        if product.stock <= 0 && !allow_negative {
            return Err(format!("'{}' no tiene stock disponible", product.name));
        }
        let existing = self
            .items
            .iter()
            .position(|item| item.product_id == product.id);
        if let Some(index) = existing {
            let limit = (!allow_negative).then_some(product.stock);
            let quantity = self.items[index].quantity + 1;
            self.set_quantity(index, quantity, limit)?;
            return Ok(index);
        }
        self.items
            .push(SaleItem::new(&product.id, &product.name, 1, product.price));
        Ok(self.items.len() - 1)
    }

    pub fn set_quantity(
        &mut self,
        index: usize,
        quantity: i64,
        limit: Option<i64>,
    ) -> Result<(), String> {
        if quantity <= 0 {
            self.items.remove(index);
            return Ok(());
        }
        let item = &mut self.items[index];
        if let Some(limit) = limit.filter(|limit| quantity > *limit) {
            return Err(format!(
                "Solo hay {limit} unidad(es) de '{}'.",
                item.product_name
            ));
        }
        item.quantity = quantity;
        item.calculate();
        Ok(())
    }

    pub fn show(&self, ui: &mut Ui, selected: Option<usize>) -> Option<CartEvent> {
        if self.items.is_empty() {
            title::hint(ui, "Elija productos del catalogo o escanee un codigo");
        }
        let mut event = None;
        for (index, item) in self.items.iter().enumerate() {
            event = line(ui, index, item, selected == Some(index)).or(event);
        }
        event
    }
}

fn line(ui: &mut Ui, index: usize, item: &SaleItem, selected: bool) -> Option<CartEvent> {
    let colors = theme::palette();
    let fill = if selected {
        colors.surface_alt
    } else {
        colors.surface
    };
    let mut event = None;
    Frame::new()
        .fill(fill)
        .corner_radius(6)
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let name = Label::new(RichText::new(&item.product_name).strong())
                .sense(Sense::click())
                .selectable(false)
                .truncate();
            let response = ui.add(name);
            if response.double_clicked() {
                event = Some(CartEvent::Edit(index));
            } else if response.clicked() {
                event = Some(CartEvent::Select(index));
            }
            ui.horizontal(|ui| {
                if ui.small_button("-").clicked() {
                    event = Some(CartEvent::Step(index, -1));
                }
                let quantity = Button::new(item.quantity.to_string()).small();
                if ui.add(quantity).on_hover_text("Editar cantidad").clicked() {
                    event = Some(CartEvent::Edit(index));
                }
                if ui.small_button("+").clicked() {
                    event = Some(CartEvent::Step(index, 1));
                }
                title::hint(ui, &format!("x {}", format::amount(item.unit_price)));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.small_button("x").on_hover_text("Quitar").clicked() {
                        event = Some(CartEvent::Remove(index));
                    }
                    ui.label(format::amount(item.subtotal));
                });
            });
        });
    event
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanning_twice_increments_until_stock() {
        let product = Product {
            stock: 2,
            price: 10.into(),
            ..Product::default()
        };
        let mut cart = Cart::default();
        cart.add(&product, false).unwrap();
        cart.add(&product, false).unwrap();
        assert!(cart.add(&product, false).is_err());
        assert_eq!(cart.units(), 2);
        assert_eq!(cart.items[0].subtotal, 20.into());
        assert!(cart.add(&product, true).is_ok());
        cart.set_quantity(0, 0, None).unwrap();
        assert!(cart.items.is_empty());
    }
}
