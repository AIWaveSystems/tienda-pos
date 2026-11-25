use pos_domain::{Sale, StockMovement};

#[derive(Clone, Copy)]
pub enum SaleChange<'a> {
    None,
    Register(&'a Sale),
    Void(&'a Sale),
}

#[derive(Clone, Copy)]
pub struct StockOperation<'a> {
    pub sale: SaleChange<'a>,
    pub movements: &'a [StockMovement],
    pub allow_negative: bool,
}

impl StockOperation<'_> {
    pub fn guards(&self, movement: &StockMovement) -> bool {
        movement.quantity < 0 && !self.allow_negative
    }
}
