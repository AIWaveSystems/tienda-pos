use eframe::egui::Context;
use pos_domain::Category;

use crate::components::dialog::{Dialog, DialogEvent};
use crate::components::{field, title};
use crate::env::Env;

pub struct CategoryForm {
    item: Category,
    editing: bool,
    error: String,
}

impl CategoryForm {
    pub fn create() -> Self {
        Self {
            item: Category::default(),
            editing: false,
            error: String::new(),
        }
    }

    pub fn edit(item: Category) -> Self {
        Self {
            item,
            editing: true,
            error: String::new(),
        }
    }

    pub fn show(&mut self, ctx: &Context, env: &Env) -> Option<bool> {
        let name = if self.editing {
            "Editar Categoria"
        } else {
            "Nueva Categoria"
        };
        match Dialog::form(name, 420.0, &self.error).show(ctx, |ui| {
            title::hint(ui, "Los campos marcados con * son obligatorios.");
            field::text(ui, "Nombre *", &mut self.item.name);
            field::text(ui, "Descripcion", &mut self.item.description);
            if self.editing {
                ui.checkbox(&mut self.item.active, "Categoria activa");
            }
        }) {
            DialogEvent::None => None,
            DialogEvent::Cancel => Some(false),
            DialogEvent::Accept => match env.services.categories.save(self.item.clone()) {
                Ok(_) => Some(true),
                Err(failure) => {
                    self.error = failure.to_string();
                    None
                }
            },
        }
    }
}
