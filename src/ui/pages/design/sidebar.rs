use super::{DesignEditor, controls, icon};
use crate::{
    core::{
        design::{Countdown, LayerContent, Question, new_id},
        slides::SlideDeck,
    },
    services::db::AppDb,
    ui::{components::centered_horizontal, theme::AppTheme},
};
use eframe::egui::{self, Rect, RichText, Vec2};
use egui_phosphor::regular::*;

pub(super) fn content_icon(content: &LayerContent) -> &'static str {
    match content {
        LayerContent::Text(_) => TEXT_T,
        LayerContent::Image { .. } => IMAGE,
        LayerContent::Vector { .. } => BEZIER_CURVE,
        LayerContent::Shape { .. } => RECTANGLE,
        LayerContent::Clock { .. } => CLOCK,
        LayerContent::Countdown(_) => TIMER,
        LayerContent::Questions { .. } => QUESTION,
        LayerContent::Plugin(_) => PUZZLE_PIECE,
    }
}

#[derive(Clone, Copy)]
enum SelectAction {
    Toggle(usize),
    Range(usize),
    Single(usize),
    Secondary(usize),
}

pub(super) fn render(
    ui: &mut egui::Ui,
    editor: &mut DesignEditor,
    deck: &mut SlideDeck,
    db: &AppDb,
    theme: &AppTheme,
    state: &mut crate::core::state::ControlPanelState,
) {
    editor.hovered_layer = None;
    controls::caption(ui, "design-library", theme);
    ui.add_space(2.0);
    let mut document_action = None;
    let mut new_document = None;
    let mut delete_design_id = None;
    let library_rect =
        Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 155.0));
    crate::ui::containers::selection::file_drop::handle(ui, state, library_rect);
    let library_background = ui.interact(
        library_rect,
        ui.make_persistent_id("design_documents_background"),
        egui::Sense::click(),
    );
    egui::ScrollArea::vertical()
        .id_salt("design_documents")
        .max_height(155.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if deck.designs.is_empty() {
                controls::note(ui, "design-your-saved-slides-appear-here", theme);
            }
            for scene in &deck.designs {
                let active = editor
                    .scene
                    .as_ref()
                    .is_some_and(|selected| selected.id == scene.id);
                if editor.confirm_delete && active {
                    egui::Frame::new()
                        .fill(theme.bg_card_hover.lerp_to_gamma(theme.status_danger, 0.08))
                        .stroke(egui::Stroke::new(1.0, theme.status_danger.linear_multiply(0.45)))
                        .corner_radius(6.0)
                        .inner_margin(egui::Margin::symmetric(8, 6))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.spacing_mut().item_spacing = egui::vec2(4.0, 5.0);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(TRASH).size(12.5).color(theme.status_danger));
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(crate::ui::i18n::text(ui, "design-delete-this-design-from-the-library"))
                                            .size(10.0)
                                            .strong()
                                            .color(theme.text_primary),
                                    )
                                    .truncate(),
                                );
                            });
                            ui.horizontal(|ui| {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new(format!("{X} {}", crate::ui::i18n::text(ui, "common-cancel")))
                                                .size(10.0)
                                                .color(theme.text_primary),
                                        )
                                        .stroke(egui::Stroke::new(1.0, theme.border_default))
                                        .corner_radius(6.0)
                                        .min_size(egui::vec2(58.0, 20.0)),
                                    )
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    editor.confirm_delete = false;
                                }

                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new(format!("{TRASH} {}", crate::ui::i18n::text(ui, "common-delete")))
                                                .size(10.0)
                                                .strong()
                                                .color(egui::Color32::WHITE),
                                        )
                                        .fill(theme.status_danger)
                                        .corner_radius(6.0)
                                        .min_size(egui::vec2(68.0, 20.0)),
                                    )
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    delete_design_id = Some(scene.id.clone());
                                }
                            });
                        });
                    ui.add_space(3.0);
                    continue;
                }
                let card_height = if editor.dense_cards { 44.0 } else { 55.0 };
                let (rect, _) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), card_height),
                    egui::Sense::hover(),
                );
                let more_rect = Rect::from_center_size(
                    rect.right_center() - egui::vec2(13.0, 0.0),
                    Vec2::splat(22.0),
                );
                let response = ui.interact(
                    rect,
                    ui.make_persistent_id(("design_document", &scene.id)),
                    if ui.rect_contains_pointer(more_rect) {
                        egui::Sense::hover()
                    } else {
                        egui::Sense::click_and_drag()
                    },
                );
                super::feedback::row(ui, &response, active, theme);
                let thumb_size = if editor.dense_cards {
                    egui::vec2(34.0, 25.0)
                } else {
                    egui::vec2(43.0, 32.0)
                };
                let thumb = Rect::from_min_size(
                    rect.min + egui::vec2(7.0, (card_height - thumb_size.y) * 0.5),
                    thumb_size,
                );
                ui.painter()
                    .rect_filled(thumb, 5.0, egui::Color32::from_rgb(20, 24, 30));
                crate::render::design::paint_themed(
                    &ui.painter().with_clip_rect(thumb),
                    thumb,
                    scene,
                    &state.themes,
                );
                let text_rect = Rect::from_min_max(
                    rect.min
                        + egui::vec2(
                            if editor.dense_cards { 48.0 } else { 58.0 },
                            if editor.dense_cards { 5.0 } else { 9.0 },
                        ),
                    rect.max - egui::vec2(25.0, if editor.dense_cards { 4.0 } else { 7.0 }),
                );
                let mut label = ui.new_child(egui::UiBuilder::new().max_rect(text_rect));
                label.spacing_mut().item_spacing.y = 3.0;
                label.add(
                    egui::Label::new(
                        RichText::new(&scene.name)
                            .size(if editor.dense_cards { 11.0 } else { 11.5 })
                            .strong()
                            .color(theme.text_primary),
                    )
                    .selectable(false)
                    .truncate(),
                );
                label.add(
                    egui::Label::new(
                        RichText::new(if scene.overlay {
                            crate::ui::i18n::text(ui, "design-overlay")
                        } else {
                            crate::ui::i18n::text(ui, "control-dynamic-slide")
                        })
                        .size(if editor.dense_cards { 8.5 } else { 9.5 })
                        .color(theme.text_muted),
                    )
                    .selectable(false),
                );
                let more = hit_icon(
                    ui,
                    more_rect,
                    DOTS_THREE_VERTICAL,
                    "design-design-options",
                    &scene.id,
                    theme,
                );
                crate::ui::components::drag_drop::source(ui, &response, || {
                    crate::ui::components::drag_drop::DragPayload::for_design(scene)
                });
                egui::Popup::menu(&more).show(|ui| {
                    if let Some(action) = document_menu(ui, theme) {
                        document_action = Some((scene.clone(), action));
                    }
                });
                response.clone().context_menu(|ui| {
                    if let Some(action) = document_menu(ui, theme) {
                        document_action = Some((scene.clone(), action));
                    }
                });
                if response
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    document_action = Some((scene.clone(), 0));
                }
                ui.add_space(3.0);
            }
        });
    if let Some(target_id) = delete_design_id {
        match db.delete_design(&target_id) {
            Ok(()) => {
                deck.designs.retain(|item| item.id != target_id);
                if deck
                    .preview_design
                    .as_ref()
                    .is_some_and(|preview| preview.id == target_id)
                {
                    deck.preview_design = None;
                }
                editor.scene = None;
                editor.selected = None;
                editor.saved = true;
                editor.confirm_delete = false;
            }
            Err(error) => {
                editor.error = crate::ui::i18n::format(
                    ui,
                    "design-could-not-delete",
                    &[("error", &error.to_string())],
                );
            }
        }
    }
    library_background.context_menu(|ui| {
        new_document = super::menus::new_design_menu(ui, theme);
    });
    if let Some(kind) = new_document {
        editor.new_scene(kind, db, deck);
    }
    if let Some((mut scene, action)) = document_action {
        editor.flush(db, deck, true);
        if action == 1 {
            scene.id = new_id();
            scene.name.push_str(editor.language.text("common-copy-suffix"));
        }
        if action == 3 {
            editor.rename_in_progress = true;
            editor.flush(db, deck, true);
        }
        editor.select(scene.clone());
        if action == 1 {
            editor.saved = false;
            editor.flush(db, deck, true);
        }
        let _ = deck.preview_design_by_id(&scene.id);
        if action == 2 {
            editor.confirm_delete = true;
            ui.ctx().request_repaint();
        }
    }

    // Handle inline rename dialog across frames
    if editor.rename_in_progress {
        if let Some(scene) = editor.scene.as_mut() {
            let current_name = scene.name.clone();
            match crate::ui::components::dialog::text_input_dialog(
                ui.ctx(),
                "rename_design",
                "design-rename-title",
                "design-rename-placeholder",
                &current_name,
                theme,
            ) {
                crate::ui::components::dialog::TextInputResponse::Submitted(new_name) => {
                    scene.name = new_name;
                    editor.flush(db, deck, true);
                    editor.rename_in_progress = false;
                }
                crate::ui::components::dialog::TextInputResponse::Cancelled => {
                    editor.rename_in_progress = false;
                }
                crate::ui::components::dialog::TextInputResponse::Open => {}
            }
        } else {
            editor.rename_in_progress = false;
        }
    }
    let scene_in_library = deck
        .designs
        .iter()
        .any(|s| editor.scene.as_ref().map_or(false, |cur| cur.id == s.id));
    if editor.confirm_delete && !scene_in_library {
        egui::Frame::new()
            .fill(theme.bg_card_hover.lerp_to_gamma(theme.status_danger, 0.08))
            .stroke(egui::Stroke::new(1.0, theme.status_danger.linear_multiply(0.45)))
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(8, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(TRASH).size(12.5).color(theme.status_danger));
                    ui.add(
                        egui::Label::new(
                            RichText::new(crate::ui::i18n::text(ui, "design-delete-this-design-from-the-library"))
                                .size(10.0)
                                .strong()
                                .color(theme.text_primary),
                        )
                        .truncate(),
                    );
                });
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(format!("{X} {}", crate::ui::i18n::text(ui, "common-cancel")))
                                    .size(10.0)
                                    .color(theme.text_primary),
                            )
                            .stroke(egui::Stroke::new(1.0, theme.border_default))
                            .corner_radius(6.0)
                            .min_size(egui::vec2(58.0, 20.0)),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        editor.confirm_delete = false;
                    }
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(format!("{TRASH} {}", crate::ui::i18n::text(ui, "common-delete")))
                                    .size(10.0)
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(theme.status_danger)
                            .corner_radius(6.0)
                            .min_size(egui::vec2(68.0, 20.0)),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                        && let Some(scene) = editor.scene.as_ref()
                    {
                        let target_id = scene.id.clone();
                        match db.delete_design(&target_id) {
                            Ok(()) => {
                                deck.designs.retain(|item| item.id != target_id);
                                if deck
                                    .preview_design
                                    .as_ref()
                                    .is_some_and(|preview| preview.id == target_id)
                                {
                                    deck.preview_design = None;
                                }
                                editor.scene = None;
                                editor.selected = None;
                                editor.saved = true;
                                editor.confirm_delete = false;
                            }
                            Err(error) => {
                                editor.error = crate::ui::i18n::format(
                                    ui,
                                    "design-could-not-delete",
                                    &[("error", &error.to_string())],
                                );
                            }
                        }
                    }
                });
            });
    }
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(crate::ui::i18n::text(ui, "design-layers"))
                .size(12.5)
                .strong()
                .color(theme.text_primary),
        );
        ui.label(
            RichText::new(format!(
                "{:02}",
                editor.scene.as_ref().map_or(0, |scene| scene.layers.len())
            ))
            .size(10.0)
            .color(theme.text_muted),
        );
        if editor.selected_layers.len() > 1 {
            let all_grouped = editor.selected_layers.iter().all(|&idx| {
                editor.scene.as_ref().and_then(|s| s.layers.get(idx)).and_then(|l| l.group_id.as_ref()).is_some()
            });
            if all_grouped {
                if ui.small_button(RichText::new(format!("{FOLDER_MINUS} {}", crate::ui::i18n::text(ui, "design-ungroup"))).size(9.5)).clicked() {
                    editor.ungroup_selected();
                }
            } else if ui.small_button(RichText::new(format!("{FOLDER_PLUS} {}", crate::ui::i18n::text(ui, "design-group"))).size(9.5)).clicked() {
                editor.group_selected();
            }
        }
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), 24.0),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                let add = icon(ui, PLUS, crate::ui::i18n::text(ui, "design-add-layer"), theme);
                egui::Popup::menu(&add).show(|ui| add_menu(ui, editor));
            },
        );
    });
    let before = editor.scene.clone();
    let mut layer_action = None;
    let mut layer_drop = None;
    let mut select_action = None;
    let selected_set = editor.selected_layers.clone();
    let selected_idx = editor.selected;
    let height = (ui.available_height() - 42.0).max(70.0);
    let layer_background = ui.interact(
        Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), height)),
        ui.make_persistent_id("design_layers_background"),
        egui::Sense::click(),
    );
    egui::ScrollArea::vertical()
        .id_salt("design_layer_list")
        .auto_shrink([false, false])
        .max_height(height)
        .show(ui, |ui| {
            if editor
                .scene
                .as_ref()
                .is_none_or(|scene| scene.layers.is_empty())
            {
                ui.add_space(10.0);
                controls::empty(
                    ui,
                    STACK,
                    "design-your-canvas-is-empty",
                    "design-add-text-images-or-dynamic-information",
                    theme,
                );
                ui.add_space(6.0);
                ui.add_enabled_ui(editor.scene.is_some(), |ui| {
                    centered_horizontal(ui, "design_empty_add_layer", 140.0, |ui| {
                        ui.menu_button(format!("{PLUS} {}", crate::ui::i18n::text(ui, "design-add-layer")), |ui| {
                            add_menu(ui, editor)
                        });
                    });
                });
            } else if let Some(scene) = &mut editor.scene {
                let layer_count = scene.layers.len();
                let all_layer_ids: Vec<String> = scene.layers.iter().map(|l| l.id.clone()).collect();
                for index in (0..scene.layers.len()).rev() {
                    let layer = &mut scene.layers[index];
                    let selected = selected_set.contains(&index) || selected_idx == Some(index);
                    let row_height = if editor.dense_cards { 38.0 } else { 46.0 };
                    let (rect, _) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), row_height),
                        egui::Sense::hover(),
                    );
                    let response = ui.interact(
                        rect,
                        ui.make_persistent_id(("design_layer_row", &layer.id)),
                        egui::Sense::click_and_drag(),
                    );
                    let hovered = super::feedback::row(ui, &response, selected, theme);
                    if hovered { editor.hovered_layer = Some(index); }
                    if response.drag_started()
                        && !selected {
                            select_action = Some(SelectAction::Single(index));
                        }
                    let drag_ids = if selected && selected_set.len() > 1 {
                        selected_set.iter().filter_map(|&i| all_layer_ids.get(i).cloned()).collect()
                    } else {
                        vec![layer.id.clone()]
                    };
                    let doc_id = scene.id.clone();
                    let layer_name = layer.name.clone();
                    let layer_label = layer.content.label().to_string();
                    let icon_glyph = content_icon(&layer.content);
                    let before_clone = before.clone();
                    crate::ui::components::drag_drop::source(ui, &response, move || {
                        use crate::ui::components::drag_drop::{DragPayload, DragSource};
                        let mut payload = DragPayload::new(DragSource::Layers { document: doc_id.clone(), ids: drag_ids.clone() }, &layer_name, &layer_label, icon_glyph);
                        payload.design = before_clone.clone(); payload
                    });
                    if let Some((payload, after)) = crate::ui::components::drag_drop::reorder_target(ui, response.rect, theme, false, |source| matches!(source, crate::ui::components::drag_drop::DragSource::Layers { document, ids } if document == &scene.id && !ids.contains(&layer.id)))
                        && let crate::ui::components::drag_drop::DragSource::Layers { ids, .. } = &payload.source { layer_drop = Some((ids.clone(), layer.id.clone(), !after)); }
                    if layer.group_id.is_some() {
                        ui.painter().rect_filled(
                            Rect::from_min_size(
                                rect.min + egui::vec2(2.0, if editor.dense_cards { 9.0 } else { 11.0 }),
                                egui::vec2(2.0, if editor.dense_cards { 20.0 } else { 24.0 }),
                            ),
                            1.0,
                            theme.accent.gamma_multiply(0.4),
                        );
                    }
                    if selected {
                        ui.painter().rect_filled(
                            Rect::from_min_size(
                                rect.min + egui::vec2(0.0, if editor.dense_cards { 7.0 } else { 9.0 }),
                                egui::vec2(2.0, if editor.dense_cards { 24.0 } else { 28.0 }),
                            ),
                            2.0,
                            theme.accent,
                        );
                    }
                    ui.painter().text(
                        rect.left_center() + egui::vec2(18.0, 0.0),
                        egui::Align2::CENTER_CENTER,
                        content_icon(&layer.content),
                        egui::FontId::proportional(17.0),
                        if selected {
                            theme.accent
                        } else {
                            theme.icon_muted
                        },
                    );
                    let mut text =
                        ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_max(
                            rect.min + egui::vec2(34.0, if editor.dense_cards { 4.0 } else { 7.0 }),
                            rect.max - egui::vec2(49.0, if editor.dense_cards { 2.0 } else { 4.0 }),
                        )));
                    text.spacing_mut().item_spacing.y = 2.0;
                    text.add(
                        egui::Label::new(
                            RichText::new(&layer.name)
                                .size(if editor.dense_cards { 11.0 } else { 11.5 })
                                .color(theme.text_primary),
                        )
                        .selectable(false)
                        .truncate(),
                    );
                    let subtitle = if let Some(gname) = &layer.group_name {
                        format!("{} · {}", crate::ui::i18n::text(ui, layer.content.label()), gname)
                    } else {
                        crate::ui::i18n::text(ui, layer.content.label()).to_string()
                    };
                    text.add(egui::Label::new(
                        RichText::new(subtitle)
                            .size(if editor.dense_cards { 8.0 } else { 9.0 })
                            .color(theme.text_muted),
                    ).selectable(false));
                    if (selected || hovered || !layer.visible)
                        && hit_icon(
                            ui,
                            Rect::from_center_size(
                                rect.right_center() - egui::vec2(35.0, 0.0),
                                Vec2::splat(21.0),
                            ),
                            if layer.visible { EYE } else { EYE_SLASH },
                            "design-visibility",
                            &format!("eye{}", layer.id),
                            theme,
                        )
                        .clicked()
                        {
                            layer.visible = !layer.visible;
                        }
                    if (selected || hovered || layer.locked)
                        && hit_icon(
                            ui,
                            Rect::from_center_size(
                                rect.right_center() - egui::vec2(12.0, 0.0),
                                Vec2::splat(21.0),
                            ),
                            if layer.locked {
                                LOCK_SIMPLE
                            } else {
                                LOCK_SIMPLE_OPEN
                            },
                            "design-lock-layer",
                            &format!("lock{}", layer.id),
                            theme,
                        )
                        .clicked()
                        {
                            layer.locked = !layer.locked;
                        }
                    if response
                        .clone()
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        let modifiers = ui.input(|i| i.modifiers);
                        if modifiers.ctrl || modifiers.command {
                            select_action = Some(SelectAction::Toggle(index));
                        } else if modifiers.shift {
                            select_action = Some(SelectAction::Range(index));
                        } else {
                            select_action = Some(SelectAction::Single(index));
                        }
                    }
                    if response.secondary_clicked()
                        && !selected {
                            select_action = Some(SelectAction::Secondary(index));
                        }
                    response.context_menu(|ui| {
                        if let Some(action) = super::menus::layer_menu(ui, layer, index, layer_count, selected_set.len(), theme) {
                            layer_action = Some((index, action));
                        }
                    });
                    ui.add_space(3.0);
                }
            }
        });
    if let Some(action) = select_action {
        match action {
            SelectAction::Toggle(idx) => editor.select_layer(idx, true),
            SelectAction::Range(idx) => editor.select_layer_range(idx),
            SelectAction::Single(idx) => editor.select_layer(idx, false),
            SelectAction::Secondary(idx) => {
                if !editor.is_layer_selected(idx) {
                    editor.select_layer(idx, false);
                }
            }
        }
        editor.inspector_tab = 1;
    }
    if let Some((ids, target, after)) = layer_drop
        && let Some(scene) = &mut editor.scene {
            let selected_id = editor
                .selected
                .and_then(|index| scene.layers.get(index))
                .map(|layer| layer.id.clone());
            crate::core::interaction::move_items(
                &mut scene.layers,
                &ids,
                &target,
                after,
                |layer| &layer.id,
            );
            editor.selected =
                selected_id.and_then(|id| scene.layers.iter().position(|layer| layer.id == id));
        }
    if let Some(before) = before {
        editor.changed(before, true);
    }
    centered_horizontal(ui, "design_layer_actions", 130.0, |ui| {
        egui::Frame::new()
            .fill(theme.bg_app)
            .corner_radius(8.0)
            .inner_margin(4)
            .show(ui, |ui| {
                ui.add_enabled_ui(editor.selected.is_some(), |ui| {
                    ui.horizontal(|ui| {
                        if icon(ui, ARROW_UP, crate::ui::i18n::text(ui, "design-bring-forward"), theme).clicked() {
                            layer_action = editor
                                .selected
                                .map(|index| (index, super::menus::LayerAction::Forward));
                        }
                        if icon(ui, ARROW_DOWN, crate::ui::i18n::text(ui, "design-send-backward"), theme).clicked() {
                            layer_action = editor
                                .selected
                                .map(|index| (index, super::menus::LayerAction::Backward));
                        }
                        if icon(ui, COPY, crate::ui::i18n::text(ui, "design-duplicate-ctrl-d"), theme).clicked() {
                            layer_action = editor
                                .selected
                                .map(|index| (index, super::menus::LayerAction::Duplicate));
                        }
                        if icon(ui, TRASH, crate::ui::i18n::text(ui, "design-delete-layer-delete"), theme).clicked() {
                            layer_action = editor
                                .selected
                                .map(|index| (index, super::menus::LayerAction::Delete));
                        }
                    });
                });
            });
    });
    if let Some((index, action)) = layer_action {
        super::menus::apply_layer(editor, index, action);
    }
    layer_background.context_menu(|ui| {
        crate::ui::components::context_menu_header(ui);
        ui.add_enabled_ui(editor.scene.is_some(), |ui| {
            ui.menu_button(crate::ui::i18n::format(ui, "design-add-layer-label", &[("icon", PLUS)]), |ui| {
                add_menu(ui, editor)
            });
        });
        crate::ui::components::context_menu_separator(ui);
        if let Some(kind) = super::menus::new_design_menu(ui, theme) {
            editor.new_scene(kind, db, deck);
        }
    });
}

