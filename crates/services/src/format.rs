use chrono::NaiveDateTime;
use rust_decimal::{Decimal, RoundingStrategy};

pub fn amount(value: Decimal) -> String {
    let rounded = value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
    let text = format!("{:.2}", rounded.abs());
    let (whole, cents) = text.split_once('.').unwrap_or((&text, "00"));
    let sign = if rounded.is_sign_negative() && !rounded.is_zero() {
        "-"
    } else {
        ""
    };
    format!("{sign}{}.{cents}", group(whole))
}

pub fn money(value: Decimal, currency: &str) -> String {
    format!("{} {currency}", amount(value))
}

pub fn minute(moment: NaiveDateTime) -> String {
    moment.format("%Y-%m-%d %H:%M").to_string()
}

pub fn stamp(moment: NaiveDateTime) -> String {
    moment.format("%d/%m/%Y %H:%M").to_string()
}

fn group(digits: &str) -> String {
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}
