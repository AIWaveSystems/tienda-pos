use eframe::egui::{Color32, Context, RichText};

use super::dialog::{Dialog, DialogEvent};
use crate::theme;

pub struct Notice {
    title: String,
    text: String,
    color: Color32,
}

impl Notice {
    pub fn info(title: &str, text: impl Into<String>) -> Self {
        Self {
            title: title.to_owned(),
            text: text.into(),
            color: theme::FG,
        }
    }

    pub fn warning(title: &str, text: impl Into<String>) -> Self {
        Self {
            color: theme::WARNING,
            ..Self::info(title, text)
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
            ui.label(RichText::new(&self.text).color(self.color));
        }) != DialogEvent::None
    }
}
