use crate::core::layout::ContainerKind;
use crate::core::models::{PlaylistItemKind, QuickItemKind};
use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use crate::ui::components::{
    centered_horizontal, context_menu_danger_item, context_menu_disabled_item, context_menu_header,
    context_menu_item, context_menu_separator, render_header_bar, toolbar_icon_button,
};
use crate::ui::AppTheme;
use eframe::egui::{self, RichText, Vec2};
use egui_phosphor::regular::{
    ARROW_DOWN, ARROW_UP, BOOK_OPEN, CHECK_CIRCLE, CHURCH, CIRCLE, COPY_SIMPLE, EYE, FILE_TEXT, HEART, LIST, MUSIC_NOTE,
    PENCIL_SIMPLE, PLAY, PUSH_PIN, TRASH, VIDEO,
};

pub fn render_playlist_column(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    slide_deck: &mut SlideDeck,
    is_presentation_open: &mut bool,
    _db: Option<&AppDb>,
) {
    let theme = state.theme.clone();
    let language = crate::ui::i18n::language(ui.ctx());
    let column_rect = ui.available_rect_before_wrap();

    // Barra de título com drag handle
    render_header_bar(ui, ContainerKind::Playlist, "workspace-playlist", &theme);
    ui.add_space(2.0);

    // Botões embaixo da barra de título (centralizados)
    centered_horizontal(ui, "playlist_toolbar", 160.0, |ui| {
        if toolbar_icon_button(ui, PLAY, language.text("workspace-play-selected-item"), &theme).clicked() {
            activate_selected_playlist_items(state, slide_deck, is_presentation_open, true);
        }
        let reorder = toolbar_icon_button(ui, LIST, language.text("workspace-reorder-playlist"), &theme);
        egui::Popup::menu(&reorder).show(|ui| {
            let selected = state.selected_playlist_index;
            if ui
                .add_enabled(
                    selected.is_some_and(|index| index > 0),
                    egui::Button::new(crate::ui::i18n::text(ui, "workspace-move-selected-item-up")),
                )
                .clicked()
            {
                if let Some(index) = selected
                    && index > 0
                    && index < state.playlist.len()
                {
                    state.playlist.swap(index, index - 1);
                    state.selected_playlist_index = Some(index - 1);
                }
                ui.close();
            }
            if ui
                .add_enabled(
                    selected.is_some_and(|index| index + 1 < state.playlist.len()),
                    egui::Button::new(crate::ui::i18n::text(ui, "workspace-move-selected-item-down")),
                )
                .clicked()
            {
                if let Some(index) = selected
                    && index + 1 < state.playlist.len()
                {
                    state.playlist.swap(index, index + 1);
                    state.selected_playlist_index = Some(index + 1);
                }
                ui.close();
            }
            ui.separator();
            if ui.button(crate::ui::i18n::text(ui, "workspace-sort-by-title")).clicked() {
                state
                    .playlist
                    .sort_by_key(|a| a.title.to_lowercase());
                state.selected_playlist_index = None;
                ui.close();
            }
            if ui.button(crate::ui::i18n::text(ui, "workspace-reverse-order")).clicked() {
                state.playlist.reverse();
                state.selected_playlist_index = None;
                ui.close();
            }
            ui.separator();
            if ui.button(crate::ui::i18n::text(ui, "workspace-mark-all-as-played")).clicked() {
                for item in &mut state.playlist {
                    item.played = true;
                }
                ui.close();
            }
            if ui.button(crate::ui::i18n::text(ui, "workspace-mark-all-as-not-played")).clicked() {
                for item in &mut state.playlist {
                    item.played = false;
                }
                ui.close();
            }
        });
        if toolbar_icon_button(ui, PENCIL_SIMPLE, crate::ui::i18n::text(ui, "workspace-edit-item"), &theme).clicked()
            && let Some(idx) = state.selected_playlist_index
                && idx < state.playlist.len() {
                    let item = &state.playlist[idx];
                    if item.kind == PlaylistItemKind::Song {
                        if let Some(s) = slide_deck.library().iter().find(|s| s.title == item.title)
                        {
                            state.song_editor.title = s.title.clone();
                            state.song_editor.artist = s.artist.clone();
                            state.song_editor.text = s.text.clone();
                            state.song_editor.folder = s.folder.clone();
                            state.song_editor.tags = s.tags.clone();
                            state.song_editor.open = true;
                        }
                    } else {
                        state.notify(state.settings.language.text("workspace-only-songs-can-be-edited-here"));
                    }
                }
        if toolbar_icon_button(ui, TRASH, crate::ui::i18n::text(ui, "workspace-remove-from-playlist"), &theme).clicked() {
            if !state.playlist_selection.selected.is_empty() {
                state.playlist.retain(|it| !state.playlist_selection.selected.contains(&it.id));
                state.playlist_selection.clear();
                state.selected_playlist_index = None;
            } else if let Some(idx) = state.selected_playlist_index
                && idx < state.playlist.len() {
                    state.playlist.remove(idx);
                    state.selected_playlist_index = None;
                }
        }
        if toolbar_icon_button(ui, HEART, crate::ui::i18n::text(ui, "workspace-save-playlist"), &theme).clicked() {
            state.capture_profiles();
            state.notify(state.settings.language.text("workspace-playlist-saved-in-preferences"));
        }
    });

    ui.add_space(2.0);

    let footer_h = 66.0;
    let scroll_h = (ui.available_height() - footer_h).max(40.0);

    // Playlist items list with Status checkbox: "Já foi tocada no culto?"
    egui::ScrollArea::vertical()
        .id_salt("playlist_scroll_area")
        .auto_shrink([false, false])
        .max_height(scroll_h)
        .show(ui, |ui| {
            let mut toggled_played_idx = None;
            let mut play_selection = false;
            let mut preview_selection = false;
            let mut action_move_up = None;
            let mut action_move_down = None;
            let mut action_add_to_shortcuts = None;
            let mut action_edit_song = None;
            let mut action_remove_idx = None;
            let mut action_mark_all_played = false;
            let mut action_mark_all_unplayed = false;
            let mut action_clear_playlist = false;
            let mut reorder = None;

            let order: Vec<String> = state.playlist.iter().map(|item| item.id.clone()).collect();
            state.playlist_selection.retain(&order);

            let total_items = state.playlist.len();

            for (idx, item) in state.playlist.iter().enumerate() {
                let is_selected = state.playlist_selection.selected.contains(&item.id)
                    || (state.playlist_selection.selected.is_empty() && state.selected_playlist_index == Some(idx));
                let row_w = ui.available_width();
                let (rect, resp) =
                    ui.allocate_exact_size(Vec2::new(row_w, 28.0), egui::Sense::click_and_drag());

                crate::ui::components::interaction::row_emphasis(ui, &resp, is_selected, &theme);
                if resp.drag_started() {
                    if !state.playlist_selection.selected.contains(&item.id) {
                        state.playlist_selection.clear();
                        state.playlist_selection.click(item.id.clone(), &order, false, false);
                    }
                    state.selected_playlist_index = Some(idx);
                }

                let circle_rect = egui::Rect::from_center_size(
                    egui::pos2(rect.max.x - 14.0, rect.center().y),
                    Vec2::new(20.0, 20.0),
                );
                let circle_resp = ui
                    .interact(
                        circle_rect,
                        ui.scope_id().with(("circle_btn", idx)),
                        egui::Sense::click(),
                    )
                    .on_hover_text(if item.played {
                        crate::ui::i18n::text(ui, "workspace-status-already-played-in-this-service-click-to-toggle")
                    } else {
                        crate::ui::i18n::text(ui, "workspace-status-not-played-yet-click-to-toggle")
                    });

                if circle_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if circle_resp.clicked() {
                    toggled_played_idx = Some(idx);
                }

                let (c_icon, c_color) = if item.played {
                    (CHECK_CIRCLE, theme.accent)
                } else {
                    (CIRCLE, theme.text_muted)
                };
                let circle_galley = ui.painter().layout_no_wrap(
                    c_icon.to_string(),
                    egui::FontId::proportional(16.0),
                    c_color,
                );
                ui.painter().galley(
                    egui::pos2(
                        circle_rect.center().x - circle_galley.size().x / 2.0,
                        circle_rect.center().y - circle_galley.size().y / 2.0,
                    ),
                    circle_galley,
                    c_color,
                );

                let kind_icon = match item.kind {
                    PlaylistItemKind::Song => MUSIC_NOTE,
                    PlaylistItemKind::Media => VIDEO,
                    PlaylistItemKind::Bible => BOOK_OPEN,
                    PlaylistItemKind::Text => FILE_TEXT,
                    PlaylistItemKind::Design => egui_phosphor::regular::STACK,
                };
                let drag_ids = state.playlist_selection.drag_ids(&item.id, &order);
                crate::ui::components::drag_drop::source(ui, &resp, || {
                    use crate::ui::components::drag_drop::{DragPayload, DragSource};
                    let mut payload = DragPayload::new(DragSource::Playlist(drag_ids.clone()), &item.title, &item.subtitle, kind_icon);
                    for did in &drag_ids {
                        if let Some(ditem) = state.playlist.iter().find(|it| &it.id == did) {
                            let mut quick = crate::core::models::QuickItem::from_playlist(ditem);
                            if quick.kind == QuickItemKind::Song && quick.content.is_empty() {
                                quick.content = slide_deck.library().iter().find(|song| song.title == ditem.title && song.artist == ditem.subtitle).map(|song| song.text.clone()).unwrap_or_default();
                            }
                            if quick.kind == QuickItemKind::Design { payload.design = slide_deck.designs.iter().find(|scene| scene.id == ditem.content).cloned(); }
                            payload.items.push(quick);
                        }
                    }
                    payload
                });
                if let Some((payload, after)) = crate::ui::components::drag_drop::reorder_target(ui, resp.rect, &theme, false, |source| matches!(source, crate::ui::components::drag_drop::DragSource::Playlist(ids) if !ids.contains(&item.id)))
                    && let crate::ui::components::drag_drop::DragSource::Playlist(ids) = &payload.source { reorder = Some((ids.clone(), item.id.clone(), after)); }
                let title_x = rect.min.x + 26.0;
                let max_text_x = rect.max.x - 30.0;
                let avail_w = (max_text_x - title_x).max(10.0);
                let text_clip = egui::Rect::from_min_max(
                    rect.min,
                    egui::pos2(max_text_x, rect.max.y),
                );
                let painter = ui.painter().with_clip_rect(text_clip);
                let text_y = rect.center().y;
                let icon_galley = painter.layout_no_wrap(
                    kind_icon.to_string(),
                    egui::FontId::proportional(13.0),
                    theme.icon_default,
                );
                painter.galley(
                    egui::pos2(rect.min.x + 8.0, text_y - icon_galley.size().y / 2.0),
                    icon_galley,
                    theme.icon_default,
                );

                crate::ui::components::interaction::render_item_title_and_subtitle(
                    ui,
                    &painter,
                    &item.title,
                    &item.subtitle,
                    egui::FontId::proportional(12.0),
                    egui::FontId::proportional(11.0),
                    theme.text_primary,
                    theme.text_muted,
                    egui::pos2(title_x, text_y),
                    avail_w,
                );

                let resp = if item.subtitle.is_empty() {
                    resp.on_hover_text(&item.title)
                } else {
                    resp.on_hover_text(format!("{}\n{}", item.title, item.subtitle))
                };

                if resp.hovered() && !circle_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if resp.clicked() && !circle_resp.clicked() {
                    let (shift, ctrl) = ui.input(|i| (i.modifiers.shift, i.modifiers.ctrl || i.modifiers.command));
                    state.playlist_selection.click(item.id.clone(), &order, shift, ctrl);
                    state.selected_playlist_index = Some(idx);
                    preview_selection = true;
                }
                if resp.double_clicked() {
                    let (shift, ctrl) = ui.input(|i| (i.modifiers.shift, i.modifiers.ctrl || i.modifiers.command));
                    if !shift && !ctrl && !state.playlist_selection.selected.contains(&item.id) {
                        state.playlist_selection.clear();
                        state.playlist_selection.click(item.id.clone(), &order, false, false);
                        state.selected_playlist_index = Some(idx);
                    }
                    play_selection = true;
                }

                // Menu de contexto com clique com o botão direito
                resp.context_menu(|ui| {
                    if !state.playlist_selection.selected.contains(&item.id) {
                        state.playlist_selection.clear();
                        state.playlist_selection.click(item.id.clone(), &order, false, false);
                        state.selected_playlist_index = Some(idx);
                    }
                    context_menu_header(ui);
                    if context_menu_item(ui, PLAY, "workspace-present-now", &theme).clicked() {
                        play_selection = true;
                        ui.close();
                    }
                    if context_menu_item(ui, EYE, "workspace-preview", &theme).clicked() {
                        preview_selection = true;
                        ui.close();
                    }
                    let status_label = if item.played {
                        "workspace-mark-as-not-played"
                    } else {
                        "workspace-mark-as-played"
                    };
                    let status_icon = if item.played { CIRCLE } else { CHECK_CIRCLE };
                    if context_menu_item(ui, status_icon, status_label, &theme).clicked() {
                        toggled_played_idx = Some(idx);
                        ui.close();
                    }

                    context_menu_separator(ui);

                    if idx > 0 {
                        if context_menu_item(ui, ARROW_UP, "workspace-move-up", &theme).clicked() {
                            action_move_up = Some(idx);
                            ui.close();
                        }
                    } else {
                        context_menu_disabled_item(ui, ARROW_UP, "workspace-move-up", &theme);
                    }

                    if idx + 1 < total_items {
                        if context_menu_item(ui, ARROW_DOWN, "workspace-move-down", &theme).clicked() {
                            action_move_down = Some(idx);
                            ui.close();
                        }
                    } else {
                        context_menu_disabled_item(ui, ARROW_DOWN, "workspace-move-down", &theme);
                    }

                    context_menu_separator(ui);

                    if context_menu_item(ui, PUSH_PIN, "workspace-pin-to-quick-bar", &theme).clicked() {
                        action_add_to_shortcuts = Some(idx);
                        ui.close();
                    }

                    if item.kind == PlaylistItemKind::Song
                        && context_menu_item(ui, PENCIL_SIMPLE, "workspace-edit-song", &theme).clicked()
                        {
                            action_edit_song = Some(item.title.clone());
                            ui.close();
                        }

                    context_menu_separator(ui);

                    if context_menu_danger_item(ui, TRASH, "workspace-remove-from-playlist", &theme).clicked() {
                        action_remove_idx = Some(idx);
                        ui.close();
                    }
                });

                ui.add_space(2.0);
            }

            // Menu de contexto ao clicar fora dos itens (área vazia)
            if !state.playlist.is_empty() {
                let avail_space = ui.available_size();
                if avail_space.y > 10.0 {
                    let (_, empty_resp) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), avail_space.y),
                        egui::Sense::click(),
                    );
                    empty_resp.context_menu(|ui| {
                        context_menu_header(ui);
                        if context_menu_item(ui, CHECK_CIRCLE, "workspace-mark-all-as-played", &theme)
                            .clicked()
                        {
                            action_mark_all_played = true;
                            ui.close();
                        }
                        if context_menu_item(ui, CIRCLE, "workspace-mark-all-as-not-played", &theme)
                            .clicked()
                        {
                            action_mark_all_unplayed = true;
                            ui.close();
                        }
                        context_menu_separator(ui);
                        if context_menu_danger_item(ui, TRASH, "workspace-clear-playlist", &theme)
                            .clicked()
                        {
                            action_clear_playlist = true;
                            ui.close();
                        }
                    });
                }
            } else {
                ui.add_space(28.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(CHURCH)
                            .size(32.0)
                            .color(theme.text_secondary.linear_multiply(0.6)),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(crate::ui::i18n::text(ui, "workspace-no-items-for-event"))
                            .size(12.0)
                            .strong()
                            .color(theme.text_primary),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(crate::ui::i18n::text(ui, "workspace-add-items-hint"))
                            .size(10.5)
                            .color(theme.text_secondary),
                    );
                    ui.add_space(10.0);

                    let cur_date = state.current_service_date();
                    let prev_date = state.settings.service().step_date(&cur_date, -1);
                    let prev_has_items = state
                        .settings
                        .service()
                        .get_session(&prev_date)
                        .is_some_and(|s| !s.playlist.is_empty());
                    if prev_has_items {
                        let prev_label = crate::core::state::events::format_date_display_localized(&prev_date, language);
                        let copy_prev_txt = language.format("workspace-copy-previous-with-date", &[("date", &prev_label)]);
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(format!(
                                        "{COPY_SIMPLE}  {copy_prev_txt}"
                                    ))
                                    .size(11.0)
                                    .color(theme.accent),
                                )
                                .fill(theme.bg_pill)
                                .stroke(egui::Stroke::new(1.0, theme.accent))
                                .corner_radius(6.0),
                            )
                            .clicked()
                        {
                            state.copy_items_from_previous_service();
                        }
                        ui.add_space(6.0);
                    }

                    if state.settings.services.len() > 1 {
                        let other_services: Vec<_> = state
                            .settings
                            .services
                            .iter()
                            .filter(|s| s.id != state.settings.active_service && !s.playlist.is_empty())
                            .cloned()
                            .collect();

                        if !other_services.is_empty() {
                            let copy_btn = egui::Button::new(
                                RichText::new(format!("{COPY_SIMPLE}  {}", crate::ui::i18n::text(ui, "workspace-copy-from-event")))
                                    .size(11.0)
                                    .color(theme.text_secondary),
                            )
                            .fill(theme.bg_pill)
                            .stroke(egui::Stroke::new(1.0, theme.border_default))
                            .corner_radius(6.0);

                            let copy_resp = ui.add(copy_btn);
                            egui::Popup::menu(&copy_resp).show(|ui| {
                                ui.label(
                                    RichText::new(crate::ui::i18n::text(ui, "workspace-copy-from-event"))
                                        .size(11.0)
                                        .strong()
                                        .color(theme.text_secondary),
                                );
                                ui.separator();
                                let mut copy_from_id = None;
                                for s in &other_services {
                                    let count_str = s.playlist.len().to_string();
                                    let items_txt = language.format("workspace-items-count", &[("count", &count_str)]);
                                    let label = format!("{} ({items_txt})", s.name);
                                    if ui.button(label).clicked() {
                                        copy_from_id = Some(s.id.clone());
                                        ui.close();
                                    }
                                }
                                if let Some(src_id) = copy_from_id {
                                    state.copy_items_from_event(&src_id);
                                }
                            });
                        }
                    }
                });
            }

            // Ações de reordenação e modificação da lista
            if let Some(idx) = action_move_up
                && idx > 0 && idx < state.playlist.len() {
                    state.playlist.swap(idx, idx - 1);
                    state.selected_playlist_index = Some(idx - 1);
                }
            if let Some(idx) = action_move_down
                && idx + 1 < state.playlist.len() {
                    state.playlist.swap(idx, idx + 1);
                    state.selected_playlist_index = Some(idx + 1);
                }
            if let Some(idx) = action_add_to_shortcuts
                && idx < state.playlist.len() {
                    let item = &state.playlist[idx];
                    let (qkind, content) = match item.kind {
                        PlaylistItemKind::Song => {
                            let c = slide_deck
                                .library()
                                .iter()
                                .find(|s| s.title == item.title)
                                .map(|s| s.text.clone())
                                .unwrap_or_default();
                            (QuickItemKind::Song, c)
                        }
                        PlaylistItemKind::Media => (QuickItemKind::Media, item.content.clone()),
                        PlaylistItemKind::Bible => (QuickItemKind::Verse, item.content.clone()),
                        PlaylistItemKind::Text => (QuickItemKind::Text, item.content.clone()),
                        PlaylistItemKind::Design => (QuickItemKind::Design, item.content.clone()),
                    };
                    state.add_quick_item(item.title.clone(), item.subtitle.clone(), qkind, content);
                }
            if let Some(title) = action_edit_song
                && let Some(s) = slide_deck.library().iter().find(|s| s.title == title) {
                    state.song_editor.title = s.title.clone();
                    state.song_editor.artist = s.artist.clone();
                    state.song_editor.text = s.text.clone();
                    state.song_editor.folder = s.folder.clone();
                    state.song_editor.tags = s.tags.clone();
                    state.song_editor.open = true;
                }
            if let Some(idx) = action_remove_idx {
                if idx < state.playlist.len() && state.playlist_selection.selected.contains(&state.playlist[idx].id) && state.playlist_selection.selected.len() > 1 {
                    state.playlist.retain(|it| !state.playlist_selection.selected.contains(&it.id));
                    state.playlist_selection.clear();
                    state.selected_playlist_index = None;
                } else if idx < state.playlist.len() {
                    state.playlist.remove(idx);
                    if state.selected_playlist_index == Some(idx) {
                        state.selected_playlist_index = None;
                    } else if let Some(sel) = state.selected_playlist_index
                        && sel > idx {
                            state.selected_playlist_index = Some(sel - 1);
                        }
                }
            }
            if action_mark_all_played {
                for it in &mut state.playlist {
                    it.played = true;
                }
            }
            if action_mark_all_unplayed {
                for it in &mut state.playlist {
                    it.played = false;
                }
            }
            if action_clear_playlist {
                state.playlist.clear();
                state.selected_playlist_index = None;
            }

            if ui.memory(|m| m.focused().is_none()) {
                if ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && (!state.playlist_selection.selected.is_empty() || state.selected_playlist_index.is_some())
                {
                    play_selection = true;
                }
                if ui.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::A)) {
                    state.playlist_selection.selected = order.into_iter().collect();
                    preview_selection = true;
                }
                if ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace))
                    && !state.playlist_selection.selected.is_empty()
                {
                    state.playlist.retain(|it| !state.playlist_selection.selected.contains(&it.id));
                    state.playlist_selection.clear();
                    state.selected_playlist_index = None;
                }
            }

            if let Some(idx) = toggled_played_idx {
                if let Some(target_item) = state.playlist.get(idx) {
                    let target_id = target_item.id.clone();
                    if state.playlist_selection.selected.contains(&target_id) && state.playlist_selection.selected.len() > 1 {
                        let new_played = !target_item.played;
                        for it in &mut state.playlist {
                            if state.playlist_selection.selected.contains(&it.id) {
                                it.played = new_played;
                            }
                        }
                    } else {
                        state.playlist[idx].played = !state.playlist[idx].played;
                    }
                }
            }
            if play_selection {
                activate_selected_playlist_items(state, slide_deck, is_presentation_open, true);
            } else if preview_selection {
                activate_selected_playlist_items(state, slide_deck, is_presentation_open, false);
            }
            if let Some((ids, target, after)) = reorder {
                let selected = state.selected_playlist_index.and_then(|index| state.playlist.get(index)).map(|item| item.id.clone());
                crate::core::interaction::move_items(&mut state.playlist, &ids, &target, after, |item| &item.id);
                state.selected_playlist_index = selected.and_then(|id| state.playlist.iter().position(|item| item.id == id));
            }
        });
    if !egui::DragAndDrop::payload::<crate::ui::components::drag_drop::DragPayload>(ui.ctx())
        .is_some_and(|payload| {
            matches!(
                payload.source,
                crate::ui::components::drag_drop::DragSource::Playlist(_)
            )
        })
        && let Some(payload) = crate::ui::components::drag_drop::content_target(
            ui,
            column_rect,
            &theme,
            "drag-drop-playlist",
        ) {
            if let crate::ui::components::drag_drop::DragSource::ChatMessage(id) = payload.source
                && let Some(service) = &state.chat.service
                    && let Err(error) = service.request_action(&state.chat.token, id, "playlist") {
                        state.chat.error = error;
                    }
            state.playlist.extend(
                payload
                    .items
                    .iter()
                    .map(crate::core::models::QuickItem::to_playlist),
            );
        }

    ui.add_space(2.0);
    render_event_footer(ui, state, &theme);
}

