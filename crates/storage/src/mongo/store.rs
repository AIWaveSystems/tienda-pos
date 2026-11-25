use std::time::Duration;

use mongodb::bson::{doc, Document};
use mongodb::options::{ClientOptions, IndexOptions};
use mongodb::sync::{Client as MongoClient, Collection, Database};
use mongodb::IndexModel;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use pos_config::model::MongoSettings;
use pos_config::BackendKind;
use pos_domain::{Category, Client, Product, Sale, StockMovement, User};

use super::repository::MongoRepository;
use crate::{Repository, Result, StockOperation, StorageError, Store};

pub struct MongoConnection {
    pub uri: String,
    pub database: String,
}

impl MongoConnection {
    pub fn new(settings: &MongoSettings, password: &str) -> Self {
        let encode = |text: &str| utf8_percent_encode(text, NON_ALPHANUMERIC).to_string();
        let account = if settings.user.is_empty() {
            String::new()
        } else {
            format!("{}:{}@", encode(&settings.user), encode(password))
        };
        let uri = if settings.use_srv {
            format!("mongodb+srv://{account}{}", settings.host)
        } else {
            format!("mongodb://{account}{}:{}", settings.host, settings.port)
        };
        Self {
            uri,
            database: settings.database.clone(),
        }
    }
}

pub struct MongoStore {
    database: Database,
    users: MongoRepository<User>,
    categories: MongoRepository<Category>,
    clients: MongoRepository<Client>,
    pub(super) products: MongoRepository<Product>,
    pub(super) sales: MongoRepository<Sale>,
    pub(super) movements: MongoRepository<StockMovement>,
}

impl MongoStore {
    pub fn connect(connection: &MongoConnection) -> Result<Self> {
        if connection.database.is_empty() {
            return Err(StorageError::Config(
                "Falta configurar la base de datos de MongoDB en Configuracion > Almacenamiento."
                    .to_owned(),
            ));
        }
        let mut options = ClientOptions::parse(&connection.uri)
            .run()
            .map_err(failure)?;
        options.server_selection_timeout = Some(Duration::from_secs(4));
        let client = MongoClient::with_options(options).map_err(failure)?;
        let database = client.database(&connection.database);
        let store = Self {
            users: MongoRepository::new(&database),
            categories: MongoRepository::new(&database),
            clients: MongoRepository::new(&database),
            products: MongoRepository::new(&database),
            sales: MongoRepository::new(&database),
            movements: MongoRepository::new(&database),
            database,
        };
        store.ping()?;
        store.create_indexes()?;
        Ok(store)
    }

    fn create_indexes(&self) -> Result<()> {
        index(self.users.collection(), "username", true)?;
        index(self.clients.collection(), "tax_id", false)?;
        index(self.products.collection(), "barcode", false)?;
        index(self.products.collection(), "category_id", false)?;
        index(self.movements.collection(), "product_id", false)?;
        index(self.sales.collection(), "created_at", false)?;
        index(self.sales.collection(), "client_id", false)?;
        index(self.sales.collection(), "items.product_id", false)
    }
}

impl Store for MongoStore {
    fn kind(&self) -> BackendKind {
        BackendKind::Mongodb
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

    fn movements(&self) -> &dyn Repository<StockMovement> {
        &self.movements
    }

    fn apply_stock(&self, operation: &StockOperation) -> Result<()> {
        self.apply(operation)
    }

    fn ping(&self) -> Result<()> {
        self.database
            .run_command(doc! { "ping": 1 })
            .run()
            .map_err(failure)?;
        Ok(())
    }
}

fn index(collection: &Collection<Document>, field: &str, unique: bool) -> Result<()> {
    let model = IndexModel::builder()
        .keys(doc! { field: 1 })
        .options(IndexOptions::builder().unique(unique).build())
        .build();
    collection.create_index(model).run().map_err(failure)?;
    Ok(())
}

pub fn failure(error: mongodb::error::Error) -> StorageError {
    let text = error.to_string();
    if text.contains("E11000") {
        return StorageError::Conflict(text);
    }
    StorageError::Connection(text)
}
