//! Sidebar panels for the song editor: connected versions and the per-word emphasis action.

use crate::core::markup;
use crate::core::models::Song;
use crate::core::state::ControlPanelState;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use egui_phosphor::regular::{LINK, MAGNIFYING_GLASS, PLUS, X};

pub(super) const TEXT_EDIT_ID: &str = "song_editor_lyrics_text";

fn label(ui: &mut egui::Ui, text: impl Into<String>, size: f32, color: Color32) {
    ui.label(RichText::new(text).size(size).color(color));
}

pub(super) fn versions_panel(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
    library: &[Song],
    db: Option<&crate::services::db::AppDb>,
) {
    let current_title = state.song_editor.title.trim().to_owned();
    let linked: Vec<Song> = library
        .iter()
        .filter(|song| {
            song.id
                .is_some_and(|id| state.song_editor.linked_versions.contains(&id))
        })
        .cloned()
        .collect();

    egui::Frame::new()
        .fill(tokens.bg_card)
        .stroke(Stroke::new(1.0, tokens.border_active))
        .corner_radius(10.0)
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.label(RichText::new(LINK).size(14.0).color(tokens.accent));
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "workspace-song-versions"))
                        .size(12.5)
                        .strong()
                        .color(tokens.text_primary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !linked.is_empty() {
                        label(
                            ui,
                            state.settings.language.count("workspace-versions-count", "workspace-versions-count", linked.len()),
                            10.5,
                            tokens.text_muted,
                        );
                    }
                });
            });
            ui.add_space(2.0);
            label(
                ui,
                crate::ui::i18n::text(ui, "workspace-link-arrangements-translations-or-recordings-of-the-same-song"),
                10.5,
                tokens.text_secondary,
            );
            ui.add_space(8.0);

            if linked.is_empty() {
                egui::Frame::new()
                    .fill(tokens.bg_pill)
                    .corner_radius(7.0)
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        label(
                            ui,
                            crate::ui::i18n::text(ui, "workspace-no-linked-versions-yet"),
                            11.0,
                            tokens.text_muted,
                        );
                    });
            } else {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(5.0, 5.0);
                    for song in &linked {
                        egui::Frame::new()
                            .fill(tokens.bg_pill)
                            .stroke(Stroke::new(1.0, tokens.border_default))
                            .corner_radius(7.0)
                            .inner_margin(egui::Margin::symmetric(9, 4))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    label(ui, song.title.clone(), 11.0, tokens.text_primary);
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new(X)
                                                    .size(10.0)
                                                    .color(tokens.text_muted),
                                            )
                                            .frame(false)
                                            .small(),
                                        )
                                        .on_hover_text(crate::ui::i18n::text(ui, "workspace-unlink-this-version"))
                                        .clicked()
                                    {
                                        unlink(state, song.id, db);
                                    }
                                });
                            });
                    }
                });
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(MAGNIFYING_GLASS)
                        .size(12.0)
                        .color(tokens.text_muted),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut state.song_editor.versions_query)
                        .id_salt("song_versions_search")
                        .hint_text(crate::ui::i18n::text(ui, "workspace-find-a-song-to-link"))
                        .text_color(tokens.text_primary)
                        .desired_width(f32::INFINITY),
                );
            });
            ui.add_space(6.0);

            let query = state.song_editor.versions_query.trim().to_lowercase();
            let mut candidates: Vec<Song> = library
                .iter()
                .filter(|song| song.title.trim() != current_title)
                .filter(|song| {
                    !song
                        .id
                        .is_some_and(|id| state.song_editor.linked_versions.contains(&id))
                })
                .filter(|song| {
                    query.is_empty()
                        || song.title.to_lowercase().contains(&query)
                        || song.artist.to_lowercase().contains(&query)
                })
                .cloned()
                .collect();
            candidates.sort_by_key(|a| a.title.to_lowercase());
            candidates.truncate(24);

            if candidates.is_empty() {
                label(
                    ui,
                    if library.len() <= 1 {
                        crate::ui::i18n::text(ui, "workspace-save-another-song-before-linking-it")
                    } else {
                        crate::ui::i18n::text(ui, "workspace-no-songs-found")
                    },
                    10.5,
                    tokens.text_muted,
                );
            } else {
                egui::ScrollArea::vertical()
                    .id_salt("song_versions_candidates")
                    .max_height(156.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for song in &candidates {
                            let mut text = format!("{PLUS}  {}", song.title);
                            if !song.artist.is_empty() {
                                text.push_str(&format!("   · {}", song.artist));
                            }
                            let button = egui::Button::new(
                                RichText::new(text).size(11.0).color(tokens.text_primary),
                            )
                            .frame(false)
                            .min_size(Vec2::new(ui.available_width(), 24.0));
                            if ui
                                .add(button)
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                            {
                                link(state, song.id, db);
                            }
                        }
                    });
            }
        });
    ui.add_space(6.0);
}

