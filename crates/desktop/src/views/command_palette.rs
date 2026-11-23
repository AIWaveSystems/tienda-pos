use eframe::egui::{Context, Key};

use super::navigation::Route;
use crate::components::dialog::{Dialog, DialogEvent};
use crate::components::field;
use crate::env::Env;
use crate::input;

pub enum PaletteEvent {
    None,
    Close,
    Open(Route),
}

#[derive(Default)]
pub struct CommandPalette {
    query: String,
    position: usize,
}

impl CommandPalette {
    pub fn show(&mut self, ctx: &Context, env: &Env) -> PaletteEvent {
        let query = self.query.trim().to_lowercase();
        let routes: Vec<Route> = Route::available(env)
            .into_iter()
            .filter(|route| route.label().to_lowercase().contains(&query))
            .collect();
        if input::pressed(ctx, Key::ArrowDown) {
            self.position += 1;
        }
        if input::pressed(ctx, Key::ArrowUp) {
            self.position = self.position.saturating_sub(1);
        }
        self.position = self.position.min(routes.len().saturating_sub(1));
        let mut chosen = None;
        let dialog = Dialog {
            title: "Ir a...",
            width: 420.0,
            accept: "Abrir",
            cancel: "Cerrar",
            error: "",
        };
        let event = dialog.show(ctx, |ui| {
            let response = field::search(ui, &mut self.query, "Buscar vista", f32::INFINITY);
            response.request_focus();
            for (index, route) in routes.iter().enumerate() {
                let label = ui.selectable_label(index == self.position, route.label());
                if label.clicked() {
                    chosen = Some(*route);
                }
            }
        });
        if event == DialogEvent::Accept {
            chosen = chosen.or(routes.get(self.position).copied());
        }
        match (chosen, event) {
            (Some(route), _) => PaletteEvent::Open(route),
            (None, DialogEvent::Cancel) => PaletteEvent::Close,
            _ => PaletteEvent::None,
        }
    }
}
