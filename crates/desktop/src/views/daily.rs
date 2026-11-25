use std::str::FromStr;

use chrono::{NaiveDate, TimeDelta};
use eframe::egui::{Context, Key, ScrollArea, Ui};
use pos_domain::now;
use pos_services::daily::DailyReport;
use pos_services::{format, print, Span};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

use super::view::View;
use crate::components::chart::{bars, Bar};
use crate::components::table::{left, right, table, Column, Row};
use crate::components::{button, card, field, loading, title};
use crate::env::Env;
use crate::input;
use crate::task::Task;
use crate::theme;

const METHODS: [Column; 5] = [
    left("Metodo", 180.0),
    right("Ventas", 120.0),
    right("Entradas", 120.0),
    right("Gastos", 120.0),
    right("Balance", 120.0),
];

const SALES: [Column; 6] = [
    left("Cliente", 180.0),
    left("Hora", 70.0),
    left("Venta", 100.0),
    left("Vendedor", 130.0),
    left("Pago", 130.0),
    right("Total", 120.0),
];

const ENTRIES: [Column; 5] = [
    left("Concepto", 200.0),
    left("Hora", 70.0),
    left("Tipo", 90.0),
    left("Autor", 130.0),
    right("Valor", 120.0),
];

pub struct DailyView {
    date: String,
    counted: String,
    task: Option<Task<DailyReport>>,
    report: Option<DailyReport>,
}

impl DailyView {
    pub fn new(ctx: &Context, env: &Env) -> Self {
        let mut view = Self {
            date: now().date().to_string(),
            counted: String::new(),
            task: None,
            report: None,
        };
        view.load(ctx, env, Span::today(now()));
        view
    }

    fn load(&mut self, ctx: &Context, env: &Env, span: Span) {
        let services = env.services.clone();
        self.date = span.from.date().to_string();
        self.counted.clear();
        self.task = Some(Task::spawn(ctx, move || services.reports.daily(span)));
    }

    fn open_date(&mut self, ctx: &Context, env: &mut Env, days: i64) {
        let current = now();
        match shifted(&self.date, days) {
            Some(date) if date <= current.date() => self.load(ctx, env, Span::of(date, current)),
            Some(_) => env.warn("Informe diario", "La fecha no puede ser futura"),
            None => env.warn("Informe diario", "Use el formato AAAA-MM-DD"),
        }
    }

    fn toolbar(&mut self, ui: &mut Ui, env: &mut Env) {
        let ctx = ui.ctx().clone();
        ui.horizontal(|ui| {
            if button::plain(ui, "Hoy") {
                self.load(&ctx, env, Span::today(now()));
            }
            if button::plain(ui, "<") {
                self.open_date(&ctx, env, -1);
            }
            field::caption(ui, "Fecha:");
            let response = field::search(ui, &mut self.date, "AAAA-MM-DD", 110.0);
            let entered = response.lost_focus() && ui.input(|state| state.key_pressed(Key::Enter));
            if button::plain(ui, ">") {
                self.open_date(&ctx, env, 1);
            }
            if button::plain(ui, "Consultar") || entered {
                self.open_date(&ctx, env, 0);
            }
        });
    }

    fn count(&mut self, ui: &mut Ui, expected: Decimal) {
        let colors = theme::palette();
        title::heading(ui, "Arqueo de efectivo");
        card::surface(ui, |ui| {
            ui.label(format!("Efectivo esperado: {}", format::amount(expected)));
            field::text(ui, "Efectivo contado", &mut self.counted);
            if self.counted.trim().is_empty() {
                return;
            }
            let Ok(counted) = Decimal::from_str(&self.counted.trim().replace(',', ".")) else {
                title::status(ui, "Escriba un numero valido", colors.danger);
                return;
            };
            let difference = counted - expected;
            let (text, color) = if difference == Decimal::ZERO {
                ("Caja cuadrada".to_owned(), colors.success)
            } else if difference > Decimal::ZERO {
                (
                    format!("Sobrante: {}", format::amount(difference)),
                    colors.warning,
                )
            } else {
                (
                    format!("Faltante: {}", format::amount(-difference)),
                    colors.danger,
                )
            };
            title::status(ui, &text, color);
        });
    }
}

