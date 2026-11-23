use eframe::egui::{Align, Color32, Layout, RichText, Sense, Ui};
use egui_extras::{Column as Width, TableBuilder};

use crate::theme;

#[derive(Clone, Copy)]
pub struct Column {
    pub title: &'static str,
    pub width: f32,
    pub right: bool,
}

pub const fn left(title: &'static str, width: f32) -> Column {
    Column {
        title,
        width,
        right: false,
    }
}

pub const fn right(title: &'static str, width: f32) -> Column {
    Column {
        title,
        width,
        right: true,
    }
}

#[derive(Default)]
pub struct Row {
    pub cells: Vec<String>,
    pub color: Option<Color32>,
}

impl Row {
    pub fn new(cells: Vec<String>) -> Self {
        Self { cells, color: None }
    }
}

#[derive(Default)]
pub struct TableEvent {
    pub clicked: Option<usize>,
    pub activated: Option<usize>,
}

pub fn table(
    ui: &mut Ui,
    id: &str,
    columns: &[Column],
    rows: &[Row],
    selected: Option<usize>,
    height: f32,
) -> TableEvent {
    let mut event = TableEvent::default();
    let mut builder = TableBuilder::new(ui)
        .id_salt(id)
        .striped(true)
        .resizable(true)
        .sense(Sense::click())
        .cell_layout(Layout::left_to_right(Align::Center))
        .auto_shrink([false, false])
        .min_scrolled_height(0.0)
        .max_scroll_height(height);
    for (index, column) in columns.iter().enumerate() {
        builder = builder.column(if index == 0 {
            Width::remainder().at_least(column.width).clip(true)
        } else {
            Width::initial(column.width).at_least(40.0).clip(true)
        });
    }
    builder
        .header(30.0, |mut header| {
            for column in columns {
                header.col(|ui| {
                    cell(
                        ui,
                        column.right,
                        RichText::new(column.title).small().strong(),
                    );
                });
            }
        })
        .body(|body| {
            body.rows(30.0, rows.len(), |mut line| {
                let index = line.index();
                let row = &rows[index];
                line.set_selected(selected == Some(index));
                for (column, value) in columns.iter().zip(&row.cells) {
                    line.col(|ui| {
                        let color = row.color.unwrap_or(theme::FG);
                        cell(ui, column.right, RichText::new(value).color(color));
                    });
                }
                let response = line.response();
                if response.clicked() {
                    event.clicked = Some(index);
                }
                if response.double_clicked() {
                    event.activated = Some(index);
                }
            });
        });
    event
}

fn cell(ui: &mut Ui, right: bool, text: RichText) {
    let layout = if right {
        Layout::right_to_left(Align::Center)
    } else {
        Layout::left_to_right(Align::Center)
    };
    ui.with_layout(layout, |ui| {
        ui.add(eframe::egui::Label::new(text).selectable(false).truncate());
    });
}
