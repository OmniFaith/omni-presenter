use super::model::{LibraryEntry, LibraryKind};
use crate::{
    core::{
        models::{MediaItem, PlaylistItem},
        slides::SlideDeck,
        state::ControlPanelState,
    },
    services::db::AppDb,
};

pub(super) enum LibraryAction {
    Preview(i64),
    Present(i64),
    Edit(i64),
    Delete(Vec<i64>),
    Move(Vec<i64>, String),
    Playlist(Vec<i64>),
    Pin(Vec<i64>),
    Convert(i64),
    Info(i64),
    DeleteFolder(String),
    Create(Option<String>),
}

pub(super) fn create(
    kind: LibraryKind,
    folder: Option<&str>,
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
) {
    match kind {
        LibraryKind::Song => {
            state.song_editor = Default::default();
            state.song_editor.title = state.settings.language.text("library-new-song").into();
            state.song_editor.folder = folder.unwrap_or_default().into();
            state.song_editor.open = true;
        }
        LibraryKind::Media => import_media(state, db, folder),
        LibraryKind::Text => {
            state.text_editor_title.clear();
            state.text_editor_content.clear();
            state.editing_text_id = None;
            state.text_editor_folder = folder.unwrap_or_default().into();
            state.show_create_text_modal = true;
        }
        LibraryKind::Verse => {
            state.verse_editor_ref = format!(
                "{} {}:{}",
                state.bible_book, state.bible_chapter, state.bible_verse
            );
            state.verse_editor_version = state.bible_version.clone();
            state.verse_editor_text.clear();
            state.verse_editor_folder = folder.unwrap_or_default().into();
            state.show_save_verse_modal = true;
        }
    }
}

pub fn import_media(state: &mut ControlPanelState, db: Option<&AppDb>, folder: Option<&str>) {
    let _ = db;
    if state.media_import_task.is_some() || state.media_conversion_prompt.is_some() {
        return;
    }
    let language = state.settings.language;
    let title = language.text("library-import").to_owned();
    let media = language.text("library-media").to_owned();
    state.media_import_task = Some((
        folder.unwrap_or_default().into(),
        crate::services::tasks::BackgroundTask::spawn(move || {
            Ok(rfd::FileDialog::new()
                .set_title(title)
                .add_filter(
                    media,
                    &[
                        "png", "jpg", "jpeg", "webp", "gif", "bmp", "mp4", "mov", "webm", "mkv",
                        "avi", "pdf", "mp3", "wav", "ogg", "m4a", "flac", "aac",
                    ],
                )
                .pick_files()
                .unwrap_or_default())
        }),
    ));
}

/// Share the asynchronous import path with OS file drops, without a dialog.
pub fn import_media_paths(state: &mut ControlPanelState, mut paths: Vec<std::path::PathBuf>) {
    if state.media_import_task.is_some() || state.media_conversion_prompt.is_some() {
        state.notify(state.settings.language.text("workspace-wait-for-the-current-media-import-to-finish"));
        return;
    }
    paths.retain(|path| MediaItem::detect_type(&path.to_string_lossy()) != "other");
    if paths.is_empty() {
        state.notify(state.settings.language.text("workspace-no-supported-media-files-were-found"));
        return;
    }
    let existing: std::collections::HashSet<String> = state
        .media_items
        .iter()
        .map(|item| item.file_path.clone())
        .collect();
    state.media_import_task = Some((
        String::new(),
        crate::services::tasks::BackgroundTask::spawn(move || {
            let mut unique = std::collections::HashSet::new();
            let mut imports = Vec::new();
            for path in paths {
                if MediaItem::detect_type(&path.to_string_lossy()) == "other" || !path.is_file() {
                    continue;
                }
                let path = path.canonicalize().map_err(|error| error.to_string())?;
                if !existing.contains(path.to_string_lossy().as_ref())
                    && unique.insert(path.clone())
                {
                    imports.push(path);
                }
            }
            Ok(imports)
        }),
    ));
    state.selected_tab = 1;
    state.active_sidebar_page = crate::core::state::SidebarPage::Home;
    state.search_query.clear();
}

pub fn poll_media_import(
    ctx: &eframe::egui::Context,
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
) {
    let Some((_, task)) = &state.media_import_task else {
        return;
    };
    ctx.request_repaint_after(std::time::Duration::from_millis(33));
    let Some(result) = task.take() else {
        return;
    };
    let (folder, _) = state.media_import_task.take().unwrap();
    let converting = std::mem::take(&mut state.media_conversion_running);
    let paths = match result {
        Ok(paths) => paths,
        Err(error) => {
            if converting {
                state.media_conversion_progress = None;
                state.media_conversion_prompt = state.media_conversion_retry.take();
                state.notify(state.settings.language.format("library-conversion-error", &[("error", &error)]));
            } else {
                state.notify(error);
            }
            return;
        }
    };
    if converting {
        state.media_conversion_retry = None;
        state.media_conversion_progress = None;
    }
    if paths.is_empty() {
        return;
    }
    if !converting && state.settings.ask_media_conversion
        && paths.iter().any(|path| crate::services::media_conversion::convertible(path))
    {
        state.media_conversion_prompt = Some(crate::services::media_conversion::Prompt::new(paths, folder, true));
        return;
    }
    finish_import(state, db, paths, &folder);
}

