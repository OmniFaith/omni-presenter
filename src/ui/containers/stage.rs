use crate::core::layout::ContainerKind;
use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use crate::render::paint_theme_bg;
use crate::ui::components::render_header_bar;
use eframe::egui::{self, Color32};

pub fn render_stage_column(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    presenting: &mut bool,
) {
    let theme = state.theme.clone();
    render_header_bar(ui, ContainerKind::Stage, "workspace-live-output", &theme);
    if let Some(extra) = deck.live_overlay.clone() {
        crate::ui::components::centered_horizontal(ui, "live_extra_controls", 140.0, |ui| {
            ui.label(
                egui::RichText::new(crate::ui::i18n::format(ui, "workspace-overlay", &[("name", &extra.name)]))
                    .size(10.0)
                    .color(theme.accent),
            );
            if theme
                .icon_button(
                    ui,
                    egui_phosphor::regular::X,
                    crate::ui::i18n::text(ui, "workspace-remove-overlay-without-stopping-the-presentation"),
                    false,
                    egui::Vec2::splat(22.0),
                )
                .clicked()
            {
                deck.live_overlay = None;
            }
        });
    }
    ui.add_space(2.0);

    let live_title = deck.live_title();
    let live_artist = deck.live_artist();
    let live_subtitle = if !live_title.is_empty() && !live_artist.is_empty() {
        format!("{live_title} · {live_artist}")
    } else if !live_title.is_empty() {
        live_title.to_string()
    } else {
        live_artist.to_string()
    };
    let hidden = deck.is_content_hidden();
    let has_subtitle = !live_subtitle.is_empty() && *presenting && !hidden;
    let subtitle_reserve = if has_subtitle { 24.0 } else { 4.0 };

    let aspect = state.presentation_aspect_ratio.clamp(0.5, 3.5);
    let width = (ui.available_width() - 1.0).max(40.0);
    let avail_h = (ui.available_height() - subtitle_reserve).max(10.0);
    let height = (width / aspect).min(avail_h);
    let live_theme = crate::services::media::live_theme(state, deck);
    let animate = !state.settings.operator().reduce_motion;
    let (slot, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let live_rect =
        egui::Rect::from_center_size(slot.center(), egui::vec2(height * aspect, height));
    if *presenting && !hidden {
        if let Some(scene) = &deck.live_design {
            crate::render::design::paint_themed_with_transition(
                ui.painter(),
                live_rect,
                scene,
                &state.themes,
                egui::Id::NULL.with("stage_live_design_transition"),
                animate,
            );
        } else {
            paint_theme_bg(ui.painter(), live_rect, &live_theme, theme.radius_lg);
            crate::services::effects::render(
                ui.painter(),
                live_rect,
                &live_theme,
                deck.live_slide_text(),
                egui::Id::NULL.with("stage_live_transition"),
                animate,
            );
        }
        if let Some(overlay) = &deck.live_overlay {
            crate::render::design::paint_with_transition(
                ui.painter(),
                live_rect,
                overlay,
                egui::Id::NULL.with("stage_live_overlay_transition"),
                animate,
            );
        }

    } else {
        if hidden {
            ui.painter()
                .rect_filled(live_rect, theme.radius_lg, Color32::BLACK);
        } else {
            let idle = state
                .themes
                .iter()
                .find(|t| t.id == state.settings.service().idle_theme_id)
                .unwrap_or(&live_theme);
            paint_theme_bg(ui.painter(), live_rect, idle, theme.radius_lg);
            ui.painter()
                .rect_filled(live_rect, theme.radius_lg, Color32::from_black_alpha(60));
        }
        crate::services::effects::render(
            ui.painter(),
            live_rect,
            &live_theme,
            "",
            egui::Id::unique("stage_live_transition"),
            false,
        );
        ui.painter().text(
            live_rect.center(),
            egui::Align2::CENTER_CENTER,
            if hidden {
                crate::ui::i18n::text(ui, "control-native-hidden")
            } else {
                crate::ui::i18n::text(ui, "control-native-ready")
            },
            egui::FontId::proportional(12.0),
            Color32::from_rgb(230, 235, 245),
        );
    }
    ui.painter().rect_stroke(
        live_rect,
        theme.radius_lg,
        egui::Stroke::new(1.0, theme.border_subtle),
        egui::StrokeKind::Inside,
    );

    if has_subtitle {
        ui.add_space(3.0);
        let avail_w = ui.available_width();
        let (label_rect, _) = ui.allocate_exact_size(
            egui::vec2(avail_w, 16.0),
            egui::Sense::hover(),
        );
        let text_galley = ui.painter().layout_no_wrap(
            live_subtitle,
            egui::FontId::proportional(10.5),
            theme.text_secondary,
        );
        let text_pos = egui::pos2(
            (label_rect.center().x - text_galley.size().x / 2.0).max(label_rect.left()),
            label_rect.center().y - text_galley.size().y / 2.0,
        );
        ui.painter().galley(text_pos, text_galley, theme.text_secondary);
    }
}

pub fn render_preview_column(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    presenting: &mut bool,
) {
    let theme = state.theme.clone();
    render_header_bar(ui, ContainerKind::Preview, "workspace-preview-label", &theme);
    ui.add_space(2.0);

    let title = deck.title();
    let artist = deck.artist();
    let subtitle = if !title.is_empty() && !artist.is_empty() {
        format!("{title} · {artist}")
    } else if !title.is_empty() {
        title.to_string()
    } else {
        artist.to_string()
    };
    let has_subtitle = !subtitle.is_empty();
    let subtitle_reserve = if has_subtitle { 24.0 } else { 4.0 };

    let aspect = state.presentation_aspect_ratio.clamp(0.5, 3.5);
    let width = (ui.available_width() - 1.0).max(40.0);
    let avail_h = (ui.available_height() - subtitle_reserve).max(10.0);
    let height = (width / aspect).min(avail_h);
    let preview_theme = crate::services::media::preview_theme(state, deck);
    let animate = !state.settings.operator().reduce_motion;

    let (slot, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());
    let preview_rect =
        egui::Rect::from_center_size(slot.center(), egui::vec2(height * aspect, height));
    if let Some(scene) = &mut deck.preview_design
        && !scene.overlay
        && scene.background_mode != "color"
    {
        scene.background_mode = "theme".into();
        scene.theme_id = Some(state.preview_theme().id.clone());
    }
    if let Some(scene) = &deck.preview_design {
        if scene.overlay && *presenting {
            if let Some(base) = &deck.live_design {
                crate::render::design::paint_themed(
                    ui.painter(),
                    preview_rect,
                    base,
                    &state.themes,
                );
            } else {
                let live_theme = crate::services::media::live_theme(state, deck);
                paint_theme_bg(ui.painter(), preview_rect, &live_theme, 0.0);
                crate::render::render_theme_lyrics(
                    ui.painter(),
                    preview_rect,
                    &live_theme,
                    deck.live_slide_text(),
                );
            }
        }
        crate::render::design::paint_themed(ui.painter(), preview_rect, scene, &state.themes);
    } else {
        paint_theme_bg(ui.painter(), preview_rect, &preview_theme, theme.radius_lg);
        crate::services::effects::render(
            ui.painter(),
            preview_rect,
            &preview_theme,
            deck.preview_slide_text(),
            egui::Id::NULL.with("stage_preview_transition"),
            animate,
        );
    }
    let hover = ui
        .ctx()
        .animate_bool(response.id.with("hover"), response.hovered());
    ui.painter().rect_stroke(
        preview_rect,
        theme.radius_lg,
        egui::Stroke::new(
            1.0 + hover,
            theme.border_subtle.lerp_to_gamma(theme.accent, hover),
        ),
        egui::StrokeKind::Inside,
    );
    if response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(crate::ui::i18n::text(ui, "workspace-click-to-present-this-preview-live"))
        .clicked()
        && (deck.preview_design.is_some()
            || !deck.preview_slide_text().is_empty()
            || !deck.slides().is_empty()
            || deck.preview_media.is_some())
    {
        deck.project_preview();
        if !deck
            .preview_design
            .as_ref()
            .is_some_and(|scene| scene.overlay)
        {
            state.apply_theme_to_live();
        }
        *presenting = true;
        state.show_presentation_dock = state.settings.operator().show_dock;
        if let Some(media) = &mut state.active_player_media {
            if media.kind == crate::core::state::PlayerMediaKind::Video
                && deck.live_media.as_ref().map(|m| &m.path) == Some(&media.path)
            {
                media.is_projecting = true;
                if media.is_playing {
                    let pos = media.video_position();
                    let _ = state.audio_player.play_video_stream(
                        media.title.clone(),
                        media.path.clone(),
                        pos,
                    );
                }
            }
        }
    }
    if has_subtitle {
        ui.add_space(3.0);
        let avail_w = ui.available_width();
        let (label_rect, _) = ui.allocate_exact_size(
            egui::vec2(avail_w, 16.0),
            egui::Sense::hover(),
        );
        let text_galley = ui.painter().layout_no_wrap(
            subtitle,
            egui::FontId::proportional(10.5),
            theme.text_secondary,
        );
        let text_pos = egui::pos2(
            (label_rect.center().x - text_galley.size().x / 2.0).max(label_rect.left()),
            label_rect.center().y - text_galley.size().y / 2.0,
        );
        ui.painter().galley(text_pos, text_galley, theme.text_secondary);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_stage_breakdown() {
        let mut state = ControlPanelState::default();
        let mut deck = SlideDeck::default();
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            render_stage_column(ui, &mut state, &mut deck, &mut false)
        });
        output.textures_delta.clear();
        assert!(!output.shapes.is_empty());
    }

    #[test]
    fn test_preview_column_renders() {
        let mut state = ControlPanelState::default();
        let mut deck = SlideDeck::default();
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            render_preview_column(ui, &mut state, &mut deck, &mut false)
        });
        output.textures_delta.clear();
        assert!(!output.shapes.is_empty());
    }
}
