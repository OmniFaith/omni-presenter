//! Native egui typography exported as a transparent layer for browser outputs.
//! Layout, fonts, Bible references and markup go through the projection renderer,
//! not a second browser implementation. Expensive work runs off the UI thread.
use crate::{core::models::PresentationTheme, services::tasks::BackgroundTask};
use eframe::egui::{self, ColorImage, Rect, TextureId};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::{Arc, Mutex, OnceLock},
};

struct Entry {
    task: Option<BackgroundTask<Vec<u8>>>,
    image: Option<Arc<Vec<u8>>>,
}

pub fn request(
    theme: &PresentationTheme,
    text: &str,
    aspect: f32,
) -> (String, Option<Arc<Vec<u8>>>) {
    static CACHE: OnceLock<Mutex<HashMap<String, Entry>>> = OnceLock::new();
    let aspect = if aspect.is_finite() && aspect > 0.0 {
        aspect.clamp(0.25, 4.0)
    } else {
        16.0 / 9.0
    };
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    format!("{theme:?}").hash(&mut hash);
    text.hash(&mut hash);
    aspect.to_bits().hash(&mut hash);
    let key = format!("native-text-{:016x}.png", hash.finish());
    let mut cache = CACHE.get_or_init(Mutex::default).lock().unwrap();
    for entry in cache.values_mut() {
        if let Some(result) = entry.task.as_ref().and_then(BackgroundTask::take) {
            entry.task = None;
            if let Ok(bytes) = result {
                entry.image = Some(Arc::new(bytes));
            }
        }
    }
    if !cache.contains_key(&key) {
        if cache.len() >= 48 {
            cache.retain(|_, e| e.task.is_some());
        }
        // Bound pending work as well as completed layers.
        if cache.values().filter(|e| e.task.is_some()).count() >= 4 {
            return (key, None);
        }
        let theme = theme.clone();
        let text = text.to_owned();
        cache.insert(
            key.clone(),
            Entry {
                task: Some(BackgroundTask::spawn(move || render(&theme, &text, aspect))),
                image: None,
            },
        );
    }
    let entry = cache.get_mut(&key).unwrap();
    if let Some(result) = entry.task.as_ref().and_then(BackgroundTask::take) {
        entry.task = None;
        if let Ok(bytes) = result {
            entry.image = Some(Arc::new(bytes));
        }
    }
    (key, entry.image.clone())
}

fn render(theme: &PresentationTheme, text: &str, aspect: f32) -> Result<Vec<u8>, String> {
    let height = 720;
    let width = (height as f32 * aspect).round() as usize;
    let rect = Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width as f32, height as f32));
    let ctx = egui::Context::default();
    ctx.set_fonts(super::fonts::definitions());
    let mut textures = HashMap::<TextureId, ColorImage>::new();
    let mut shapes = Vec::new();
    // FontInsert is applied on the following frame. The warm-up also gathers
    // the full atlas before subsequent texture deltas become partial patches.
    for _ in 0..3 {
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(rect),
                ..Default::default()
            },
            |ui| {
                crate::render::render(ui.painter(), rect, theme, text);
            },
        );
        for (id, deltas) in &output.textures_delta.set {
            for delta in deltas {
                let egui::ImageData::Color(image) = &delta.image;
                if let Some([x, y]) = delta.pos {
                    if let Some(texture) = textures.get_mut(id) {
                        for row in 0..image.height() {
                            let offset = (y + row) * texture.width() + x;
                            texture.pixels[offset..offset + image.width()].copy_from_slice(
                                &image.pixels[row * image.width()..(row + 1) * image.width()],
                            );
                        }
                    }
                } else {
                    textures.insert(*id, image.as_ref().clone());
                }
            }
        }
        shapes = std::mem::take(&mut output.shapes);
        output.textures_delta.clear();
    }
    let mut pixels = vec![[0.0_f32; 4]; width * height];
    for primitive in ctx.tessellate(shapes, 1.0) {
        if let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive {
            let Some(texture) = textures.get(&mesh.texture_id) else {
                continue;
            };
            for indices in mesh.indices.as_chunks::<3>().0 {
                let vertices = [
                    mesh.vertices[indices[0] as usize],
                    mesh.vertices[indices[1] as usize],
                    mesh.vertices[indices[2] as usize],
                ];
                triangle(
                    &mut pixels,
                    width,
                    height,
                    primitive.clip_rect,
                    vertices,
                    texture,
                );
            }
        }
    }
    let mut rgba = Vec::with_capacity(pixels.len() * 4);
    for pixel in pixels {
        let alpha = pixel[3].clamp(0.0, 1.0);
        for channel in &pixel[..3] {
            rgba.push(if alpha > 0.0 {
                (channel / alpha * 255.0).round().clamp(0.0, 255.0) as u8
            } else {
                0
            });
        }
        rgba.push((alpha * 255.0).round() as u8);
    }
    let image = image::RgbaImage::from_raw(width as u32, height as u32, rgba)
        .ok_or("Invalid text image")?;
    let mut bytes = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut bytes, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(bytes.into_inner())
}