fn link(state: &mut ControlPanelState, id: Option<i64>, db: Option<&crate::services::db::AppDb>) {
    let Some(id) = id else {
        state.notify(state.settings.language.text("workspace-save-the-song-before-linking-versions"));
        return;
    };
    if !state.song_editor.linked_versions.contains(&id) {
        state.song_editor.linked_versions.push(id);
    }
    persist_links(state, db);
}

fn unlink(state: &mut ControlPanelState, id: Option<i64>, db: Option<&crate::services::db::AppDb>) {
    let Some(id) = id else { return };
    state
        .song_editor
        .linked_versions
        .retain(|linked| *linked != id);
    persist_links(state, db);
}

fn persist_links(state: &mut ControlPanelState, db: Option<&crate::services::db::AppDb>) {
    let Some(db) = db else { return };
    let Some(id) = db
        .search_songs(state.song_editor.title.trim())
        .ok()
        .and_then(|songs| {
            songs
                .into_iter()
                .find(|song| song.title == state.song_editor.title.trim())
                .and_then(|song| song.id)
        })
    else {
        return;
    };
    let _ = db.set_song_versions(id, &state.song_editor.linked_versions);
}

#[allow(dead_code)]
pub(super) fn apply_emphasis(ui: &egui::Ui, state: &mut ControlPanelState) -> bool {
    let (start, end) = state.song_editor.selection;
    let Some((text, new_start, new_end)) =
        markup::toggle_emphasis(&state.song_editor.text, start, end)
    else {
        state.notify(state.settings.language.text("workspace-select-the-word-you-want-to-highlight"));
        return false;
    };
    state.song_editor.text = text;
    state.song_editor.selection = (new_start, new_end);
    let id = egui::Id::unique(TEXT_EDIT_ID);
    if let Some(mut edit) = egui::TextEdit::load_state(ui.ctx(), id) {
        edit.cursor
            .set_char_range(Some(egui::text_selection::CCursorRange::two(
                egui::text::CCursor::new(new_start),
                egui::text::CCursor::new(new_end),
            )));
        edit.store(ui.ctx(), id);
    }
    true
}

#[allow(dead_code)]
pub(super) fn insert_break_tag(ui: &egui::Ui, state: &mut ControlPanelState, tag: &str) -> bool {
    let (start, end) = state.song_editor.selection;
    let chars: Vec<char> = state.song_editor.text.chars().collect();
    let start = start.min(chars.len());
    let end = end.min(chars.len());
    let (start, end) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };

    let offset = |idx: usize| chars[..idx].iter().map(|c| c.len_utf8()).sum::<usize>();
    let byte_start = offset(start);
    let byte_end = offset(end);

    let mut new_text = String::with_capacity(state.song_editor.text.len() + tag.len());
    new_text.push_str(&state.song_editor.text[..byte_start]);
    new_text.push_str(tag);
    new_text.push_str(&state.song_editor.text[byte_end..]);

    state.song_editor.text = new_text;
    let new_pos = start + tag.chars().count();
    state.song_editor.selection = (new_pos, new_pos);

    let id = egui::Id::unique(TEXT_EDIT_ID);
    if let Some(mut edit) = egui::TextEdit::load_state(ui.ctx(), id) {
        edit.cursor
            .set_char_range(Some(egui::text_selection::CCursorRange::one(
                egui::text::CCursor::new(new_pos),
            )));
        edit.store(ui.ctx(), id);
    }
    true
}

#[allow(dead_code)]
pub(super) fn prefix_lines(state: &mut ControlPanelState, prefix: &str) {
    state.song_editor.text = state
        .song_editor
        .text
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                line.to_owned()
            } else {
                format!("{prefix}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
}

#[allow(dead_code)]
pub(super) fn number_lines(state: &mut ControlPanelState) {
    let mut counter = 0usize;
    state.song_editor.text = state
        .song_editor
        .text
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                line.to_owned()
            } else {
                counter += 1;
                format!("{counter}. {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
}

#[allow(dead_code)]
pub(super) fn indent_lines(state: &mut ControlPanelState, indent: bool) {
    state.song_editor.text = state
        .song_editor
        .text
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                line.to_owned()
            } else if indent {
                format!("  {line}")
            } else {
                let mut trimmed = line;
                for _ in 0..2 {
                    if let Some(rest) = trimmed.strip_prefix(' ') {
                        trimmed = rest;
                    }
                }
                trimmed.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
}

#[allow(dead_code)]
pub(super) fn normalize_whitespace(state: &mut ControlPanelState) {
    let mut out = String::new();
    let mut blank_run = 0;
    for line in state.song_editor.text.lines() {
        if line.trim().is_empty() {
            blank_run += 1;
            if blank_run > 1 {
                continue;
            }
            out.push('\n');
        } else {
            blank_run = 0;
            out.push_str(line.trim_end());
            out.push('\n');
        }
    }
    state.song_editor.text = out.trim_end().to_owned();
}
