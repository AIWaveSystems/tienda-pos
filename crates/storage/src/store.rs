use pos_config::BackendKind;
use pos_domain::{Category, Client, Product, Sale, User};

use crate::{Repository, Result};

pub trait Store: Send + Sync {
    fn kind(&self) -> BackendKind;

    fn users(&self) -> &dyn Repository<User>;

    fn categories(&self) -> &dyn Repository<Category>;

    fn clients(&self) -> &dyn Repository<Client>;

    fn products(&self) -> &dyn Repository<Product>;

    fn sales(&self) -> &dyn Repository<Sale>;

    fn register_sale(&self, sale: &Sale, allow_negative: bool) -> Result<()>;

    fn ping(&self) -> Result<()>;

    fn close(&self) {}
}
