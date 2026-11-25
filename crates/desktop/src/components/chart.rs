use eframe::egui::{Align2, CornerRadius, FontId, Pos2, Rect, Sense, Ui, Vec2};

use crate::theme;

pub struct Bar {
    pub label: String,
    pub value: f32,
    pub detail: String,
}

pub fn bars(ui: &mut Ui, items: &[Bar], height: f32) {
    let colors = theme::palette();
    let (area, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    let highest = items.iter().map(|item| item.value).fold(0.0_f32, f32::max);
    if highest <= 0.0 || items.is_empty() {
        return;
    }
    let slot = area.width() / items.len() as f32;
    let base = area.bottom() - 20.0;
    let reach = base - area.top();
    for (index, item) in items.iter().enumerate() {
        let left = area.left() + slot * index as f32;
        let top = base - reach * (item.value.max(0.0) / highest);
        let bar = Rect::from_min_max(
            Pos2::new(left + slot * 0.18, top),
            Pos2::new(left + slot * 0.82, base),
        );
        let column = Rect::from_min_max(
            Pos2::new(left, area.top()),
            Pos2::new(left + slot, area.bottom()),
        );
        let response = ui
            .interact(column, ui.id().with(("bar", index)), Sense::hover())
            .on_hover_text(&item.detail);
        let color = if response.hovered() {
            colors.purple
        } else {
            colors.accent
        };
        ui.painter().rect_filled(bar, CornerRadius::same(3), color);
        ui.painter().text(
            Pos2::new(left + slot / 2.0, area.bottom()),
            Align2::CENTER_BOTTOM,
            &item.label,
            FontId::proportional(11.0),
            colors.subtle,
        );
    }
}
