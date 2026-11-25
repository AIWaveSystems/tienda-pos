use pos_domain::Permission;

use crate::env::Env;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Route {
    Dashboard,
    Inventory,
    Categories,
    Movements,
    Clients,
    NewSale,
    History,
    Cash,
    Daily,
    Reports,
    Users,
    Settings,
}

impl Route {
    pub const ALL: [Self; 12] = [
        Self::Dashboard,
        Self::Inventory,
        Self::Categories,
        Self::Movements,
        Self::Clients,
        Self::NewSale,
        Self::History,
        Self::Cash,
        Self::Daily,
        Self::Reports,
        Self::Users,
        Self::Settings,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Categories => "Categorias",
            Self::Movements => "Movimientos",
            Self::Daily => "Informe diario",
            other => other.permission().label(),
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Dashboard => "🏠",
            Self::Inventory => "📦",
            Self::Categories => "🏷",
            Self::Movements => "🔄",
            Self::Clients => "👥",
            Self::NewSale => "💳",
            Self::History => "📋",
            Self::Cash => "💰",
            Self::Daily => "📅",
            Self::Reports => "📊",
            Self::Users => "👤",
            Self::Settings => "⚙",
        }
    }

    pub fn permission(self) -> Permission {
        match self {
            Self::Dashboard => Permission::Dashboard,
            Self::Inventory | Self::Categories | Self::Movements => Permission::Inventario,
            Self::Clients => Permission::Clientes,
            Self::NewSale => Permission::Ventas,
            Self::History => Permission::Historial,
            Self::Cash => Permission::Caja,
            Self::Reports | Self::Daily => Permission::Informes,
            Self::Users => Permission::Usuarios,
            Self::Settings => Permission::Configuracion,
        }
    }

    pub fn available(env: &Env) -> Vec<Self> {
        Self::ALL
            .into_iter()
            .filter(|route| env.can(route.permission()))
            .collect()
    }
}
