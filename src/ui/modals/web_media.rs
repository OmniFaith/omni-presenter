use crate::{
    core::state::ControlPanelState,
    services::{
        db::AppDb,
        tasks::BackgroundTask,
        web_video::{download_web_video, fetch_web_video_title, is_valid_web_url},
    },
    ui::components::dialog,
};
use eframe::egui::{self, RichText};
use egui_phosphor::regular::*;

pub fn render_web_media_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
) {
    let language = state.settings.language;

    // 1. Poll running background task
    if let Some(task) = &state.web_media_modal.task
        && let Some(result) = task.take()
    {
        state.web_media_modal.task = None;
        state.web_media_modal.is_loading = false;
        match result {
            Ok((title, file_path)) => {
                let folder = state.web_media_modal.target_folder.clone();
                if let Some(db) = db {
                    if let Err(e) = db.add_media_item(&title, &file_path, "video", &folder) {
                        state.web_media_modal.error_message = Some(e.to_string());
                        return;
                    }
                    state.reload_media_items(db);
                } else {
                    let id = state
                        .media_items
                        .iter()
                        .filter_map(|item| item.id)
                        .min()
                        .unwrap_or(0)
                        .min(0)
                        - 1;
                    state.media_items.push(crate::core::models::MediaItem {
                        id: Some(id),
                        title: title.clone(),
                        file_path: file_path.clone(),
                        file_type: "video".into(),
                        folder: folder.clone(),
                    });
                }
                if !folder.is_empty() {
                    state.open_media_folders.insert(folder);
                }
                state.notify(language.text("library-web-video-success"));
                state.web_media_modal.reset();
                state.web_media_modal.open = false;
            }
            Err(err) => {
                state.web_media_modal.error_message = Some(err);
            }
        }
    }

    if !state.web_media_modal.open {
        return;
    }

    let tokens = state.theme.clone();
    let mut should_submit = false;
    let mut should_cancel = false;

    // Collect available folders
    let mut folders: Vec<String> = db
        .and_then(|d| d.get_folders_by_category("media").ok())
        .unwrap_or_default();
    for item in &state.media_items {
        if !item.folder.is_empty() && !folders.contains(&item.folder) {
            folders.push(item.folder.clone());
        }
    }
    folders.sort();

    let close = dialog::show(
        ctx,
        "web_media_modal",
        "library-add-web-video",
        "library-web-video-subtitle",
        460.0,
        &tokens,
        |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

            // 1. URL field
            ui.label(
                RichText::new(language.text("library-web-video-url"))
                    .size(12.0)
                    .strong()
                    .color(tokens.text_primary),
            );

            let url_edit = ui.add(
                egui::TextEdit::singleline(&mut state.web_media_modal.url)
                    .hint_text(language.text("library-web-video-url-hint"))
                    .desired_width(ui.available_width()),
            );

            if state.web_media_modal.url.is_empty() && !state.web_media_modal.is_loading {
                url_edit.request_focus();
            }

            if url_edit.lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                && !state.web_media_modal.url.trim().is_empty()
                && !state.web_media_modal.is_loading
            {
                should_submit = true;
            }

            ui.add_space(4.0);

            // 2. Title field (optional)
            ui.label(
                RichText::new(language.text("library-web-video-title"))
                    .size(12.0)
                    .strong()
                    .color(tokens.text_primary),
            );

            let title_edit = ui.add(
                egui::TextEdit::singleline(&mut state.web_media_modal.title)
                    .hint_text(language.text("library-web-video-title-hint"))
                    .desired_width(ui.available_width()),
            );

            if title_edit.lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                && !state.web_media_modal.url.trim().is_empty()
                && !state.web_media_modal.is_loading
            {
                should_submit = true;
            }

            ui.add_space(4.0);

            // 3. Target folder selection
            ui.label(
                RichText::new(language.text("library-web-video-folder"))
                    .size(12.0)
                    .strong()
                    .color(tokens.text_primary),
            );

            let root_label = language.text("library-web-video-folder-root");
            let selected_label = if state.web_media_modal.target_folder.is_empty() {
                root_label
            } else {
                &state.web_media_modal.target_folder
            };

            egui::ComboBox::from_id_salt("web_media_folder_select")
                .selected_text(selected_label)
                .width(ui.available_width() - 8.0)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(
                            state.web_media_modal.target_folder.is_empty(),
                            format!("{FOLDER} {root_label}"),
                        )
                        .clicked()
                    {
                        state.web_media_modal.target_folder.clear();
                    }
                    for f in &folders {
                        if ui
                            .selectable_label(
                                state.web_media_modal.target_folder == *f,
                                format!("{FOLDER} {f}"),
                            )
                            .clicked()
                        {
                            state.web_media_modal.target_folder = f.clone();
                        }
                    }
                });

            ui.add_space(4.0);

            // 4. Download vs Direct Stream toggle
            ui.checkbox(
                &mut state.web_media_modal.download_offline,
                language.text("library-web-video-offline-checkbox"),
            );
            ui.label(
                RichText::new(if state.web_media_modal.download_offline {
                    language.text("library-web-video-downloading")
                } else {
                    language.text("library-web-video-stream-hint")
                })
                .size(11.0)
                .color(tokens.text_muted),
            );

            // 5. Status or Error indicator
            if state.web_media_modal.is_loading {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spinner();
                    let loading_text = if state.web_media_modal.download_offline {
                        language.text("library-web-video-downloading")
                    } else {
                        language.text("services-loading-video")
                    };
                    ui.label(
                        RichText::new(loading_text)
                            .size(13.0)
                            .color(tokens.accent),
                    );
                });
            } else if let Some(error) = &state.web_media_modal.error_message {
                ui.add_space(6.0);
                let formatted = language.format("library-web-video-error", &[("error", error)]);
                ui.label(
                    RichText::new(format!("{WARNING} {formatted}"))
                        .size(12.0)
                        .color(tokens.status_danger),
                );
            }

            ui.add_space(10.0);

            // 6. Action Buttons
            ui.horizontal(|ui| {
                if dialog::action(ui, "common-cancel", false, &tokens).clicked()
                    && !state.web_media_modal.is_loading
                {
                    should_cancel = true;
                }

                let can_submit = !state.web_media_modal.url.trim().is_empty()
                    && !state.web_media_modal.is_loading;
                ui.add_enabled_ui(can_submit, |ui| {
                    if dialog::action(ui, "library-web-video-download", true, &tokens).clicked() {
                        should_submit = true;
                    }
                });
            });
        },
    );

    if close || should_cancel {
        state.web_media_modal.reset();
        state.web_media_modal.open = false;
    } else if should_submit {
        let raw_url = state.web_media_modal.url.trim().to_string();
        if !is_valid_web_url(&raw_url) {
            state.web_media_modal.error_message =
                Some(language.text("library-web-video-invalid-url").to_string());
        } else {
            state.web_media_modal.is_loading = true;
            state.web_media_modal.error_message = None;
            let title_opt = if state.web_media_modal.title.trim().is_empty() {
                None
            } else {
                Some(state.web_media_modal.title.trim().to_string())
            };
            let url_clone = raw_url.clone();
            let download = state.web_media_modal.download_offline;

            state.web_media_modal.task = Some(BackgroundTask::spawn_with_repaint(ctx, move || {
                if download {
                    let (title, path) = download_web_video(&url_clone, title_opt.as_deref())?;
                    Ok((title, path.to_string_lossy().to_string()))
                } else {
                    let title = if let Some(t) = title_opt {
                        t
                    } else {
                        fetch_web_video_title(&url_clone)?
                    };
                    Ok((title, url_clone))
                }
            }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_media_modal_defaults_and_reset() {
        let mut state = ControlPanelState::default();
        assert!(!state.web_media_modal.open);
        assert!(state.web_media_modal.url.is_empty());
        assert!(state.web_media_modal.title.is_empty());
        assert!(!state.web_media_modal.download_offline);

        state.web_media_modal.open = true;
        state.web_media_modal.url = "https://youtube.com/watch?v=123".into();
        state.web_media_modal.title = "Sample Video".into();
        state.web_media_modal.download_offline = true;
        state.web_media_modal.is_loading = true;
        state.web_media_modal.error_message = Some("Sample error".into());

        state.web_media_modal.reset();
        assert!(state.web_media_modal.url.is_empty());
        assert!(state.web_media_modal.title.is_empty());
        assert!(!state.web_media_modal.download_offline);
        assert!(!state.web_media_modal.is_loading);
        assert!(state.web_media_modal.error_message.is_none());
        assert!(state.web_media_modal.open); // reset does not toggle open
    }

    #[test]
    fn test_web_media_modal_renders_without_panicking() {
        let ctx = egui::Context::default();
        let mut state = ControlPanelState::default();
        let db = AppDb::open_in_memory().unwrap();

        let raw = egui::RawInput::default();

        // 1. Closed state
        let mut output = ctx.run_ui(raw.clone(), |ui| {
            render_web_media_modal(ui.ctx(), &mut state, Some(&db));
        });
        output.textures_delta.clear();

        // 2. Open state with content
        state.web_media_modal.open = true;
        state.web_media_modal.url = "https://www.youtube.com/watch?v=aqz-KE-bpKQ".into();
        state.web_media_modal.title = "Test".into();
        let mut output = ctx.run_ui(raw, |ui| {
            render_web_media_modal(ui.ctx(), &mut state, Some(&db));
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_web_media_database_integration() {
        let mut state = ControlPanelState::default();
        let db = AppDb::open_in_memory().unwrap();

        db.add_media_item("Web Stream", "https://youtube.com/watch?v=123", "video", "Web Videos")
            .unwrap();
        state.reload_media_items(&db);

        assert_eq!(state.media_items.len(), 1);
        assert_eq!(state.media_items[0].title, "Web Stream");
        assert_eq!(state.media_items[0].file_path, "https://youtube.com/watch?v=123");
        assert_eq!(state.media_items[0].file_type, "video");
        assert_eq!(state.media_items[0].folder, "Web Videos");
    }
}
