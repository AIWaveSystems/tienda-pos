use std::env;
use std::sync::Arc;

use pos_config::model::SqlSettings;
use pos_domain::{Product, Sale, SaleItem};
use pos_services::{Period, Services};
use pos_storage::{SqlConnection, SqlStore, Store};
use rust_decimal::Decimal;

fn setting(name: &str, fallback: &str) -> String {
    env::var(format!("POS_TEST_POSTGRES_{name}")).unwrap_or_else(|_| fallback.to_owned())
}

fn connect() -> Arc<dyn Store> {
    let database = setting("DATABASE", "pos");
    assert!(database == "pos" || database.ends_with("_test"));
    let settings = SqlSettings {
        host: setting("HOST", "127.0.0.1"),
        port: setting("PORT", "35432").parse().unwrap(),
        database,
        user: setting("USER", "pos"),
        ssl: false,
    };
    let connection = SqlConnection::new(&settings, &setting("PASSWORD", "p@ss:w/rd"));
    Arc::new(SqlStore::postgres(&connection).unwrap())
}

fn clean(store: &dyn Store) {
    store.users().replace_all(&[]).unwrap();
    store.products().replace_all(&[]).unwrap();
    store.sales().replace_all(&[]).unwrap();
    store.movements().replace_all(&[]).unwrap();
}

#[test]
#[ignore = "requiere un servidor PostgreSQL"]
fn a_business_day_on_postgres() {
    let store = connect();
    clean(store.as_ref());
    let services = Services::new(store.clone());
    services.auth.ensure_default_user().unwrap();
    let user = services.auth.authenticate("admin", "admin123").unwrap();
    assert!(user.unwrap().is_default);
    let bread = services
        .inventory
        .save(Product {
            name: "Pan".into(),
            price: Decimal::new(1999, 2),
            cost: Decimal::from(10),
            stock: 10,
            ..Product::default()
        })
        .unwrap();
    let order = |quantity: i64| Sale {
        items: vec![SaleItem::new(&bread.id, "Pan", quantity, bread.price)],
        seller: "Ana".into(),
        ..Sale::default()
    };
    let rate = Decimal::new(19, 2);
    let kept = services.sales.register(order(3), rate, false).unwrap();
    let undone = services.sales.register(order(2), rate, false).unwrap();
    assert!(services.sales.register(order(6), rate, false).is_err());
    services.sales.void(undone.clone(), "Luis").unwrap();
    assert!(services.sales.void(undone, "Luis").is_err());
    services
        .inventory
        .adjust_stock(&bread, -1, "Merma", "Ana")
        .unwrap();
    assert_eq!(
        services.inventory.find(&bread.id).unwrap().unwrap().stock,
        6
    );
    assert_eq!(services.inventory.movements().unwrap().len(), 4);
    let overview = services.reports.overview(5, Period::Today).unwrap();
    assert_eq!(overview.sales.count, 1);
    assert_eq!(overview.sales.total, kept.total);
    assert_eq!(overview.sellers[0].name, "Ana");
    clean(store.as_ref());
    services.close();
}
