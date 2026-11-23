use pos_domain::{
    Category, Client, Entity, PaymentMethod, Permission, Product, Role, Sale, SaleItem, User,
};
use pos_storage::{Repository, Store};
use rust_decimal::Decimal;

fn exercise<T>(repository: &dyn Repository<T>, first: T, second: T, changed: T)
where
    T: Entity + PartialEq + std::fmt::Debug,
{
    repository.replace_all(&[]).unwrap();
    assert_eq!(repository.count().unwrap(), 0);
    repository.upsert(&first).unwrap();
    assert_eq!(repository.find(first.id()).unwrap().as_ref(), Some(&first));
    repository.upsert(&changed).unwrap();
    assert_eq!(
        repository.find(first.id()).unwrap().as_ref(),
        Some(&changed)
    );
    repository
        .upsert_many(&[second.clone(), changed.clone()])
        .unwrap();
    assert_eq!(repository.count().unwrap(), 2);
    let all = repository.all().unwrap();
    assert!(all.contains(&second) && all.contains(&changed));
    assert!(repository.find("ausente").unwrap().is_none());
    assert!(!repository.delete("ausente").unwrap());
    assert!(repository.delete(second.id()).unwrap());
    repository.replace_all(&[second.clone()]).unwrap();
    assert_eq!(repository.all().unwrap(), vec![second]);
    repository.replace_all(&[]).unwrap();
}

pub fn contract(store: &dyn Store) {
    store.ping().unwrap();
    let user = User {
        username: "ana".into(),
        full_name: "Ana Ñandú".into(),
        password_hash: "pbkdf2$1$aa$bb".into(),
        role: Role::Personalizado,
        permissions: vec![Permission::Ventas, Permission::Clientes],
        ..User::default()
    };
    let mut changed = user.clone();
    changed.permissions = vec![Permission::Informes];
    changed.last_login = Some(changed.created_at);
    let other = User {
        username: "luis".into(),
        ..User::default()
    };
    exercise(store.users(), user, other, changed);

    let category = Category {
        name: "Bebidas".into(),
        ..Category::default()
    };
    let mut changed = category.clone();
    changed.active = false;
    exercise(store.categories(), category, Category::default(), changed);

    let client = Client {
        name: "Cañón's \"Tienda\"; DROP TABLE pos_clients".into(),
        tax_id: "900123".into(),
        ..Client::default()
    };
    let mut changed = client.clone();
    changed.notes = "línea 1\nlínea 2 😀".into();
    exercise(store.clients(), client, Client::default(), changed);

    let product = Product {
        name: "Café".into(),
        price: Decimal::new(1_234_567, 2),
        cost: Decimal::new(1, 1),
        stock: 7,
        barcode: "770".into(),
        ..Product::default()
    };
    let mut changed = product.clone();
    changed.stock = -3;
    exercise(store.products(), product, Product::default(), changed);

    let mut sale = Sale {
        client_name: "Sin cliente".into(),
        payment_method: PaymentMethod::TarjetaDebito,
        items: vec![
            SaleItem::new("p1", "Uno", 2, Decimal::new(1999, 2)),
            SaleItem::new("p2", "Dos", 1, Decimal::new(5, 1)),
        ],
        ..Sale::default()
    };
    sale.calculate_totals(Decimal::new(19, 2));
    let mut changed = sale.clone();
    changed.items.remove(0);
    changed.calculate_totals(Decimal::ZERO);
    exercise(store.sales(), sale, Sale::default(), changed);
}
