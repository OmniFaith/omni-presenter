use super::{DesignEditor, TimelineDrag, TimelineDragHandle, TimelineMode};
use crate::{core::models::PresentationTheme, ui::theme::AppTheme};
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Stroke, Vec2};
use egui_phosphor::regular::*;

pub(super) fn render(
    ui: &mut egui::Ui,
    editor: &mut DesignEditor,
    theme: &AppTheme,
    themes: &[PresentationTheme],
) {
    if editor.scene.is_none() {
        return;
    }
    if let Some(scene) = &mut editor.scene {
        scene.ensure_slides();
    }

    let total_slide_ms = {
        let scene = editor.scene.as_ref().unwrap();
        scene.active_slide_duration_ms().max(500)
    };

    // 1. Process active timeline dragging across frames
    let pointer_down = ui.input(|i| i.pointer.primary_down());
    let pointer_pos = ui.input(|i| i.pointer.latest_pos());

    if let Some(drag) = editor.timeline_drag {
        if pointer_down {
            if let (Some(pos), Some(scene)) = (pointer_pos, &mut editor.scene) {
                if let Some(layer) = scene.layers.get_mut(drag.layer_index) {
                    // Estimated track width for drag delta calculation
                    let track_w = (ui.available_width() - 170.0).max(120.0);
                    let delta_x = pos.x - drag.initial_pointer_x;
                    let delta_ms = ((delta_x / track_w) * total_slide_ms as f32) as i64;

                    let snap = |ms: i64| -> u32 {
                        let snapped = ((ms + 50) / 100) * 100;
                        snapped.max(0) as u32
                    };

                    match drag.handle {
                        TimelineDragHandle::Start => {
                            let max_start = (drag.initial_end_ms as i64 - 100).max(0);
                            let raw_start = (drag.initial_start_ms as i64 + delta_ms).clamp(0, max_start);
                            layer.animation.start_ms = snap(raw_start);
                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                        }
                        TimelineDragHandle::End => {
                            let min_end = drag.initial_start_ms as i64 + 100;
                            let raw_end = (drag.initial_end_ms as i64 + delta_ms).clamp(min_end, total_slide_ms as i64);
                            layer.animation.end_ms = snap(raw_end).min(total_slide_ms as u32);
                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                        }
                        TimelineDragHandle::Move => {
                            let dur = drag.initial_end_ms.saturating_sub(drag.initial_start_ms);
                            let max_start = (total_slide_ms as i64 - dur as i64).max(0);
                            let raw_start = (drag.initial_start_ms as i64 + delta_ms).clamp(0, max_start);
                            let new_start = snap(raw_start);
                            layer.animation.start_ms = new_start;
                            layer.animation.end_ms = new_start + dur;
                            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                        }
                    }
                    editor.saved = false;
                    editor.pending_since = Some(std::time::Instant::now());
                }
            }
        } else {
            editor.timeline_drag = None;
        }
    }

    let is_playing = editor.animation_playing;
    let is_loop = editor.animation_loop;
    let show_filmstrip = editor.show_filmstrip;
    let timeline_mode = editor.timeline_mode;
    let curr_time_ms = editor.animation_time_ms;

    let (max_sec, active_idx, slide_count, current_slide_duration, transition_name, slide_docs) = {
        let scene = editor.scene.as_ref().unwrap();
        let max_sec = (total_slide_ms as f32 / 1000.0).max(0.5);
        let active_idx = scene.active_slide_index;
        let slide_count = scene.slides.len();
        let current_slide_duration = scene
            .slides
            .get(active_idx)
            .map(|s| s.duration_secs)
            .unwrap_or(scene.slide_duration_secs);
        let transition_name = scene.transition.clone();

        let mut docs = Vec::with_capacity(scene.slides.len());
        for (idx, slide) in scene.slides.iter().enumerate() {
            let mut doc = scene.clone();
            if idx == active_idx {
                doc.layers = scene.layers.clone();
            } else {
                doc.layers = slide.layers.clone();
            }
            docs.push((idx, slide.name.clone(), slide.duration_secs, doc));
        }

        (
            max_sec,
            active_idx,
            slide_count,
            current_slide_duration,
            transition_name,
            docs,
        )
    };

    let mut toggle_play = false;
    let mut reset_play = false;
    let mut toggle_loop = false;
    let mut toggle_filmstrip = false;
    let mut set_mode: Option<TimelineMode> = None;
    let mut scrub_to_ms: Option<u64> = None;
    let mut new_slide_duration: Option<f32> = None;
    let mut switch_to: Option<usize> = None;
    let mut add_requested = false;
    let mut duplicate_requested = false;
    let mut delete_requested: Option<usize> = None;
    let mut select_layer: Option<usize> = None;
    let mut start_drag: Option<TimelineDrag> = None;

    let curr_sec = (curr_time_ms as f32 / 1000.0).min(max_sec);

    // --- 0. Vertical Resize Splitter Handle ---
    if show_filmstrip && timeline_mode == TimelineMode::Layers {
        let (split_rect, split_resp) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), 10.0),
            egui::Sense::click_and_drag(),
        );

        if split_resp.hovered() || split_resp.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }

        if split_resp.dragged() {
            let dy = split_resp.drag_delta().y;
            editor.timeline_height = (editor.timeline_height - dy).clamp(60.0, 550.0);
            ui.ctx().request_repaint();
        }

        if split_resp.double_clicked() {
            editor.timeline_height = if (editor.timeline_height - 140.0).abs() < 15.0 {
                240.0
            } else {
                140.0
            };
            ui.ctx().request_repaint();
        }

        let is_active = split_resp.hovered() || split_resp.dragged();
        let grip_anim = ui.ctx().animate_bool_with_time(split_resp.id.with("split_hov"), is_active, 0.12);
        let grip_w = 44.0 + grip_anim * 16.0;
        let grip_h = 4.0;
        let grip_color = theme.border_default.lerp_to_gamma(theme.accent, grip_anim);

        let grip_rect = Rect::from_center_size(
            split_rect.center(),
            Vec2::new(grip_w, grip_h),
        );
        ui.painter().rect_filled(grip_rect, 2.0, grip_color);

        split_resp.on_hover_text(crate::ui::i18n::text(ui, "design-resize-timeline"));
        ui.add_space(2.0);
    }

    let mut cycle_height = false;

    // Dock container card
    egui::Frame::new()
        .fill(theme.bg_card)
        .stroke(Stroke::new(1.0, theme.border_default))
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            // --- 1. Playback & Timeline Header Bar ---
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;

                // Play / Pause Button
                let play_icon = if is_playing { PAUSE } else { PLAY };
                let play_text = if is_playing {
                    crate::ui::i18n::text(ui, "design-pause")
                } else {
                    crate::ui::i18n::text(ui, "design-preview-animation")
                };

                let (play_rect, play_resp) = ui.allocate_exact_size(Vec2::new(170.0, 28.0), egui::Sense::click());
                if play_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                let play_hov = ui.ctx().animate_bool_with_time(play_resp.id.with("play_hov"), play_resp.hovered(), 0.12);
                let play_hold = ui.ctx().animate_bool_with_time(play_resp.id.with("play_hold"), play_resp.is_pointer_button_down_on(), 0.12);

                let base_play_bg = if is_playing { theme.status_danger } else { theme.accent };
                let play_bg = base_play_bg.lerp_to_gamma(Color32::WHITE, play_hov * 0.14);
                let draw_play_rect = play_rect.shrink(play_hold * 0.8);

                ui.painter().rect_filled(draw_play_rect, 6.0, play_bg);
                ui.painter().rect_stroke(
                    draw_play_rect,
                    6.0,
                    Stroke::new(1.0, if play_hov > 0.0 { Color32::WHITE } else { base_play_bg }),
                    egui::StrokeKind::Inside,
                );
                ui.painter().text(
                    draw_play_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{play_icon}  {play_text}"),
                    egui::FontId::proportional(11.5),
                    Color32::WHITE,
                );
                if play_resp.clicked() {
                    toggle_play = true;
                }

                // Reset Button
                let (reset_rect, reset_resp) = ui.allocate_exact_size(Vec2::new(28.0, 28.0), egui::Sense::click());
                if reset_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                let reset_hov = ui.ctx().animate_bool_with_time(reset_resp.id.with("rst_hov"), reset_resp.hovered(), 0.12);
                let reset_hold = ui.ctx().animate_bool_with_time(reset_resp.id.with("rst_hld"), reset_resp.is_pointer_button_down_on(), 0.12);
                let reset_bg = theme.bg_app.lerp_to_gamma(theme.bg_card_hover, reset_hov);
                let draw_reset_rect = reset_rect.shrink(reset_hold * 0.6);

                ui.painter().rect_filled(draw_reset_rect, 6.0, reset_bg);
                ui.painter().rect_stroke(
                    draw_reset_rect,
                    6.0,
                    Stroke::new(1.0, if reset_hov > 0.0 { theme.accent } else { theme.border_default }),
                    egui::StrokeKind::Inside,
                );
                ui.painter().text(
                    draw_reset_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    ARROW_COUNTER_CLOCKWISE,
                    egui::FontId::proportional(13.0),
                    if reset_hov > 0.0 { theme.accent } else { theme.text_secondary },
                );
                if reset_resp.clicked() {
                    reset_play = true;
                }
                let _ = reset_resp.on_hover_text(crate::ui::i18n::text(ui, "design-restart-countdown"));

                // Loop Button
                let (loop_rect, loop_resp) = ui.allocate_exact_size(Vec2::new(28.0, 28.0), egui::Sense::click());
                if loop_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                let loop_hov = ui.ctx().animate_bool_with_time(loop_resp.id.with("loop_hov"), loop_resp.hovered(), 0.12);
                let loop_hold = ui.ctx().animate_bool_with_time(loop_resp.id.with("loop_hld"), loop_resp.is_pointer_button_down_on(), 0.12);
                let loop_bg = if is_loop {
                    theme.accent_soft.lerp_to_gamma(theme.accent, loop_hov * 0.1)
                } else {
                    theme.bg_app.lerp_to_gamma(theme.bg_card_hover, loop_hov)
                };
                let draw_loop_rect = loop_rect.shrink(loop_hold * 0.6);

                ui.painter().rect_filled(draw_loop_rect, 6.0, loop_bg);
                ui.painter().rect_stroke(
                    draw_loop_rect,
                    6.0,
                    Stroke::new(1.0, if is_loop || loop_hov > 0.0 { theme.accent } else { theme.border_default }),
                    egui::StrokeKind::Inside,
                );
                ui.painter().text(
                    draw_loop_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    REPEAT,
                    egui::FontId::proportional(13.0),
                    if is_loop || loop_hov > 0.0 { theme.accent } else { theme.text_secondary },
                );
                if loop_resp.clicked() {
                    toggle_loop = true;
                }
                let _ = loop_resp.on_hover_text(crate::ui::i18n::text(ui, "design-loop"));

                // Divider before Mode Switcher
                ui.add_space(4.0);
                ui.painter().line_segment(
                    [Pos2::new(ui.cursor().min.x, ui.cursor().center().y - 10.0), Pos2::new(ui.cursor().min.x, ui.cursor().center().y + 10.0)],
                    Stroke::new(1.0, theme.border_default),
                );
                ui.add_space(6.0);

                // Segmented Control: Exactly identical widths & heights with animated on-hold / hover
                let seg_total_w = 236.0f32;
                let seg_h = 28.0f32;
                let (seg_rect, _) = ui.allocate_exact_size(Vec2::new(seg_total_w, seg_h), egui::Sense::hover());

                ui.painter().rect_filled(seg_rect, 6.0, theme.bg_app);
                ui.painter().rect_stroke(seg_rect, 6.0, Stroke::new(1.0, theme.border_default), egui::StrokeKind::Inside);

                let half_w = (seg_total_w - 4.0) * 0.5; // 116.0 px each
                let seg1_rect = Rect::from_min_size(seg_rect.min + egui::vec2(2.0, 2.0), Vec2::new(half_w, seg_h - 4.0));
                let seg2_rect = Rect::from_min_size(Pos2::new(seg1_rect.max.x, seg1_rect.min.y), Vec2::new(half_w, seg_h - 4.0));

                // Segment 1: Linha do tempo
                let is_layers = timeline_mode == TimelineMode::Layers;
                let resp1 = ui.interact(seg1_rect, ui.make_persistent_id("seg_btn_timeline"), egui::Sense::click());
                if resp1.clicked() && !is_layers {
                    set_mode = Some(TimelineMode::Layers);
                }
                if resp1.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                let hov1 = ui.ctx().animate_bool_with_time(resp1.id.with("hov"), resp1.hovered(), 0.12);
                let hold1 = ui.ctx().animate_bool_with_time(resp1.id.with("hold"), resp1.is_pointer_button_down_on(), 0.12);

                let bg1 = if is_layers {
                    theme.accent.lerp_to_gamma(Color32::WHITE, hov1 * 0.1)
                } else if hov1 > 0.0 || hold1 > 0.0 {
                    theme.bg_card_hover.lerp_to_gamma(theme.accent_soft, hold1)
                } else {
                    Color32::TRANSPARENT
                };
                let draw1 = seg1_rect.shrink(hold1 * 0.6);
                ui.painter().rect_filled(draw1, 4.5, bg1);
                if is_layers {
                    ui.painter().rect_stroke(draw1, 4.5, Stroke::new(1.0, theme.accent), egui::StrokeKind::Inside);
                }
                ui.painter().text(
                    draw1.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{CLOCK} {}", crate::ui::i18n::text(ui, "design-timeline")),
                    egui::FontId::proportional(11.0),
                    if is_layers { Color32::WHITE } else if hov1 > 0.0 { theme.text_primary } else { theme.text_secondary },
                );

                // Segment 2: Slide
                let is_slides = timeline_mode == TimelineMode::Slides;
                let resp2 = ui.interact(seg2_rect, ui.make_persistent_id("seg_btn_slides"), egui::Sense::click());
                if resp2.clicked() && !is_slides {
                    set_mode = Some(TimelineMode::Slides);
                }
                if resp2.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                let hov2 = ui.ctx().animate_bool_with_time(resp2.id.with("hov"), resp2.hovered(), 0.12);
                let hold2 = ui.ctx().animate_bool_with_time(resp2.id.with("hold"), resp2.is_pointer_button_down_on(), 0.12);

                let bg2 = if is_slides {
                    theme.accent.lerp_to_gamma(Color32::WHITE, hov2 * 0.1)
                } else if hov2 > 0.0 || hold2 > 0.0 {
                    theme.bg_card_hover.lerp_to_gamma(theme.accent_soft, hold2)
                } else {
                    Color32::TRANSPARENT
                };
                let draw2 = seg2_rect.shrink(hold2 * 0.6);
                ui.painter().rect_filled(draw2, 4.5, bg2);
                if is_slides {
                    ui.painter().rect_stroke(draw2, 4.5, Stroke::new(1.0, theme.accent), egui::StrokeKind::Inside);
                }
                ui.painter().text(
                    draw2.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{FILM_STRIP} {}", crate::ui::i18n::text(ui, "design-slide")),
                    egui::FontId::proportional(11.0),
                    if is_slides { Color32::WHITE } else if hov2 > 0.0 { theme.text_primary } else { theme.text_secondary },
                );

                // Generous separation and Scrubber Timeline (in Slides mode)
                if timeline_mode == TimelineMode::Slides {
                    ui.add_space(16.0);
                    ui.painter().line_segment(
                        [Pos2::new(ui.cursor().min.x, ui.cursor().center().y - 10.0), Pos2::new(ui.cursor().min.x, ui.cursor().center().y + 10.0)],
                        Stroke::new(1.0, theme.border_default),
                    );
                    ui.add_space(16.0);

                    let scrubber_w = (ui.available_width() - 230.0).max(60.0);
                    let (track_rect, track_resp) = ui.allocate_exact_size(
                        Vec2::new(scrubber_w, 28.0),
                        egui::Sense::click_and_drag(),
                    );

                    let margin_x = 8.0f32;
                    let usable_w = (track_rect.width() - margin_x * 2.0).max(10.0);
                    let track_min_x = track_rect.min.x + margin_x;

                    if track_resp.clicked() || track_resp.dragged() {
                        if let Some(pos) = track_resp.interact_pointer_pos() {
                            let frac =
                                ((pos.x - track_min_x) / usable_w).clamp(0.0, 1.0);
                            scrub_to_ms = Some((frac * max_sec * 1000.0) as u64);
                        }
                    }
                    if track_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }

                    let track_bar = Rect::from_min_max(
                        Pos2::new(track_min_x, track_rect.center().y - 3.0),
                        Pos2::new(track_min_x + usable_w, track_rect.center().y + 3.0),
                    );
                    ui.painter().rect_filled(track_bar, 3.0, theme.border_default);

                    let progress_frac = (curr_sec / max_sec).clamp(0.0, 1.0);
                    let progress_w = track_bar.width() * progress_frac;
                    if progress_w > 0.0 {
                        let filled_rect = Rect::from_min_size(
                            track_bar.min,
                            Vec2::new(progress_w, track_bar.height()),
                        );
                        ui.painter().rect_filled(filled_rect, 3.0, theme.accent);
                    }

                    let knob_center = Pos2::new(track_bar.min.x + progress_w, track_bar.center().y);
                    let is_hot = track_resp.hovered() || track_resp.dragged();
                    let knob_radius = if is_hot { 7.5 } else { 5.5 };
                    ui.painter().circle_filled(knob_center, knob_radius, Color32::WHITE);
                    ui.painter().circle_stroke(
                        knob_center,
                        knob_radius,
                        Stroke::new(1.5, theme.accent),
                    );
                }

                // Time Counter label & Duration Quick Edit
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Collapse/Expand filmstrip button with on-hold and hover
                    let toggle_icon = if show_filmstrip { CARET_DOWN } else { CARET_UP };
                    let (tog_rect, tog_resp) = ui.allocate_exact_size(Vec2::new(28.0, 28.0), egui::Sense::click());
                    if tog_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    let tog_hov = ui.ctx().animate_bool_with_time(tog_resp.id.with("tog_hov"), tog_resp.hovered(), 0.12);
                    let tog_hold = ui.ctx().animate_bool_with_time(tog_resp.id.with("tog_hld"), tog_resp.is_pointer_button_down_on(), 0.12);
                    let tog_bg = theme.bg_app.lerp_to_gamma(theme.bg_card_hover, tog_hov);
                    let draw_tog = tog_rect.shrink(tog_hold * 0.6);

                    ui.painter().rect_filled(draw_tog, 6.0, tog_bg);
                    ui.painter().rect_stroke(
                        draw_tog,
                        6.0,
                        Stroke::new(1.0, if tog_hov > 0.0 { theme.accent } else { theme.border_default }),
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().text(
                        draw_tog.center(),
                        egui::Align2::CENTER_CENTER,
                        toggle_icon,
                        egui::FontId::proportional(13.0),
                        if tog_hov > 0.0 { theme.accent } else { theme.text_secondary },
                    );
                    if tog_resp.clicked() {
                        toggle_filmstrip = true;
                    }

                    // Optional timeline height cycle button in Layers mode
                    if show_filmstrip && timeline_mode == TimelineMode::Layers {
                        let (h_rect, h_resp) = ui.allocate_exact_size(Vec2::new(28.0, 28.0), egui::Sense::click());
                        if h_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        let h_hov = ui.ctx().animate_bool_with_time(h_resp.id.with("h_hov"), h_resp.hovered(), 0.12);
                        let h_hold = ui.ctx().animate_bool_with_time(h_resp.id.with("h_hld"), h_resp.is_pointer_button_down_on(), 0.12);
                        let h_bg = theme.bg_app.lerp_to_gamma(theme.bg_card_hover, h_hov);
                        let draw_h = h_rect.shrink(h_hold * 0.6);

                        ui.painter().rect_filled(draw_h, 6.0, h_bg);
                        ui.painter().rect_stroke(
                            draw_h,
                            6.0,
                            Stroke::new(1.0, if h_hov > 0.0 { theme.accent } else { theme.border_default }),
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().text(
                            draw_h.center(),
                            egui::Align2::CENTER_CENTER,
                            ARROWS_DOWN_UP,
                            egui::FontId::proportional(12.0),
                            if h_hov > 0.0 { theme.accent } else { theme.text_secondary },
                        );
                        if h_resp.clicked() {
                            cycle_height = true;
                        }
                        h_resp.on_hover_text(crate::ui::i18n::text(ui, "design-toggle-timeline-height"));
                    }

                    // Slide duration quick editor
                    let mut dur = current_slide_duration;
                    let dv = ui.add(
                        egui::DragValue::new(&mut dur)
                            .range(0.5..=120.0)
                            .speed(0.2)
                            .suffix("s"),
                    )
                    .on_hover_text(crate::ui::i18n::text(ui, "design-slide-duration"));
                    if dv.changed() {
                        new_slide_duration = Some(dur);
                    }

                    ui.label(RichText::new(CLOCK).size(13.0).color(theme.text_muted));

                    // Time Counter label
                    ui.label(
                        RichText::new(format!("{:.1}s / {:.1}s", curr_sec, max_sec))
                            .size(11.0)
                            .color(theme.text_secondary)
                            .monospace(),
                    );
                });
            });

            // --- 2. Body Area (Layers Timeline or Slides Filmstrip) ---
            if show_filmstrip {
                ui.add_space(4.0);
                let sep_rect = Rect::from_min_size(
                    Pos2::new(ui.cursor().min.x, ui.cursor().min.y),
                    Vec2::new(ui.available_width(), 1.0),
                );
                ui.painter().rect_filled(sep_rect, 0.0, theme.border_default);
                ui.add_space(5.0);

                match timeline_mode {
                    TimelineMode::Layers => {
                        super::animation_timeline::render_layers_timeline(
                            ui,
                            editor,
                            theme,
                            total_slide_ms,
                            curr_time_ms,
                            &mut scrub_to_ms,
                            &mut select_layer,
                            &mut start_drag,
                        );
                    }
                    TimelineMode::Slides => {
                        render_slides_filmstrip(
                            ui,
                            theme,
                            themes,
                            active_idx,
                            slide_count,
                            &transition_name,
                            &slide_docs,
                            &mut switch_to,
                            &mut add_requested,
                            &mut duplicate_requested,
                            &mut delete_requested,
                        );
                    }
                }
            }
        });

    // Execute requested actions
    if let Some(mode) = set_mode {
        editor.timeline_mode = mode;
    }
    if toggle_play {
        editor.toggle_animation();
    }
    if reset_play {
        editor.animation_time_ms = 0;
        if editor.animation_playing {
            editor.animation_started_at = Some(std::time::Instant::now());
        }
    }
    if toggle_loop {
        editor.animation_loop = !editor.animation_loop;
    }
    if toggle_filmstrip {
        editor.show_filmstrip = !editor.show_filmstrip;
    }
    if cycle_height {
        editor.timeline_height = if editor.timeline_height < 170.0 {
            240.0
        } else if editor.timeline_height < 290.0 {
            360.0
        } else {
            140.0
        };
        ui.ctx().request_repaint();
    }
    if let Some(ms) = scrub_to_ms {
        editor.pause_animation();
        editor.animation_time_ms = ms;
    }
    if let (Some(dur), Some(scene)) = (new_slide_duration, &mut editor.scene) {
        let active_idx = scene.active_slide_index;
        if let Some(slide) = scene.slides.get_mut(active_idx) {
            slide.duration_secs = dur;
        } else {
            scene.slide_duration_secs = dur;
        }
    }
    if let Some(idx) = select_layer {
        editor.select_layer(idx, false);
        editor.inspector_tab = 1;
        editor.layer_inspector_tab = Some(1);
    }
    if let Some(drag) = start_drag {
        editor.timeline_drag = Some(drag);
    }
    if let Some(target) = switch_to {
        editor.pause_animation();
        editor.animation_time_ms = 0;
        editor.selected = None;
        editor.selected_layers.clear();
        if let Some(scene) = &mut editor.scene {
            scene.set_active_slide(target);
        }
    }
    if add_requested {
        editor.pause_animation();
        editor.animation_time_ms = 0;
        editor.selected = None;
        editor.selected_layers.clear();
        if let Some(scene) = &mut editor.scene {
            scene.add_slide();
        }
    }
    if duplicate_requested {
        editor.pause_animation();
        editor.animation_time_ms = 0;
        editor.selected = None;
        editor.selected_layers.clear();
        if let Some(scene) = &mut editor.scene {
            scene.duplicate_current_slide();
        }
    }
    if let Some(del_idx) = delete_requested {
        editor.pause_animation();
        editor.animation_time_ms = 0;
        editor.selected = None;
        editor.selected_layers.clear();
        if let Some(scene) = &mut editor.scene {
            scene.delete_slide(del_idx);
        }
    }
}

