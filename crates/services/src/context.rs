use std::sync::Arc;

use pos_storage::Store;

use crate::{
    AuthService, CategoryService, ClientService, InventoryService, ReportService, SalesService,
};

pub struct Services {
    pub auth: AuthService,
    pub categories: CategoryService,
    pub clients: ClientService,
    pub inventory: InventoryService,
    pub sales: SalesService,
    pub reports: ReportService,
    store: Arc<dyn Store>,
}

impl Services {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self {
            auth: AuthService::new(store.clone()),
            categories: CategoryService::new(store.clone()),
            clients: ClientService::new(store.clone()),
            inventory: InventoryService::new(store.clone()),
            sales: SalesService::new(store.clone()),
            reports: ReportService::new(store.clone()),
            store,
        }
    }

    pub fn close(&self) {
        self.auth.logout();
        self.store.close();
    }
}
