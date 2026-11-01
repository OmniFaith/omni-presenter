use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use crate::ui::components::dialog;
use eframe::egui::{self, RichText};

pub fn render_verse_saver_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
) {
    if !state.show_save_verse_modal {
        return;
    }

    let tokens = state.theme.clone();
    let language = state.settings.language;
    let mut should_save = false;
    let mut should_cancel = false;

    let close = dialog::show(
        ctx,
        "verse_saver_modal",
        language.text("workspace-save-verse"),
        language.text("workspace-save-the-bible-passage-to-favorites-for-quick-presentation"),
        460.0,
        &tokens,
        |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(crate::ui::i18n::text(ui, "chat-reference")).size(12.0).strong().color(tokens.text_primary));
                    ui.add(
                        egui::TextEdit::singleline(&mut state.verse_editor_ref)
                            .hint_text(crate::ui::i18n::text(ui, "workspace-e-g-john-3-16-psalms-23-1"))
                            .desired_width(220.0),
                    );
                });

                ui.vertical(|ui| {
                    ui.label(RichText::new(crate::ui::i18n::text(ui, "workspace-version")).size(12.0).strong().color(tokens.text_primary));
                    ui.add(
                        egui::TextEdit::singleline(&mut state.verse_editor_version)
                            .hint_text(crate::ui::i18n::text(ui, "workspace-e-g-kjv-asv-niv"))
                            .desired_width(120.0),
                    );
                });
            });

            ui.add_space(4.0);
            ui.label(RichText::new(crate::ui::i18n::text(ui, "workspace-folder-optional")).size(12.0).strong().color(tokens.text_primary));
            ui.add(
                egui::TextEdit::singleline(&mut state.verse_editor_folder)
                    .hint_text(crate::ui::i18n::text(ui, "workspace-no-folder"))
                    .desired_width(ui.available_width()),
            );

            ui.add_space(4.0);
            ui.label(RichText::new(crate::ui::i18n::text(ui, "workspace-verse-text")).size(12.0).strong().color(tokens.text_primary));
            ui.add(
                egui::TextEdit::multiline(&mut state.verse_editor_text)
                    .hint_text(crate::ui::i18n::text(ui, "workspace-bible-text-to-present"))
                    .desired_width(ui.available_width())
                    .desired_rows(5),
            );

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if dialog::action(ui, crate::ui::i18n::text(ui, "common-cancel"), false, &tokens).clicked() {
                    should_cancel = true;
                }
                let can_save = !state.verse_editor_ref.trim().is_empty() && !state.verse_editor_text.trim().is_empty();
                ui.add_enabled_ui(can_save, |ui| {
                    if dialog::action(ui, crate::ui::i18n::text(ui, "workspace-save-verse-label"), true, &tokens).clicked() {
                        should_save = true;
                    }
                });
            });
        },
    );

    if should_save {
        let reference = state.verse_editor_ref.trim().to_string();
        let version = state.verse_editor_version.trim().to_string();
        let text = state.verse_editor_text.trim().to_string();
        let folder = state.verse_editor_folder.trim().to_string();

        if !reference.is_empty() && !text.is_empty() {
            if let Some(app_db) = db {
                let _ = app_db.add_saved_verse(&reference, &version, &text, &folder);
                state.reload_saved_verses(app_db);
                state.notify(state.settings.language.text("workspace-verse-saved-successfully"));
            }
            if !folder.is_empty() {
                state.open_verse_folders.insert(folder);
            }
        }

        state.verse_editor_ref.clear();
        state.verse_editor_version.clear();
        state.verse_editor_text.clear();
        state.verse_editor_folder.clear();
        state.show_save_verse_modal = false;
    } else if should_cancel || close {
        state.verse_editor_ref.clear();
        state.verse_editor_version.clear();
        state.verse_editor_text.clear();
        state.verse_editor_folder.clear();
        state.show_save_verse_modal = false;
    }
}
