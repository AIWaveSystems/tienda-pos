mod cart;
mod quantity;

use std::str::FromStr;

use eframe::egui::{Align, Context, Id, Key, Layout, RichText, ScrollArea, Ui};
use pos_domain::{Client, PaymentMethod, Product, Sale};
use pos_services::{format, inventory, print, Result};
use rust_decimal::Decimal;

use super::view::View;
use crate::components::confirm::Confirm;
use crate::components::table::{left, right, table, Column, Row};
use crate::components::{button, card, field, select, title};
use crate::env::Env;
use crate::task::Task;
use crate::{input, theme};
use cart::Cart;
use quantity::QuantityDialog;

const COLUMNS: [Column; 4] = [
    left("Producto", 200.0),
    right("Precio U.", 110.0),
    right("Cant.", 70.0),
    right("Subtotal", 120.0),
];

const WITHOUT_CLIENT: &str = "Sin cliente";

pub struct NewSaleView {
    load: Option<Task<(Vec<Product>, Vec<Client>)>>,
    products: Vec<Product>,
    clients: Vec<Client>,
    client_names: Vec<String>,
    client: usize,
    method: usize,
    query: String,
    cart: Cart,
    selected: Option<usize>,
    discount: String,
    quantity: Option<QuantityDialog>,
    confirming: Option<Confirm>,
    focus: bool,
}

impl NewSaleView {
    pub fn new(ctx: &Context, env: &Env) -> Self {
        let method = env.config.sales.default_payment_method;
        let mut view = Self {
            load: None,
            products: Vec::new(),
            clients: Vec::new(),
            client_names: vec![WITHOUT_CLIENT.to_owned()],
            client: 0,
            method: PaymentMethod::ALL
                .iter()
                .position(|item| *item == method)
                .unwrap_or_default(),
            query: String::new(),
            cart: Cart::default(),
            selected: None,
            discount: "0".to_owned(),
            quantity: None,
            confirming: None,
            focus: true,
        };
        view.refresh(ctx, env);
        view
    }

    fn refresh(&mut self, ctx: &Context, env: &Env) {
        let services = env.services.clone();
        self.load = Some(Task::spawn(
            ctx,
            move || -> Result<(Vec<Product>, Vec<Client>)> {
                Ok((
                    services.inventory.list(false)?,
                    services.clients.list(false)?,
                ))
            },
        ));
    }

    fn poll(&mut self, env: &mut Env) {
        let Some(outcome) = Task::poll(&mut self.load) else {
            return;
        };
        if let Some((products, clients)) = env.check(outcome) {
            let chosen = self.chosen_client().map(|client| client.id.clone());
            self.products = products;
            self.clients = clients;
            self.client_names = std::iter::once(WITHOUT_CLIENT.to_owned())
                .chain(self.clients.iter().map(|client| client.name.clone()))
                .collect();
            self.client = self
                .clients
                .iter()
                .position(|client| Some(&client.id) == chosen.as_ref())
                .map_or(0, |index| index + 1);
        }
    }

    fn chosen_client(&self) -> Option<&Client> {
        self.clients.get(self.client.checked_sub(1)?)
    }

    fn found(&self) -> Vec<Product> {
        if self.query.trim().is_empty() {
            return Vec::new();
        }
        inventory::search(&self.products, &self.query, |_| String::new())
            .into_iter()
            .cloned()
            .collect()
    }

    fn header(&mut self, ui: &mut Ui) {
        let methods: Vec<String> = PaymentMethod::ALL
            .iter()
            .map(|method| method.label().to_owned())
            .collect();
        ui.horizontal(|ui| {
            field::caption(ui, "Cliente (opcional):");
            select::select(ui, "client", &self.client_names, &mut self.client, 240.0);
            field::caption(ui, "Metodo de pago:");
            select::select(ui, "method", &methods, &mut self.method, 170.0);
        });
    }

    fn search(&mut self, ui: &mut Ui, env: &mut Env) {
        let response = ui
            .push_id(Id::new("sale_search"), |ui| {
                field::search(
                    ui,
                    &mut self.query,
                    "Producto o codigo de barras",
                    f32::INFINITY,
                )
            })
            .inner;
        if std::mem::take(&mut self.focus) {
            response.request_focus();
        }
        let found = self.found();
        let entered = response.lost_focus() && ui.input(|state| state.key_pressed(Key::Enter));
        if entered && !input::modal_open(ui.ctx()) {
            let exact = found.iter().find(|item| item.has_barcode(&self.query));
            let single = (found.len() == 1).then(|| &found[0]);
            if let Some(product) = exact.or(single).cloned() {
                self.add(&product, env);
                self.query.clear();
            }
            self.focus = true;
        }
        if found.is_empty() {
            return;
        }
        ScrollArea::vertical().max_height(130.0).show(ui, |ui| {
            for product in &found {
                let mut text = format!("{} - ${}", product.name, format::amount(product.price));
                if product.low_stock() {
                    text.push_str(&format!(" [stock: {}]", product.stock));
                }
                if ui.selectable_label(false, text).clicked() {
                    self.add(product, env);
                }
            }
        });
    }

