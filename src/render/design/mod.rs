//! Native scenes use cached layer textures and GPU blending. The CPU raster is
//! retained for optional PNG exports and contexts without a GPU renderer.
mod gpu;
mod raster;
mod sprites;
mod text;
mod vector;
pub use gpu::install;

use crate::core::design::{DesignDocument, now_ms};
use eframe::egui;
use image::RgbaImage;
use std::{
    collections::VecDeque,
    hash::{Hash, Hasher},
    sync::{Arc, Mutex, OnceLock},
};

pub struct DesignFrame {
    pub key: u64,
    pub image: RgbaImage,
    png: OnceLock<Vec<u8>>,
}
impl DesignFrame {
    pub fn png(&self) -> &[u8] {
        self.png.get_or_init(|| {
            let mut bytes = std::io::Cursor::new(Vec::new());
            image::write_buffer_with_format(
                &mut bytes,
                self.image.as_raw(),
                self.image.width(),
                self.image.height(),
                image::ColorType::Rgba8,
                image::ImageFormat::Png,
            )
            .expect("valid design raster");
            bytes.into_inner()
        })
    }
}

pub fn frame(scene: &DesignDocument, width: u32) -> Arc<DesignFrame> {
    let now = now_ms();
    let is_dynamic = scene.is_dynamic();
    let resolved = crate::core::plugins::runtime::materialize(scene, now);
    let scene = resolved.as_ref();
    let width = width.clamp(160, 1920);
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    serde_json::to_vec(scene)
        .unwrap_or_default()
        .hash(&mut hash);
    width.hash(&mut hash);
    if is_dynamic {
        (now / 16).hash(&mut hash);
    }
    let key = hash.finish();
    static CACHE: OnceLock<Mutex<VecDeque<Arc<DesignFrame>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Mutex::default);
    if let Some(found) = cache
        .lock()
        .unwrap()
        .iter()
        .find(|entry| entry.key == key)
        .cloned()
    {
        return found;
    }
    let image = raster::render(scene, width, now);
    let result = Arc::new(DesignFrame {
        key,
        image,
        png: OnceLock::new(),
    });
    let mut cache = cache.lock().unwrap();
    if cache.len() >= 6 {
        cache.pop_front();
    }
    cache.push_back(result.clone());
    result
}

pub fn paint(painter: &egui::Painter, rect: egui::Rect, scene: &DesignDocument) {
    let is_dynamic = scene.is_dynamic();
    if is_dynamic {
        painter
            .ctx()
            .request_repaint_after(std::time::Duration::from_millis(16));
    }
    let resolved = crate::core::plugins::runtime::materialize(scene, now_ms());
    if gpu::paint(painter, rect, resolved.as_ref(), None) {
        if is_dynamic {
            painter
                .ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));
        }
        return;
    }
    let mut adapted = scene.clone();
    let ar = rect.aspect_ratio().clamp(0.25, 4.0);
    let (tex_w, tex_h) = if ar >= 1.0 {
        let w = 1280.0f32;
        (w as u32, (w / ar).round().clamp(64.0, 1920.0) as u32)
    } else {
        let h = 1280.0f32;
        ((h * ar).round().clamp(64.0, 1920.0) as u32, h as u32)
    };
    adapted.width = tex_w;
    adapted.height = tex_h;
    let rendered = frame(&adapted, tex_w);
    let id = egui::Id::NULL.with(("design_texture", &scene.id));
    let cached = painter
        .ctx()
        .data(|data| data.get_temp::<(u64, egui::TextureHandle)>(id));
    let texture = if let Some((key, texture)) = cached.filter(|(key, _)| *key == rendered.key) {
        let _ = key;
        texture
    } else {
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [
                rendered.image.width() as usize,
                rendered.image.height() as usize,
            ],
            rendered.image.as_raw(),
        );
        let texture = painter.ctx().load_texture(
            format!("design-{}", scene.id),
            image,
            egui::TextureOptions::LINEAR,
        );
        painter
            .ctx()
            .data_mut(|data| data.insert_temp(id, (rendered.key, texture.clone())));
        texture
    };
    painter.image(
        texture.id(),
        rect,
        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE.gamma_multiply(painter.opacity()),
    );
    if is_dynamic {
        painter
            .ctx()
            .request_repaint_after(std::time::Duration::from_millis(16));
    }
}

