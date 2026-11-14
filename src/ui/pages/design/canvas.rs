use super::DesignEditor;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Pos2, Rect, Vec2};

#[derive(Clone, Copy, Debug)]
pub(super) struct DragLayerInfo {
    pub index: usize,
    pub rect: [f32; 4],
}

pub(super) struct Drag {
    pub origin: Pos2,
    pub rect: [f32; 4],
    pub rotation: f32,
    pub handle: Option<super::geometry::Handle>,
    pub layers: Vec<DragLayerInfo>,
    pub marquee: Option<Rect>,
}

pub(super) fn guides(scene: &crate::core::design::DesignDocument, exclude: usize) -> (Vec<f32>, Vec<f32>) {
    let mut xs = vec![0.0, 0.5, 1.0];
    let mut ys = vec![0.0, 0.5, 1.0];
    for (index, layer) in scene.layers.iter().enumerate() {
        if index == exclude || !layer.visible {
            continue;
        }
        xs.extend([
            layer.rect[0],
            layer.rect[0] + layer.rect[2] * 0.5,
            layer.rect[0] + layer.rect[2],
        ]);
        ys.extend([
            layer.rect[1],
            layer.rect[1] + layer.rect[3] * 0.5,
            layer.rect[1] + layer.rect[3],
        ]);
    }
    (xs, ys)
}

fn layer_rect(canvas: Rect, rect: [f32; 4]) -> Rect {
    Rect::from_min_size(
        canvas.min + egui::vec2(rect[0] * canvas.width(), rect[1] * canvas.height()),
        egui::vec2(rect[2] * canvas.width(), rect[3] * canvas.height()),
    )
}

