use std::time::{Duration, Instant};

use eframe::egui::{Align2, Area, Context, Frame, Id, Margin, Order, RichText, Stroke, Vec2};

use crate::theme;

const LIFETIME: Duration = Duration::from_secs(4);

pub struct Toast {
    text: String,
    created: Instant,
}

impl Toast {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            created: Instant::now(),
        }
    }

    fn alive(&self) -> bool {
        self.created.elapsed() < LIFETIME
    }
}

pub fn show(ctx: &Context, toasts: &mut Vec<Toast>) {
    toasts.retain(Toast::alive);
    if toasts.is_empty() {
        return;
    }
    let colors = theme::palette();
    Area::new(Id::new("toasts"))
        .order(Order::Tooltip)
        .interactable(false)
        .anchor(Align2::CENTER_BOTTOM, Vec2::new(0.0, -20.0))
        .show(ctx, |ui| {
            for toast in toasts.iter() {
                Frame::new()
                    .fill(colors.surface)
                    .stroke(Stroke::new(1.0_f32, colors.success))
                    .corner_radius(6)
                    .inner_margin(Margin::symmetric(16, 12))
                    .show(ui, |ui| {
                        ui.label(RichText::new(&toast.text).color(colors.success));
                    });
            }
        });
    ctx.request_repaint_after(Duration::from_millis(500));
}
