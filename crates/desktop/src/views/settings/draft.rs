use std::str::FromStr;

use pos_config::model::{General, Inventory, Sales};
use pos_config::{AppConfig, PrinterFormat};
use pos_domain::PaymentMethod;
use rust_decimal::Decimal;

pub struct Draft {
    pub general: General,
    pub inventory: Inventory,
    pub sales: Sales,
    pub tax: String,
    pub min_stock: String,
    pub method: usize,
    pub format: usize,
    pub width: String,
    pub height: String,
    pub sidebar: String,
}

impl Draft {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            general: config.general.clone(),
            inventory: config.inventory.clone(),
            sales: config.sales.clone(),
            tax: (config.general.tax_rate * Decimal::ONE_HUNDRED)
                .normalize()
                .to_string(),
            min_stock: config.inventory.min_stock_alert.to_string(),
            method: position(&PaymentMethod::ALL, config.sales.default_payment_method),
            format: position(&PrinterFormat::ALL, config.sales.printer_format),
            width: config.ui.window_width.to_string(),
            height: config.ui.window_height.to_string(),
            sidebar: config.ui.sidebar_width.to_string(),
        }
    }

    pub fn methods() -> Vec<String> {
        PaymentMethod::ALL
            .iter()
            .map(|method| method.label().to_owned())
            .collect()
    }

    pub fn formats() -> Vec<String> {
        PrinterFormat::ALL
            .iter()
            .map(|format| format.label().to_owned())
            .collect()
    }

    pub fn apply(&self, config: &mut AppConfig) -> Result<(), String> {
        config.general = self.general.clone();
        config.general.tax_rate = self.tax_rate()?;
        config.general.currency = self.general.currency.trim().to_uppercase();
        if config.general.currency.is_empty() {
            return Err("La moneda es obligatoria".to_owned());
        }
        config.inventory = self.inventory.clone();
        config.inventory.min_stock_alert = whole(&self.min_stock, "stock minimo", 0, 1000)?;
        config.sales = self.sales.clone();
        config.sales.default_payment_method = PaymentMethod::ALL[self.method];
        config.sales.printer_format = PrinterFormat::ALL[self.format];
        config.ui.window_width = whole(&self.width, "ancho de ventana", 640, 3840)?;
        config.ui.window_height = whole(&self.height, "alto de ventana", 480, 2160)?;
        config.ui.sidebar_width = whole(&self.sidebar, "ancho del menu lateral", 150, 400)?;
        Ok(())
    }

    fn tax_rate(&self) -> Result<Decimal, String> {
        let percent = Decimal::from_str(&self.tax.trim().replace(',', "."))
            .map_err(|_| "El impuesto debe ser un numero (por ejemplo 19)".to_owned())?;
        if percent < Decimal::ZERO || percent > Decimal::ONE_HUNDRED {
            return Err("El impuesto debe estar entre 0 y 100".to_owned());
        }
        Ok(percent / Decimal::ONE_HUNDRED)
    }
}

pub fn whole(text: &str, label: &str, low: i64, high: i64) -> Result<i64, String> {
    let value: i64 = text
        .trim()
        .parse()
        .map_err(|_| format!("El valor de {label} debe ser un numero entero"))?;
    if !(low..=high).contains(&value) {
        return Err(format!(
            "El valor de {label} debe estar entre {low} y {high}"
        ));
    }
    Ok(value)
}

fn position<T: PartialEq>(options: &[T], value: T) -> usize {
    options
        .iter()
        .position(|option| *option == value)
        .unwrap_or_default()
}
