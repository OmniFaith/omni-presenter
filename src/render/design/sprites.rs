//! Cache layer content, never its position: moving a layer only changes GPU uniforms.
use crate::core::{
    design::{BlendMode, DesignDocument, LayerContent, now_ms},
    models::PresentationTheme,
};
use image::{Rgba, RgbaImage};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::{Arc, Mutex, OnceLock},
};

pub(super) struct Sprite {
    pub key: u64,
    pub image: Arc<RgbaImage>,
    pub rect: [f32; 4],
    pub content: [f32; 2],
    pub opacity: f32,
    pub blend: BlendMode,
    pub fit: f32,
    pub radius: f32,
    pub shape: bool,
    pub shape_kind: f32,
    pub line_width: f32,
    pub rotation: f32,
    pub blur: f32,
    pub stroke: [f32; 2],
    pub stroke_color: [u8; 4],
    pub shadow: [f32; 4],
    pub shadow_color: [u8; 4],
    pub tint: [u8; 4],
}

fn bounds(layer: &crate::core::design::DesignLayer, aspect: f32) -> [f32; 4] {
    let width = layer.rect[2] * aspect * 720.0;
    let height = layer.rect[3] * 720.0;
    let pad = layer.blur
        + if layer.stroke.enabled {
            layer.stroke.width
        } else {
            0.0
        }
        + if layer.shadow.enabled {
            layer.shadow.blur + layer.shadow.x.abs().max(layer.shadow.y.abs())
        } else {
            0.0
        };
    let angle = layer.rotation.to_radians();
    let (sin, cos) = angle.sin_cos();
    let half_w = (width * 0.5 + pad) * cos.abs() + (height * 0.5 + pad) * sin.abs();
    let half_h = (width * 0.5 + pad) * sin.abs() + (height * 0.5 + pad) * cos.abs();
    let cx = layer.rect[0] + layer.rect[2] * 0.5;
    let cy = layer.rect[1] + layer.rect[3] * 0.5;
    [
        cx - half_w / (aspect * 720.0),
        cy - half_h / 720.0,
        half_w * 2.0 / (aspect * 720.0),
        half_h * 2.0 / 720.0,
    ]
}

fn theme_background(
    theme: &PresentationTheme,
    ctx: Option<&eframe::egui::Context>,
) -> Option<Arc<RgbaImage>> {
    static CACHE: OnceLock<Mutex<HashMap<u64, Arc<RgbaImage>>>> = OnceLock::new();
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    theme.id.hash(&mut hash);
    theme.bg_type.hash(&mut hash);
    theme.bg_image_path.hash(&mut hash);
    theme.bg_top.r().hash(&mut hash);
    theme.bg_top.g().hash(&mut hash);
    theme.bg_top.b().hash(&mut hash);
    theme.bg_bottom.r().hash(&mut hash);
    theme.bg_bottom.g().hash(&mut hash);
    theme.bg_bottom.b().hash(&mut hash);
    theme.is_gradient.hash(&mut hash);
    theme.options.gradient_angle.to_bits().hash(&mut hash);
    theme
        .options
        .background_brightness
        .to_bits()
        .hash(&mut hash);
    theme.options.background_overlay.to_bits().hash(&mut hash);
    let key = hash.finish();
    let cache = CACHE.get_or_init(Mutex::default);
    if let Some(image) = cache.lock().unwrap().get(&key).cloned() {
        return Some(image);
    }

    let mut image = if theme.bg_type == "image" && !theme.bg_image_path.is_empty() {
        load_image(&theme.bg_image_path, ctx)
            .or_else(|| {
                if ctx.is_none() {
                    Some(Arc::new(gradient_background(theme)))
                } else {
                    None
                }
            })?
            .as_ref()
            .clone()
    } else {
        gradient_background(theme)
    };
    let brightness = theme.options.background_brightness;
    let overlay = theme.options.background_overlay.clamp(0.0, 1.0);
    if (brightness - 1.0).abs() > f32::EPSILON || overlay > 0.0 {
        for pixel in image.pixels_mut() {
            for channel in 0..3 {
                let mut value = pixel[channel] as f32;
                if brightness < 1.0 {
                    value *= brightness.clamp(0.0, 1.0);
                } else if brightness > 1.0 {
                    let alpha = ((brightness - 1.0) * 0.35).clamp(0.0, 1.0);
                    value = value * (1.0 - alpha) + 255.0 * alpha;
                }
                if overlay > 0.0 {
                    value *= 1.0 - overlay;
                }
                pixel[channel] = value.round().clamp(0.0, 255.0) as u8;
            }
            pixel[3] = 255;
        }
    }
    let image = Arc::new(image);
    let mut cache = cache.lock().unwrap();
    if cache.len() > 16 {
        cache.clear();
    }
    cache.insert(key, image.clone());
    Some(image)
}