pub(super) fn render(
    ui: &mut egui::Ui,
    editor: &mut DesignEditor,
    theme: &AppTheme,
    state: &crate::core::state::ControlPanelState,
    deck: &crate::core::slides::SlideDeck,
    presenting: bool,
) {
    let Some(scene) = editor.scene.as_ref() else {
        ui.add_space(60.0);
        ui.vertical_centered(|ui| {
            ui.heading(crate::ui::i18n::text(ui, "design-your-next-visual-starts-here"));
            ui.add_space(10.0);
            ui.label(crate::ui::i18n::text(ui, "design-choose-new-design-and-start-from-scratch-or-use-a"));
            ui.label(crate::ui::i18n::text(ui, "design-combine-text-images-and-automatically-updated-information"));
        });
        return;
    };
    ui.add_space(26.0);
    let available = ui.available_size();
    let aspect = editor
        .preview_aspect_ratio
        .unwrap_or(state.presentation_aspect_ratio)
        .clamp(0.4, 4.0);
    let reserve = if available.y > 340.0 {
        if editor.show_filmstrip {
            match editor.timeline_mode {
                super::TimelineMode::Layers => {
                    (80.0 + editor.timeline_height).min(available.y - 120.0)
                }
                super::TimelineMode::Slides => 155.0,
            }
        } else {
            75.0
        }
    } else if available.y > 240.0 {
        if editor.show_filmstrip {
            (60.0 + editor.timeline_height * 0.5).min(available.y - 100.0)
        } else {
            60.0
        }
    } else {
        10.0
    };
    let max_h = (available.y - reserve).max(90.0);
    let w = available.x.min(max_h * aspect).max(90.0);
    let size = Vec2::new(w, w / aspect);
    let (slot, _) = ui.allocate_exact_size(egui::vec2(available.x, size.y), egui::Sense::hover());
    let canvas = Rect::from_center_size(slot.center(), size);
    let response = ui.interact(
        canvas.expand(28.0).intersect(ui.clip_rect()),
        ui.scope_id().with("design_canvas"),
        egui::Sense::click_and_drag(),
    );
    // Checkerboard makes transparency visible without changing the exported scene.
    let checker_base = if theme.bg_app.r() > 128 {
        egui::Color32::from_rgb(25, 29, 38)
    } else {
        theme.bg_app
    };
    ui.painter().rect_filled(canvas, 0.0, checker_base);
    let painter = ui.painter().with_clip_rect(canvas);
    for y in (0..size.y as usize).step_by(16) {
        for x in (0..size.x as usize).step_by(16) {
            if (x / 16 + y / 16) % 2 == 0 {
                painter.rect_filled(
                    Rect::from_min_size(
                        canvas.min + egui::vec2(x as f32, y as f32),
                        Vec2::splat(16.0),
                    ),
                    0.0,
                    checker_base.lerp_to_gamma(egui::Color32::from_rgb(50, 56, 70), 0.35),
                );
            }
        }
    }

    let total_slide_ms = scene.active_slide_duration_ms();
    let current_ms = if editor.animation_playing {
        if let Some(start) = editor.animation_started_at {
            let elapsed = start.elapsed().as_millis() as u64;
            if elapsed >= total_slide_ms {
                if editor.animation_loop {
                    editor.animation_started_at = Some(std::time::Instant::now());
                    0
                } else {
                    editor.animation_playing = false;
                    editor.animation_started_at = None;
                    editor.animation_time_ms = total_slide_ms;
                    total_slide_ms
                }
            } else {
                ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));
                editor.animation_time_ms = elapsed;
                elapsed
            }
        } else {
            0
        }
    } else {
        editor.animation_time_ms
    };

    let mut render_scene = scene.clone();
    if current_ms > 0 || editor.animation_playing {
        render_scene.layers = scene.animated_layers_at(current_ms);
    }
    if let Some(preview_theme_id) = &editor.preview_theme_id {
        if let Some(preview_theme) = state
            .themes
            .iter()
            .find(|t| &t.id == preview_theme_id)
            .or_else(|| Some(state.preview_theme()))
        {
            crate::render::paint_theme_bg(&painter, canvas, preview_theme, 0.0);
            render_scene.theme_id = Some(preview_theme.id.clone());
            if matches!(preview_theme.bg_type.as_str(), "shader" | "video") {
                ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));
            }
        }
    } else if render_scene.theme_id.is_none() {
        render_scene.theme_id = Some(state.preview_theme().id.clone());
    }
    if scene.overlay && presenting {
        if let Some(base) = &deck.live_design {
            crate::render::design::paint_themed(&painter, canvas, base, &state.themes);
        } else {
            let live_theme = crate::services::media::live_theme(state, deck);
            crate::render::paint_theme_bg(&painter, canvas, &live_theme, 0.0);
            crate::render::render_theme_lyrics(
                &painter,
                canvas,
                &live_theme,
                deck.live_slide_text(),
            );
        }
    }
    crate::render::design::paint_themed(&painter, canvas, &render_scene, &state.themes);
    let hovered = if ui.rect_contains_pointer(canvas) {
        ui.input(|input| input.pointer.hover_pos())
            .and_then(|pointer| {
                scene
                    .layers
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, layer)| {
                        layer.visible && super::geometry::contains(layer_rect(canvas, layer.rect),layer.rotation,pointer)
                    })
                    .map(|(index, _)| index)
            })
    } else {
        None
    };
    let emphasis = if editor.drag.is_some() {
        editor.selected
    } else {
        editor.hovered_layer.or(hovered)
    };
    for (index, layer) in scene
        .layers
        .iter()
        .enumerate()
        .filter(|(_, layer)| layer.visible)
    {
        let id = response.id.with(("layer_feedback", &layer.id));
        let hover = ui.ctx().animate_bool_with_time(
            id.with("hover"),
            emphasis == Some(index),
            ui.style().animation_time,
        );
        let hold = ui.ctx().animate_bool_with_time(
            id.with("hold"),
            emphasis == Some(index) && response.is_pointer_button_down_on(),
            ui.style().animation_time,
        );
        if hover > 0.0 || hold > 0.0 {
            let rect = layer_rect(canvas, layer.rect);
            painter.rect_filled(
                rect,
                0.0,
                theme.accent.gamma_multiply(hover * 0.04 + hold * 0.08),
            );
            painter.rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(
                    1.0 + hold,
                    theme.accent.gamma_multiply(hover * 0.7 + hold * 0.3),
                ),
                egui::StrokeKind::Inside,
            );
        }
    }
    if let Some(index) = hovered {
        ui.ctx().set_cursor_icon(if scene.layers[index].locked {
            egui::CursorIcon::NotAllowed
        } else if response.dragged() {
            egui::CursorIcon::Grabbing
        } else {
            egui::CursorIcon::Grab
        });
    }
    let menu_id = response.id.with(("context_layer", &scene.id));
    if response.secondary_clicked() {
        ui.ctx().data_mut(|data| data.insert_temp(menu_id, hovered));
        if let Some(index) = hovered {
            if !editor.is_layer_selected(index) {
                editor.select_layer(index, false);
            }
            editor.inspector_tab = 1;
        }
    }
    painter.rect_stroke(
        canvas,
        0.0,
        egui::Stroke::new(1.0, theme.border_default),
        egui::StrokeKind::Inside,
    );
    super::canvas_interaction::render(ui,editor,canvas,&response,theme);
    let menu_layer = ui
        .ctx()
        .data_mut(|data| data.get_temp::<Option<usize>>(menu_id))
        .flatten();
    let mut menu_action = None;
    response.context_menu(|ui| {
        if let Some(index) = menu_layer {
            if let Some(scene) = &editor.scene
                && let Some(layer) = scene.layers.get(index) {
                    menu_action =
                        super::menus::layer_menu(ui, layer, index, scene.layers.len(), editor.selected_layers.len(), theme)
                            .map(|action| (index, action));
                }
        } else {
            crate::ui::components::context_menu_header(ui);
            ui.menu_button(
                crate::ui::i18n::format(ui, "design-add-layer-label", &[("icon", egui_phosphor::regular::PLUS)]),
                |ui| super::sidebar::add_menu(ui, editor),
            );
        }
    });
    if let Some((index, action)) = menu_action {
        super::menus::apply_layer(editor, index, action);
    }
    ui.add_space(8.0);
    super::animation_bar::render(ui, editor, theme, &state.themes);
}