pub fn paint_themed(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene: &DesignDocument,
    themes: &[crate::core::models::PresentationTheme],
) {
    let is_dynamic = scene.is_dynamic();
    if is_dynamic {
        painter
            .ctx()
            .request_repaint_after(std::time::Duration::from_millis(16));
    }
    let theme = themes
        .iter()
        .find(|theme| Some(&theme.id) == scene.theme_id.as_ref())
        .or_else(|| themes.first());
    let inherited = scene.resolve_theme_fonts(theme);
    let scene = inherited.as_ref();
    if scene.background_mode == "theme"
        && let Some(theme) = theme.filter(|theme| matches!(theme.bg_type.as_str(), "shader" | "video"))
    {
        crate::render::paint_theme_bg(painter, rect, theme, 0.0);
        paint(painter, rect, scene);
        return;
    }
    let resolved = crate::core::plugins::runtime::materialize(scene, now_ms());
    if scene.background_mode == "theme"
        && gpu::paint(painter, rect, resolved.as_ref(), theme)
    {
        if is_dynamic {
            painter
                .ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));
        }
        return;
    }
    if scene.background_mode == "theme"
        && let Some(theme) = theme
    {
        crate::render::paint_theme_bg(painter, rect, theme, 0.0);
    }
    paint(painter, rect, scene);
}

#[derive(Clone)]
pub struct DesignTransition {
    pub previous: Option<DesignDocument>,
    pub current: Option<DesignDocument>,
    pub started: f64,
}

pub fn paint_themed_with_transition(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene: &DesignDocument,
    themes: &[crate::core::models::PresentationTheme],
    id: egui::Id,
    animate: bool,
) {
    let now = painter.ctx().input(|i| i.time);
    let mut transition = painter
        .ctx()
        .data_mut(|d| d.get_temp::<DesignTransition>(id))
        .unwrap_or_else(|| DesignTransition {
            previous: None,
            current: Some(scene.clone()),
            started: now,
        });

    let is_new = transition.current.as_ref().map(|s| &s.id) != Some(&scene.id);
    if is_new {
        transition.previous = transition.current.take();
        transition.current = Some(scene.clone());
        transition.started = now;
    } else {
        transition.current = Some(scene.clone());
    }

    let duration = scene.transition_ms as f64 / 1000.0;
    let raw = if animate && scene.transition != "cut" && duration > 0.0 && transition.previous.is_some() {
        ((now - transition.started) / duration).clamp(0.0, 1.0) as f32
    } else {
        1.0
    };

    let t = match scene.transition_easing.as_str() {
        "linear" => raw,
        "ease_in" => raw * raw,
        "ease_out" => 1.0 - (1.0 - raw).powi(3),
        "ease_in_out" => raw * raw * (3.0 - 2.0 * raw),
        _ => raw * raw * (3.0 - 2.0 * raw),
    };

    if raw < 1.0 && let Some(prev) = &transition.previous {
        painter.ctx().request_repaint();
        let effect = scene.transition.as_str();
        let strength = scene.transition_strength;
        draw_design_layer(
            painter,
            rect,
            prev,
            themes,
            effect,
            t,
            false,
            strength,
        );
        draw_design_layer(
            painter,
            rect,
            scene,
            themes,
            effect,
            t,
            true,
            strength,
        );
    } else {
        if raw >= 1.0 {
            transition.previous = None;
        }
        paint_themed(painter, rect, scene, themes);
    }

    painter.ctx().data_mut(|d| d.insert_temp(id, transition));
}

pub fn paint_with_transition(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene: &DesignDocument,
    id: egui::Id,
    animate: bool,
) {
    paint_themed_with_transition(painter, rect, scene, &[], id, animate);
}