fn load_image(path: &str, ctx: Option<&eframe::egui::Context>) -> Option<Arc<RgbaImage>> {
    match ctx {
        Some(ctx) => match crate::services::images::request(ctx, path, 2560) {
            crate::services::images::ImageLoad::Ready(image) => Some(image.rgba),
            _ => None,
        },
        None => super::raster::source_image(path),
    }
}

fn gradient_background(theme: &PresentationTheme) -> RgbaImage {
    if !theme.is_gradient && theme.bg_type != "gradient" {
        return RgbaImage::from_pixel(
            1,
            1,
            Rgba([theme.bg_top.r(), theme.bg_top.g(), theme.bg_top.b(), 255]),
        );
    }
    let angle = theme.options.gradient_angle.rem_euclid(360.0);
    if angle.abs() < 0.01 {
        let steps = 256;
        let mut image = RgbaImage::new(1, steps);
        for y in 0..steps {
            let t = y as f32 / (steps - 1) as f32;
            let mix = |top: u8, bottom: u8| -> u8 {
                (top as f32 + (bottom as f32 - top as f32) * t)
                    .round()
                    .clamp(0.0, 255.0) as u8
            };
            image.put_pixel(
                0,
                y,
                Rgba([
                    mix(theme.bg_top.r(), theme.bg_bottom.r()),
                    mix(theme.bg_top.g(), theme.bg_bottom.g()),
                    mix(theme.bg_top.b(), theme.bg_bottom.b()),
                    255,
                ]),
            );
        }
        return image;
    }

    let size = 256;
    let mut image = RgbaImage::new(size, size);
    let rad = angle.to_radians();
    let dir_x = rad.sin();
    let dir_y = rad.cos();
    let half = (size as f32) * 0.5;
    let max_p = (half * dir_x.abs() + half * dir_y.abs()).max(0.001);

    for y in 0..size {
        let dy = y as f32 - half;
        for x in 0..size {
            let dx = x as f32 - half;
            let p = dx * dir_x + dy * dir_y;
            let t = ((p + max_p) / (2.0 * max_p)).clamp(0.0, 1.0);
            let mix = |top: u8, bottom: u8| -> u8 {
                (top as f32 + (bottom as f32 - top as f32) * t)
                    .round()
                    .clamp(0.0, 255.0) as u8
            };
            image.put_pixel(
                x,
                y,
                Rgba([
                    mix(theme.bg_top.r(), theme.bg_bottom.r()),
                    mix(theme.bg_top.g(), theme.bg_bottom.g()),
                    mix(theme.bg_top.b(), theme.bg_bottom.b()),
                    255,
                ]),
            );
        }
    }
    image
}

fn theme_background_sprite(
    theme: &PresentationTheme,
    ctx: Option<&eframe::egui::Context>,
) -> Option<Sprite> {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    "design-theme-background".hash(&mut hash);
    theme.id.hash(&mut hash);
    theme.bg_type.hash(&mut hash);
    theme.bg_image_path.hash(&mut hash);
    theme.bg_top.r().hash(&mut hash);
    theme.bg_top.g().hash(&mut hash);
    theme.bg_top.b().hash(&mut hash);
    theme.bg_bottom.r().hash(&mut hash);
    theme.bg_bottom.g().hash(&mut hash);
    theme.bg_bottom.b().hash(&mut hash);
    theme.options.gradient_angle.to_bits().hash(&mut hash);
    theme
        .options
        .background_brightness
        .to_bits()
        .hash(&mut hash);
    theme.options.background_overlay.to_bits().hash(&mut hash);
    Some(Sprite {
        key: hash.finish(),
        image: theme_background(theme, ctx)?,
        rect: [0.0, 0.0, 1.0, 1.0],
        content: [1.0, 1.0],
        opacity: 1.0,
        blend: BlendMode::Normal,
        fit: 0.0,
        radius: 0.0,
        shape: false,
        shape_kind: 0.0,
        line_width: 1.0,
        rotation: 0.0,
        blur: 0.0,
        stroke: [0.0, 0.0],
        stroke_color: [0; 4],
        shadow: [0.0; 4],
        shadow_color: [0; 4],
        tint: [255; 4],
    })
}

