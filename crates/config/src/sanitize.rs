use pos_domain::PaymentMethod;
use rust_decimal::Decimal;
use serde_json::Value;

use crate::model::*;
use crate::{BackendKind, PrinterFormat};

impl Default for AppConfig {
    fn default() -> Self {
        sanitize(&Value::Null)
    }
}

pub fn sanitize(root: &Value) -> AppConfig {
    let general = &root["general"];
    let inventory = &root["inventory"];
    let sales = &root["sales"];
    let ui = &root["ui"];
    let storage = &root["storage"];
    AppConfig {
        schema_version: SCHEMA_VERSION,
        general: General {
            tax_rate: rate(&general["tax_rate"]),
            currency: required(&general["currency"], "COP"),
            business_name: text(&general["business_name"], ""),
            business_tax_id: text(&general["business_tax_id"], ""),
            business_address: text(&general["business_address"], ""),
            business_phone: text(&general["business_phone"], ""),
        },
        inventory: Inventory {
            min_stock_alert: whole(&inventory["min_stock_alert"], 0, 1000, 5),
            allow_negative_stock: flag(&inventory["allow_negative_stock"], false),
            low_stock_on_dashboard: flag(&inventory["low_stock_on_dashboard"], true),
        },
        sales: Sales {
            default_payment_method: option(
                &sales["default_payment_method"],
                PaymentMethod::from_key,
            ),
            print_receipt: flag(&sales["print_receipt"], false),
            receipt_footer: text(&sales["receipt_footer"], "Gracias por su compra"),
            confirm_before_saving: flag(&sales["confirm_before_saving"], true),
            printer_format: option(&sales["printer_format"], PrinterFormat::from_key),
        },
        ui: Interface {
            theme: required(&ui["theme"], "dark"),
            sidebar_width: whole(&ui["sidebar_width"], 150, 400, 220),
            window_width: whole(&ui["window_width"], 640, 3840, 1000),
            window_height: whole(&ui["window_height"], 480, 2160, 650),
        },
        storage: Storage {
            backend: option(&storage["backend"], BackendKind::from_key),
            mysql: sql(&storage["mysql"], 3306),
            postgresql: sql(&storage["postgresql"], 5432),
            mongodb: mongo(&storage["mongodb"]),
        },
    }
}

fn sql(section: &Value, port: i64) -> SqlSettings {
    SqlSettings {
        host: required(&section["host"], "localhost"),
        port: whole(&section["port"], 1, 65535, port) as u16,
        database: text(&section["database"], ""),
        user: text(&section["user"], ""),
        ssl: flag(&section["ssl"], false),
    }
}

fn mongo(section: &Value) -> MongoSettings {
    MongoSettings {
        host: required(&section["host"], "localhost"),
        port: whole(&section["port"], 1, 65535, 27017) as u16,
        database: text(&section["database"], ""),
        user: text(&section["user"], ""),
        use_srv: flag(&section["use_srv"], false),
    }
}

fn text(value: &Value, fallback: &str) -> String {
    value.as_str().unwrap_or(fallback).to_owned()
}

fn required(value: &Value, fallback: &str) -> String {
    value
        .as_str()
        .filter(|content| !content.is_empty())
        .unwrap_or(fallback)
        .to_owned()
}

fn flag(value: &Value, fallback: bool) -> bool {
    value.as_bool().unwrap_or(fallback)
}

fn number(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str()?.trim().parse().ok())
        .filter(|content: &f64| content.is_finite())
}

fn whole(value: &Value, low: i64, high: i64, fallback: i64) -> i64 {
    number(value).map_or(fallback, |content| (content as i64).clamp(low, high))
}

fn rate(value: &Value) -> Decimal {
    number(value)
        .and_then(|content| Decimal::try_from(content.clamp(0.0, 1.0)).ok())
        .unwrap_or(Decimal::new(19, 2))
}

fn option<T: Default>(value: &Value, parse: impl Fn(&str) -> Option<T>) -> T {
    value.as_str().and_then(parse).unwrap_or_default()
}
