mod draft;
mod storage;

use eframe::egui::{Align, Color32, Context, Layout, ScrollArea, Ui};
use pos_config::AppConfig;

use super::view::View;
use crate::components::confirm::Confirm;
use crate::components::{button, card, field, select, title};
use crate::env::Env;
use crate::task::Task;
use crate::theme;
use draft::Draft;
use storage::StoragePanel;

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Business,
    Inventory,
    Sales,
    Interface,
    Storage,
}

const TABS: [(Tab, &str); 5] = [
    (Tab::Business, "Negocio"),
    (Tab::Inventory, "Inventario"),
    (Tab::Sales, "Ventas"),
    (Tab::Interface, "Interfaz"),
    (Tab::Storage, "Almacenamiento"),
];

pub struct SettingsView {
    tab: Tab,
    draft: Draft,
    storage: StoragePanel,
    status: (String, Color32),
    restoring: Option<Confirm>,
    migrating: Option<(Confirm, AppConfig)>,
    copy: Option<Task<AppConfig>>,
}

impl SettingsView {
    pub fn new(env: &Env) -> Self {
        Self {
            tab: Tab::Business,
            draft: Draft::new(&env.config),
            storage: StoragePanel::new(env),
            status: (String::new(), theme::palette().success),
            restoring: None,
            migrating: None,
            copy: None,
        }
    }

    fn reload(&mut self, env: &Env) {
        self.draft = Draft::new(&env.config);
        self.storage = StoragePanel::new(env);
    }

    fn tabs(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for (tab, label) in TABS {
                ui.selectable_value(&mut self.tab, tab, label);
            }
        });
        ui.separator();
    }

    fn content(&mut self, ui: &mut Ui, env: &mut Env) {
        match self.tab {
            Tab::Business => business(ui, &mut self.draft),
            Tab::Inventory => inventory(ui, &mut self.draft),
            Tab::Sales => sales(ui, &mut self.draft),
            Tab::Interface => interface(ui, &mut self.draft),
            Tab::Storage => self.storage.show(ui, env),
        }
    }

    fn footer(&mut self, ui: &mut Ui, env: &mut Env) {
        let idle = self.copy.is_none();
        ui.add_enabled_ui(idle, |ui| {
            ui.horizontal(|ui| {
                if button::plain(ui, "Restaurar valores por defecto") {
                    self.restoring = Some(Confirm::new(
                        "Restaurar configuracion",
                        "Se restauraran todos los parametros a sus valores por defecto.\n\
                         Se guardara una copia del archivo actual.\n\n¿Desea continuar?",
                    ));
                }
                title::status(ui, &self.status.0, self.status.1);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if button::accent(ui, "Guardar cambios") {
                        self.save(env);
                    }
                    if button::plain(ui, "Descartar") {
                        self.reload(env);
                        self.status.0.clear();
                    }
                });
            });
        });
    }

    fn save(&mut self, env: &mut Env) {
        let config = match self.collect(env) {
            Ok(config) => config,
            Err(message) => {
                self.status = (message, theme::palette().danger);
                return;
            }
        };
        let (old, new) = (env.config.storage.backend, config.storage.backend);
        if old == new {
            self.finish(env, config, false);
            return;
        }
        if let Err(message) = self.storage.ready() {
            self.status = (message, theme::palette().danger);
            return;
        }
        let text = format!(
            "Se copiaran los datos actuales (usuarios, categorias, clientes, productos, ventas) \
             desde '{}' hacia '{}'. Los datos en el almacenamiento anterior no se eliminaran.\n\n\
             ¿Desea continuar con el cambio?",
            old.label(),
            new.label()
        );
        self.migrating = Some((Confirm::new("Cambiar almacenamiento", text), config));
    }

    fn collect(&self, env: &Env) -> Result<AppConfig, String> {
        let mut config = env.config.clone();
        self.draft.apply(&mut config)?;
        self.storage.apply(&mut config)?;
        Ok(config)
    }

    fn finish(&mut self, env: &mut Env, config: AppConfig, migrated: bool) {
        let outcome = self
            .storage
            .store_password(env)
            .and_then(|()| env.store.save(&config));
        if let Err(failure) = outcome {
            self.status = (failure.to_string(), theme::palette().danger);
            return;
        }
        env.config = config;
        let mut message = "Configuracion guardada".to_owned();
        if migrated {
            message.push_str(". Reinicie la aplicacion para usar el nuevo almacenamiento.");
        }
        self.status = (message, theme::palette().success);
    }

    fn dialogs(&mut self, ctx: &Context, env: &mut Env) {
        if let Some(accepted) = self.restoring.as_ref().and_then(|item| item.show(ctx)) {
            self.restoring = None;
            if accepted {
                let outcome = env.store.reset();
                if let Some(config) = env.check(outcome) {
                    env.config = config;
                    self.reload(env);
                    self.status = (
                        "Valores por defecto restaurados".to_owned(),
                        theme::palette().success,
                    );
                }
            }
        }
        let answer = self
            .migrating
            .as_ref()
            .and_then(|(confirm, _)| confirm.show(ctx));
        let Some(accepted) = answer else {
            return;
        };
        let Some((_, mut config)) = self.migrating.take() else {
            return;
        };
        if accepted {
            self.status = ("Copiando datos...".to_owned(), theme::palette().muted);
            self.copy = Some(self.storage.migrate(ctx, env, config));
        } else {
            config.storage = env.config.storage.clone();
            self.finish(env, config, false);
        }
    }

    fn poll(&mut self, env: &mut Env) {
        match Task::poll(&mut self.copy) {
            Some(Ok(config)) => self.finish(env, config, true),
            Some(Err(failure)) => {
                self.status = (
                    format!("No se pudo migrar los datos: {failure}"),
                    theme::palette().danger,
                );
            }
            None => {}
        }
    }
}

