mod sidebar;

use eframe::egui::{CentralPanel, Context, Frame, Key, Margin, RichText, SidePanel, Ui};

use crate::components::confirm::Confirm;
use crate::env::Env;
use crate::views::categories::CategoriesView;
use crate::views::clients::ClientsView;
use crate::views::command_palette::{CommandPalette, PaletteEvent};
use crate::views::dashboard::DashboardView;
use crate::views::history::HistoryView;
use crate::views::inventory::InventoryView;
use crate::views::movements::MovementsView;
use crate::views::navigation::Route;
use crate::views::new_sale::NewSaleView;
use crate::views::profile::ProfileDialog;
use crate::views::reports::ReportsView;
use crate::views::settings::SettingsView;
use crate::views::users::UsersView;
use crate::views::view::View;
use crate::{input, theme};
use sidebar::SidebarEvent;

pub enum ShellEvent {
    None,
    Logout,
}

pub struct Shell {
    route: Option<Route>,
    view: Option<Box<dyn View>>,
    palette: Option<CommandPalette>,
    profile: Option<ProfileDialog>,
    leaving: Option<Confirm>,
}

impl Shell {
    pub fn new(ctx: &Context, env: &Env) -> Self {
        let mut shell = Self {
            route: None,
            view: None,
            palette: None,
            profile: None,
            leaving: None,
        };
        if let Some(route) = Route::available(env).first() {
            shell.open(ctx, env, *route);
        }
        shell
    }

    pub fn show(&mut self, ctx: &Context, env: &mut Env) -> ShellEvent {
        if !input::modal_open(ctx) && input::command(ctx, Key::K) {
            self.palette = Some(CommandPalette::default());
        }
        let panel = SidePanel::left("sidebar")
            .exact_width(env.config.ui.sidebar_width as f32)
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(theme::SIDEBAR)
                    .inner_margin(Margin::symmetric(0, 18)),
            );
        match panel
            .show(ctx, |ui| sidebar::show(ui, env, self.route))
            .inner
        {
            SidebarEvent::None => {}
            SidebarEvent::Open(route) => self.open(ctx, env, route),
            SidebarEvent::Profile => self.profile = ProfileDialog::open(env),
            SidebarEvent::Logout => {
                self.leaving = Some(Confirm::new(
                    "Cerrar sesion",
                    "¿Desea cerrar la sesion actual?",
                ));
            }
        }
        CentralPanel::default()
            .frame(Frame::new().fill(theme::BG).inner_margin(Margin::same(22)))
            .show(ctx, |ui| match &mut self.view {
                Some(view) if self.route.is_some_and(|route| env.can(route.permission())) => {
                    view.show(ui, env);
                }
                _ => denied(ui),
            });
        self.show_dialogs(ctx, env)
    }

    fn show_dialogs(&mut self, ctx: &Context, env: &mut Env) -> ShellEvent {
        if let Some(palette) = &mut self.palette {
            match palette.show(ctx, env) {
                PaletteEvent::None => {}
                PaletteEvent::Close => self.palette = None,
                PaletteEvent::Open(route) => {
                    self.palette = None;
                    self.open(ctx, env, route);
                }
            }
        }
        if let Some(profile) = &mut self.profile {
            if !profile.show(ctx, env) {
                self.profile = None;
            }
        }
        match self.leaving.as_ref().and_then(|confirm| confirm.show(ctx)) {
            Some(true) => return ShellEvent::Logout,
            Some(false) => self.leaving = None,
            None => {}
        }
        ShellEvent::None
    }

    fn open(&mut self, ctx: &Context, env: &Env, route: Route) {
        self.route = Some(route);
        self.view = Some(match route {
            Route::Dashboard => Box::new(DashboardView::new(ctx, env)),
            Route::Inventory => Box::new(InventoryView::new(ctx, env)),
            Route::Categories => Box::new(CategoriesView::new(ctx, env)),
            Route::Movements => Box::new(MovementsView::new(ctx, env)),
            Route::Clients => Box::new(ClientsView::new(ctx, env)),
            Route::NewSale => Box::new(NewSaleView::new(ctx, env)),
            Route::History => Box::new(HistoryView::new(ctx, env)),
            Route::Reports => Box::new(ReportsView::new(ctx, env)),
            Route::Users => Box::new(UsersView::new(ctx, env)),
            Route::Settings => Box::new(SettingsView::new(env)),
        });
    }
}

fn denied(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(120.0);
        ui.label(RichText::new("Sin acceso").size(24.0).color(theme::WARNING));
        ui.label("Su usuario no tiene permisos asignados para esta vista.");
        ui.label("Solicite acceso a un administrador.");
    });
}
