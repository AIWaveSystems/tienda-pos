use std::sync::Arc;

use pos_config::{AppConfig, PrinterFormat};
use pos_domain::{
    now, CashEntry, CashKind, Category, Client, PaymentMethod, Permission, Product, Role, Sale,
    SaleItem,
};
use pos_services::{
    client, format, inventory, print, sales, Period, SaleFilter, Services, Span, UserDraft,
};
use pos_storage::{DocumentStore, Store};
use rust_decimal::Decimal;

fn services() -> (tempfile::TempDir, Services) {
    let folder = tempfile::tempdir().unwrap();
    let store = Arc::new(DocumentStore::local(folder.path()).unwrap());
    (folder, Services::new(store))
}

fn draft(username: &str, role: Role) -> UserDraft {
    UserDraft {
        username: username.into(),
        full_name: "Nombre".into(),
        role,
        permissions: Vec::new(),
        active: true,
    }
}

fn product(name: &str, stock: i64) -> Product {
    Product {
        name: name.into(),
        price: Decimal::from(100),
        cost: Decimal::from(60),
        stock,
        ..Product::default()
    }
}

#[test]
fn default_user_must_change_password() {
    let (_folder, services) = services();
    let auth = &services.auth;
    let admin = auth.ensure_default_user().unwrap();
    assert_eq!(auth.ensure_default_user().unwrap().id, admin.id);
    assert!(auth.authenticate("admin", "mala").unwrap().is_none());
    assert!(
        auth.authenticate(" ADMIN ", "admin123")
            .unwrap()
            .unwrap()
            .is_default
    );
    assert!(auth
        .change_password(&admin.id, "admin123", "admin123")
        .is_err());
    assert!(auth
        .change_password(&admin.id, "admin123", "corta")
        .is_err());
    auth.change_password(&admin.id, "admin123", "segura1")
        .unwrap();
    assert!(!auth.current().unwrap().is_default);
    assert!(auth.can(Permission::Usuarios));
    auth.logout();
    assert!(!auth.can(Permission::Usuarios));
}

#[test]
fn last_active_admin_is_protected() {
    let (_folder, services) = services();
    let auth = &services.auth;
    let admin = auth.ensure_default_user().unwrap();
    assert!(auth.delete_user(&admin.id).is_err());
    assert!(auth.set_active(&admin.id, false).is_err());
    assert!(auth
        .update_user(&admin.id, &draft("admin", Role::Vendedor))
        .is_err());
    let second = auth
        .create_user(&draft("jefe", Role::Admin), "segura1")
        .unwrap();
    auth.set_active(&second.id, false).unwrap();
    auth.delete_user(&second.id).unwrap();
    assert!(auth.set_active(&admin.id, false).is_err());
}

#[test]
fn users_are_validated() {
    let (_folder, services) = services();
    let auth = &services.auth;
    auth.ensure_default_user().unwrap();
    for name in ["", "ab", "con espacio", "ADMIN"] {
        assert!(auth
            .create_user(&draft(name, Role::Vendedor), "segura1")
            .is_err());
    }
    assert!(auth
        .create_user(&draft("caja", Role::Personalizado), "segura1")
        .is_err());
    let user = auth
        .create_user(&draft("caja", Role::Vendedor), "segura1")
        .unwrap();
    assert!(user.has_permission(Permission::Ventas));
    assert!(!user.has_permission(Permission::Inventario));
}

#[test]
fn catalog_rejects_duplicates() {
    let (_folder, services) = services();
    let category = |name: &str| Category {
        name: name.into(),
        ..Category::default()
    };
    services.categories.save(category("Pan")).unwrap();
    assert!(services.categories.save(category(" pan ")).is_err());
    let client = |tax: &str, email: &str| Client {
        name: "Ana".into(),
        tax_id: tax.into(),
        email: email.into(),
        ..Client::default()
    };
    services.clients.save(client("1", "a@b.co")).unwrap();
    assert!(services.clients.save(client("1", "")).is_err());
    assert!(services.clients.save(client("2", "a@b")).is_err());
    let coded = |name: &str| Product {
        barcode: "770".into(),
        ..product(name, 1)
    };
    services.inventory.save(coded("Pan")).unwrap();
    assert!(services.inventory.save(coded("Leche")).is_err());
}

