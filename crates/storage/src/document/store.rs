use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use pos_config::BackendKind;
use pos_domain::{now, Category, Client, Product, Sale, SaleItem, User};

use super::{DocumentRepository, FileBackend, LocalFiles};
use crate::{Repository, Result, StorageError, Store};

pub struct DocumentStore {
    files: Arc<dyn FileBackend>,
    lock: Arc<Mutex<()>>,
    users: DocumentRepository<User>,
    categories: DocumentRepository<Category>,
    clients: DocumentRepository<Client>,
    products: DocumentRepository<Product>,
    sales: DocumentRepository<Sale>,
}

impl DocumentStore {
    pub fn new(files: Arc<dyn FileBackend>) -> Self {
        let lock = Arc::new(Mutex::new(()));
        Self {
            users: DocumentRepository::new(files.clone(), lock.clone()),
            categories: DocumentRepository::new(files.clone(), lock.clone()),
            clients: DocumentRepository::new(files.clone(), lock.clone()),
            products: DocumentRepository::new(files.clone(), lock.clone()),
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

    fn register_sale(&self, sale: &Sale, allow_negative: bool) -> Result<()> {
        let _guard = self.lock.lock().unwrap_or_else(PoisonError::into_inner);
        let products = self.products.load()?;
        let mut sales = self.sales.load()?;
        let mut updated = products.items.clone();
        for item in &sale.items {
            take(&mut updated, item, allow_negative)?;
        }
        sales.items.push(sale.clone());
        self.products
            .store(&updated, products.revision.as_deref())?;
        let outcome = self.sales.store(&sales.items, sales.revision.as_deref());
        if outcome.is_err() {
            let current = self.products.load()?.revision;
            self.products.store(&products.items, current.as_deref())?;
        }
        outcome
    }

    fn ping(&self) -> Result<()> {
        self.files.ping()
    }
}

fn take(products: &mut [Product], item: &SaleItem, allow_negative: bool) -> Result<()> {
    let found = products
        .iter_mut()
        .find(|product| product.id == item.product_id);
    match found {
        Some(product) if allow_negative || product.stock >= item.quantity => {
            product.stock -= item.quantity;
            product.updated_at = now();
            Ok(())
        }
        None if allow_negative => Ok(()),
        _ => Err(StorageError::Stock(item.product_name.clone())),
    }
}
