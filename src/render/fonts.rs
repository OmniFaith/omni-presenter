//! The same base/fallback fonts for native windows and exported text layers.
use eframe::egui;
use std::sync::{Arc, OnceLock};

pub(crate) fn definitions() -> egui::FontDefinitions {
    static FONTS: OnceLock<egui::FontDefinitions> = OnceLock::new();
    FONTS
        .get_or_init(|| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "custom-icons".into(),
                Arc::new(egui::FontData::from_static(include_bytes!(
                    "../../assets/fonts/custom-icons.ttf"
                ))),
            );
            fonts.font_data.insert(
                "phosphor".into(),
                Arc::new(egui::FontData::from_static(
                    egui_phosphor::Variant::Regular.font_bytes(),
                )),
            );
            if let Some(keys) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                keys.extend(["custom-icons".into(), "phosphor".into()]);
            }
            if let Some(bytes) = crate::core::models::fonts::color_emoji_font_bytes() {
                fonts.font_data.insert(
                    "color-emoji".into(),
                    Arc::new(egui::FontData::from_owned(bytes)),
                );
                for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                    if let Some(keys) = fonts.families.get_mut(&family) {
                        keys.push("color-emoji".into());
                    }
                }
            }
            fonts
        })
        .clone()
}
