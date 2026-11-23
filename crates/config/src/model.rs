use pos_domain::PaymentMethod;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::{BackendKind, PrinterFormat};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AppConfig {
    pub schema_version: u32,
    pub general: General,
    pub inventory: Inventory,
    pub sales: Sales,
    pub ui: Interface,
    pub storage: Storage,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct General {
    #[serde(with = "rust_decimal::serde::float")]
    pub tax_rate: Decimal,
    pub currency: String,
    pub business_name: String,
    pub business_tax_id: String,
    pub business_address: String,
    pub business_phone: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Inventory {
    pub min_stock_alert: i64,
    pub allow_negative_stock: bool,
    pub low_stock_on_dashboard: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Sales {
    pub default_payment_method: PaymentMethod,
    pub print_receipt: bool,
    pub receipt_footer: String,
    pub confirm_before_saving: bool,
    pub printer_format: PrinterFormat,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Interface {
    pub theme: String,
    pub sidebar_width: i64,
    pub window_width: i64,
    pub window_height: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Storage {
    pub backend: BackendKind,
    pub mysql: SqlSettings,
    pub postgresql: SqlSettings,
    pub mongodb: MongoSettings,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SqlSettings {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub ssl: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MongoSettings {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub use_srv: bool,
}

impl AppConfig {
    pub fn business_name(&self) -> &str {
        let name = self.general.business_name.trim();
        if name.is_empty() {
            "Ventas POS"
        } else {
            name
        }
    }
}
