use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrinterFormat {
    #[default]
    #[serde(rename = "a4")]
    A4,
    #[serde(rename = "thermal_80")]
    Thermal80,
    #[serde(rename = "thermal_58")]
    Thermal58,
}

impl PrinterFormat {
    pub const ALL: [Self; 3] = [Self::A4, Self::Thermal80, Self::Thermal58];

    pub fn key(self) -> &'static str {
        match self {
            Self::A4 => "a4",
            Self::Thermal80 => "thermal_80",
            Self::Thermal58 => "thermal_58",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::A4 => "Hoja normal (A4 / carta)",
            Self::Thermal80 => "Impresora termica 80mm",
            Self::Thermal58 => "Impresora termica 58mm",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|format| format.key() == key)
    }
}
