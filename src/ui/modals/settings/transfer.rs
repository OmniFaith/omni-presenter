use crate::{
    core::{models::Song, preferences::AppSettings, state::ControlPanelState},
    services::{db::AppDb, tasks::BackgroundTask, transfer::Package},
};
use eframe::egui;
use std::path::PathBuf;

#[path = "transfer_cards.rs"]
mod cards;

const ID: &str = "workspace_transfer_ui";
enum ResultData {
    Export(Package),
    Inspect(PathBuf, Package),
    Lyrics(Vec<Song>),
    Added,
    Example,
}
#[derive(Clone, Default)]
struct TransferUi {
    task: Option<BackgroundTask<Option<ResultData>>>,
    package: Option<(PathBuf, Package)>,
    songs: Vec<(bool, Song)>,
    confirmed: bool,
    status: String,
    error: bool,
}
#[derive(Clone, Default)]
pub struct RestoreRequest {
    pub source: PathBuf,
}

fn get(ctx: &egui::Context) -> TransferUi {
    // Move large previews out instead of cloning potentially megabytes every frame.
    ctx.data_mut(|d| d.remove_temp(egui::Id::unique(ID)))
        .unwrap_or_default()
}
fn put(ctx: &egui::Context, value: TransferUi) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::unique(ID), value));
}

pub fn take_restore(ctx: &egui::Context) -> Option<RestoreRequest> {
    ctx.data_mut(|d| d.remove_temp(egui::Id::unique("workspace_restore_request")))
}

