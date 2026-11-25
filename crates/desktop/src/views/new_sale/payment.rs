use std::str::FromStr;

use pos_domain::{PaymentMethod, Sale};
use rust_decimal::Decimal;

pub struct Payment {
    pub method: usize,
    pub discount: String,
    pub received: String,
    pub notes: String,
}

impl Payment {
    pub fn new(method: PaymentMethod) -> Self {
        Self {
            method: PaymentMethod::ALL
                .iter()
                .position(|item| *item == method)
                .unwrap_or_default(),
            discount: "0".to_owned(),
            received: String::new(),
            notes: String::new(),
        }
    }

    pub fn reset(&mut self) {
        self.discount = "0".to_owned();
        self.received.clear();
        self.notes.clear();
    }

    pub fn method(&self) -> PaymentMethod {
        PaymentMethod::ALL[self.method]
    }

    pub fn is_cash(&self) -> bool {
        self.method() == PaymentMethod::Efectivo
    }

    pub fn discount(&self) -> Option<Decimal> {
        amount(&self.discount)
    }

    pub fn change(&self, total: Decimal) -> Option<Decimal> {
        if !self.is_cash() || self.received.trim().is_empty() {
            return None;
        }
        amount(&self.received).map(|received| received - total)
    }

    pub fn apply(&self, sale: &mut Sale) {
        sale.payment_method = self.method();
        sale.discount = self.discount().unwrap_or_default();
        sale.notes = self.notes.trim().to_owned();
    }

    pub fn validate(&self, total: Decimal) -> Result<(), String> {
        if self.discount().is_none() {
            return Err("El descuento debe ser un valor numerico".to_owned());
        }
        if !self.is_cash() || self.received.trim().is_empty() {
            return Ok(());
        }
        match self.change(total) {
            None => Err("El valor recibido debe ser numerico".to_owned()),
            Some(change) if change < Decimal::ZERO => {
                Err("El valor recibido no cubre el total".to_owned())
            }
            Some(_) => Ok(()),
        }
    }
}

fn amount(text: &str) -> Option<Decimal> {
    match text.trim() {
        "" => Some(Decimal::ZERO),
        value => Decimal::from_str(&value.replace(',', ".")).ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cash_must_cover_the_total() {
        let mut payment = Payment::new(PaymentMethod::Efectivo);
        let total = Decimal::from(1000);
        assert!(payment.validate(total).is_ok());
        payment.received = "900".into();
        assert!(payment.validate(total).is_err());
        payment.received = "x".into();
        assert!(payment.validate(total).is_err());
        payment.received = "1500,50".into();
        assert!(payment.validate(total).is_ok());
        assert_eq!(payment.change(total), Some(Decimal::new(50050, 2)));
        payment.method = 1;
        assert_eq!(payment.change(total), None);
        payment.discount = "abc".into();
        assert!(payment.validate(total).is_err());
    }
}
