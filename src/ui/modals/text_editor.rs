use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use crate::ui::components::dialog;
use eframe::egui::{self, RichText};

pub fn render_text_editor_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
) {
    if !state.show_create_text_modal {
        return;
    }

    let tokens = state.theme.clone();
    let mut should_save = false;
    let mut should_cancel = false;
    let is_editing = state.editing_text_id.is_some();
    let modal_title = if is_editing {
        state.settings.language.text("workspace-edit-text")
    } else {
        state.settings.language.text("workspace-new-text")
    };
    let modal_desc = state.settings.language.text("workspace-create-slides-with-custom-text-notices-or-messages");

    let close = dialog::show(
        ctx,
        "text_editor_modal",
        modal_title,
        modal_desc,
        460.0,
        &tokens,
        |ui| {
            ui.label(RichText::new(crate::ui::i18n::text(ui, "common-title")).size(12.0).strong().color(tokens.text_primary));
            ui.add(
                egui::TextEdit::singleline(&mut state.text_editor_title)
                    .hint_text(crate::ui::i18n::text(ui, "workspace-e-g-weekly-notices-liturgy-welcome"))
                    .desired_width(ui.available_width()),
            );

            ui.add_space(4.0);
            ui.label(RichText::new(crate::ui::i18n::text(ui, "workspace-folder-optional")).size(12.0).strong().color(tokens.text_primary));
            ui.add(
                egui::TextEdit::singleline(&mut state.text_editor_folder)
                    .hint_text(crate::ui::i18n::text(ui, "workspace-no-folder"))
                    .desired_width(ui.available_width()),
            );

            ui.add_space(4.0);
            ui.label(RichText::new(crate::ui::i18n::text(ui, "workspace-slide-content")).size(12.0).strong().color(tokens.text_primary));
            let tags_ref = &state.settings.strophe_tags;
            let tokens_ref = &tokens;
            let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
                let job = crate::ui::components::lyrics_highlighter::highlight_lyrics_job(
                    buf.as_str(),
                    tags_ref,
                    tokens_ref,
                    13.0,
                    wrap_width,
                    false,
                );
                ui.fonts_mut(|f| f.layout_job(job))
            };
            ui.add(
                egui::TextEdit::multiline(&mut state.text_editor_content)
                    .hint_text(crate::ui::i18n::text(ui, "workspace-enter-your-text-here-each-line-or-paragraph-will-be"))
                    .desired_width(ui.available_width())
                    .desired_rows(6)
                    .layouter(&mut layouter),
            );

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if dialog::action(ui, crate::ui::i18n::text(ui, "common-cancel"), false, &tokens).clicked() {
                    should_cancel = true;
                }
                let can_save = !state.text_editor_title.trim().is_empty();
                ui.add_enabled_ui(can_save, |ui| {
                    if dialog::action(ui, crate::ui::i18n::text(ui, "common-save-item"), true, &tokens).clicked() {
                        should_save = true;
                    }
                });
            });
        },
    );

    if should_save {
        let title = state.text_editor_title.trim().to_string();
        let content = state.text_editor_content.trim().to_string();
        let folder = state.text_editor_folder.trim().to_string();

        if !title.is_empty() {
            if let Some(app_db) = db {
                if let Some(id) = state.editing_text_id {
                    let _ = app_db.update_custom_text(id, &title, &content, &folder);
                    state.notify(state.settings.language.text("workspace-text-updated-successfully"));
                } else {
                    let _ = app_db.add_custom_text(&title, &content, &folder);
                    state.notify(state.settings.language.text("workspace-text-created-successfully"));
                }
                state.reload_custom_texts(app_db);
            }
            if !folder.is_empty() {
                state.open_text_folders.insert(folder);
            }
        }

        state.text_editor_title.clear();
        state.text_editor_content.clear();
        state.editing_text_id = None;
        state.show_create_text_modal = false;
    } else if should_cancel || close {
        state.text_editor_title.clear();
        state.text_editor_content.clear();
        state.editing_text_id = None;
        state.show_create_text_modal = false;
    }
}
