use eframe::egui::{Button, Color32, RichText, Ui, Vec2};

use crate::theme;

pub fn accent(ui: &mut Ui, text: &str) -> bool {
    filled(ui, text, theme::palette().accent)
}

pub fn danger(ui: &mut Ui, text: &str) -> bool {
    filled(ui, text, theme::palette().danger)
}

pub fn plain(ui: &mut Ui, text: &str) -> bool {
    ui.button(text).clicked()
}

pub fn wide_accent(ui: &mut Ui, text: &str) -> bool {
    let size = Vec2::new(ui.available_width(), 38.0);
    let label = RichText::new(text)
        .color(theme::palette().on_accent)
        .strong();
    let button = Button::new(label)
        .fill(theme::palette().accent)
        .min_size(size);
    ui.vertical_centered(|ui| ui.add(button).clicked()).inner
}

fn filled(ui: &mut Ui, text: &str, color: Color32) -> bool {
    let label = RichText::new(text)
        .color(theme::palette().on_accent)
        .strong();
    ui.add(Button::new(label).fill(color)).clicked()
}
