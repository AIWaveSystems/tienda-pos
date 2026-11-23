use serde::{Deserialize, Serialize};

use crate::Permission;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Admin,
    Vendedor,
    Bodega,
    #[default]
    #[serde(other)]
    Personalizado,
}

impl Role {
    pub const ALL: [Self; 4] = [
        Self::Admin,
        Self::Vendedor,
        Self::Bodega,
        Self::Personalizado,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Vendedor => "vendedor",
            Self::Bodega => "bodega",
            Self::Personalizado => "personalizado",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Admin => "Administrador",
            Self::Vendedor => "Vendedor",
            Self::Bodega => "Bodega",
            Self::Personalizado => "Personalizado",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Admin => "Acceso total al sistema",
            Self::Vendedor => "Vende y gestiona clientes",
            Self::Bodega => "Gestiona inventario y consulta informes",
            Self::Personalizado => "Permisos seleccionados manualmente",
        }
    }

    pub fn permissions(self) -> Vec<Permission> {
        use Permission::*;
        match self {
            Self::Admin => Permission::ALL.to_vec(),
            Self::Vendedor => vec![Dashboard, Clientes, Ventas, Historial],
            Self::Bodega => vec![Dashboard, Inventario, Informes],
            Self::Personalizado => Vec::new(),
        }
    }

    pub fn is_locked(self) -> bool {
        self != Self::Personalizado
    }

    pub fn from_key(key: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|role| role.key() == key)
            .unwrap_or_default()
    }
}
