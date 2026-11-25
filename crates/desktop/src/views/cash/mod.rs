mod form;

use eframe::egui::{Context, Ui};
use pos_domain::{CashEntry, CashKind};
use pos_services::{cash, format, print};
use rust_decimal::Decimal;

use super::view::View;
use crate::components::confirm::Confirm;
use crate::components::list::{List, ListAction};
use crate::components::table::{left, right, Column, Row};
use crate::components::{button, card, title};
use crate::env::Env;
use crate::task::Task;
use crate::theme;
use form::CashForm;

const COLUMNS: [Column; 8] = [
    left("Concepto", 180.0),
    left("Fecha", 140.0),
    left("Tipo", 80.0),
    left("Categoria", 110.0),
    left("Metodo", 110.0),
    right("Valor", 110.0),
    left("Autor", 110.0),
    left("Estado", 80.0),
];

pub struct CashView {
    list: List<CashEntry>,
    form: Option<CashForm>,
    voiding: Option<(Confirm, String)>,
}

impl CashView {
    pub fn new(ctx: &Context, env: &Env) -> Self {
        let mut view = Self {
            list: List::new("cash"),
            form: None,
            voiding: None,
        };
        view.refresh(ctx, env);
        view
    }

    fn refresh(&mut self, ctx: &Context, env: &Env) {
        let services = env.services.clone();
        self.list
            .load(Task::spawn(ctx, move || services.cash.list()));
    }

    fn found(&self) -> Vec<CashEntry> {
        self.list
            .items
            .iter()
            .filter(|entry| cash::matches(entry, &self.list.query))
            .cloned()
            .collect()
    }

    fn print(&self, env: &mut Env) {
        let found = self.found();
        let (income, expense) = totals(&found);
        let listing = print::Listing {
            title: "Entradas y gastos".to_owned(),
            columns: COLUMNS
                .iter()
                .map(|column| print::Column {
                    label: column.title,
                    numeric: column.right,
                })
                .collect(),
            rows: found.iter().map(|entry| cells(entry).cells).collect(),
            summary: vec![
                ("Entradas".to_owned(), format::amount(income)),
                ("Gastos".to_owned(), format::amount(expense)),
                ("Balance".to_owned(), format::amount(income - expense)),
            ],
            ..print::Listing::default()
        };
        let detail = format!("{} registro(s)", found.len());
        env.print(print::table(&listing, &env.config), detail);
    }

    fn ask_void(&mut self, env: &mut Env) {
        self.voiding = self.list.require(env).map(|entry| {
            let text = format!(
                "Se anulara '{}' por {}.\nEsta accion no se puede deshacer.",
                entry.concept,
                format::amount(entry.amount)
            );
            (Confirm::new("Anular movimiento", text), entry.id)
        });
    }

    fn dialogs(&mut self, ctx: &Context, env: &mut Env) {
        if let Some(form) = &mut self.form {
            if let Some(saved) = form.show(ctx, env) {
                self.form = None;
                if saved {
                    env.toast("Movimiento de caja registrado");
                    self.refresh(ctx, env);
                }
            }
        }
        let Some((confirm, id)) = &self.voiding else {
            return;
        };
        if let Some(accepted) = confirm.show(ctx) {
            if accepted {
                let outcome = env.services.cash.void(id);
                env.check(outcome);
                self.refresh(ctx, env);
            }
            self.voiding = None;
        }
    }
}

impl View for CashView {
    fn show(&mut self, ui: &mut Ui, env: &mut Env) {
        self.list.poll(env);
        title::title_bar(ui, "Caja", |ui| {
            if button::accent(ui, "+ Entrada") {
                self.form = Some(CashForm::new(CashKind::Entrada));
            }
            if button::danger(ui, "+ Gasto") {
                self.form = Some(CashForm::new(CashKind::Gasto));
            }
        });
        let found = self.found();
        summary(ui, &found);
        self.list.search(
            ui,
            "Buscar por concepto, categoria, metodo o autor",
            f32::INFINITY,
        );
        let matches: Vec<&CashEntry> = found.iter().collect();
        let mut action = self.list.show(ui, &COLUMNS, &matches, cells, 90.0);
        ui.horizontal(|ui| {
            if button::plain(ui, "Imprimir") {
                action = Some(ListAction::Print);
            }
            if env.is_admin() && button::danger(ui, "Anular") {
                action = Some(ListAction::Delete);
            }
            title::hint(ui, &format!("{} movimiento(s)", found.len()));
        });
        match action {
            Some(ListAction::Refresh) => self.refresh(ui.ctx(), env),
            Some(ListAction::Print) => self.print(env),
            Some(ListAction::Delete) if env.is_admin() => self.ask_void(env),
            _ => {}
        }
        self.dialogs(&ui.ctx().clone(), env);
    }
}

fn totals(entries: &[CashEntry]) -> (Decimal, Decimal) {
    (
        cash::total(entries, CashKind::Entrada),
        cash::total(entries, CashKind::Gasto),
    )
}

fn summary(ui: &mut Ui, entries: &[CashEntry]) {
    let colors = theme::palette();
    let (income, expense) = totals(entries);
    card::stats(
        ui,
        &[
            ("Entradas", format::amount(income), colors.success),
            ("Gastos", format::amount(expense), colors.danger),
            ("Balance", format::amount(income - expense), colors.accent),
        ],
    );
}

fn cells(entry: &CashEntry) -> Row {
    let colors = theme::palette();
    let (state, color) = if entry.is_voided() {
        ("Anulado", colors.subtle)
    } else if entry.kind == CashKind::Entrada {
        ("Activo", colors.success)
    } else {
        ("Activo", colors.danger)
    };
    Row {
        cells: vec![
            entry.concept.clone(),
            format::minute(entry.created_at),
            entry.kind.label().to_owned(),
            entry.category.clone(),
            entry.payment_method.label().to_owned(),
            format::amount(entry.signed()),
            entry.author.clone(),
            state.to_owned(),
        ],
        color: Some(color),
    }
}
