use eframe::egui::{
    ColorImage, Context, Image, ProgressBar, RichText, TextureHandle, TextureOptions, Ui,
};

use crate::boot::Progress;
use crate::theme;

pub struct SplashView {
    progress: Progress,
    logo: Option<TextureHandle>,
    failure: Option<String>,
}

impl SplashView {
    pub fn new(ctx: &Context, progress: Progress) -> Self {
        Self {
            progress,
            logo: logo(ctx),
            failure: None,
        }
    }

    pub fn fail(&mut self, failure: String) {
        self.failure = Some(failure);
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let (message, fraction) = self.progress.get();
        ui.add_space(ui.available_height() * 0.25);
        ui.vertical_centered(|ui| {
            if let Some(logo) = &self.logo {
                ui.add(Image::new(logo).fit_to_exact_size([88.0, 88.0].into()));
            }
            ui.label(RichText::new("Ventas POS").size(26.0).strong());
            ui.label(RichText::new("Sistema de punto de venta e inventario").color(theme::SUBTLE));
            ui.add_space(16.0);
            match &self.failure {
                Some(failure) => {
                    ui.label(RichText::new(failure).color(theme::DANGER));
                }
                None => {
                    ui.add(
                        ProgressBar::new(fraction)
                            .desired_width(340.0)
                            .desired_height(5.0)
                            .fill(theme::ACCENT),
                    );
                    ui.label(RichText::new(message).color(theme::MUTED));
                }
            }
            ui.label(
                RichText::new(concat!("v", env!("CARGO_PKG_VERSION")))
                    .small()
                    .color(theme::SUBTLE),
            );
        });
    }
}

fn logo(ctx: &Context) -> Option<TextureHandle> {
    let image = image::load_from_memory(theme::ICON).ok()?.to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let pixels = ColorImage::from_rgba_unmultiplied(size, image.as_raw());
    Some(ctx.load_texture("logo", pixels, TextureOptions::LINEAR))
}
