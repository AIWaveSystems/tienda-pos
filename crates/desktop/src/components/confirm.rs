use eframe::egui::Context;

use super::dialog::{Dialog, DialogEvent};

pub struct Confirm {
    title: String,
    text: String,
}

impl Confirm {
    pub fn new(title: &str, text: impl Into<String>) -> Self {
        Self {
            title: title.to_owned(),
            text: text.into(),
        }
    }

    pub fn show(&self, ctx: &Context) -> Option<bool> {
        let dialog = Dialog {
            title: &self.title,
            width: 380.0,
            accept: "Si",
            cancel: "No",
            error: "",
        };
        match dialog.show(ctx, |ui| {
            ui.label(&self.text);
        }) {
            DialogEvent::None => None,
            DialogEvent::Accept => Some(true),
            DialogEvent::Cancel => Some(false),
        }
    }
}
