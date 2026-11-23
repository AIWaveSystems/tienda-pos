use pos_domain::Permission;

use crate::env::Env;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Route {
    Dashboard,
    Inventory,
    Categories,
    Clients,
    NewSale,
    History,
    Reports,
    Users,
    Settings,
}

impl Route {
    pub const ALL: [Self; 9] = [
        Self::Dashboard,
        Self::Inventory,
        Self::Categories,
        Self::Clients,
        Self::NewSale,
        Self::History,
        Self::Reports,
        Self::Users,
        Self::Settings,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Categories => "Categorias",
            other => other.permission().label(),
        }
    }

    pub fn permission(self) -> Permission {
        match self {
            Self::Dashboard => Permission::Dashboard,
            Self::Inventory | Self::Categories => Permission::Inventario,
            Self::Clients => Permission::Clientes,
            Self::NewSale => Permission::Ventas,
            Self::History => Permission::Historial,
            Self::Reports => Permission::Informes,
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
