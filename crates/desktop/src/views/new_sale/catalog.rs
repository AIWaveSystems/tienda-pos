use eframe::egui::{Button, Key, RichText, ScrollArea, TextWrapMode, Ui, Vec2};
use pos_domain::{Category, Product};
use pos_services::{format, inventory};

use crate::components::{field, title};
use crate::{input, theme};

const CARD: Vec2 = Vec2::new(158.0, 86.0);
const LIMIT: usize = 120;

#[derive(Default)]
pub struct Catalog {
    pub products: Vec<Product>,
    pub categories: Vec<Category>,
    pub query: String,
    category: Option<String>,
    focus: bool,
}

impl Catalog {
    pub fn request_focus(&mut self) {
        self.focus = true;
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.focus = true;
    }

    pub fn show(&mut self, ui: &mut Ui) -> Option<Product> {
        let scanned = self.search(ui);
        self.chips(ui);
        let clicked = self.grid(ui);
        scanned.or(clicked)
    }

    fn search(&mut self, ui: &mut Ui) -> Option<Product> {
        let response = field::named_search(
            ui,
            "sale",
            &mut self.query,
            "Producto o codigo de barras",
            f32::INFINITY,
        );
        if std::mem::take(&mut self.focus) {
            response.request_focus();
        }
        let entered = response.lost_focus() && ui.input(|state| state.key_pressed(Key::Enter));
        if !entered || input::modal_open(ui.ctx()) {
            return None;
        }
        self.focus = true;
        let chosen = self.only_match();
        if chosen.is_some() {
            self.query.clear();
        }
        chosen
    }

    fn only_match(&self) -> Option<Product> {
        if self.query.trim().is_empty() {
            return None;
        }
        let found = inventory::search(&self.products, &self.query, |_| String::new());
        let exact = found
            .iter()
            .find(|product| product.has_barcode(&self.query));
        let single = (found.len() == 1).then(|| &found[0]);
        exact.or(single).map(|product| (*product).clone())
    }

    fn chips(&mut self, ui: &mut Ui) {
        if self.categories.is_empty() {
            return;
        }
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut self.category, None, "Todas");
            for category in &self.categories {
                let value = Some(category.id.clone());
                ui.selectable_value(&mut self.category, value, &category.name);
            }
        });
    }

    fn visible(&self) -> Vec<&Product> {
        inventory::search(&self.products, &self.query, |_| String::new())
            .into_iter()
            .filter(|product| {
                self.category
                    .as_ref()
                    .is_none_or(|category| *category == product.category_id)
            })
            .collect()
    }

    fn grid(&self, ui: &mut Ui) -> Option<Product> {
        let found = self.visible();
        if found.is_empty() {
            title::hint(ui, "No hay productos para mostrar");
            return None;
        }
        let mut chosen = None;
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for product in found.iter().take(LIMIT) {
                        if card(ui, product) {
                            chosen = Some((*product).clone());
                        }
                    }
                });
                if found.len() > LIMIT {
                    title::hint(ui, "Escriba para acotar la busqueda");
                }
            });
        chosen
    }
}

fn card(ui: &mut Ui, product: &Product) -> bool {
    let colors = theme::palette();
    let color = if product.stock <= 0 {
        colors.danger
    } else if product.low_stock() {
        colors.warning
    } else {
        colors.fg
    };
    let text = format!(
        "{}\n${}\nStock: {}",
        shorten(&product.name),
        format::amount(product.price),
        product.stock
    );
    let button = Button::new(RichText::new(text).color(color))
        .fill(colors.surface)
        .wrap_mode(TextWrapMode::Extend)
        .min_size(CARD);
    ui.add_sized(CARD, button)
        .on_hover_text(&product.name)
        .clicked()
}

fn shorten(name: &str) -> String {
    const LENGTH: usize = 17;
    if name.chars().count() <= LENGTH {
        return name.to_owned();
    }
    let start: String = name.chars().take(LENGTH - 1).collect();
    format!("{start}…")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_unambiguous_searches_are_added() {
        let product = |name: &str, barcode: &str| Product {
            name: name.into(),
            barcode: barcode.into(),
            ..Product::default()
        };
        let mut catalog = Catalog {
            products: vec![product("Pan", "12"), product("Panela", "123")],
            ..Catalog::default()
        };
        assert!(catalog.only_match().is_none());
        catalog.query = "pan".into();
        assert!(catalog.only_match().is_none());
        catalog.query = "12".into();
        assert_eq!(catalog.only_match().unwrap().name, "Pan");
        catalog.query = "panela".into();
        assert_eq!(catalog.only_match().unwrap().name, "Panela");
    }
}