    fn add(&mut self, product: &Product, env: &mut Env) {
        let allow = env.config.inventory.allow_negative_stock;
        match self.cart.add(product, allow) {
            Ok(index) => self.selected = Some(index),
            Err(message) => env.warn("Stock insuficiente", message),
        }
    }

    fn limit(&self, index: usize, env: &Env) -> Option<i64> {
        if env.config.inventory.allow_negative_stock {
            return None;
        }
        let item = &self.cart.items[index];
        self.products
            .iter()
            .find(|product| product.id == item.product_id)
            .map(|product| product.stock)
    }

    fn step(&mut self, delta: i64, env: &mut Env) {
        let Some(index) = self.selected else {
            env.info("Seleccionar", "Seleccione un producto del carrito.");
            return;
        };
        let quantity = self.cart.items[index].quantity + delta;
        let limit = self.limit(index, env);
        if let Err(message) = self.cart.set_quantity(index, quantity, limit) {
            env.warn("Stock insuficiente", message);
        }
        self.keep_selection();
    }

    fn edit(&mut self, env: &mut Env) {
        match self.selected {
            Some(index) => {
                let limit = self.limit(index, env);
                self.quantity = Some(QuantityDialog::new(
                    index,
                    self.cart.items[index].clone(),
                    limit,
                ));
            }
            None => env.info("Seleccionar", "Seleccione un producto del carrito."),
        }
    }

    fn remove(&mut self) {
        if let Some(index) = self.selected {
            self.cart.items.remove(index);
            self.keep_selection();
        }
    }

    fn clear(&mut self) {
        self.cart.items.clear();
        self.selected = None;
    }

    fn keep_selection(&mut self) {
        if self
            .selected
            .is_some_and(|index| index >= self.cart.items.len())
        {
            self.selected = None;
        }
    }

    fn cart_table(&mut self, ui: &mut Ui, env: &mut Env) {
        let rows: Vec<Row> = self
            .cart
            .items
            .iter()
            .map(|item| {
                Row::new(vec![
                    item.product_name.clone(),
                    format::amount(item.unit_price),
                    item.quantity.to_string(),
                    format::amount(item.subtotal),
                ])
            })
            .collect();
        let height = (ui.available_height() - 210.0).max(110.0);
        let event = ui
            .allocate_ui([ui.available_width(), height].into(), |ui| {
                table(ui, "cart", &COLUMNS, &rows, self.selected, height - 34.0)
            })
            .inner;
        if let Some(index) = event.clicked.or(event.activated) {
            self.selected = Some(index);
        }
        if event.activated.is_some() {
            self.edit(env);
        }
    }

    fn actions(&mut self, ui: &mut Ui, env: &mut Env) {
        ui.horizontal(|ui| {
            if button::plain(ui, "-") {
                self.step(-1, env);
            }
            if button::plain(ui, "+") {
                self.step(1, env);
            }
            if button::plain(ui, "Cantidad...") {
                self.edit(env);
            }
            if button::plain(ui, "Quitar") {
                self.remove();
            }
            if button::plain(ui, "Limpiar") {
                self.clear();
            }
            field::caption(ui, "Descuento:");
            field::input(ui, &mut self.discount, 90.0);
        });
    }

    fn totals(&mut self, ui: &mut Ui, env: &mut Env) {
        let preview = self.preview(env.config.general.tax_rate);
        let percent = (env.config.general.tax_rate * Decimal::ONE_HUNDRED).normalize();
        let mut confirm = false;
        card::surface(ui, |ui| {
            ui.with_layout(Layout::top_down(Align::Max), |ui| {
                title::hint(
                    ui,
                    &format!("Subtotal: ${}", format::amount(preview.subtotal)),
                );
                title::hint(
                    ui,
                    &format!("Impuesto ({percent}%): ${}", format::amount(preview.tax)),
                );
                ui.label(
                    RichText::new(format!("Total: ${}", format::amount(preview.total)))
                        .size(19.0)
                        .strong()
                        .color(theme::SUCCESS),
                );
                confirm = button::accent(ui, "Confirmar Venta");
            });
        });
        if confirm {
            self.request(env);
        }
    }

