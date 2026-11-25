use std::path::PathBuf;

use eframe::egui::{Color32, Context, RichText, Ui};
use pos_config::{AppConfig, BackendKind, ConfigError, Credentials};
use pos_storage::cloud::CloudAccount;
use pos_storage::{copy_all, StorageError, StoreFactory};

use super::draft::whole;
use crate::components::{button, card, field, select, title};
use crate::env::Env;
use crate::task::Task;
use crate::theme;

#[derive(Clone, Default, PartialEq)]
struct Connection {
    host: String,
    port: String,
    database: String,
    user: String,
    secure: bool,
    password: String,
}

pub struct StoragePanel {
    choice: usize,
    fields: Connection,
    config: AppConfig,
    account: String,
    tested: Option<(BackendKind, Connection)>,
    testing: Option<(Connection, Task<()>)>,
    linking: Option<Task<CloudAccount>>,
    saving: Option<Task<PathBuf>>,
    status: (String, Color32),
}

impl StoragePanel {
    pub fn new(env: &Env) -> Self {
        let backend = env.config.storage.backend;
        let mut panel = Self {
            choice: BackendKind::ALL
                .iter()
                .position(|kind| *kind == backend)
                .unwrap_or_default(),
            fields: Connection::default(),
            config: env.config.clone(),
            account: String::new(),
            tested: None,
            testing: None,
            linking: None,
            saving: None,
            status: (String::new(), theme::palette().muted),
        };
        panel.reset(env);
        panel
    }

    fn backend(&self) -> BackendKind {
        BackendKind::ALL[self.choice]
    }

    fn reset(&mut self, env: &Env) {
        let storage = &self.config.storage;
        self.tested = None;
        self.testing = None;
        self.status.0.clear();
        self.account.clear();
        self.fields = match self.backend() {
            BackendKind::Mysql | BackendKind::Postgresql => {
                let sql = match self.backend() {
                    BackendKind::Mysql => &storage.mysql,
                    _ => &storage.postgresql,
                };
                Connection {
                    host: sql.host.clone(),
                    port: sql.port.to_string(),
                    database: sql.database.clone(),
                    user: sql.user.clone(),
                    secure: sql.ssl,
                    password: String::new(),
                }
            }
            BackendKind::Mongodb => Connection {
                host: storage.mongodb.host.clone(),
                port: storage.mongodb.port.to_string(),
                database: storage.mongodb.database.clone(),
                user: storage.mongodb.user.clone(),
                secure: storage.mongodb.use_srv,
                password: String::new(),
            },
            _ => Connection::default(),
        };
        if self.backend().is_cloud() {
            match env.credentials.load(self.backend()) {
                Ok(credentials) => self.account = connected(&credentials),
                Err(failure) => self.status = (failure.to_string(), theme::palette().danger),
            }
        }
    }

    pub fn ready(&self) -> Result<(), String> {
        let backend = self.backend();
        if backend == BackendKind::LocalJson {
            return Ok(());
        }
        if backend.is_cloud() {
            if self.account.is_empty() {
                return Err(
                    "Debe conectar la cuenta antes de cambiar a este almacenamiento.".to_owned(),
                );
            }
            return Ok(());
        }
        if self.tested != Some((backend, self.fields.clone())) {
            return Err(
                "Debe probar la conexion antes de cambiar a este almacenamiento.".to_owned(),
            );
        }
        Ok(())
    }

    pub fn apply(&self, config: &mut AppConfig) -> Result<(), String> {
        let backend = self.backend();
        config.storage.backend = backend;
        if !backend.is_database() {
            return Ok(());
        }
        let port = whole(&self.fields.port, "puerto", 1, 65535)? as u16;
        let host = match self.fields.host.trim() {
            "" => "localhost".to_owned(),
            host => host.to_owned(),
        };
        let database = self.fields.database.trim().to_owned();
        let user = self.fields.user.trim().to_owned();
        if backend == BackendKind::Mongodb {
            let mongo = &mut config.storage.mongodb;
            mongo.host = host;
            mongo.port = port;
            mongo.database = database;
            mongo.user = user;
            mongo.use_srv = self.fields.secure;
            return Ok(());
        }
        let sql = match backend {
            BackendKind::Mysql => &mut config.storage.mysql,
            _ => &mut config.storage.postgresql,
        };
        sql.host = host;
        sql.port = port;
        sql.database = database;
        sql.user = user;
        sql.ssl = self.fields.secure;
        Ok(())
    }

