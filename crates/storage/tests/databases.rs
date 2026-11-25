mod common;

use std::env;

use pos_config::model::{MongoSettings, SqlSettings};
use pos_storage::{MongoConnection, MongoStore, SqlConnection, SqlStore, Store};

const PASSWORD: &str = "p@ss:w/rd";

fn setting(engine: &str, name: &str, fallback: &str) -> String {
    env::var(format!("POS_TEST_{engine}_{name}")).unwrap_or_else(|_| fallback.to_owned())
}

fn sql(engine: &str, port: &str) -> SqlConnection {
    let settings = SqlSettings {
        host: setting(engine, "HOST", "127.0.0.1"),
        port: setting(engine, "PORT", port).parse().unwrap(),
        database: setting(engine, "DATABASE", "pos"),
        user: setting(engine, "USER", "pos"),
        ssl: false,
    };
    assert_disposable(&settings.database);
    SqlConnection::new(&settings, &setting(engine, "PASSWORD", PASSWORD))
}

fn assert_disposable(database: &str) {
    assert!(
        database == "pos" || database.ends_with("_test"),
        "las pruebas borran datos: use una base terminada en _test"
    );
}

#[test]
#[ignore = "requiere un servidor MySQL"]
fn mysql_fulfills_the_contract() {
    let connection = sql("MYSQL", "33306");
    common::contract(&SqlStore::mysql(&connection).unwrap());
    let reopened = SqlStore::mysql(&connection).unwrap();
    reopened.ping().unwrap();
    reopened.close();
    assert!(reopened.ping().is_err());
}

#[test]
#[ignore = "requiere un servidor PostgreSQL"]
fn postgres_fulfills_the_contract() {
    let connection = sql("POSTGRES", "35432");
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
        host: setting("MONGO", "HOST", "127.0.0.1"),
        port: setting("MONGO", "PORT", "37017").parse().unwrap(),
        database: setting("MONGO", "DATABASE", "pos"),
        user: setting("MONGO", "USER", "pos"),
        use_srv: false,
    };
    assert_disposable(&settings.database);
    let connection = MongoConnection::new(&settings, &setting("MONGO", "PASSWORD", PASSWORD));
    common::contract(&MongoStore::connect(&connection).unwrap());
    MongoStore::connect(&connection).unwrap();
}

#[test]
fn missing_database_is_a_configuration_error() {
    let mut connection = sql("NONE", "1");
    connection.database.clear();
    assert!(SqlStore::mysql(&connection).is_err());
    assert!(SqlStore::postgres(&connection).is_err());
}
