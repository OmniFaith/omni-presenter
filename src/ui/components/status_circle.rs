use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, Painter, Pos2, Stroke, Vec2};

/// Paints a modern circular checkbox with fill and crisp inner checkmark
pub fn paint_status_circle(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    played: bool,
    hovered: bool,
    theme: &AppTheme,
) {
    if played {
        // Checked: container fill in gray according to theme (theme.text_muted) to signify completed/inactive,
        // or accent color on hover, with crisp white checkmark inside
        let (fill_col, stroke_col) = if hovered {
            (theme.accent, theme.accent)
        } else {
            (theme.text_muted, theme.text_muted)
        };
        painter.circle_filled(center, radius, fill_col);
        painter.circle_stroke(center, radius, Stroke::new(1.0, stroke_col));

        let check_scale = radius / 8.5;
        let check_stroke = Stroke::new((1.6 * check_scale).max(1.2), Color32::WHITE);
        painter.line_segment(
            [
                Pos2::new(center.x - 3.2 * check_scale, center.y + 0.2 * check_scale),
                Pos2::new(center.x - 0.7 * check_scale, center.y + 2.7 * check_scale),
            ],
            check_stroke,
        );
        painter.line_segment(
            [
                Pos2::new(center.x - 0.7 * check_scale, center.y + 2.7 * check_scale),
                Pos2::new(center.x + 3.5 * check_scale, center.y - 2.0 * check_scale),
            ],
            check_stroke,
        );
    } else {
        // Unchecked: subtle translucent container fill with fine border stroke
        let (fill_col, stroke_col) = if hovered {
            (theme.bg_card_hover, theme.accent)
        } else {
            (
                Color32::from_rgba_unmultiplied(128, 128, 128, 25),
                Color32::from_rgba_unmultiplied(140, 140, 140, 95),
            )
        };
        painter.circle_filled(center, radius, fill_col);
        painter.circle_stroke(center, radius, Stroke::new(1.2, stroke_col));
    }
}

/// Renders the standalone status indicator circle button (played vs pending)
pub fn render_status_circle(ui: &mut egui::Ui, played: bool, theme: &AppTheme) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(16.0), egui::Sense::click());
    let resp = resp.on_hover_text(if played {
        crate::ui::i18n::text(ui, "workspace-status-already-played-in-this-service-click-to-toggle")
    } else {
        crate::ui::i18n::text(ui, "workspace-status-not-played-yet-click-to-toggle")
    });
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    paint_status_circle(ui.painter(), rect.center(), 6.5, played, resp.hovered(), theme);
    resp
}
