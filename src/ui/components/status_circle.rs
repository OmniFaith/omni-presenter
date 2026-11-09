use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, Stroke};
use egui_phosphor::regular::{CHECK_CIRCLE, CIRCLE};

/// Renders the status indicator circle (played vs pending)
pub fn render_status_circle(ui: &mut egui::Ui, played: bool, theme: &AppTheme) -> egui::Response {
    let (icon, color) = if played {
        (CHECK_CIRCLE, theme.accent)
    } else {
        (CIRCLE, theme.text_muted)
    };

    let btn = egui::Button::new(egui::RichText::new(icon).size(16.0).color(color))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::NONE);

    let resp = ui.add(btn).on_hover_text(if played {
        crate::ui::i18n::text(ui, "workspace-status-already-played-in-this-service-click-to-toggle")
    } else {
        crate::ui::i18n::text(ui, "workspace-status-not-played-yet-click-to-toggle")
    });
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}