/// Renders the 16:9 filmstrip of slides with live miniatures
fn render_slides_filmstrip(
    ui: &mut egui::Ui,
    theme: &AppTheme,
    themes: &[PresentationTheme],
    active_idx: usize,
    slide_count: usize,
    transition_name: &str,
    slide_docs: &[(usize, String, f32, crate::core::design::DesignDocument)],
    switch_to: &mut Option<usize>,
    add_requested: &mut bool,
    duplicate_requested: &mut bool,
    delete_requested: &mut Option<usize>,
) {
    egui::ScrollArea::horizontal()
        .id_salt("design_slide_filmstrip")
        .max_height(78.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;

                for (idx, name, duration_secs, doc) in slide_docs {
                    let idx = *idx;
                    let is_active = idx == active_idx;

                    // Transition arrow
                    if idx > 0 {
                        ui.label(RichText::new("›").size(14.0).color(theme.text_muted))
                            .on_hover_text(transition_name);
                    }

                    // Card 104x74 with hover & hold emphasis
                    let card_size = Vec2::new(104.0, 74.0);
                    let (rect, resp) = ui.allocate_exact_size(card_size, egui::Sense::click());
                    if resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    let card_hov = ui.ctx().animate_bool_with_time(resp.id.with("card_hov"), resp.hovered(), 0.12);
                    let card_hold = ui.ctx().animate_bool_with_time(resp.id.with("card_hold"), resp.is_pointer_button_down_on(), 0.12);

                    let thumb_rect = Rect::from_min_size(rect.min, Vec2::new(104.0, 56.0)).shrink(card_hold * 0.5);
                    let label_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x, rect.min.y + 57.0),
                        Vec2::new(104.0, 16.0),
                    );

                    // Miniature slide preview
                    ui.painter().rect_filled(thumb_rect, 5.0, Color32::from_rgb(18, 21, 26));
                    crate::render::design::paint_themed(
                        &ui.painter().with_clip_rect(thumb_rect),
                        thumb_rect,
                        doc,
                        themes,
                    );

                    // Top-left slide number badge
                    let badge_rect = Rect::from_min_size(
                        thumb_rect.min + egui::vec2(3.0, 3.0),
                        Vec2::new(18.0, 13.0),
                    );
                    let badge_bg = if is_active {
                        theme.accent
                    } else {
                        Color32::from_rgba_unmultiplied(0, 0, 0, 180)
                    };
                    ui.painter().rect_filled(badge_rect, 3.0, badge_bg);
                    ui.painter().text(
                        badge_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("#{}", idx + 1),
                        egui::FontId::proportional(9.0),
                        Color32::WHITE,
                    );

                    // Bottom-right duration badge
                    let dur_badge_rect = Rect::from_min_size(
                        thumb_rect.max - egui::vec2(30.0, 15.0),
                        Vec2::new(27.0, 12.0),
                    );
                    ui.painter().rect_filled(
                        dur_badge_rect,
                        3.0,
                        Color32::from_rgba_unmultiplied(0, 0, 0, 180),
                    );
                    ui.painter().text(
                        dur_badge_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("{:.1}s", duration_secs),
                        egui::FontId::proportional(8.5),
                        Color32::from_rgb(220, 225, 230),
                    );

                    // Outline stroke around thumbnail
                    let stroke_color = if is_active {
                        theme.accent
                    } else if card_hov > 0.0 {
                        theme.accent.gamma_multiply(0.75)
                    } else {
                        theme.border_default
                    };
                    ui.painter().rect_stroke(
                        thumb_rect,
                        5.0,
                        Stroke::new(if is_active { 2.0 } else { 1.0 }, stroke_color),
                        egui::StrokeKind::Inside,
                    );

                    // Truncated slide name
                    let mut label_ui = ui.new_child(egui::UiBuilder::new().max_rect(label_rect));
                    label_ui.centered_and_justified(|ui| {
                        ui.add(
                            egui::Label::new(
                                RichText::new(name).size(9.5).color(if is_active {
                                    theme.text_primary
                                } else if card_hov > 0.0 {
                                    theme.text_primary
                                } else {
                                    theme.text_secondary
                                }),
                            )
                            .truncate(),
                        );
                    });

                    if resp.clicked() && !is_active {
                        *switch_to = Some(idx);
                    }

                    resp.context_menu(|ui| {
                        if ui
                            .button(crate::ui::i18n::format(
                                ui,
                                "design-duplicate-slide",
                                &[("icon", COPY)],
                            ))
                            .clicked()
                        {
                            *duplicate_requested = true;
                            ui.close();
                        }
                        if slide_count > 1
                            && ui
                                .button(
                                    RichText::new(crate::ui::i18n::format(
                                        ui,
                                        "design-delete-slide",
                                        &[("icon", TRASH)],
                                    ))
                                    .color(theme.status_danger),
                                )
                                .clicked()
                        {
                            *delete_requested = Some(idx);
                            ui.close();
                        }
                    });
                }

                // Add slide card with hover & hold emphasis
                let add_size = Vec2::new(104.0, 74.0);
                let (add_rect, add_resp) = ui.allocate_exact_size(add_size, egui::Sense::click());
                if add_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                let add_hov = ui.ctx().animate_bool_with_time(add_resp.id.with("add_hov"), add_resp.hovered(), 0.12);
                let add_hold = ui.ctx().animate_bool_with_time(add_resp.id.with("add_hold"), add_resp.is_pointer_button_down_on(), 0.12);

                let add_box = Rect::from_min_size(add_rect.min, Vec2::new(104.0, 56.0)).shrink(add_hold * 0.5);

                let add_bg = if add_hov > 0.0 {
                    theme.bg_app.lerp_to_gamma(theme.bg_card_hover, add_hov)
                } else {
                    Color32::from_rgba_unmultiplied(0, 0, 0, 40)
                };
                let add_stroke = if add_hov > 0.0 {
                    theme.accent
                } else {
                    theme.border_default
                };

                ui.painter().rect_filled(add_box, 5.0, add_bg);
                ui.painter().rect_stroke(
                    add_box,
                    5.0,
                    Stroke::new(1.2, add_stroke),
                    egui::StrokeKind::Inside,
                );

                ui.painter().text(
                    add_box.center() - egui::vec2(0.0, 6.0),
                    egui::Align2::CENTER_CENTER,
                    PLUS,
                    egui::FontId::proportional(16.0),
                    if add_hov > 0.0 { theme.accent } else { theme.text_secondary },
                );
                ui.painter().text(
                    add_box.center() + egui::vec2(0.0, 11.0),
                    egui::Align2::CENTER_CENTER,
                    crate::ui::i18n::text(ui, "design-add-slide"),
                    egui::FontId::proportional(9.0),
                    if add_hov > 0.0 { theme.accent } else { theme.text_muted },
                );

                if add_resp.clicked() {
                    *add_requested = true;
                }
            });
        });
}
