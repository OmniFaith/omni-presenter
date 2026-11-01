use crate::ui::AppTheme;
use eframe::egui::{self, Color32, Stroke};

/// Containment keeps a row highlighted over its text and child action buttons.
/// Selected items still have distinct hover and pressed states.
pub(super) fn row(
    ui: &egui::Ui,
    response: &egui::Response,
    selected: bool,
    theme: &AppTheme,
) -> bool {
    let hovered = ui.is_enabled() && ui.rect_contains_pointer(response.rect);
    let time = ui.style().animation_time;
    let hover = ui
        .ctx()
        .animate_bool_with_time(response.id.with("hover"), hovered, time);
    let hold = ui.ctx().animate_bool_with_time(
        response.id.with("hold"),
        response.is_pointer_button_down_on(),
        time,
    );
    let fill = if selected {
        theme
            .accent_soft
            .lerp_to_gamma(theme.accent, hover * 0.12 + hold * 0.18)
    } else {
        Color32::TRANSPARENT
            .lerp_to_gamma(theme.bg_card_hover, hover)
            .lerp_to_gamma(theme.accent_soft, hold * 0.8)
    };
    ui.painter().rect(
        response.rect.shrink(hold * 0.4),
        8.0,
        fill,
        Stroke::new(1.0, theme.accent.gamma_multiply(hover * 0.35 + hold * 0.5)),
        egui::StrokeKind::Inside,
    );
    hovered
}

pub(super) fn thumbnail(
    ui: &egui::Ui,
    response: &egui::Response,
    selected: bool,
    theme: &AppTheme,
) {
    let hovered = ui.is_enabled() && ui.rect_contains_pointer(response.rect);
    let hover = ui.ctx().animate_bool_with_time(
        response.id.with("hover"),
        hovered,
        ui.style().animation_time,
    );
    let hold = ui.ctx().animate_bool_with_time(
        response.id.with("hold"),
        response.is_pointer_button_down_on(),
        ui.style().animation_time,
    );
    if hover > 0.0 || hold > 0.0 {
        ui.painter().rect_filled(
            response.rect,
            6.0,
            theme.accent.gamma_multiply(hover * 0.05 + hold * 0.12),
        );
    }
    ui.painter().rect_stroke(
        response.rect,
        6.0,
        Stroke::new(
            1.0 + hold,
            theme.accent.gamma_multiply(if selected {
                0.8
            } else {
                hover * 0.65 + hold * 0.35
            }),
        ),
        egui::StrokeKind::Inside,
    );
}
