mod cart;
mod catalog;
mod payment;
mod quantity;

use eframe::egui::{
    Align, Context, Frame, Key, Layout, Margin, RichText, ScrollArea, SidePanel, Ui,
};
use pos_domain::{Category, Client, PaymentMethod, Product, Sale};
use pos_services::{format, print, Result};
use rust_decimal::Decimal;

use super::view::View;
use crate::components::confirm::Confirm;
use crate::components::{button, card, field, select, title};
use crate::env::Env;
use crate::task::Task;
use crate::{input, theme};
use cart::{Cart, CartEvent};
use catalog::Catalog;
use payment::Payment;
use quantity::QuantityDialog;

const WITHOUT_CLIENT: &str = "Sin cliente";
const PANEL_WIDTH: f32 = 380.0;

struct Loaded {
    products: Vec<Product>,
    categories: Vec<Category>,
    clients: Vec<Client>,
}

pub struct NewSaleView {
    load: Option<Task<Loaded>>,
    catalog: Catalog,
    clients: Vec<Client>,
    client_names: Vec<String>,
    client: usize,
    cart: Cart,
    selected: Option<usize>,
    payment: Payment,
    quantity: Option<QuantityDialog>,
    confirming: Option<Confirm>,
}

impl NewSaleView {
    pub fn new(ctx: &Context, env: &Env) -> Self {
        let mut view = Self {
            load: None,
            catalog: Catalog::default(),
            clients: Vec::new(),
            client_names: vec![WITHOUT_CLIENT.to_owned()],
            client: 0,
            cart: Cart::default(),
            selected: None,
            payment: Payment::new(env.config.sales.default_payment_method),
            quantity: None,
            confirming: None,
        };
        view.catalog.request_focus();
        view.refresh(ctx, env);
        view
    }

    fn refresh(&mut self, ctx: &Context, env: &Env) {
        let services = env.services.clone();
        self.load = Some(Task::spawn(ctx, move || -> Result<Loaded> {
            Ok(Loaded {
                products: services.inventory.list(false)?,
                categories: services.categories.list(false)?,
                clients: services.clients.list(false)?,
            })
        }));
    }

    fn poll(&mut self, env: &mut Env) {
        let Some(outcome) = Task::poll(&mut self.load) else {
            return;
        };
        if let Some(loaded) = env.check(outcome) {
            let chosen = self.chosen_client().map(|client| client.id.clone());
            self.catalog.products = loaded.products;
            self.catalog.categories = loaded.categories;
            self.clients = loaded.clients;
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
        self.catalog
            .products
            .iter()
            .find(|product| product.id == item.product_id)
            .map(|product| product.stock)
    }

    fn step(&mut self, index: usize, delta: i64, env: &mut Env) {
        let quantity = self.cart.items[index].quantity + delta;
        let limit = self.limit(index, env);
        if let Err(message) = self.cart.set_quantity(index, quantity, limit) {
            env.warn("Stock insuficiente", message);
        }
        self.keep_selection();
    }

    fn edit(&mut self, index: usize, env: &Env) {
        let limit = self.limit(index, env);
        let item = self.cart.items[index].clone();
        self.quantity = Some(QuantityDialog::new(index, item, limit));
    }

    fn remove(&mut self, index: usize) {
        self.cart.items.remove(index);
        self.keep_selection();
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

    fn panel(&mut self, ui: &mut Ui, env: &mut Env) {
        let methods: Vec<String> = PaymentMethod::ALL
            .iter()
            .map(|method| method.label().to_owned())
            .collect();
        select::labelled(ui, "Cliente", &self.client_names, &mut self.client);
        select::labelled(ui, "Metodo de pago", &methods, &mut self.payment.method);
        ui.horizontal(|ui| {
            title::heading(ui, &format!("Carrito ({})", self.cart.units()));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if button::plain(ui, "Limpiar") {
                    self.clear();
                }
            });
        });
        self.lines(ui, env);
        self.totals(ui, env);
    }