impl View for DailyView {
    fn show(&mut self, ui: &mut Ui, env: &mut Env) {
        if let Some(outcome) = Task::poll(&mut self.task) {
            self.report = env.check(outcome);
        }
        let ctx = ui.ctx().clone();
        let free = !input::modal_open(&ctx);
        let mut printing = free && input::command(&ctx, Key::P);
        if free && input::pressed(&ctx, Key::F5) {
            self.open_date(&ctx, env, 0);
        }
        title::title_bar(ui, "Informe diario", |ui| {
            printing |= button::accent(ui, "Imprimir");
        });
        self.toolbar(ui, env);
        if self.task.is_some() {
            loading::loading(ui, "Cargando informe...");
            return;
        }
        let Some(report) = self.report.take() else {
            return;
        };
        title::hint(ui, &report.span.label());
        if printing {
            let document = print::daily_report(&report, &env.config);
            env.print(document, report.span.label());
        }
        ScrollArea::vertical().show(ui, |ui| {
            totals(ui, &report);
            self.count(ui, report.expected_cash());
            details(ui, &report);
        });
        self.report = Some(report);
    }
}

fn totals(ui: &mut Ui, report: &DailyReport) {
    let colors = theme::palette();
    card::stats(
        ui,
        &[
            (
                "Ventas",
                format!("{} ({} anuladas)", report.summary.count, report.voided),
                colors.accent,
            ),
            (
                "Total vendido",
                format::amount(report.summary.total),
                colors.success,
            ),
            ("Entradas", format::amount(report.income), colors.success),
            ("Gastos", format::amount(report.expense), colors.danger),
        ],
    );
    card::stats(
        ui,
        &[
            (
                "Balance del dia",
                format::amount(report.balance()),
                colors.purple,
            ),
            (
                "Promedio por venta",
                format::amount(report.summary.average),
                colors.accent,
            ),
            (
                "Utilidad estimada",
                format::amount(report.profit),
                colors.warning,
            ),
        ],
    );
}

fn details(ui: &mut Ui, report: &DailyReport) {
    title::heading(ui, "Ventas por hora");
    if report.sales.is_empty() {
        title::hint(ui, "No hay ventas en este dia");
    } else {
        card::surface(ui, |ui| bars(ui, &hours(report), 130.0));
    }
    title::heading(ui, "Por metodo de pago");
    let methods = report
        .methods
        .iter()
        .map(|item| {
            Row::new(vec![
                item.method.label().to_owned(),
                format::amount(item.sales),
                format::amount(item.income),
                format::amount(item.expense),
                format::amount(item.balance()),
            ])
        })
        .collect();
    section(ui, "daily_methods", &METHODS, methods);
    title::heading(ui, "Ventas del dia");
    let sales = report
        .sales
        .iter()
        .map(|sale| {
            Row::new(vec![
                sale.client_name.clone(),
                sale.created_at.format("%H:%M").to_string(),
                sale.short_id().to_owned(),
                sale.seller.clone(),
                sale.payment_method.label().to_owned(),
                format::amount(sale.total),
            ])
        })
        .collect();
    section(ui, "daily_sales", &SALES, sales);
    title::heading(ui, "Entradas y gastos del dia");
    let entries = report
        .entries
        .iter()
        .map(|entry| {
            Row::new(vec![
                entry.concept.clone(),
                entry.created_at.format("%H:%M").to_string(),
                entry.kind.label().to_owned(),
                entry.author.clone(),
                format::amount(entry.signed()),
            ])
        })
        .collect();
    section(ui, "daily_entries", &ENTRIES, entries);
}

fn hours(report: &DailyReport) -> Vec<Bar> {
    report
        .hours
        .iter()
        .map(|item| Bar {
            label: format!("{:02}", item.hour),
            value: item.total.to_f32().unwrap_or_default(),
            detail: format!(
                "{:02}:00 - {} ({} ventas)",
                item.hour,
                format::amount(item.total),
                item.count
            ),
        })
        .collect()
}

fn section(ui: &mut Ui, id: &str, columns: &[Column], rows: Vec<Row>) {
    if rows.is_empty() {
        title::hint(ui, "Sin registros");
        return;
    }
    let height = 34.0 * rows.len().clamp(1, 7) as f32 + 40.0;
    ui.allocate_ui([ui.available_width(), height].into(), |ui| {
        table(ui, id, columns, &rows, None, height - 34.0);
    });
}

fn shifted(text: &str, days: i64) -> Option<NaiveDate> {
    let date = NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d").ok()?;
    date.checked_add_signed(TimeDelta::days(days))
}
