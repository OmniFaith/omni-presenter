use crate::core::commands::Command;
use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use crate::ui::components::presentation_dock::{
    dock_button, presentation_timer_badge, preview_mode_control, render_slide_card,
    transport_controls, Tone,
};
use eframe::egui::{self, Vec2};
use egui_phosphor::regular::{EYE, EYE_SLASH, PLAY, X};

pub fn render_preview_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    slide_deck: &mut SlideDeck,
    is_presentation_open: &mut bool,
    is_modal_open: &mut bool,
) {
    if !*is_modal_open {
        return;
    }

    let tokens = state.theme.clone();
    let language = state.settings.language;
    let viewport_id = egui::ViewportId::from_hash_of("presentation_preview_viewport");
    let window_title = format!(
        "Omni Presenter — {} ({})",
        language.text("control-inspect-slides"),
        slide_deck.live_title()
    );
    let mut builder = egui::ViewportBuilder::default()
        .with_title(window_title)
        .with_inner_size([960.0, 440.0])
        .with_min_inner_size([540.0, 260.0])
        .with_decorations(true)
        .with_resizable(true)
        .with_active(true);

    if let Some(icon) = crate::services::windowing::app_icon() {
        builder = builder.with_icon(icon);
    }

    ctx.show_viewport_immediate(viewport_id, builder, |ctx, _class| {
        // Close modal on window close or Escape
        if ctx.input(|i| i.viewport().close_requested())
            || ctx.input(|i| i.key_pressed(egui::Key::Escape))
        {
            *is_modal_open = false;
        }

        // Keyboard navigation within the floating window follows the operator's
        // configured shortcuts, just like the main presentation window.
        let has_prev = slide_deck.current_index() > 0;
        let has_next = slide_deck.total_slides() > 0
            && (slide_deck.current_index() + 1 < slide_deck.total_slides() || *is_presentation_open);
        let shortcuts = state.settings.shortcuts.clone();
        let shortcuts_enabled = state.settings.operator().keyboard_shortcuts;
        let not_editing_text = !ctx.memory(|m| m.focused().is_some());

        if shortcuts_enabled && not_editing_text {
            if has_prev && shortcuts.pressed(ctx, Command::PreviousSlide) {
                slide_deck.prev_slide();
                ctx.request_repaint();
            }
            if has_next && shortcuts.pressed(ctx, Command::NextSlide) {
                if !slide_deck.next_slide() && *is_presentation_open {
                    *is_presentation_open = false;
                }
                ctx.request_repaint();
            }
            if has_prev && shortcuts.pressed(ctx, Command::FirstSlide) {
                slide_deck.first_slide();
                ctx.request_repaint();
            }
            if has_next && shortcuts.pressed(ctx, Command::LastSlide) {
                slide_deck.last_slide();
                ctx.request_repaint();
            }
            if shortcuts.pressed(ctx, Command::ToggleHide) {
                slide_deck.toggle_hide_content();
                ctx.request_repaint();
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(tokens.bg_panel).inner_margin(14))
            .show(ctx, |ui| {
                // ROW 1: Status, title, slide indicator, and primary action buttons
                ui.horizontal(|ui| {
                    // Live Status Indicator Pill
                    if *is_presentation_open {
                        let (badge_rect, _) =
                            ui.allocate_exact_size(Vec2::new(76.0, 24.0), egui::Sense::hover());
                        ui.painter()
                            .rect_filled(badge_rect, tokens.radius_sm, tokens.status_success_bg);
                        ui.painter().text(
                            badge_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            language.text("control-live-badge"),
                            egui::FontId::proportional(11.0),
                            tokens.status_success_fg,
                        );
                    } else {
                        let (badge_rect, _) =
                            ui.allocate_exact_size(Vec2::new(76.0, 24.0), egui::Sense::hover());
                        ui.painter()
                            .rect_filled(badge_rect, tokens.radius_sm, tokens.bg_pill);
                        ui.painter().text(
                            badge_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            language.text("control-standby"),
                            egui::FontId::proportional(11.0),
                            tokens.text_muted,
                        );
                    }

                    ui.add_space(8.0);

                    // Title & Artist
                    ui.label(
                        egui::RichText::new(slide_deck.live_title())
                            .strong()
                            .size(14.0)
                            .color(tokens.text_primary),
                    );
                    if !slide_deck.live_artist().is_empty() {
                        ui.label(
                            egui::RichText::new(format!("— {}", slide_deck.live_artist()))
                                .size(12.5)
                                .color(tokens.text_secondary),
                        );
                    }

                    // Slide counter badge
                    let current_idx = slide_deck.live_index();
                    let total_slides = slide_deck.total_slides();
                    if total_slides > 0 {
                        let separator = language.text("control-of");
                        let slide_label = if slide_deck.is_live_song() {
                            if current_idx == 0 {
                                language.text("control-title-slide").to_string()
                            } else if slide_deck.live_slide_text().trim().is_empty() && current_idx + 1 == total_slides {
                                language.text("control-blank-slide").to_string()
                            } else {
                                let total_strophes = if slide_deck.live_slides().last().map(|s| s.trim().is_empty()).unwrap_or(false) {
                                    total_slides.saturating_sub(2)
                                } else {
                                    total_slides.saturating_sub(1)
                                };
                                format!(
                                    "{} {separator} {}",
                                    current_idx,
                                    total_strophes
                                )
                            }
                        } else if slide_deck.live_slide_text().trim().is_empty() && current_idx + 1 == total_slides {
                            language.text("control-blank-slide").to_string()
                        } else {
                            format!("{} {separator} {}", current_idx + 1, total_slides)
                        };
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(format!("({slide_label})"))
                                .size(12.0)
                                .color(tokens.text_muted),
                        );
                    }

                    // Right side controls: Projection open/close, Blackout toggle, Live Timer
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Open/Close physical presentation window
                        let (proj_label, proj_tone, proj_icon) = if *is_presentation_open {
                            (language.text("control-close-presentation"), Tone::Danger, X)
                        } else {
                            (language.text("control-open-presentation"), Tone::Accent, PLAY)
                        };
                        if dock_button(ui, &tokens, Some(proj_icon), proj_label, proj_tone, false)
                            .on_hover_text(language.text("control-open-presentation-hint"))
                            .clicked()
                        {
                            *is_presentation_open = !*is_presentation_open;
                        }

                        ui.add_space(4.0);

                        // Blackout toggle
                        let is_hidden = slide_deck.is_content_hidden();
                        let (blackout_label, blackout_tone) = if is_hidden {
                            (language.text("control-show-screen"), Tone::Warning)
                        } else {
                            (language.text("control-blackout"), Tone::Neutral)
                        };
                        if dock_button(
                            ui,
                            &tokens,
                            Some(if is_hidden { EYE } else { EYE_SLASH }),
                            blackout_label,
                            blackout_tone,
                            is_hidden,
                        )
                        .on_hover_text(language.text("control-blackout-hint"))
                        .clicked()
                        {
                            slide_deck.toggle_hide_content();
                        }

                        ui.add_space(6.0);

                        // Live presentation timer
                        presentation_timer_badge(
                            ui,
                            &tokens,
                            language,
                            &mut state.presentation_started_at,
                            &mut state.presentation_timer_seconds_mode,
                        );
                    });
                });

                ui.add_space(8.0);

                // ROW 2: Transport controls and viewing options (Text/Bg mode, Zoom sizes)
                let zoom_id = ui.make_persistent_id("floating_modal_card_zoom");
                let mut zoom = ui
                    .ctx()
                    .data_mut(|d| d.get_temp::<usize>(zoom_id))
                    .unwrap_or(1); // 0=P (160), 1=M (210), 2=G (270)

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(language.text("control-navigation"))
                            .size(11.0)
                            .color(tokens.text_muted),
                    );
                    ui.add_space(2.0);

                    egui::Frame::new()
                        .fill(tokens.bg_pill)
                        .corner_radius(tokens.radius_md)
                        .inner_margin(egui::Margin::symmetric(4, 3))
                        .show(ui, |ui| {
                            transport_controls(ui, &tokens, slide_deck, state, is_presentation_open);
                        });

                    // Right side: Zoom sizes & Preview mode toggle
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Zoom pills: P (160px), M (210px), G (270px)
                        let sizes = [("P", 160.0), ("M", 210.0), ("G", 270.0)];
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            for (idx, (label, _)) in sizes.iter().enumerate() {
                                let selected = zoom == idx;
                                if dock_button(
                                    ui,
                                    &tokens,
                                    None,
                                    label,
                                    if selected {
                                        Tone::Accent
                                    } else {
                                        Tone::Neutral
                                    },
                                    selected,
                                )
                                .on_hover_text(language.format(
                                    "control-card-size",
                                    &[(
                                        "size",
                                        language.text(match idx {
                                            0 => "control-zoom-small",
                                            1 => "control-zoom-medium",
                                            _ => "control-zoom-large",
                                        }),
                                    )],
                                ))
                                .clicked()
                                {
                                    zoom = idx;
                                    ui.ctx().data_mut(|d| d.insert_temp(zoom_id, zoom));
                                }
                            }
                        });

                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(language.text("control-cards"))
                                .size(11.0)
                                .color(tokens.text_muted),
                        );

                        ui.add_space(8.0);

                        // Mode toggle: Texto / Fundo
                        if preview_mode_control(ui, &tokens, language, &mut state.dock_preview_bg) {
                            ui.ctx().request_repaint();
                        }

                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(language.text("control-view"))
                                .size(11.0)
                                .color(tokens.text_muted),
                        );
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                let total = slide_deck.total_slides();
                if total == 0 {
                    ui.vertical_centered(|ui| {
                        ui.add_space(40.0);
                        ui.label(
                            egui::RichText::new(language.text("control-no-slides-live"))
                                .color(tokens.text_secondary),
                        );
                    });
                    return;
                }

                let card_w = match zoom {
                    0 => 160.0,
                    2 => 270.0,
                    _ => 210.0,
                };

                let aspect = state.presentation_aspect_ratio.clamp(0.5, 3.5);
                let with_bg = state.dock_preview_bg;
                let live_theme = crate::services::media::live_theme(state, slide_deck);

                let preview_scroll_id =
                    ui.make_persistent_id("floating_preview_carousel_scroll_state");
                let current_idx = slide_deck.live_index();
                let current_slide_key = (
                    slide_deck.live_title().to_string(),
                    slide_deck.live_artist().to_string(),
                    slide_deck.total_live_slides(),
                    current_idx,
                    zoom,
                );

                let scroll_frames: u8 = ui.ctx().data_mut(|d| {
                    let prev_key = d.get_temp::<(String, String, usize, usize, usize)>(
                        preview_scroll_id.with("key"),
                    );
                    let mut frames = d
                        .get_temp::<u8>(preview_scroll_id.with("frames"))
                        .unwrap_or(0);
                    if prev_key.as_ref() != Some(&current_slide_key) {
                        d.insert_temp(preview_scroll_id.with("key"), current_slide_key);
                        frames = 2;
                    }
                    frames
                });

                let should_scroll = scroll_frames > 0;
                if should_scroll {
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(
                            preview_scroll_id.with("frames"),
                            scroll_frames.saturating_sub(1),
                        );
                    });
                    ui.ctx().request_repaint();
                }

                egui::ScrollArea::horizontal()
                    .id_salt("floating_preview_carousel_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let slides = slide_deck.live_slides().to_vec();
                            let is_hidden = slide_deck.is_content_hidden();
                            let is_song = slide_deck.is_live_song();

                            for (index, slide_content) in slides.iter().enumerate() {
                                let is_active = index == current_idx;
                                let response = render_slide_card(
                                    ui,
                                    &tokens,
                                    &live_theme,
                                    slide_content,
                                    index,
                                    is_active,
                                    is_hidden,
                                    aspect,
                                    with_bg,
                                    card_w,
                                    is_song,
                                );
                                if is_active && should_scroll {
                                    response.scroll_to_me(Some(egui::Align::Center));
                                }
                                if response.clicked() {
                                    slide_deck.select_live_slide(index);
                                    if !*is_presentation_open {
                                        *is_presentation_open = true;
                                    }
                                }
                                ui.add_space(12.0);
                            }
                        });
                    });
            });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_external_window_renders_and_toggles() {
        let mut state = ControlPanelState::default();
        let mut slide_deck = SlideDeck::default();
        slide_deck.project_custom_content(
            "Título".to_owned(),
            "Artista".to_owned(),
            "Estrofe 1\n\nEstrofe 2".to_owned(),
        );
        let mut is_presentation_open = false;
        let mut is_modal_open = true;

        let ctx = egui::Context::default();
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(1000.0, 600.0),
            )),
            ..Default::default()
        };

        let mut output = ctx.run_ui(raw_input, |_| {
            render_preview_modal(
                &ctx,
                &mut state,
                &mut slide_deck,
                &mut is_presentation_open,
                &mut is_modal_open,
            );
        });
        output.textures_delta.clear();
        assert!(is_modal_open);
    }
}
