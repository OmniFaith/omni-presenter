mod actions;
mod conversion;
pub(crate) mod file_drop;
mod model;
#[cfg(test)]
mod tests;
mod toolbar;

use crate::{
    core::{bible::fold, slides::SlideDeck, state::ControlPanelState},
    services::db::AppDb,
    ui::components::*,
};
use actions::LibraryAction;
pub use actions::import_media;
pub use actions::poll_media_import;
pub use conversion::render_media_conversion;
use eframe::egui;
use egui_phosphor::regular::*;
use model::{LibraryEntry, LibraryKind};

pub fn render_selection_column(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    presenting: &mut bool,
    db: Option<&AppDb>,
) {
    file_drop::handle(ui, state, ui.available_rect_before_wrap());
    toolbar::toolbar(ui, state, deck, db);
    if state.selected_tab == 4 {
        crate::ui::pages::design::library::render_selection(ui, state, deck, presenting, db);
        return;
    }
    let kind = LibraryKind::from_tab(state.selected_tab);
    let entries = model::entries(kind, state, deck);
    let folders = collect_folders(kind, state, &entries, db);
    let query = fold(state.search_query.trim());
    let filtered: Vec<_> = entries
        .iter()
        .filter(|entry| {
            (kind != LibraryKind::Song
                || state.tag_filter.is_empty()
                || entry
                    .tags
                    .split(',')
                    .any(|tag| tag.trim().eq_ignore_ascii_case(&state.tag_filter)))
                && query.split_whitespace().all(|token| {
                    fold(&format!(
                        "{} {} {}",
                        entry.title, entry.subtitle, entry.content
                    ))
                    .contains(token)
                })
        })
        .collect();
    let mut visible = Vec::new();
    for folder in &folders {
        if kind.folders(state).contains(folder) || !query.is_empty() {
            visible.extend(
                filtered
                    .iter()
                    .filter(|item| &item.folder == folder)
                    .map(|item| item.id),
            );
        }
    }
    visible.extend(
        filtered
            .iter()
            .filter(|item| item.folder.is_empty())
            .map(|item| item.id),
    );
    let mut action = None;
    if ui.is_enabled()
        && !ui.ctx().egui_wants_keyboard_input()
        && !egui::DragAndDrop::has_any_payload(ui.ctx())
        && ui.ctx().memory(|memory| memory.top_modal_layer().is_none())
    {
        if ui.input(|input| {
            (input.modifiers.ctrl || input.modifiers.command) && input.key_pressed(egui::Key::A)
        }) {
            *kind.selected_mut(state) = visible.iter().copied().collect();
            *kind.anchor(state) = visible.first().copied();
        }
        if !kind.selected(state).is_empty()
            && ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            kind.clear(state);
        }
        if ui.input(|input| input.key_pressed(egui::Key::Delete))
            && !kind.selected(state).is_empty()
        {
            action = Some(LibraryAction::Delete(
                kind.selected(state).iter().copied().collect(),
            ));
        }
    }
    selection_bar(ui, state, kind, &folders, &mut action);
    let theme = state.theme.clone();
    let language = state.settings.language;
    egui::ScrollArea::vertical()
        .id_salt("selection_scroll_area")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for folder in &folders {
                let items: Vec<_> = filtered
                    .iter()
                    .filter(|item| &item.folder == folder)
                    .copied()
                    .collect();
                if !query.is_empty() && items.is_empty() {
                    continue;
                }
                let open = kind.folders(state).contains(folder) || !query.is_empty();
                let folder_id =
                    ui.make_persistent_id(("library_folder_btn", kind.category(), folder));
                let (f_rect, response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 28.0),
                    egui::Sense::click(),
                );

                let pointer_pos = ui.ctx().pointer_latest_pos().or_else(|| ui.ctx().pointer_interact_pos());
                let is_folder_hovered =
                    pointer_pos.is_some_and(|pt| f_rect.expand2(egui::vec2(0.0, 4.0)).contains(pt));
                let drag_payload =
                    egui::DragAndDrop::payload::<crate::ui::components::drag_drop::DragPayload>(
                        ui.ctx(),
                    );
                let is_drag_library = drag_payload.as_ref().is_some_and(|p| {
                    matches!(
                        p.source,
                        crate::ui::components::drag_drop::DragSource::Library
                    ) && (!p.library_ids.is_empty() || !p.items.is_empty())
                });
                let is_drop_target_active = is_folder_hovered && is_drag_library;

                if is_drop_target_active {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Copy);
                    let pointer_released = ui.input(|input| {
                        input.pointer.any_released()
                            || input.pointer.button_released(egui::PointerButton::Primary)
                            || (!input.pointer.is_decidedly_dragging() && input.pointer.primary_released())
                    });
                    if pointer_released {
                        if let Some(payload) = drag_payload.as_ref() {
                            let ids: Vec<i64> = if !payload.library_ids.is_empty() {
                                payload.library_ids.clone()
                            } else {
                                payload
                                    .items
                                    .iter()
                                    .filter_map(|it| {
                                        it.id.split('_').last().and_then(|s| s.parse().ok())
                                    })
                                    .collect()
                            };
                            if !ids.is_empty() {
                                action = Some(LibraryAction::Move(ids, folder.clone()));
                                kind.folders_mut(state).insert(folder.clone());
                                egui::DragAndDrop::clear_payload(ui.ctx());
                                ui.ctx().request_repaint();
                            }
                        }
                    }
                }

                let f_hover = ui
                    .ctx()
                    .animate_bool(folder_id.with("h"), response.hovered());
                let f_down = ui
                    .ctx()
                    .animate_bool(folder_id.with("d"), response.is_pointer_button_down_on());
                let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

                let f_fill = if is_drop_target_active {
                    theme.accent.linear_multiply(0.22)
                } else if f_down > 0.05 {
                    theme
                        .bg_card_hover
                        .lerp_to_gamma(theme.accent_soft, f_down * 0.45)
                } else if f_hover > 0.05 {
                    egui::Color32::TRANSPARENT.lerp_to_gamma(theme.bg_card_hover, f_hover)
                } else {
                    egui::Color32::TRANSPARENT
                };
                let f_stroke = if is_drop_target_active {
                    egui::Stroke::new(1.5, theme.accent)
                } else if f_down > 0.05 {
                    egui::Stroke::new(1.0, theme.accent.linear_multiply(0.7))
                } else if f_hover > 0.05 {
                    egui::Stroke::new(1.0, theme.border_subtle)
                } else {
                    egui::Stroke::NONE
                };
                let f_draw_rect = f_rect.shrink(f_down * 0.5);
                ui.painter().rect(
                    f_draw_rect,
                    theme.radius_md,
                    f_fill,
                    f_stroke,
                    egui::StrokeKind::Inside,
                );

                let f_text_y = f_draw_rect.center().y;
                let f_caret = if open { CARET_DOWN } else { CARET_RIGHT };
                let f_caret_g = ui.painter().layout_no_wrap(
                    f_caret.to_owned(),
                    egui::FontId::proportional(11.0),
                    theme.icon_muted,
                );
                ui.painter().galley(
                    egui::pos2(f_draw_rect.min.x + 8.0, f_text_y - f_caret_g.size().y / 2.0),
                    f_caret_g,
                    theme.icon_muted,
                );

                let folder_glyph = if is_drop_target_active || open {
                    FOLDER_OPEN
                } else {
                    FOLDER
                };
                let folder_color = if is_drop_target_active || open {
                    theme.accent
                } else {
                    theme.icon_muted
                };
                let f_icon_g = ui.painter().layout_no_wrap(
                    folder_glyph.to_owned(),
                    egui::FontId::proportional(12.5),
                    folder_color,
                );
                ui.painter().galley(
                    egui::pos2(f_draw_rect.min.x + 24.0, f_text_y - f_icon_g.size().y / 2.0),
                    f_icon_g,
                    folder_color,
                );

                let f_name_g = ui.painter().layout_no_wrap(
                    folder.clone(),
                    egui::FontId::proportional(12.5),
                    theme.text_primary,
                );
                ui.painter().galley(
                    egui::pos2(f_draw_rect.min.x + 42.0, f_text_y - f_name_g.size().y / 2.0),
                    f_name_g,
                    theme.text_primary,
                );

                if is_drop_target_active {
                    let move_txt = language.text("common-move");
                    let move_g = ui.painter().layout_no_wrap(
                        move_txt.to_owned(),
                        egui::FontId::proportional(11.0),
                        theme.accent,
                    );
                    ui.painter().galley(
                        egui::pos2(
                            f_draw_rect.max.x - move_g.size().x - 8.0,
                            f_text_y - move_g.size().y / 2.0,
                        ),
                        move_g,
                        theme.accent,
                    );
                } else {
                    let f_count_g = ui.painter().layout_no_wrap(
                        items.len().to_string(),
                        egui::FontId::proportional(11.0),
                        theme.text_muted,
                    );
                    ui.painter().galley(
                        egui::pos2(
                            f_draw_rect.max.x - f_count_g.size().x - 8.0,
                            f_text_y - f_count_g.size().y / 2.0,
                        ),
                        f_count_g,
                        theme.text_muted,
                    );
                }

                if response.clicked()
                    && !kind.folders_mut(state).remove(folder) {
                        kind.folders_mut(state).insert(folder.clone());
                    }
                response.context_menu(|ui| {
                    context_menu_header(ui);
                    if context_menu_item(ui, PLUS, language.text(kind.create_label()), &theme)
                        .clicked()
                    {
                        action = Some(LibraryAction::Create(Some(folder.clone())));
                        ui.close();
                    }
                    if kind == LibraryKind::Media
                        && context_menu_item(ui, LINK, language.text("library-add-web-video"), &theme)
                            .clicked()
                    {
                        state.web_media_modal.target_folder = folder.clone();
                        state.web_media_modal.open = true;
                        ui.close();
                    }
                    if context_menu_item(
                        ui,
                        LIST_PLUS,
                        language.text("common-add-playlist"),
                        &theme,
                    )
                    .clicked()
                    {
                        action = Some(LibraryAction::Playlist(
                            entries
                                .iter()
                                .filter(|item| &item.folder == folder)
                                .map(|item| item.id)
                                .collect(),
                        ));
                        ui.close();
                    }
                    if context_menu_item(
                        ui,
                        FOLDER_PLUS,
                        language.text("common-new-folder"),
                        &theme,
                    )
                    .clicked()
                    {
                        state.show_create_folder_modal = true;
                        ui.close();
                    }
                    context_menu_separator(ui);
                    if context_menu_danger_item(
                        ui,
                        TRASH,
                        language.text("common-delete-folder"),
                        &theme,
                    )
                    .clicked()
                    {
                        action = Some(LibraryAction::DeleteFolder(folder.clone()));
                        ui.close();
                    }
                });
                if open {
                    ui.indent(("library_folder", folder), |ui| {
                        if items.is_empty() {
                            let (slot_rect, _) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), 26.0),
                                egui::Sense::hover(),
                            );
                            let pointer_pos = ui.ctx().pointer_latest_pos().or_else(|| ui.ctx().pointer_interact_pos());
                            let is_slot_hovered = pointer_pos.is_some_and(|pt| slot_rect.expand2(egui::vec2(0.0, 3.0)).contains(pt));
                            let is_slot_drop_active = is_slot_hovered && is_drag_library;
                            let slot_stroke = if is_slot_drop_active {
                                egui::Stroke::new(1.5, theme.accent)
                            } else {
                                egui::Stroke::new(1.0, theme.border_subtle)
                            };
                            let slot_bg = if is_slot_drop_active {
                                theme.accent.linear_multiply(0.18)
                            } else {
                                theme.bg_card.linear_multiply(0.4)
                            };
                            ui.painter().rect(slot_rect.shrink(1.0), theme.radius_sm, slot_bg, slot_stroke, egui::StrokeKind::Inside);
                            let slot_msg = if is_slot_drop_active {
                                language.text("common-move")
                            } else {
                                language.text("workspace-no-items-for-event")
                            };
                            let sg = ui.painter().layout_no_wrap(
                                format!("{FOLDER}  {slot_msg}"),
                                egui::FontId::proportional(11.0),
                                if is_slot_drop_active { theme.accent } else { theme.text_muted },
                            );
                            ui.painter().galley(slot_rect.center() - sg.size() * 0.5, sg, if is_slot_drop_active { theme.accent } else { theme.text_muted });

                            let pointer_released = ui.input(|input| {
                                input.pointer.any_released()
                                    || input.pointer.button_released(egui::PointerButton::Primary)
                                    || (!input.pointer.is_decidedly_dragging() && input.pointer.primary_released())
                            });
                            if is_slot_drop_active && pointer_released {
                                if let Some(payload) = drag_payload.as_ref() {
                                    let ids: Vec<i64> = if !payload.library_ids.is_empty() {
                                        payload.library_ids.clone()
                                    } else {
                                        payload.items.iter().filter_map(|it| it.id.split('_').last().and_then(|s| s.parse().ok())).collect()
                                    };
                                    if !ids.is_empty() {
                                        action = Some(LibraryAction::Move(ids, folder.clone()));
                                        kind.folders_mut(state).insert(folder.clone());
                                        egui::DragAndDrop::clear_payload(ui.ctx());
                                        ui.ctx().request_repaint();
                                    }
                                }
                            }
                        }
                        for item in items {
                            row(ui, state, deck, kind, item, &visible, &folders, &mut action);
                        }
                    });
                }
            }
            for item in filtered.iter().filter(|item| item.folder.is_empty()) {
                row(ui, state, deck, kind, item, &visible, &folders, &mut action);
            }
            if filtered.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(28.0);
                    let (icon, hint_key) = if !query.is_empty() {
                        (MAGNIFYING_GLASS, "library-empty-search-hint")
                    } else {
                        match kind {
                            LibraryKind::Song => (kind.icon(), "library-empty-songs-hint"),
                            LibraryKind::Text => (kind.icon(), "library-empty-texts-hint"),
                            LibraryKind::Media => (kind.icon(), "library-empty-media-hint"),
                            LibraryKind::Verse => (kind.icon(), "workspace-no-books-or-verses-found"),
                        }
                    };
                    ui.label(
                        egui::RichText::new(icon)
                            .size(32.0)
                            .color(theme.text_secondary.linear_multiply(0.4)),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(language.text("common-empty"))
                            .size(12.5)
                            .strong()
                            .color(theme.text_primary),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new(language.text(hint_key))
                            .size(10.5)
                            .color(theme.text_secondary),
                    );
                    ui.add_space(10.0);
                    if !query.is_empty() {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(language.text("library-clear-search"))
                                        .size(11.0),
                                )
                                .fill(theme.bg_card)
                                .stroke(egui::Stroke::new(1.0, theme.border_default))
                                .corner_radius(6.0),
                            )
                            .clicked()
                        {
                            state.search_query.clear();
                        }
                    } else {
                        let btn_label = format!("+  {}", language.text(kind.create_label()));
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new(btn_label).size(11.0))
                                    .fill(theme.bg_card)
                                    .stroke(egui::Stroke::new(1.0, theme.border_default))
                                    .corner_radius(6.0),
                            )
                            .clicked()
                        {
                            action = Some(LibraryAction::Create(None));
                        }
                    }
                });
            }
            let empty_h = (ui.available_height() - 8.0).max(40.0);
            let (root_rect, response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), empty_h),
                egui::Sense::click(),
            );
            let pointer_pos = ui.ctx().pointer_latest_pos().or_else(|| ui.ctx().pointer_interact_pos());
            let is_root_hover = pointer_pos.is_some_and(|pt| root_rect.intersect(ui.clip_rect()).contains(pt));
            let drag_payload =
                egui::DragAndDrop::payload::<crate::ui::components::drag_drop::DragPayload>(
                    ui.ctx(),
                );
            let is_drag_library = drag_payload.as_ref().is_some_and(|p| {
                matches!(
                    p.source,
                    crate::ui::components::drag_drop::DragSource::Library
                ) && (!p.library_ids.is_empty() || !p.items.is_empty())
            });
            let is_root_drop_active = is_root_hover && is_drag_library;

            if is_root_drop_active {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Copy);
                ui.painter().rect(
                    root_rect.shrink(4.0),
                    theme.radius_md,
                    theme.accent.linear_multiply(0.12),
                    egui::Stroke::new(1.5, theme.accent),
                    egui::StrokeKind::Inside,
                );
                let root_txt = language.text("common-root");
                let move_g = ui.painter().layout_no_wrap(
                    format!("{FOLDER}  {root_txt}"),
                    egui::FontId::proportional(11.5),
                    theme.accent,
                );
                ui.painter().galley(
                    root_rect.center() - move_g.size() * 0.5,
                    move_g,
                    theme.accent,
                );
                let pointer_released = ui.input(|input| {
                    input.pointer.any_released()
                        || input.pointer.button_released(egui::PointerButton::Primary)
                });
                if pointer_released {
                    if let Some(payload) = drag_payload.as_ref() {
                        let ids: Vec<i64> = if !payload.library_ids.is_empty() {
                            payload.library_ids.clone()
                        } else {
                            payload
                                .items
                                .iter()
                                .filter_map(|it| {
                                    it.id.split('_').last().and_then(|s| s.parse().ok())
                                })
                                .collect()
                        };
                        if !ids.is_empty() {
                            action = Some(LibraryAction::Move(ids, String::new()));
                            egui::DragAndDrop::clear_payload(ui.ctx());
                            ui.ctx().request_repaint();
                        }
                    }
                }
            }

            if response.clicked() {
                kind.clear(state);
            }
            response.context_menu(|ui| {
                context_menu_header(ui);
                if context_menu_item(ui, PLUS, language.text(kind.create_label()), &theme).clicked()
                {
                    action = Some(LibraryAction::Create(None));
                    ui.close();
                }
                if kind == LibraryKind::Media
                    && context_menu_item(ui, LINK, language.text("library-add-web-video"), &theme)
                        .clicked()
                {
                    state.web_media_modal.target_folder.clear();
                    state.web_media_modal.open = true;
                    ui.close();
                }
                if context_menu_item(ui, FOLDER_PLUS, language.text("common-new-folder"), &theme)
                    .clicked()
                {
                    state.show_create_folder_modal = true;
                    ui.close();
                }
            });
        });
    if let Some(action) = action {
        actions::execute(action, kind, &entries, state, deck, presenting, db);
    }
}

