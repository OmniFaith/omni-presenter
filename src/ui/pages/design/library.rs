use crate::core::{
    design::DesignDocument,
    models::{PlaylistItem, PlaylistItemKind, QuickItemKind},
    slides::SlideDeck,
    state::{ControlPanelState, SidebarPage},
};
use crate::services::db::AppDb;
use crate::ui::components::{
    context_menu_danger_item, context_menu_disabled_item, context_menu_header, context_menu_item,
    context_menu_separator, context_menu_submenu,
};
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, RichText, Vec2};
use egui_phosphor::regular::*;

#[path = "library_interaction.rs"]
mod interactions;
#[path = "library_preview.rs"]
mod preview;
use interactions::handle_design_click;
pub use preview::render_preview;

pub fn pin(state: &mut ControlPanelState, scene: &DesignDocument) {
    state.add_quick_item(
        scene.name.clone(),
        state.settings.language.text("workspace-design").to_string(),
        QuickItemKind::Design,
        scene.id.clone(),
    );
    state.notify(state.settings.language.format(
        "design-pinned-to-the-quick-bar",
        &[("name", &scene.name)],
    ));
}
pub fn playlist(state: &mut ControlPanelState, scene: &DesignDocument) {
    state.playlist.push(PlaylistItem {
        id: crate::core::design::new_id(),
        title: scene.name.clone(),
        subtitle: state.settings.language.text("workspace-design").to_string(),
        kind: PlaylistItemKind::Design,
        played: false,
        content: scene.id.clone(),
    });
    state.notify(state.settings.language.format(
        "design-added-to-the-playlist",
        &[("name", &scene.name)],
    ));
}

#[derive(Default)]
struct DesignSelectionActions {
    preview: Option<String>,
    project: Option<String>,
    edit: Option<String>,
    pin: Option<String>,
    playlist: Vec<String>,
    delete_ids: Vec<String>,
    move_ids: Option<(Vec<String>, String)>,
    delete_folder: Option<String>,
    playlist_folder: Option<String>,
}

fn design_context_menu(
    ui: &mut egui::Ui,
    scene: &DesignDocument,
    folders: &[String],
    state: &mut ControlPanelState,
    theme: &AppTheme,
    actions: &mut DesignSelectionActions,
) {
    context_menu_header(ui);
    if context_menu_item(ui, PENCIL_SIMPLE, "design-edit-design", theme).clicked() {
        actions.edit = Some(scene.id.clone());
        ui.close();
    }
    if context_menu_item(ui, PLAY, "control-project", theme).clicked() {
        actions.project = Some(scene.id.clone());
        ui.close();
    }
    if context_menu_item(ui, PUSH_PIN, "design-pin-to-quick-bar", theme).clicked() {
        actions.pin = Some(scene.id.clone());
        ui.close();
    }
    if context_menu_item(ui, LIST_PLUS, "design-add-to-playlist", theme).clicked() {
        actions.playlist.push(scene.id.clone());
        ui.close();
    }
    context_menu_submenu(ui, FOLDER, "design-move-to-folder", theme, |ui| {
        if folders.is_empty() {
            context_menu_disabled_item(ui, FOLDER, "design-no-folders", theme);
        }
        for folder in folders {
            if context_menu_item(ui, FOLDER, folder, theme).clicked() {
                actions.move_ids = Some((vec![scene.id.clone()], folder.clone()));
                ui.close();
            }
        }
        if !scene.folder.is_empty()
            && context_menu_item(ui, FOLDER, "design-move-to-root-folder", theme).clicked()
        {
            actions.move_ids = Some((vec![scene.id.clone()], String::new()));
            ui.close();
        }
        context_menu_separator(ui);
        if context_menu_item(ui, FOLDER_PLUS, "design-new-folder", theme).clicked() {
            state.show_create_folder_modal = true;
            ui.close();
        }
    });
    context_menu_separator(ui);
    if context_menu_danger_item(ui, TRASH, "design-delete-design", theme).clicked() {
        actions.delete_ids = vec![scene.id.clone()];
        ui.close();
    }
}

