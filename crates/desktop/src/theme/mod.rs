mod palette;
mod style;

use std::sync::atomic::{AtomicBool, Ordering};

use eframe::egui::Context;
use palette::{Palette, DARK, LIGHT};

pub const ICON: &[u8] = include_bytes!("../../../../assets/icon.png");

pub const NAMES: [(&str, &str); 2] = [("dark", "Oscuro"), ("light", "Claro")];

static LIGHT_MODE: AtomicBool = AtomicBool::new(false);

pub fn apply(ctx: &Context, name: &str) {
    LIGHT_MODE.store(name == "light", Ordering::Relaxed);
    style::apply(ctx, palette(), name == "light");
}

pub fn palette() -> &'static Palette {
    if LIGHT_MODE.load(Ordering::Relaxed) {
        &LIGHT
    } else {
        &DARK
    }
}