#[test]
fn sale_discounts_stock() {
    let (_folder, services) = services();
    let bread = services.inventory.save(product("Pan", 5)).unwrap();
    let sale = |quantity: i64| Sale {
        items: vec![SaleItem::new(&bread.id, "Pan", quantity, bread.price)],
        ..Sale::default()
    };
    let rate = Decimal::new(19, 2);
    let registered = services.sales.register(sale(3), rate, false).unwrap();
    assert_eq!(registered.total, Decimal::from(357));
    assert_eq!(
        services.inventory.find(&bread.id).unwrap().unwrap().stock,
        2
    );
    assert!(services.sales.register(sale(3), rate, false).is_err());
    assert!(services
        .sales
        .register(Sale::default(), rate, false)
        .is_err());
    assert_eq!(services.sales.list().unwrap().len(), 1);
    let overview = services.reports.overview(10, Period::Fortnight).unwrap();
    assert_eq!(overview.sales.count, 1);
    assert_eq!(overview.top[0].quantity, 3);
    assert_eq!(overview.daily.len(), 14);
    assert_eq!(overview.daily[13].total, Decimal::from(357));
    assert_eq!(overview.inventory.value, Decimal::from(120));
    assert_eq!(overview.profit, Decimal::from(120));
    assert_eq!(overview.sellers[0].name, "Sin vendedor");
}

#[test]
fn searches_behave_like_the_original() {
    let mut exact = product("Zeta", 1);
    exact.barcode = "12".into();
    let mut partial = product("Alfa", 1);
    partial.barcode = "123".into();
    let products = [partial, exact];
    let found = inventory::search(&products, "12", |_| String::new());
    assert_eq!(found[0].name, "Zeta");
    let buyer = Client {
        name: "Ana".into(),
        tax_id: "900".into(),
        ..Client::default()
    };
    assert!(client::matches(&buyer, "ANA"));
    let sale = Sale {
        client_id: buyer.id.clone(),
        ..Sale::default()
    };
    let by = |query: &str, from: &str| SaleFilter {
        query: query.into(),
        from: from.into(),
        to: String::new(),
    };
    let list = [sale];
    let buyers = [buyer];
    assert_eq!(sales::filter(&list, &buyers, &by("900", "")).len(), 1);
    assert!(sales::filter(&list, &buyers, &by("", "2999-01-01")).is_empty());
}

#[test]
fn documents_escape_content_and_follow_the_paper() {
    let mut config = AppConfig::default();
    config.general.business_name = "<b>Tienda</b>".into();
    config.sales.printer_format = PrinterFormat::Thermal58;
    let mut sale = Sale {
        items: vec![SaleItem::new("p", "<script>", 1, Decimal::from(1500))],
        ..Sale::default()
    };
    sale.calculate_totals(Decimal::ZERO);
    let document = print::receipt(&sale, &config);
    assert!(document.html.contains("&lt;script&gt;"));
    assert!(!document.html.contains("<script>"));
    assert!(document.html.contains("58mm"));
    assert!(document.html.contains("1,500.00 COP"));
    assert!(document.file_name().starts_with("Comprobante_"));
}

#[test]
fn amounts_are_grouped() {
    assert_eq!(format::amount(Decimal::new(123_456_789, 2)), "1,234,567.89");
    assert_eq!(format::amount(Decimal::new(-5, 1)), "-0.50");
    assert_eq!(format::amount(Decimal::ZERO), "0.00");
    assert_eq!(format::amount(Decimal::new(1005, 3)), "1.01");
}

#[test]
fn voided_sales_return_stock_and_leave_reports() {
    let (_folder, services) = services();
    let bread = services.inventory.save(product("Pan", 5)).unwrap();
    let sale = Sale {
        items: vec![SaleItem::new(&bread.id, "Pan", 3, bread.price)],
        seller: "ana".into(),
        ..Sale::default()
    };
    let sale = services.sales.register(sale, Decimal::ZERO, false).unwrap();
    let voided = services.sales.void(sale.clone(), "luis").unwrap();
    assert!(voided.is_voided());
    assert!(services.sales.void(sale, "luis").is_err());
    assert_eq!(
        services.inventory.find(&bread.id).unwrap().unwrap().stock,
        5
    );
    assert_eq!(
        services
            .reports
            .overview(10, Period::Fortnight)
            .unwrap()
            .sales
            .count,
        0
    );
    let history = services.inventory.movements().unwrap();
    assert_eq!(history.len(), 2);
    assert!(history
        .iter()
        .any(|item| item.author == "luis" && item.quantity == 3));
}

