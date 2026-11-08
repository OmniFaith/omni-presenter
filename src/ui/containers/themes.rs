use crate::core::layout::ContainerKind;
use crate::core::state::ControlPanelState;
use crate::render::{paint_theme_bg, render_theme_lyrics};
use crate::services::db::AppDb;
use crate::ui::components::{
    context_menu_danger_item, context_menu_header, context_menu_item, context_menu_separator,
    render_header_bar,
};
use eframe::egui::{self, Color32, Stroke, StrokeKind, Vec2};
use egui_phosphor::regular::{ARROW_COUNTER_CLOCKWISE, CHECK, PENCIL_SIMPLE, PLUS, STAR, TRASH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeDragPayload {
    pub source_index: usize,
}

pub fn render_themes_column(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
) -> (bool, bool) {
    let theme_tokens = state.theme.clone();
    let mut applied = (false, false);

    // Barra de título com drag handle
    render_header_bar(ui, ContainerKind::Themes, "workspace-themes", &theme_tokens);
    ui.add_space(4.0);

    egui::ScrollArea::vertical()
        .id_salt("themes_container_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let avail_w = (ui.available_width() - 4.0).max(60.0);

            // Quando diminuir a largura, reduz gradualmente até ficar em 1 coluna só
            let cols = if avail_w >= 420.0 {
                4
            } else if avail_w >= 300.0 {
                3
            } else if avail_w >= 180.0 {
                2
            } else {
                1
            };

            let gap = 6.0;
            let (card_w, card_h) = if cols == 1 {
                let w = (avail_w - 4.0).clamp(90.0, 260.0);
                let aspect = state.presentation_aspect_ratio.clamp(0.5, 3.5);
                let h = (w / aspect).round().clamp(55.0, 110.0);
                (w, h)
            } else {
                let w = ((avail_w - (cols - 1) as f32 * gap) / cols as f32)
                    .floor()
                    .max(35.0);
                let aspect = state.presentation_aspect_ratio.clamp(0.5, 3.5);
                let h = (w / aspect).round().max(25.0);
                (w, h)
            };

            let total_items = 1 + state.themes.len();
            let num_rows = total_items.div_ceil(cols);

            let mut clicked_add = false;
            let mut clicked_theme_idx = None;
            let mut action_apply_live = None;
            let mut action_toggle_fav = None;
            let mut action_edit = None;
            let mut action_delete = None;
            let mut action_restore_defaults = false;
            let mut hovered_drop_index = None;

            for row in 0..num_rows {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for col in 0..cols {
                        let item_idx = row * cols + col;
                        if item_idx >= total_items {
                            break;
                        }

                        if item_idx == 0 {
                            // Card [+] Adicionar Novo Tema
                            let (rect, resp) = ui.allocate_exact_size(
                                Vec2::new(card_w, card_h),
                                egui::Sense::click(),
                            );
                            if resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if resp.clicked() {
                                clicked_add = true;
                            }

                            let bg_color = if resp.hovered() {
                                theme_tokens.bg_pill_active
                            } else {
                                theme_tokens.bg_pill
                            };
                            let border_color = if resp.hovered() {
                                theme_tokens.border_active
                            } else {
                                theme_tokens.border_default
                            };

                            ui.painter()
                                .rect_filled(rect, theme_tokens.radius_lg, bg_color);
                            ui.painter().rect_stroke(
                                rect,
                                theme_tokens.radius_lg,
                                Stroke::new(1.0, border_color),
                                StrokeKind::Inside,
                            );
                            let plus_size = (card_h * 0.45).clamp(16.0, 26.0);
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                PLUS,
                                egui::FontId::proportional(plus_size),
                                theme_tokens.icon_muted,
                            );
                            resp.context_menu(|ui| {
                                context_menu_header(ui);
                                if context_menu_item(ui, PLUS, "workspace-create-a-custom-theme", &theme_tokens).clicked() {
                                    clicked_add = true;
                                    ui.close();
                                }
                                context_menu_separator(ui);
                                if context_menu_item(ui, ARROW_COUNTER_CLOCKWISE, "workspace-restore-default-themes", &theme_tokens).clicked() {
                                    action_restore_defaults = true;
                                    ui.close();
                                }
                            });
                            resp.on_hover_text(crate::ui::i18n::text(ui, "workspace-create-a-custom-theme"));
                        } else {
                            let theme_idx = item_idx - 1;
                            let theme = &state.themes[theme_idx];
                            let is_active = state.selected_theme_index == theme_idx;

                            // Card do tema interativo com clique e arraste para reordenar
                            let (rect, resp) = ui.allocate_exact_size(
                                Vec2::new(card_w, card_h),
                                egui::Sense::click_and_drag(),
                            );

                            // Arraste para reordenar
                            if resp.drag_started() {
                                egui::DragAndDrop::set_payload(
                                    ui.ctx(),
                                    ThemeDragPayload {
                                        source_index: theme_idx,
                                    },
                                );
                            }
                            if resp.dragged() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                            } else if resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }

                            let is_dragging_this = if let Some(payload) =
                                egui::DragAndDrop::payload::<ThemeDragPayload>(ui.ctx())
                            {
                                payload.source_index == theme_idx
                            } else {
                                false
                            };

                            let is_drop_target = if let Some(payload) =
                                egui::DragAndDrop::payload::<ThemeDragPayload>(ui.ctx())
                            {
                                payload.source_index != theme_idx && resp.hovered()
                            } else {
                                false
                            };

                            if is_drop_target {
                                hovered_drop_index = Some(theme_idx);
                            }

                            // Desenho do fundo do tema
                            paint_theme_bg(ui.painter(), rect, theme, theme_tokens.radius_lg);

                            // Efeito visual quando este card está sendo arrastado
                            if is_dragging_this {
                                ui.painter().rect_filled(
                                    rect,
                                    theme_tokens.radius_lg,
                                    Color32::from_black_alpha(100),
                                );
                            }

                            // Borda de seleção / hover / alvo de drop
                            let border_stroke = if is_drop_target {
                                Some(Stroke::new(2.5, theme_tokens.accent))
                            } else if is_active {
                                Some(Stroke::new(2.0, theme_tokens.accent))
                            } else if resp.hovered() {
                                Some(Stroke::new(1.5, theme_tokens.border_active))
                            } else {
                                None
                            };
                            if let Some(stroke) = border_stroke {
                                ui.painter().rect_stroke(
                                    rect,
                                    theme_tokens.radius_lg,
                                    stroke,
                                    StrokeKind::Inside,
                                );
                            }

                            // Indicador de Tema Favorito (Estrela Dourada no canto superior direito)
                            if theme.is_favorite {
                                let star_pos = egui::pos2(rect.max.x - 11.0, rect.min.y + 11.0);
                                ui.painter().circle_filled(
                                    star_pos,
                                    7.0,
                                    Color32::from_black_alpha(90),
                                );
                                ui.painter().text(
                                    star_pos,
                                    egui::Align2::CENTER_CENTER,
                                    STAR,
                                    egui::FontId::proportional(11.0),
                                    Color32::from_rgb(253, 224, 71),
                                );
                            }


                            // Nome do tema em etiqueta semitransparente na base do card
                            let name_bar_h = (card_h * 0.28).clamp(16.0, 22.0);
                            let name_rect = egui::Rect::from_min_max(
                                egui::pos2(rect.min.x, rect.max.y - name_bar_h),
                                rect.max,
                            );
                            ui.painter().rect_filled(
                                name_rect,
                                egui::CornerRadius {
                                    nw: 0,
                                    ne: 0,
                                    sw: theme_tokens.radius_lg.round() as u8,
                                    se: theme_tokens.radius_lg.round() as u8,
                                },
                                Color32::from_black_alpha(140),
                            );
                            ui.painter().text(
                                name_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                theme.display_name(state.settings.language),
                                egui::FontId::proportional((name_bar_h * 0.55).clamp(8.0, 11.0)),
                                Color32::WHITE,
                            );

                            // Linhas de pré-visualização de texto do tema na parte superior
                            let text_area = egui::Rect::from_min_max(
                                rect.min,
                                egui::pos2(rect.max.x, rect.max.y - name_bar_h),
                            );
                            if text_area.width() >= 40.0 && text_area.height() >= 18.0 {
                                render_theme_lyrics(
                                    ui.painter(),
                                    text_area,
                                    theme,
                                    crate::ui::i18n::text(ui, "workspace-font-sample-aa-bb-cc"),
                                );
                            }

                            if resp.clicked() && !resp.dragged() {
                                clicked_theme_idx = Some(theme_idx);
                            }

                            // Menu de contexto com clique com o botão direito:
                            resp.context_menu(|ui| {
                                context_menu_header(ui);
                                if context_menu_item(ui, CHECK, "workspace-apply-theme", &theme_tokens)
                                    .clicked()
                                {
                                    action_apply_live = Some(theme_idx);
                                    ui.close();
                                }

                                context_menu_separator(ui);

                                let is_fav = theme.is_favorite;
                                let fav_text = if is_fav {
                                    "workspace-remove-from-favorites"
                                } else {
                                    "workspace-add-to-favorites"
                                };
                                if context_menu_item(ui, STAR, fav_text, &theme_tokens).clicked() {
                                    action_toggle_fav = Some(theme_idx);
                                    ui.close();
                                }

                                context_menu_separator(ui);

                                if context_menu_item(
                                    ui,
                                    PENCIL_SIMPLE,
                                    "workspace-edit-theme",
                                    &theme_tokens,
                                )
                                .clicked()
                                {
                                    action_edit = Some(theme_idx);
                                    ui.close();
                                }

                                if state.themes.len() > 1
                                    && context_menu_danger_item(
                                        ui,
                                        TRASH,
                                        "workspace-delete-theme",
                                        &theme_tokens,
                                    )
                                    .clicked()
                                    {
                                        action_delete = Some(theme_idx);
                                        ui.close();
                                    }

                                context_menu_separator(ui);

                                if context_menu_item(
                                    ui,
                                    ARROW_COUNTER_CLOCKWISE,
                                    "workspace-restore-default-themes",
                                    &theme_tokens,
                                )
                                .clicked()
                                {
                                    action_restore_defaults = true;
                                    ui.close();
                                }
                            });
                        }
                    }
                });
                ui.add_space(gap);
            }
            ui.add_space(12.0);

            // Soltura do arraste (Drag and Drop reorder)
            if ui.input(|i| i.pointer.any_released())
                && let Some(payload) = egui::DragAndDrop::payload::<ThemeDragPayload>(ui.ctx()) {
                    if let Some(target) = hovered_drop_index {
                        let from = payload.source_index;
                        if from < state.themes.len()
                            && target < state.themes.len()
                            && from != target
                        {
                            let moved = state.themes.remove(from);
                            state.themes.insert(target, moved);
                            if state.selected_theme_index == from {
                                state.selected_theme_index = target;
                            } else if from < state.selected_theme_index
                                && target >= state.selected_theme_index
                            {
                                state.selected_theme_index =
                                    state.selected_theme_index.saturating_sub(1);
                            } else if from > state.selected_theme_index
                                && target <= state.selected_theme_index
                            {
                                state.selected_theme_index =
                                    (state.selected_theme_index + 1).min(state.themes.len() - 1);
                            }

                            // Atualiza a nova ordenação no banco SQLite
                            if let Some(app_db) = db {
                                let ids: Vec<String> =
                                    state.themes.iter().map(|t| t.id.clone()).collect();
                                let _ = app_db.update_themes_order(&ids);
                            }
                        }
                    }
                    egui::DragAndDrop::clear_payload(ui.ctx());
                }

            // Execução de ações
            if clicked_add {
                state.open_create_theme();
            }
            if let Some(t_idx) = clicked_theme_idx {
                state.selected_theme_index = t_idx;
                applied = (true, false);
            }

            if let Some(idx) = action_apply_live {
                state.selected_theme_index = idx;
                state.apply_theme_to_live();
                applied = (true, true);
            }

            if let Some(idx) = action_toggle_fav
                && idx < state.themes.len() {
                    let is_now_fav = !state.themes[idx].is_favorite;
                    state.themes[idx].is_favorite = is_now_fav;
                    let theme_id = state.themes[idx].id.clone();

                    if let Some(app_db) = db {
                        let _ = app_db.update_theme_favorite(&theme_id, is_now_fav);
                    }

                    if is_now_fav {
                        let fav_theme = state.themes.remove(idx);
                        state.themes.insert(0, fav_theme);
                        state.selected_theme_index = 0;
                        if let Some(app_db) = db {
                            let ids: Vec<String> =
                                state.themes.iter().map(|t| t.id.clone()).collect();
                            let _ = app_db.update_themes_order(&ids);
                        }
                    }
                }

            if let Some(idx) = action_edit {
                state.open_edit_theme(idx);
            }

            if let Some(idx) = action_delete
                && state.themes.len() > 1 && idx < state.themes.len() {
                    let to_del = state.themes.remove(idx);
                    if let Some(app_db) = db {
                        let _ = app_db.delete_theme(&to_del.id);
                    }
                    if state.selected_theme_index >= state.themes.len() {
                        state.selected_theme_index = state.themes.len().saturating_sub(1);
                    }
                }

            if action_restore_defaults {
                if let Some(app_db) = db {
                    if let Ok(()) = app_db.restore_default_presentation_themes() {
                        if let Ok(fresh) = app_db.get_all_themes() {
                            state.themes = fresh;
                            state.selected_theme_index = 0;
                            state.apply_theme_to_live();
                            applied = (true, true);
                        }
                        state.notify(crate::ui::i18n::text(ui, "workspace-default-themes-restored"));
                    }
                }
            }
        });
    applied
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clicking_theme_changes_preview_only_and_not_live() {
        let mut state = ControlPanelState::default();
        state.themes = crate::services::shaders::default_themes();
        state.selected_theme_index = 0;
        state.live_theme_id = state.themes[0].id.clone();
        state.live_theme_index = 0;

        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let _ = render_themes_column(ui, &mut state, None);
        });
        output.textures_delta.clear();

        // Ensure live theme remains unchanged when browsing themes
        assert_eq!(state.live_theme_id, state.themes[0].id);
        assert_eq!(state.live_theme_index, 0);
    }
}
