use eframe::egui::{Color32, Context};
use pos_services::print::{self, Document, PrintOutcome, Printers};

use super::dialog::{Dialog, DialogEvent};
use super::{button, select, title};
use crate::task::Task;
use crate::theme;

const BROWSER: &str = "Elegir en el navegador (PDF)";

pub struct PrintDialog {
    document: Document,
    detail: String,
    detecting: Option<Task<Printers>>,
    started: bool,
    options: Vec<String>,
    selected: usize,
    status: (String, Color32),
}

impl PrintDialog {
    pub fn new(document: Document, detail: impl Into<String>) -> Self {
        Self {
            document,
            detail: detail.into(),
            detecting: None,
            started: false,
            options: vec![BROWSER.to_owned()],
            selected: 0,
            status: (String::new(), theme::palette().fg),
        }
    }

    pub fn show(&mut self, ctx: &Context) -> bool {
        self.detect(ctx);
        let name = self.document.name.clone();
        let dialog = Dialog {
            title: &name,
            width: 460.0,
            accept: "Imprimir",
            cancel: "Cerrar",
            error: "",
        };
        match dialog.show(ctx, |ui| {
            title::hint(ui, &self.detail);
            select::labelled(ui, "Impresora", &self.options, &mut self.selected);
            if self.detecting.is_some() {
                title::hint(ui, "Buscando impresoras...");
            } else {
                let count = self.options.len() - 1;
                title::hint(ui, &format!("{count} impresora(s) disponible(s)."));
            }
            ui.horizontal(|ui| {
                if button::plain(ui, "Vista previa") {
                    self.preview();
                }
                if button::plain(ui, "Guardar como...") {
                    self.save();
                }
                if self.document.csv.is_some() && button::plain(ui, "Exportar CSV") {
                    self.export();
                }
            });
            title::status(ui, &self.status.0, self.status.1);
        }) {
            DialogEvent::Accept => {
                self.print();
                true
            }
            DialogEvent::Cancel => false,
            DialogEvent::None => true,
        }
    }

    fn detect(&mut self, ctx: &Context) {
        if !self.started {
            self.started = true;
            self.detecting = Some(Task::spawn(ctx, || Ok::<_, String>(print::list_printers())));
        }
        if let Some(Ok(printers)) = Task::poll(&mut self.detecting) {
            self.options = printers.names;
            self.options.push(BROWSER.to_owned());
            self.selected = self
                .options
                .iter()
                .position(|name| *name == printers.default)
                .unwrap_or(self.options.len() - 1);
        }
    }

    fn preview(&mut self) {
        self.status = match print::preview(&self.document) {
            Ok(()) => (
                "Vista previa abierta en el navegador".to_owned(),
                theme::palette().success,
            ),
            Err(failure) => (
                format!("No se pudo abrir la vista previa: {failure}"),
                theme::palette().danger,
            ),
        };
    }

    fn print(&mut self) {
        let printer = self.options[self.selected].clone();
        if printer == BROWSER {
            self.preview();
            return;
        }
        self.status = match print::print(&self.document, &printer) {
            Ok(PrintOutcome::Sent) => (format!("Enviado a {printer}"), theme::palette().success),
            Ok(PrintOutcome::OpenedInBrowser) => (
                format!("No se pudo enviar a {printer}. Se abrio en el navegador."),
                theme::palette().warning,
            ),
            Err(failure) => (failure.to_string(), theme::palette().danger),
        };
    }

    fn export(&mut self) {
        let Some(content) = &self.document.csv else {
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .set_file_name(format!("{}.csv", self.document.file_name()))
            .add_filter("CSV", &["csv"])
            .save_file()
        else {
            return;
        };
        self.status = match print::export(content, &path) {
            Ok(()) => (
                format!("Exportado a {}", path.display()),
                theme::palette().success,
            ),
            Err(failure) => (failure.to_string(), theme::palette().danger),
        };
    }

    fn save(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .set_file_name(format!("{}.html", self.document.file_name()))
            .add_filter("Documento HTML", &["html"])
            .save_file()
        else {
            return;
        };
        self.status = match print::save(&self.document, &path) {
            Ok(()) => (
                format!("Guardado en {}", path.display()),
                theme::palette().success,
            ),
            Err(failure) => (failure.to_string(), theme::palette().danger),
        };
    }
}
