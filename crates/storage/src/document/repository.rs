use std::marker::PhantomData;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use pos_domain::Entity;
use serde::Serialize;
use serde_json::Value;

use super::FileBackend;
use crate::{Repository, Result, StorageError};

const SCHEMA_VERSION: u32 = 1;

#[derive(Serialize)]
struct Document<'a, T> {
    schema_version: u32,
    items: &'a [T],
}

pub struct Loaded<T> {
    pub items: Vec<T>,
    pub revision: Option<String>,
}

pub struct DocumentRepository<T> {
    files: Arc<dyn FileBackend>,
    lock: Arc<Mutex<()>>,
    entity: PhantomData<fn() -> T>,
}

impl<T: Entity> DocumentRepository<T> {
    pub fn new(files: Arc<dyn FileBackend>, lock: Arc<Mutex<()>>) -> Self {
        Self {
            files,
            lock,
            entity: PhantomData,
        }
    }

    pub(super) fn load(&self) -> Result<Loaded<T>> {
        let name = Self::file_name();
        let Some(snapshot) = self.files.read(&name)? else {
            return Ok(Loaded {
                items: Vec::new(),
                revision: None,
            });
        };
        let Ok(value) = serde_json::from_slice::<Value>(&snapshot.content) else {
            if self.files.quarantine(&name)? {
                return Ok(Loaded {
                    items: Vec::new(),
                    revision: None,
                });
            }
            return Err(StorageError::Data(format!("{name} no es JSON")));
        };
        Ok(Loaded {
            items: serde_json::from_value(items_of(value))?,
            revision: Some(snapshot.revision),
        })
    }

    pub(super) fn store(&self, items: &[T], revision: Option<&str>) -> Result<()> {
        let document = Document {
            schema_version: SCHEMA_VERSION,
            items,
        };
        let content = serde_json::to_vec_pretty(&document)?;
        self.files.write(&Self::file_name(), &content, revision)
    }

    fn file_name() -> String {
        format!("{}.json", T::COLLECTION)
    }

    fn guard(&self) -> MutexGuard<'_, ()> {
        self.lock.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl<T: Entity> Repository<T> for DocumentRepository<T> {
    fn all(&self) -> Result<Vec<T>> {
        let _guard = self.guard();
        Ok(self.load()?.items)
    }

    fn find(&self, id: &str) -> Result<Option<T>> {
        Ok(self.all()?.into_iter().find(|item| item.id() == id))
    }

    fn upsert(&self, item: &T) -> Result<()> {
        self.upsert_many(std::slice::from_ref(item))
    }

    fn upsert_many(&self, items: &[T]) -> Result<()> {
        let _guard = self.guard();
        let mut loaded = self.load()?;
        for item in items {
            merge(&mut loaded.items, item);
        }
        self.store(&loaded.items, loaded.revision.as_deref())
    }

    fn replace_all(&self, items: &[T]) -> Result<()> {
        let _guard = self.guard();
        let revision = self.files.read(&Self::file_name())?.map(|s| s.revision);
        self.store(items, revision.as_deref())
    }

    fn delete(&self, id: &str) -> Result<bool> {
        let _guard = self.guard();
        let mut loaded = self.load()?;
        let before = loaded.items.len();
        loaded.items.retain(|item| item.id() != id);
        if loaded.items.len() == before {
            return Ok(false);
        }
        self.store(&loaded.items, loaded.revision.as_deref())?;
        Ok(true)
    }

    fn count(&self) -> Result<usize> {
        Ok(self.all()?.len())
    }
}

fn merge<T: Entity>(items: &mut Vec<T>, item: &T) {
    match items.iter_mut().find(|current| current.id() == item.id()) {
        Some(current) => *current = item.clone(),
        None => items.push(item.clone()),
    }
}

fn items_of(mut value: Value) -> Value {
    if value.is_array() {
        return value;
    }
    match value.get_mut("items") {
        Some(items) => items.take(),
        None => Value::Array(Vec::new()),
    }
}