pub(super) fn finish_import(state: &mut ControlPanelState, db: Option<&AppDb>, paths: Vec<std::path::PathBuf>, folder: &str) {
    let language = state.settings.language;
    for path in paths {
        let title = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or_else(|| language.text("workspace-media"));
        let path = path.to_string_lossy();
        let file_type = MediaItem::detect_type(&path);
        if let Some(db) = db {
            if let Err(error) = db.add_media_item(title, &path, file_type, folder) {
                state.notify(
                    language.format("library-import-error", &[("error", &error.to_string())]),
                );
                return;
            }
        } else {
            let id = state
                .media_items
                .iter()
                .filter_map(|item| item.id)
                .min()
                .unwrap_or(0)
                .min(0)
                - 1;
            state.media_items.push(MediaItem {
                id: Some(id),
                title: title.into(),
                file_path: path.into(),
                file_type: file_type.into(),
                folder: folder.into(),
            });
        }
    }
    if let Some(db) = db {
        state.reload_media_items(db);
    }
    if !folder.is_empty() {
        state.open_media_folders.insert(folder.into());
    }
    state.notify(language.text("common-done"));
}

pub(super) fn execute(
    action: LibraryAction,
    kind: LibraryKind,
    entries: &[LibraryEntry],
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    presenting: &mut bool,
    db: Option<&AppDb>,
) {
    let result = execute_inner(action, kind, entries, state, deck, presenting, db);
    if let Err(error) = result {
        state.notify(error.to_string());
    }
}

fn execute_inner(
    action: LibraryAction,
    kind: LibraryKind,
    entries: &[LibraryEntry],
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    presenting: &mut bool,
    db: Option<&AppDb>,
) -> rusqlite::Result<()> {
    match action {
        LibraryAction::Create(folder) => create(kind, folder.as_deref(), state, db),
        LibraryAction::Preview(id) | LibraryAction::Present(id) => {
            let project = matches!(action, LibraryAction::Present(_));
            let Some(item) = entries.iter().find(|item| item.id == id) else {
                return Ok(());
            };
            match kind {
                LibraryKind::Song => {
                    if project {
                        deck.project_song(item.index);
                    } else {
                        deck.preview_song(item.index);
                    }
                    deck.select_preview_slide(0);
                }
                LibraryKind::Media => {
                    match crate::services::media::activate(
                        state,
                        deck,
                        &item.title,
                        &item.content,
                        "",
                        project,
                    ) {
                        Ok(live) => {
                            if live {
                                *presenting = true;
                            }
                        }
                        Err(error) => state.notify(error),
                    }
                    return Ok(());
                }
                _ => {
                    if project {
                        deck.project_custom_content(
                            item.title.clone(),
                            item.subtitle.clone(),
                            item.content.clone(),
                        );
                    } else {
                        deck.load_custom_content(
                            item.title.clone(),
                            item.subtitle.clone(),
                            item.content.clone(),
                        );
                    }
                }
            }
            if project {
                *presenting = true;
                state.apply_theme_to_live();
                state.show_presentation_dock = state.settings.operator().show_dock;
            }
        }
        LibraryAction::Edit(id) => {
            if let Some(item) = entries.iter().find(|item| item.id == id) {
                match kind {
                    LibraryKind::Song => {
                        state.song_editor = Default::default();
                        state.song_editor.title = item.title.clone();
                        state.song_editor.artist = item.subtitle.clone();
                        state.song_editor.text = item.content.clone();
                        state.song_editor.tags = item.tags.clone();
                        state.song_editor.folder = item.folder.clone();
                        state.song_editor.open = true;
                    }
                    LibraryKind::Text => {
                        state.editing_text_id = Some(id);
                        state.text_editor_title = item.title.clone();
                        state.text_editor_content = item.content.clone();
                        state.text_editor_folder = item.folder.clone();
                        state.show_create_text_modal = true;
                    }
                    _ => {}
                }
            }
        }
        LibraryAction::Playlist(ids) => append_items(kind, entries, &ids, state, false),
        LibraryAction::Pin(ids) => append_items(kind, entries, &ids, state, true),
        LibraryAction::Convert(id) => {
            if state.media_import_task.is_some() || state.media_conversion_prompt.is_some() {
                state.notify(state.settings.language.text("workspace-wait-for-the-current-media-import-to-finish"));
            } else if let Some(item) = entries.iter().find(|item| item.id == id) {
                let path = std::path::PathBuf::from(&item.content);
                if crate::services::media_conversion::convertible(&path) {
                    state.media_conversion_prompt = Some(crate::services::media_conversion::Prompt::new(vec![path], item.folder.clone(), false));
                } else {
                    state.notify(state.settings.language.text("library-conversion-unsupported"));
                }
            }
        }
        LibraryAction::Info(id) => {
            if let Some(item) = entries.iter().find(|item| item.id == id) {
                let details = crate::services::media_info::probe_media_details(&item.content);
                state.media_info_dialog = Some(details);
            }
        }
        LibraryAction::Delete(ids) => {
            for id in &ids {
                if let Some(db) = db {
                    match kind {
                        LibraryKind::Song => db.delete_song(*id)?,
                        LibraryKind::Media => db.delete_media_item(*id)?,
                        LibraryKind::Text => db.delete_custom_text(*id)?,
                        LibraryKind::Verse => db.delete_saved_verse(*id)?,
                    }
                } else {
                    match kind {
                        LibraryKind::Song => {}
                        LibraryKind::Media => state.media_items.retain(|item| item.id != Some(*id)),
                        LibraryKind::Text => state.custom_texts.retain(|item| item.id != Some(*id)),
                        LibraryKind::Verse => {
                            state.saved_verses.retain(|item| item.id != Some(*id))
                        }
                    }
                }
            }
            kind.clear(state);
            reload(kind, state, deck, db);
        }
        LibraryAction::Move(ids, folder) => {
            for id in ids {
                if let Some(db) = db {
                    if id > 0 {
                        match kind {
                            LibraryKind::Song => db.move_song_to_folder(id, &folder)?,
                            LibraryKind::Media => db.move_media_to_folder(id, &folder)?,
                            LibraryKind::Text => db.move_text_to_folder(id, &folder)?,
                            LibraryKind::Verse => db.move_verse_to_folder(id, &folder)?,
                        }
                    } else {
                        move_in_memory(kind, state, deck, id, &folder);
                    }
                } else {
                    move_in_memory(kind, state, deck, id, &folder);
                }
            }
            if !folder.is_empty() {
                kind.folders_mut(state).insert(folder);
            }
            reload(kind, state, deck, db);
        }
        LibraryAction::DeleteFolder(folder) => {
            if let Some(db) = db {
                if kind == LibraryKind::Song {
                    db.delete_folder(&folder)?;
                } else {
                    db.delete_folder_from_category(kind.category(), &folder)?;
                }
                reload(kind, state, deck, Some(db));
            } else {
                for item in entries.iter().filter(|item| item.folder == folder) {
                    move_in_memory(kind, state, deck, item.id, "");
                }
            }
            kind.folders_mut(state).remove(&folder);
        }
    }
    Ok(())
}