pub fn activate_selected_playlist_items(
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    presenting: &mut bool,
    project: bool,
) {
    let order: Vec<String> = state.playlist.iter().map(|item| item.id.clone()).collect();
    state.playlist_selection.retain(&order);

    let selected_count = state.playlist_selection.selected.len();
    if selected_count <= 1 {
        let index = if let Some(id) = state.playlist_selection.selected.iter().next() {
            state.playlist.iter().position(|it| &it.id == id)
        } else {
            state.selected_playlist_index
        };
        if let Some(idx) = index {
            activate_item(state, deck, presenting, idx, project);
        }
        return;
    }

    let selected_items: Vec<&crate::core::models::PlaylistItem> = state
        .playlist
        .iter()
        .filter(|item| state.playlist_selection.selected.contains(&item.id))
        .collect();

    if selected_items.is_empty() {
        return;
    }

    let mut all_slides = Vec::new();

    for item in &selected_items {
        match item.kind {
            PlaylistItemKind::Song => {
                let text = deck
                    .library()
                    .iter()
                    .find(|s| s.title == item.title && (item.subtitle.is_empty() || s.artist == item.subtitle))
                    .map(|s| s.text.as_str())
                    .unwrap_or(if !item.content.is_empty() { &item.content } else { &item.title });
                let song_slides = crate::core::slides::build_song_slides(&item.title, &item.subtitle, text);
                for slide in song_slides {
                    if !slide.trim().is_empty() {
                        all_slides.push(slide);
                    }
                }
            }
            PlaylistItemKind::Bible | PlaylistItemKind::Text => {
                let text = if !item.content.is_empty() {
                    &item.content
                } else {
                    &item.title
                };
                let parsed = crate::core::slides::parse_slides_from_text(text);
                if parsed.is_empty() {
                    all_slides.push(text.to_string());
                } else {
                    all_slides.extend(parsed);
                }
            }
            PlaylistItemKind::Design => {
                if !item.content.is_empty() {
                    let parsed = crate::core::slides::parse_slides_from_text(&item.content);
                    if parsed.is_empty() {
                        all_slides.push(item.content.clone());
                    } else {
                        all_slides.extend(parsed);
                    }
                } else {
                    all_slides.push(item.title.clone());
                }
            }
            PlaylistItemKind::Media => {
                all_slides.push(item.title.clone());
            }
        }
    }

    if all_slides.is_empty() {
        all_slides.push(String::new());
    }

    let titles_joined = selected_items
        .iter()
        .map(|it| it.title.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let combined_title = if titles_joined.len() <= 45 {
        titles_joined
    } else {
        format!("{} (+{} itens)", selected_items[0].title, selected_items.len() - 1)
    };
    let combined_artist = selected_items
        .first()
        .map(|it| it.subtitle.clone())
        .unwrap_or_default();

    deck.load_custom_slides(combined_title, combined_artist, all_slides);
    if project {
        deck.project_preview();
        if !deck
            .preview_design
            .as_ref()
            .is_some_and(|scene| scene.overlay)
        {
            state.apply_theme_to_live();
        }
        *presenting = true;
        state.show_presentation_dock = state.settings.operator().show_dock;
    }
}

fn activate_item(
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    presenting: &mut bool,
    index: usize,
    project: bool,
) {
    let Some(item) = state.playlist.get(index).cloned() else {
        return;
    };
    state.selected_playlist_index = Some(index);
    if item.kind == PlaylistItemKind::Media {
        match crate::services::media::activate(
            state,
            deck,
            &item.title,
            &item.content,
            &item.subtitle,
            project,
        ) {
            Ok(started) => {
                if started {
                    *presenting = true;
                }
            }
            Err(error) => state.notify(error),
        }
    } else if let Err(error) = item.preview(deck) {
        state.notify(error);
    } else if project {
        deck.project_preview();
        if !deck
            .preview_design
            .as_ref()
            .is_some_and(|scene| scene.overlay)
        {
            state.apply_theme_to_live();
        }
        *presenting = true;
        state.show_presentation_dock = state.settings.operator().show_dock;
    }
}

fn render_event_footer(ui: &mut egui::Ui, state: &mut ControlPanelState, theme: &AppTheme) {
    super::playlist_footer::render_event_footer(ui, state, theme);
}
