use crate::core::bible::BibleTranslation;
use crate::core::state::{ControlPanelState, SettingsTab};
use crate::services::db::AppDb;
use eframe::egui::{self, RichText};

pub(super) fn open_bible_settings(state: &mut ControlPanelState) {
    state.settings_ui.open = true;
    state.settings_ui.tab = SettingsTab::Bible;
    state.bible.import_error.clear();
}

pub fn render_import_dialog(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
) {
    if !state.bible.show_import {
        return;
    }
    let mut open = true;
    let mut import = false;
    let mut cancel = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
    egui::Window::new(state.settings.language.text("workspace-add-bible-translation")).id(egui::Id::unique("bible_import")).open(&mut open).collapsible(false).resizable(false).default_width(430.0).anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO).show(ctx, |ui| {
        ui.label(crate::ui::i18n::text(ui, "workspace-add-a-translation-to-look-up-and-present-offline"));
        ui.add_space(10.0);
        ui.horizontal(|ui| { ui.label(crate::ui::i18n::text(ui, "workspace-code")); ui.add(egui::TextEdit::singleline(&mut state.bible.import_code).desired_width(100.0).hint_text(crate::ui::i18n::text(ui, "workspace-kjv-asv"))); });
        ui.horizontal(|ui| { ui.label(crate::ui::i18n::text(ui, "common-name")); ui.add(egui::TextEdit::singleline(&mut state.bible.import_name).desired_width(290.0).hint_text(crate::ui::i18n::text(ui, "workspace-translation-name"))); });
        let import_picker_id = ui.make_persistent_id("bible_import_file_picker");
        let import_task = ui
            .ctx()
            .data_mut(|data| data.get_temp::<crate::services::tasks::BackgroundTask<Option<std::path::PathBuf>>>(import_picker_id));
        if let Some(task) = &import_task {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(33));
            if let Some(result) = task.take() {
                ui.ctx().data_mut(|data| data.remove::<crate::services::tasks::BackgroundTask<Option<std::path::PathBuf>>>(import_picker_id));
                if let Ok(Some(path)) = result {
                    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(crate::ui::i18n::text(ui, "workspace-bible"));
                    if state.bible.import_code.is_empty() { state.bible.import_code = stem.chars().filter(|c| c.is_alphanumeric()).take(12).collect::<String>().to_uppercase(); }
                    if state.bible.import_name.is_empty() { state.bible.import_name = stem.into(); }
                    state.bible.import_path = Some(path);
                }
            }
        }
        ui.horizontal(|ui| {
            if ui.add_enabled(import_task.is_none(), egui::Button::new(crate::ui::i18n::text(ui, "workspace-select-json-file"))).clicked() {
                let filter = crate::ui::i18n::text(ui, "workspace-bible-json").to_owned();
                let task = crate::services::tasks::BackgroundTask::spawn_with_repaint(ui.ctx(), move || {
                    Ok(rfd::FileDialog::new().add_filter(filter, &["json"]).pick_file())
                });
                ui.ctx().data_mut(|data| data.insert_temp(import_picker_id, task));
            }
            if import_task.is_some() {
                ui.spinner();
            }
        });
        if let Some(path) = &state.bible.import_path { ui.label(RichText::new(path.file_name().unwrap_or_default().to_string_lossy()).size(12.0).color(state.theme.text_secondary)); }
        egui::CollapsingHeader::new(crate::ui::i18n::text(ui, "workspace-accepted-format")).show(ui, |ui| {
            ui.label(crate::ui::i18n::text(ui, "workspace-json-with-66-books-in-biblical-order-each-book-contains"));
            ui.code(r#"[{"name":"Gênesis","chapters":[["Versículo 1","Versículo 2"]]}, …]"#);
            ui.label(crate::ui::i18n::text(ui, "workspace-objects-with-books-and-the-scrollmapper-format-are-also-supported"));
        });
        if !state.bible.import_error.is_empty() { ui.colored_label(state.theme.status_danger, crate::ui::i18n::message(ui, &state.bible.import_error)); }
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.add_enabled(state.bible.import_path.is_some(), egui::Button::new(crate::ui::i18n::text(ui, "workspace-add-translation"))).clicked() { import = true; }
            if ui.button(crate::ui::i18n::text(ui, "control-cancel")).clicked() { cancel = true; }
        });
    });
    if import {
        let result = (|| {
            let path = state
                .bible
                .import_path
                .as_ref()
                .ok_or_else(|| state.settings.language.text("workspace-select-a-json-file").to_owned())?;
            if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 40 * 1024 * 1024 {
                return Err(state.settings.language.text("workspace-the-file-exceeds-the-40-mb-limit").to_owned());
            }
            let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
            let translation = BibleTranslation::from_json(
                &bytes,
                &state.bible.import_code,
                &state.bible.import_name,
            )?;
            let db = db.ok_or_else(|| state.settings.language.text("workspace-the-database-is-unavailable-to-save-this-translation").to_owned())?;
            db.save_bible_translation(&translation)
                .map_err(|e| state.settings.language.format("workspace-could-not-save-label", &[("e", &e.to_string())]))?;
            Ok::<_, String>(translation)
        })();
        match result {
            Ok(translation) => {
                state.bible_version = translation.id.clone();
                state.notify(state.settings.language.format("workspace-added", &[("name", &translation.name)]));
                state.bible.library.insert(translation);
                state.bible.preview_key.clear();
                state.bible.search_key.clear();
                state.bible.scroll_to_verse = true;
                state.bible.show_import = false;
            }
            Err(error) => state.bible.import_error = error,
        }
    }
    if !open || cancel {
        state.bible.show_import = false;
    }
}
