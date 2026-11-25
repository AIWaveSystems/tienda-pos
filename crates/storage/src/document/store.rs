use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use pos_config::BackendKind;
use pos_domain::{Category, Client, Product, Sale, StockMovement, User};

use super::{DocumentRepository, FileBackend, LocalFiles};
use crate::{Repository, Result, StockOperation, Store};

pub struct DocumentStore {
    files: Arc<dyn FileBackend>,
    pub(super) lock: Arc<Mutex<()>>,
    users: DocumentRepository<User>,
    categories: DocumentRepository<Category>,
    clients: DocumentRepository<Client>,
    pub(super) products: DocumentRepository<Product>,
    pub(super) sales: DocumentRepository<Sale>,
    pub(super) movements: DocumentRepository<StockMovement>,
}

impl DocumentStore {
    pub fn new(files: Arc<dyn FileBackend>) -> Self {
        let lock = Arc::new(Mutex::new(()));
        Self {
            users: DocumentRepository::new(files.clone(), lock.clone()),
            categories: DocumentRepository::new(files.clone(), lock.clone()),
            clients: DocumentRepository::new(files.clone(), lock.clone()),
            products: DocumentRepository::new(files.clone(), lock.clone()),
            movements: DocumentRepository::new(files.clone(), lock.clone()),
            sales: DocumentRepository::new(files.clone(), lock.clone()),
            files,
            lock,
        }
    }

    pub fn local(folder: impl Into<PathBuf>) -> Result<Self> {
        Ok(Self::new(Arc::new(LocalFiles::new(folder)?)))
    }
}

impl Store for DocumentStore {
    fn kind(&self) -> BackendKind {
        self.files.kind()
    }

    fn users(&self) -> &dyn Repository<User> {
        &self.users
    }

    fn categories(&self) -> &dyn Repository<Category> {
        &self.categories
    }

    fn clients(&self) -> &dyn Repository<Client> {
        &self.clients
    }

    fn products(&self) -> &dyn Repository<Product> {
        &self.products
    }

    fn sales(&self) -> &dyn Repository<Sale> {
        &self.sales
    }

    fn movements(&self) -> &dyn Repository<StockMovement> {
        &self.movements
    }

    fn apply_stock(&self, operation: &StockOperation) -> Result<()> {
        self.apply(operation)
    }

    fn ping(&self) -> Result<()> {
        self.files.ping()
    }
}
