use crate::core::i18n::Language;
use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, FontId, Margin, Stroke, StrokeKind, Vec2};
use egui_phosphor::regular::{
    ARROW_SQUARE_OUT, CHECK, EYE, EYE_SLASH, GEAR, PALETTE, PLAY, TELEVISION_SIMPLE, X,
};

pub(crate) const BTN_H: f32 = 26.0;
const BTN_PAD: f32 = 10.0;
const BTN_GAP: f32 = 6.0;
const ICON_W: f32 = 15.0;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Tone {
    Neutral,
    Accent,
    Warning,
    Danger,
}

fn text_width(ui: &egui::Ui, text: &str, size: f32) -> f32 {
    ui.painter()
        .layout_no_wrap(text.to_owned(), FontId::proportional(size), Color32::WHITE)
        .size()
        .x
}

fn tone_colors(tokens: &AppTheme, tone: Tone) -> (Color32, Color32) {
    match tone {
        Tone::Neutral => (tokens.text_secondary, tokens.border_default),
        Tone::Accent => (tokens.accent, tokens.accent.linear_multiply(0.45)),
        Tone::Warning => (
            tokens.status_warning_fg,
            tokens.status_warning_fg.linear_multiply(0.45),
        ),
        Tone::Danger => (
            tokens.status_danger,
            tokens.status_danger.linear_multiply(0.4),
        ),
    }
}

/// A pill-shaped button matching the ones used across the rest of the interface.
pub(crate) fn dock_button(
    ui: &mut egui::Ui,
    tokens: &AppTheme,
    icon: Option<&str>,
    label: &str,
    tone: Tone,
    active: bool,
) -> egui::Response {
    let size = 12.0;
    let label_w = if label.is_empty() {
        0.0
    } else {
        text_width(ui, label, size)
    };
    let icon_w = if icon.is_some() { ICON_W } else { 0.0 };
    let gap = if icon.is_some() && !label.is_empty() {
        BTN_GAP
    } else {
        0.0
    };
    let width = BTN_PAD * 2.0 + icon_w + gap + label_w;

    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, BTN_H), egui::Sense::click());
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let hover = ui
        .ctx()
        .animate_bool(response.id.with("hover"), response.hovered());
    let (fg, border) = tone_colors(tokens, tone);
    let bg = if active {
        tokens.accent_soft
    } else {
        tokens.bg_pill.lerp_to_gamma(tokens.bg_pill_active, hover)
    };
    ui.painter().rect_filled(rect, tokens.radius_md, bg);
    ui.painter().rect_stroke(
        rect,
        tokens.radius_md,
        Stroke::new(1.0, if active { tokens.accent } else { border }),
        StrokeKind::Inside,
    );

    let fg = if active { tokens.accent } else { fg };
    let mut x = rect.min.x + BTN_PAD;
    if let Some(icon) = icon {
        ui.painter().text(
            egui::pos2(x + ICON_W * 0.5, rect.center().y),
            egui::Align2::CENTER_CENTER,
            icon,
            FontId::proportional(13.0),
            fg,
        );
        x += ICON_W + gap;
    }
    if !label.is_empty() {
        ui.painter().text(
            egui::pos2(x, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            FontId::proportional(size),
            fg,
        );
    }
    response
}

/// Segmented "Texto / Fundo" toggle controlling the slide-card preview rendering.
pub(crate) fn preview_mode_control(
    ui: &mut egui::Ui,
    tokens: &AppTheme,
    language: Language,
    with_bg: &mut bool,
) -> bool {
    let options = [
        (language.text("control-text-mode"), false),
        (language.text("control-background-mode"), true),
    ];
    let widths: Vec<f32> = options
        .iter()
        .map(|(label, _)| text_width(ui, label, 11.5) + 24.0)
        .collect();
    let total_w: f32 = widths.iter().sum();

    let (rect, _) = ui.allocate_exact_size(Vec2::new(total_w, BTN_H), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, tokens.radius_md, tokens.bg_pill);
    ui.painter().rect_stroke(
        rect,
        tokens.radius_md,
        Stroke::new(1.0, tokens.border_default),
        StrokeKind::Inside,
    );

    let mut changed = false;
    let mut x = rect.min.x;
    for (index, (label, value)) in options.iter().enumerate() {
        let seg_rect =
            egui::Rect::from_min_size(egui::pos2(x, rect.min.y), Vec2::new(widths[index], BTN_H));
        let response = ui.interact(
            seg_rect,
            ui.make_persistent_id(("dock_preview_mode", index)),
            egui::Sense::click(),
        );
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        let selected = *with_bg == *value;
        if response.clicked() && !selected {
            *with_bg = *value;
            changed = true;
        }
        if selected {
            let inner = seg_rect.shrink(2.0);
            ui.painter()
                .rect_filled(inner, tokens.radius_sm, tokens.bg_panel);
            ui.painter().rect_stroke(
                inner,
                tokens.radius_sm,
                Stroke::new(1.0, tokens.accent),
                StrokeKind::Inside,
            );
        }
        ui.painter().text(
            seg_rect.center(),
            egui::Align2::CENTER_CENTER,
            *label,
            FontId::proportional(11.5),
            if selected {
                tokens.accent
            } else {
                tokens.text_secondary
            },
        );
        x += widths[index];
    }
    changed
}

