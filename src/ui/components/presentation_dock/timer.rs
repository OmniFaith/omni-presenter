use crate::core::i18n::Language;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, FontId, Stroke, StrokeKind, Vec2};
use egui_phosphor::regular::TIMER;

use super::BTN_H;

/// Compact pill showing elapsed presentation time with live tick, formatting toggle and reset.
pub(crate) fn presentation_timer_badge(
    ui: &mut egui::Ui,
    tokens: &AppTheme,
    language: Language,
    started_at: &mut Option<std::time::Instant>,
    seconds_mode: &mut bool,
) {
    let elapsed_secs = started_at.map(|t| t.elapsed().as_secs()).unwrap_or(0);
    let hours = elapsed_secs / 3600;
    let mins = (elapsed_secs % 3600) / 60;
    let secs = elapsed_secs % 60;

    let time_str_mmss = if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{:02}:{:02}", mins, secs)
    };

    let time_display = if *seconds_mode {
        format!("{}s", elapsed_secs)
    } else {
        time_str_mmss.clone()
    };

    let font = FontId::monospace(11.5);
    let text_w = ui
        .painter()
        .layout_no_wrap(time_display.clone(), font.clone(), tokens.text_primary)
        .size()
        .x;

    let pad_x = 8.0;
    let icon_w = 14.0;
    let gap = 5.0;
    let total_w = pad_x + icon_w + gap + text_w + pad_x;

    let (rect, response) = ui.allocate_exact_size(Vec2::new(total_w, BTN_H), egui::Sense::click());

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    if response.double_clicked() {
        *started_at = Some(std::time::Instant::now());
    } else if response.clicked() {
        *seconds_mode = !*seconds_mode;
    }

    let bg_color = if response.hovered() {
        tokens.bg_pill.lerp_to_gamma(tokens.bg_pill_active, 0.7)
    } else {
        tokens.bg_pill
    };

    let stroke_color = if response.hovered() {
        tokens.accent.linear_multiply(0.5)
    } else {
        tokens.border_default
    };

    ui.painter()
        .rect_filled(rect, tokens.radius_md, bg_color);
    ui.painter().rect_stroke(
        rect,
        tokens.radius_md,
        Stroke::new(1.0, stroke_color),
        StrokeKind::Inside,
    );

    // Draw TIMER icon
    let icon_rect = egui::Rect::from_min_size(
        egui::pos2(rect.min.x + pad_x, rect.min.y),
        Vec2::new(icon_w, BTN_H),
    );
    ui.painter().text(
        icon_rect.center(),
        egui::Align2::CENTER_CENTER,
        TIMER,
        FontId::proportional(13.0),
        tokens.accent,
    );

    // Draw time text
    let text_pos = egui::pos2(rect.min.x + pad_x + icon_w + gap, rect.center().y);
    ui.painter().text(
        text_pos,
        egui::Align2::LEFT_CENTER,
        time_display,
        font,
        tokens.text_primary,
    );

    response
        .on_hover_ui(|ui| {
            ui.label(
                egui::RichText::new(language.text("control-timer-title"))
                    .strong()
                    .size(12.0)
                    .color(tokens.text_primary),
            );
            ui.label(
                egui::RichText::new(language.format(
                    "control-timer-elapsed",
                    &[
                        ("seconds", &elapsed_secs.to_string()),
                        ("formatted", &time_str_mmss),
                    ],
                ))
                .size(11.5)
                .color(tokens.text_secondary),
            );
            ui.add_space(3.0);
            ui.label(
                egui::RichText::new(language.text("control-timer-click-format"))
                    .size(10.5)
                    .color(tokens.text_muted),
            );
            ui.label(
                egui::RichText::new(language.text("control-timer-double-click-reset"))
                    .size(10.5)
                    .color(tokens.text_muted),
            );
        })
        .context_menu(|ui| {
            if ui
                .button(format!("{TIMER} {}", language.text("control-timer-reset")))
                .clicked()
            {
                *started_at = Some(std::time::Instant::now());
                ui.close();
            }
            if ui
                .button(if *seconds_mode {
                    language.text("control-timer-to-mmss")
                } else {
                    language.text("control-timer-to-seconds")
                })
                .clicked()
            {
                *seconds_mode = !*seconds_mode;
                ui.close();
            }
        });
}
