use std::time::Duration;

use sqlx::mysql::{
    MySqlArguments, MySqlConnectOptions, MySqlPool, MySqlPoolOptions, MySqlRow, MySqlSslMode,
};
use sqlx::query::Query;
use sqlx::{Executor, MySql, Row as _};
use tokio::runtime::Runtime;

use super::dialect::Dialect;
use super::driver::{failure, SqlDriver, Statement};
use super::store::SqlConnection;
use super::value::{Kind, Row, SqlValue};
use crate::Result;

pub struct MySqlDriver {
    runtime: Runtime,
    pool: MySqlPool,
}

impl MySqlDriver {
    pub fn connect(settings: &SqlConnection) -> Result<Self> {
        let runtime = Runtime::new()?;
        let options = MySqlConnectOptions::new()
            .host(&settings.host)
            .port(settings.port)
            .database(&settings.database)
            .username(&settings.user)
            .password(&settings.password)
            .charset("utf8mb4")
            .ssl_mode(if settings.ssl {
                MySqlSslMode::VerifyIdentity
            } else {
                MySqlSslMode::Preferred
            });
        let pool = runtime
            .block_on(
                MySqlPoolOptions::new()
                    .max_connections(5)
                    .acquire_timeout(Duration::from_secs(8))
                    .connect_with(options),
            )
            .map_err(failure)?;
        Ok(Self { runtime, pool })
    }
}

impl SqlDriver for MySqlDriver {
    fn dialect(&self) -> Dialect {
        Dialect::MySql
    }

    fn query(&self, statement: &Statement, kinds: &[Kind]) -> Result<Vec<Row>> {
        let rows = self
            .runtime
            .block_on(bind(statement).fetch_all(&self.pool))
            .map_err(failure)?;
        rows.iter()
            .map(|row| decode(row, kinds).map_err(failure))
            .collect()
    }

    fn transaction(&self, statements: &[Statement]) -> Result<Vec<u64>> {
        self.runtime
            .block_on(async {
                let mut transaction = self.pool.begin().await?;
                let mut affected = Vec::with_capacity(statements.len());
                for statement in statements {
                    let outcome = bind(statement).execute(&mut *transaction).await?;
                    affected.push(outcome.rows_affected());
                }
                transaction.commit().await?;
                Ok(affected)
            })
            .map_err(failure)
    }

    fn migrate(&self, statements: &[String]) -> Result<()> {
        self.runtime
            .block_on(async {
                for statement in statements {
                    self.pool.execute(statement.as_str()).await?;
                }
                Ok(())
            })
            .map_err(failure)
    }

    fn close(&self) {
        self.runtime.block_on(self.pool.close());
    }
}

fn bind(statement: &Statement) -> Query<'_, MySql, MySqlArguments> {
    statement
        .values
        .iter()
        .fold(sqlx::query(&statement.sql), |query, value| match value {
            SqlValue::Text(value) => query.bind(value),
            SqlValue::Int(value) => query.bind(value),
            SqlValue::Bool(value) => query.bind(value),
            SqlValue::Money(value) => query.bind(value),
            SqlValue::Moment(value) => query.bind(value),
        })
}

fn decode(row: &MySqlRow, kinds: &[Kind]) -> sqlx::Result<Row> {
    let values = kinds
        .iter()
        .enumerate()
        .map(|(index, kind)| {
            Ok(match kind {
                Kind::Id | Kind::Short | Kind::Long => SqlValue::Text(row.try_get(index)?),
                Kind::Int => SqlValue::Int(row.try_get(index)?),
                Kind::Bool => SqlValue::Bool(row.try_get(index)?),
                Kind::Money => SqlValue::Money(row.try_get(index)?),
                Kind::Moment | Kind::OptionalMoment => SqlValue::Moment(row.try_get(index)?),
            })
        })
        .collect::<sqlx::Result<Vec<_>>>()?;
    Ok(Row::new(values))
}