pub(crate) fn live_theme_control(
    ui: &mut egui::Ui,
    tokens: &AppTheme,
    language: Language,
    state: &mut ControlPanelState,
    slide_deck: Option<&mut SlideDeck>,
) {
    let current_theme = state.live_theme();
    let current_id = current_theme.id.clone();
    let localized_name = current_theme.display_name(language);
    let label = format!("{}: {}", language.text("control-live-theme"), localized_name);

    let popup_id = ui.make_persistent_id("dock_live_theme_popup");
    let is_open = egui::Popup::is_id_open(ui.ctx(), popup_id);

    let response = dock_button(
        ui,
        tokens,
        Some(PALETTE),
        &label,
        if is_open { Tone::Accent } else { Tone::Neutral },
        is_open,
    )
    .on_hover_text(language.text("control-change-live-theme"));

    let mut selected_theme = None;
    egui::Popup::menu(&response)
        .id(popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.set_max_width(260.0);
            egui::ScrollArea::vertical()
                .max_height(280.0)
                .show(ui, |ui| {
                    for (idx, theme) in state.themes.iter().enumerate() {
                        let is_selected = theme.id == current_id;
                        let theme_name = theme.display_name(language);
                        let text = if is_selected {
                            format!("{} {}", CHECK, theme_name)
                        } else {
                            format!("   {}", theme_name)
                        };
                        if ui.selectable_label(is_selected, text).clicked() {
                            selected_theme = Some(idx);
                            egui::Popup::close_id(ui.ctx(), popup_id);
                        }
                    }
                });
        });

    if let Some(idx) = selected_theme {
        state.selected_theme_index = idx;
        state.apply_theme_to_live();
        if let Some(deck) = slide_deck {
            if let Some(scene) = &mut deck.live_design
                && !scene.overlay
                && scene.background_mode != "color"
            {
                scene.background_mode = "theme".into();
                scene.theme_id = Some(state.live_theme_id.clone());
            }
        }
    }
}

mod slide_card;
mod timer;
mod transport;

pub(crate) use slide_card::render_slide_card;
pub(crate) use timer::presentation_timer_badge;
pub(crate) use transport::transport_controls;
#[cfg(test)]
pub(crate) use transport::jump_input;

