use crate::ui::components::{
    context_menu_danger_item, context_menu_disabled_item, context_menu_header, context_menu_item,
    context_menu_separator,
};
use crate::ui::AppTheme;
use eframe::egui::{self, Vec2};
use egui_phosphor::regular::{ARROW_DOWN, ARROW_UP, DROP, PENCIL_SIMPLE, PLUS, ROWS, TRASH};

pub const SEPARATOR_PALETTE: &[(&str, &str)] = &[
    ("#3B82F6", "Azul"),
    ("#6366F1", "Índigo"),
    ("#8B5CF6", "Roxo"),
    ("#EC4899", "Rosa"),
    ("#EF4444", "Vermelho"),
    ("#F97316", "Laranja"),
    ("#EAB308", "Amarelo"),
    ("#10B981", "Verde"),
    ("#14B8A6", "Teal"),
    ("#06B6D4", "Ciano"),
    ("#64748B", "Ardósia"),
    ("#374151", "Escuro"),
];

pub fn parse_hex_color(hex: &str, fallback: egui::Color32) -> egui::Color32 {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
        ) {
            return egui::Color32::from_rgb(r, g, b);
        }
    } else if clean.len() == 8 {
        if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
            u8::from_str_radix(&clean[6..8], 16),
        ) {
            return egui::Color32::from_rgba_premultiplied(r, g, b, a);
        }
    }
    fallback
}

pub fn text_color_for_bg(bg: egui::Color32) -> egui::Color32 {
    let lum = 0.299 * (bg.r() as f32) + 0.587 * (bg.g() as f32) + 0.114 * (bg.b() as f32);
    if lum > 150.0 {
        egui::Color32::from_rgb(20, 24, 33)
    } else {
        egui::Color32::WHITE
    }
}

pub enum SeparatorAction {
    None,
    StartRename(String),
    SaveTitle(String, String),
    CancelRename,
    SetColor(String, String),
    MoveUp(usize),
    MoveDown(usize),
    AddSeparatorAfter(usize),
    Remove(usize),
}

