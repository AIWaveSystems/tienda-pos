use eframe::egui::{Id, Key, Ui};
use pos_domain::Entity;

use super::paginator::Paginator;
use super::table::{table, Column, Row};
use super::{field, loading};
use crate::env::Env;
use crate::input;
use crate::task::Task;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ListAction {
    Open,
    Delete,
    Print,
    Refresh,
}

pub struct List<T> {
    pub items: Vec<T>,
    pub query: String,
    id: &'static str,
    pages: Paginator,
    selected: Option<String>,
    load: Option<Task<Vec<T>>>,
    focus: bool,
}

impl<T: Entity> List<T> {
    pub fn new(id: &'static str) -> Self {
        Self {
            items: Vec::new(),
            query: String::new(),
            id,
            pages: Paginator::default(),
            selected: None,
            load: None,
            focus: true,
        }
    }

    pub fn load(&mut self, task: Task<Vec<T>>) {
        self.load = Some(task);
    }

    pub fn poll(&mut self, env: &mut Env) {
        if let Some(outcome) = Task::poll(&mut self.load) {
            if let Some(items) = env.check(outcome) {
                self.items = items;
            }
        }
    }

    pub fn selected(&self) -> Option<&T> {
        let id = self.selected.as_deref()?;
        self.items.iter().find(|item| item.id() == id)
    }

    pub fn require(&self, env: &mut Env) -> Option<T> {
        let found = self.selected().cloned();
        if found.is_none() {
            env.warn("Seleccionar", "Seleccione un registro de la lista");
        }
        found
    }

    pub fn reset_page(&mut self) {
        self.pages.reset();
    }

    pub fn search(&mut self, ui: &mut Ui, hint: &str, width: f32) {
        let response = ui
            .push_id(Id::new((self.id, "search")), |ui| {
                field::search(ui, &mut self.query, hint, width)
            })
            .inner;
        if std::mem::take(&mut self.focus) {
            response.request_focus();
        }
        if response.changed() {
            self.pages.reset();
            self.selected = None;
        }
        if response.has_focus() {
            if input::pressed(ui.ctx(), Key::ArrowDown) {
                response.surrender_focus();
            }
            if input::pressed(ui.ctx(), Key::Escape) {
                self.query.clear();
            }
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        columns: &[Column],
        matches: &[&T],
        row: impl Fn(&T) -> Row,
        reserved: f32,
    ) -> Option<ListAction> {
        if self.load.is_some() {
            loading::loading(ui, "Cargando...");
            return None;
        }
        let range = self.pages.range(matches.len());
        let visible = &matches[range];
        let current = self.position(visible);
        let rows: Vec<Row> = visible.iter().map(|item| row(item)).collect();
        let height = (ui.available_height() - reserved).max(120.0);
        let event = ui
            .allocate_ui([ui.available_width(), height].into(), |ui| {
                table(ui, self.id, columns, &rows, current, height - 34.0)
            })
            .inner;
        if let Some(index) = event.clicked.or(event.activated) {
            self.selected = Some(visible[index].id().to_owned());
        }
        ui.horizontal(|ui| self.pages.show(ui, matches.len()));
        let action = self.keys(ui, visible);
        event.activated.map(|_| ListAction::Open).or(action)
    }

    fn keys(&mut self, ui: &Ui, visible: &[&T]) -> Option<ListAction> {
        let ctx = ui.ctx();
        if input::modal_open(ctx) {
            return None;
        }
        if input::command(ctx, Key::F) {
            self.focus = true;
        }
        if input::command(ctx, Key::P) {
            return Some(ListAction::Print);
        }
        if input::pressed(ctx, Key::F5) {
            return Some(ListAction::Refresh);
        }
        if !input::free(ctx) {
            return None;
        }
        if input::pressed(ctx, Key::PageDown) {
            self.pages.next();
        }
        if input::pressed(ctx, Key::PageUp) {
            self.pages.previous();
        }
        if input::pressed(ctx, Key::Escape) {
            self.focus = true;
        }
        self.arrows(ui, visible);
        self.selected.as_ref()?;
        if input::pressed(ctx, Key::Delete) {
            return Some(ListAction::Delete);
        }
        input::pressed(ctx, Key::Enter).then_some(ListAction::Open)
    }

    fn arrows(&mut self, ui: &Ui, visible: &[&T]) {
        let Some(last) = visible.len().checked_sub(1) else {
            return;
        };
        let current = self.position(visible);
        let target = if input::pressed(ui.ctx(), Key::ArrowDown) {
            Some(current.map_or(0, |index| (index + 1).min(last)))
        } else if input::pressed(ui.ctx(), Key::ArrowUp) {
            if current == Some(0) {
                self.focus = true;
            }
            Some(current.map_or(0, |index| index.saturating_sub(1)))
        } else if self.selected.is_none() && input::pressed(ui.ctx(), Key::Enter) {
            Some(0)
        } else {
            None
        };
        if let Some(index) = target {
            self.selected = Some(visible[index].id().to_owned());
        }
    }

    fn position(&self, visible: &[&T]) -> Option<usize> {
        let id = self.selected.as_deref()?;
        visible.iter().position(|item| item.id() == id)
    }
}