#[test]
fn adjustments_are_recorded_and_editing_keeps_stock() {
    let (_folder, services) = services();
    let bread = services.inventory.save(product("Pan", 5)).unwrap();
    let inventory = &services.inventory;
    assert!(inventory.adjust_stock(&bread, -6, "Merma", "ana").is_err());
    assert!(inventory.adjust_stock(&bread, 1, " ", "ana").is_err());
    assert!(inventory.adjust_stock(&bread, 0, "Nada", "ana").is_err());
    inventory.adjust_stock(&bread, -2, "Merma", "ana").unwrap();
    let mut edited = bread.clone();
    edited.name = "Pan fresco".into();
    assert_eq!(inventory.save(edited).unwrap().stock, 3);
    assert_eq!(inventory.movements().unwrap()[0].reference, "Merma");
}

#[test]
fn discounts_cannot_exceed_the_total() {
    let (_folder, services) = services();
    let bread = services.inventory.save(product("Pan", 5)).unwrap();
    for discount in [Decimal::from(-1), Decimal::from(101)] {
        let sale = Sale {
            items: vec![SaleItem::new(&bread.id, "Pan", 1, bread.price)],
            discount,
            ..Sale::default()
        };
        assert!(services.sales.register(sale, Decimal::ZERO, false).is_err());
    }
}

fn movement(kind: CashKind, concept: &str, amount: i64) -> CashEntry {
    CashEntry {
        kind,
        concept: concept.into(),
        amount: amount.into(),
        ..CashEntry::default()
    }
}

#[test]
fn cash_entries_are_validated_and_voided_once() {
    let (_folder, services) = services();
    let cash = &services.cash;
    assert!(cash.register(movement(CashKind::Gasto, " ", 10)).is_err());
    assert!(cash.register(movement(CashKind::Gasto, "Luz", 0)).is_err());
    assert!(cash.register(movement(CashKind::Gasto, "Luz", -5)).is_err());
    let entry = cash.register(movement(CashKind::Gasto, "Luz", 10)).unwrap();
    cash.void(&entry.id).unwrap();
    assert!(cash.void(&entry.id).is_err());
    assert!(cash.void("ausente").is_err());
}

#[test]
fn the_daily_report_balances_the_day() {
    let (_folder, services) = services();
    let bread = services.inventory.save(product("Pan", 50)).unwrap();
    let sell = |method: PaymentMethod| {
        let sale = Sale {
            items: vec![SaleItem::new(&bread.id, "Pan", 2, bread.price)],
            payment_method: method,
            ..Sale::default()
        };
        services.sales.register(sale, Decimal::ZERO, false).unwrap()
    };
    sell(PaymentMethod::Efectivo);
    sell(PaymentMethod::TarjetaDebito);
    let voided = sell(PaymentMethod::Efectivo);
    services.sales.void(voided, "ana").unwrap();
    let cash = &services.cash;
    cash.register(movement(CashKind::Entrada, "Base", 500))
        .unwrap();
    cash.register(movement(CashKind::Gasto, "Luz", 120))
        .unwrap();
    let removed = cash
        .register(movement(CashKind::Gasto, "Error", 999))
        .unwrap();
    cash.void(&removed.id).unwrap();
    let report = services.reports.daily(Span::today(now())).unwrap();
    assert_eq!(report.summary.count, 2);
    assert_eq!(report.voided, 1);
    assert_eq!(report.income, Decimal::from(500));
    assert_eq!(report.expense, Decimal::from(120));
    assert_eq!(report.balance(), Decimal::from(780));
    assert_eq!(report.expected_cash(), Decimal::from(580));
    assert_eq!(report.hours.iter().map(|hour| hour.count).sum::<usize>(), 2);
    let yesterday = now().date().pred_opt().unwrap();
    let empty = services.reports.daily(Span::day(yesterday)).unwrap();
    assert_eq!(empty.summary.count, 0);
    assert!(empty.entries.is_empty());
    let document = print::daily_report(&report, &AppConfig::default());
    assert!(document.html.contains("Base") && !document.html.contains("Error"));
}

#[test]
fn backups_contain_every_collection() {
    let (folder, services) = services();
    services.inventory.save(product("Pan", 5)).unwrap();
    services
        .cash
        .register(movement(CashKind::Gasto, "Luz", 1))
        .unwrap();
    let target = services.backup(&folder.path().join("copias")).unwrap();
    let copy = DocumentStore::local(target).unwrap();
    assert_eq!(copy.products().count().unwrap(), 1);
    assert_eq!(copy.cash().count().unwrap(), 1);
}
