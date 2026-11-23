use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    #[default]
    Efectivo,
    TarjetaCredito,
    TarjetaDebito,
    Transferencia,
    #[serde(other)]
    Otro,
}

impl PaymentMethod {
    pub const ALL: [Self; 5] = [
        Self::Efectivo,
        Self::TarjetaCredito,
        Self::TarjetaDebito,
        Self::Transferencia,
        Self::Otro,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::Efectivo => "efectivo",
            Self::TarjetaCredito => "tarjeta_credito",
            Self::TarjetaDebito => "tarjeta_debito",
            Self::Transferencia => "transferencia",
            Self::Otro => "otro",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Efectivo => "Efectivo",
            Self::TarjetaCredito => "Tarjeta de credito",
            Self::TarjetaDebito => "Tarjeta de debito",
            Self::Transferencia => "Transferencia",
            Self::Otro => "Otro",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|method| method.key() == key)
    }
}
