use pos_domain::{Product, SaleItem};

#[derive(Default)]
pub struct Cart {
    pub items: Vec<SaleItem>,
}

impl Cart {
    pub fn add(&mut self, product: &Product, allow_negative: bool) -> Result<usize, String> {
        if product.stock <= 0 && !allow_negative {
            return Err(format!("'{}' no tiene stock disponible", product.name));
        }
        let existing = self
            .items
            .iter()
            .position(|item| item.product_id == product.id);
        if let Some(index) = existing {
            let limit = (!allow_negative).then_some(product.stock);
            let quantity = self.items[index].quantity + 1;
            self.set_quantity(index, quantity, limit)?;
            return Ok(index);
        }
        self.items
            .push(SaleItem::new(&product.id, &product.name, 1, product.price));
        Ok(self.items.len() - 1)
    }

    pub fn set_quantity(
        &mut self,
        index: usize,
        quantity: i64,
        limit: Option<i64>,
    ) -> Result<(), String> {
        if quantity <= 0 {
            self.items.remove(index);
            return Ok(());
        }
        let item = &mut self.items[index];
        if let Some(limit) = limit.filter(|limit| quantity > *limit) {
            return Err(format!(
                "Solo hay {limit} unidad(es) de '{}'.",
                item.product_name
            ));
        }
        item.quantity = quantity;
        item.calculate();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanning_twice_increments_until_stock() {
        let product = Product {
            stock: 2,
            price: 10.into(),
            ..Product::default()
        };
        let mut cart = Cart::default();
        cart.add(&product, false).unwrap();
        cart.add(&product, false).unwrap();
        assert!(cart.add(&product, false).is_err());
        assert_eq!(cart.items.len(), 1);
        assert_eq!(cart.items[0].subtotal, 20.into());
        assert!(cart.add(&product, true).is_ok());
        cart.set_quantity(0, 0, None).unwrap();
        assert!(cart.items.is_empty());
    }
}
