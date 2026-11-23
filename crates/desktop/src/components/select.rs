use eframe::egui::{ComboBox, Id, Key, PopupCloseBehavior, ScrollArea, Ui};

use super::field;

pub fn select(ui: &mut Ui, id: &str, options: &[String], selected: &mut usize, width: f32) -> bool {
    let memory = Id::new(("select", id));
    let mut query: String = ui.data(|data| data.get_temp(memory)).unwrap_or_default();
    let before = *selected;
    let current = options.get(*selected).cloned().unwrap_or_default();
    let opened = ComboBox::from_id_salt(id)
        .selected_text(current)
        .width(width)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show_ui(ui, |ui| {
            let response = field::search(ui, &mut query, "Filtrar", f32::INFINITY);
            if !ui.memory(|memory| memory.has_focus(response.id)) && !response.lost_focus() {
                response.request_focus();
            }
            let visible = ordered(options, &query);
            let accepted = response.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter));
            let mut chosen = accepted.then(|| visible.first().copied()).flatten();
            ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                for index in visible {
                    if ui
                        .selectable_label(index == *selected, &options[index])
                        .clicked()
                    {
                        chosen = Some(index);
                    }
                }
            });
            if let Some(index) = chosen {
                *selected = index;
                ui.memory_mut(|memory| memory.close_popup());
            }
            chosen.is_some()
        });
    let closed = opened.inner.is_none_or(|chosen| chosen);
    ui.data_mut(|data| data.insert_temp(memory, if closed { String::new() } else { query }));
    *selected != before
}

pub fn labelled(ui: &mut Ui, label: &str, options: &[String], selected: &mut usize) -> bool {
    field::caption(ui, label);
    let width = ui.available_width();
    select(ui, label, options, selected, width)
}

pub fn ordered(options: &[String], query: &str) -> Vec<usize> {
    let query = query.trim().to_lowercase();
    let mut found: Vec<usize> = (0..options.len())
        .filter(|index| options[*index].to_lowercase().contains(&query))
        .collect();
    found.sort_by_key(|index| !options[*index].to_lowercase().starts_with(&query));
    found
}

#[cfg(test)]
mod tests {
    use super::ordered;

    #[test]
    fn prefixes_come_first() {
        let options = ["Sin cliente", "Tacna", "Clara"].map(str::to_owned);
        assert_eq!(ordered(&options, "cl"), vec![2, 0]);
        assert_eq!(ordered(&options, ""), vec![0, 1, 2]);
        assert!(ordered(&options, "zz").is_empty());
    }
}
