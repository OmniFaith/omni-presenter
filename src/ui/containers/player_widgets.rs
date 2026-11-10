use crate::core::i18n::Language;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, Vec2};
use egui_phosphor::regular::{PAUSE, PLAY};

pub fn format_time(secs: f32) -> String {
    if secs.is_nan() || secs < 0.0 {
        return "00:00".to_string();
    }
    let total_secs = secs as u64;
    let m = total_secs / 60;
    let s = total_secs % 60;
    format!("{m:02}:{s:02}")
}

pub fn render_custom_scrubber(
    ui: &mut egui::Ui,
    cur_pos: f32,
    total_dur: f32,
    desired_w: f32,
    theme: &AppTheme,
    language: Language,
) -> Option<f32> {
    let desired_h = 14.0;
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(desired_w, desired_h),
        egui::Sense::click_and_drag(),
    );

    let painter = ui.painter();
    let is_hovered = resp.hovered();
    let is_dragged = resp.dragged();

    if is_hovered || is_dragged {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let progress_ratio = if is_dragged && let Some(mouse_pos) = resp.interact_pointer_pos() {
        ((mouse_pos.x - rect.min.x) / rect.width().max(1.0)).clamp(0.0, 1.0)
    } else if total_dur > 0.0 {
        (cur_pos / total_dur).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let center_y = rect.center().y;
    let rail_h = if is_hovered || is_dragged { 4.5 } else { 3.0 };
    let rail_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(rect.width(), rail_h));
    painter.rect_filled(rail_rect, rail_h * 0.5, theme.border_default);

    // Trilha preenchida com a cor de destaque
    let fill_x = rect.min.x + (rect.width() * progress_ratio);
    if fill_x > rect.min.x {
        let fill_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x, center_y - rail_h * 0.5),
            egui::pos2(fill_x, center_y + rail_h * 0.5),
        );
        painter.rect_filled(fill_rect, rail_h * 0.5, theme.accent);
    }

    // Cursor / Knob estilizado
    let knob_radius = if is_hovered || is_dragged { 5.0 } else { 3.5 };
    let knob_pos = egui::pos2(fill_x.clamp(rect.min.x, rect.max.x), center_y);

    if is_hovered || is_dragged {
        painter.circle_filled(knob_pos, knob_radius + 2.0, theme.accent_soft);
    }

    painter.circle_filled(knob_pos, knob_radius, Color32::WHITE);
    painter.circle_stroke(knob_pos, knob_radius, egui::Stroke::new(1.5, theme.accent));

    let seek_result = if resp.clicked() || resp.drag_stopped() {
        if let Some(mouse_pos) = resp.interact_pointer_pos() {
            let rel_x = (mouse_pos.x - rect.min.x).clamp(0.0, rect.width());
            let new_ratio = if rect.width() > 0.0 {
                rel_x / rect.width()
            } else {
                0.0
            };
            Some(new_ratio * total_dur)
        } else {
            None
        }
    } else {
        None
    };

    if is_hovered
        && let Some(mouse_pos) = resp.hover_pos() {
            let rel_x = (mouse_pos.x - rect.min.x).clamp(0.0, rect.width());
            let hover_time = if rect.width() > 0.0 {
                (rel_x / rect.width()) * total_dur
            } else {
                0.0
            };
            resp.on_hover_text(language.format(
                "control-seek-to",
                &[("time", &format_time(hover_time))],
            ));
        }

    seek_result
}

pub fn render_custom_volume_slider(
    ui: &mut egui::Ui,
    vol: f32,
    desired_w: f32,
    theme: &AppTheme,
    language: Language,
) -> Option<f32> {
    let desired_h = 14.0;
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(desired_w, desired_h),
        egui::Sense::click_and_drag(),
    );

    let painter = ui.painter();
    let is_hovered = resp.hovered();
    let is_dragged = resp.dragged();

    if is_hovered || is_dragged {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let progress = vol.clamp(0.0, 1.0);
    let center_y = rect.center().y;
    let rail_h = if is_hovered || is_dragged { 4.0 } else { 2.5 };

    let rail_rect = egui::Rect::from_min_max(
        egui::pos2(rect.min.x, center_y - rail_h * 0.5),
        egui::pos2(rect.max.x, center_y + rail_h * 0.5),
    );
    painter.rect_filled(rail_rect, rail_h * 0.5, theme.border_default);

    let fill_x = rect.min.x + (rect.width() * progress);
    if fill_x > rect.min.x {
        let fill_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x, center_y - rail_h * 0.5),
            egui::pos2(fill_x, center_y + rail_h * 0.5),
        );
        painter.rect_filled(fill_rect, rail_h * 0.5, theme.accent);
    }

    let knob_radius = if is_hovered || is_dragged { 4.0 } else { 3.0 };
    let knob_pos = egui::pos2(fill_x.clamp(rect.min.x, rect.max.x), center_y);
    painter.circle_filled(knob_pos, knob_radius, Color32::WHITE);
    painter.circle_stroke(knob_pos, knob_radius, egui::Stroke::new(1.2, theme.accent));

    let vol_result = if resp.clicked() || resp.dragged() {
        if let Some(mouse_pos) = resp.interact_pointer_pos() {
            let rel_x = (mouse_pos.x - rect.min.x).clamp(0.0, rect.width());
            let new_vol = if rect.width() > 0.0 {
                (rel_x / rect.width()).clamp(0.0, 1.0)
            } else {
                0.0
            };
            Some(new_vol)
        } else {
            None
        }
    } else {
        None
    };

    resp.on_hover_text(language.format(
        "control-volume",
        &[("percent", &format!("{:.0}", vol * 100.0))],
    ));

    vol_result
}

pub fn render_hero_play_button(
    ui: &mut egui::Ui,
    is_playing: bool,
    theme: &AppTheme,
    language: Language,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(30.0), egui::Sense::click());
    if resp.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
    let center = rect.center();
    let bg = if resp.is_pointer_button_down_on() {
        theme.accent.gamma_multiply(0.85)
    } else if resp.hovered() {
        theme.accent.gamma_multiply(1.08)
    } else {
        theme.accent
    };
    if resp.hovered() { ui.painter().circle_filled(center, 16.5, theme.accent_soft); }
    ui.painter().circle_filled(center, 15.0, bg);
    let (icon, offset) = if is_playing { (PAUSE, egui::vec2(0.0, 0.0)) } else { (PLAY, egui::vec2(1.0, 0.0)) };
    ui.painter().text(center + offset, egui::Align2::CENTER_CENTER, icon, egui::FontId::proportional(14.0), theme.text_on_accent);
    resp.on_hover_text(if is_playing { language.text("control-pause-media") } else { language.text("control-play-media") })
}

pub fn render_icon_circle_button(
    ui: &mut egui::Ui,
    icon: &str,
    size: f32,
    icon_size: f32,
    is_active: bool,
    theme: &AppTheme,
    tooltip: &str,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::click());
    if resp.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
    let center = rect.center();
    let bg = if is_active { theme.accent_soft } else if resp.is_pointer_button_down_on() { theme.bg_pill_active } else if resp.hovered() { theme.bg_pill } else { Color32::TRANSPARENT };
    if bg != Color32::TRANSPARENT { ui.painter().circle_filled(center, size * 0.5, bg); }
    let color = if is_active { theme.accent } else if resp.hovered() { theme.text_primary } else { theme.text_secondary };
    ui.painter().text(center, egui::Align2::CENTER_CENTER, icon, egui::FontId::proportional(icon_size), color);
    resp.on_hover_text(crate::ui::i18n::text(ui, tooltip))
}

