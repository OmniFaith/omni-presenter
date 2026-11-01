use crate::core::state::{ControlPanelState, ThemeDragHandle};
use crate::render::paint_theme_bg;
use eframe::egui::{self, Color32, Stroke, StrokeKind, Vec2};

pub(super) fn preview(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    canvas_area_rect: egui::Rect,
) -> bool {
    let mut push_snapshot = false;
    ui.painter()
        .rect_filled(canvas_area_rect, 10.0, state.theme.bg_pill);
    ui.painter().rect_stroke(
        canvas_area_rect,
        10.0,
        Stroke::new(1.0, state.theme.border_default),
        StrokeKind::Inside,
    );

    let aspect = state.presentation_aspect_ratio.clamp(1.0, 2.5);
    let zoom = state.theme_editor.zoom.clamp(0.5, 1.5);

    let avail_w = (canvas_area_rect.width() - 44.0).max(100.0);
    let avail_h = (canvas_area_rect.height() - 44.0).max(100.0);

    let mut canvas_w = (avail_w * 0.96 * zoom).min(avail_w);
    let mut canvas_h = canvas_w / aspect;

    if canvas_h > avail_h {
        canvas_h = avail_h;
        canvas_w = canvas_h * aspect;
    }

    let canvas_center = canvas_area_rect.center();
    let canvas_rect = egui::Rect::from_center_size(canvas_center, Vec2::new(canvas_w, canvas_h));

    let slide_shadow = egui::epaint::Shadow {
        offset: [0, 10],
        blur: 28,
        spread: 0,
        color: Color32::from_black_alpha(75),
    };
    ui.painter().add(slide_shadow.as_shape(canvas_rect, 10.0));

    paint_theme_bg(ui.painter(), canvas_rect, &state.theme_editor.editing, 10.0);
    ui.painter().rect_stroke(
        canvas_rect,
        10.0,
        Stroke::new(1.5, state.theme.border_active),
        StrokeKind::Inside,
    );

    let box_x = state.theme_editor.editing.box_x.clamp(0.01, 0.90);
    let box_y = state.theme_editor.editing.box_y.clamp(0.01, 0.90);
    let box_w = state.theme_editor.editing.box_w.clamp(0.10, 0.99 - box_x);
    let box_h = state.theme_editor.editing.box_h.clamp(0.10, 0.99 - box_y);

    let box_min =
        canvas_rect.min + egui::vec2(box_x * canvas_rect.width(), box_y * canvas_rect.height());
    let box_size = egui::vec2(box_w * canvas_rect.width(), box_h * canvas_rect.height());
    let bounding_box_rect = egui::Rect::from_min_size(box_min, box_size);

    let sample_lyric = match state.theme_editor.sample_index % 3 {
        1 => "settings-for-god-so-loved-the-world-that-he-gave-his-label",
        2 => "settings-to-him-be-the-glory-forever-amen",
        _ => "settings-though-i-may-weep-joy-will-come-in-the-morning",
    };
    crate::services::effects::render(
        ui.painter(),
        canvas_rect,
        &state.theme_editor.editing,
        state.settings.language.text(sample_lyric),
        egui::Id::unique("theme_editor_transition"),
        !state.settings.operator().reduce_motion,
    );

    let handle_radius = 5.0;
    let handles = [
        (
            ThemeDragHandle::TopLeft,
            bounding_box_rect.min,
            egui::CursorIcon::ResizeNwSe,
        ),
        (
            ThemeDragHandle::TopCenter,
            egui::pos2(bounding_box_rect.center().x, bounding_box_rect.min.y),
            egui::CursorIcon::ResizeVertical,
        ),
        (
            ThemeDragHandle::TopRight,
            egui::pos2(bounding_box_rect.max.x, bounding_box_rect.min.y),
            egui::CursorIcon::ResizeNeSw,
        ),
        (
            ThemeDragHandle::LeftCenter,
            egui::pos2(bounding_box_rect.min.x, bounding_box_rect.center().y),
            egui::CursorIcon::ResizeHorizontal,
        ),
        (
            ThemeDragHandle::RightCenter,
            egui::pos2(bounding_box_rect.max.x, bounding_box_rect.center().y),
            egui::CursorIcon::ResizeHorizontal,
        ),
        (
            ThemeDragHandle::BottomLeft,
            egui::pos2(bounding_box_rect.min.x, bounding_box_rect.max.y),
            egui::CursorIcon::ResizeNeSw,
        ),
        (
            ThemeDragHandle::BottomCenter,
            egui::pos2(bounding_box_rect.center().x, bounding_box_rect.max.y),
            egui::CursorIcon::ResizeVertical,
        ),
        (
            ThemeDragHandle::BottomRight,
            bounding_box_rect.max,
            egui::CursorIcon::ResizeNwSe,
        ),
    ];

    // Desenha linha de seleção e alças somente se o texto estiver selecionado
    if state.theme_editor.is_text_selected {
        ui.painter().rect_stroke(
            bounding_box_rect,
            state
                .theme_editor
                .editing
                .box_radius
                .min(bounding_box_rect.height() * 0.5),
            Stroke::new(1.5, state.theme.accent),
            StrokeKind::Inside,
        );

        for (_handle_kind, pos, _cursor) in &handles {
            ui.painter()
                .circle_filled(*pos, handle_radius, Color32::WHITE);
            ui.painter().circle_stroke(
                *pos,
                handle_radius,
                Stroke::new(1.5, state.theme.accent),
            );
        }
    }

    let any_popup_open = egui::Popup::is_any_open(ui.ctx());
    if any_popup_open {
        state.theme_editor.active_handle = None;
    }

    let pointer_pos = ui.input(|i| i.pointer.interact_pos());
    let is_pointer_down = ui.input(|i| i.pointer.primary_down());
    let primary_clicked = ui.input(|i| i.pointer.primary_clicked());
    let drag_delta = ui.input(|i| i.pointer.delta());

    if !any_popup_open
        && let Some(pos) = pointer_pos
            && canvas_rect.contains(pos) {
                // Seleciona ao clicar no texto/alças, deseleciona ao clicar fora no canvas
                if primary_clicked {
                    let clicked_handle = if state.theme_editor.is_text_selected {
                        handles
                            .iter()
                            .any(|(_, h_pos, _)| h_pos.distance(pos) <= handle_radius + 4.0)
                    } else {
                        false
                    };

                    if bounding_box_rect.contains(pos) || clicked_handle {
                        state.theme_editor.is_text_selected = true;
                        state.theme_editor.sidebar_tab = 1; // Alterna automaticamente para a aba "Texto"
                    } else {
                        state.theme_editor.is_text_selected = false;
                        state.theme_editor.active_handle = None;
                        state.theme_editor.sidebar_tab = 0; // Alterna automaticamente para a aba "Fundo"
                    }
                }

                // Inicia arrasto da caixa ou alça
                if is_pointer_down && state.theme_editor.active_handle.is_none() {
                    if state.theme_editor.is_text_selected {
                        for (handle_kind, h_pos, _cursor) in &handles {
                            if h_pos.distance(pos) <= handle_radius + 4.0 {
                                state.theme_editor.active_handle = Some(*handle_kind);
                                state.theme_editor.sidebar_tab = 1;
                                break;
                            }
                        }
                    }
                    if state.theme_editor.active_handle.is_none() && bounding_box_rect.contains(pos)
                    {
                        state.theme_editor.is_text_selected = true;
                        state.theme_editor.sidebar_tab = 1;
                        state.theme_editor.active_handle = Some(ThemeDragHandle::MoveBox);
                    }
                }

                // Atualiza o cursor do mouse
                if state.theme_editor.active_handle.is_none() {
                    let mut found_hover = false;
                    if state.theme_editor.is_text_selected {
                        for (_handle_kind, h_pos, cursor) in &handles {
                            if h_pos.distance(pos) <= handle_radius + 4.0 {
                                ui.ctx().set_cursor_icon(*cursor);
                                found_hover = true;
                                break;
                            }
                        }
                        if !found_hover && bounding_box_rect.contains(pos) {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::Move);
                        }
                    } else if bounding_box_rect.contains(pos) {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                }
            }

    let mut active_snap_x = None;
    let mut active_snap_y = None;

    if !any_popup_open
        && is_pointer_down
        && (drag_delta.x.abs() > 0.01 || drag_delta.y.abs() > 0.01)
    {
        if let Some(handle) = state.theme_editor.active_handle {
            let norm_dx = drag_delta.x / canvas_rect.width();
            let norm_dy = drag_delta.y / canvas_rect.height();

            let guide_xs = [0.05, 0.10, 0.50, 0.90, 0.95];
            let guide_ys = [0.05, 0.10, 0.50, 0.65, 0.70, 0.90, 0.95];
            let threshold_x = 9.0 / canvas_rect.width();
            let threshold_y = 9.0 / canvas_rect.height();

            let snap_val = |val: f32, guides: &[f32], thresh: f32| -> (f32, Option<f32>) {
                let mut best = None;
                let mut min_d = thresh;
                for &g in guides {
                    let d = (val - g).abs();
                    if d < min_d {
                        min_d = d;
                        best = Some(g);
                    }
                }
                if let Some(g) = best {
                    (g, Some(g))
                } else {
                    (val, None)
                }
            };

            match handle {
                ThemeDragHandle::MoveBox => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Move);
                    let mut target_x = (state.theme_editor.editing.box_x + norm_dx)
                        .clamp(0.01, 0.99 - box_w);
                    let mut target_y = (state.theme_editor.editing.box_y + norm_dy)
                        .clamp(0.01, 0.99 - box_h);

                    // Snap horizontal (esquerda, centro, direita)
                    let anchors_x = [target_x, target_x + box_w * 0.5, target_x + box_w];
                    let offsets_x = [0.0, box_w * 0.5, box_w];
                    let mut min_dx = threshold_x;
                    for (anchor, offset) in anchors_x.iter().zip(offsets_x.iter()) {
                        for &gx in &guide_xs {
                            let d = (anchor - gx).abs();
                            if d < min_dx {
                                min_dx = d;
                                let candidate_x = gx - offset;
                                if candidate_x >= 0.01 && candidate_x + box_w <= 0.99 {
                                    target_x = candidate_x;
                                    active_snap_x = Some(gx);
                                }
                            }
                        }
                    }

                    // Snap vertical (topo, centro, base)
                    let anchors_y = [target_y, target_y + box_h * 0.5, target_y + box_h];
                    let offsets_y = [0.0, box_h * 0.5, box_h];
                    let mut min_dy = threshold_y;
                    for (anchor, offset) in anchors_y.iter().zip(offsets_y.iter()) {
                        for &gy in &guide_ys {
                            let d = (anchor - gy).abs();
                            if d < min_dy {
                                min_dy = d;
                                let candidate_y = gy - offset;
                                if candidate_y >= 0.01 && candidate_y + box_h <= 0.99 {
                                    target_y = candidate_y;
                                    active_snap_y = Some(gy);
                                }
                            }
                        }
                    }

                    state.theme_editor.editing.box_x = target_x;
                    state.theme_editor.editing.box_y = target_y;
                    push_snapshot = true;
                }
                ThemeDragHandle::TopLeft => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                    let mut new_x = (state.theme_editor.editing.box_x + norm_dx).clamp(
                        0.01,
                        state.theme_editor.editing.box_x + state.theme_editor.editing.box_w - 0.10,
                    );
                    let mut new_y = (state.theme_editor.editing.box_y + norm_dy).clamp(
                        0.01,
                        state.theme_editor.editing.box_y + state.theme_editor.editing.box_h - 0.10,
                    );
                    let (snapped_x, sx) = snap_val(new_x, &guide_xs, threshold_x);
                    if snapped_x
                        <= state.theme_editor.editing.box_x + state.theme_editor.editing.box_w
                            - 0.10
                    {
                        new_x = snapped_x;
                        active_snap_x = sx;
                    }
                    let (snapped_y, sy) = snap_val(new_y, &guide_ys, threshold_y);
                    if snapped_y
                        <= state.theme_editor.editing.box_y + state.theme_editor.editing.box_h
                            - 0.10
                    {
                        new_y = snapped_y;
                        active_snap_y = sy;
                    }
                    state.theme_editor.editing.box_w += state.theme_editor.editing.box_x - new_x;
                    state.theme_editor.editing.box_h += state.theme_editor.editing.box_y - new_y;
                    state.theme_editor.editing.box_x = new_x;
                    state.theme_editor.editing.box_y = new_y;
                    push_snapshot = true;
                }
                ThemeDragHandle::TopCenter => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
                    let mut new_y = (state.theme_editor.editing.box_y + norm_dy).clamp(
                        0.01,
                        state.theme_editor.editing.box_y + state.theme_editor.editing.box_h - 0.10,
                    );
                    let (snapped_y, sy) = snap_val(new_y, &guide_ys, threshold_y);
                    if snapped_y
                        <= state.theme_editor.editing.box_y + state.theme_editor.editing.box_h
                            - 0.10
                    {
                        new_y = snapped_y;
                        active_snap_y = sy;
                    }
                    state.theme_editor.editing.box_h += state.theme_editor.editing.box_y - new_y;
                    state.theme_editor.editing.box_y = new_y;
                    push_snapshot = true;
                }
                ThemeDragHandle::TopRight => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNeSw);
                    let mut new_y = (state.theme_editor.editing.box_y + norm_dy).clamp(
                        0.01,
                        state.theme_editor.editing.box_y + state.theme_editor.editing.box_h - 0.10,
                    );
                    let (snapped_y, sy) = snap_val(new_y, &guide_ys, threshold_y);
                    if snapped_y
                        <= state.theme_editor.editing.box_y + state.theme_editor.editing.box_h
                            - 0.10
                    {
                        new_y = snapped_y;
                        active_snap_y = sy;
                    }
                    let mut new_right = (state.theme_editor.editing.box_x
                        + state.theme_editor.editing.box_w
                        + norm_dx)
                        .clamp(state.theme_editor.editing.box_x + 0.10, 0.99);
                    let (snapped_r, sx) = snap_val(new_right, &guide_xs, threshold_x);
                    if snapped_r >= state.theme_editor.editing.box_x + 0.10 {
                        new_right = snapped_r;
                        active_snap_x = sx;
                    }
                    state.theme_editor.editing.box_w = new_right - state.theme_editor.editing.box_x;
                    state.theme_editor.editing.box_h += state.theme_editor.editing.box_y - new_y;
                    state.theme_editor.editing.box_y = new_y;
                    push_snapshot = true;
                }
                ThemeDragHandle::LeftCenter => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                    let mut new_x = (state.theme_editor.editing.box_x + norm_dx).clamp(
                        0.01,
                        state.theme_editor.editing.box_x + state.theme_editor.editing.box_w - 0.10,
                    );
                    let (snapped_x, sx) = snap_val(new_x, &guide_xs, threshold_x);
                    if snapped_x
                        <= state.theme_editor.editing.box_x + state.theme_editor.editing.box_w
                            - 0.10
                    {
                        new_x = snapped_x;
                        active_snap_x = sx;
                    }
                    state.theme_editor.editing.box_w += state.theme_editor.editing.box_x - new_x;
                    state.theme_editor.editing.box_x = new_x;
                    push_snapshot = true;
                }
                ThemeDragHandle::RightCenter => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                    let mut new_right = (state.theme_editor.editing.box_x
                        + state.theme_editor.editing.box_w
                        + norm_dx)
                        .clamp(state.theme_editor.editing.box_x + 0.10, 0.99);
                    let (snapped_r, sx) = snap_val(new_right, &guide_xs, threshold_x);
                    if snapped_r >= state.theme_editor.editing.box_x + 0.10 {
                        new_right = snapped_r;
                        active_snap_x = sx;
                    }
                    state.theme_editor.editing.box_w = new_right - state.theme_editor.editing.box_x;
                    push_snapshot = true;
                }
                ThemeDragHandle::BottomLeft => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNeSw);
                    let mut new_x = (state.theme_editor.editing.box_x + norm_dx).clamp(
                        0.01,
                        state.theme_editor.editing.box_x + state.theme_editor.editing.box_w - 0.10,
                    );
                    let (snapped_x, sx) = snap_val(new_x, &guide_xs, threshold_x);
                    if snapped_x
                        <= state.theme_editor.editing.box_x + state.theme_editor.editing.box_w
                            - 0.10
                    {
                        new_x = snapped_x;
                        active_snap_x = sx;
                    }
                    let mut new_bottom = (state.theme_editor.editing.box_y
                        + state.theme_editor.editing.box_h
                        + norm_dy)
                        .clamp(state.theme_editor.editing.box_y + 0.10, 0.99);
                    let (snapped_b, sy) = snap_val(new_bottom, &guide_ys, threshold_y);
                    if snapped_b >= state.theme_editor.editing.box_y + 0.10 {
                        new_bottom = snapped_b;
                        active_snap_y = sy;
                    }
                    state.theme_editor.editing.box_w += state.theme_editor.editing.box_x - new_x;
                    state.theme_editor.editing.box_x = new_x;
                    state.theme_editor.editing.box_h =
                        new_bottom - state.theme_editor.editing.box_y;
                    push_snapshot = true;
                }
                ThemeDragHandle::BottomCenter => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
                    let mut new_bottom = (state.theme_editor.editing.box_y
                        + state.theme_editor.editing.box_h
                        + norm_dy)
                        .clamp(state.theme_editor.editing.box_y + 0.10, 0.99);
                    let (snapped_b, sy) = snap_val(new_bottom, &guide_ys, threshold_y);
                    if snapped_b >= state.theme_editor.editing.box_y + 0.10 {
                        new_bottom = snapped_b;
                        active_snap_y = sy;
                    }
                    state.theme_editor.editing.box_h =
                        new_bottom - state.theme_editor.editing.box_y;
                    push_snapshot = true;
                }
                ThemeDragHandle::BottomRight => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                    let mut new_right = (state.theme_editor.editing.box_x
                        + state.theme_editor.editing.box_w
                        + norm_dx)
                        .clamp(state.theme_editor.editing.box_x + 0.10, 0.99);
                    let (snapped_r, sx) = snap_val(new_right, &guide_xs, threshold_x);
                    if snapped_r >= state.theme_editor.editing.box_x + 0.10 {
                        new_right = snapped_r;
                        active_snap_x = sx;
                    }
                    let mut new_bottom = (state.theme_editor.editing.box_y
                        + state.theme_editor.editing.box_h
                        + norm_dy)
                        .clamp(state.theme_editor.editing.box_y + 0.10, 0.99);
                    let (snapped_b, sy) = snap_val(new_bottom, &guide_ys, threshold_y);
                    if snapped_b >= state.theme_editor.editing.box_y + 0.10 {
                        new_bottom = snapped_b;
                        active_snap_y = sy;
                    }
                    state.theme_editor.editing.box_w = new_right - state.theme_editor.editing.box_x;
                    state.theme_editor.editing.box_h =
                        new_bottom - state.theme_editor.editing.box_y;
                    push_snapshot = true;
                }
            }
        }
    } else {
        state.theme_editor.active_handle = None;
    }

    // Desenha linhas de alinhamento magnético quando ativas
    if let Some(gx) = active_snap_x {
        let px = canvas_rect.min.x + gx * canvas_rect.width();
        let is_center = (gx - 0.5).abs() < 0.001;
        let stroke_c = if is_center {
            Color32::from_rgb(239, 68, 68)
        } else {
            state.theme.accent.gamma_multiply(0.85)
        };
        ui.painter().line_segment(
            [
                egui::pos2(px, canvas_rect.min.y),
                egui::pos2(px, canvas_rect.max.y),
            ],
            Stroke::new(1.5, stroke_c),
        );
        if is_center {
            ui.painter().circle_filled(
                egui::pos2(px, canvas_rect.center().y),
                3.5,
                stroke_c,
            );
        }
    }

    if let Some(gy) = active_snap_y {
        let py = canvas_rect.min.y + gy * canvas_rect.height();
        let is_center = (gy - 0.5).abs() < 0.001;
        let stroke_c = if is_center {
            Color32::from_rgb(239, 68, 68)
        } else {
            state.theme.accent.gamma_multiply(0.85)
        };
        ui.painter().line_segment(
            [
                egui::pos2(canvas_rect.min.x, py),
                egui::pos2(canvas_rect.max.x, py),
            ],
            Stroke::new(1.5, stroke_c),
        );
        if is_center {
            ui.painter().circle_filled(
                egui::pos2(canvas_rect.center().x, py),
                3.5,
                stroke_c,
            );
        }
    }

    push_snapshot
}
