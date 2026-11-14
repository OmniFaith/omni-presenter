use super::{DesignEditor, TimelineDrag, TimelineDragHandle};
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Stroke, Vec2};

/// Renders the multi-track layer timeline with individual layer timing bars
pub(super) fn render_layers_timeline(
    ui: &mut egui::Ui,
    editor: &DesignEditor,
    theme: &AppTheme,
    total_slide_ms: u64,
    curr_time_ms: u64,
    scrub_to_ms: &mut Option<u64>,
    select_layer: &mut Option<usize>,
    start_drag: &mut Option<TimelineDrag>,
) {
    let Some(scene) = editor.scene.as_ref() else { return; };
    if scene.layers.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.label(
                RichText::new(crate::ui::i18n::text(ui, "design-no-layers-on-slide"))
                    .size(11.0)
                    .color(theme.text_muted),
            );
            ui.add_space(10.0);
        });
        return;
    }

    let label_width = 135.0f32;

    // --- Time Ruler Header ---
    let (ruler_rect, ruler_resp) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 18.0),
        egui::Sense::click_and_drag(),
    );

    let ruler_track_rect = Rect::from_min_max(
        Pos2::new(ruler_rect.min.x + label_width, ruler_rect.min.y),
        ruler_rect.max,
    );

    if ruler_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    // Clicking / dragging on the ruler scrubs the playhead
    if ruler_resp.clicked() || ruler_resp.dragged() {
        if let Some(pos) = ruler_resp.interact_pointer_pos() {
            let frac = ((pos.x - ruler_track_rect.min.x) / ruler_track_rect.width()).clamp(0.0, 1.0);
            *scrub_to_ms = Some((frac * total_slide_ms as f32) as u64);
        }
    }

    // Paint ruler background & ticks
    ui.painter().rect_filled(ruler_rect, 3.0, Color32::from_rgba_unmultiplied(0, 0, 0, 30));
    ui.painter().text(
        Pos2::new(ruler_rect.min.x + 8.0, ruler_rect.center().y),
        egui::Align2::LEFT_CENTER,
        crate::ui::i18n::text(ui, "design-layers"),
        egui::FontId::proportional(10.0),
        theme.text_muted,
    );

    let max_sec = total_slide_ms as f32 / 1000.0;
    let sec_step = if max_sec > 15.0 { 2.0 } else { 1.0 };
    let mut s = 0.0f32;
    while s <= max_sec + 0.01 {
        let x = ruler_track_rect.min.x + (s / max_sec) * ruler_track_rect.width();
        if x <= ruler_track_rect.max.x + 1.0 {
            ui.painter().line_segment(
                [Pos2::new(x, ruler_rect.max.y - 6.0), Pos2::new(x, ruler_rect.max.y)],
                Stroke::new(1.0, theme.border_default),
            );
            ui.painter().text(
                Pos2::new(x + 2.0, ruler_rect.min.y + 2.0),
                egui::Align2::LEFT_TOP,
                format!("{:.0}s", s),
                egui::FontId::proportional(8.5),
                theme.text_muted,
            );
        }
        s += sec_step;
    }

    ui.add_space(2.0);

    // --- Vertical list of layer tracks ---
    let tracks_h = editor.timeline_height.clamp(60.0, 600.0);
    let tracks_scroll = egui::ScrollArea::vertical()
        .id_salt("design_layer_tracks_scroll")
        .max_height(tracks_h)
        .min_scrolled_height(tracks_h)
        .auto_shrink([false, false]);

    let tracks_content = tracks_scroll.show(ui, |ui| {
        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        let primary_pressed = ui.input(|i| i.pointer.primary_pressed());

        for (idx, layer) in scene.layers.iter().enumerate() {
            let is_selected = editor.is_layer_selected(idx);
            let row_h = 24.0f32;
            let (row_rect, _) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), row_h),
                egui::Sense::hover(),
            );

            let label_rect = Rect::from_min_max(
                row_rect.min,
                Pos2::new(row_rect.min.x + label_width - 6.0, row_rect.max.y),
            );
            let lane_rect = Rect::from_min_max(
                Pos2::new(row_rect.min.x + label_width, row_rect.min.y),
                row_rect.max,
            );

            // Left: Layer Item pill button
            let label_resp = ui.interact(label_rect, ui.make_persistent_id(("timeline_layer_lbl", idx)), egui::Sense::click());
            if label_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if label_resp.clicked() {
                *select_layer = Some(idx);
            }

            let lbl_hov = ui.ctx().animate_bool_with_time(label_resp.id.with("hov"), label_resp.hovered(), 0.12);
            let lbl_hold = ui.ctx().animate_bool_with_time(label_resp.id.with("hold"), label_resp.is_pointer_button_down_on(), 0.12);

            let label_bg = if is_selected {
                theme.accent_soft.lerp_to_gamma(theme.accent, lbl_hov * 0.1)
            } else if lbl_hov > 0.0 || lbl_hold > 0.0 {
                theme.bg_card_hover.lerp_to_gamma(theme.accent_soft, lbl_hold)
            } else {
                Color32::TRANSPARENT
            };
            let draw_lbl = label_rect.shrink(lbl_hold * 0.5);
            ui.painter().rect_filled(draw_lbl, 4.0, label_bg);
            if is_selected || lbl_hov > 0.0 {
                ui.painter().rect_stroke(
                    draw_lbl,
                    4.0,
                    Stroke::new(1.0, if is_selected { theme.accent } else { theme.border_default }),
                    egui::StrokeKind::Inside,
                );
            }

            // Layer icon & name
            let icon_str = super::sidebar::content_icon(&layer.content);
            ui.painter().text(
                Pos2::new(draw_lbl.min.x + 6.0, draw_lbl.center().y),
                egui::Align2::LEFT_CENTER,
                icon_str,
                egui::FontId::proportional(11.0),
                if is_selected { theme.accent } else { theme.text_secondary },
            );
            let truncated_name = if layer.name.chars().count() > 14 {
                format!("{}…", layer.name.chars().take(13).collect::<String>())
            } else {
                layer.name.clone()
            };
            ui.painter().text(
                Pos2::new(draw_lbl.min.x + 22.0, draw_lbl.center().y),
                egui::Align2::LEFT_CENTER,
                truncated_name,
                egui::FontId::proportional(10.0),
                if is_selected { theme.text_primary } else { theme.text_secondary },
            );

            // Right: Layer Track Lane & Timeline Bar
            ui.painter().line_segment(
                [Pos2::new(lane_rect.min.x, lane_rect.center().y), Pos2::new(lane_rect.max.x, lane_rect.center().y)],
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 12)),
            );

            let start_ms = layer.animation.start_ms;
            let end_ms = if layer.animation.end_ms == 0 || layer.animation.end_ms > total_slide_ms as u32 {
                total_slide_ms as u32
            } else {
                layer.animation.end_ms
            };

            let start_x = lane_rect.min.x + (start_ms as f32 / total_slide_ms as f32) * lane_rect.width();
            let end_x = lane_rect.min.x + (end_ms as f32 / total_slide_ms as f32) * lane_rect.width();

            let bar_rect = Rect::from_min_max(
                Pos2::new(start_x, row_rect.min.y + 2.0),
                Pos2::new(end_x.max(start_x + 14.0), row_rect.max.y - 2.0),
            );

            // Drag detection
            let left_zone = Rect::from_min_max(
                Pos2::new(bar_rect.min.x - 4.0, bar_rect.min.y),
                Pos2::new(bar_rect.min.x + 8.0, bar_rect.max.y),
            );
            let right_zone = Rect::from_min_max(
                Pos2::new(bar_rect.max.x - 8.0, bar_rect.min.y),
                Pos2::new(bar_rect.max.x + 4.0, bar_rect.max.y),
            );

            let mut is_bar_hovered = false;
            let is_dragging_this = editor.timeline_drag.as_ref().is_some_and(|d| d.layer_index == idx);

            if let Some(pos) = pointer_pos {
                if left_zone.contains(pos) {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                    is_bar_hovered = true;
                    if primary_pressed && editor.timeline_drag.is_none() {
                        *start_drag = Some(TimelineDrag {
                            layer_index: idx,
                            handle: TimelineDragHandle::Start,
                            initial_pointer_x: pos.x,
                            initial_start_ms: start_ms,
                            initial_end_ms: end_ms,
                        });
                        *select_layer = Some(idx);
                    }
                } else if right_zone.contains(pos) {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                    is_bar_hovered = true;
                    if primary_pressed && editor.timeline_drag.is_none() {
                        *start_drag = Some(TimelineDrag {
                            layer_index: idx,
                            handle: TimelineDragHandle::End,
                            initial_pointer_x: pos.x,
                            initial_start_ms: start_ms,
                            initial_end_ms: end_ms,
                        });
                        *select_layer = Some(idx);
                    }
                } else if bar_rect.contains(pos) {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                    is_bar_hovered = true;
                    if primary_pressed && editor.timeline_drag.is_none() {
                        *start_drag = Some(TimelineDrag {
                            layer_index: idx,
                            handle: TimelineDragHandle::Move,
                            initial_pointer_x: pos.x,
                            initial_start_ms: start_ms,
                            initial_end_ms: end_ms,
                        });
                        *select_layer = Some(idx);
                    }
                }
            }

            // Bar background & stroke with hover/active emphasis
            let bar_bg = if is_selected || is_dragging_this {
                theme.accent
            } else if is_bar_hovered {
                Color32::from_rgb(48, 66, 90)
            } else {
                Color32::from_rgb(38, 52, 70)
            };
            let bar_stroke = if is_selected || is_dragging_this {
                Stroke::new(1.5, Color32::WHITE)
            } else if is_bar_hovered {
                Stroke::new(1.2, Color32::WHITE)
            } else {
                Stroke::new(1.0, theme.accent)
            };
            ui.painter().rect_filled(bar_rect, 4.0, bar_bg);
            ui.painter().rect_stroke(bar_rect, 4.0, bar_stroke, egui::StrokeKind::Inside);

            // Trim handles indicators on edges
            let handle_w = 5.0f32;
            let left_handle = Rect::from_min_max(bar_rect.min, Pos2::new(bar_rect.min.x + handle_w, bar_rect.max.y));
            let right_handle = Rect::from_min_max(Pos2::new(bar_rect.max.x - handle_w, bar_rect.min.y), bar_rect.max);
            ui.painter().rect_filled(left_handle, 2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 75));
            ui.painter().rect_filled(right_handle, 2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 75));

            // Time text inside bar if wide enough
            if bar_rect.width() > 64.0 {
                let time_text = format!("{:.1}s - {:.1}s", start_ms as f32 / 1000.0, end_ms as f32 / 1000.0);
                ui.painter().text(
                    bar_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    time_text,
                    egui::FontId::proportional(8.5),
                    Color32::WHITE,
                );
            }

            ui.add_space(2.0);
        }
    });

    // --- Vertical Playhead Needle across tracks ---
    let needle_frac = (curr_time_ms as f32 / total_slide_ms as f32).clamp(0.0, 1.0);
    let needle_x = ruler_track_rect.min.x + needle_frac * ruler_track_rect.width();
    let needle_top = ruler_rect.min.y;
    let needle_bottom = tracks_content.inner_rect.max.y.max(ruler_rect.bottom() + 20.0);

    // Vertical line
    ui.painter().line_segment(
        [Pos2::new(needle_x, needle_top), Pos2::new(needle_x, needle_bottom)],
        Stroke::new(1.5, theme.accent),
    );

    // Needle cap/triangle on ruler
    let cap_pts = [
        Pos2::new(needle_x - 4.5, needle_top),
        Pos2::new(needle_x + 4.5, needle_top),
        Pos2::new(needle_x, needle_top + 6.0),
    ];
    ui.painter().add(egui::Shape::convex_polygon(
        cap_pts.to_vec(),
        theme.accent,
        Stroke::NONE,
    ));
}

