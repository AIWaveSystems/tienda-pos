mod common;

use std::env;

use pos_config::model::{MongoSettings, SqlSettings};
use pos_storage::{MongoConnection, MongoStore, SqlConnection, SqlStore, Store};

const PASSWORD: &str = "p@ss:w/rd";

fn port(variable: &str, fallback: u16) -> u16 {
    env::var(variable)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(fallback)
}

fn sql(port: u16) -> SqlConnection {
    let settings = SqlSettings {
        host: "127.0.0.1".into(),
        port,
        database: "pos".into(),
        user: "pos".into(),
        ssl: false,
    };
    SqlConnection::new(&settings, PASSWORD)
}

#[test]
#[ignore = "requiere un servidor MySQL"]
fn mysql_fulfills_the_contract() {
    let connection = sql(port("POS_TEST_MYSQL_PORT", 33306));
    common::contract(&SqlStore::mysql(&connection).unwrap());
    let reopened = SqlStore::mysql(&connection).unwrap();
    reopened.ping().unwrap();
    reopened.close();
    assert!(reopened.ping().is_err());
}

#[test]
#[ignore = "requiere un servidor PostgreSQL"]
fn postgres_fulfills_the_contract() {
    let connection = sql(port("POS_TEST_POSTGRES_PORT", 35432));
    common::contract(&SqlStore::postgres(&connection).unwrap());
    let reopened = SqlStore::postgres(&connection).unwrap();
    reopened.ping().unwrap();
    reopened.close();
    assert!(reopened.ping().is_err());
}

#[test]
#[ignore = "requiere un servidor MongoDB"]
fn mongo_fulfills_the_contract() {
    let settings = MongoSettings {
        host: "127.0.0.1".into(),
        port: port("POS_TEST_MONGO_PORT", 37017),
        database: "pos".into(),
        user: "pos".into(),
        use_srv: false,
    };
    let connection = MongoConnection::new(&settings, PASSWORD);
    common::contract(&MongoStore::connect(&connection).unwrap());
    MongoStore::connect(&connection).unwrap();
}

#[test]
fn missing_database_is_a_configuration_error() {
    let mut connection = sql(1);
    connection.database.clear();
    assert!(SqlStore::mysql(&connection).is_err());
    assert!(SqlStore::postgres(&connection).is_err());
}
