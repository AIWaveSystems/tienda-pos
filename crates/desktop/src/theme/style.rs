use eframe::egui::{
    Context, CornerRadius, FontFamily, FontId, Margin, Stroke, TextStyle, Theme, Visuals,
};

use super::Palette;

pub fn apply(ctx: &Context, colors: &Palette, light: bool) {
    let theme = if light { Theme::Light } else { Theme::Dark };
    ctx.set_theme(theme);
    ctx.set_visuals(visuals(colors, theme));
    ctx.all_styles_mut(|style| {
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

fn visuals(colors: &Palette, theme: Theme) -> Visuals {
    let mut visuals = theme.default_visuals();
    visuals.override_text_color = Some(colors.fg);
    visuals.panel_fill = colors.bg;
    visuals.window_fill = colors.bg;
    visuals.window_stroke = Stroke::new(1.0_f32, colors.border);
    visuals.extreme_bg_color = colors.surface;
    visuals.faint_bg_color = colors.sidebar;
    visuals.hyperlink_color = colors.accent;
    visuals.selection.bg_fill = colors.accent.gamma_multiply(0.35);
    visuals.selection.stroke = Stroke::new(1.0_f32, colors.accent);
    let widgets = &mut visuals.widgets;
    for (state, fill, border) in [
        (&mut widgets.noninteractive, colors.bg, colors.surface_alt),
        (&mut widgets.inactive, colors.surface_alt, colors.border),
        (&mut widgets.hovered, colors.border, colors.accent),
        (&mut widgets.active, colors.surface, colors.accent),
        (&mut widgets.open, colors.surface, colors.accent),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = Stroke::new(1.0_f32, border);
        state.fg_stroke = Stroke::new(1.0_f32, colors.fg);
        state.corner_radius = CornerRadius::same(6);
    }
    visuals
}
