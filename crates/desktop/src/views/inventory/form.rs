use std::str::FromStr;

use eframe::egui::{Context, Ui};
use pos_domain::{margin, Category, Product};
use pos_services::format;
use rust_decimal::Decimal;

use crate::components::dialog::{Dialog, DialogEvent};
use crate::components::{field, select, title};
use crate::env::Env;
use crate::theme;

const UNCATEGORIZED: &str = "(Sin categoria)";

pub struct ProductForm {
    item: Product,
    editing: bool,
    price: String,
    cost: String,
    stock: String,
    min_stock: String,
    categories: Vec<String>,
    category_ids: Vec<String>,
    category: usize,
    error: String,
}

impl ProductForm {
    pub fn create(min_stock: i64, categories: &[Category]) -> Self {
        let item = Product {
            min_stock,
            ..Product::default()
        };
        Self::build(item, categories, false)
    }

    pub fn edit(item: Product, categories: &[Category]) -> Self {
        Self::build(item, categories, true)
    }

    fn build(item: Product, categories: &[Category], editing: bool) -> Self {
        let mut names = vec![UNCATEGORIZED.to_owned()];
        let mut ids = vec![String::new()];
        for category in categories {
            if category.active || category.id == item.category_id {
                names.push(category.name.clone());
                ids.push(category.id.clone());
            }
        }
        Self {
            price: item.price.normalize().to_string(),
            cost: item.cost.normalize().to_string(),
            stock: item.stock.to_string(),
            min_stock: item.min_stock.to_string(),
            category: ids
                .iter()
                .position(|id| *id == item.category_id)
                .unwrap_or_default(),
            categories: names,
            category_ids: ids,
            item,
            editing,
            error: String::new(),
        }
    }

    pub fn show(&mut self, ctx: &Context, env: &Env) -> Option<bool> {
        let name = if self.editing {
            "Editar Producto"
        } else {
            "Nuevo Producto"
        };
        let error = self.error.clone();
        let currency = env.config.general.currency.clone();
        match Dialog::form(name, 480.0, &error).show(ctx, |ui| self.fields(ui, &currency)) {
            DialogEvent::None => None,
            DialogEvent::Cancel => Some(false),
            DialogEvent::Accept => match self.save(env) {
                Ok(()) => Some(true),
                Err(failure) => {
                    self.error = failure;
                    None
                }
            },
        }
    }

    fn fields(&mut self, ui: &mut Ui, currency: &str) {
        title::hint(ui, "Los campos marcados con * son obligatorios.");
        field::text(ui, "Nombre *", &mut self.item.name);
        field::text(ui, "Descripcion", &mut self.item.description);
        field::text(ui, "Codigo de barras", &mut self.item.barcode);
        select::labelled(ui, "Categoria", &self.categories, &mut self.category);
        ui.columns(2, |columns| {
            let price = format!("Precio de venta * ({currency})");
            field::text(&mut columns[0], &price, &mut self.price);
            field::text(
                &mut columns[1],
                &format!("Costo ({currency})"),
                &mut self.cost,
            );
            columns[0].add_enabled_ui(!self.editing, |ui| {
                field::text(ui, "Stock actual", &mut self.stock);
            });
            field::text(&mut columns[1], "Stock minimo", &mut self.min_stock);
        });
        if self.editing {
            title::hint(ui, "El stock se cambia con Ajustar Stock.");
        }
        self.margin(ui);
        if self.editing {
            ui.checkbox(&mut self.item.active, "Producto activo");
        }
    }

    fn margin(&self, ui: &mut Ui) {
        let (Ok(price), Ok(cost)) = (decimal(&self.price), decimal(&self.cost)) else {
            return;
        };
        if price <= Decimal::ZERO || cost <= Decimal::ZERO {
            return;
        }
        let color = if price >= cost {
            theme::SUCCESS
        } else {
            theme::DANGER
        };
        let text = format!(
            "Margen: {:.1}%   Utilidad por unidad: {}",
            margin(price, cost).round_dp(1),
            format::amount(price - cost)
        );
        title::status(ui, &text, color);
    }

    fn save(&mut self, env: &Env) -> Result<(), String> {
        self.item.price = decimal(&self.price).map_err(|_| invalid("El precio"))?;
        self.item.cost = decimal(&self.cost).map_err(|_| invalid("El costo"))?;
        self.item.stock = whole(&self.stock).map_err(|_| invalid("El stock"))?;
        self.item.min_stock = whole(&self.min_stock).map_err(|_| invalid("El stock minimo"))?;
        if self.item.min_stock < 0 {
            return Err("El stock minimo no puede ser negativo".to_owned());
        }
        self.item.category_id = self.category_ids[self.category].clone();
        env.services
            .inventory
            .save(self.item.clone())
            .map(|_| ())
            .map_err(|failure| failure.to_string())
    }
}

fn invalid(label: &str) -> String {
    format!("{label} debe ser un numero valido")
}

fn decimal(text: &str) -> Result<Decimal, rust_decimal::Error> {
    match text.trim() {
        "" => Ok(Decimal::ZERO),
        value => Decimal::from_str(&value.replace(',', ".")),
    }
}

fn whole(text: &str) -> Result<i64, std::num::ParseIntError> {
    match text.trim() {
        "" => Ok(0),
        value => value.parse(),
    }
}