fn collect_folders(
    kind: LibraryKind,
    state: &ControlPanelState,
    entries: &[LibraryEntry],
    db: Option<&AppDb>,
) -> Vec<String> {
    let mut folders = db
        .and_then(|db| {
            if kind == LibraryKind::Song {
                db.get_folders().ok()
            } else {
                db.get_folders_by_category(kind.category()).ok()
            }
        })
        .unwrap_or_default();
    folders.extend(entries.iter().map(|item| item.folder.clone()));
    folders.extend(kind.folders(state).iter().cloned());
    folders.retain(|folder| !folder.is_empty());
    folders.sort_by_key(|folder| fold(folder));
    folders.dedup();
    folders
}

fn selection_bar(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    kind: LibraryKind,
    folders: &[String],
    action: &mut Option<LibraryAction>,
) {
    let ids: Vec<_> = kind.selected(state).iter().copied().collect();
    if ids.len() < 2 {
        return;
    }
    let theme = state.theme.clone();
    let language = state.settings.language;
    ui.horizontal(|ui| {
        if crate::ui::components::interaction::selection_summary(ui, ids.len(), &theme).clicked() {
            kind.clear(state);
        }
        if ui
            .button(TRASH)
            .on_hover_text(language.text("common-delete"))
            .clicked()
        {
            *action = Some(LibraryAction::Delete(ids.clone()));
        }
        let response = ui
            .button(FOLDER)
            .on_hover_text(language.text("common-move"));
        egui::Popup::menu(&response).show(|ui| {
            move_menu(ui, state, folders, &ids, action);
        });
        if ui
            .button(LIST_PLUS)
            .on_hover_text(language.text("common-add-playlist"))
            .clicked()
        {
            *action = Some(LibraryAction::Playlist(ids.clone()));
        }
        if ui
            .button(PUSH_PIN)
            .on_hover_text(language.text("common-pin"))
            .clicked()
        {
            *action = Some(LibraryAction::Pin(ids));
        }
    });
}