    fn preview(&self, tax_rate: Decimal) -> Sale {
        let mut sale = Sale {
            items: self.cart.items.clone(),
            discount: self.discount_value().unwrap_or_default(),
            ..Sale::default()
        };
        sale.calculate_totals(tax_rate);
        sale
    }

    fn discount_value(&self) -> Option<Decimal> {
        let text = self.discount.trim().replace(',', ".");
        if text.is_empty() {
            return Some(Decimal::ZERO);
        }
        Decimal::from_str(&text).ok()
    }

    fn request(&mut self, env: &mut Env) {
        if self.cart.items.is_empty() {
            env.warn("Carrito vacio", "Agregue productos antes de confirmar");
            return;
        }
        if self.discount_value().is_none() {
            env.warn("Descuento invalido", "Ingrese un valor numerico");
            return;
        }
        if !env.config.sales.confirm_before_saving {
            self.register(env);
            return;
        }
        let total = self.preview(env.config.general.tax_rate).total;
        self.confirming = Some(Confirm::new(
            "Confirmar venta",
            format!(
                "Se registrara una venta por {}.\n\n¿Desea continuar?",
                format::money(total, &env.config.general.currency)
            ),
        ));
    }

    fn register(&mut self, env: &mut Env) {
        let mut sale = self.preview(env.config.general.tax_rate);
        sale.payment_method = PaymentMethod::ALL[self.method];
        sale.seller = env.author();
        sale.client_name = WITHOUT_CLIENT.to_owned();
        if let Some(client) = self.chosen_client() {
            sale.client_id = client.id.clone();
            sale.client_name = client.name.clone();
        }
        let outcome = env.services.sales.register(
            sale,
            env.config.general.tax_rate,
            env.config.inventory.allow_negative_stock,
        );
        let Some(sale) = env.check(outcome) else {
            return;
        };
        let total = format::money(sale.total, &env.config.general.currency);
        if env.config.sales.print_receipt {
            env.print(
                print::receipt(&sale, &env.config),
                format!("Venta por {total}"),
            );
        } else {
            env.info(
                "Venta registrada",
                format!(
                    "Venta #{} por {total} registrada exitosamente",
                    sale.short_id()
                ),
            );
        }
        self.clear();
        self.discount = "0".to_owned();
        self.query.clear();
        self.client = 0;
        self.focus = true;
    }

    fn keys(&mut self, ui: &Ui, env: &mut Env) {
        let ctx = ui.ctx();
        if input::modal_open(ctx) {
            return;
        }
        if input::command(ctx, Key::Enter) {
            self.request(env);
        }
        if input::command(ctx, Key::L) {
            self.clear();
        }
        if input::command(ctx, Key::F) || input::pressed(ctx, Key::F2) {
            self.focus = true;
        }
        if input::pressed(ctx, Key::F5) {
            self.refresh(ctx, env);
        }
        if !input::free(ctx) || self.selected.is_none() {
            return;
        }
        if input::pressed(ctx, Key::Plus) {
            self.step(1, env);
        }
        if input::pressed(ctx, Key::Minus) {
            self.step(-1, env);
        }
        if input::pressed(ctx, Key::Delete) {
            self.remove();
        }
        if input::pressed(ctx, Key::Enter) {
            self.edit(env);
        }
    }

    fn dialogs(&mut self, ctx: &Context, env: &mut Env) {
        if let Some(dialog) = &mut self.quantity {
            if dialog.show(ctx, &mut self.cart) {
                self.quantity = None;
                self.keep_selection();
            }
        }
        if let Some(accepted) = self.confirming.as_ref().and_then(|item| item.show(ctx)) {
            self.confirming = None;
            if accepted {
                self.register(env);
                self.refresh(ctx, env);
            }
        }
    }
}

impl View for NewSaleView {
    fn show(&mut self, ui: &mut Ui, env: &mut Env) {
        self.poll(env);
        self.keys(ui, env);
        title::title(ui, "Nueva Venta", "");
        self.header(ui);
        self.search(ui, env);
        if self.load.is_some() {
            title::hint(ui, "Cargando productos y clientes...");
        }
        self.cart_table(ui, env);
        self.actions(ui, env);
        self.totals(ui, env);
        self.dialogs(&ui.ctx().clone(), env);
    }
}
