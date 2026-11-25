use std::str::FromStr;

use eframe::egui::Context;
use pos_domain::{CashEntry, CashKind, PaymentMethod};
use rust_decimal::Decimal;

use crate::components::dialog::{Dialog, DialogEvent};
use crate::components::{field, select};
use crate::env::Env;

const INCOME: [&str; 4] = ["Base de caja", "Abono de cliente", "Prestamo", "Otro"];
const EXPENSE: [&str; 7] = [
    "Proveedores",
    "Servicios",
    "Arriendo",
    "Nomina",
    "Transporte",
    "Retiro",
    "Otro",
];

pub struct CashForm {
    kind: CashKind,
    concept: String,
    amount: String,
    category: usize,
    method: usize,
    error: String,
    focused: bool,
}

impl CashForm {
    pub fn new(kind: CashKind) -> Self {
        Self {
            kind,
            concept: String::new(),
            amount: String::new(),
            category: 0,
            method: 0,
            error: String::new(),
            focused: false,
        }
    }

    fn categories(&self) -> Vec<String> {
        let names: &[&str] = match self.kind {
            CashKind::Entrada => &INCOME,
            CashKind::Gasto => &EXPENSE,
        };
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    pub fn show(&mut self, ctx: &Context, env: &Env) -> Option<bool> {
        let name = match self.kind {
            CashKind::Entrada => "Nueva entrada de dinero",
            CashKind::Gasto => "Nuevo gasto",
        };
        let methods: Vec<String> = PaymentMethod::ALL
            .iter()
            .map(|method| method.label().to_owned())
            .collect();
        let categories = self.categories();
        let error = self.error.clone();
        match Dialog::form(name, 420.0, &error).show(ctx, |ui| {
            let concept = field::text(ui, "Concepto *", &mut self.concept);
            if !std::mem::replace(&mut self.focused, true) {
                concept.request_focus();
            }
            field::text(ui, "Valor *", &mut self.amount);
            select::labelled(ui, "Categoria", &categories, &mut self.category);
            select::labelled(ui, "Metodo", &methods, &mut self.method);
        }) {
            DialogEvent::None => None,
            DialogEvent::Cancel => Some(false),
            DialogEvent::Accept => self.save(env, &categories),
        }
    }

    fn save(&mut self, env: &Env, categories: &[String]) -> Option<bool> {
        let Ok(amount) = Decimal::from_str(&self.amount.trim().replace(',', ".")) else {
            self.error = "El valor debe ser un numero".to_owned();
            return None;
        };
        let entry = CashEntry {
            kind: self.kind,
            concept: self.concept.clone(),
            category: categories[self.category].clone(),
            amount,
            payment_method: PaymentMethod::ALL[self.method],
            author: env.author(),
            ..CashEntry::default()
        };
        match env.services.cash.register(entry) {
            Ok(_) => Some(true),
            Err(failure) => {
                self.error = failure.to_string();
                None
            }
        }
    }
}
