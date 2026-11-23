use eframe::egui::{Color32, Frame, Margin, RichText, Ui};

use crate::theme;

pub fn surface<R>(ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(theme::SURFACE)
        .corner_radius(6)
        .inner_margin(Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            content(ui)
        })
        .inner
}

pub fn stats(ui: &mut Ui, cards: &[(&str, String, Color32)]) {
    ui.columns(cards.len(), |columns| {
        for (ui, (label, value, color)) in columns.iter_mut().zip(cards) {
            surface(ui, |ui| {
                ui.label(RichText::new(*label).color(theme::MUTED));
                ui.label(RichText::new(value).size(20.0).strong().color(*color));
            });
        }
    });
}

pub fn check(ui: &mut Ui, value: &mut bool, label: &str, detail: &str) -> bool {
    surface(ui, |ui| {
        let changed = ui.checkbox(value, label).changed();
        if !detail.is_empty() {
            ui.label(RichText::new(detail).small().color(theme::SUBTLE));
        }
        changed
    })
}
