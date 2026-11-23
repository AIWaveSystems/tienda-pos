use eframe::egui::Context;
use pos_domain::Client;

use crate::components::dialog::{Dialog, DialogEvent};
use crate::components::{field, title};
use crate::env::Env;

pub struct ClientForm {
    item: Client,
    editing: bool,
    error: String,
}

impl ClientForm {
    pub fn create() -> Self {
        Self {
            item: Client::default(),
            editing: false,
            error: String::new(),
        }
    }

    pub fn edit(item: Client) -> Self {
        Self {
            item,
            editing: true,
            error: String::new(),
        }
    }

    pub fn show(&mut self, ctx: &Context, env: &Env) -> Option<bool> {
        let name = if self.editing {
            "Editar Cliente"
        } else {
            "Nuevo Cliente"
        };
        match Dialog::form(name, 460.0, &self.error).show(ctx, |ui| {
            title::hint(ui, "Los campos marcados con * son obligatorios.");
            field::text(ui, "Nombre *", &mut self.item.name);
            field::text(ui, "Email", &mut self.item.email);
            field::text(ui, "Telefono", &mut self.item.phone);
            field::text(ui, "Direccion", &mut self.item.address);
            field::text(ui, "NIT / ID Fiscal", &mut self.item.tax_id);
            field::text(ui, "Notas", &mut self.item.notes);
            if self.editing {
                ui.checkbox(&mut self.item.active, "Cliente activo");
            }
        }) {
            DialogEvent::None => None,
            DialogEvent::Cancel => Some(false),
            DialogEvent::Accept => match env.services.clients.save(self.item.clone()) {
                Ok(_) => Some(true),
                Err(failure) => {
                    self.error = failure.to_string();
                    None
                }
            },
        }
    }
}
