use std::collections::HashSet;

use pos_domain::{CashEntry, Category, Client, Product, Sale, StockMovement, User};

use super::driver::{SqlDriver, Statement};
use super::schema::{SqlRecord, Table};
use super::value::Kind;
use crate::Result;

pub fn migrate(driver: &dyn SqlDriver) -> Result<()> {
    record::<User>(driver)?;
    record::<Category>(driver)?;
    record::<Client>(driver)?;
    record::<Product>(driver)?;
    record::<Sale>(driver)?;
    record::<CashEntry>(driver)?;
    record::<StockMovement>(driver)
}

fn record<T: SqlRecord>(driver: &dyn SqlDriver) -> Result<()> {
    table(driver, &T::TABLE)?;
    match T::CHILD {
        Some(child) => table(driver, &child),
        None => Ok(()),
    }
}

fn table(driver: &dyn SqlDriver, table: &Table) -> Result<()> {
    let dialect = driver.dialect();
    driver.migrate(&dialect.create(table))?;
    let query = Statement::new(dialect.columns_query(), vec![table.name.into()]);
    let existing = driver
        .query(&query, &[Kind::Short])?
        .iter_mut()
        .map(|row| row.text())
        .collect::<Result<HashSet<String>>>()?;
    let missing: Vec<String> = table
        .columns
        .iter()
        .filter(|column| !existing.contains(column.name))
        .map(|column| dialect.add_column(table, column))
        .collect();
    driver.migrate(&missing)
}
