use pos_config::{AppConfig, PrinterFormat};
use pos_domain::now;

use super::style::style;
use crate::format;

#[derive(Clone, Debug)]
pub struct Document {
    pub name: String,
    pub html: String,
}

impl Document {
    pub fn new(name: &str, body: &str, config: &AppConfig, paper: PrinterFormat) -> Self {
        let html = format!(
            "<!doctype html><html lang='es'><head><meta charset='utf-8'><title>{}</title>\
             <style>{}</style></head><body><div class='doc'>{}{body}\
             <footer><span>Generado el {}</span><span>Ventas POS</span></footer>\
             </div></body></html>",
            escape(name),
            style(paper),
            header(config),
            format::stamp(now())
        );
        Self {
            name: name.to_owned(),
            html,
        }
    }

    pub fn file_name(&self) -> String {
        let cleaned: String = self
            .name
            .chars()
            .filter(|letter| letter.is_alphanumeric() || "-_ ".contains(*letter))
            .collect();
        let base = match cleaned.trim() {
            "" => "documento",
            name => name,
        };
        base.replace(' ', "_").chars().take(60).collect()
    }
}

pub fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

pub fn cards(summary: &[(String, String)]) -> String {
    if summary.is_empty() {
        return String::new();
    }
    let body: String = summary
        .iter()
        .map(|(label, value)| {
            format!(
                "<div class='card'><div class='label'>{}</div><div class='value'>{}</div></div>",
                escape(label),
                escape(value)
            )
        })
        .collect();
    format!("<div class='cards'>{body}</div>")
}

pub fn subtitle(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    format!("<p class='subtitle'>{}</p>", escape(text))
}

fn header(config: &AppConfig) -> String {
    let general = &config.general;
    let details: Vec<String> = [
        ("NIT / ID: ", &general.business_tax_id),
        ("", &general.business_address),
        ("Tel: ", &general.business_phone),
    ]
    .into_iter()
    .filter(|(_, value)| !value.trim().is_empty())
    .map(|(label, value)| format!("{label}{}", escape(value)))
    .collect();
    format!(
        "<header><p class='business'>{}</p><div class='business-meta'>{}</div></header>",
        escape(config.business_name()),
        details.join("<br>")
    )
}