pub fn render_presentation_dock(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    slide_deck: &mut SlideDeck,
    is_presentation_open: &mut bool,
    is_preview_modal_open: &mut bool,
) {
    if *is_presentation_open {
        if state.presentation_started_at.is_none() {
            state.presentation_started_at = Some(std::time::Instant::now());
        }
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(500));
    } else {
        state.presentation_started_at = None;
    }

    let tokens = state.theme.clone();
    let language = state.settings.language;
    let live_theme = crate::services::media::live_theme(state, slide_deck);
    let total_live = slide_deck.total_live_slides();
    let current_live = slide_deck.live_index();

    // Header: title on the left, preview mode + actions on the right.
    ui.horizontal(|ui| {
        let (chip_rect, _) = ui.allocate_exact_size(Vec2::new(24.0, 24.0), egui::Sense::hover());
        ui.painter()
            .rect_filled(chip_rect, tokens.radius_sm, tokens.accent_soft);
        ui.painter().text(
            chip_rect.center(),
            egui::Align2::CENTER_CENTER,
            TELEVISION_SIMPLE,
            FontId::proportional(14.0),
            tokens.accent,
        );
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(slide_deck.live_title())
                .strong()
                .size(13.0)
                .color(tokens.text_primary),
        );
        if slide_deck.live_design.is_some() {
            ui.label(
                egui::RichText::new(language.text("control-dynamic-design"))
                    .size(11.5)
                    .color(tokens.text_muted),
            );
        } else if total_live > 0 {
            let separator = language.text("control-of");
            let label = if slide_deck.is_live_song() {
                if current_live == 0 {
                    language.text("control-title-slide").to_string()
                } else if slide_deck.live_slide_text().trim().is_empty() && current_live + 1 == total_live {
                    language.text("control-blank-slide").to_string()
                } else {
                    let total_strophes = if slide_deck.live_slides().last().map(|s| s.trim().is_empty()).unwrap_or(false) {
                        total_live.saturating_sub(2)
                    } else {
                        total_live.saturating_sub(1)
                    };
                    format!(
                        "{} {separator} {}",
                        current_live,
                        total_strophes
                    )
                }
            } else if slide_deck.live_slide_text().trim().is_empty() && current_live + 1 == total_live {
                language.text("control-blank-slide").to_string()
            } else {
                format!("{} {separator} {}", current_live + 1, total_live)
            };
            ui.label(
                egui::RichText::new(label)
                    .size(11.5)
                    .color(tokens.text_muted),
            );
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if slide_deck.live_overlay.is_some()
                && tokens
                    .icon_button(
                        ui,
                        X,
                        language.text("control-remove-overlay"),
                        false,
                        Vec2::splat(BTN_H),
                    )
                    .clicked()
            {
                slide_deck.live_overlay = None;
            }
            if dock_button(
                ui,
                &tokens,
                Some(EYE_SLASH),
                language.text("control-hide-panel"),
                Tone::Neutral,
                false,
            )
            .on_hover_text(language.text("control-hide-dock"))
            .clicked()
            {
                state.show_presentation_dock = false;
            }

            let is_preview_open = *is_preview_modal_open;
            if dock_button(
                ui,
                &tokens,
                Some(ARROW_SQUARE_OUT),
                language.text("control-inspect-slides"),
                Tone::Accent,
                is_preview_open,
            )
            .on_hover_text(language.text("control-floating-control-hint"))
            .clicked()
            {
                *is_preview_modal_open = !*is_preview_modal_open;
            }

            ui.add_space(4.0);
            live_theme_control(ui, &tokens, language, state, Some(slide_deck));

            ui.add_space(4.0);
            if preview_mode_control(ui, &tokens, language, &mut state.dock_preview_bg) {
                ui.ctx().request_repaint();
            }
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(language.text("control-preview-label"))
                    .size(11.0)
                    .color(tokens.text_muted),
            );
            ui.add_space(10.0);
            presentation_timer_badge(
                ui,
                &tokens,
                language,
                &mut state.presentation_started_at,
                &mut state.presentation_timer_seconds_mode,
            );
        });
    });

    ui.add_space(4.0);

    // Compute dynamic card dimensions based strictly on remaining vertical space in the dock.
    let aspect = state.presentation_aspect_ratio.clamp(0.5, 3.5);
    let current_y = ui.cursor().min.y;
    let bottom_y = ui.max_rect().bottom();
    // Transport controls height is 32px, plus reserve 4px gap above it = 36px.
    let transport_reserve = 36.0;
    let available_for_carousel = (bottom_y - current_y - transport_reserve).max(30.0);
    // Inside the carousel, a card badge is 18px and scrollbar can take up to 12px = 30px.
    let card_h = (available_for_carousel - 30.0).clamp(20.0, 320.0).floor();
    let card_w = (card_h * aspect).floor().clamp(40.0, 500.0);

    // Horizontal carousel of slide cards.
    if let Some(scene) = &slide_deck.live_design {
        let h = card_h;
        let w = (h * aspect).floor();
        let (rect, _) = ui.allocate_exact_size(Vec2::new(w, h), egui::Sense::hover());
        let animate = !state.settings.operator().reduce_motion;
        crate::render::design::paint_themed_with_transition(
            ui.painter(), rect, scene, &state.themes, egui::Id::NULL.with("dock_live_design_transition"), animate,
        );
    } else if total_live == 0 {
        ui.vertical_centered(|ui| {
            let pad = (available_for_carousel * 0.5 - 12.0).max(4.0);
            ui.add_space(pad);
            ui.label(
                egui::RichText::new(language.text("control-no-slides"))
                    .color(tokens.text_secondary),
            );
        });
    } else {
        let dock_scroll_id = ui.make_persistent_id("dock_carousel_scroll_state");
        let current_slide_key = (
            slide_deck.live_title().to_string(),
            slide_deck.live_artist().to_string(),
            total_live,
            current_live,
            card_w as i32,
        );

        let scroll_frames: u8 = ui.ctx().data_mut(|d| {
            let prev_key =
                d.get_temp::<(String, String, usize, usize, i32)>(dock_scroll_id.with("key"));
            let mut frames = d
                .get_temp::<u8>(dock_scroll_id.with("frames"))
                .unwrap_or(0);
            if prev_key.as_ref() != Some(&current_slide_key) {
                d.insert_temp(dock_scroll_id.with("key"), current_slide_key);
                frames = 2;
            }
            frames
        });

        let should_scroll = scroll_frames > 0;
        if should_scroll {
            ui.ctx().data_mut(|d| {
                d.insert_temp(dock_scroll_id.with("frames"), scroll_frames.saturating_sub(1));
            });
            ui.ctx().request_repaint();
        }

        egui::ScrollArea::horizontal()
            .id_salt("bottom_presentation_carousel_scroll")
            .auto_shrink([false, true])
            .max_height(available_for_carousel)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let slides = slide_deck.live_slides().to_vec();
                    let current_idx = slide_deck.live_index();
                    let is_hidden = slide_deck.is_content_hidden();
                    let with_bg = state.dock_preview_bg;

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
                                state.presentation_started_at = Some(std::time::Instant::now());
                            }
                        }
                        ui.add_space(10.0);
                    }
                });
            });
    }

    // Space before the transport bar so it sits pinned neatly at the bottom:
    let remaining_before_transport = (ui.max_rect().bottom() - ui.cursor().min.y - 32.0).max(0.0);
    if remaining_before_transport > 0.0 {
        ui.add_space(remaining_before_transport);
    }

    // Transport bar: config + progress on the left, navigation + jump in the middle,
    // hide/presentation controls on the right.
    ui.horizontal(|ui| {
        let full_w = ui.available_width();
        let row_start_x = ui.cursor().min.x;
        let center_x = row_start_x + full_w * 0.5;

        if tokens
            .icon_button(
                ui,
                GEAR,
                language.text("control-settings-presentation"),
                false,
                Vec2::splat(BTN_H),
            )
            .clicked()
        {
            state.show_presentation_settings_modal = true;
        }

        // Progress bar
        let progress = slide_deck.progress();
        let bar_size = Vec2::new(110.0, 6.0);
        let (bar_rect, _) = ui.allocate_exact_size(bar_size, egui::Sense::hover());
        ui.painter()
            .rect_filled(bar_rect, 3.0, tokens.bg_pill_active);
        if progress > 0.0 {
            let fill_w = (bar_size.x * progress).clamp(6.0, bar_size.x);
            let fill_rect = egui::Rect::from_min_size(bar_rect.min, Vec2::new(fill_w, bar_size.y));
            ui.painter().rect_filled(fill_rect, 3.0, tokens.accent);
        }

        // Center grouped navigation controls.
        let nav_id = ui.make_persistent_id("dock_nav_controls_width");
        let nav_w = ui
            .ctx()
            .data_mut(|d| d.get_temp::<f32>(nav_id))
            .unwrap_or(162.0);
        let target_x = center_x - nav_w * 0.5;
        let space_before = (target_x - ui.cursor().min.x).max(8.0);
        ui.add_space(space_before);

        // Grouped navigation controls.
        let nav_res = egui::Frame::new()
            .fill(tokens.bg_pill)
            .corner_radius(tokens.radius_md)
            .inner_margin(Margin::symmetric(4, 3))
            .show(ui, |ui| {
                transport_controls(ui, &tokens, slide_deck, state, is_presentation_open);
            });

        let actual_w = nav_res.response.rect.width();
        if actual_w > 0.0 && (actual_w - nav_w).abs() > 0.5 {
            ui.ctx().data_mut(|d| d.insert_temp(nav_id, actual_w));
        }

        // Right side: blackout toggle and presentation open/close.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let (presentation_label, presentation_tone, presentation_icon) =
                if *is_presentation_open {
                    (language.text("control-close-presentation"), Tone::Danger, X)
                } else {
                    (language.text("control-open-presentation"), Tone::Accent, PLAY)
                };
            if dock_button(
                ui,
                &tokens,
                Some(presentation_icon),
                presentation_label,
                presentation_tone,
                false,
            )
            .clicked()
            {
                *is_presentation_open = !*is_presentation_open;
                if *is_presentation_open {
                    state.presentation_started_at = Some(std::time::Instant::now());
                    if let Some(media) = &mut state.active_player_media {
                        if media.kind == crate::core::state::PlayerMediaKind::Video
                            && slide_deck.live_media.as_ref().map(|m| &m.path) == Some(&media.path)
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
                } else {
                    state.presentation_started_at = None;
                    if let Some(media) = &mut state.active_player_media {
                        if media.is_projecting {
                            media.is_projecting = false;
                            if media.is_playing {
                                media.is_playing = false;
                                media.video_paused_at = Some(media.video_position());
                                crate::services::video::stop(ui.ctx(), &media.path);
                            }
                        }
                    }
                    state.audio_player.pause();
                }
            }

            let hidden = slide_deck.is_content_hidden();
            if dock_button(
                ui,
                &tokens,
                Some(if hidden { EYE } else { EYE_SLASH }),
                if hidden {
                    language.text("control-show-screen")
                } else {
                    language.text("control-hide-screen")
                },
                if hidden { Tone::Warning } else { Tone::Neutral },
                hidden,
            )
            .on_hover_text(language.text("control-hide-screen-hint"))
            .clicked()
            {
                slide_deck.toggle_hide_content();
            }
        });
    });
    ui.expand_to_include_rect(ui.max_rect());
}

#[cfg(test)]
mod tests;
