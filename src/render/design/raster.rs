use crate::core::design::{BlendMode, DesignDocument, LayerContent};
use image::{Rgba, RgbaImage};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

pub(super) fn composite(dst: &mut Rgba<u8>, src: Rgba<u8>, opacity: f32, mode: BlendMode) {
    let sa = src[3] as f32 / 255.0 * opacity.clamp(0.0, 1.0);
    if sa <= 0.0 {
        return;
    }
    let da = dst[3] as f32 / 255.0;
    let alpha = sa + da * (1.0 - sa);
    for c in 0..3 {
        let s = src[c] as f32 / 255.0;
        let d = dst[c] as f32 / 255.0;
        let mixed = match mode {
            BlendMode::Normal => s,
            BlendMode::Multiply => s * d,
            BlendMode::Screen => s + d - s * d,
            BlendMode::Overlay => {
                if d <= 0.5 {
                    2.0 * s * d
                } else {
                    1.0 - 2.0 * (1.0 - s) * (1.0 - d)
                }
            }
            BlendMode::Darken => s.min(d),
            BlendMode::Lighten => s.max(d),
            BlendMode::ColorDodge => {
                if s >= 1.0 { 1.0 } else { (d / (1.0 - s)).min(1.0) }
            }
            BlendMode::ColorBurn => {
                if s <= 0.0 { 0.0 } else { 1.0 - ((1.0 - d) / s).min(1.0) }
            }
            BlendMode::HardLight => {
                if s <= 0.5 {
                    2.0 * s * d
                } else {
                    1.0 - 2.0 * (1.0 - s) * (1.0 - d)
                }
            }
            BlendMode::SoftLight => {
                if s <= 0.5 {
                    d - (1.0 - 2.0 * s) * d * (1.0 - d)
                } else {
                    let curve = if d <= 0.25 {
                        ((16.0 * d - 12.0) * d + 4.0) * d
                    } else {
                        d.sqrt()
                    };
                    d + (2.0 * s - 1.0) * (curve - d)
                }
            }
            BlendMode::Difference => (s - d).abs(),
            BlendMode::Exclusion => s + d - 2.0 * s * d,
        };
        dst[c] = (((sa * (1.0 - da) * s + sa * da * mixed + (1.0 - sa) * da * d) / alpha) * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8;
    }
    dst[3] = (alpha * 255.0).round() as u8;
}

pub(super) fn source_image(path: &str) -> Option<Arc<RgbaImage>> {
    static IMAGES: OnceLock<Mutex<HashMap<String, Arc<RgbaImage>>>> = OnceLock::new();
    let cache = IMAGES.get_or_init(Mutex::default);
    if let Some(found) = cache.lock().unwrap().get(path).cloned() {
        return Some(found);
    }
    let source = image::open(path).ok()?.into_rgba8();
    let source = if source.width().max(source.height()) > 2560 {
        image::DynamicImage::ImageRgba8(source)
            .thumbnail(2560, 2560)
            .into_rgba8()
    } else {
        source
    };
    let image = Arc::new(source);
    let mut cache = cache.lock().unwrap();
    if cache.len() >= 16 {
        cache.clear();
    }
    cache.insert(path.into(), image.clone());
    Some(image)
}

pub(super) fn render(scene: &DesignDocument, width: u32, now: u64) -> RgbaImage {
    let height = (width as f32 / scene.aspect().clamp(0.25, 4.0)).round() as u32;
    let mut canvas = RgbaImage::from_pixel(
        width,
        height,
        Rgba(if scene.background_mode == "color" {
            scene.background
        } else {
            [0, 0, 0, 0]
        }),
    );
    let scale = height as f32 / 720.0;
    for layer in scene
        .layers
        .iter()
        .filter(|layer| layer.visible && layer.opacity > 0.0)
    {
        let x = (layer.rect[0] * width as f32).round() as i32;
        let y = (layer.rect[1] * height as f32).round() as i32;
        let w = (layer.rect[2].clamp(0.001, 1.0) * width as f32)
            .round()
            .max(1.0) as u32;
        let h = (layer.rect[3].clamp(0.001, 1.0) * height as f32)
            .round()
            .max(1.0) as u32;
        let source = match &layer.content {
            LayerContent::Vector{svg}=>super::vector::render(svg,w,h).unwrap_or_else(||RgbaImage::new(w,h)),
            LayerContent::Image { path, cover } => {
                if let Some(source) = source_image(path) {
                    let image = image::DynamicImage::ImageRgba8((*source).clone());
                    if *cover {
                        image
                            .resize_to_fill(w, h, image::imageops::FilterType::Triangle)
                            .into_rgba8()
                    } else {
                        let resized = image
                            .resize(w, h, image::imageops::FilterType::Triangle)
                            .into_rgba8();
                        let mut output = RgbaImage::new(w, h);
                        image::imageops::overlay(
                            &mut output,
                            &resized,
                            ((w - resized.width()) / 2) as i64,
                            ((h - resized.height()) / 2) as i64,
                        );
                        output
                    }
                } else {
                    let mut output = RgbaImage::from_pixel(w, h, Rgba([90, 40, 55, 255]));
                    super::text::draw(&mut output, "Imagem indisponível", layer, scale);
                    output
                }
            }
            LayerContent::Shape { radius } => {
                let mut output = RgbaImage::from_pixel(w, h, Rgba(layer.color));
                let r = (radius * scale).min(w.min(h) as f32 / 2.0).max(0.0);
                if r > 0.0 {
                    for (px, py, pixel) in output.enumerate_pixels_mut() {
                        let dx = (r - px as f32 - 0.5)
                            .max(px as f32 + 0.5 - (w as f32 - r))
                            .max(0.0);
                        let dy = (r - py as f32 - 0.5)
                            .max(py as f32 + 0.5 - (h as f32 - r))
                            .max(0.0);
                        let coverage = (r + 0.5 - (dx * dx + dy * dy).sqrt()).clamp(0.0, 1.0);
                        pixel[3] = (pixel[3] as f32 * coverage) as u8;
                    }
                }
                output
            }
            _ => {
                let mut output = RgbaImage::new(w, h);
                if let Some(text) = layer.content.text_at(now) {
                    super::text::draw(&mut output, &text, layer, scale);
                }
                output
            }
        };
        for (sx, sy, pixel) in source.enumerate_pixels() {
            let dx = x + sx as i32;
            let dy = y + sy as i32;
            if dx >= 0 && dy >= 0 && dx < width as i32 && dy < height as i32 {
                composite(
                    canvas.get_pixel_mut(dx as u32, dy as u32),
                    *pixel,
                    layer.opacity,
                    layer.blend,
                );
            }
        }
    }
    canvas
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::design::DesignLayer;
    #[test]
    fn blend_alpha_and_stacking() {
        let mut base = Rgba([100, 100, 100, 255]);
        composite(&mut base, Rgba([200, 0, 0, 255]), 0.5, BlendMode::Normal);
        assert_eq!(base, Rgba([150, 50, 50, 255]));
        let mut transparent = Rgba([0, 0, 0, 0]);
        composite(
            &mut transparent,
            Rgba([200, 50, 10, 128]),
            1.0,
            BlendMode::Multiply,
        );
        assert_eq!(transparent, Rgba([200, 50, 10, 128]));
        let mut scene = DesignDocument::default();
        let mut layer = DesignLayer::new(LayerContent::Shape { radius: 0.0 });
        layer.rect = [0.0, 0.0, 1.0, 1.0];
        layer.color = [255, 0, 0, 255];
        scene.layers.push(layer.clone());
        layer.color = [0, 0, 255, 255];
        layer.rect = [0.5, 0.0, 0.5, 1.0];
        scene.layers.push(layer);
        let result = render(&scene, 160, 0);
        assert_eq!(*result.get_pixel(20, 20), Rgba([255, 0, 0, 255]));
        assert_eq!(*result.get_pixel(120, 20), Rgba([0, 0, 255, 255]));
    }
}
