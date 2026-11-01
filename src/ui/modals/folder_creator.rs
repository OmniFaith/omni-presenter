use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use crate::ui::components::dialog;
use eframe::egui::{self, RichText};

pub fn render_folder_creator_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
) {
    if !state.show_create_folder_modal {
        return;
    }

    let tokens = state.theme.clone();
    let language = state.settings.language;
    let mut should_create = false;
    let mut should_cancel = false;

    let tab_hint = match state.selected_tab {
        0 => "workspace-e-g-official-hymnal-youth-worship",
        1 => "workspace-e-g-worship-backgrounds-intros",
        2 => "workspace-e-g-notices-sunday-liturgy",
        3 => "workspace-e-g-monthly-readings-highlights",
        4 => "workspace-e-g-service-notices-themes",
        _ => "workspace-folder-name",
    };

    let close = dialog::show(
        ctx,
        "folder_creator_modal",
        language.text("workspace-new-folder"),
        language.text("workspace-enter-a-folder-name-to-organize-your-items"),
        380.0,
        &tokens,
        |ui| {
            ui.label(
                RichText::new(crate::ui::i18n::text(ui, "workspace-folder-name"))
                    .size(12.0)
                    .strong()
                    .color(tokens.text_primary),
            );

            let text_res = ui.add(
                egui::TextEdit::singleline(&mut state.new_folder_name)
                    .hint_text(crate::ui::i18n::text(ui, tab_hint))
                    .desired_width(ui.available_width()),
            );

            // Foca o campo de texto automaticamente ao abrir
            if state.new_folder_name.is_empty() {
                text_res.request_focus();
            }

            if text_res.lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                && !state.new_folder_name.trim().is_empty()
            {
                should_create = true;
            }

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if dialog::action(ui, crate::ui::i18n::text(ui, "common-cancel"), false, &tokens).clicked() {
                    should_cancel = true;
                }

                let can_create = !state.new_folder_name.trim().is_empty();
                ui.add_enabled_ui(can_create, |ui| {
                    if dialog::action(ui, crate::ui::i18n::text(ui, "workspace-create-folder"), true, &tokens).clicked() {
                        should_create = true;
                    }
                });
            });
        },
    );

    if close || should_cancel {
        state.new_folder_name.clear();
        state.show_create_folder_modal = false;
    } else if should_create {
        let trimmed = state.new_folder_name.trim().to_string();
        if !trimmed.is_empty() {
            match state.selected_tab {
                0 => {
                    if let Some(app_db) = db {
                        let _ = app_db.add_folder(&trimmed);
                    }
                    state.open_folders.insert(trimmed);
                }
                1 => {
                    if let Some(app_db) = db {
                        let _ = app_db.add_folder_to_category("media", &trimmed);
                    }
                    state.open_media_folders.insert(trimmed);
                }
                2 => {
                    if let Some(app_db) = db {
                        let _ = app_db.add_folder_to_category("text", &trimmed);
                    }
                    state.open_text_folders.insert(trimmed);
                }
                3 => {
                    if let Some(app_db) = db {
                        let _ = app_db.add_folder_to_category("bible", &trimmed);
                    }
                    state.open_verse_folders.insert(trimmed);
                }
                4 => {
                    if let Some(app_db) = db {
                        let _ = app_db.add_folder_to_category("design", &trimmed);
                    }
                    state.open_design_folders.insert(trimmed);
                }
                _ => {}
            }
        }
        state.new_folder_name.clear();
        state.show_create_folder_modal = false;
    }
}