fn edge(a: egui::Pos2, b: egui::Pos2, p: egui::Pos2) -> f32 {
    (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)
}

fn triangle(
    pixels: &mut [[f32; 4]],
    width: usize,
    height: usize,
    clip: Rect,
    mut v: [egui::epaint::Vertex; 3],
    texture: &ColorImage,
) {
    let mut area = edge(v[0].pos, v[1].pos, v[2].pos);
    if area.abs() < 0.0001 {
        return;
    }
    if area < 0.0 {
        v.swap(1, 2);
        area = -area;
    }
    let min_x = v
        .iter()
        .map(|v| v.pos.x)
        .fold(f32::INFINITY, f32::min)
        .max(clip.min.x)
        .max(0.0)
        .floor() as usize;
    let min_y = v
        .iter()
        .map(|v| v.pos.y)
        .fold(f32::INFINITY, f32::min)
        .max(clip.min.y)
        .max(0.0)
        .floor() as usize;
    let max_x = v
        .iter()
        .map(|v| v.pos.x)
        .fold(f32::NEG_INFINITY, f32::max)
        .min(clip.max.x)
        .min(width as f32)
        .ceil()
        .max(0.0) as usize;
    let max_y = v
        .iter()
        .map(|v| v.pos.y)
        .fold(f32::NEG_INFINITY, f32::max)
        .min(clip.max.y)
        .min(height as f32)
        .ceil()
        .max(0.0) as usize;
    let inclusive = |a: egui::Pos2, b: egui::Pos2| a.y < b.y || (a.y == b.y && a.x > b.x);
    let colors = v.map(|v| v.color.to_array().map(|c| c as f32 / 255.0));
    for y in min_y..max_y {
        for x in min_x..max_x {
            let p = egui::pos2(x as f32 + 0.5, y as f32 + 0.5);
            if !clip.contains(p) {
                continue;
            }
            let edges = [
                edge(v[1].pos, v[2].pos, p),
                edge(v[2].pos, v[0].pos, p),
                edge(v[0].pos, v[1].pos, p),
            ];
            if edges.iter().enumerate().any(|(i, e)| {
                *e < 0.0 || (*e == 0.0 && !inclusive(v[(i + 1) % 3].pos, v[(i + 2) % 3].pos))
            }) {
                continue;
            }
            let weights = edges.map(|e| e / area);
            let uv = v[0].uv.to_vec2() * weights[0]
                + v[1].uv.to_vec2() * weights[1]
                + v[2].uv.to_vec2() * weights[2];
            let sample = sample(texture, uv.x, uv.y);
            let color: [f32; 4] = std::array::from_fn(|c| {
                (colors[0][c] * weights[0] + colors[1][c] * weights[1] + colors[2][c] * weights[2])
                    * sample[c]
            });
            let target = &mut pixels[y * width + x];
            for c in 0..4 {
                target[c] = color[c] + target[c] * (1.0 - color[3]);
            }
        }
    }
}

fn sample(texture: &ColorImage, u: f32, v: f32) -> [f32; 4] {
    let x = (u * texture.width() as f32 - 0.5).clamp(0.0, texture.width().saturating_sub(1) as f32);
    let y =
        (v * texture.height() as f32 - 0.5).clamp(0.0, texture.height().saturating_sub(1) as f32);
    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(texture.width() - 1);
    let y1 = (y0 + 1).min(texture.height() - 1);
    let c = |x: usize, y: usize| texture.pixels[y * texture.width() + x].to_array();
    let a = c(x0, y0);
    let b = c(x1, y0);
    let bottom = c(x0, y1);
    let d = c(x1, y1);
    std::array::from_fn(|i| {
        ((a[i] as f32 * (1.0 - x.fract()) + b[i] as f32 * x.fract()) * (1.0 - y.fract())
            + (bottom[i] as f32 * (1.0 - x.fract()) + d[i] as f32 * x.fract()) * y.fract())
            / 255.0
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_text_layer_preserves_reference_markup_and_transparency() {
        let theme = PresentationTheme {
            font_family: "Proportional".into(),
            shadow_enabled: false,
            ..Default::default()
        };
        let bytes = render(&theme, "//ref-pos: below\n//ref-align: right\n//valign: center\n**Verse** [4:3] text\nJohn 3.16", 16.0 / 9.0).unwrap();
        let image = image::load_from_memory(&bytes).unwrap().into_rgba8();
        assert_eq!(image.dimensions(), (1280, 720));
        assert!(image.pixels().any(|p| p[3] > 0));
        assert_eq!(image.get_pixel(0, 0)[3], 0);
    }
}
