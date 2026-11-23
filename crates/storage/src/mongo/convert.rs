use std::str::FromStr;

use mongodb::bson::{Bson, Decimal128, Document};
use pos_domain::Entity;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde_json::{Map, Value};

use crate::{Result, StorageError};

const MONEY: [&str; 7] = [
    "price",
    "cost",
    "unit_price",
    "subtotal",
    "tax",
    "discount",
    "total",
];

pub fn to_document<T: Entity>(item: &T) -> Result<Document> {
    let Value::Object(mut fields) = serde_json::to_value(item)? else {
        return Err(invalid("el registro no es un objeto"));
    };
    if let Some(id) = fields.remove("id") {
        fields.insert("_id".to_owned(), id);
    }
    encode_fields(fields)
}

pub fn from_document<T: Entity>(document: Document) -> Result<T> {
    let mut fields = decode_fields(document)?;
    if let Some(id) = fields.remove("_id") {
        fields.insert("id".to_owned(), id);
    }
    Ok(serde_json::from_value(Value::Object(fields))?)
}

fn encode_fields(fields: Map<String, Value>) -> Result<Document> {
    fields
        .into_iter()
        .map(|(key, value)| {
            let encoded = if MONEY.contains(&key.as_str()) {
                Bson::Decimal128(money(&value)?)
            } else {
                encode(value)?
            };
            Ok((key, encoded))
        })
        .collect()
}

fn encode(value: Value) -> Result<Bson> {
    match value {
        Value::Object(fields) => Ok(Bson::Document(encode_fields(fields)?)),
        Value::Array(items) => Ok(Bson::Array(
            items.into_iter().map(encode).collect::<Result<_>>()?,
        )),
        Value::Number(number) => number
            .as_i64()
            .map(Bson::Int64)
            .or_else(|| number.as_f64().map(Bson::Double))
            .ok_or_else(|| invalid("numero fuera de rango")),
        other => Bson::try_from(other).map_err(|failure| invalid(&failure.to_string())),
    }
}

fn money(value: &Value) -> Result<Decimal128> {
    let amount = value
        .as_f64()
        .and_then(|number| Decimal::try_from(number).ok())
        .ok_or_else(|| invalid("importe invalido"))?;
    Decimal128::from_str(&amount.to_string()).map_err(|failure| invalid(&failure.to_string()))
}

fn decode_fields(document: Document) -> Result<Map<String, Value>> {
    document
        .into_iter()
        .map(|(key, value)| Ok((key, decode(value)?)))
        .collect()
}

fn decode(value: Bson) -> Result<Value> {
    match value {
        Bson::Document(document) => Ok(Value::Object(decode_fields(document)?)),
        Bson::Array(items) => Ok(Value::Array(
            items.into_iter().map(decode).collect::<Result<_>>()?,
        )),
        Bson::Decimal128(amount) => {
            let text = amount.to_string();
            Decimal::from_str(&text)
                .or_else(|_| Decimal::from_scientific(&text))
                .ok()
                .and_then(|amount| amount.to_f64())
                .map(Value::from)
                .ok_or_else(|| invalid("importe invalido"))
        }
        Bson::Int32(number) => Ok(number.into()),
        Bson::Int64(number) => Ok(number.into()),
        other => Ok(other.into_relaxed_extjson()),
    }
}

fn invalid(detail: &str) -> StorageError {
    StorageError::Data(detail.to_owned())
}
