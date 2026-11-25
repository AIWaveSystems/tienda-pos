use eframe::egui::{Align, Button, Color32, Layout, RichText, Ui, Vec2};

use crate::env::Env;
use crate::theme;
use crate::views::navigation::Route;

pub enum SidebarEvent {
    None,
    Open(Route),
    Profile,
    Logout,
}

pub fn show(ui: &mut Ui, env: &Env, current: Option<Route>) -> SidebarEvent {
    let mut event = SidebarEvent::None;
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new(env.config.business_name())
                .size(19.0)
                .strong()
                .color(theme::palette().accent),
        );
        if let Some(user) = env.services.auth.current() {
            ui.label(user.full_name);
            ui.label(
                RichText::new(user.role.label())
                    .small()
                    .color(theme::palette().subtle),
            );
        }
    });
    ui.add_space(8.0);
    ui.separator();
    ui.spacing_mut().item_spacing.y = 0.0;
    let routes = Route::available(env);
    let reserved = 110.0;
    let height =
        ((ui.available_height() - reserved) / routes.len().max(1) as f32).clamp(26.0, 38.0);
    for route in routes {
        let selected = current == Some(route);
        let color = if selected {
            theme::palette().accent
        } else {
            theme::palette().fg
        };
        let label = format!("{}  {}", route.icon(), route.label());
        if entry(ui, &label, color, selected, height) {
            event = SidebarEvent::Open(route);
        }
    }
    ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
        if entry(ui, "Cerrar sesion", theme::palette().danger, false, 36.0) {
            event = SidebarEvent::Logout;
        }
        if entry(ui, "Mi Perfil", theme::palette().muted, false, 36.0) {
            event = SidebarEvent::Profile;
        }
        ui.separator();
    });
    event
}

fn entry(ui: &mut Ui, text: &str, color: Color32, selected: bool, height: f32) -> bool {
    let fill = if selected {
        theme::palette().surface
    } else {
        theme::palette().sidebar
    };
    let size = Vec2::new(ui.available_width(), height);
    let button = Button::new(RichText::new(format!("   {text}")).color(color))
        .fill(fill)
        .stroke(eframe::egui::Stroke::NONE)
        .corner_radius(0);
    ui.allocate_ui_with_layout(
        size,
        Layout::left_to_right(Align::Center)
            .with_main_justify(true)
            .with_main_align(Align::Min),
        |ui| ui.add(button.min_size(size)).clicked(),
    )
    .inner
}