    pub fn store_password(&self, env: &Env) -> Result<(), ConfigError> {
        if !self.backend().is_database() || self.fields.password.is_empty() {
            return Ok(());
        }
        let credentials = Credentials {
            password: self.fields.password.clone(),
            ..Credentials::default()
        };
        env.credentials.save(self.backend(), &credentials)
    }

    pub fn migrate(&self, ctx: &Context, env: &Env, config: AppConfig) -> Task<AppConfig> {
        let factory = StoreFactory::new(env.store.paths().clone(), env.credentials.clone());
        let current = env.config.clone();
        let credentials = self.credentials(env);
        Task::spawn(ctx, move || -> Result<AppConfig, StorageError> {
            let source = factory.primary(&current)?;
            let target = if config.storage.backend == BackendKind::LocalJson {
                factory.local()?
            } else {
                StoreFactory::connect(&config, &credentials?)?
            };
            copy_all(source.as_ref(), target.as_ref())?;
            source.close();
            target.close();
            Ok(config)
        })
    }

    fn credentials(&self, env: &Env) -> Result<Credentials, ConfigError> {
        if self.backend().is_database() && !self.fields.password.is_empty() {
            return Ok(Credentials {
                password: self.fields.password.clone(),
                ..Credentials::default()
            });
        }
        env.credentials.load(self.backend())
    }

    pub fn show(&mut self, ui: &mut Ui, env: &mut Env) {
        self.poll(env);
        title::heading(ui, "Ubicacion de los archivos");
        for (label, path) in [
            ("Configuracion", env.store.paths().config_file()),
            ("Datos", env.store.paths().data_dir.clone()),
        ] {
            card::surface(ui, |ui| {
                title::hint(ui, label);
                ui.label(path.display().to_string());
            });
        }
        self.backup(ui, env);
        title::heading(ui, "Motor de almacenamiento");
        let labels: Vec<String> = BackendKind::ALL
            .iter()
            .map(|kind| kind.label().to_owned())
            .collect();
        if select::labelled(ui, "Backend", &labels, &mut self.choice) {
            self.reset(env);
        }
        let backend = self.backend();
        if backend == BackendKind::LocalJson {
            title::hint(ui, "No requiere configuracion adicional.");
        } else {
            title::hint(
                ui,
                "Mientras este motor este activo, los datos se respaldan automaticamente \
                 en una copia local (JSON) cada pocos segundos.",
            );
        }
        if backend.is_cloud() {
            self.cloud(ui, env);
        } else if backend.is_database() {
            self.database(ui, env);
        }
        title::status(ui, &self.status.0, self.status.1);
        ui.label(
            RichText::new("Cambiar el motor de almacenamiento requiere reiniciar la aplicacion.")
                .small()
                .color(theme::palette().warning),
        );
    }