#[allow(clippy::too_many_arguments)]
fn render_design_grid_item(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    presenting: &mut bool,
    theme: &AppTheme,
    scene: &DesignDocument,
    visible_ids: &[String],
    folders: &[String],
    actions: &mut DesignSelectionActions,
) {
    let selected = state.selected_design_ids.contains(&scene.id);
    let preview = deck
        .preview_design
        .as_ref()
        .is_some_and(|preview| preview.id == scene.id);
    egui::Frame::new()
        .fill(if selected || preview {
            theme.bg_card_active
        } else {
            theme.bg_panel
        })
        .corner_radius(9.0)
        .inner_margin(8)
        .show(ui, |ui| {
            let width = ui.available_width();
            let height = (width / state.presentation_aspect_ratio.clamp(0.5, 3.5)).max(54.0);
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click_and_drag());
            ui.painter()
                .rect_filled(rect, 6.0, Color32::from_rgb(20, 24, 30));
            crate::render::design::paint_themed(ui.painter(), rect, scene, &state.themes);
            super::feedback::thumbnail(ui, &response, selected || preview, theme);
            if selected {
                ui.painter().rect_stroke(
                    rect,
                    6.0,
                    egui::Stroke::new(1.5, theme.accent),
                    egui::StrokeKind::Inside,
                );
            }
            handle_design_click(&response, ui, state, scene, visible_ids, actions);
            interactions::source(ui, &response, state, deck, scene);
            if response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            response
                .context_menu(|ui| design_context_menu(ui, scene, folders, state, theme, actions));
            ui.add_space(4.0);
            ui.label(
                RichText::new(&scene.name)
                    .strong()
                    .size(12.0)
                    .color(theme.text_primary),
            );
            ui.horizontal(|ui| {
                if super::icon(ui, PLAY, "control-project", theme).clicked() {
                    project(state, deck, presenting, scene);
                }
                if super::icon(ui, PENCIL_SIMPLE, "design-edit-design", theme).clicked() {
                    let _ = deck.preview_design_by_id(&scene.id);
                    state.active_sidebar_page = SidebarPage::Design;
                }
                if super::icon(ui, PUSH_PIN, "design-pin-to-quick-bar", theme).clicked() {
                    pin(state, scene);
                }
                if super::icon(ui, LIST_PLUS, "design-add-selection-to-playlist", theme).clicked()
                {
                    playlist(state, scene);
                }
            });
        });
    ui.add_space(6.0);
}

#[allow(clippy::too_many_arguments)]
fn render_design_list_item(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    theme: &AppTheme,
    scene: &DesignDocument,
    visible_ids: &[String],
    folders: &[String],
    actions: &mut DesignSelectionActions,
) {
    let selected = state.selected_design_ids.contains(&scene.id);
    let preview = deck
        .preview_design
        .as_ref()
        .is_some_and(|preview| preview.id == scene.id);
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 46.0),
        egui::Sense::click_and_drag(),
    );
    super::feedback::row(ui, &response, selected || preview, theme);
    if selected && state.selected_design_ids.len() > 1 {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(rect.min + egui::vec2(2.0, 6.0), egui::vec2(2.5, 34.0)),
            1.5,
            theme.accent,
        );
    }
    let thumb = egui::Rect::from_min_size(rect.min + egui::vec2(8.0, 6.0), egui::vec2(58.0, 34.0));
    ui.painter()
        .rect_filled(thumb, 4.0, Color32::from_rgb(20, 24, 30));
    crate::render::design::paint_themed(ui.painter(), thumb, scene, &state.themes);
    if selected {
        ui.painter().rect_stroke(
            thumb,
            4.0,
            egui::Stroke::new(1.0, theme.accent),
            egui::StrokeKind::Inside,
        );
    }
    let text_x = thumb.right() + 8.0;
    let name = ui.painter().layout_no_wrap(
        scene.name.clone(),
        egui::FontId::proportional(12.5),
        theme.text_primary,
    );
    ui.painter().galley(
        egui::pos2(text_x, rect.center().y - name.size().y + 2.0),
        name,
        theme.text_primary,
    );
    let subtitle = if scene.folder.is_empty() {
        if scene.overlay {
            crate::ui::i18n::text(ui, "design-overlay")
        } else {
            crate::ui::i18n::text(ui, "design-slide")
        }
    } else {
        scene.folder.as_str()
    };
    let subtitle = ui.painter().layout_no_wrap(
        subtitle.to_owned(),
        egui::FontId::proportional(10.5),
        theme.text_muted,
    );
    ui.painter().galley(
        egui::pos2(text_x, rect.center().y + 3.0),
        subtitle,
        theme.text_muted,
    );
    handle_design_click(&response, ui, state, scene, visible_ids, actions);
    interactions::source(ui, &response, state, deck, scene);
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.context_menu(|ui| design_context_menu(ui, scene, folders, state, theme, actions));
}

