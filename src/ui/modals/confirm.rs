use crate::{
    core::{
        slides::SlideDeck,
        state::{ConfirmAction, ControlPanelState},
    },
    services::db::AppDb,
};
use eframe::egui::{self, RichText};
use egui_phosphor::regular::{TRASH, X};

pub fn render_confirm_dialog(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    db: Option<&AppDb>,
) {
    let Some(dialog) = state.confirm_dialog.clone() else {
        return;
    };

    let language = state.settings.language;
    let tokens = &state.theme;
    let mut confirmed = false;
    let mut closed = false;

    let frame = egui::Frame::new()
        .fill(tokens.bg_panel)
        .stroke(egui::Stroke::new(1.0, tokens.border_default))
        .corner_radius(14.0)
        .inner_margin(20)
        .shadow(egui::epaint::Shadow {
            offset: [0, 12],
            blur: 40,
            spread: 0,
            color: egui::Color32::from_black_alpha(70),
        });

    let modal = egui::Modal::new(egui::Id::unique("app_confirm_modal"))
        .frame(frame)
        .backdrop_color(egui::Color32::from_black_alpha(110));

    let response = modal.show(ctx, |ui| {
        ui.set_width(400.0_f32.min((ctx.content_rect().width() - 48.0).max(220.0)));
        ui.spacing_mut().item_spacing = egui::vec2(8.0, 12.0);

        // Header: icon, title, close button
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(TRASH)
                    .size(18.0)
                    .color(tokens.status_danger),
            );
            ui.label(
                RichText::new(&dialog.title)
                    .size(17.0)
                    .strong()
                    .color(tokens.text_primary),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if tokens
                    .icon_button(
                        ui,
                        X,
                        language.text("common-close"),
                        false,
                        egui::Vec2::splat(26.0),
                    )
                    .clicked()
                {
                    closed = true;
                }
            });
        });

        ui.add_space(4.0);

        // Message
        ui.label(
            RichText::new(&dialog.message)
                .size(13.5)
                .color(tokens.text_secondary),
        );

        ui.add_space(8.0);

        // Buttons
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let delete_label = dialog
                    .confirm_label
                    .as_deref()
                    .unwrap_or_else(|| language.text("common-confirm-delete"));

                let delete_btn = ui.add(
                    egui::Button::new(
                        RichText::new(delete_label)
                            .strong()
                            .color(egui::Color32::WHITE),
                    )
                    .fill(tokens.status_danger)
                    .stroke(egui::Stroke::NONE)
                    .corner_radius(8.0)
                    .min_size(egui::Vec2::new(88.0, 32.0)),
                );

                if delete_btn.clicked() {
                    confirmed = true;
                }

                ui.add_space(8.0);

                let cancel_btn = ui.add(
                    egui::Button::new(
                        RichText::new(language.text("common-cancel"))
                            .color(tokens.text_primary),
                    )
                    .fill(tokens.bg_pill)
                    .stroke(egui::Stroke::new(1.0, tokens.border_default))
                    .corner_radius(8.0)
                    .min_size(egui::Vec2::new(80.0, 32.0)),
                );

                if cancel_btn.clicked() {
                    closed = true;
                }
            });
        });
    });

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        closed = true;
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
        confirmed = true;
    }

    if confirmed {
        execute_confirm_action(&dialog.action, state, deck, db);
        state.confirm_dialog = None;
    } else if closed || response.should_close() {
        state.confirm_dialog = None;
    }
}

