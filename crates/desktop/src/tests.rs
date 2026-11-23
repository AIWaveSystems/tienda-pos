use std::fs;
use std::time::Duration;

use eframe::egui::accesskit::Role;
use eframe::egui::{Key, Modifiers};
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use pos_config::{ConfigStore, Paths};
use pos_domain::{Client, Product};
use pos_storage::{DocumentStore, Store};

use crate::app::App;
use crate::theme;
use crate::views::navigation::Route;

struct Desk {
    folder: tempfile::TempDir,
    harness: Harness<'static>,
}

impl Desk {
    fn start() -> Self {
        let folder = tempfile::tempdir().unwrap();
        let paths = Paths::rooted(folder.path());
        let store = DocumentStore::local(&paths.data_dir).unwrap();
        for name in ["Pan", "Leche"] {
            let product = Product {
                name: name.into(),
                price: 1000.into(),
                stock: 3,
                ..Product::default()
            };
            store.products().upsert(&product).unwrap();
        }
        let client = Client {
            name: "Ana".into(),
            ..Client::default()
        };
        store.clients().upsert(&client).unwrap();
        let config = ConfigStore::new(paths);
        let mut app = None;
        let harness = Harness::builder()
            .with_size([1100.0, 760.0])
            .build(move |ctx| {
                app.get_or_insert_with(|| {
                    theme::apply(ctx);
                    App::new(ctx, config.clone())
                })
                .tick(ctx);
            });
        let mut desk = Self { folder, harness };
        desk.wait_for("Ingresar");
        desk
    }

    fn step(&mut self) {
        for _ in 0..4 {
            self.harness.step();
        }
    }

    fn sees(&self, label: &str) -> bool {
        self.harness
            .query_all_by_label_contains(label)
            .next()
            .is_some()
    }

    fn wait_for(&mut self, label: &str) {
        for _ in 0..400 {
            self.step();
            if self.sees(label) {
                return;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        panic!("no aparecio: {label}");
    }

    fn click(&mut self, label: &str) {
        let node = self.harness.get_all_by_label(label).last();
        node.unwrap().click();
        self.step();
    }

    fn open(&mut self, label: &str) {
        let node = self.harness.get_all_by_label_contains(label).next();
        node.unwrap().click();
        self.step();
    }

    fn write(&mut self, label: &str, text: &str) {
        self.harness
            .get_by_role_and_label(Role::TextInput, label)
            .focus();
        self.step();
        self.type_text(text);
    }

    fn write_password(&mut self, position: usize, text: &str) {
        self.harness
            .get_all_by_role(Role::PasswordInput)
            .nth(position)
            .unwrap()
            .focus();
        self.step();
        self.type_text(text);
    }

    fn type_text(&mut self, text: &str) {
        self.harness
            .input_mut()
            .events
            .push(eframe::egui::Event::Text(text.into()));
        self.step();
    }

    fn press(&mut self, modifiers: Modifiers, key: Key) {
        self.harness.press_key_modifiers(modifiers, key);
        self.step();
    }

    fn sign_in(&mut self) {
        self.write("Usuario", "admin");
        self.write_password(0, "admin123");
        self.click("Ingresar");
        self.wait_for("Cambio de contrasena requerido");
        for (position, text) in ["admin123", "segura1", "segura1"].into_iter().enumerate() {
            self.write_password(position, text);
        }
        self.click("Guardar contrasena");
        self.wait_for("Contrasena actualizada");
        self.click("Aceptar");
        self.wait_for("Ingresos");
    }
}

#[test]
fn wrong_password_is_rejected() {
    let mut desk = Desk::start();
    desk.write("Usuario", "admin");
    desk.write_password(0, "incorrecta");
    desk.click("Ingresar");
    desk.wait_for("Usuario o contrasena incorrectos");
}

#[test]
fn every_view_opens() {
    let mut desk = Desk::start();
    desk.sign_in();
    for (route, content) in [
        (Route::Inventory, "Leche"),
        (Route::Categories, "Sin registros"),
        (Route::Clients, "Ana"),
        (Route::NewSale, "Confirmar Venta"),
        (Route::History, "Sin registros"),
        (Route::Reports, "Estado del Inventario"),
        (Route::Users, "Sesion actual"),
        (Route::Settings, "Datos del negocio"),
        (Route::Dashboard, "Ingresos"),
    ] {
        desk.open(route.label());
        desk.wait_for(content);
    }
}

#[test]
fn a_sale_can_be_made_with_the_keyboard() {
    let mut desk = Desk::start();
    desk.sign_in();
    desk.open(Route::NewSale.label());
    desk.wait_for("Confirmar Venta");
    for _ in 0..2 {
        desk.type_text("pan");
        desk.press(Modifiers::NONE, Key::Enter);
    }
    desk.wait_for("Total: $2,380.00");
    desk.press(Modifiers::COMMAND, Key::Enter);
    desk.wait_for("¿Desea continuar?");
    desk.click("Si");
    desk.wait_for("registrada exitosamente");
    let store = DocumentStore::local(desk.folder.path().join("data")).unwrap();
    assert_eq!(store.sales().count().unwrap(), 1);
    let products = store.products().all().unwrap();
    let bread = products.iter().find(|item| item.name == "Pan").unwrap();
    assert_eq!(bread.stock, 1);
}

#[test]
fn a_category_can_be_created_and_is_validated() {
    let mut desk = Desk::start();
    desk.sign_in();
    desk.open(Route::Categories.label());
    desk.wait_for("Sin registros");
    desk.click("+ Nueva Categoria");
    desk.click("Guardar");
    desk.wait_for("El nombre es obligatorio");
    desk.write("Nombre *", "Bebidas");
    desk.click("Guardar");
    desk.wait_for("1 categoria(s)");
    let saved = fs::read_to_string(desk.folder.path().join("data/categories.json")).unwrap();
    assert!(saved.contains("Bebidas"));
}

#[test]
fn the_palette_navigates() {
    let mut desk = Desk::start();
    desk.sign_in();
    desk.press(Modifiers::COMMAND, Key::K);
    desk.wait_for("Ir a...");
    desk.type_text("usu");
    desk.press(Modifiers::NONE, Key::Enter);
    desk.wait_for("Sesion actual");
}

#[test]
fn settings_are_validated_and_saved() {
    let mut desk = Desk::start();
    desk.sign_in();
    desk.open(Route::Settings.label());
    desk.wait_for("Datos del negocio");
    desk.write("Nombre del negocio", "Tienda Uno");
    desk.click("Guardar cambios");
    desk.wait_for("Configuracion guardada");
    let config = ConfigStore::new(Paths::rooted(desk.folder.path()));
    assert_eq!(config.load().unwrap().general.business_name, "Tienda Uno");
    desk.write("Impuesto / IVA (%)", "x");
    desk.click("Guardar cambios");
    desk.wait_for("El impuesto debe ser un numero");
}
