use std::sync::Arc;

use pos_config::model::SqlSettings;
use pos_config::BackendKind;
use pos_domain::{Category, Client, Product, Sale, User};

use super::dialect::Dialect;
use super::driver::{SqlDriver, Statement};
use super::mysql::MySqlDriver;
use super::postgres::PostgresDriver;
use super::repository::SqlRepository;
use super::schema::SqlRecord;
use crate::{Repository, Result, StorageError, Store};

pub struct SqlConnection {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: String,
    pub ssl: bool,
}

impl SqlConnection {
    pub fn new(settings: &SqlSettings, password: &str) -> Self {
        Self {
            host: settings.host.clone(),
            port: settings.port,
            database: settings.database.clone(),
            user: settings.user.clone(),
            password: password.to_owned(),
            ssl: settings.ssl,
        }
    }

    fn validate(&self, label: &str) -> Result<()> {
        if self.database.is_empty() || self.user.is_empty() {
            return Err(StorageError::Config(format!(
                "Falta configurar la base de datos {label} en Configuracion > Almacenamiento."
            )));
        }
        Ok(())
    }
}

pub struct SqlStore {
    kind: BackendKind,
    driver: Arc<dyn SqlDriver>,
    users: SqlRepository<User>,
    categories: SqlRepository<Category>,
    clients: SqlRepository<Client>,
    products: SqlRepository<Product>,
    sales: SqlRepository<Sale>,
}

impl SqlStore {
    pub fn mysql(connection: &SqlConnection) -> Result<Self> {
        connection.validate(BackendKind::Mysql.label())?;
        Self::open(
            BackendKind::Mysql,
            Arc::new(MySqlDriver::connect(connection)?),
        )
    }

    pub fn postgres(connection: &SqlConnection) -> Result<Self> {
        connection.validate(BackendKind::Postgresql.label())?;
        Self::open(
            BackendKind::Postgresql,
            Arc::new(PostgresDriver::connect(connection)?),
        )
    }

    fn open(kind: BackendKind, driver: Arc<dyn SqlDriver>) -> Result<Self> {
        driver.migrate(&schema(driver.dialect()))?;
        Ok(Self {
            kind,
            users: SqlRepository::new(driver.clone()),
            categories: SqlRepository::new(driver.clone()),
            clients: SqlRepository::new(driver.clone()),
            products: SqlRepository::new(driver.clone()),
            sales: SqlRepository::new(driver.clone()),
            driver,
        })
    }
}

fn schema(dialect: Dialect) -> Vec<String> {
    let mut statements = tables::<User>(dialect);
    statements.extend(tables::<Category>(dialect));
    statements.extend(tables::<Client>(dialect));
    statements.extend(tables::<Product>(dialect));
    statements.extend(tables::<Sale>(dialect));
    statements
}

fn tables<T: SqlRecord>(dialect: Dialect) -> Vec<String> {
    let mut statements = dialect.create(&T::TABLE);
    if let Some(child) = T::CHILD {
        statements.extend(dialect.create(&child));
    }
    statements
}

impl Store for SqlStore {
    fn kind(&self) -> BackendKind {
        self.kind
    }

    fn users(&self) -> &dyn Repository<User> {
        &self.users
    }

    fn categories(&self) -> &dyn Repository<Category> {
        &self.categories
    }

    fn clients(&self) -> &dyn Repository<Client> {
        &self.clients
    }

    fn products(&self) -> &dyn Repository<Product> {
        &self.products
    }

    fn sales(&self) -> &dyn Repository<Sale> {
        &self.sales
    }

    fn ping(&self) -> Result<()> {
        self.driver
            .query(&Statement::new("SELECT 1", Vec::new()), &[])?;
        Ok(())
    }

    fn close(&self) {
        self.driver.close();
    }
}