pub fn render_separator_item(
    ui: &mut egui::Ui,
    item: &crate::core::models::PlaylistItem,
    idx: usize,
    total_items: usize,
    is_selected: bool,
    is_renaming: bool,
    drag_ids: Vec<String>,
    theme: &AppTheme,
) -> (SeparatorAction, bool, bool, Option<(Vec<String>, String, bool)>) {
    let row_w = ui.available_width();
    let row_h = 28.0;
    let base_color = parse_hex_color(&item.content, theme.accent);

    // Subtle transparent tint for badge background (alpha ~38)
    let pill_bg = egui::Color32::from_rgba_unmultiplied(
        base_color.r(),
        base_color.g(),
        base_color.b(),
        38,
    );
    // Soft transparent gray border suitable for both light and dark themes
    let default_border = egui::Color32::from_rgba_unmultiplied(140, 140, 140, 50);
    let line_color = egui::Color32::from_rgba_unmultiplied(
        base_color.r(),
        base_color.g(),
        base_color.b(),
        85,
    );
    let line_stroke = egui::Stroke::new(1.0, line_color);

    if is_renaming {
        let (rect, _resp) = ui.allocate_exact_size(Vec2::new(row_w, row_h), egui::Sense::hover());
        let center = rect.center();
        let edit_w = (row_w * 0.55).clamp(140.0, 260.0);
        let pill_rect = egui::Rect::from_center_size(center, Vec2::new(edit_w, 22.0));

        // Flanking lines for the inline editor
        if pill_rect.min.x > rect.min.x + 14.0 {
            ui.painter().line_segment(
                [egui::pos2(rect.min.x + 8.0, center.y), egui::pos2(pill_rect.min.x - 4.0, center.y)],
                line_stroke,
            );
        }
        if pill_rect.max.x < rect.max.x - 14.0 {
            ui.painter().line_segment(
                [egui::pos2(pill_rect.max.x + 4.0, center.y), egui::pos2(rect.max.x - 8.0, center.y)],
                line_stroke,
            );
        }

        // Inline edit background & focus border
        let edit_border = egui::Color32::from_rgba_unmultiplied(
            base_color.r(),
            base_color.g(),
            base_color.b(),
            180,
        );
        ui.painter().rect_filled(pill_rect, 11.0, pill_bg);
        ui.painter().rect_stroke(pill_rect, 11.0, egui::Stroke::new(1.0, edit_border), egui::StrokeKind::Inside);

        let edit_id = ui.scope_id().with(("sep_inline_edit", &item.id));
        let mut temp_text = ui.ctx().data_mut(|d| {
            d.get_temp::<String>(edit_id).unwrap_or_else(|| item.title.clone())
        });

        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(pill_rect.shrink2(Vec2::new(8.0, 1.0)))
                .layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight)),
        );

        let edit_resp = child_ui.add(
            egui::TextEdit::singleline(&mut temp_text)
                .font(egui::FontId::proportional(12.0))
                .text_color(theme.text_primary)
                .frame(egui::Frame::NONE)
                .hint_text(crate::ui::i18n::text(ui, "workspace-separator-name-placeholder")),
        );

        let focus_key = edit_id.with("focus_requested");
        if ui.ctx().data_mut(|d| d.get_temp::<bool>(focus_key)).is_none() {
            edit_resp.request_focus();
            ui.ctx().data_mut(|d| d.insert_temp(focus_key, true));
        }

        let mut action = SeparatorAction::None;
        if ui.input(|i| i.key_pressed(egui::Key::Enter))
            || (edit_resp.lost_focus() && !edit_resp.gained_focus())
        {
            action = SeparatorAction::SaveTitle(item.id.clone(), temp_text.trim().to_string());
            ui.ctx().data_mut(|d| {
                d.remove::<String>(edit_id);
                d.remove::<bool>(focus_key);
            });
        } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            action = SeparatorAction::CancelRename;
            ui.ctx().data_mut(|d| {
                d.remove::<String>(edit_id);
                d.remove::<bool>(focus_key);
            });
        } else {
            ui.ctx().data_mut(|d| d.insert_temp(edit_id, temp_text));
        }

        return (action, false, false, None);
    }

    let (rect, resp) =
        ui.allocate_exact_size(Vec2::new(row_w, row_h), egui::Sense::click_and_drag());

    let border_color = if is_selected {
        egui::Color32::from_rgba_unmultiplied(base_color.r(), base_color.g(), base_color.b(), 180)
    } else if resp.hovered() {
        egui::Color32::from_rgba_unmultiplied(160, 160, 160, 90)
    } else {
        default_border
    };

    let title_trimmed = item.title.trim();
    let center = rect.center();

    if title_trimmed.is_empty() {
        // Sem texto: linha de fora a fora contínua
        let line_w = if is_selected { 2.0 } else if resp.hovered() { 1.5 } else { 1.0 };
        let full_stroke = egui::Stroke::new(line_w, if is_selected { base_color } else { line_color });
        ui.painter().line_segment(
            [egui::pos2(rect.min.x + 8.0, center.y), egui::pos2(rect.max.x - 8.0, center.y)],
            full_stroke,
        );
    } else {
        // Com texto: texto centralizado com linhas na esquerda e na direita
        let font_id = egui::FontId::proportional(12.0);
        let galley = ui.painter().layout_no_wrap(title_trimmed.to_string(), font_id, theme.text_primary);
        let text_w = galley.size().x;
        let text_h = galley.size().y;
        let pill_w = (text_w + 24.0).min(row_w - 32.0);
        let pill_h = 22.0;
        let pill_rect = egui::Rect::from_center_size(center, Vec2::new(pill_w, pill_h));

        // Linha esquerda
        if pill_rect.min.x > rect.min.x + 14.0 {
            ui.painter().line_segment(
                [egui::pos2(rect.min.x + 8.0, center.y), egui::pos2(pill_rect.min.x - 4.0, center.y)],
                line_stroke,
            );
        }
        // Linha direita
        if pill_rect.max.x < rect.max.x - 14.0 {
            ui.painter().line_segment(
                [egui::pos2(pill_rect.max.x + 4.0, center.y), egui::pos2(rect.max.x - 8.0, center.y)],
                line_stroke,
            );
        }

        // Badge transparente com borda cinza suave
        ui.painter().rect_filled(pill_rect, 11.0, pill_bg);
        ui.painter().rect_stroke(pill_rect, 11.0, egui::Stroke::new(1.0, border_color), egui::StrokeKind::Inside);

        // Texto perfeitamente centralizado horizontalmente e verticalmente
        ui.painter().galley(
            egui::pos2(center.x - text_w / 2.0, center.y - text_h / 2.0),
            galley,
            theme.text_primary,
        );
    }

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let resp = if title_trimmed.is_empty() {
        resp.on_hover_text(crate::ui::i18n::text(ui, "workspace-new-separator"))
    } else {
        resp.on_hover_text(&item.title)
    };

    let drag_started = resp.drag_started();
    let clicked = resp.clicked();

    let mut action = SeparatorAction::None;
    if resp.double_clicked() {
        action = SeparatorAction::StartRename(item.id.clone());
    }

    crate::ui::components::drag_drop::source(ui, &resp, || {
        use crate::ui::components::drag_drop::{DragPayload, DragSource};
        DragPayload::new(DragSource::Playlist(drag_ids.clone()), &item.title, "", ROWS)
    });

    let mut reorder = None;
    if let Some((payload, after)) = crate::ui::components::drag_drop::reorder_target(
        ui,
        resp.rect,
        theme,
        false,
        |source| matches!(source, crate::ui::components::drag_drop::DragSource::Playlist(ids) if !ids.contains(&item.id)),
    ) && let crate::ui::components::drag_drop::DragSource::Playlist(ids) = &payload.source {
        reorder = Some((ids.clone(), item.id.clone(), after));
    }

    resp.context_menu(|ui| {
        context_menu_header(ui);

        if context_menu_item(ui, PENCIL_SIMPLE, "workspace-rename-separator", theme).clicked() {
            action = SeparatorAction::StartRename(item.id.clone());
            ui.close();
        }

        context_menu_separator(ui);

        // Seção de cor embutida diretamente no menu de contexto (sem submenus flutuantes que quebram a borda)
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            let icon_galley = ui.painter().layout_no_wrap(
                DROP.to_string(),
                egui::FontId::proportional(13.0),
                theme.icon_default,
            );
            let drop_y = ui.cursor().center().y - icon_galley.size().y / 2.0;
            ui.painter().galley(egui::pos2(ui.cursor().min.x, drop_y), icon_galley, theme.icon_default);
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new(crate::ui::i18n::text(ui, "workspace-change-color"))
                    .size(11.5)
                    .color(theme.text_secondary),
            );
        });
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(6.0, 5.0);
                // Grade de cores linha 1
                ui.horizontal(|ui| {
                    for (hex, _label) in &SEPARATOR_PALETTE[..6] {
                        let col = parse_hex_color(hex, theme.accent);
                        let is_current = item.content.eq_ignore_ascii_case(hex);
                        let (c_rect, c_resp) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::click());
                        if c_resp.clicked() {
                            action = SeparatorAction::SetColor(item.id.clone(), hex.to_string());
                            ui.close();
                        }
                        if c_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        ui.painter().circle_filled(c_rect.center(), 8.0, col);
                        if is_current {
                            ui.painter().circle_stroke(c_rect.center(), 9.5, egui::Stroke::new(2.0, theme.accent));
                            ui.painter().circle_filled(c_rect.center(), 2.5, text_color_for_bg(col));
                        } else if c_resp.hovered() {
                            ui.painter().circle_stroke(c_rect.center(), 9.5, egui::Stroke::new(1.5, theme.text_primary));
                        }
                    }
                });
                // Grade de cores linha 2
                ui.horizontal(|ui| {
                    for (hex, _label) in &SEPARATOR_PALETTE[6..12] {
                        let col = parse_hex_color(hex, theme.accent);
                        let is_current = item.content.eq_ignore_ascii_case(hex);
                        let (c_rect, c_resp) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::click());
                        if c_resp.clicked() {
                            action = SeparatorAction::SetColor(item.id.clone(), hex.to_string());
                            ui.close();
                        }
                        if c_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        ui.painter().circle_filled(c_rect.center(), 8.0, col);
                        if is_current {
                            ui.painter().circle_stroke(c_rect.center(), 9.5, egui::Stroke::new(2.0, theme.accent));
                            ui.painter().circle_filled(c_rect.center(), 2.5, text_color_for_bg(col));
                        } else if c_resp.hovered() {
                            ui.painter().circle_stroke(c_rect.center(), 9.5, egui::Stroke::new(1.5, theme.text_primary));
                        }
                    }
                });
                // Cor personalizada
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(crate::ui::i18n::text(ui, "workspace-custom-color"))
                            .size(11.0)
                            .color(theme.text_secondary),
                    );
                    let cur = parse_hex_color(&item.content, theme.accent);
                    let mut rgb = [cur.r(), cur.g(), cur.b()];
                    if ui.color_edit_button_srgb(&mut rgb).changed() {
                        action = SeparatorAction::SetColor(
                            item.id.clone(),
                            format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]),
                        );
                    }
                });
            });
        });

        context_menu_separator(ui);

        if idx > 0 {
            if context_menu_item(ui, ARROW_UP, "workspace-move-up", theme).clicked() {
                action = SeparatorAction::MoveUp(idx);
                ui.close();
            }
        } else {
            context_menu_disabled_item(ui, ARROW_UP, "workspace-move-up", theme);
        }

        if idx + 1 < total_items {
            if context_menu_item(ui, ARROW_DOWN, "workspace-move-down", theme).clicked() {
                action = SeparatorAction::MoveDown(idx);
                ui.close();
            }
        } else {
            context_menu_disabled_item(ui, ARROW_DOWN, "workspace-move-down", theme);
        }

        context_menu_separator(ui);

        if context_menu_item(ui, PLUS, "workspace-add-separator", theme).clicked() {
            action = SeparatorAction::AddSeparatorAfter(idx);
            ui.close();
        }

        context_menu_separator(ui);

        if context_menu_danger_item(ui, TRASH, "workspace-remove-separator", theme).clicked() {
            action = SeparatorAction::Remove(idx);
            ui.close();
        }
    });

    (action, clicked, drag_started, reorder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{PlaylistItem, PlaylistItemKind};
    use crate::core::slides::SlideDeck;

    #[test]
    fn test_parse_hex_color_valid_and_fallback() {
        let fallback = egui::Color32::from_rgb(10, 20, 30);
        assert_eq!(
            parse_hex_color("#3B82F6", fallback),
            egui::Color32::from_rgb(0x3B, 0x82, 0xF6)
        );
        assert_eq!(
            parse_hex_color("EF4444", fallback),
            egui::Color32::from_rgb(0xEF, 0x44, 0x44)
        );
        assert_eq!(parse_hex_color("invalid", fallback), fallback);
    }

    #[test]
    fn test_text_color_contrast() {
        let yellow = egui::Color32::from_rgb(234, 179, 8);
        assert_eq!(text_color_for_bg(yellow), egui::Color32::from_rgb(20, 24, 33));

        let dark = egui::Color32::from_rgb(30, 41, 59);
        assert_eq!(text_color_for_bg(dark), egui::Color32::WHITE);
    }

    #[test]
    fn test_separator_preview_does_not_fail() {
        let mut deck = SlideDeck::new("");
        let sep = PlaylistItem {
            id: "sep-1".into(),
            title: "MOMENTO DE LOUVOR".into(),
            subtitle: String::new(),
            kind: PlaylistItemKind::Separator,
            played: false,
            content: "#3B82F6".into(),
        };
        assert!(sep.preview(&mut deck).is_ok());
    }

    #[test]
    fn test_separator_empty_title_allowed() {
        let mut deck = SlideDeck::new("");
        let sep = PlaylistItem {
            id: "sep-empty".into(),
            title: String::new(),
            subtitle: String::new(),
            kind: PlaylistItemKind::Separator,
            played: false,
            content: "#10B981".into(),
        };
        assert!(sep.preview(&mut deck).is_ok());
        assert!(sep.title.is_empty());
    }
}
