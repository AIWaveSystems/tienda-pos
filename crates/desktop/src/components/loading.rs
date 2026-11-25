use eframe::egui::{RichText, Spinner, Ui};

use crate::theme;

pub fn loading(ui: &mut Ui, text: &str) {
    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        ui.add(Spinner::new().size(32.0).color(theme::palette().accent));
        ui.label(RichText::new(text).color(theme::palette().muted));
    });
}
