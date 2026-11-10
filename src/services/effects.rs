use crate::core::models::PresentationTheme;
use crate::render::render_theme_lyrics;
use eframe::egui::{self, Rect, Vec2};

#[derive(Clone)]
struct Transition {
    previous: String,
    current: String,
    previous_theme: PresentationTheme,
    current_theme: PresentationTheme,
    started: f64,
}

pub fn render(
    painter: &egui::Painter,
    rect: Rect,
    theme: &PresentationTheme,
    text: &str,
    id: egui::Id,
    animate: bool,
) {
    let now = painter.ctx().input(|i| i.time);
    let mut transition = painter
        .ctx()
        .data_mut(|d| d.get_temp::<Transition>(id))
        .unwrap_or_else(|| Transition {
            previous: String::new(),
            current: text.into(),
            previous_theme: theme.clone(),
            current_theme: theme.clone(),
            started: now,
        });
    if transition.current != text || transition.current_theme.transition != theme.transition {
        transition.previous = transition.current;
        transition.previous_theme = transition.current_theme;
        transition.current = text.into();
        transition.current_theme = theme.clone();
        transition.started = now;
    } else {
        transition.current_theme = theme.clone();
    }
    if text.is_empty() {
        transition.previous.clear();
        painter.ctx().data_mut(|d| d.insert_temp(id, transition));
        return;
    }
    let duration = theme.options.transition_ms as f64 / 1000.0;
    let raw = if animate && theme.transition != "cut" && duration > 0.0 {
        ((now - transition.started) / duration).clamp(0.0, 1.0) as f32
    } else {
        1.0
    };
    let t = match theme.options.transition_easing.as_str() {
        "linear" => raw,
        "ease_out" => 1.0 - (1.0 - raw).powi(3),
        _ => raw * raw * (3.0 - 2.0 * raw),
    };
    if raw < 1.0 {
        painter.ctx().request_repaint();
        draw_layer(
            painter,
            rect,
            &transition.previous_theme,
            &transition.previous,
            &theme.transition,
            t,
            false,
            theme.options.transition_strength,
        );
        draw_layer(
            painter,
            rect,
            theme,
            text,
            &theme.transition,
            t,
            true,
            theme.options.transition_strength,
        );
    } else {
        render_theme_lyrics(painter, rect, theme, text);
    }
    painter.ctx().data_mut(|d| d.insert_temp(id, transition));
}