fn draw_design_layer(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene: &DesignDocument,
    themes: &[crate::core::models::PresentationTheme],
    effect: &str,
    t: f32,
    incoming: bool,
    strength: f32,
) {
    let progress = if incoming { t } else { 1.0 - t };
    let mut target = rect;
    let mut layer = painter.with_clip_rect(painter.clip_rect().intersect(rect));
    layer.multiply_opacity(progress);

    let s = (strength / 12.0).clamp(0.1, 5.0);

    match effect {
        "slide_left" | "slide" => {
            let offset = if incoming { (1.0 - t) * s } else { -t * s };
            target = rect.translate(egui::vec2(offset * rect.width(), 0.0));
        }
        "slide_right" => {
            let offset = if incoming { -(1.0 - t) * s } else { t * s };
            target = rect.translate(egui::vec2(offset * rect.width(), 0.0));
        }
        "slide_up" => {
            let offset = if incoming { (1.0 - t) * s } else { -t * s };
            target = rect.translate(egui::vec2(0.0, offset * rect.height()));
        }
        "slide_down" => {
            let offset = if incoming { -(1.0 - t) * s } else { t * s };
            target = rect.translate(egui::vec2(0.0, offset * rect.height()));
        }
        "zoom" => {
            let size = if incoming {
                (1.0 - 0.16 * s) + (0.16 * s) * t
            } else {
                1.0 + (0.16 * s) * t
            };
            target = egui::Rect::from_center_size(rect.center(), rect.size() * size);
        }
        "zoom_out" => {
            let size = if incoming {
                (1.0 + 0.16 * s) - (0.16 * s) * t
            } else {
                1.0 - (0.16 * s) * t
            };
            target = egui::Rect::from_center_size(rect.center(), rect.size() * size);
        }
        "wipe" => {
            let clip = if incoming {
                egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * t, rect.height()))
            } else {
                egui::Rect::from_min_max(
                    egui::pos2(rect.left() + rect.width() * t, rect.top()),
                    rect.max,
                )
            };
            layer.set_clip_rect(layer.clip_rect().intersect(clip));
        }
        _ => {}
    }

    if themes.is_empty() {
        paint(&layer, target, scene);
    } else {
        paint_themed(&layer, target, scene, themes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::design::DesignDocument;
    use eframe::egui::{self, Rect, Vec2};

    #[test]
    fn design_document_serialization_preserves_transition() {
        let mut doc = DesignDocument::default();
        doc.transition = "slide_left".into();
        doc.transition_ms = 450;
        doc.transition_strength = 20.0;
        doc.transition_easing = "ease_out".into();

        let json = serde_json::to_string(&doc).expect("serialize");
        let restored: DesignDocument = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.transition, "slide_left");
        assert_eq!(restored.transition_ms, 450);
        assert_eq!(restored.transition_strength, 20.0);
        assert_eq!(restored.transition_easing, "ease_out");
    }

    #[test]
    fn dynamic_slides_transition_smoothly_between_each_other() {
        let ctx = egui::Context::default();
        let trans_id = egui::Id::unique("test_design_transition");
        let rect = Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(1280.0, 720.0));

        let mut scoreboard = DesignDocument::default();
        scoreboard.id = "scoreboard-slide".into();
        scoreboard.name = "Placar interativo".into();
        scoreboard.transition = "fade".into();
        scoreboard.transition_ms = 400;

        let mut announcements = DesignDocument::default();
        announcements.id = "announcements-slide".into();
        announcements.name = "Avisos rotativos".into();
        announcements.transition = "slide_left".into();
        announcements.transition_ms = 400;

        // Frame 1: Initial presentation of Scoreboard
        let mut input = egui::RawInput::default();
        input.time = Some(10.0);
        let mut out = ctx.run_ui(input, |ui| {
            paint_themed_with_transition(
                ui.painter(),
                rect,
                &scoreboard,
                &[],
                trans_id,
                true,
            );
        });
        out.textures_delta.clear();

        let state = ctx.data(|d| d.get_temp::<DesignTransition>(trans_id)).expect("state exists");
        assert_eq!(state.current.as_ref().map(|s| s.id.as_str()), Some("scoreboard-slide"));
        assert!(state.previous.is_none());

        // Frame 2: Switch to Announcements at t = 10.1s (100ms in, transition in progress)
        let mut input = egui::RawInput::default();
        input.time = Some(10.1);
        let mut out = ctx.run_ui(input, |ui| {
            paint_themed_with_transition(
                ui.painter(),
                rect,
                &announcements,
                &[],
                trans_id,
                true,
            );
        });
        out.textures_delta.clear();

        let state = ctx.data(|d| d.get_temp::<DesignTransition>(trans_id)).expect("state exists");
        assert_eq!(state.previous.as_ref().map(|s| s.id.as_str()), Some("scoreboard-slide"));
        assert_eq!(state.current.as_ref().map(|s| s.id.as_str()), Some("announcements-slide"));
        assert_eq!(state.started, 10.1);

        // Frame 3: During transition at t = 10.3s (200ms in, midway through 400ms transition)
        let mut input = egui::RawInput::default();
        input.time = Some(10.3);
        let mut out = ctx.run_ui(input, |ui| {
            paint_themed_with_transition(
                ui.painter(),
                rect,
                &announcements,
                &[],
                trans_id,
                true,
            );
        });
        out.textures_delta.clear();

        let state = ctx.data(|d| d.get_temp::<DesignTransition>(trans_id)).expect("state exists");
        assert!(state.previous.is_some(), "previous slide retained during transition");

        // Frame 4: After transition completes at t = 10.6s (500ms in, past 400ms duration)
        let mut input = egui::RawInput::default();
        input.time = Some(10.6);
        let mut out = ctx.run_ui(input, |ui| {
            paint_themed_with_transition(
                ui.painter(),
                rect,
                &announcements,
                &[],
                trans_id,
                true,
            );
        });
        out.textures_delta.clear();

        let state = ctx.data(|d| d.get_temp::<DesignTransition>(trans_id)).expect("state exists");
        assert!(state.previous.is_none(), "previous slide cleared after transition finishes");
        assert_eq!(state.current.as_ref().map(|s| s.id.as_str()), Some("announcements-slide"));
    }
}
