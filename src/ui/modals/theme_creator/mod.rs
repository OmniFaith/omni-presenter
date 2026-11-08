mod background;
mod box_style;
mod preview;
mod shader_controls;
#[cfg(test)]
mod tests;
mod text;
mod widgets;

use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use eframe::egui::{self, Color32, RichText, Stroke, StrokeKind, Vec2};
use egui_phosphor::regular::{ARROW_U_UP_LEFT, ARROW_U_UP_RIGHT};

pub fn render_theme_creator_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
) {
    let modal_id = egui::Id::unique("theme_creator_modal_overlay");
    if !state.theme_editor.open {
        ctx.data_mut(|d| d.remove_temp::<u64>(modal_id));
        return;
    }

    let current_frame = ctx.cumulative_frame_nr();
    let opened_frame = ctx.data_mut(|d| *d.get_temp_mut_or_insert_with(modal_id, || current_frame));

    let screen_rect = ctx.input(|i| i.viewport_rect());
    let mut should_save = false;
    let mut should_cancel = false;
    let mut push_snapshot = false;

    egui::Area::new(modal_id)
        .order(egui::Order::Foreground)
        .fixed_pos(screen_rect.min)
        .show(ctx, |ui| {
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                state.theme_editor.open = false;
                ctx.data_mut(|d| d.remove_temp::<u64>(modal_id));
                return;
            }

            let _ = ui.allocate_rect(screen_rect, egui::Sense::click_and_drag());
            ui.painter()
                .rect_filled(screen_rect, 0.0, Color32::from_black_alpha(140));

            let modal_w = (screen_rect.width() - 40.0)
                .clamp(860.0, 1180.0)
                .min(screen_rect.width() - 20.0);
            let modal_h = (screen_rect.height() - 40.0)
                .clamp(520.0, 800.0)
                .min(screen_rect.height() - 20.0);
            let modal_rect =
                egui::Rect::from_center_size(screen_rect.center(), Vec2::new(modal_w, modal_h));

            if current_frame > opened_frame && !egui::Popup::is_any_open(ctx)
                && ui.input(|i| i.pointer.any_click())
                    && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
                        && !modal_rect.contains(pos) && state.theme_editor.active_handle.is_none() {
                            state.theme_editor.open = false;
                            ctx.data_mut(|d| d.remove_temp::<u64>(modal_id));
                            return;
                        }

            let ambient_shadow = egui::epaint::Shadow {
                offset: [0, 14],
                blur: 32,
                spread: 0,
                color: Color32::from_black_alpha(80),
            };
            ui.painter().add(ambient_shadow.as_shape(modal_rect, 14.0));

            ui.painter()
                .rect_filled(modal_rect, 14.0, state.theme.bg_app);
            ui.painter().rect_stroke(
                modal_rect,
                14.0,
                Stroke::new(1.0, state.theme.border_default),
                StrokeKind::Inside,
            );

            let inner_rect = modal_rect.shrink(14.0);

            let header_h = 42.0;
            let header_rect =
                egui::Rect::from_min_size(inner_rect.min, Vec2::new(inner_rect.width(), header_h));
            let mut header_ui = ui.new_child(egui::UiBuilder::new().max_rect(header_rect));

            header_ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                let can_undo = !state.theme_editor.undo_stack.is_empty();
                let undo_btn = egui::Button::new(RichText::new(ARROW_U_UP_LEFT).size(15.0))
                    .fill(state.theme.bg_pill)
                    .stroke(Stroke::NONE)
                    .corner_radius(6.0)
                    .min_size(Vec2::new(32.0, 32.0));
                if ui
                    .add_enabled(can_undo, undo_btn)
                    .on_hover_text(crate::ui::i18n::text(ui, "settings-undo-change-ctrl-z"))
                    .clicked()
                    && let Some(prev) = state.theme_editor.undo_stack.pop() {
                        state
                            .theme_editor
                            .redo_stack
                            .push(state.theme_editor.editing.clone());
                        state.theme_editor.editing = prev;
                    }

                let can_redo = !state.theme_editor.redo_stack.is_empty();
                let redo_btn = egui::Button::new(RichText::new(ARROW_U_UP_RIGHT).size(15.0))
                    .fill(state.theme.bg_pill)
                    .stroke(Stroke::NONE)
                    .corner_radius(6.0)
                    .min_size(Vec2::new(32.0, 32.0));
                if ui
                    .add_enabled(can_redo, redo_btn)
                    .on_hover_text(crate::ui::i18n::text(ui, "settings-redo-change-ctrl-y"))
                    .clicked()
                    && let Some(next) = state.theme_editor.redo_stack.pop() {
                        state
                            .theme_editor
                            .undo_stack
                            .push(state.theme_editor.editing.clone());
                        state.theme_editor.editing = next;
                    }

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "settings-theme-name"))
                        .strong()
                        .size(12.5)
                        .color(state.theme.text_primary),
                );
                let title_edit = egui::TextEdit::singleline(&mut state.theme_editor.editing.name)
                    .desired_width(190.0)
                    .margin(egui::Margin::symmetric(8, 6));
                if ui.add(title_edit).changed() {
                    push_snapshot = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;

                    let save_btn = egui::Button::new(
                        RichText::new(crate::ui::i18n::text(ui, "settings-save-theme"))
                            .size(13.0)
                            .strong()
                            .color(state.theme.text_on_accent),
                    )
                    .fill(state.theme.accent)
                    .corner_radius(6.0)
                    .min_size(Vec2::new(104.0, 32.0));

                    if ui
                        .add(save_btn)
                        .on_hover_text(crate::ui::i18n::text(ui, "settings-save-theme-in-the-sqlite-database"))
                        .clicked()
                    {
                        should_save = true;
                    }

                    let cancel_btn = egui::Button::new(
                        RichText::new(crate::ui::i18n::text(ui, "control-cancel"))
                            .size(13.0)
                            .color(state.theme.text_secondary),
                    )
                    .fill(state.theme.bg_pill)
                    .corner_radius(6.0)
                    .min_size(Vec2::new(82.0, 32.0));

                    if ui.add(cancel_btn).clicked() {
                        should_cancel = true;
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    let zoom_pct = format!("{:.0}%", state.theme_editor.zoom * 100.0);
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(zoom_pct)
                                    .size(11.5)
                                    .color(state.theme.text_secondary),
                            )
                            .fill(state.theme.bg_pill)
                            .min_size(Vec2::new(44.0, 24.0)),
                        )
                        .on_hover_text(crate::ui::i18n::text(ui, "settings-reset-zoom-to-100"))
                        .clicked()
                    {
                        state.theme_editor.zoom = 1.0;
                    }

                    ui.add(
                        egui::Slider::new(&mut state.theme_editor.zoom, 0.5..=1.5)
                            .show_value(false),
                    );
                    ui.label(
                        RichText::new(crate::ui::i18n::text(ui, "workspace-zoom"))
                            .size(12.0)
                            .color(state.theme.text_secondary),
                    );
                });
            });

            let body_top = header_rect.max.y + 10.0;
            let body_rect =
                egui::Rect::from_min_max(egui::pos2(inner_rect.min.x, body_top), inner_rect.max);

            let sidebar_w = 336.0;
            let sidebar_rect =
                egui::Rect::from_min_size(body_rect.min, Vec2::new(sidebar_w, body_rect.height()));

            let mut sb_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(sidebar_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            sb_ui.set_clip_rect(sidebar_rect);
            sb_ui.set_min_size(sidebar_rect.size());

            crate::ui::pages::design::panel(&state.theme).show(&mut sb_ui, |ui| {
                crate::ui::pages::design::controls::configure(ui, &state.theme);

                if let Some(tab_idx) = crate::ui::pages::design::controls::segments(
                    ui,
                    &["design-background", "design-text", "settings-text-box"],
                    state.theme_editor.sidebar_tab,
                    &state.theme,
                ) {
                    state.theme_editor.sidebar_tab = tab_idx;
                }

                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .id_salt("theme_editor_sidebar_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_max_width(ui.available_width());
                        ui.spacing_mut().item_spacing.y = 5.0;
                        let changed = match state.theme_editor.sidebar_tab {
                            0 => background::background_tab(ui, state),
                            1 => text::text_tab(ui, state, ui.available_width()),
                            2 => box_style::box_tab(ui, state),
                            _ => false,
                        };
                        push_snapshot |= changed;
                    });
            });

            let canvas_gap = 14.0;
            let canvas_area_x = sidebar_rect.max.x + canvas_gap;
            let canvas_area_rect =
                egui::Rect::from_min_max(egui::pos2(canvas_area_x, body_rect.min.y), body_rect.max);

            push_snapshot |= preview::preview(ui, state, canvas_area_rect);
        });

    if push_snapshot {
        if state.theme_editor.undo_stack.len() > 30 {
            state.theme_editor.undo_stack.remove(0);
        }
        state
            .theme_editor
            .undo_stack
            .push(state.theme_editor.editing.clone());
        state.theme_editor.redo_stack.clear();
    }

    if should_save {
        let theme_to_save = state.theme_editor.editing.clone();

        if let Some(edit_idx) = state.theme_editor.editing_index {
            if edit_idx < state.themes.len() {
                state.themes[edit_idx] = theme_to_save.clone();
            }
            if let Some(app_db) = db {
                let _ = app_db.save_theme(&theme_to_save);
            }
        } else {
            state.themes.insert(0, theme_to_save.clone());
            state.selected_theme_index = 0;
            if let Some(app_db) = db {
                let _ = app_db.save_theme(&theme_to_save);
                let ids: Vec<String> = state.themes.iter().map(|t| t.id.clone()).collect();
                let _ = app_db.update_themes_order(&ids);
            }
        }

        if let Some(idx) = state.themes.iter().position(|t| t.id == state.live_theme_id) {
            state.live_theme_index = idx;
        }

        if state.theme_editor.editing_index == Some(state.live_theme_index)
            || state.themes.len() == 1
        {
            state.apply_theme_to_live();
        }

        state.theme_editor.editing_index = None;
        state.theme_editor.open = false;
        ctx.data_mut(|d| d.remove_temp::<u64>(modal_id));
    } else if should_cancel {
        state.theme_editor.editing_index = None;
        state.theme_editor.open = false;
        ctx.data_mut(|d| d.remove_temp::<u64>(modal_id));
    }
}