fn draw_layer(
    painter: &egui::Painter,
    rect: Rect,
    theme: &PresentationTheme,
    text: &str,
    effect: &str,
    t: f32,
    incoming: bool,
    strength: f32,
) {
    if text.is_empty() {
        return;
    }
    let progress = if incoming { t } else { 1.0 - t };
    let mut layer = painter.with_clip_rect(painter.clip_rect().intersect(rect));
    layer.multiply_opacity(progress);
    let mut target = rect;
    match effect {
        "slide_left" | "slide_right" | "slide_up" | "slide_down" | "slide" => {
            let axis = match effect {
                "slide_left" => Vec2::new(1.0, 0.0),
                "slide_right" => Vec2::new(-1.0, 0.0),
                "slide_up" => Vec2::new(0.0, 1.0),
                "slide_down" => Vec2::new(0.0, -1.0),
                _ => Vec2::new(1.0, 0.0),
            };
            let offset = if incoming { 1.0 - t } else { -t };
            target = rect.translate(axis * rect.size() * offset);
        }
        "zoom" => {
            let size = if incoming {
                0.84 + 0.16 * t
            } else {
                1.0 + 0.16 * t
            };
            target = Rect::from_center_size(rect.center(), rect.size() * size);
        }
        "zoom_out" => {
            let size = if incoming {
                1.16 - 0.16 * t
            } else {
                1.0 - 0.16 * t
            };
            target = Rect::from_center_size(rect.center(), rect.size() * size);
        }
        "wipe" => {
            let clip = if incoming {
                Rect::from_min_size(rect.min, egui::vec2(rect.width() * t, rect.height()))
            } else {
                Rect::from_min_max(
                    egui::pos2(rect.left() + rect.width() * t, rect.top()),
                    rect.max,
                )
            };
            layer.set_clip_rect(layer.clip_rect().intersect(clip));
        }
        "wipe_down" => {
            let clip = if incoming {
                Rect::from_min_size(rect.min, egui::vec2(rect.width(), rect.height() * t))
            } else {
                Rect::from_min_max(
                    egui::pos2(rect.left(), rect.top() + rect.height() * t),
                    rect.max,
                )
            };
            layer.set_clip_rect(layer.clip_rect().intersect(clip));
        }
        "split" => {
            let half_w = rect.width() * 0.5;
            let split_w = half_w * t;
            let clip = Rect::from_center_size(rect.center(), egui::vec2(split_w * 2.0, rect.height()));
            if incoming {
                layer.set_clip_rect(layer.clip_rect().intersect(clip));
            } else {
                layer.multiply_opacity((1.0 - t).max(0.0));
            }
        }
        "flip" => {
            let scale_x = if incoming {
                (t * std::f32::consts::PI * 0.5).sin().max(0.01)
            } else {
                ((1.0 - t) * std::f32::consts::PI * 0.5).sin().max(0.01)
            };
            let w = (rect.width() * scale_x).max(1.0);
            target = Rect::from_center_size(rect.center(), egui::vec2(w, rect.height()));
        }
        "cube" => {
            let offset_x = if incoming { (1.0 - t) * rect.width() } else { -t * rect.width() };
            let scale = if incoming { 0.85 + 0.15 * t } else { 1.0 - 0.15 * t };
            let sz = rect.size() * scale;
            let c = rect.center() + egui::vec2(offset_x, 0.0);
            target = Rect::from_center_size(c, sz);
        }
        "pop" => {
            let s = if incoming {
                let freq = 6.0;
                let decay = 3.5;
                1.0 - (-decay * t).exp() * (freq * t).cos() * 0.25
            } else {
                1.0 - t * 0.15
            };
            target = Rect::from_center_size(rect.center(), rect.size() * s);
        }
        "blur" => {
            let blur = (1.0 - progress) * strength * rect.height() / 720.0;
            let defocus_scale = 1.0 + (1.0 - progress) * 0.025;
            let target = Rect::from_center_size(rect.center(), rect.size() * defocus_scale);
            crate::render::render_theme_lyrics_blurred(&layer, target, theme, text, blur);
            return;
        }
        _ => {}
    }
    render_theme_lyrics(&layer, target, theme, text);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn slide_effects_start_finish_and_clear_without_stale_text() {
        let ctx = egui::Context::default();
        let id = egui::Id::unique("test_transition");
        for effect in [
            "blur",
            "fade",
            "slide_left",
            "slide_right",
            "slide_up",
            "slide_down",
            "slide",
            "zoom",
            "zoom_out",
            "wipe",
            "wipe_down",
            "split",
            "flip",
            "cube",
            "pop",
            "cut",
        ] {
            for (time, text) in [
                (0.0, "Primeira"),
                (1.0, "Segunda"),
                (1.2, "Segunda"),
                (2.0, "Segunda"),
                (3.0, ""),
            ] {
                let input = egui::RawInput {
                    time: Some(time),
                    ..Default::default()
                };
                let mut output = ctx.run_ui(input, |ui| {
                    let theme = PresentationTheme {
                        transition: effect.into(),
                        font_family: "Proportional".into(),
                        ..Default::default()
                    };
                    render(
                        ui.painter(),
                        Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(640.0, 360.0)),
                        &theme,
                        text,
                        id,
                        true,
                    );
                });
                output.textures_delta.clear();
            }
            assert!(
                ctx.data_mut(|d| d.get_temp::<Transition>(id))
                    .unwrap()
                    .previous
                    .is_empty()
            );
        }
    }
}
