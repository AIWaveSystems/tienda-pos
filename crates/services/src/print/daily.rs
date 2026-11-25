use pos_config::AppConfig;

use super::document::Document;
use super::report::{report, Section};
use super::table::Column;
use crate::daily::DailyReport;
use crate::format;

pub fn daily_report(daily: &DailyReport, config: &AppConfig) -> Document {
    let currency = config.general.currency.as_str();
    let sections = [
        totals(daily, currency),
        methods(daily),
        sales(daily),
        entries(daily),
    ];
    report("Informe diario", &daily.span.label(), &sections, config)
}

fn totals(daily: &DailyReport, currency: &str) -> Section {
    let money = |value| format::money(value, currency);
    Section {
        title: "Resumen".to_owned(),
        summary: [
            ("Ventas", daily.summary.count.to_string()),
            ("Anuladas", daily.voided.to_string()),
            ("Total vendido", money(daily.summary.total)),
            ("Entradas", money(daily.income)),
            ("Gastos", money(daily.expense)),
            ("Balance", money(daily.balance())),
            ("Efectivo esperado", money(daily.expected_cash())),
            ("Utilidad estimada", money(daily.profit)),
        ]
        .into_iter()
        .map(|(label, value)| (label.to_owned(), value))
        .collect(),
        ..Section::default()
    }
}

fn methods(daily: &DailyReport) -> Section {
    Section {
        title: "Por metodo de pago".to_owned(),
        columns: vec![
            Column::text("Metodo"),
            Column::number("Ventas"),
            Column::number("Entradas"),
            Column::number("Gastos"),
            Column::number("Balance"),
        ],
        rows: daily
            .methods
            .iter()
            .map(|item| {
                vec![
                    item.method.label().to_owned(),
                    format::amount(item.sales),
                    format::amount(item.income),
                    format::amount(item.expense),
                    format::amount(item.balance()),
                ]
            })
            .collect(),
        ..Section::default()
    }
}

fn sales(daily: &DailyReport) -> Section {
    Section {
        title: "Ventas".to_owned(),
        columns: vec![
            Column::text("Venta"),
            Column::text("Hora"),
            Column::text("Cliente"),
            Column::text("Vendedor"),
            Column::text("Pago"),
            Column::number("Total"),
        ],
        rows: daily
            .sales
            .iter()
            .map(|sale| {
                vec![
                    sale.short_id().to_owned(),
                    sale.created_at.format("%H:%M").to_string(),
                    sale.client_name.clone(),
                    sale.seller.clone(),
                    sale.payment_method.label().to_owned(),
                    format::amount(sale.total),
                ]
            })
            .collect(),
        ..Section::default()
    }
}

fn entries(daily: &DailyReport) -> Section {
    Section {
        title: "Entradas y gastos".to_owned(),
        columns: vec![
            Column::text("Hora"),
            Column::text("Tipo"),
            Column::text("Concepto"),
            Column::text("Autor"),
            Column::number("Valor"),
        ],
        rows: daily
            .entries
            .iter()
            .map(|entry| {
                vec![
                    entry.created_at.format("%H:%M").to_string(),
                    entry.kind.label().to_owned(),
                    entry.concept.clone(),
                    entry.author.clone(),
                    format::amount(entry.signed()),
                ]
            })
            .collect(),
        ..Section::default()
    }
}