/// Always polled by the application, even if Settings has been closed.
pub fn poll(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    deck: &mut crate::core::slides::SlideDeck,
    db: &AppDb,
) {
    let mut data = get(ctx);
    if let Some(task) = &data.task {
        if let Some(result) = task.take() {
            data.task = None;
            data.error = false;
            match result {
                Ok(Some(ResultData::Export(package))) => {
                    data.status = state.settings.language.text("transfer-exported").into();
                    if !package.missing.is_empty() {
                        data.status.push_str(&format!(
                            "\n{}: {}",
                            state.settings.language.text("transfer-missing"),
                            package.missing.join(", ")
                        ));
                    }
                }
                Ok(Some(ResultData::Inspect(path, package))) => {
                    data.package = Some((path, package));
                    data.confirmed = false;
                }
                Ok(Some(ResultData::Lyrics(songs))) => {
                    data.songs = songs.into_iter().map(|s| (true, s)).collect();
                }
                Ok(Some(ResultData::Added)) => {
                    deck.reload_from_db(db);
                    state.reload_all_selection_items(db);
                    data.songs.clear();
                    data.status = state.settings.language.text("transfer-lyrics-added").into();
                }
                Ok(Some(ResultData::Example)) => {
                    data.status = state.settings.language.text("transfer-exported").into();
                }
                Ok(None) => {}
                Err(error) => {
                    data.error = true;
                    data.status = state.settings.language.text(&error).to_owned();
                }
            }
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
    put(ctx, data);
}

pub fn start_lyrics(ctx: &egui::Context) {
    let mut data = get(ctx);
    if data.task.is_some() {
        put(ctx, data);
        return;
    }
    data.status.clear();
    let language = crate::ui::i18n::language(ctx);
    data.task = Some(BackgroundTask::spawn_with_repaint(ctx, move || {
        let Some(paths) = rfd::FileDialog::new()
            .add_filter(
                "Lyrics",
                &[
                    "txt", "text", "md", "markdown", "lrc", "srt", "vtt", "pptx", "docx", "odt",
                    "odp",
                ],
            )
            .add_filter("All files", &["*"])
            .pick_files()
        else {
            return Ok(None);
        };
        if paths.len() > 500 {
            return Err("lyrics-too-large".into());
        }
        let mut songs = Vec::new();
        let mut total = 0;
        for path in paths {
            let song = crate::services::lyrics_import::read(&path)
                .map_err(|e| format!("{}: {}", path.display(), language.text(&e)))?;
            total += song.text.len();
            if total > 64 * 1024 * 1024 {
                return Err("lyrics-too-large".into());
            }
            songs.push(song);
        }
        Ok(Some(ResultData::Lyrics(songs)))
    }));
    put(ctx, data);
}

pub fn render(ui: &mut egui::Ui, draft: &AppSettings, db: Option<&AppDb>) {
    let language = draft.language;
    let Some(db) = db else {
        return;
    };
    let mut data = get(ui.ctx());
    let (theme, _) = crate::ui::theme::AppTheme::resolve(draft, ui.ctx());
    let busy = data.task.is_some();
    ui.add_enabled_ui(!busy, |ui| {
        let actions = cards::show(ui, language, &theme);
        if actions.export {
            let db = db.clone();
            let settings = draft.clone();
            data.status.clear();
            data.task = Some(BackgroundTask::spawn_with_repaint(ui.ctx(), move || {
                let Some(path) = rfd::FileDialog::new()
                    .add_filter("Omni workspace ZIP", &["zip"])
                    .set_file_name("omni-presenter-backup.zip")
                    .save_file()
                else {
                    return Ok(None);
                };
                crate::services::transfer::export(&db, &settings, &path)
                    .map(|p| Some(ResultData::Export(p)))
            }));
        }
        if actions.import {
            let db = db.clone();
            data.status.clear();
            data.task = Some(BackgroundTask::spawn_with_repaint(ui.ctx(), move || {
                let Some(path) = rfd::FileDialog::new()
                    .add_filter("Omni workspace ZIP", &["zip"])
                    .pick_file()
                else {
                    return Ok(None);
                };
                let package = crate::services::transfer::inspect(&path, &db)?;
                Ok(Some(ResultData::Inspect(path, package)))
            }));
        }
        if let Some((path, package)) = &data.package {
            ui.separator();
            ui.label(
                egui::RichText::new(path.file_name().unwrap_or_default().to_string_lossy())
                    .strong(),
            )
            .on_hover_text(path.display().to_string());
            egui::CollapsingHeader::new(language.text("transfer-package-details"))
                .id_salt("transfer_package_details")
                .show(ui, |ui| {
                    ui.label(format!("Omni Presenter {}", package.application_version));
                    for table in &package.tables {
                        ui.label(format!("{}: {}", table.name, table.rows.len()));
                    }
                });
            ui.label(format!(
                "{}: {}",
                language.text("transfer-assets"),
                package.assets.len()
            ));
            if !package.missing.is_empty() {
                ui.colored_label(
                    theme.status_warning_fg,
                    format!(
                        "{}: {}",
                        language.text("transfer-missing"),
                        package.missing.join(", ")
                    ),
                );
            }
            ui.colored_label(
                theme.status_warning_fg,
                language.text("transfer-restore-warning"),
            );
            ui.checkbox(&mut data.confirmed, language.text("transfer-confirm"));
            let mut cancel = false;
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled(
                        data.confirmed,
                        egui::Button::new(language.text("transfer-restore")),
                    )
                    .clicked()
                {
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(
                            egui::Id::unique("workspace_restore_request"),
                            RestoreRequest {
                                source: path.clone(),
                            },
                        )
                    });
                    cancel = true;
                }
                if ui.button(language.text("common-cancel")).clicked() {
                    cancel = true;
                }
            });
            if cancel {
                data.package = None;
                data.confirmed = false;
            }
        }
        if actions.lyrics {
            put(ui.ctx(), std::mem::take(&mut data));
            start_lyrics(ui.ctx());
            data = get(ui.ctx());
        }
        if actions.example {
            data.task = Some(BackgroundTask::spawn_with_repaint(ui.ctx(), || {
                let Some(path) = rfd::FileDialog::new()
                    .set_file_name("exemplo-importacao-letras.txt")
                    .save_file()
                else {
                    return Ok(None);
                };
                std::fs::write(path, crate::services::lyrics_import::EXAMPLE)
                    .map_err(|e| e.to_string())?;
                Ok(Some(ResultData::Example))
            }));
        }
        if !data.songs.is_empty() {
            ui.label(language.text("transfer-lyrics-preview"));
            for (i, (selected, song)) in data.songs.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(selected, "");
                        ui.text_edit_singleline(&mut song.title);
                    });
                    egui::CollapsingHeader::new(language.text("transfer-lyrics-edit")).show(
                        ui,
                        |ui| {
                            ui.label(language.text("transfer-artist"));
                            ui.text_edit_singleline(&mut song.artist);
                            ui.label(language.text("transfer-folder"));
                            ui.text_edit_singleline(&mut song.folder);
                            ui.label(language.text("transfer-tags"));
                            ui.text_edit_singleline(&mut song.tags);
                            ui.add(
                                egui::TextEdit::multiline(&mut song.text)
                                    .desired_rows(8)
                                    .desired_width(f32::INFINITY),
                            );
                        },
                    );
                });
            }
            let valid = data.songs.iter().any(|(selected, _)| *selected)
                && data
                    .songs
                    .iter()
                    .filter(|(selected, _)| *selected)
                    .all(|(_, s)| !s.title.trim().is_empty() && !s.text.trim().is_empty());
            if ui
                .add_enabled(
                    valid,
                    egui::Button::new(language.text("transfer-lyrics-add")),
                )
                .clicked()
            {
                let db = db.clone();
                let songs: Vec<_> = data
                    .songs
                    .iter()
                    .filter(|(selected, _)| *selected)
                    .map(|(_, s)| s.clone())
                    .collect();
                data.task = Some(BackgroundTask::spawn_with_repaint(ui.ctx(), move || {
                    db.import_lyrics(&songs)?;
                    Ok(Some(ResultData::Added))
                }));
            }
            if ui.button(language.text("common-cancel")).clicked() {
                data.songs.clear();
            }
        }
    });
    cards::details(ui, language, &theme);
    if data.task.is_some() {
        ui.spinner();
        ui.label(language.text("transfer-working"));
    }
    if !data.status.is_empty() {
        if data.error {
            ui.colored_label(theme.status_danger, &data.status);
        } else {
            ui.label(&data.status);
        }
    }
    put(ui.ctx(), data);
}

#[cfg(test)]
#[path = "transfer_tests.rs"]
mod tests;
