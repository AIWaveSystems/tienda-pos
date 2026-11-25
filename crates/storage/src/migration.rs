use crate::{Result, Store};

pub fn copy_all(source: &dyn Store, target: &dyn Store) -> Result<()> {
    target.users().upsert_many(&source.users().all()?)?;
    target
        .categories()
        .upsert_many(&source.categories().all()?)?;
    target.clients().upsert_many(&source.clients().all()?)?;
    target.products().upsert_many(&source.products().all()?)?;
    target.movements().upsert_many(&source.movements().all()?)?;
    target.sales().upsert_many(&source.sales().all()?)
}
