use eframe::egui::{Align, Context, Frame, Id, Layout, Margin, Modal, RichText, ScrollArea, Ui};

use super::button;
use crate::{input, theme};

#[derive(Clone, Copy, PartialEq)]
pub enum DialogEvent {
    None,
    Accept,
    Cancel,
}

pub struct Dialog<'a> {
    pub title: &'a str,
    pub width: f32,
    pub accept: &'a str,
    pub cancel: &'a str,
    pub error: &'a str,
}

impl<'a> Dialog<'a> {
    pub fn form(title: &'a str, width: f32, error: &'a str) -> Self {
        Self {
            title,
            width,
            accept: "Guardar",
            cancel: "Cancelar",
            error,
        }
    }

    pub fn show(&self, ctx: &Context, body: impl FnOnce(&mut Ui)) -> DialogEvent {
        let mut event = DialogEvent::None;
        let frame = Frame::popup(&ctx.style())
            .fill(theme::palette().bg)
            .inner_margin(Margin::same(24));
        let modal = Modal::new(Id::new(("dialog", self.title)))
            .frame(frame)
            .show(ctx, |ui| {
                ui.set_width(self.width);
                ui.label(
                    RichText::new(self.title)
                        .size(18.0)
                        .strong()
                        .color(theme::palette().accent),
                );
                ui.add_space(6.0);
                ScrollArea::vertical()
                    .max_height(ctx.screen_rect().height() * 0.62)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        body(ui);
                    });
                ui.add_space(10.0);
                if !self.error.is_empty() {
                    ui.label(RichText::new(self.error).color(theme::palette().danger));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if button::accent(ui, self.accept) {
                        event = DialogEvent::Accept;
                    }
                    if button::plain(ui, self.cancel) {
                        event = DialogEvent::Cancel;
                    }
                });
            });
        if event == DialogEvent::None && modal.is_top_modal {
            if modal.should_close() {
                event = DialogEvent::Cancel;
            } else if !modal.any_popup_open && input::pressed(ctx, eframe::egui::Key::Enter) {
                event = DialogEvent::Accept;
            }
        }
        event
    }
}
