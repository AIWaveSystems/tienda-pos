use std::marker::PhantomData;

use mongodb::bson::{doc, Document};
use mongodb::sync::{Collection, Database};
use pos_domain::Entity;

use super::convert::{from_document, to_document};
use super::store::failure;
use crate::{Repository, Result};

pub struct MongoRepository<T> {
    collection: Collection<Document>,
    entity: PhantomData<fn() -> T>,
}

impl<T: Entity> MongoRepository<T> {
    pub fn new(database: &Database) -> Self {
        Self {
            collection: database.collection(&format!("pos_{}", T::COLLECTION)),
            entity: PhantomData,
        }
    }

    pub fn collection(&self) -> &Collection<Document> {
        &self.collection
    }
}

impl<T: Entity> Repository<T> for MongoRepository<T> {
    fn all(&self) -> Result<Vec<T>> {
        let cursor = self
            .collection
            .find(doc! {})
            .sort(doc! { "created_at": 1, "_id": 1 })
            .run()
            .map_err(failure)?;
        cursor
            .map(|entry| from_document(entry.map_err(failure)?))
            .collect()
    }

    fn find(&self, id: &str) -> Result<Option<T>> {
        self.collection
            .find_one(doc! { "_id": id })
            .run()
            .map_err(failure)?
            .map(from_document)
            .transpose()
    }

    fn upsert(&self, item: &T) -> Result<()> {
        self.collection
            .replace_one(doc! { "_id": item.id() }, to_document(item)?)
            .upsert(true)
            .run()
            .map_err(failure)?;
        Ok(())
    }

    fn upsert_many(&self, items: &[T]) -> Result<()> {
        items.iter().try_for_each(|item| self.upsert(item))
    }

    fn replace_all(&self, items: &[T]) -> Result<()> {
        self.upsert_many(items)?;
        let keep: Vec<&str> = items.iter().map(Entity::id).collect();
        self.collection
            .delete_many(doc! { "_id": { "$nin": keep } })
            .run()
            .map_err(failure)?;
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<bool> {
        let outcome = self
            .collection
            .delete_one(doc! { "_id": id })
            .run()
            .map_err(failure)?;
        Ok(outcome.deleted_count > 0)
    }

    fn count(&self) -> Result<usize> {
        let total = self
            .collection
            .count_documents(doc! {})
            .run()
            .map_err(failure)?;
        Ok(total as usize)
    }
}
