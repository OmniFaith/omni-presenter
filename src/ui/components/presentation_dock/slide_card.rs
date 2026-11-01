use crate::core::models::PresentationTheme;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, FontId, Stroke, StrokeKind, Vec2};

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_slide_card(
    ui: &mut egui::Ui,
    tokens: &AppTheme,
    live_theme: &PresentationTheme,
    slide_content: &str,
    index: usize,
    is_active: bool,
    is_hidden: bool,
    aspect: f32,
    with_bg: bool,
    card_w: f32,
    is_song: bool,
) -> egui::Response {
    let card_h = (card_w / aspect).floor();
    let item_total_h = card_h + 18.0;

    let (item_rect, response) =
        ui.allocate_exact_size(Vec2::new(card_w, item_total_h), egui::Sense::click());

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let card_rect = egui::Rect::from_min_size(item_rect.min, Vec2::new(card_w, card_h));

    let border_stroke = if is_active {
        if is_hidden {
            Stroke::new(2.5, tokens.status_warning_fg)
        } else {
            Stroke::new(2.0, tokens.accent)
        }
    } else if response.hovered() {
        Stroke::new(1.5, tokens.border_active)
    } else {
        Stroke::new(1.0, tokens.border_default)
    };

    let blackout = is_hidden && is_active;
    if blackout {
        ui.painter()
            .rect_filled(card_rect, tokens.radius_md, Color32::from_rgb(24, 24, 24));
    } else if with_bg {
        crate::render::paint_theme_bg(ui.painter(), card_rect, live_theme, tokens.radius_md);
        crate::render::render_theme_lyrics(ui.painter(), card_rect, live_theme, slide_content);
    } else {
        ui.painter()
            .rect_filled(card_rect, tokens.radius_md, tokens.bg_pill);
        let scale = (card_w / 144.0).clamp(0.6, 3.5);
        let base_size = 10.5 * scale;
        let lines: Vec<&str> = slide_content.lines().collect();

        // Shrink the font so the widest line fits inside the card.
        let widest = lines
            .iter()
            .map(|line| {
                ui.painter()
                    .layout_no_wrap(
                        line.trim().to_owned(),
                        FontId::proportional(base_size),
                        tokens.text_primary,
                    )
                    .size()
                    .x
            })
            .fold(0.0_f32, f32::max);
        let max_w = (card_rect.width() - 10.0).max(1.0);
        let fit = if widest > max_w {
            (max_w / widest).clamp(0.4, 1.0)
        } else {
            1.0
        };

        if slide_content.trim().is_empty() {
            ui.painter().text(
                card_rect.center(),
                egui::Align2::CENTER_CENTER,
                "—",
                FontId::proportional(14.0),
                tokens.text_muted,
            );
        } else {
            let font_size = base_size * fit;
            let line_height = font_size * 1.35;
            let total_text_h = lines.len() as f32 * line_height;
            let start_y = card_rect.center().y - total_text_h / 2.0;
            let painter = ui
                .painter()
                .with_clip_rect(card_rect.intersect(ui.clip_rect()));
            for (line_idx, line) in lines.iter().enumerate() {
                painter.text(
                    egui::pos2(
                        card_rect.center().x,
                        start_y + line_idx as f32 * line_height + line_height * 0.5,
                    ),
                    egui::Align2::CENTER_CENTER,
                    line.trim(),
                    FontId::proportional(font_size),
                    tokens.text_primary,
                );
            }
        }
    }

    ui.painter().rect_stroke(
        card_rect,
        tokens.radius_md,
        border_stroke,
        StrokeKind::Inside,
    );

    // Numbered badge under the card
    let badge_center = egui::pos2(card_rect.center().x, card_rect.bottom() + 9.0);
    let is_blank = slide_content.trim().is_empty();
    let tag_opt = crate::core::slides::extract_strophe_tag(slide_content);

    let (badge_label, badge_color) = if let Some(tag) = &tag_opt {
        let color = match tag.to_lowercase().as_str() {
            t if t.contains("refr") || t.contains("chorus") => egui::Color32::from_rgb(14, 165, 233),
            t if t.contains("pont") || t.contains("bridge") => egui::Color32::from_rgb(168, 85, 247),
            t if t.contains("intro") => egui::Color32::from_rgb(16, 185, 129),
            t if t.contains("final") || t.contains("ending") => egui::Color32::from_rgb(249, 115, 22),
            _ => tokens.accent,
        };
        let lang = crate::ui::i18n::language(ui.ctx());
        let display_tag = crate::core::preferences::StropheTagDefinition::localize_tag_name(tag, lang);
        (format!("{index} · {display_tag}"), color)
    } else if is_song {
        if index == 0 {
            ("T".to_string(), if is_active { tokens.accent } else { tokens.text_muted })
        } else if is_blank {
            ("—".to_string(), if is_active { tokens.accent } else { tokens.text_muted })
        } else {
            (index.to_string(), if is_active { tokens.accent } else { tokens.text_muted })
        }
    } else if is_blank {
        ("—".to_string(), if is_active { tokens.accent } else { tokens.text_muted })
    } else {
        ((index + 1).to_string(), if is_active { tokens.accent } else { tokens.text_muted })
    };

    let text_w = badge_label.len() as f32 * 6.5 + 14.0;
    let badge_w = text_w.clamp(24.0, card_w - 8.0);
    let badge_rect = egui::Rect::from_center_size(badge_center, Vec2::new(badge_w, 14.0));
    ui.painter()
        .rect_filled(badge_rect, tokens.radius_sm, if tag_opt.is_some() { badge_color.linear_multiply(0.18) } else { tokens.bg_pill });
    ui.painter().text(
        badge_center,
        egui::Align2::CENTER_CENTER,
        badge_label,
        FontId::proportional(10.0),
        badge_color,
    );


    let hover_text = if is_blank {
        "Slide em branco"
    } else {
        slide_content.trim()
    };
    response.on_hover_text(hover_text)
}
