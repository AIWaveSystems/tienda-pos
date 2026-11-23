use eframe::egui::Ui;

use crate::env::Env;

pub trait View {
    fn show(&mut self, ui: &mut Ui, env: &mut Env);
}
