use eframe::egui::{
    Align2, CursorIcon, FontId, Id, Margin, Pos2, Rect, Response, RichText, Sense, TextEdit, Ui,
    WidgetInfo, WidgetType,
};

use crate::theme;

const EYE_WIDTH: f32 = 34.0;
const PASSWORD_MARGIN: Margin = Margin {
    left: 8,
    right: 36,
    top: 6,
    bottom: 6,
};

pub fn text(ui: &mut Ui, label: &str, value: &mut String) -> Response {
    let name = caption(ui, label).id;
    input(ui, value, f32::INFINITY).labelled_by(name)
}

pub fn password(ui: &mut Ui, label: &str, value: &mut String) -> Response {
    let name = caption(ui, label).id;
    let memory = ui.id().with(("password_visible", label));
    let visible = !value.is_empty() && ui.data(|data| data.get_temp(memory).unwrap_or(false));
    let response = ui
        .add(
            edit(value)
                .password(!visible)
                .margin(PASSWORD_MARGIN)
                .desired_width(f32::INFINITY),
        )
        .labelled_by(name);
    let toggled = !value.is_empty() && eye(ui, &response, visible, label);
    ui.data_mut(|data| data.insert_temp(memory, visible != toggled));
    response
}

fn eye(ui: &mut Ui, field: &Response, visible: bool, label: &str) -> bool {
    let colors = theme::palette();
    let (color, action) = if visible {
        (colors.accent, "Ocultar")
    } else {
        (colors.subtle, "Mostrar")
    };
    let area = Rect::from_min_max(
        Pos2::new(field.rect.right() - EYE_WIDTH, field.rect.top()),
        field.rect.max,
    );
    let hint = format!("{action} {}", label.to_lowercase());
    let response = ui
        .interact(area, field.id.with("eye"), Sense::CLICK)
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(action);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, &hint));
    ui.painter().text(
        area.center(),
        Align2::CENTER_CENTER,
        "👁",
        FontId::proportional(16.0),
        color,
    );
    response.clicked()
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