    fn lines(&mut self, ui: &mut Ui, env: &mut Env) {
        let height = (ui.available_height() - 290.0).max(90.0);
        let mut event = None;
        ScrollArea::vertical()
            .max_height(height)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| event = self.cart.show(ui, self.selected));
        match event {
            None => {}
            Some(CartEvent::Select(index)) => self.selected = Some(index),
            Some(CartEvent::Step(index, delta)) => self.step(index, delta, env),
            Some(CartEvent::Edit(index)) => self.edit(index, env),
            Some(CartEvent::Remove(index)) => self.remove(index),
        }
    }

    fn totals(&mut self, ui: &mut Ui, env: &mut Env) {
        let colors = theme::palette();
        let sale = self.preview(env.config.general.tax_rate);
        let percent = (env.config.general.tax_rate * Decimal::ONE_HUNDRED).normalize();
        ui.columns(2, |columns| {
            field::text(&mut columns[0], "Descuento", &mut self.payment.discount);
            columns[1].add_enabled_ui(self.payment.is_cash(), |ui| {
                field::text(ui, "Recibido", &mut self.payment.received);
            });
        });
        field::search(ui, &mut self.payment.notes, "Notas", f32::INFINITY);
        card::surface(ui, |ui| {
            ui.with_layout(Layout::top_down(Align::Max), |ui| {
                title::hint(
                    ui,
                    &format!(
                        "Subtotal: ${}   Impuesto ({percent}%): ${}",
                        format::amount(sale.subtotal),
                        format::amount(sale.tax)
                    ),
                );
                ui.label(
                    RichText::new(format!("Total: ${}", format::amount(sale.total)))
                        .size(22.0)
                        .strong()
                        .color(colors.success),
                );
                if let Some(change) = self.payment.change(sale.total) {
                    let color = if change < Decimal::ZERO {
                        colors.danger
                    } else {
                        colors.accent
                    };
                    let text = format!("Cambio: ${}", format::amount(change));
                    title::status(ui, &text, color);
                }
            });
        });
        if button::wide_accent(ui, "Confirmar Venta") {
            self.request(env);
        }
    }

    fn preview(&self, tax_rate: Decimal) -> Sale {
        let mut sale = Sale {
            items: self.cart.items.clone(),
            ..Sale::default()
        };
        self.payment.apply(&mut sale);
        sale.calculate_totals(tax_rate);
        sale
    }

    fn request(&mut self, env: &mut Env) {
        if self.cart.items.is_empty() {
            env.warn("Carrito vacio", "Agregue productos antes de confirmar");
            return;
        }
        let total = self.preview(env.config.general.tax_rate).total;
        if let Err(message) = self.payment.validate(total) {
            env.warn("Revise el cobro", message);
            return;
        }
        if !env.config.sales.confirm_before_saving {
            self.register(env);
            return;
        }
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
        let mut text = format!(
            "Venta #{} por {total} registrada exitosamente",
            sale.short_id()
        );
        if let Some(change) = self.payment.change(sale.total) {
            text.push_str(&format!("\nCambio: ${}", format::amount(change)));
        }
        env.toast(text);
        if env.config.sales.print_receipt {
            let document = print::receipt(&sale, &env.config);
            env.print(document, format!("Venta por {total}"));
        }
        self.clear();
        self.payment.reset();
        self.catalog.clear();
        self.client = 0;
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
            self.catalog.request_focus();
        }
        if input::pressed(ctx, Key::F5) {
            self.refresh(ctx, env);
        }
        let Some(index) = self.selected.filter(|_| input::free(ctx)) else {
            return;
        };
        if input::pressed(ctx, Key::Plus) {
            self.step(index, 1, env);
        } else if input::pressed(ctx, Key::Minus) {
            self.step(index, -1, env);
        } else if input::pressed(ctx, Key::Delete) {
            self.remove(index);
        } else if input::pressed(ctx, Key::Enter) {
            self.edit(index, env);
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
        let frame = Frame::new().inner_margin(Margin {
            left: 18,
            ..Margin::ZERO
        });
        SidePanel::right("sale_panel")
            .exact_width(PANEL_WIDTH)
            .resizable(false)
            .frame(frame)
            .show_inside(ui, |ui| self.panel(ui, env));
        title::title(ui, "Nueva Venta", "");
        if self.load.is_some() {
            title::hint(ui, "Cargando productos y clientes...");
        }
        if let Some(product) = self.catalog.show(ui) {
            self.add(&product, env);
        }
        self.dialogs(&ui.ctx().clone(), env);
    }
}
