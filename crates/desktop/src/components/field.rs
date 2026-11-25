use eframe::egui::{Id, Margin, Response, RichText, TextEdit, Ui};

use crate::theme;

pub fn text(ui: &mut Ui, label: &str, value: &mut String) -> Response {
    let name = caption(ui, label).id;
    input(ui, value, f32::INFINITY).labelled_by(name)
}

pub fn password(ui: &mut Ui, label: &str, value: &mut String) -> Response {
    let name = caption(ui, label).id;
    ui.add(edit(value).password(true).desired_width(f32::INFINITY))
        .labelled_by(name)
}

pub fn input(ui: &mut Ui, value: &mut String, width: f32) -> Response {
    ui.add(edit(value).desired_width(width))
}

pub fn search(ui: &mut Ui, value: &mut String, hint: &str, width: f32) -> Response {
    ui.add(
        edit(value)
            .hint_text(RichText::new(hint).color(theme::palette().subtle))
            .desired_width(width),
    )
}

pub fn named_search(
    ui: &mut Ui,
    name: &str,
    value: &mut String,
    hint: &str,
    width: f32,
) -> Response {
    ui.add(
        edit(value)
            .id(Id::new(("search", name)))
            .hint_text(RichText::new(hint).color(theme::palette().subtle))
            .desired_width(width),
    )
}

pub fn caption(ui: &mut Ui, label: &str) -> Response {
    ui.label(RichText::new(label).color(theme::palette().muted))
}

fn edit(value: &mut String) -> TextEdit<'_> {
    TextEdit::singleline(value).margin(Margin::symmetric(8, 6))
}
