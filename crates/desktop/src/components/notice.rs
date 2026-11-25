use eframe::egui::{Context, RichText};

use super::dialog::{Dialog, DialogEvent};
use crate::theme;

pub struct Notice {
    title: String,
    text: String,
}

impl Notice {
    pub fn warning(title: &str, text: impl Into<String>) -> Self {
        Self {
            title: title.to_owned(),
            text: text.into(),
        }
    }

    pub fn show(&self, ctx: &Context) -> bool {
        let dialog = Dialog {
            title: &self.title,
            width: 400.0,
            accept: "Aceptar",
            cancel: "Cerrar",
            error: "",
        };
        dialog.show(ctx, |ui| {
            ui.label(RichText::new(&self.text).color(theme::palette().warning));
        }) != DialogEvent::None
    }
}
