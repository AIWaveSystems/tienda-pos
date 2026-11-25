use pos_config::{AppConfig, PrinterFormat};

use super::csv::csv;
use super::document::{cards, escape, subtitle, Document};

#[derive(Clone, Copy, Debug)]
pub struct Column {
    pub label: &'static str,
    pub numeric: bool,
}

impl Column {
    pub const fn text(label: &'static str) -> Self {
        Self {
            label,
            numeric: false,
        }
    }

    pub const fn number(label: &'static str) -> Self {
        Self {
            label,
            numeric: true,
        }
    }

    fn class(self) -> &'static str {
        if self.numeric {
            " class='num'"
        } else {
            ""
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Listing {
    pub title: String,
    pub subtitle: String,
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<String>>,
    pub summary: Vec<(String, String)>,
}

pub fn table(listing: &Listing, config: &AppConfig) -> Document {
    let body = format!(
        "<h1>{}</h1>{}{}{}<p class='subtitle'>Total de registros: {}</p>",
        escape(&listing.title),
        subtitle(&listing.subtitle),
        cards(&listing.summary),
        grid(&listing.columns, &listing.rows),
        listing.rows.len()
    );
    let mut document = Document::new(&listing.title, &body, config, PrinterFormat::A4);
    document.csv = Some(csv(listing));
    document
}

pub fn grid(columns: &[Column], rows: &[Vec<String>]) -> String {
    let head: String = columns
        .iter()
        .map(|column| format!("<th{}>{}</th>", column.class(), escape(column.label)))
        .collect();
    let body: String = if rows.is_empty() {
        format!(
            "<tr><td colspan='{}' class='empty'>Sin registros</td></tr>",
            columns.len()
        )
    } else {
        rows.iter().map(|row| line(columns, row)).collect()
    };
    format!("<table><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table>")
}

fn line(columns: &[Column], row: &[String]) -> String {
    let cells: String = columns
        .iter()
        .zip(row)
        .map(|(column, value)| format!("<td{}>{}</td>", column.class(), escape(value)))
        .collect();
    format!("<tr>{cells}</tr>")
}
