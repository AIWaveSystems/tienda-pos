use pos_config::BackendKind;
use pos_domain::{Category, Client, Product, Sale, StockMovement, User};

use crate::{Repository, Result, StockOperation};

pub trait Store: Send + Sync {
    fn kind(&self) -> BackendKind;

    fn users(&self) -> &dyn Repository<User>;

    fn categories(&self) -> &dyn Repository<Category>;

    fn clients(&self) -> &dyn Repository<Client>;

    fn products(&self) -> &dyn Repository<Product>;

    fn sales(&self) -> &dyn Repository<Sale>;

    fn movements(&self) -> &dyn Repository<StockMovement>;

    fn apply_stock(&self, operation: &StockOperation) -> Result<()>;

    fn ping(&self) -> Result<()>;

    fn close(&self) {}
}