pub fn render_selection(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    presenting: &mut bool,
    db: Option<&AppDb>,
) {
    let theme = state.theme.clone();
    let background = ui.interact(
        ui.available_rect_before_wrap(),
        ui.make_persistent_id("design_library_background"),
        egui::Sense::click(),
    );
    let mut create = None;
    background.context_menu(|ui| {
        create = super::menus::new_design_menu(ui, &theme);
        context_menu_separator(ui);
        if context_menu_item(ui, FOLDER_PLUS, "design-new-folder", &theme).clicked() {
            state.show_create_folder_modal = true;
            ui.close();
        }
    });
    if let Some(kind) = create {
        create_design(state, deck, db, kind);
    }
    if deck.designs.is_empty() {
        ui.add_space(20.0);
        ui.label(crate::ui::i18n::text(ui, "design-your-designs-will-appear-here-automatically"));
        let btn_text = crate::ui::i18n::format(ui, "design-create-design", &[("icon", PLUS)]);
        if ui.button(btn_text).clicked() {
            state.active_sidebar_page = SidebarPage::Design;
        }
        return;
    }

    let query = crate::core::bible::fold(&state.search_query);
    let filtered: Vec<DesignDocument> = deck
        .designs
        .iter()
        .filter(|scene| crate::core::bible::fold(&scene.name).contains(&query))
        .cloned()
        .collect();
    if filtered.is_empty() {
        ui.add_space(12.0);
        ui.label(
            RichText::new(crate::ui::i18n::text(ui, "design-no-designs-found"))
                .size(12.0)
                .color(theme.text_muted),
        );
        return;
    }

    let mut folders: Vec<String> = Vec::new();
    if let Some(app_db) = db
        && let Ok(db_folders) = app_db.get_folders_by_category("design") {
            for folder in db_folders {
                if !folder.is_empty() && !folders.contains(&folder) {
                    folders.push(folder);
                }
            }
        }
    for scene in &deck.designs {
        if !scene.folder.is_empty() && !folders.contains(&scene.folder) {
            folders.push(scene.folder.clone());
        }
    }
    for folder in &state.open_design_folders {
        if !folder.is_empty() && !folders.contains(folder) {
            folders.push(folder.clone());
        }
    }
    folders.sort_by_key(|folder| folder.to_lowercase());

    let mut groups: Vec<(Option<String>, Vec<DesignDocument>)> = folders
        .iter()
        .map(|folder| {
            let items = filtered
                .iter()
                .filter(|scene| &scene.folder == folder)
                .cloned()
                .collect();
            (Some(folder.clone()), items)
        })
        .collect();
    if !query.is_empty() {
        groups.retain(|(_, items)| !items.is_empty());
    }
    let root: Vec<DesignDocument> = filtered
        .iter()
        .filter(|scene| scene.folder.is_empty())
        .cloned()
        .collect();
    if !root.is_empty() {
        groups.push((None, root));
    }

    let mut visible_ids: Vec<String> = Vec::new();
    for (folder, items) in &groups {
        let visible = folder
            .as_ref()
            .is_none_or(|folder| state.open_design_folders.contains(folder));
        if visible {
            visible_ids.extend(items.iter().map(|scene| scene.id.clone()));
        }
    }

    ui.horizontal(|ui| {
        let count = filtered.len();
        ui.label(
            RichText::new(state.settings.language.count("design-count-count", "design-count-count", count))
            .size(10.5)
            .color(theme.text_muted),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            crate::ui::components::interaction::view_toggle(
                ui,
                &mut state.design_list_view,
                &theme,
            );
        });
    });
    ui.add_space(4.0);

    let mut actions = DesignSelectionActions::default();

    if state.selected_design_ids.len() > 1 {
        let ids: Vec<String> = state.selected_design_ids.iter().cloned().collect();
        let count = ids.len();
        let count_str = count.to_string();
        egui::Frame::new()
            .fill(theme.bg_card)
            .corner_radius(theme.radius_md)
            .inner_margin(egui::Margin::symmetric(8, 4))
            .stroke(egui::Stroke::new(1.0, theme.accent.linear_multiply(0.4)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(crate::ui::i18n::format(
                            ui,
                            "design-selected",
                            &[("count", &count_str)],
                        ))
                        .size(10.5)
                        .strong()
                        .color(theme.accent),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if super::icon(ui, X, "design-clear-selection", &theme).clicked() {
                            state.clear_design_selection();
                        }
                        let delete_tooltip = crate::ui::i18n::format(
                            ui,
                            "design-delete-designs",
                            &[("count", &count_str)],
                        );
                        if super::icon(ui, TRASH, &delete_tooltip, &theme)
                            .clicked()
                        {
                            actions.delete_ids = ids.clone();
                        }
                        let move_btn =
                            super::icon(ui, FOLDER, "design-move-selection-to-a-folder", &theme);
                        egui::Popup::menu(&move_btn).show(|ui| {
                            context_menu_header(ui);
                            if folders.is_empty() {
                                context_menu_disabled_item(ui, FOLDER, "design-no-folders", &theme);
                            }
                            for folder in &folders {
                                if context_menu_item(ui, FOLDER, folder, &theme).clicked() {
                                    actions.move_ids = Some((ids.clone(), folder.clone()));
                                    ui.close();
                                }
                            }
                            if context_menu_item(ui, FOLDER, "design-move-to-root-folder", &theme)
                                .clicked()
                            {
                                actions.move_ids = Some((ids.clone(), String::new()));
                                ui.close();
                            }
                            context_menu_separator(ui);
                            if context_menu_item(ui, FOLDER_PLUS, "design-new-folder", &theme).clicked()
                            {
                                state.show_create_folder_modal = true;
                                ui.close();
                            }
                        });
                        if super::icon(
                            ui,
                            LIST_PLUS,
                            "design-add-selection-to-playlist",
                            &theme,
                        )
                        .clicked()
                        {
                            actions.playlist.extend(ids.clone());
                        }
                    });
                });
            });
        ui.add_space(4.0);
    }

    let scroll_height = ui.available_height().max(60.0);
    egui::ScrollArea::vertical()
        .id_salt("design_library")
        .max_height(scroll_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (folder, items) in &groups {
                let is_open = folder
                    .as_ref()
                    .is_none_or(|folder| state.open_design_folders.contains(folder));
                if let Some(folder) = folder {
                    let (rect, response) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), 26.0),
                        egui::Sense::click(),
                    );
                    if response.hovered() {
                        ui.painter()
                            .rect_filled(rect, theme.radius_md, theme.bg_card_hover);
                    }
                    let text_y = rect.center().y;
                    let arrow = if is_open { CARET_DOWN } else { CARET_RIGHT };
                    let arrow = ui.painter().layout_no_wrap(
                        arrow.to_owned(),
                        egui::FontId::proportional(11.0),
                        theme.icon_muted,
                    );
                    let arrow_x = rect.max.x - arrow.size().x - 6.0;
                    ui.painter().galley(
                        egui::pos2(arrow_x, text_y - arrow.size().y / 2.0),
                        arrow,
                        theme.icon_muted,
                    );
                    let count = ui.painter().layout_no_wrap(
                        items.len().to_string(),
                        egui::FontId::proportional(10.5),
                        theme.text_muted,
                    );
                    let count_x = arrow_x - count.size().x - 8.0;
                    ui.painter().galley(
                        egui::pos2(count_x, text_y - count.size().y / 2.0),
                        count,
                        theme.text_muted,
                    );
                    let label = ui.painter().layout_no_wrap(
                        format!("{FOLDER} {folder}"),
                        egui::FontId::proportional(12.0),
                        theme.text_primary,
                    );
                    let label_rect =
                        egui::Rect::from_min_max(rect.min, egui::pos2(count_x - 6.0, rect.max.y));
                    ui.painter().with_clip_rect(label_rect).galley(
                        egui::pos2(rect.min.x + 6.0, text_y - label.size().y / 2.0),
                        label,
                        theme.text_primary,
                    );
                    if response.clicked() {
                        if is_open {
                            state.open_design_folders.remove(folder);
                        } else {
                            state.open_design_folders.insert(folder.clone());
                        }
                    }
                    response.context_menu(|ui| {
                        context_menu_header(ui);
                        if context_menu_item(ui, FOLDER_PLUS, "design-new-folder", &theme).clicked() {
                            state.show_create_folder_modal = true;
                            ui.close();
                        }
                        if context_menu_item(
                            ui,
                            LIST_PLUS,
                            "design-add-designs-to-playlist",
                            &theme,
                        )
                        .clicked()
                        {
                            actions.playlist_folder = Some(folder.clone());
                            ui.close();
                        }
                        context_menu_separator(ui);
                        if context_menu_danger_item(ui, TRASH, "design-delete-folder", &theme).clicked() {
                            actions.delete_folder = Some(folder.clone());
                            ui.close();
                        }
                    });
                    if is_open && items.is_empty() {
                        ui.label(
                            RichText::new(crate::ui::i18n::text(ui, "design-empty-folder"))
                                .size(10.5)
                                .color(theme.text_muted),
                        );
                    }
                } else if !folders.is_empty() {
                    ui.add_space(6.0);
                    let (rect, _) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), 24.0),
                        egui::Sense::hover(),
                    );
                    let text_y = rect.center().y;
                    let label = ui.painter().layout_no_wrap(
                        crate::ui::i18n::text(ui, "workspace-no-folder").to_owned(),
                        egui::FontId::proportional(11.0),
                        theme.text_secondary,
                    );
                    ui.painter().galley(
                        egui::pos2(rect.min.x + 6.0, text_y - label.size().y / 2.0),
                        label,
                        theme.text_secondary,
                    );
                    let count = ui.painter().layout_no_wrap(
                        items.len().to_string(),
                        egui::FontId::proportional(10.5),
                        theme.text_muted,
                    );
                    ui.painter().galley(
                        egui::pos2(
                            rect.max.x - count.size().x - 6.0,
                            text_y - count.size().y / 2.0,
                        ),
                        count,
                        theme.text_muted,
                    );
                }
                if is_open {
                    for scene in items {
                        if state.design_list_view {
                            render_design_list_item(
                                ui,
                                state,
                                deck,
                                &theme,
                                scene,
                                &visible_ids,
                                &folders,
                                &mut actions,
                            );
                        } else {
                            render_design_grid_item(
                                ui,
                                state,
                                deck,
                                presenting,
                                &theme,
                                scene,
                                &visible_ids,
                                &folders,
                                &mut actions,
                            );
                        }
                    }
                }
            }
        });

    if let Some(id) = &actions.preview
        && let Err(error) = deck.preview_design_by_id(id) {
            state.notify(error);
        }
    if let Some(id) = &actions.project
        && let Some(scene) = deck.designs.iter().find(|scene| &scene.id == id).cloned()
    {
        project(state, deck, presenting, &scene);
    }
    if let Some(id) = &actions.edit
        && deck.designs.iter().any(|scene| &scene.id == id)
    {
        let _ = deck.preview_design_by_id(id);
        state.active_sidebar_page = SidebarPage::Design;
    }
    if let Some(id) = &actions.pin
        && let Some(scene) = deck.designs.iter().find(|scene| &scene.id == id)
    {
        pin(state, scene);
    }
    if !actions.playlist.is_empty() {
        let mut count = 0;
        for id in &actions.playlist {
            if let Some(scene) = deck.designs.iter().find(|scene| &scene.id == id) {
                state.playlist.push(PlaylistItem {
                    id: crate::core::design::new_id(),
                    title: scene.name.clone(),
                    subtitle: state.settings.language.text("workspace-design").to_string(),
                    kind: PlaylistItemKind::Design,
                    played: false,
                    content: scene.id.clone(),
                });
                count += 1;
            }
        }
        if count > 0 {
            state.notify(state.settings.language.format(
                "design-design-s-added-to-the-playlist",
                &[("count", &count.to_string())],
            ));
        }
    }
    if let Some(folder) = actions.playlist_folder {
        let scenes: Vec<DesignDocument> = deck
            .designs
            .iter()
            .filter(|scene| scene.folder == folder)
            .cloned()
            .collect();
        for scene in &scenes {
            state.playlist.push(PlaylistItem {
                id: crate::core::design::new_id(),
                title: scene.name.clone(),
                subtitle: state.settings.language.text("workspace-design").to_string(),
                kind: PlaylistItemKind::Design,
                played: false,
                content: scene.id.clone(),
            });
        }
        let count_str = scenes.len().to_string();
        state.notify(state.settings.language.format(
            "design-design-s-from-folder-added-to-the-playlist",
            &[("count", &count_str), ("folder", &folder)],
        ));
    }
    if let Some(folder) = actions.delete_folder {
        if let Some(app_db) = db {
            let _ = app_db.delete_folder_from_category("design", &folder);
        }
        for scene in deck
            .designs
            .iter_mut()
            .filter(|scene| scene.folder == folder)
        {
            scene.folder.clear();
            if let Some(app_db) = db {
                let _ = app_db.save_design(scene);
            }
        }
        state.open_design_folders.remove(&folder);
        state.notify(state.settings.language.format("design-folder-removed", &[("folder", &folder)]));
    }
    if let Some((ids, target)) = actions.move_ids {
        let count = ids.len();
        let count_str = count.to_string();
        if !target.is_empty() {
            if let Some(app_db) = db {
                let _ = app_db.add_folder_to_category("design", &target);
            }
            state.open_design_folders.insert(target.clone());
        }
        for scene in deck
            .designs
            .iter_mut()
            .filter(|scene| ids.contains(&scene.id))
        {
            scene.folder = target.clone();
            if let Some(app_db) = db {
                let _ = app_db.save_design(scene);
            }
        }
        if target.is_empty() {
            state.notify(state.settings.language.format(
                "design-design-s-moved-to-the-root-folder",
                &[("count", &count_str)],
            ));
        } else {
            state.notify(state.settings.language.format(
                "design-design-s-moved-to",
                &[("count", &count_str), ("target", &target)],
            ));
        }
    }
    if !actions.delete_ids.is_empty() {
        let ids: std::collections::HashSet<String> = actions.delete_ids.iter().cloned().collect();
        let count = ids.len();
        if let Some(app_db) = db {
            for id in &ids {
                let _ = app_db.delete_design(id);
            }
        }
        deck.designs.retain(|scene| !ids.contains(&scene.id));
        if deck
            .preview_design
            .as_ref()
            .is_some_and(|preview| ids.contains(&preview.id))
        {
            let index = deck.selected_song_index();
            if deck.library().get(index).is_some() {
                deck.select_song(index);
            } else {
                deck.load_custom_content(String::new(), String::new(), String::new());
            }
        }
        if deck
            .live_design
            .as_ref()
            .is_some_and(|live| ids.contains(&live.id))
        {
            deck.live_design = None;
        }
        if deck
            .live_overlay
            .as_ref()
            .is_some_and(|live| ids.contains(&live.id))
        {
            deck.live_overlay = None;
        }
        state
            .playlist
            .retain(|item| !(item.kind == PlaylistItemKind::Design && ids.contains(&item.content)));
        state.clear_design_selection();
        state.notify(state.settings.language.format(
            "design-design-s-deleted",
            &[("count", &count.to_string())],
        ));
    }
}

fn create_design(
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    db: Option<&AppDb>,
    kind: usize,
) {
    let scene = DesignDocument::template_localized(kind, state.settings.language);
    if let Some(db) = db
        && let Err(error) = db.save_design(&scene) {
            let error_str = error.to_string();
            state.notify(state.settings.language.format(
                "design-could-not-create-the-design",
                &[("error", &error_str)],
            ));
            return;
        }
    let id = scene.id.clone();
    deck.designs.push(scene);
    if let Err(error) = deck.preview_design_by_id(&id) {
        state.notify(error);
        return;
    }
    state.clear_design_selection();
    state.selected_design_ids.insert(id);
    state.active_sidebar_page = SidebarPage::Design;
}

fn project(
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    presenting: &mut bool,
    scene: &DesignDocument,
) {
    if let Err(error) = deck.preview_design_by_id(&scene.id) {
        state.notify(error);
        return;
    }
    deck.project_preview();
    *presenting = true;
    state.show_presentation_dock = state.settings.operator().show_dock;
}
