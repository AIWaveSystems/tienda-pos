use eframe::egui::{
    Color32, Context, CornerRadius, FontFamily, FontId, Margin, Stroke, TextStyle, Theme, Visuals,
};

pub const ICON: &[u8] = include_bytes!("../../../assets/icon.png");

pub const BG: Color32 = Color32::from_rgb(0x1e, 0x1e, 0x2e);
pub const SIDEBAR: Color32 = Color32::from_rgb(0x18, 0x18, 0x25);
pub const SURFACE: Color32 = Color32::from_rgb(0x31, 0x32, 0x44);
pub const SURFACE_ALT: Color32 = Color32::from_rgb(0x45, 0x47, 0x5a);
pub const BORDER: Color32 = Color32::from_rgb(0x58, 0x5b, 0x70);
pub const FG: Color32 = Color32::from_rgb(0xcd, 0xd6, 0xf4);
pub const MUTED: Color32 = Color32::from_rgb(0xa6, 0xad, 0xc8);
pub const SUBTLE: Color32 = Color32::from_rgb(0x7f, 0x84, 0x9c);
pub const ACCENT: Color32 = Color32::from_rgb(0x89, 0xb4, 0xfa);
pub const SUCCESS: Color32 = Color32::from_rgb(0xa6, 0xe3, 0xa1);
pub const WARNING: Color32 = Color32::from_rgb(0xf9, 0xe2, 0xaf);
pub const DANGER: Color32 = Color32::from_rgb(0xf3, 0x8b, 0xa8);
pub const PURPLE: Color32 = Color32::from_rgb(0xcb, 0xa6, 0xf7);

pub fn apply(ctx: &Context) {
    ctx.set_theme(Theme::Dark);
    ctx.set_visuals(visuals());
    ctx.style_mut(|style| {
        style.spacing.item_spacing = [8.0, 8.0].into();
        style.spacing.button_padding = [14.0, 7.0].into();
        style.spacing.window_margin = Margin::same(22);
        style.spacing.interact_size.y = 30.0;
        for (kind, size) in [
            (TextStyle::Small, 12.0),
            (TextStyle::Body, 14.0),
            (TextStyle::Button, 14.0),
            (TextStyle::Heading, 22.0),
        ] {
            style
                .text_styles
                .insert(kind, FontId::new(size, FontFamily::Proportional));
        }
    });
}

fn visuals() -> Visuals {
    let mut visuals = Visuals::dark();
    visuals.override_text_color = Some(FG);
    visuals.panel_fill = BG;
    visuals.window_fill = BG;
    visuals.window_stroke = Stroke::new(1.0_f32, BORDER);
    visuals.extreme_bg_color = SURFACE;
    visuals.faint_bg_color = SIDEBAR;
    visuals.hyperlink_color = ACCENT;
    visuals.selection.bg_fill = ACCENT.gamma_multiply(0.35);
    visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);
    let widgets = &mut visuals.widgets;
    for (state, fill, border) in [
        (&mut widgets.noninteractive, BG, SURFACE_ALT),
        (&mut widgets.inactive, SURFACE_ALT, BORDER),
        (&mut widgets.hovered, BORDER, ACCENT),
        (&mut widgets.active, SURFACE, ACCENT),
        (&mut widgets.open, SURFACE, ACCENT),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = Stroke::new(1.0_f32, border);
        state.fg_stroke = Stroke::new(1.0_f32, FG);
        state.corner_radius = CornerRadius::same(4);
    }
    visuals
}
