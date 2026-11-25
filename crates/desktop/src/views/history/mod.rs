mod detail;

use eframe::egui::{Context, Ui};
use pos_domain::{Client, Sale};
use pos_services::{format, print, report, sales, Result, SaleFilter};

use super::view::View;
use crate::components::confirm::Confirm;
use crate::components::list::{List, ListAction};
use crate::components::table::{left, right, Column, Row};
use crate::components::{button, field, title};
use crate::env::Env;
use crate::task::Task;
use crate::theme;
use detail::DetailDialog;

const COLUMNS: [Column; 8] = [
    left("ID", 90.0),
    left("Fecha", 140.0),
    left("Cliente", 170.0),
    right("Items", 60.0),
    right("Total", 120.0),
    left("Pago", 130.0),
    left("Vendedor", 120.0),
    left("Estado", 90.0),
];

pub struct HistoryView {
    list: List<Sale>,
    clients: Vec<Client>,
    loading: Option<Task<Vec<Client>>>,
    from: String,
    to: String,
    detail: Option<DetailDialog>,
    voiding: Option<(Confirm, Sale)>,
}

impl HistoryView {
    pub fn new(ctx: &Context, env: &Env) -> Self {
        let mut view = Self {
            list: List::new("sales"),
            clients: Vec::new(),
            loading: None,
            from: String::new(),
            to: String::new(),
            detail: None,
            voiding: None,
        };
        view.refresh(ctx, env);
        view
    }

    fn refresh(&mut self, ctx: &Context, env: &Env) {
        let services = env.services.clone();
        self.list
            .load(Task::spawn(ctx, move || -> Result<Vec<Sale>> {
                let mut items = services.sales.list()?;
                items.sort_by_key(|sale| std::cmp::Reverse(sale.created_at));
                Ok(items)
            }));
        let services = env.services.clone();
        self.loading = Some(Task::spawn(ctx, move || services.clients.list(true)));
    }

    fn filter(&self) -> SaleFilter {
        SaleFilter {
            query: self.list.query.clone(),
            from: self.from.clone(),
            to: self.to.clone(),
        }
    }

    fn filters(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.list.search(ui, "Buscar", 220.0);
            for (label, value) in [("Desde:", &mut self.from), ("Hasta:", &mut self.to)] {
                field::caption(ui, label);
                field::search(ui, value, "AAAA-MM-DD", 100.0);
            }
            if button::plain(ui, "Limpiar") {
                self.list.query.clear();
                self.from.clear();
                self.to.clear();
                self.list.reset_page();
            }
        });
    }

    fn print(&self, env: &mut Env) {
        let found = sales::filter(&self.list.items, &self.clients, &self.filter());
        let owned: Vec<Sale> = found
            .iter()
            .filter(|sale| !sale.is_voided())
            .map(|sale| (*sale).clone())
            .collect();
        let summary = report::summarize(&owned);
        let listing = print::Listing {
            title: "Historial de Ventas".to_owned(),
            columns: COLUMNS
                .iter()
                .map(|column| print::Column {
                    label: column.title,
                    numeric: column.right,
                })
                .collect(),
            rows: found.iter().map(|sale| cells(sale).cells).collect(),
            summary: vec![
                ("Ventas".to_owned(), summary.count.to_string()),
                ("Total".to_owned(), format::amount(summary.total)),
                ("Promedio".to_owned(), format::amount(summary.average)),
            ],
            ..print::Listing::default()
        };
        let detail = format!("{} registro(s)", found.len());
        env.print(print::table(&listing, &env.config), detail);
    }

    fn ask_void(&mut self, env: &mut Env) {
        let Some(sale) = self.list.require(env) else {
            return;
        };
        if sale.is_voided() {
            env.warn("Anular venta", "La venta ya fue anulada");
            return;
        }
        let text = format!(
            "Se anulara la venta #{} y sus productos volveran al inventario.\n\
             Esta accion no se puede deshacer.",
            sale.short_id()
        );
        self.voiding = Some((Confirm::new("Anular venta", text), sale));
    }

    fn void(&mut self, ctx: &Context, env: &mut Env) {
        let answer = self
            .voiding
            .as_ref()
            .and_then(|(confirm, _)| confirm.show(ctx));
        let Some(accepted) = answer else {
            return;
        };
        if let Some((_, sale)) = self.voiding.take().filter(|_| accepted) {
            let outcome = env.services.sales.void(sale, &env.author());
            env.check(outcome);
            self.refresh(ctx, env);
        }
    }

    fn perform(&mut self, action: ListAction, ctx: &Context, env: &mut Env) {
        match action {
            ListAction::Refresh => self.refresh(ctx, env),
            ListAction::Print => self.print(env),
            ListAction::Open => self.detail = self.list.require(env).map(DetailDialog::new),
            ListAction::Delete => {}
        }
    }
}

impl View for HistoryView {
    fn show(&mut self, ui: &mut Ui, env: &mut Env) {
        self.list.poll(env);
        if let Some(outcome) = Task::poll(&mut self.loading) {
            self.clients = env.check(outcome).unwrap_or_default();
        }
        title::title(ui, "Historial de Ventas", "");
        self.filters(ui);
        let items = std::mem::take(&mut self.list.items);
        let matches = sales::filter(&items, &self.clients, &self.filter());
        let total = matches
            .iter()
            .filter(|sale| !sale.is_voided())
            .map(|sale| sale.total)
            .sum();
        let summary = format!(
            "{} venta(s) | Total: ${}",
            matches.len(),
            format::amount(total)
        );
        let mut action = self.list.show(ui, &COLUMNS, &matches, cells, 90.0);
        self.list.items = items;
        ui.horizontal(|ui| {
            if button::plain(ui, "Ver Detalle") {
                action = Some(ListAction::Open);
            }
            if button::plain(ui, "Imprimir") {
                action = Some(ListAction::Print);
            }
            if env.is_admin() && button::danger(ui, "Anular venta") {
                self.ask_void(env);
            }
            title::hint(ui, &summary);
        });
        if let Some(action) = action {
            self.perform(action, ui.ctx(), env);
        }
        self.void(ui.ctx(), env);
        if let Some(detail) = &self.detail {
            if !detail.show(ui.ctx(), env) {
                self.detail = None;
            }
        }
    }
}

fn cells(sale: &Sale) -> Row {
    let client = match sale.client_name.as_str() {
        "" => "Sin cliente",
        name => name,
    };
    let voided = sale.is_voided();
    let mut row = Row::new(vec![
        sale.short_id().to_owned(),
        format::minute(sale.created_at),
        client.to_owned(),
        sale.items.len().to_string(),
        format::amount(sale.total),
        sale.payment_method.label().to_owned(),
        sale.seller.clone(),
        if voided { "Anulada" } else { "Activa" }.to_owned(),
    ]);
    row.color = voided.then_some(theme::SUBTLE);
    row
}
