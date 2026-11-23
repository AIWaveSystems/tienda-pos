use std::ops::Range;

use eframe::egui::{Button, RichText, Ui};

use super::select::select;
use crate::theme;

const SIZES: [usize; 5] = [20, 30, 40, 50, 100];
const ALL: usize = SIZES.len();

pub struct Paginator {
    choice: usize,
    page: usize,
}

impl Default for Paginator {
    fn default() -> Self {
        Self { choice: 1, page: 0 }
    }
}

impl Paginator {
    pub fn reset(&mut self) {
        self.page = 0;
    }

    pub fn next(&mut self) {
        self.page += 1;
    }

    pub fn previous(&mut self) {
        self.page = self.page.saturating_sub(1);
    }

    pub fn range(&mut self, total: usize) -> Range<usize> {
        let size = self.size(total);
        self.page = self.page.min(self.pages(total) - 1);
        let start = self.page * size;
        start..(start + size).min(total)
    }

    pub fn show(&mut self, ui: &mut Ui, total: usize) {
        let options: Vec<String> = SIZES
            .iter()
            .map(usize::to_string)
            .chain(["Todos".to_owned()])
            .collect();
        if select(ui, "page_size", &options, &mut self.choice, 80.0) {
            self.page = 0;
        }
        ui.label(RichText::new("por pagina").small().color(theme::SUBTLE));
        let last = self.pages(total) - 1;
        let shown = self.range(total);
        if step(ui, "<<", self.page > 0) {
            self.page = 0;
        }
        if step(ui, "<", self.page > 0) {
            self.previous();
        }
        ui.label(RichText::new(summary(&shown, total, self.page, last)).color(theme::MUTED));
        if step(ui, ">", self.page < last) {
            self.next();
        }
        if step(ui, ">>", self.page < last) {
            self.page = last;
        }
    }

    fn size(&self, total: usize) -> usize {
        if self.choice == ALL {
            total.max(1)
        } else {
            SIZES[self.choice]
        }
    }

    fn pages(&self, total: usize) -> usize {
        total.div_ceil(self.size(total)).max(1)
    }
}

fn step(ui: &mut Ui, text: &str, enabled: bool) -> bool {
    ui.add_enabled(enabled, Button::new(text)).clicked()
}

fn summary(shown: &Range<usize>, total: usize, page: usize, last: usize) -> String {
    if total == 0 {
        return "Sin registros".to_owned();
    }
    format!(
        "{}-{} de {total}   (pagina {}/{})",
        shown.start + 1,
        shown.end,
        page + 1,
        last + 1
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_are_clamped() {
        let mut pages = Paginator::default();
        pages.next();
        assert_eq!(pages.range(65), 30..60);
        pages.next();
        pages.next();
        assert_eq!(pages.range(65), 60..65);
        assert_eq!(pages.range(10), 0..10);
        pages.choice = ALL;
        assert_eq!(pages.range(500), 0..500);
        assert_eq!(pages.range(0), 0..0);
    }
}
