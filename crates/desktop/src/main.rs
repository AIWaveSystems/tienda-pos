#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod boot;
mod components;
mod env;
mod input;
mod task;
#[cfg(test)]
mod tests;
mod theme;
mod views;

use app::App;
use eframe::egui::ViewportBuilder;
use pos_config::{ConfigStore, Paths};

fn main() -> eframe::Result {
    let store = ConfigStore::new(Paths::detect());
    let ui = store.load().unwrap_or_default().ui;
    let mut viewport = ViewportBuilder::default()
        .with_title("Ventas POS - Sistema de Punto de Venta")
        .with_inner_size([ui.window_width as f32, ui.window_height as f32])
        .with_min_inner_size([880.0, 560.0]);
    if let Ok(icon) = eframe::icon_data::from_png_bytes(theme::ICON) {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport,
        centered: true,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "ventas-pos-rust",
        options,
        Box::new(|creation| {
            theme::apply(&creation.egui_ctx);
            Ok(Box::new(App::new(&creation.egui_ctx, store)))
        }),
    )
}
