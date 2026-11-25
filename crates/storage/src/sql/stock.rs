use pos_domain::{now, Product, Sale, StockMovement};

use super::dialect::Dialect;
use super::driver::{Check, Statement};
use super::schema::SqlRecord;
use crate::StockOperation;

pub fn statements(dialect: Dialect, operation: &StockOperation) -> Vec<Statement> {
    let mut movements: Vec<&StockMovement> = operation.movements.iter().collect();
    movements.sort_by(|a, b| a.product_id.cmp(&b.product_id));
    let insert = dialect.insert(&StockMovement::TABLE);
    let mut statements: Vec<Statement> = movements
        .into_iter()
        .map(|movement| move_stock(dialect, movement, operation.guards(movement)))
        .collect();
    statements.extend(
        operation
            .movements
            .iter()
            .map(|movement| Statement::new(insert.clone(), movement.values())),
    );
    statements
}

pub fn void(dialect: Dialect, sale: &Sale) -> Statement {
    let sql = format!(
        "UPDATE {} SET voided_at = {} WHERE id = {} AND voided_at IS NULL",
        Sale::TABLE.name,
        dialect.placeholder(1),
        dialect.placeholder(2)
    );
    Statement::new(sql, vec![sale.voided_at.into(), (&sale.id).into()]).requiring(Check::NotVoided)
}

fn move_stock(dialect: Dialect, movement: &StockMovement, guarded: bool) -> Statement {
    let mut sql = format!(
        "UPDATE {} SET stock = stock + {}, updated_at = {} WHERE id = {}",
        Product::TABLE.name,
        dialect.placeholder(1),
        dialect.placeholder(2),
        dialect.placeholder(3)
    );
    let mut values = vec![
        movement.quantity.into(),
        now().into(),
        (&movement.product_id).into(),
    ];
    if !guarded {
        return Statement::new(sql, values);
    }
    sql.push_str(&format!(" AND stock >= {}", dialect.placeholder(4)));
    values.push((-movement.quantity).into());
    Statement::new(sql, values).requiring(Check::Stock(movement.product_name.clone()))
}