    fn backup(&mut self, ui: &mut Ui, env: &mut Env) {
        title::heading(ui, "Copia de seguridad");
        title::hint(
            ui,
            "Guarda todos los datos actuales en una carpeta JSON con fecha.",
        );
        match Task::poll(&mut self.saving) {
            Some(Ok(path)) => env.toast(format!("Copia creada en {}", path.display())),
            Some(Err(failure)) => env.warn("Copia de seguridad", failure),
            None => {}
        }
        let idle = self.saving.is_none();
        let clicked = ui
            .add_enabled_ui(idle, |ui| button::plain(ui, "Crear copia..."))
            .inner;
        if !clicked {
            return;
        }
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            let services = env.services.clone();
            self.saving = Some(Task::spawn(ui.ctx(), move || services.backup(&folder)));
        }
    }

    fn database(&mut self, ui: &mut Ui, env: &Env) {
        field::text(ui, "Host", &mut self.fields.host);
        field::text(ui, "Puerto", &mut self.fields.port);
        field::text(ui, "Base de datos", &mut self.fields.database);
        field::text(ui, "Usuario", &mut self.fields.user);
        field::password(
            ui,
            "Contrasena (vacia para usar la guardada)",
            &mut self.fields.password,
        );
        let label = if self.backend() == BackendKind::Mongodb {
            "Usar mongodb+srv (Atlas)"
        } else {
            "Usar SSL/TLS"
        };
        card::check(ui, &mut self.fields.secure, label, "");
        let idle = self.testing.is_none();
        let clicked = ui
            .add_enabled_ui(idle, |ui| button::plain(ui, "Probar conexion"))
            .inner;
        if clicked {
            self.test(ui.ctx(), env);
        }
    }

    fn test(&mut self, ctx: &Context, env: &Env) {
        let mut config = self.config.clone();
        if let Err(message) = self.apply(&mut config) {
            self.status = (message, theme::palette().danger);
            return;
        }
        let credentials = self.credentials(env);
        self.status = ("Probando conexion...".to_owned(), theme::palette().muted);
        let task = Task::spawn(ctx, move || -> Result<(), StorageError> {
            let store = StoreFactory::connect(&config, &credentials?)?;
            store.ping()?;
            store.close();
            Ok(())
        });
        self.testing = Some((self.fields.clone(), task));
    }

    fn cloud(&mut self, ui: &mut Ui, env: &Env) {
        ui.label(
            RichText::new(
                "Cada operacion requiere conexion a internet y es mas lenta. No recomendado \
                 si mas de una caja usa el mismo almacenamiento al mismo tiempo.",
            )
            .small()
            .color(theme::palette().warning),
        );
        let backend = self.backend();
        if !self.account.is_empty() {
            let text = format!("Conectado como: {}", self.account);
            title::status(ui, &text, theme::palette().success);
            if button::plain(ui, "Desconectar") {
                match env.credentials.clear(backend) {
                    Ok(()) => self.reset(env),
                    Err(failure) => self.status = (failure.to_string(), theme::palette().danger),
                }
            }
            return;
        }
        let idle = self.linking.is_none();
        let label = format!("Conectar con {}", backend.label());
        let clicked = ui
            .add_enabled_ui(idle, |ui| button::plain(ui, &label))
            .inner;
        if clicked {
            self.status = (
                "Abriendo el navegador para autorizar el acceso...".to_owned(),
                theme::palette().muted,
            );
            let credentials = env.credentials.clone();
            self.linking = Some(Task::spawn(
                ui.ctx(),
                move || -> Result<CloudAccount, StorageError> {
                    let account = CloudAccount::authorize(backend)?;
                    credentials.save(
                        backend,
                        &Credentials {
                            refresh_token: account.refresh_token.clone(),
                            account_email: account.email.clone(),
                            ..Credentials::default()
                        },
                    )?;
                    Ok(account)
                },
            ));
        }
    }

    fn poll(&mut self, env: &Env) {
        let Some((fields, task)) = self.testing.take() else {
            self.poll_account(env);
            return;
        };
        let mut slot = Some(task);
        match Task::poll(&mut slot) {
            Some(Ok(())) => {
                self.tested = Some((self.backend(), fields));
                self.status = ("Conexion exitosa".to_owned(), theme::palette().success);
            }
            Some(Err(failure)) => {
                self.tested = None;
                self.status = (
                    format!("Error de conexion: {failure}"),
                    theme::palette().danger,
                );
            }
            None => self.testing = slot.map(|task| (fields, task)),
        }
        self.poll_account(env);
    }

    fn poll_account(&mut self, env: &Env) {
        match Task::poll(&mut self.linking) {
            Some(Ok(_)) => {
                self.reset(env);
                self.status = (
                    "Cuenta conectada correctamente".to_owned(),
                    theme::palette().success,
                );
            }
            Some(Err(failure)) => {
                self.status = (
                    format!("No se pudo conectar: {failure}"),
                    theme::palette().danger,
                );
            }
            None => {}
        }
    }
}

fn connected(credentials: &Credentials) -> String {
    if credentials.refresh_token.is_empty() {
        return String::new();
    }
    match credentials.account_email.as_str() {
        "" => "cuenta conectada".to_owned(),
        email => email.to_owned(),
    }
}