pub(super) fn append_items(
    kind: LibraryKind,
    entries: &[LibraryEntry],
    ids: &[i64],
    state: &mut ControlPanelState,
    pin: bool,
) {
    for item in entries.iter().filter(|item| ids.contains(&item.id)) {
        if pin {
            state.add_quick_item(
                item.title.clone(),
                item.subtitle.clone(),
                kind.quick_kind(),
                item.content.clone(),
            );
        } else {
            state.playlist.push(PlaylistItem {
                id: crate::core::preferences::new_id(kind.category()),
                title: item.title.clone(),
                subtitle: item.subtitle.clone(),
                content: item.content.clone(),
                kind: kind.playlist_kind(),
                played: false,
            });
        }
    }
}

fn reload(
    kind: LibraryKind,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    db: Option<&AppDb>,
) {
    if let Some(db) = db {
        match kind {
            LibraryKind::Song => deck.reload_from_db(db),
            LibraryKind::Media => state.reload_media_items(db),
            LibraryKind::Text => state.reload_custom_texts(db),
            LibraryKind::Verse => state.reload_saved_verses(db),
        }
    }
}

fn move_in_memory(
    kind: LibraryKind,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    id: i64,
    folder: &str,
) {
    match kind {
        LibraryKind::Song => {
            if id > 0 {
                if let Some(song) = deck.library_mut().iter_mut().find(|s| s.id == Some(id)) {
                    song.folder = folder.into();
                }
            } else {
                let idx = (-id - 1) as usize;
                if idx < deck.library().len() {
                    deck.library_mut()[idx].folder = folder.into();
                }
            }
        }
        LibraryKind::Media => {
            if id > 0 {
                if let Some(item) = state
                    .media_items
                    .iter_mut()
                    .find(|item| item.id == Some(id))
                {
                    item.folder = folder.into();
                }
            } else {
                let idx = (-id - 1) as usize;
                if idx < state.media_items.len() {
                    state.media_items[idx].folder = folder.into();
                }
            }
        }
        LibraryKind::Text => {
            if id > 0 {
                if let Some(item) = state
                    .custom_texts
                    .iter_mut()
                    .find(|item| item.id == Some(id))
                {
                    item.folder = folder.into();
                }
            } else {
                let idx = (-id - 1) as usize;
                if idx < state.custom_texts.len() {
                    state.custom_texts[idx].folder = folder.into();
                }
            }
        }
        LibraryKind::Verse => {
            if id > 0 {
                if let Some(item) = state
                    .saved_verses
                    .iter_mut()
                    .find(|item| item.id == Some(id))
                {
                    item.folder = folder.into();
                }
            } else {
                let idx = (-id - 1) as usize;
                if idx < state.saved_verses.len() {
                    state.saved_verses[idx].folder = folder.into();
                }
            }
        }
    }
}
