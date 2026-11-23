use eframe::egui::{CentralPanel, Context, ViewportCommand};
use pos_config::{ConfigStore, CredentialStore};

use crate::boot::{self, Booted, Progress};
use crate::components::confirm::Confirm;
use crate::components::notice::Notice;
use crate::env::Env;
use crate::task::Task;
use crate::views::change_password::ChangePasswordView;
use crate::views::login::LoginView;
use crate::views::shell::{Shell, ShellEvent};
use crate::views::splash::SplashView;

enum Screen {
    Splash(SplashView),
    Login(LoginView),
    ChangePassword(ChangePasswordView),
    Shell(Box<Shell>),
}

pub struct App {
    screen: Screen,
    store: ConfigStore,
    credentials: CredentialStore,
    boot: Option<Task<Booted>>,
    env: Option<Env>,
    leaving: Option<Confirm>,
    closing: bool,
}

impl App {
    pub fn new(ctx: &Context, store: ConfigStore) -> Self {
        let credentials = CredentialStore::new(store.paths());
        let progress = Progress::default();
        let task = {
            let (store, credentials, progress) =
                (store.clone(), credentials.clone(), progress.clone());
            Task::spawn(ctx, move || boot::boot(&store, &credentials, &progress))
        };
        Self {
            screen: Screen::Splash(SplashView::new(ctx, progress)),
            store,
            credentials,
            boot: Some(task),
            env: None,
            leaving: None,
            closing: false,
        }
    }

    fn finish_boot(&mut self) {
        match Task::poll(&mut self.boot) {
            Some(Ok(booted)) => {
                let mut env = Env {
                    services: booted.services,
                    config: booted.config,
                    store: self.store.clone(),
                    credentials: self.credentials.clone(),
                    notices: Vec::new(),
                    printing: None,
                };
                if let Some(text) = booted.warning {
                    env.warn("Almacenamiento no disponible", text);
                }
                self.env = Some(env);
                self.screen = Screen::Login(LoginView::default());
            }
            Some(Err(failure)) => {
                if let Screen::Splash(view) = &mut self.screen {
                    view.fail(failure);
                }
            }
            None => {}
        }
    }

    fn show_screen(&mut self, ctx: &Context) {
        let Some(env) = &mut self.env else {
            if let Screen::Splash(view) = &mut self.screen {
                CentralPanel::default().show(ctx, |ui| view.show(ui));
            }
            return;
        };
        let next = match &mut self.screen {
            Screen::Splash(_) => None,
            Screen::Login(view) => CentralPanel::default()
                .show(ctx, |ui| view.show(ui, env))
                .inner
                .map(|user| {
                    if user.is_default {
                        Screen::ChangePassword(ChangePasswordView::new(user.id))
                    } else {
                        Screen::Shell(Box::new(Shell::new(ctx, env)))
                    }
                }),
            Screen::ChangePassword(view) => CentralPanel::default()
                .show(ctx, |ui| view.show(ui, env))
                .inner
                .then(|| Screen::Shell(Box::new(Shell::new(ctx, env)))),
            Screen::Shell(shell) => match shell.show(ctx, env) {
                ShellEvent::None => None,
                ShellEvent::Logout => {
                    env.services.auth.logout();
                    Some(Screen::Login(LoginView::default()))
                }
            },
        };
        if let Some(screen) = next {
            self.screen = screen;
        }
    }

    fn show_overlays(&mut self, ctx: &Context) {
        if let Some(env) = &mut self.env {
            if let Some(mut dialog) = env.printing.take() {
                if dialog.show(ctx) {
                    env.printing = Some(dialog);
                }
            }
            if env
                .notices
                .first()
                .is_some_and(|notice| Notice::show(notice, ctx))
            {
                env.notices.remove(0);
            }
        }
        if let Some(confirm) = &self.leaving {
            match confirm.show(ctx) {
                Some(true) => {
                    self.closing = true;
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
                Some(false) => self.leaving = None,
                None => {}
            }
        }
    }

    pub fn tick(&mut self, ctx: &Context) {
        self.finish_boot();
        self.guard_close(ctx);
        self.show_screen(ctx);
        self.show_overlays(ctx);
    }

    fn guard_close(&mut self, ctx: &Context) {
        let requested = ctx.input(|input| input.viewport().close_requested());
        if requested && !self.closing && self.env.is_some() {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.leaving = Some(Confirm::new("Salir", "¿Desea salir del sistema?"));
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.tick(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(env) = &self.env {
            env.services.close();
        }
    }
}