fn document_menu(ui: &mut egui::Ui, theme: &AppTheme) -> Option<u8> {
    let mut action = None;
    for (value, label) in [
        (0, crate::ui::i18n::format(ui, "design-open-design", &[("icon", FOLDER_OPEN)])),
        (1, crate::ui::i18n::format(ui, "design-duplicate-design-label", &[("icon", COPY)])),
        (3, crate::ui::i18n::format(ui, "design-rename-design", &[("icon", PENCIL)])),
    ] {
        if ui.button(label).clicked() {
            action = Some(value);
            ui.close();
        }
    }
    ui.separator();
    let delete_label = crate::ui::i18n::format(ui, "design-delete-design-variant-3", &[("icon", TRASH)]);
    if ui
        .button(RichText::new(delete_label).color(theme.status_danger))
        .clicked()
    {
        action = Some(2);
        ui.close();
    }
    action
}
fn hit_icon(
    ui: &mut egui::Ui,
    rect: Rect,
    glyph: &str,
    label: &str,
    id: &str,
    theme: &AppTheme,
) -> egui::Response {
    let response = ui.interact(rect, ui.make_persistent_id(id), egui::Sense::click());
    if response.hovered() || response.is_pointer_button_down_on() {
        ui.painter().rect_filled(
            rect,
            5.0,
            if response.is_pointer_button_down_on() {
                theme.accent_soft
            } else {
                theme.bg_pill_active
            },
        );
    }
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        egui::FontId::proportional(13.0),
        if response.hovered() {
            theme.accent
        } else {
            theme.icon_muted
        },
    );
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(crate::ui::i18n::text(ui, label))
}
pub(super) fn add_menu(ui: &mut egui::Ui, editor: &mut DesignEditor) {
    let language = crate::ui::i18n::language(ui.ctx());
    for (kind, label) in [
        (0, crate::ui::i18n::format(ui, "design-text-label", &[("icon", TEXT_T)])),
        (1, crate::ui::i18n::format(ui, "design-image", &[("icon", IMAGE)])),
        (2, crate::ui::i18n::format(ui, "design-shape-label", &[("icon", RECTANGLE)])),
        (3, crate::ui::i18n::format(ui, "design-clock-label", &[("icon", CLOCK)])),
        (4, crate::ui::i18n::format(ui, "design-countdown-label", &[("icon", TIMER)])),
        (5, crate::ui::i18n::format(ui, "design-questions-and-answers-label", &[("icon", QUESTION)])),
    ] {
        if ui.button(label).clicked() {
            let content = match kind {
                0 => Some(LayerContent::Text(language.text("design-your-text-here").into())),
                1 => {
                    if editor.image_import.is_none()
                        && let Some(scene) = &editor.scene {
                            editor.image_import = Some((
                                scene.id.clone(),
                                None,
                                crate::services::tasks::BackgroundTask::spawn_with_repaint(
                                    ui.ctx(),
                                    move || super::inspector::import_image(language),
                                ),
                            ));
                        }
                    None
                }
                2 => Some(LayerContent::Shape { radius: 16.0 }),
                3 => Some(LayerContent::Clock {
                    seconds: false,
                    date: false,
                }),
                4 => Some(LayerContent::Countdown(Countdown::default())),
                _ => Some(LayerContent::Questions {
                    items: vec![Question {
                        question: language.text("design-write-your-question").into(),
                        answer: language.text("design-write-the-answer").into(),
                    }],
                    current: 0,
                    revealed: false,
                }),
            };
            if let Some(content) = content {
                editor.add_layer(content);
            }
            ui.close();
        }
    }
    let mut selected = None;
    let language = crate::ui::i18n::language(ui.ctx());
    for plugin in editor.plugins.iter().filter(|plugin| plugin.enabled && !plugin.package.components.is_empty()) {
        ui.separator();
        ui.label(egui::RichText::new(plugin.package.localized_name(language)).small());
        for component in &plugin.package.components {
            if ui.button(format!("{PUZZLE_PIECE} {}", component.localized_name(language))).on_hover_text(component.localized_description(language)).clicked() {
                selected = Some(crate::core::plugins::PluginInstance::new_localized(plugin, component, language));
                ui.close();
            }
        }
    }
    if let Some(instance) = selected {
        let name = instance.definition.localized_name(language).to_owned();
        editor.add_layer(LayerContent::Plugin(Box::new(instance)));
        if let Some(layer) = editor.scene.as_mut().and_then(|scene| scene.layers.last_mut()) { layer.name = name; }
    }
    if editor.plugins.iter().all(|plugin| !plugin.enabled || plugin.package.components.is_empty()) {
        ui.label(egui::RichText::new(language.text("plugins-no-components")).small());
    }
}