pub fn execute_confirm_action(
    action: &ConfirmAction,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    db: Option<&AppDb>,
) {
    match action {
        ConfirmAction::DeleteTheme { id, .. } => {
            if state.themes.len() > 1 {
                if let Some(idx) = state.themes.iter().position(|t| &t.id == id) {
                    state.themes.remove(idx);
                    if let Some(db) = db {
                        let _ = db.delete_theme(id);
                    }
                    if state.selected_theme_index >= state.themes.len() {
                        state.selected_theme_index = state.themes.len().saturating_sub(1);
                    }
                }
            }
        }
        ConfirmAction::DeleteSongs { ids } => {
            if let Some(db) = db {
                for id in ids {
                    let _ = db.delete_song(*id);
                }
                deck.reload_from_db(db);
            }
            state.clear_song_selection();
        }
        ConfirmAction::DeleteMedia { ids } => {
            if let Some(db) = db {
                for id in ids {
                    let _ = db.delete_media_item(*id);
                }
                state.reload_media_items(db);
            } else {
                state.media_items.retain(|item| !item.id.is_some_and(|id| ids.contains(&id)));
            }
            state.clear_media_selection();
        }
        ConfirmAction::DeleteTexts { ids } => {
            if let Some(db) = db {
                for id in ids {
                    let _ = db.delete_custom_text(*id);
                }
                state.reload_custom_texts(db);
            } else {
                state.custom_texts.retain(|item| !item.id.is_some_and(|id| ids.contains(&id)));
            }
            state.clear_text_selection();
        }
        ConfirmAction::DeleteSavedVerses { ids } => {
            if let Some(db) = db {
                for id in ids {
                    let _ = db.delete_saved_verse(*id);
                }
                state.reload_saved_verses(db);
            } else {
                state.saved_verses.retain(|item| !item.id.is_some_and(|id| ids.contains(&id)));
            }
            state.clear_verse_selection();
        }
        ConfirmAction::DeleteFolder { category, folder } => {
            if let Some(db) = db {
                if category == "song" {
                    let _ = db.delete_folder(folder);
                    deck.reload_from_db(db);
                } else {
                    let _ = db.delete_folder_from_category(category, folder);
                    match category.as_str() {
                        "media" => state.reload_media_items(db),
                        "text" => state.reload_custom_texts(db),
                        "bible" => state.reload_saved_verses(db),
                        _ => {}
                    }
                }
            }
            match category.as_str() {
                "song" => {
                    state.open_folders.remove(folder);
                }
                "media" => {
                    state.open_media_folders.remove(folder);
                }
                "text" => {
                    state.open_text_folders.remove(folder);
                }
                "bible" => {
                    state.open_verse_folders.remove(folder);
                }
                _ => {}
            }
        }
        ConfirmAction::ClearPlaylist => {
            state.playlist.clear();
            state.playlist_selection.clear();
            state.selected_playlist_index = None;
        }
        ConfirmAction::DeletePlaylistItems { indices } => {
            let mut to_remove = indices.clone();
            to_remove.sort_unstable_by(|a, b| b.cmp(a));
            for idx in to_remove {
                if idx < state.playlist.len() {
                    state.playlist.remove(idx);
                }
            }
            state.playlist_selection.clear();
            state.selected_playlist_index = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{CustomTextItem, MediaItem, PlaylistItem, PresentationTheme};

    #[test]
    fn test_execute_confirm_delete_theme() {
        let mut state = ControlPanelState::default();
        let mut deck = SlideDeck::default();
        let custom_theme = PresentationTheme {
            id: "custom_theme_1".into(),
            name: "Meu Tema".into(),
            ..Default::default()
        };
        state.themes.push(custom_theme);
        assert!(state.themes.iter().any(|t| t.id == "custom_theme_1"));

        execute_confirm_action(
            &ConfirmAction::DeleteTheme {
                id: "custom_theme_1".into(),
                name: "Meu Tema".into(),
            },
            &mut state,
            &mut deck,
            None,
        );

        assert!(!state.themes.iter().any(|t| t.id == "custom_theme_1"));
    }

    #[test]
    fn test_execute_confirm_delete_media_and_text() {
        let mut state = ControlPanelState::default();
        let mut deck = SlideDeck::default();

        let mut item = MediaItem::new("Background Video", "/path/to/video.mp4", "");
        item.id = Some(42);
        state.media_items.push(item);
        state.custom_texts.push(CustomTextItem {
            id: Some(99),
            title: "Avisos".into(),
            content: "Texto dos avisos".into(),
            folder: String::new(),
        });

        execute_confirm_action(
            &ConfirmAction::DeleteMedia { ids: vec![42] },
            &mut state,
            &mut deck,
            None,
        );
        assert!(state.media_items.is_empty());

        execute_confirm_action(
            &ConfirmAction::DeleteTexts { ids: vec![99] },
            &mut state,
            &mut deck,
            None,
        );
        assert!(state.custom_texts.is_empty());
    }

    #[test]
    fn test_execute_confirm_clear_playlist() {
        let mut state = ControlPanelState::default();
        let mut deck = SlideDeck::default();

        state.playlist.push(PlaylistItem {
            id: "item1".into(),
            title: "Musica 1".into(),
            subtitle: "Artista".into(),
            content: "Letra".into(),
            kind: crate::core::models::PlaylistItemKind::Song,
            played: false,
        });
        state.selected_playlist_index = Some(0);

        execute_confirm_action(&ConfirmAction::ClearPlaylist, &mut state, &mut deck, None);

        assert!(state.playlist.is_empty());
        assert_eq!(state.selected_playlist_index, None);
    }
}
