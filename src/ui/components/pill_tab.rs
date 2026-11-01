use crate::ui::theme::AppTheme;
use eframe::egui::{self, Stroke, Vec2};

/// Renders a horizontal bar of modern rounded pill tabs, centered in available width
pub fn render_pill_tabs(
    ui: &mut egui::Ui,
    selected_idx: usize,
    tabs: &[(&str, &str)],
    theme: &AppTheme,
) -> Option<usize> {
    ui.scope(|ui| {
        let mut newly_selected = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(5.0, 5.0);

            for (tab_idx, (icon, label)) in tabs.iter().enumerate() {
                let is_sel = selected_idx == tab_idx;
                let id = ui.make_persistent_id(("pill", tab_idx));

                let dummy_text = format!("{} {}", icon, label);
                let text_font = egui::FontId::proportional(11.5);
                let galley = ui.painter().layout_no_wrap(
                    dummy_text,
                    text_font,
                    theme.text_primary,
                );
                let pad_x = 9.0;
                let min_w = (galley.size().x + pad_x * 2.0).max(50.0);
                let (rect, resp) = ui.allocate_exact_size(
                    Vec2::new(min_w, 25.0),
                    egui::Sense::click(),
                );

                let hover = ui.ctx().animate_bool(id.with("h"), resp.hovered());
                let down = ui.ctx().animate_bool(id.with("d"), resp.is_pointer_button_down_on());
                let selected = ui.ctx().animate_bool(id.with("s"), is_sel);
                let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);

                let is_pressed = resp.is_pointer_button_down_on();

                let bg = if is_sel {
                    if down > 0.05 {
                        theme.accent_soft.lerp_to_gamma(theme.bg_card_active, down * 0.35)
                    } else {
                        theme.bg_pill.lerp_to_gamma(theme.accent_soft, selected)
                    }
                } else if down > 0.05 {
                    theme.bg_card_hover.lerp_to_gamma(theme.accent_soft, down * 0.45)
                } else if hover > 0.05 {
                    theme.bg_pill.lerp_to_gamma(theme.bg_card_hover, hover)
                } else {
                    theme.bg_pill
                };

                let stroke = if is_sel {
                    if down > 0.05 {
                        Stroke::new(1.3, theme.accent)
                    } else {
                        Stroke::new(1.0, theme.accent.linear_multiply(0.65))
                    }
                } else if down > 0.05 {
                    Stroke::new(1.2, theme.accent.linear_multiply(0.85))
                } else if hover > 0.05 {
                    Stroke::new(
                        1.0,
                        theme.border_default.lerp_to_gamma(theme.accent.linear_multiply(0.5), hover),
                    )
                } else {
                    Stroke::new(1.0, theme.border_default)
                };

                let draw_rect = rect.shrink(down * 0.75);
                ui.painter().rect(draw_rect, theme.radius_sm, bg, stroke, egui::StrokeKind::Inside);

                let text_color = if is_sel || is_pressed {
                    theme.accent
                } else if hover > 0.5 {
                    theme.text_primary
                } else {
                    theme.icon_default
                };

                let text_g = ui.painter().layout_no_wrap(
                    format!("{} {}", icon, label),
                    if is_sel || is_pressed {
                        egui::FontId::proportional(11.8)
                    } else {
                        egui::FontId::proportional(11.5)
                    },
                    text_color,
                );
                ui.painter().galley(
                    egui::pos2(
                        draw_rect.center().x - text_g.size().x / 2.0,
                        draw_rect.center().y - text_g.size().y / 2.0,
                    ),
                    text_g,
                    text_color,
                );

                if resp.clicked() {
                    newly_selected = Some(tab_idx);
                }
            }
        });
        newly_selected
    })
    .inner
}
