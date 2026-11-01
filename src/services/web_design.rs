//! Publish editable scene data and its assets, not a fixed-size screenshot.
use super::Published;
use crate::core::design::{DesignDocument, LayerContent, now_ms};
use std::hash::{Hash, Hasher};

fn key(value: &str) -> String {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hash);
    format!("{:016x}", hash.finish())
}

pub(super) fn prepare(scene: &DesignDocument, data: &mut Published) -> serde_json::Value {
    let resolved = crate::core::plugins::runtime::materialize(scene, now_ms());
    let scene = resolved.as_ref();
    let mut value = serde_json::to_value(scene).unwrap_or_default();
    let now = now_ms();
    let mut used = std::collections::HashSet::new();
    if let Some(layers) = value["layers"].as_array_mut() {
        layers.retain(|layer| layer["visible"] == true);
        for (layer, output) in scene
            .layers
            .iter()
            .filter(|layer| layer.visible)
            .zip(layers)
        {
            if let LayerContent::Image { path, .. } = &layer.content {
                let id = key(&format!("image:{path}"));
                used.insert(id.clone());
                if !data.design_assets.contains_key(&id)
                    && let Ok(bytes) = std::fs::read(path)
                {
                    let mime = match image::guess_format(&bytes).ok() {
                        Some(image::ImageFormat::Png) => "image/png",
                        Some(image::ImageFormat::Jpeg) => "image/jpeg",
                        Some(image::ImageFormat::WebP) => "image/webp",
                        Some(image::ImageFormat::Gif) => "image/gif",
                        _ => "application/octet-stream",
                    };
                    data.design_assets.insert(id.clone(), (bytes, mime));
                }
                output["imageUrl"] = format!("/design-assets/{id}").into();
                if let Some(image) = output["content"]["Image"].as_object_mut() {
                    image.remove("path");
                }
            } else if let Some(text) = layer.content.text_at(now) {
                output["displayText"] = text.into();
                let weight = layer.effective_font_weight();
                output["font_weight"] = weight.into();
                let id = key(&format!(
                    "font:{}:{}:{}", layer.font_family, weight, layer.text_style.italic,
                ));
                used.insert(id.clone());
                data.design_assets.entry(id.clone()).or_insert_with(|| {
                    (
                        crate::core::models::fonts::presentation_font_bytes(
                            &layer.font_family,
                            weight,
                            layer.text_style.italic,
                        ),
                        "font/ttf",
                    )
                });
                output["fontUrl"] = format!("/design-assets/{id}").into();
            }
        }
    }
    if data.design_assets.len() > 64 {
        data.design_assets.retain(|key, _| used.contains(key));
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::design::DesignLayer;
    #[test]
    fn plugin_components_publish_portable_layers_without_scripts() {
        let plugin = crate::core::plugins::bundled().remove(0);
        let instance = crate::core::plugins::PluginInstance::new(&plugin, &plugin.package.components[0]);
        let scene = DesignDocument { layers: vec![DesignLayer::new(LayerContent::Plugin(Box::new(instance)))], ..Default::default() };
        let mut data = Published::default();
        let value = prepare(&scene, &mut data);
        let layers = value["layers"].as_array().unwrap();
        assert!(!layers.is_empty());
        assert!(!value.to_string().contains("score_a"));
        let texts: Vec<&str> = layers
            .iter()
            .filter_map(|layer| layer["displayText"].as_str())
            .collect();
        assert!(texts.iter().any(|t| t.contains('0')), "score text should be present: {texts:?}");
    }
    #[test]
    fn scene_assets_and_dynamic_text_are_published_without_local_paths() {
        let path = std::env::temp_dir().join(format!(
            "omni-presenter-design-{}.png",
            crate::core::design::new_id()
        ));
        image::RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 128]))
            .save(&path)
            .unwrap();
        let mut scene = DesignDocument::default();
        scene.layers.push(DesignLayer::new(LayerContent::Image {
            path: path.to_string_lossy().into_owned(),
            cover: true,
        }));
        scene.layers.push(DesignLayer::new(LayerContent::Clock {
            seconds: true,
            date: false,
        }));
        let mut hidden = scene.layers[0].clone();
        hidden.visible = false;
        scene.layers.push(hidden);
        let mut data = Published::default();
        let value = prepare(&scene, &mut data);
        assert!(!value.to_string().contains(path.to_str().unwrap()));
        assert_eq!(value["layers"].as_array().unwrap().len(), 2);
        let key = value["layers"][0]["imageUrl"]
            .as_str()
            .unwrap()
            .strip_prefix("/design-assets/")
            .unwrap();
        let (bytes, mime) = &data.design_assets[key];
        assert_eq!(*mime, "image/png");
        assert_eq!(image::load_from_memory(bytes).unwrap().width(), 2);
        assert_eq!(value["layers"][1]["displayText"].as_str().unwrap().len(), 8);
        assert!(
            value["layers"][1]["fontUrl"]
                .as_str()
                .unwrap()
                .starts_with("/design-assets/")
        );
        let count = data.design_assets.len();
        prepare(&scene, &mut data);
        assert_eq!(data.design_assets.len(), count);
        std::fs::remove_file(path).unwrap();
    }
}
