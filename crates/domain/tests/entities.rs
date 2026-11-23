use pos_domain::{password, Permission, Product, Role, Sale, SaleItem, User};
use rust_decimal::Decimal;

#[test]
fn totals_use_exact_decimals() {
    let mut sale = Sale {
        items: vec![
            SaleItem::new("a", "A", 3, Decimal::new(10, 2)),
            SaleItem::new("b", "B", 1, Decimal::new(20, 2)),
        ],
        discount: Decimal::new(5, 2),
        ..Sale::default()
    };
    sale.calculate_totals(Decimal::new(19, 2));
    assert_eq!(sale.subtotal, Decimal::new(50, 2));
    assert_eq!(sale.tax, Decimal::new(95, 3));
    assert_eq!(sale.total, Decimal::new(545, 3));
}

#[test]
fn margin_and_low_stock() {
    let product = Product {
        price: Decimal::from(150),
        cost: Decimal::from(100),
        stock: 5,
        min_stock: 5,
        ..Product::default()
    };
    assert_eq!(product.profit_margin(), Decimal::from(50));
    assert!(product.low_stock());
    assert_eq!(Product::default().profit_margin(), Decimal::ZERO);
}

#[test]
fn reads_documents_written_by_the_original() {
    let text = r#"{"id":"0123456789ab","username":"ana","role":"desconocido",
        "permissions":["ventas","ventas","inexistente"],"last_login":"","extra":1,
        "created_at":"2026-08-18T10:00:00.123456","updated_at":"2026-08-18T10:00:00"}"#;
    let user: User = serde_json::from_str(text).unwrap();
    assert_eq!(user.role, Role::Personalizado);
    assert_eq!(user.permissions, vec![Permission::Ventas]);
    assert_eq!(user.last_login, None);
    let product: Product = serde_json::from_str(r#"{"name":"Pan","price":0.1,"cost":2}"#).unwrap();
    assert_eq!(product.price, Decimal::new(1, 1));
    assert_eq!(product.cost, Decimal::from(2));
}

#[test]
fn documents_round_trip() {
    let mut user = User::default();
    user.last_login = Some(user.created_at);
    let text = serde_json::to_string(&user).unwrap();
    assert_eq!(serde_json::from_str::<User>(&text).unwrap(), user);
}

#[test]
fn passwords_match_python_vectors() {
    let stored = "pbkdf2$1000$00112233445566778899aabbccddeeff$f7ee5af46bc8d0ecfab1ec64ad031cacdfde80b4cbfa8713e43f4477fc185d81";
    assert!(password::verify("admin123", stored));
    assert!(!password::verify("otra", stored));
    let legacy = "abcd:bfc3b7db030ba74e08990f55a3bbe3bfa37a218af9ef8cf39ce7d55f91143f4b";
    assert!(password::verify("admin123", legacy));
    assert!(password::needs_rehash(legacy));
    assert!(password::verify("nueva", &password::hash("nueva")));
}

#[test]
fn locked_roles_ignore_manual_permissions() {
    let user = User {
        role: Role::Bodega,
        permissions: vec![Permission::Usuarios],
        ..User::default()
    };
    assert!(!user.has_permission(Permission::Usuarios));
    assert!(user.has_permission(Permission::Informes));
}
