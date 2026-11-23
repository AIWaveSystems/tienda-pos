use pos_domain::Entity;

use crate::Result;

pub trait Repository<T: Entity>: Send + Sync {
    fn all(&self) -> Result<Vec<T>>;

    fn find(&self, id: &str) -> Result<Option<T>>;

    fn upsert(&self, item: &T) -> Result<()>;

    fn upsert_many(&self, items: &[T]) -> Result<()>;

    fn replace_all(&self, items: &[T]) -> Result<()>;

    fn delete(&self, id: &str) -> Result<bool>;

    fn count(&self) -> Result<usize>;
}