#[cfg(test)]
pub(super) fn layers(
    scene: &DesignDocument,
    aspect: f32,
    theme: Option<&PresentationTheme>,
    ctx: Option<&eframe::egui::Context>,
) -> Vec<Sprite> {
    layers_at(scene, aspect, theme, ctx, 720)
}

pub(super) fn layers_at(
    scene: &DesignDocument, aspect: f32, theme: Option<&PresentationTheme>,
    ctx: Option<&eframe::egui::Context>, target_height: u32,
) -> Vec<Sprite> {
    static TEXT: OnceLock<Mutex<HashMap<u64, Arc<RgbaImage>>>> = OnceLock::new();
    static WHITE: OnceLock<Arc<RgbaImage>> = OnceLock::new();
    let white = || {
        WHITE
            .get_or_init(|| Arc::new(RgbaImage::from_pixel(1, 1, Rgba([255; 4]))))
            .clone()
    };
    let cache = TEXT.get_or_init(Mutex::default);
    let now = now_ms();
    let mut sprites = Vec::new();
    if scene.background_mode == "theme"
        && let Some(theme) = theme
        && let Some(background) = theme_background_sprite(theme, ctx) {
            sprites.push(background);
        }
    sprites.extend(
        scene
            .layers
            .iter()
            .filter(|layer| layer.visible && layer.opacity > 0.0)
            .filter_map(|layer| {
                let mut hash = std::collections::hash_map::DefaultHasher::new();
                let mut sprite = Sprite {
                    key: 0,
                    image: white(),
                    rect: bounds(layer, aspect),
                    content: [layer.rect[2], layer.rect[3]],
                    opacity: layer.opacity,
                    blend: layer.blend,
                    fit: 0.0,
                    radius: 0.0,
                    shape: false,
                    shape_kind: 0.0,
                    line_width: layer.line_width,
                    rotation: layer.rotation.to_radians(),
                    blur: layer.blur,
                    stroke: [
                        if layer.stroke.enabled { 1.0 } else { 0.0 },
                        layer.stroke.width,
                    ],
                    stroke_color: layer.stroke.color,
                    shadow: [
                        if layer.shadow.enabled { 1.0 } else { 0.0 },
                        layer.shadow.blur,
                        layer.shadow.x,
                        layer.shadow.y,
                    ],
                    shadow_color: layer.shadow.color,
                    tint: [255; 4],
                };
                match &layer.content {
                    LayerContent::Vector{svg}=>{
                        let height=target_height.max(720)as f32;
                        let w=(layer.rect[2]*height*aspect).round().max(1.)as u32;
                        let h=(layer.rect[3]*height).round().max(1.)as u32;
                        svg.hash(&mut hash);w.hash(&mut hash);h.hash(&mut hash);
                        let key=hash.finish();
                        let mut cache=cache.lock().unwrap();
                        sprite.image=if let Some(image)=cache.get(&key){image.clone()}else{
                            let image=Arc::new(super::vector::render(svg,w,h)?);
                            if cache.len()>96{cache.clear();}cache.insert(key,image.clone());image
                        };
                    }
                    LayerContent::Image { path, cover } => {
                        path.hash(&mut hash);
                        sprite.image = load_image(path, ctx)?;
                        sprite.fit = if *cover { 1.0 } else { 2.0 };
                    }
                    LayerContent::Shape { radius } => {
                        "shape".hash(&mut hash);
                        sprite.shape = true;
                        sprite.shape_kind = layer.shape as u8 as f32;
                        sprite.radius = *radius;
                        sprite.tint = layer.color;
                    }
                    _ => {
                        let height = [360,720,1080,1440,2160,2880,4096].into_iter().find(|height| *height >= target_height).unwrap_or(4096) as f32;
                        let w = (layer.rect[2] * height * aspect).round().max(1.0) as u32;
                        let h = (layer.rect[3] * height).round().max(1.0) as u32;
                        let text = layer.content.text_at(now).unwrap_or_default();
                        w.hash(&mut hash);
                        h.hash(&mut hash);
                        text.hash(&mut hash);
                        layer.font_family.hash(&mut hash);
                        layer.font_size.to_bits().hash(&mut hash);
                        layer.effective_font_weight().hash(&mut hash);
                        layer.align.hash(&mut hash);
                        layer.color.hash(&mut hash);
                        layer.text_style.line_height.to_bits().hash(&mut hash);
                        layer.text_style.letter_spacing.to_bits().hash(&mut hash);
                        layer.text_style.paragraph_spacing.to_bits().hash(&mut hash);
                        layer.text_style.italic.hash(&mut hash);
                        layer.text_style.underline.hash(&mut hash);
                        layer.text_style.strike.hash(&mut hash);
                        layer.text_style.vertical.hash(&mut hash);
                        layer.text_style.case.hash(&mut hash);
                        layer.text_style.wrap.hash(&mut hash);
                        layer.text_style.auto_fit.hash(&mut hash);
                        let key = hash.finish();
                        let mut cache = cache.lock().unwrap();
                        sprite.image = cache.get(&key).cloned().unwrap_or_else(|| {
                            let mut image = RgbaImage::new(w, h);
                            super::text::draw(&mut image, &text, layer, height / 720.0);
                            let image = Arc::new(image);
                            if cache.len() > 96 {
                                cache.clear();
                            }
                            cache.insert(key, image.clone());
                            image
                        });
                    }
                }
                sprite.key = hash.finish();
                Some(sprite)
            }),
    );
    sprites
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn changing_font_weight_invalidates_the_cached_text_texture() {
        let mut scene = DesignDocument::default();
        let mut layer = crate::core::design::DesignLayer::new(LayerContent::Text("Peso da letra".into()));
        layer.font_family = "Serif".into();
        layer.set_font_weight(300);
        scene.layers.push(layer);
        let light = layers(&scene, 16.0 / 9.0, None, None);
        scene.layers[0].set_font_weight(500);
        let medium = layers(&scene, 16.0 / 9.0, None, None);
        assert_ne!(light[0].key, medium[0].key);
        assert!(!Arc::ptr_eq(&light[0].image, &medium[0].image));
    }

    #[test]
    fn native_image_layers_appear_after_background_decode_without_caching_a_placeholder() {
        let path = std::env::temp_dir().join(format!(
            "omni-presenter-design-image-{}.png",
            crate::core::design::new_id()
        ));
        RgbaImage::from_pixel(80, 60, Rgba([140, 30, 80, 255]))
            .save(&path)
            .unwrap();
        let mut scene = DesignDocument::default();
        scene
            .layers
            .push(crate::core::design::DesignLayer::new(LayerContent::Image {
                path: path.to_string_lossy().into_owned(),
                cover: true,
            }));
        let ctx = eframe::egui::Context::default();
        assert!(layers(&scene, 16.0 / 9.0, None, Some(&ctx)).is_empty());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            let loaded = layers(&scene, 16.0 / 9.0, None, Some(&ctx));
            if let Some(image) = loaded.first() {
                assert_eq!(image.image.dimensions(), (80, 60));
                assert_eq!(*image.image.get_pixel(0, 0), Rgba([140, 30, 80, 255]));
                let repeated = layers(&scene, 16.0 / 9.0, None, Some(&ctx));
                assert_eq!(image.key, repeated[0].key);
                assert!(Arc::ptr_eq(&image.image, &repeated[0].image));
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn moving_a_layer_reuses_its_content_texture() {
        let mut scene = DesignDocument::template(2);
        let first = layers(&scene, 16.0 / 9.0, None, None);
        for layer in &mut scene.layers {
            layer.rect[0] += 0.01;
            layer.rect[1] += 0.02;
        }
        let moved = layers(&scene, 16.0 / 9.0, None, None);
        for (first, moved) in first.iter().zip(&moved) {
            assert_eq!(first.key, moved.key);
            assert!(
                Arc::ptr_eq(&first.image, &moved.image),
                "moving must not re-rasterize layer content"
            );
            assert_ne!(first.rect, moved.rect);
        }
    }
    #[test]
    fn layer_with_shadow_preserves_fill_and_sets_sprite_shadow() {
        let mut scene = DesignDocument::default();
        let mut layer = crate::core::design::DesignLayer::new(LayerContent::Shape { radius: 16.0 });
        layer.color = [23, 32, 54, 230];
        layer.shadow = crate::core::design_style::LayerShadow {
            enabled: true,
            blur: 24.0,
            x: 0.0,
            y: 8.0,
            color: [0, 0, 0, 180],
        };
        scene.layers.push(layer);
        let sprites = layers(&scene, 16.0 / 9.0, None, None);
        assert_eq!(sprites.len(), 1);
        let sprite = &sprites[0];
        assert_eq!(sprite.tint, [23, 32, 54, 230]);
        assert_eq!(sprite.shadow[0], 1.0);
        assert_eq!(sprite.shadow[1], 24.0);
        assert_eq!(sprite.shadow[2], 0.0);
        assert_eq!(sprite.shadow[3], 8.0);
        assert_eq!(sprite.shadow_color, [0, 0, 0, 180]);
    }
}

