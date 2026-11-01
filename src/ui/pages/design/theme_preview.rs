use crate::{
    core::models::PresentationTheme,
    ui::theme::AppTheme,
};
use eframe::egui::{self, Color32, RichText};
use egui_phosphor::regular::*;

pub fn show_dropdown(
    response: &egui::Response,
    popup_id: egui::Id,
    preview_theme_id: &mut Option<String>,
    theme_search: &mut String,
    themes: &[PresentationTheme],
    tokens: &AppTheme,
) {
    egui::Popup::menu(response)
        .id(popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            let popup_w = 264.0;
            ui.set_width(popup_w);
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);

            // Campo de busca compacto para acomodar muitos temas
            ui.horizontal(|ui| {
                ui.label(RichText::new(MAGNIFYING_GLASS).size(13.0).color(tokens.text_muted));
                ui.add(
                    egui::TextEdit::singleline(theme_search)
                        .hint_text(crate::ui::i18n::text(ui, "design-search-theme"))
                        .desired_width(f32::INFINITY)
                        .margin(egui::vec2(6.0, 4.0)),
                );
                if !theme_search.is_empty() && ui.small_button(X).clicked() {
                    theme_search.clear();
                }
            });

            ui.add_space(2.0);

            let search = theme_search.trim().to_lowercase();
            let show_remove = search.is_empty()
                || "remover".contains(&search)
                || "limpar".contains(&search)
                || "sem".contains(&search)
                || "desativar".contains(&search);

            let filtered_themes: Vec<_> = themes
                .iter()
                .filter(|t| {
                    search.is_empty()
                        || t.name.to_lowercase().contains(&search)
                        || crate::ui::i18n::text(ui, &t.name).to_lowercase().contains(&search)
                })
                .collect();

            if !show_remove && filtered_themes.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(14.0);
                    ui.label(RichText::new(crate::ui::i18n::text(ui, "design-no-themes-found")).size(11.0).color(tokens.text_muted));
                    ui.add_space(14.0);
                });
                return;
            }

            egui::ScrollArea::vertical()
                .id_salt("design_theme_preview_scroll")
                .max_height(280.0)
                .show(ui, |ui| {
                    let card_w = 126.0;
                    let card_h = 70.0;
                    let columns = 2;

                    egui::Grid::new("design_theme_preview_grid")
                        .spacing(egui::vec2(6.0, 6.0))
                        .show(ui, |ui| {
                            let mut col = 0;

                            // 1. Opção Remover: no mesmo estilo visual de retângulo dos temas
                            if show_remove {
                                let is_selected = preview_theme_id.is_none();
                                let (rect, resp) = ui.allocate_exact_size(
                                    egui::vec2(card_w, card_h),
                                    egui::Sense::click(),
                                );

                                let painter = ui.painter();
                                painter.rect_filled(rect, 6.0, Color32::from_rgb(22, 25, 32));

                                // Mini padrão xadrez na área superior simulando o fundo transparente
                                let top_area = egui::Rect::from_min_max(
                                    rect.min,
                                    egui::pos2(rect.max.x, rect.max.y - 20.0),
                                );
                                let tile = 8.0;
                                for y in 0..(top_area.height() / tile) as usize {
                                    for x in 0..(top_area.width() / tile) as usize {
                                        if (x + y) % 2 == 0 {
                                            painter.rect_filled(
                                                egui::Rect::from_min_size(
                                                    top_area.min + egui::vec2(x as f32 * tile, y as f32 * tile),
                                                    egui::vec2(tile, tile),
                                                ),
                                                0.0,
                                                Color32::from_rgb(32, 36, 46),
                                            );
                                        }
                                    }
                                }

                                // Ícone de remover centralizado
                                let icon_pos = egui::pos2(rect.center().x, top_area.center().y);
                                painter.text(
                                    icon_pos,
                                    egui::Align2::CENTER_CENTER,
                                    PROHIBIT,
                                    egui::FontId::proportional(20.0),
                                    if resp.hovered() {
                                        tokens.status_danger
                                    } else {
                                        tokens.text_muted
                                    },
                                );

                                // Barra inferior idêntica à dos temas
                                let name_bar = egui::Rect::from_min_max(
                                    egui::pos2(rect.min.x, rect.max.y - 20.0),
                                    rect.max,
                                );
                                painter.rect_filled(
                                    name_bar,
                                    egui::CornerRadius { nw: 0, ne: 0, sw: 6, se: 6 },
                                    Color32::from_black_alpha(160),
                                );
                                painter.text(
                                    name_bar.center(),
                                    egui::Align2::CENTER_CENTER,
                                    crate::ui::i18n::text(ui, "design-remove-preview-theme"),
                                    egui::FontId::proportional(9.5),
                                    Color32::WHITE,
                                );

                                // Contorno de seleção ou hover
                                let stroke = if is_selected {
                                    Some(egui::Stroke::new(2.0, tokens.accent))
                                } else if resp.hovered() {
                                    Some(egui::Stroke::new(1.5, tokens.border_active))
                                } else {
                                    Some(egui::Stroke::new(1.0, tokens.border_subtle))
                                };
                                if let Some(stroke) = stroke {
                                    painter.rect_stroke(rect, 6.0, stroke, egui::StrokeKind::Inside);
                                }

                                if is_selected {
                                    let check_pos = egui::pos2(rect.max.x - 10.0, rect.min.y + 10.0);
                                    painter.circle_filled(check_pos, 7.0, tokens.accent);
                                    painter.text(
                                        check_pos,
                                        egui::Align2::CENTER_CENTER,
                                        CHECK,
                                        egui::FontId::proportional(9.0),
                                        tokens.text_on_accent,
                                    );
                                }

                                if resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if resp.clicked() {
                                    *preview_theme_id = None;
                                    ui.close();
                                }

                                col += 1;
                                if col % columns == 0 {
                                    ui.end_row();
                                }
                            }

                            // 2. Retângulos dos temas
                            for item in filtered_themes {
                                let is_selected = preview_theme_id.as_deref() == Some(&item.id);
                                let (rect, resp) = ui.allocate_exact_size(
                                    egui::vec2(card_w, card_h),
                                    egui::Sense::click(),
                                );

                                crate::render::paint_theme_bg(ui.painter(), rect, item, 6.0);

                                let text_rect = egui::Rect::from_min_max(
                                    rect.min,
                                    egui::pos2(rect.max.x, rect.max.y - 20.0),
                                );
                                if text_rect.height() > 16.0 {
                                    crate::render::render_theme_lyrics(
                                        ui.painter(),
                                        text_rect,
                                        item,
                                        "Aa",
                                    );
                                }

                                let name_bar = egui::Rect::from_min_max(
                                    egui::pos2(rect.min.x, rect.max.y - 20.0),
                                    rect.max,
                                );
                                ui.painter().rect_filled(
                                    name_bar,
                                    egui::CornerRadius { nw: 0, ne: 0, sw: 6, se: 6 },
                                    Color32::from_black_alpha(160),
                                );
                                ui.painter().text(
                                    name_bar.center(),
                                    egui::Align2::CENTER_CENTER,
                                    crate::ui::i18n::text(ui, &item.name),
                                    egui::FontId::proportional(9.5),
                                    Color32::WHITE,
                                );

                                let stroke = if is_selected {
                                    Some(egui::Stroke::new(2.0, tokens.accent))
                                } else if resp.hovered() {
                                    Some(egui::Stroke::new(1.5, tokens.border_active))
                                } else {
                                    None
                                };
                                if let Some(stroke) = stroke {
                                    ui.painter().rect_stroke(rect, 6.0, stroke, egui::StrokeKind::Inside);
                                }

                                if is_selected {
                                    let check_pos = egui::pos2(rect.max.x - 10.0, rect.min.y + 10.0);
                                    ui.painter().circle_filled(check_pos, 7.0, tokens.accent);
                                    ui.painter().text(
                                        check_pos,
                                        egui::Align2::CENTER_CENTER,
                                        CHECK,
                                        egui::FontId::proportional(9.0),
                                        tokens.text_on_accent,
                                    );
                                }

                                if resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if resp.clicked() {
                                    *preview_theme_id = Some(item.id.clone());
                                    ui.close();
                                }

                                col += 1;
                                if col % columns == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                });
        });
}