fn row(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &SlideDeck,
    kind: LibraryKind,
    item: &LibraryEntry,
    visible: &[i64],
    folders: &[String],
    action: &mut Option<LibraryAction>,
) {
    let theme = state.theme.clone();
    let language = state.settings.language;
    let is_selected = kind.selected(state).contains(&item.id);
    let is_current = kind == LibraryKind::Song
        && kind.selected(state).is_empty()
        && deck.selected_song_index() == item.index;
    let is_active = is_selected || is_current;

    let id = ui.make_persistent_id((kind.category(), item.id));
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 28.0), egui::Sense::hover());
    let response = ui.interact(rect, id, egui::Sense::click_and_drag());
    let visual = crate::ui::components::interaction::row_emphasis(ui, &response, is_active, &theme);
    let hover = visual.hover;
    let down = visual.press;
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

    let is_pressed = response.is_pointer_button_down_on();

    let draw_rect = rect.shrink(down * 0.5);

    if is_selected && kind.selected(state).len() > 1 {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                draw_rect.min + egui::vec2(2.5, 5.0),
                egui::vec2(2.5, draw_rect.height() - 10.0),
            ),
            1.5,
            theme.accent,
        );
    }

    let painter = ui
        .painter()
        .with_clip_rect(draw_rect.shrink2(egui::vec2(6.0, 0.0)));
    let text_y = draw_rect.center().y;
    let start_x = draw_rect.min.x
        + if is_selected && kind.selected(state).len() > 1 {
            12.0
        } else {
            8.0
        };

    let icon_galley = painter.layout_no_wrap(
        kind.icon().to_owned(),
        egui::FontId::proportional(12.0),
        if is_active || is_pressed {
            theme.accent
        } else {
            theme.icon_muted
        },
    );
    let icon_w = icon_galley.size().x;
    painter.galley(
        egui::pos2(start_x, text_y - icon_galley.size().y / 2.0),
        icon_galley,
        if is_active || is_pressed {
            theme.accent
        } else {
            theme.icon_muted
        },
    );

    let title_x = start_x + icon_w + 6.0;
    let title_color = if is_active || is_pressed {
        theme.accent
    } else if hover > 0.5 {
        theme.text_primary
    } else {
        theme.text_primary
    };
    let title_galley = painter.layout_no_wrap(
        item.title.clone(),
        egui::FontId::proportional(12.5),
        title_color,
    );
    let title_w = title_galley.size().x;
    painter.galley(
        egui::pos2(title_x, text_y - title_galley.size().y / 2.0),
        title_galley,
        title_color,
    );

    if !item.subtitle.trim().is_empty() {
        let author_x = title_x + title_w + 8.0;
        let author_color = if is_active || is_pressed {
            theme.text_secondary
        } else {
            theme.text_muted
        };
        let author_galley = painter.layout_no_wrap(
            item.subtitle.clone(),
            egui::FontId::proportional(11.0),
            author_color,
        );
        painter.galley(
            egui::pos2(author_x, text_y - author_galley.size().y / 2.0),
            author_galley,
            author_color,
        );
    }

    let response = if item.subtitle.is_empty() {
        response.on_hover_text(&item.title)
    } else {
        response.on_hover_text(format!("{}\n{}", item.title, item.subtitle))
    };

    if response.drag_started() && !kind.selected(state).contains(&item.id) {
        model::select_range(kind, state, item.id, visible, false, false);
    }
    crate::ui::components::drag_drop::source(ui, &response, || {
        use crate::ui::components::drag_drop::{DragPayload, DragSource};
        let mut payload = DragPayload::new(
            DragSource::Library,
            &item.title,
            &item.subtitle,
            kind.icon(),
        );
        let selected = kind.selected(state);
        let matching: Vec<_> = model::entries(kind, state, deck)
            .into_iter()
            .filter(|entry| {
                if selected.contains(&item.id) {
                    selected.contains(&entry.id)
                } else {
                    entry.id == item.id
                }
            })
            .collect();
        payload.library_ids = matching.iter().map(|entry| entry.id).collect();
        payload.items = matching
            .into_iter()
            .map(|entry| crate::core::models::QuickItem {
                id: format!("{}_{}", kind.category(), entry.id),
                title: entry.title,
                subtitle: entry.subtitle,
                content: entry.content,
                kind: kind.quick_kind(),
            })
            .collect();
        payload.count = payload.items.len();
        payload
    });

    if response.clicked() {
        let (shift, ctrl) = ui.input(|input| {
            (
                input.modifiers.shift,
                input.modifiers.ctrl || input.modifiers.command,
            )
        });
        model::select_range(kind, state, item.id, visible, shift, ctrl);
        if !shift && !ctrl {
            *action = Some(LibraryAction::Preview(item.id));
        }
    }
    if response.double_clicked() {
        let ids = if is_selected && kind.selected(state).len() > 1 {
            kind.selected(state).iter().copied().collect::<Vec<_>>()
        } else {
            vec![item.id]
        };
        *action = Some(LibraryAction::Playlist(ids));
    }
    response.context_menu(|ui| {
        context_menu_header(ui);
        let ids = if is_selected && kind.selected(state).len() > 1 {
            kind.selected(state).iter().copied().collect::<Vec<_>>()
        } else {
            vec![item.id]
        };
        if context_menu_item(ui, PLAY, language.text("common-project"), &theme).clicked() {
            *action = Some(LibraryAction::Present(item.id));
            ui.close();
        }
        if context_menu_item(ui, EYE, language.text("common-preview"), &theme).clicked() {
            *action = Some(LibraryAction::Preview(item.id));
            ui.close();
        }
        if matches!(kind, LibraryKind::Song | LibraryKind::Text)
            && context_menu_item(ui, PENCIL_SIMPLE, language.text("common-edit"), &theme).clicked()
        {
            *action = Some(LibraryAction::Edit(item.id));
            ui.close();
        }
        if kind == LibraryKind::Media
            && context_menu_item(ui, ARROWS_CLOCKWISE, language.text("library-convert-media"), &theme).clicked()
        {
            *action = Some(LibraryAction::Convert(item.id));
            ui.close();
        }
        if context_menu_item(ui, LIST_PLUS, language.text("common-add-playlist"), &theme).clicked()
        {
            *action = Some(LibraryAction::Playlist(ids.clone()));
            ui.close();
        }
        if context_menu_item(ui, PUSH_PIN, language.text("common-pin"), &theme).clicked() {
            *action = Some(LibraryAction::Pin(ids.clone()));
            ui.close();
        }
        context_menu_submenu(ui, FOLDER, language.text("common-move"), &theme, |ui| {
            move_menu(ui, state, folders, &ids, action)
        });
        context_menu_separator(ui);
        if context_menu_danger_item(ui, TRASH, language.text("common-delete"), &theme).clicked() {
            *action = Some(LibraryAction::Delete(ids));
            ui.close();
        }
    });
}

fn move_menu(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    folders: &[String],
    ids: &[i64],
    action: &mut Option<LibraryAction>,
) {
    let theme = state.theme.clone();
    let language = state.settings.language;
    context_menu_header(ui);
    for folder in folders
        .iter()
        .map(String::as_str)
        .chain(std::iter::once(""))
    {
        let label = if folder.is_empty() {
            language.text("common-root")
        } else {
            folder
        };
        if context_menu_item(ui, FOLDER, label, &theme).clicked() {
            *action = Some(LibraryAction::Move(ids.to_vec(), folder.into()));
            ui.close();
        }
    }
    context_menu_separator(ui);
    if context_menu_item(ui, FOLDER_PLUS, language.text("common-new-folder"), &theme).clicked() {
        state.show_create_folder_modal = true;
        ui.close();
    }
}
