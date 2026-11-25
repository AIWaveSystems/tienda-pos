use mongodb::bson::{doc, Document};
use pos_domain::{now, Sale, StockMovement};

use super::store::{failure, MongoStore};
use crate::{Repository, Result, SaleChange, StockOperation, StorageError};

impl MongoStore {
    pub(super) fn apply(&self, operation: &StockOperation) -> Result<()> {
        if let SaleChange::Void(sale) = operation.sale {
            self.mark_voided(sale)?;
        }
        let mut moved = Vec::new();
        for movement in operation.movements {
            match self.take(movement, operation.guards(movement)) {
                Ok(()) => moved.push(movement),
                Err(failure) => {
                    self.undo(operation, &moved)?;
                    return Err(failure);
                }
            }
        }
        let outcome = self.record(operation);
        if outcome.is_err() {
            self.undo(operation, &moved)?;
        }
        outcome
    }

    fn record(&self, operation: &StockOperation) -> Result<()> {
        self.movements.upsert_many(operation.movements)?;
        match operation.sale {
            SaleChange::Register(sale) => self.sales.upsert(sale),
            _ => Ok(()),
        }
    }

    fn mark_voided(&self, sale: &Sale) -> Result<()> {
        let moment = sale.voided_at.unwrap_or_else(now);
        let outcome = self
            .sales
            .collection()
            .update_one(
                doc! { "_id": &sale.id, "voided_at": { "$in": ["", null] } },
                doc! { "$set": { "voided_at": moment.format("%Y-%m-%dT%H:%M:%S%.f").to_string() } },
            )
            .run()
            .map_err(failure)?;
        if outcome.matched_count == 0 {
            return Err(StorageError::AlreadyVoided);
        }
        Ok(())
    }

    fn take(&self, movement: &StockMovement, guarded: bool) -> Result<()> {
        let mut filter = doc! { "_id": &movement.product_id };
        if guarded {
            filter.insert("stock", doc! { "$gte": -movement.quantity });
        }
        let matched = self.move_stock(filter, movement.quantity)?;
        if matched == 0 && guarded {
            return Err(StorageError::Stock(movement.product_name.clone()));
        }
        Ok(())
    }

    fn undo(&self, operation: &StockOperation, moved: &[&StockMovement]) -> Result<()> {
        for movement in moved {
            self.move_stock(doc! { "_id": &movement.product_id }, -movement.quantity)?;
        }
        for movement in operation.movements {
            self.movements.delete(&movement.id)?;
        }
        if let SaleChange::Void(sale) = operation.sale {
            self.sales
                .collection()
                .update_one(
                    doc! { "_id": &sale.id },
                    doc! { "$set": { "voided_at": "" } },
                )
                .run()
                .map_err(failure)?;
        }
        Ok(())
    }

    fn move_stock(&self, filter: Document, delta: i64) -> Result<u64> {
        let outcome = self
            .products
            .collection()
            .update_one(filter, doc! { "$inc": { "stock": delta } })
            .run()
            .map_err(failure)?;
        Ok(outcome.matched_count)
    }
}
