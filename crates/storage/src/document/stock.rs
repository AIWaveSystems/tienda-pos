use std::sync::PoisonError;

use pos_domain::{now, Entity, Product, Sale, StockMovement};

use super::repository::{DocumentRepository, Loaded};
use super::DocumentStore;
use crate::{Result, SaleChange, StockOperation, StorageError};

impl DocumentStore {
    pub(super) fn apply(&self, operation: &StockOperation) -> Result<()> {
        let _guard = self.lock.lock().unwrap_or_else(PoisonError::into_inner);
        let products = self.products.load()?;
        let movements = self.movements.load()?;
        let sales = self.sales.load()?;
        let stock = moved(&products.items, operation)?;
        let sold = changed(&sales.items, operation.sale)?;
        let history = [movements.items.as_slice(), operation.movements].concat();
        write(&self.products, &products, &stock)?;
        if let Err(failure) = write(&self.movements, &movements, &history) {
            restore(&self.products, &products.items)?;
            return Err(failure);
        }
        if matches!(operation.sale, SaleChange::None) {
            return Ok(());
        }
        if let Err(failure) = write(&self.sales, &sales, &sold) {
            restore(&self.products, &products.items)?;
            restore(&self.movements, &movements.items)?;
            return Err(failure);
        }
        Ok(())
    }
}

fn moved(products: &[Product], operation: &StockOperation) -> Result<Vec<Product>> {
    let mut updated = products.to_vec();
    for movement in operation.movements {
        take(&mut updated, movement, operation.guards(movement))?;
    }
    Ok(updated)
}

fn take(products: &mut [Product], movement: &StockMovement, guarded: bool) -> Result<()> {
    let found = products
        .iter_mut()
        .find(|product| product.id == movement.product_id);
    match found {
        Some(product) if !guarded || product.stock >= -movement.quantity => {
            product.stock += movement.quantity;
            product.updated_at = now();
            Ok(())
        }
        None if !guarded => Ok(()),
        _ => Err(StorageError::Stock(movement.product_name.clone())),
    }
}

fn changed(sales: &[Sale], change: SaleChange) -> Result<Vec<Sale>> {
    let mut updated = sales.to_vec();
    match change {
        SaleChange::None => {}
        SaleChange::Register(sale) => updated.push(sale.clone()),
        SaleChange::Void(sale) => {
            let current = updated
                .iter_mut()
                .find(|item| item.id == sale.id && item.voided_at.is_none())
                .ok_or(StorageError::AlreadyVoided)?;
            current.voided_at = sale.voided_at;
        }
    }
    Ok(updated)
}

fn write<T: Entity>(
    repository: &DocumentRepository<T>,
    loaded: &Loaded<T>,
    items: &[T],
) -> Result<()> {
    repository.store(items, loaded.revision.as_deref())
}

fn restore<T: Entity>(repository: &DocumentRepository<T>, items: &[T]) -> Result<()> {
    let revision = repository.load()?.revision;
    repository.store(items, revision.as_deref())
}
