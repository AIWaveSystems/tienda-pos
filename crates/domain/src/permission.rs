use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    Dashboard,
    Inventario,
    Clientes,
    Ventas,
    Historial,
    Caja,
    Informes,
    Usuarios,
    Configuracion,
}

impl Permission {
    pub const ALL: [Self; 9] = [
        Self::Dashboard,
        Self::Inventario,
        Self::Clientes,
        Self::Ventas,
        Self::Historial,
        Self::Caja,
        Self::Informes,
        Self::Usuarios,
        Self::Configuracion,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::Dashboard => "dashboard",
            Self::Inventario => "inventario",
            Self::Clientes => "clientes",
            Self::Ventas => "ventas",
            Self::Historial => "historial",
            Self::Caja => "caja",
            Self::Informes => "informes",
            Self::Usuarios => "usuarios",
            Self::Configuracion => "configuracion",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Inventario => "Inventario",
            Self::Clientes => "Clientes",
            Self::Ventas => "Nueva Venta",
            Self::Historial => "Historial",
            Self::Caja => "Caja",
            Self::Informes => "Informes",
            Self::Usuarios => "Usuarios",
            Self::Configuracion => "Configuracion",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Dashboard => "Ver el resumen general del negocio",
            Self::Inventario => "Crear, editar y eliminar productos",
            Self::Clientes => "Crear, editar y eliminar clientes",
            Self::Ventas => "Registrar ventas en el punto de venta",
            Self::Historial => "Consultar las ventas registradas",
            Self::Caja => "Registrar entradas de dinero y gastos",
            Self::Informes => "Generar informes y reportes",
            Self::Usuarios => "Administrar usuarios y sus permisos",
            Self::Configuracion => "Modificar parametros del sistema",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|item| item.key() == key)
    }

    pub fn normalize(items: impl IntoIterator<Item = Self>) -> Vec<Self> {
        let mut unique = Vec::new();
        for item in items {
            if !unique.contains(&item) {
                unique.push(item);
            }
        }
        unique
    }
}

pub(crate) fn lenient<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<Permission>, D::Error> {
    let value = Value::deserialize(deserializer)?;
    let items = value.as_array().cloned().unwrap_or_default();
    Ok(Permission::normalize(
        items
            .iter()
            .filter_map(Value::as_str)
            .filter_map(Permission::from_key),
    ))
}
