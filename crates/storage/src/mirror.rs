use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use pos_config::BackendKind;
use pos_domain::{Category, Client, Entity, Product, Sale, User};

use crate::{DocumentStore, Repository, Result, Store};

const INTERVAL: Duration = Duration::from_secs(12);

type Dirty = Arc<Mutex<Vec<&'static str>>>;
type Select<T> = fn(&dyn Store) -> &dyn Repository<T>;

struct MirroredRepository<T> {
    primary: Arc<dyn Store>,
    select: Select<T>,
    dirty: Dirty,
}

impl<T: Entity> MirroredRepository<T> {
    fn inner(&self) -> &dyn Repository<T> {
        (self.select)(self.primary.as_ref())
    }

    fn mark(&self) {
        mark(&self.dirty, T::COLLECTION);
    }
}

impl<T: Entity> Repository<T> for MirroredRepository<T> {
    fn all(&self) -> Result<Vec<T>> {
        self.inner().all()
    }

    fn find(&self, id: &str) -> Result<Option<T>> {
        self.inner().find(id)
    }

    fn upsert(&self, item: &T) -> Result<()> {
        self.inner().upsert(item)?;
        self.mark();
        Ok(())
    }

    fn upsert_many(&self, items: &[T]) -> Result<()> {
        self.inner().upsert_many(items)?;
        self.mark();
        Ok(())
    }

    fn replace_all(&self, items: &[T]) -> Result<()> {
        self.inner().replace_all(items)?;
        self.mark();
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<bool> {
        let deleted = self.inner().delete(id)?;
        if deleted {
            self.mark();
        }
        Ok(deleted)
    }

    fn count(&self) -> Result<usize> {
        self.inner().count()
    }
}

struct Copier {
    primary: Arc<dyn Store>,
    mirror: Arc<DocumentStore>,
    dirty: Dirty,
}

impl Copier {
    fn flush(&self) {
        let pending = std::mem::take(&mut *lock(&self.dirty));
        for name in pending {
            if self.copy(name).is_err() {
                mark(&self.dirty, name);
            }
        }
    }

    fn copy(&self, name: &str) -> Result<()> {
        let source = self.primary.as_ref();
        let target = self.mirror.as_ref();
        match name {
            User::COLLECTION => target.users().replace_all(&source.users().all()?),
            Category::COLLECTION => target.categories().replace_all(&source.categories().all()?),
            Client::COLLECTION => target.clients().replace_all(&source.clients().all()?),
            Product::COLLECTION => target.products().replace_all(&source.products().all()?),
            _ => target.sales().replace_all(&source.sales().all()?),
        }
    }
}

pub struct MirroredStore {
    primary: Arc<dyn Store>,
    copier: Arc<Copier>,
    worker: Mutex<Option<(Sender<()>, JoinHandle<()>)>>,
    users: MirroredRepository<User>,
    categories: MirroredRepository<Category>,
    clients: MirroredRepository<Client>,
    products: MirroredRepository<Product>,
    sales: MirroredRepository<Sale>,
}

impl MirroredStore {
    pub fn new(primary: Arc<dyn Store>, mirror: DocumentStore) -> Self {
        Self::with_interval(primary, mirror, INTERVAL)
    }

    pub fn with_interval(
        primary: Arc<dyn Store>,
        mirror: DocumentStore,
        interval: Duration,
    ) -> Self {
        let dirty: Dirty = Arc::new(Mutex::new(vec![
            User::COLLECTION,
            Category::COLLECTION,
            Client::COLLECTION,
            Product::COLLECTION,
            Sale::COLLECTION,
        ]));
        let copier = Arc::new(Copier {
            primary: primary.clone(),
            mirror: Arc::new(mirror),
            dirty: dirty.clone(),
        });
        Self {
            worker: Mutex::new(Some(spawn(copier.clone(), interval))),
            copier,
            users: wrap(&primary, &dirty, |store| store.users()),
            categories: wrap(&primary, &dirty, |store| store.categories()),
            clients: wrap(&primary, &dirty, |store| store.clients()),
            products: wrap(&primary, &dirty, |store| store.products()),
            sales: wrap(&primary, &dirty, |store| store.sales()),
            primary,
        }
    }
}

impl Store for MirroredStore {
    fn kind(&self) -> BackendKind {
        self.primary.kind()
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

    fn ping(&self) -> Result<()> {
        self.primary.ping()
    }

    fn close(&self) {
        let Some((stop, worker)) = lock(&self.worker).take() else {
            return;
        };
        drop(stop);
        let _ = worker.join();
        self.copier.flush();
        self.primary.close();
    }
}

fn spawn(copier: Arc<Copier>, interval: Duration) -> (Sender<()>, JoinHandle<()>) {
    let (stop, stopped) = mpsc::channel();
    let worker = thread::spawn(move || {
        copier.flush();
        while let Err(RecvTimeoutError::Timeout) = stopped.recv_timeout(interval) {
            copier.flush();
        }
    });
    (stop, worker)
}

fn wrap<T: Entity>(
    primary: &Arc<dyn Store>,
    dirty: &Dirty,
    select: Select<T>,
) -> MirroredRepository<T> {
    MirroredRepository {
        primary: primary.clone(),
        select,
        dirty: dirty.clone(),
    }
}

fn mark(dirty: &Dirty, name: &'static str) {
    let mut names = lock(dirty);
    if !names.contains(&name) {
        names.push(name);
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
