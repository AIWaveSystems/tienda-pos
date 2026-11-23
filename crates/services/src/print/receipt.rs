use pos_config::AppConfig;
use pos_domain::Sale;
use rust_decimal::Decimal;

use super::document::{escape, Document};
use super::table::{grid, Column};
use crate::format;

const COLUMNS: [Column; 4] = [
    Column::text("Producto"),
    Column::number("Cant."),
    Column::number("Precio"),
    Column::number("Subtotal"),
];

pub fn receipt(sale: &Sale, config: &AppConfig) -> Document {
    let currency = config.general.currency.as_str();
    let rows: Vec<Vec<String>> = sale
        .items
        .iter()
        .map(|item| {
            vec![
                item.product_name.clone(),
                item.quantity.to_string(),
                format::money(item.unit_price, currency),
                format::money(item.subtotal, currency),
            ]
        })
        .collect();
    let client = match sale.client_name.as_str() {
        "" => "Consumidor final",
        name => name,
    };
    let body = format!(
        "<h1>Comprobante de venta</h1>\
         <p class='subtitle'>No. {} &nbsp;|&nbsp; {}<br>Cliente: {} &nbsp;|&nbsp; Pago: {}</p>\
         {}{}{}{}",
        escape(&sale.id),
        format::stamp(sale.created_at),
        escape(client),
        sale.payment_method.label(),
        grid(&COLUMNS, &rows),
        totals(sale, currency),
        notes(sale),
        message(config)
    );
    let name = format!("Comprobante {}", sale.short_id());
    Document::new(&name, &body, config, config.sales.printer_format)
}

fn totals(sale: &Sale, currency: &str) -> String {
    let row = |label: &str, value: String, class: &str| {
        format!("<tr{class}><td>{label}</td><td class='num'>{value}</td></tr>")
    };
    let discount = if sale.discount == Decimal::ZERO {
        String::new()
    } else {
        let value = format!("-{}", format::money(sale.discount, currency));
        row("Descuento", value, "")
    };
    format!(
        "<table class='totals'>{}{}{discount}{}</table>",
        row("Subtotal", format::money(sale.subtotal, currency), ""),
        row("Impuesto", format::money(sale.tax, currency), ""),
        row(
            "Total",
            format::money(sale.total, currency),
            " class='grand'"
        )
    )
}

fn notes(sale: &Sale) -> String {
    if sale.notes.is_empty() {
        return String::new();
    }
    format!(
        "<p class='subtitle'><strong>Notas:</strong> {}</p>",
        escape(&sale.notes)
    )
}

fn message(config: &AppConfig) -> String {
    match config.sales.receipt_footer.trim() {
        "" => String::new(),
        text => format!("<p class='message'>{}</p>", escape(text)),
    }
}
