use std::path::{Path, PathBuf};
use std::sync::Arc;

use pos_domain::now;
use pos_storage::{copy_all, DocumentStore, Store};

use crate::{
    AuthService, CashService, CategoryService, ClientService, InventoryService, ReportService,
    Result, SalesService,
};

pub struct Services {
    pub auth: AuthService,
    pub cash: CashService,
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
            cash: CashService::new(store.clone()),
            categories: CategoryService::new(store.clone()),
            clients: ClientService::new(store.clone()),
            inventory: InventoryService::new(store.clone()),
            sales: SalesService::new(store.clone()),
            reports: ReportService::new(store.clone()),
            store,
        }
    }

    pub fn backup(&self, folder: &Path) -> Result<PathBuf> {
        let stamp = now().format("%Y%m%d_%H%M%S");
        let target = folder.join(format!("ventas-pos-{stamp}"));
        let copy = DocumentStore::local(&target)?;
        copy_all(self.store.as_ref(), &copy)?;
        Ok(target)
    }

    pub fn close(&self) {
        self.auth.logout();
        self.store.close();
    }
}
