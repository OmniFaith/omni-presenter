use crate::core::bible::BibleTranslation;
use crate::core::state::ControlPanelState;
use eframe::egui;

pub(super) fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

pub(super) fn current_book(state: &ControlPanelState, bible: &BibleTranslation) -> usize {
    bible
        .books
        .iter()
        .position(|b| b.name == state.bible_book)
        .unwrap_or(0)
}

pub(super) fn grid_metrics(ui: &egui::Ui, target: f32, spacing: f32) -> (usize, f32) {
    let width = ui.available_width().max(target);
    let columns = (((width + spacing) / (target + spacing)).floor() as usize).max(1);
    let cell = ((width - (columns - 1) as f32 * spacing) / columns as f32)
        .floor()
        .max(30.0);
    (columns, cell)
}
