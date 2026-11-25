use eframe::egui::{Align, Layout, RichText, Ui};

use crate::theme;

pub fn title(ui: &mut Ui, text: &str, detail: &str) {
    ui.label(
        RichText::new(text)
            .size(24.0)
            .strong()
            .color(theme::palette().accent),
    );
    if !detail.is_empty() {
        hint(ui, detail);
    }
    ui.add_space(8.0);
}

pub fn title_bar(ui: &mut Ui, text: &str, actions: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(text)
                .size(24.0)
                .strong()
                .color(theme::palette().accent),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), actions);
    });
    ui.add_space(8.0);
}

pub fn heading(ui: &mut Ui, text: &str) {
    ui.add_space(6.0);
    ui.label(
        RichText::new(text)
            .size(16.0)
            .strong()
            .color(theme::palette().warning),
    );
}

pub fn hint(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).small().color(theme::palette().muted));
}

pub fn status(ui: &mut Ui, text: &str, color: eframe::egui::Color32) {
    if !text.is_empty() {
        ui.label(RichText::new(text).color(color));
    }
}