impl View for SettingsView {
    fn show(&mut self, ui: &mut Ui, env: &mut Env) {
        self.poll(env);
        title::title(
            ui,
            "Configuracion",
            "Parametros generales del sistema. Los cambios se aplican al guardar.",
        );
        self.tabs(ui);
        let height = (ui.available_height() - 56.0).max(120.0);
        ScrollArea::vertical()
            .max_height(height)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| self.content(ui, env));
        self.footer(ui, env);
        self.dialogs(&ui.ctx().clone(), env);
        theme::apply(ui.ctx(), &env.config.ui.theme);
    }
}

fn business(ui: &mut Ui, draft: &mut Draft) {
    title::heading(ui, "Datos del negocio");
    title::hint(ui, "Aparecen en informes y comprobantes.");
    let general = &mut draft.general;
    field::text(ui, "Nombre del negocio", &mut general.business_name);
    field::text(ui, "NIT / ID fiscal", &mut general.business_tax_id);
    field::text(ui, "Direccion", &mut general.business_address);
    field::text(ui, "Telefono", &mut general.business_phone);
    title::heading(ui, "Facturacion");
    field::text(ui, "Impuesto / IVA (%)", &mut draft.tax);
    field::text(ui, "Moneda (codigo ISO)", &mut draft.general.currency);
}

fn inventory(ui: &mut Ui, draft: &mut Draft) {
    title::heading(ui, "Control de stock");
    field::text(ui, "Stock minimo por defecto", &mut draft.min_stock);
    card::check(
        ui,
        &mut draft.inventory.allow_negative_stock,
        "Permitir stock negativo",
        "Habilita vender productos sin existencias suficientes.",
    );
    card::check(
        ui,
        &mut draft.inventory.low_stock_on_dashboard,
        "Mostrar alerta de stock bajo en el Dashboard",
        "",
    );
}

fn sales(ui: &mut Ui, draft: &mut Draft) {
    title::heading(ui, "Punto de venta");
    select::labelled(
        ui,
        "Metodo de pago por defecto",
        &Draft::methods(),
        &mut draft.method,
    );
    card::check(
        ui,
        &mut draft.sales.confirm_before_saving,
        "Pedir confirmacion antes de registrar la venta",
        "",
    );
    card::check(
        ui,
        &mut draft.sales.print_receipt,
        "Ofrecer comprobante al finalizar la venta",
        "",
    );
    title::heading(ui, "Comprobante");
    select::labelled(
        ui,
        "Formato de impresion",
        &Draft::formats(),
        &mut draft.format,
    );
    field::text(ui, "Mensaje al pie", &mut draft.sales.receipt_footer);
}

fn interface(ui: &mut Ui, draft: &mut Draft) {
    title::heading(ui, "Apariencia");
    select::labelled(ui, "Tema", &Draft::themes(), &mut draft.theme);
    title::heading(ui, "Ventana principal");
    title::hint(
        ui,
        "El tamano se aplica la proxima vez que abra el sistema.",
    );
    field::text(ui, "Ancho inicial (px)", &mut draft.width);
    field::text(ui, "Alto inicial (px)", &mut draft.height);
    field::text(ui, "Ancho del menu lateral (px)", &mut draft.sidebar);
}
