use pos_domain::{
    now, Category, Client, Entity, MovementKind, PaymentMethod, Permission, Product, Role, Sale,
    SaleItem, StockMovement, User,
};
use pos_storage::{Repository, SaleChange, StockOperation, StorageError, Store};
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
    repository
        .replace_all(std::slice::from_ref(&second))
        .unwrap();
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
    sales_are_atomic(store);
}

fn sales_are_atomic(store: &dyn Store) {
    let product = |name: &str, stock: i64| Product {
        name: name.into(),
        stock,
        ..Product::default()
    };
    clear_stock(store);
    let (full, scarce) = (product("Lleno", 100), product("Escaso", 5));
    store
        .products()
        .upsert_many(&[full.clone(), scarce.clone()])
        .unwrap();
    let order = |quantity: i64| Sale {
        items: vec![
            SaleItem::new(&full.id, &full.name, 1, Decimal::ONE),
            SaleItem::new(&scarce.id, &scarce.name, quantity, Decimal::ONE),
        ],
        ..Sale::default()
    };
    let stock = |id: &str| store.products().find(id).unwrap().unwrap().stock;

    let rejected = register(store, &order(6), false);
    assert!(matches!(rejected, Err(StorageError::Stock(name)) if name == "Escaso"));
    assert_eq!(stock(&full.id), 100);
    assert_eq!(store.sales().count().unwrap(), 0);

    let accepted = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..10)
            .map(|_| scope.spawn(|| register(store, &order(1), false).is_ok()))
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .filter(|done| *done)
            .count()
    });
    assert_eq!(accepted, 5);
    assert_eq!(stock(&scarce.id), 0);
    assert_eq!(stock(&full.id), 95);
    assert_eq!(store.sales().count().unwrap(), 5);

    register(store, &order(2), true).unwrap();
    assert_eq!(stock(&scarce.id), -2);

    let mut voided = store.sales().all().unwrap().remove(0);
    let before = stock(&full.id);
    voided.voided_at = Some(now());
    let movements = StockMovement::of_sale(&voided, MovementKind::Anulacion, "ana");
    let undo = StockOperation {
        sale: SaleChange::Void(&voided),
        movements: &movements,
        allow_negative: false,
    };
    let done = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..4)
            .map(|_| scope.spawn(|| store.apply_stock(&undo).is_ok()))
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .filter(|done| *done)
            .count()
    });
    assert_eq!(done, 1);
    assert!(matches!(
        store.apply_stock(&undo),
        Err(StorageError::AlreadyVoided)
    ));
    assert_eq!(stock(&full.id), before + 1);
    let saved = store.sales().find(&voided.id).unwrap().unwrap();
    assert_eq!(saved, voided);
    let history = store.movements().all().unwrap();
    assert_eq!(history.len(), 14);
    let total: i64 = history
        .iter()
        .filter(|item| item.product_id == scarce.id)
        .map(|item| item.quantity)
        .sum();
    assert_eq!(scarce.stock + total, stock(&scarce.id));
    clear_stock(store);
}

fn clear_stock(store: &dyn Store) {
    store.products().replace_all(&[]).unwrap();
    store.sales().replace_all(&[]).unwrap();
    store.movements().replace_all(&[]).unwrap();
}

fn register(store: &dyn Store, sale: &Sale, allow_negative: bool) -> pos_storage::Result<()> {
    store.apply_stock(&StockOperation {
        sale: SaleChange::Register(sale),
        movements: &StockMovement::of_sale(sale, MovementKind::Venta, "ana"),
        allow_negative,
    })
}
