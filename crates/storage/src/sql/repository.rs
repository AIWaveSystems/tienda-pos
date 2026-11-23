use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use super::driver::{SqlDriver, Statement};
use super::schema::{SqlRecord, Table};
use super::value::{Kind, SqlValue};
use crate::{Repository, Result};

pub struct SqlRepository<T> {
    driver: Arc<dyn SqlDriver>,
    record: PhantomData<fn() -> T>,
}

impl<T: SqlRecord> SqlRepository<T> {
    pub fn new(driver: Arc<dyn SqlDriver>) -> Self {
        Self {
            driver,
            record: PhantomData,
        }
    }

    fn fetch(&self, id: Option<&str>) -> Result<Vec<T>> {
        let rows = self.select(&T::TABLE, "id", "created_at, id", id)?;
        let mut items = rows
            .into_iter()
            .map(|mut row| T::from_row(&mut row))
            .collect::<Result<Vec<_>>>()?;
        if let Some(child) = T::CHILD {
            self.attach(&child, id, &mut items)?;
        }
        Ok(items)
    }

    fn attach(&self, child: &Table, id: Option<&str>, items: &mut [T]) -> Result<()> {
        let owner = child.columns[0].name;
        let positions: HashMap<String, usize> = items
            .iter()
            .enumerate()
            .map(|(position, item)| (item.id().to_owned(), position))
            .collect();
        for mut row in self.select(child, owner, child.primary_key, id)? {
            if let Some(position) = positions.get(&row.text()?) {
                items[*position].attach_child(&mut row)?;
            }
        }
        Ok(())
    }

    fn select(
        &self,
        table: &Table,
        key: &str,
        order: &str,
        id: Option<&str>,
    ) -> Result<Vec<super::value::Row>> {
        let filter = match id {
            Some(_) => format!(" WHERE {key} = {}", self.driver.dialect().placeholder(1)),
            None => String::new(),
        };
        let statement = Statement::new(
            format!(
                "SELECT {} FROM {}{filter} ORDER BY {order}",
                table.column_list(),
                table.name
            ),
            id.map(SqlValue::from).into_iter().collect(),
        );
        self.driver.query(&statement, &table.kinds())
    }

    pub fn save_statements(&self, item: &T) -> Vec<Statement> {
        let dialect = self.driver.dialect();
        let mut statements = vec![Statement::new(dialect.upsert(&T::TABLE), item.values())];
        if let Some(child) = T::CHILD {
            statements.push(self.delete_statement(&child, item.id()));
            let insert = dialect.insert(&child);
            statements.extend(
                item.child_values()
                    .into_iter()
                    .map(|values| Statement::new(insert.clone(), values)),
            );
        }
        statements
    }

    fn delete_statement(&self, table: &Table, id: &str) -> Statement {
        Statement::new(
            format!(
                "DELETE FROM {} WHERE {} = {}",
                table.name,
                table.columns[0].name,
                self.driver.dialect().placeholder(1)
            ),
            vec![id.into()],
        )
    }
}

impl<T: SqlRecord> Repository<T> for SqlRepository<T> {
    fn all(&self) -> Result<Vec<T>> {
        self.fetch(None)
    }

    fn find(&self, id: &str) -> Result<Option<T>> {
        Ok(self.fetch(Some(id))?.pop())
    }

    fn upsert(&self, item: &T) -> Result<()> {
        self.driver.transaction(&self.save_statements(item))?;
        Ok(())
    }

    fn upsert_many(&self, items: &[T]) -> Result<()> {
        let statements: Vec<Statement> = items
            .iter()
            .flat_map(|item| self.save_statements(item))
            .collect();
        self.driver.transaction(&statements)?;
        Ok(())
    }

    fn replace_all(&self, items: &[T]) -> Result<()> {
        let mut statements = vec![Statement::new(
            format!("DELETE FROM {}", T::TABLE.name),
            Vec::new(),
        )];
        statements.extend(items.iter().flat_map(|item| self.save_statements(item)));
        self.driver.transaction(&statements)?;
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<bool> {
        let affected = self
            .driver
            .transaction(&[self.delete_statement(&T::TABLE, id)])?;
        Ok(affected.iter().sum::<u64>() > 0)
    }

    fn count(&self) -> Result<usize> {
        let statement = Statement::new(
            format!("SELECT COUNT(*) FROM {}", T::TABLE.name),
            Vec::new(),
        );
        let mut rows = self.driver.query(&statement, &[Kind::Int])?;
        match rows.first_mut() {
            Some(row) => Ok(row.int()? as usize),
            None => Ok(0),
        }
    }
}
