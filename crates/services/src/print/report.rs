use pos_config::{AppConfig, PrinterFormat};

use super::document::{cards, escape, subtitle, Document};
use super::table::{grid, Column};
use crate::format;
use crate::report::Overview;

#[derive(Clone, Debug, Default)]
pub struct Section {
    pub title: String,
    pub summary: Vec<(String, String)>,
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<String>>,
}

pub fn report(title: &str, detail: &str, sections: &[Section], config: &AppConfig) -> Document {
    let blocks: String = sections
        .iter()
        .map(|section| {
            let body = if section.columns.is_empty() {
                String::new()
            } else {
                grid(&section.columns, &section.rows)
            };
            format!(
                "<h1>{}</h1>{}{body}",
                escape(&section.title),
                cards(&section.summary)
            )
        })
        .collect();
    let body = format!("{}{blocks}", subtitle(detail));
    Document::new(title, &body, config, PrinterFormat::A4)
}

pub fn overview_report(overview: &Overview, config: &AppConfig) -> Document {
    let currency = config.general.currency.as_str();
    let sections = [
        Section {
            title: "Resumen de ventas".to_owned(),
            summary: vec![
                pair(
                    "Total vendido",
                    format::money(overview.sales.total, currency),
                ),
                pair("Numero de ventas", overview.sales.count.to_string()),
                pair(
                    "Ticket promedio",
                    format::money(overview.sales.average, currency),
                ),
            ],
            ..Section::default()
        },
        inventory(overview, currency),
        Section {
            title: "Productos mas vendidos".to_owned(),
            columns: vec![
                Column::text("Producto"),
                Column::number("Unidades"),
                Column::number("Ingresos"),
            ],
            rows: overview
                .top
                .iter()
                .map(|item| {
                    vec![
                        item.name.clone(),
                        item.quantity.to_string(),
                        format::amount(item.revenue),
                    ]
                })
                .collect(),
            ..Section::default()
        },
        Section {
            title: format!("Ventas por dia (ultimos {} dias)", overview.daily.len()),
            columns: vec![
                Column::text("Fecha"),
                Column::number("Ventas"),
                Column::number("Total"),
            ],
            rows: overview
                .daily
                .iter()
                .filter(|day| day.count > 0)
                .map(|day| {
                    vec![
                        day.date.clone(),
                        day.count.to_string(),
                        format::amount(day.total),
                    ]
                })
                .collect(),
            ..Section::default()
        },
    ];
    report(
        "Informe general",
        "Resumen de ventas, inventario y desempeno de productos",
        &sections,
        config,
    )
}

fn inventory(overview: &Overview, currency: &str) -> Section {
    let inventory = &overview.inventory;
    Section {
        title: "Inventario".to_owned(),
        summary: vec![
            pair("Productos activos", inventory.products.to_string()),
            pair(
                "Productos con stock bajo",
                inventory.low_stock.len().to_string(),
            ),
            pair(
                "Valor del inventario",
                format::money(inventory.value, currency),
            ),
        ],
        columns: vec![
            Column::text("Producto"),
            Column::number("Stock"),
            Column::number("Minimo"),
        ],
        rows: inventory
            .low_stock
            .iter()
            .map(|product| {
                vec![
                    product.name.clone(),
                    product.stock.to_string(),
                    product.min_stock.to_string(),
                ]
            })
            .collect(),
    }
}

fn pair(label: &str, value: String) -> (String, String) {
    (label.to_owned(), value)
}
