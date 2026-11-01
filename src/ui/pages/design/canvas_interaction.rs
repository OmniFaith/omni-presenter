use super::{DesignEditor, geometry::{self, Handle}};
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Rect};

pub(super) fn render(ui: &mut egui::Ui, editor: &mut DesignEditor, canvas: Rect, response: &egui::Response, theme: &AppTheme) {
    if editor.scene.is_none() { return; }

    let (selected_layer, hot) = if let Some(scene) = editor.scene.as_ref() {
        let sel = editor.selected.and_then(|i| scene.layers.get(i).cloned());
        let pointer = ui.input(|i| i.pointer.hover_pos());
        let hot = sel.as_ref().filter(|l| !l.locked).and_then(|layer| {
            pointer.and_then(|p| geometry::hit_handle(geometry::rect(canvas, layer.rect), layer.rotation, p))
        });
        (sel, hot)
    } else {
        return;
    };

    if let Some(handle) = hot {
        ui.ctx().set_cursor_icon(handle.cursor());
    }

    // Draw bounding boxes around all selected layers
    if let Some(scene) = editor.scene.as_ref() {
        for &idx in &editor.selected_layers {
            if let Some(layer) = scene.layers.get(idx).filter(|l| l.visible) {
                let bounds = geometry::rect(canvas, layer.rect);
                let points = geometry::corners(bounds, layer.rotation);
                for i in 0..4 {
                    ui.painter().line_segment([points[i], points[(i + 1) % 4]], egui::Stroke::new(1.5, theme.accent));
                }
            }
        }
    }

    // Draw handles for the primary active layer
    if let Some(layer) = selected_layer.as_ref().filter(|l| !l.locked) {
        let bounds = geometry::rect(canvas, layer.rect);
        let top = geometry::handle_position(bounds, layer.rotation, Handle::North);
        let rotate = geometry::handle_position(bounds, layer.rotation, Handle::Rotate);
        ui.painter().line_segment([top, rotate], egui::Stroke::new(1., theme.accent));
        for handle in Handle::RESIZE {
            let center = geometry::handle_position(bounds, layer.rotation, handle);
            ui.painter().rect(
                Rect::from_center_size(center, egui::vec2(8., 8.)),
                2.,
                theme.bg_panel,
                egui::Stroke::new(1.5, theme.accent),
                egui::StrokeKind::Inside,
            );
        }
        ui.painter().circle(rotate, 5., theme.bg_panel, egui::Stroke::new(1.5, theme.accent));
    }

    if response.clicked() || response.drag_started() || (response.is_pointer_button_down_on() && ui.input(|i| i.pointer.any_pressed())) {
        let origin = ui.input(|i| i.pointer.press_origin()).or_else(|| response.interact_pointer_pos());
        if let Some(origin) = origin {
            let handle = selected_layer.as_ref().filter(|layer| !layer.locked).and_then(|layer| {
                geometry::hit_handle(geometry::rect(canvas, layer.rect), layer.rotation, origin)
            });
            let modifiers = ui.input(|i| i.modifiers);
            let additive = modifiers.shift || modifiers.ctrl || modifiers.command;

            if handle.is_none() {
                let hit = editor.scene.as_ref().and_then(|scene| {
                    scene.layers.iter().enumerate().rev().find(|(_, layer)| {
                        layer.visible && geometry::contains(geometry::rect(canvas, layer.rect), layer.rotation, origin)
                    }).map(|(i, _)| i)
                });

                if let Some(hit_idx) = hit {
                    if additive {
                        editor.select_layer(hit_idx, true);
                    } else if !editor.is_layer_selected(hit_idx) || response.clicked() {
                        editor.select_layer(hit_idx, false);
                    }
                    editor.inspector_tab = 1;
                } else if response.clicked() && !additive {
                    editor.clear_layer_selection();
                }
            }

            if response.drag_started() {
                let target_layer = editor.selected.and_then(|i| editor.scene.as_ref()?.layers.get(i).filter(|l| !l.locked).cloned());
                if let Some(layer) = target_layer {
                    let mode = handle.or_else(|| ui.input(|i| (i.modifiers.ctrl || i.modifiers.command).then_some(Handle::Rotate)));
                    let drag_layers: Vec<super::canvas::DragLayerInfo> = editor
                        .selected_layers
                        .iter()
                        .filter_map(|&i| editor.scene.as_ref()?.layers.get(i).filter(|l| !l.locked).map(|l| super::canvas::DragLayerInfo { index: i, rect: l.rect }))
                        .collect();

                    editor.drag = Some(super::canvas::Drag {
                        origin,
                        rect: layer.rect,
                        rotation: layer.rotation,
                        handle: mode,
                        layers: drag_layers,
                        marquee: None,
                    });
                    if let Some(scene) = editor.scene.clone() {
                        editor.undo.push(scene);
                        if editor.undo.len() > 60 {
                            editor.undo.remove(0);
                        }
                        editor.redo.clear();
                    }
                } else if handle.is_none() {
                    // Start rubberband / marquee selection
                    if !additive {
                        editor.clear_layer_selection();
                    }
                    editor.drag = Some(super::canvas::Drag {
                        origin,
                        rect: [0.0; 4],
                        rotation: 0.0,
                        handle: None,
                        layers: Vec::new(),
                        marquee: Some(Rect::from_two_pos(origin, origin)),
                    });
                }
            }
        }
    }

    let mut active_x = None;
    let mut active_y = None;

    if response.dragged() && let Some(pointer) = response.interact_pointer_pos() && let Some(drag) = &mut editor.drag {
        if let Some(marquee) = &mut drag.marquee {
            let marquee_rect = Rect::from_two_pos(drag.origin, pointer);
            *marquee = marquee_rect;
            if let Some(scene) = editor.scene.as_ref() {
                let mut hit_indices = std::collections::BTreeSet::new();
                for (index, layer) in scene.layers.iter().enumerate() {
                    if !layer.visible {
                        continue;
                    }
                    let lr = geometry::rect(canvas, layer.rect);
                    if marquee_rect.intersects(lr) {
                        hit_indices.insert(index);
                        if let Some(gid) = layer.group_id.as_deref() {
                            for (gi, gl) in scene.layers.iter().enumerate() {
                                if gl.group_id.as_deref() == Some(gid) {
                                    hit_indices.insert(gi);
                                }
                            }
                        }
                    }
                }
                editor.selected_layers = hit_indices;
                editor.selected = editor.selected_layers.iter().next_back().copied();
            }
        } else if let Some(index) = editor.selected {
            let (xs, ys) = super::canvas::guides(editor.scene.as_ref().unwrap(), index);
            let modifiers = ui.input(|i| i.modifiers);
            let snap = !editor.disable_snap && !modifiers.alt;
            let mut delta = pointer - drag.origin;
            let layer = &mut editor.scene.as_mut().unwrap().layers[index];

            match drag.handle {
                Some(Handle::Rotate) => {
                    let center = geometry::rect(canvas, drag.rect).center();
                    let angle = (pointer - center).angle() - (drag.origin - center).angle();
                    let mut rotation = drag.rotation + angle.to_degrees();
                    if modifiers.shift {
                        rotation = (rotation / 15.).round() * 15.;
                    }
                    layer.rotation = (rotation + 180.).rem_euclid(360.) - 180.;
                }
                Some(handle) => {
                    layer.rect = geometry::resized(canvas, drag.rect, drag.rotation, handle, delta, modifiers.shift);
                    if snap && !modifiers.shift && drag.rotation.abs() < 0.001 {
                        let axes = handle.axes();
                        if axes.x != 0. {
                            let right = layer.rect[0] + layer.rect[2];
                            let edge = if axes.x < 0. { layer.rect[0] } else { right };
                            let (value, guide) = geometry::snap(edge, 0., &xs, 6. / canvas.width());
                            active_x = guide;
                            if axes.x < 0. {
                                layer.rect[0] = value;
                                layer.rect[2] = (right - value).max(8. / canvas.width());
                            } else {
                                layer.rect[2] = (value - layer.rect[0]).max(8. / canvas.width());
                            }
                        }
                        if axes.y != 0. {
                            let bottom = layer.rect[1] + layer.rect[3];
                            let edge = if axes.y < 0. { layer.rect[1] } else { bottom };
                            let (value, guide) = geometry::snap(edge, 0., &ys, 6. / canvas.height());
                            active_y = guide;
                            if axes.y < 0. {
                                layer.rect[1] = value;
                                layer.rect[3] = (bottom - value).max(8. / canvas.height());
                            } else {
                                layer.rect[3] = (value - layer.rect[1]).max(8. / canvas.height());
                            }
                        }
                    }
                }
                None => {
                    if modifiers.shift {
                        if delta.x.abs() > delta.y.abs() {
                            delta.y = 0.;
                        } else {
                            delta.x = 0.;
                        }
                    }
                    let mut x = drag.rect[0] + delta.x / canvas.width();
                    let mut y = drag.rect[1] + delta.y / canvas.height();
                    if snap {
                        (x, active_x) = geometry::snap(x, drag.rect[2], &xs, 6. / canvas.width());
                        (y, active_y) = geometry::snap(y, drag.rect[3], &ys, 6. / canvas.height());
                    }
                    let dx = x - drag.rect[0];
                    let dy = y - drag.rect[1];
                    if let Some(scene) = editor.scene.as_mut() {
                        for layer_info in &drag.layers {
                            if let Some(l) = scene.layers.get_mut(layer_info.index) {
                                l.rect[0] = (layer_info.rect[0] + dx).clamp(0., (1. - l.rect[2]).max(0.));
                                l.rect[1] = (layer_info.rect[1] + dy).clamp(0., (1. - l.rect[3]).max(0.));
                            }
                        }
                    }
                }
            }
            editor.saved = false;
            editor.pending_since = Some(std::time::Instant::now());
        }
    }

    let painter = ui.painter().with_clip_rect(canvas);

    // Draw active marquee rectangle
    if let Some(drag) = &editor.drag && let Some(marquee) = drag.marquee {
        painter.rect_filled(marquee, 2.0, theme.accent.gamma_multiply(0.12));
        painter.rect_stroke(marquee, 2.0, egui::Stroke::new(1.0, theme.accent), egui::StrokeKind::Inside);
    }

    if let Some(x) = active_x {
        let x = canvas.left() + x * canvas.width();
        painter.line_segment([egui::pos2(x, canvas.top()), egui::pos2(x, canvas.bottom())], egui::Stroke::new(1., theme.accent));
    }
    if let Some(y) = active_y {
        let y = canvas.top() + y * canvas.height();
        painter.line_segment([egui::pos2(canvas.left(), y), egui::pos2(canvas.right(), y)], egui::Stroke::new(1., theme.accent));
    }
    if response.drag_stopped() {
        editor.drag = None;
    }
}

