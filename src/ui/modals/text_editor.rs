use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use eframe::egui;

pub fn render_text_editor_modal(
    _ctx: &egui::Context,
    state: &mut ControlPanelState,
    _db: Option<&AppDb>,
) {
    if state.show_create_text_modal {
        let id = state.editing_text_id;
        let title = std::mem::take(&mut state.text_editor_title);
        let folder = std::mem::take(&mut state.text_editor_folder);
        let content = std::mem::take(&mut state.text_editor_content);
        state.show_create_text_modal = false;
        state.editing_text_id = None;
        state.open_text_editor(id, &title, &folder, &content);
    }
}
