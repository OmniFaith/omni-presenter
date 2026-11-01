use crate::{
    core::design::{BlendMode, DesignLayer, LayerContent},
    ui::theme::AppTheme,
};
use eframe::egui::{self, RichText};

use super::inspector::{property_slider, section_header};

pub(super) fn blend_mode_label(mode: BlendMode) -> &'static str {
    match mode {
        BlendMode::Normal => "font-normal",
        BlendMode::Multiply => "design-multiply",
        BlendMode::Screen => "design-screen",
        BlendMode::Overlay => "design-overlay",
        BlendMode::Darken => "design-darken",
        BlendMode::Lighten => "design-lighten",
        BlendMode::ColorDodge => "design-color-dodge",
        BlendMode::ColorBurn => "design-color-burn",
        BlendMode::HardLight => "design-hard-light",
        BlendMode::SoftLight => "design-soft-light",
        BlendMode::Difference => "design-difference",
        BlendMode::Exclusion => "design-exclusion",
    }
}

pub(super) fn blend_mode_description(mode: BlendMode) -> &'static str {
    match mode {
        BlendMode::Normal => "design-keeps-the-original-colors",
        BlendMode::Multiply => "design-darkens-and-combines-colors",
        BlendMode::Screen => "design-combines-light-and-brightens",
        BlendMode::Overlay => "design-contrast-between-light-and-shadow",
        BlendMode::Darken => "design-preserves-the-darkest-tones",
        BlendMode::Lighten => "design-preserves-the-lightest-tones",
        BlendMode::ColorDodge => "design-intensifies-bright-areas",
        BlendMode::ColorBurn => "design-intensifies-dark-areas",
        BlendMode::HardLight => "design-light-with-strong-contrast",
        BlendMode::SoftLight => "design-soft-lighting-and-contrast",
        BlendMode::Difference => "design-highlights-the-difference-between-colors",
        BlendMode::Exclusion => "design-difference-with-reduced-contrast",
    }
}

pub(super) fn render_appearance_tab(
    ui: &mut egui::Ui,
    layer: &mut DesignLayer,
    theme: &AppTheme,
    runtime_changed: &mut bool,
) {
    ui.add_enabled_ui(!layer.locked, |ui| {
        if let LayerContent::Plugin(instance) = &mut layer.content {
            let appearance_fields: Vec<crate::core::plugins::PluginField> = instance
                .definition
                .fields
                .iter()
                .filter(|f| f.is_appearance())
                .cloned()
                .collect();
            if !appearance_fields.is_empty() {
                section_header(ui, "design-plugin-appearance", theme);
                let language = crate::ui::i18n::language(ui.ctx());
                *runtime_changed |= crate::ui::components::plugin_fields::fields(
                    ui,
                    &appearance_fields,
                    &mut instance.values,
                    language,
                    theme,
                );
                ui.add_space(6.0);
            }
        }
        if !matches!(layer.content, LayerContent::Image { .. }) {
            ui.horizontal(|ui| {
                ui.add_sized(
                    [68.0, 20.0],
                    egui::Label::new(RichText::new(crate::ui::i18n::text(ui, "common-color")).size(11.5).color(theme.text_secondary)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    crate::ui::components::color_swatch_u8_4(ui, "layer_color", &mut layer.color, true, theme);
                });
            });
            ui.add_space(2.0);
        }
        let mut opacity = layer.opacity * 100.0;
        if property_slider(ui, "design-opacity", &mut opacity, 0.0..=100.0, "%", 0, theme) {
            layer.opacity = opacity / 100.0;
        }
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.add_sized(
                [68.0, 20.0],
                egui::Label::new(RichText::new(crate::ui::i18n::text(ui, "design-blend")).size(11.5).color(theme.text_secondary)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("design_blend")
                    .selected_text(crate::ui::i18n::text(ui, blend_mode_label(layer.blend)))
                    .width((ui.available_width() - 4.0).clamp(80.0, 160.0))
                    .show_ui(ui, |ui| {
                        for mode in BlendMode::ALL {
                            ui.selectable_value(&mut layer.blend, mode, crate::ui::i18n::text(ui, blend_mode_label(mode)))
                                .on_hover_text(crate::ui::i18n::text(ui, blend_mode_description(mode)));
                        }
                    });
            });
        });
        ui.add_space(6.0);

        section_header(ui, "design-effects", theme);
        let has_effects = layer.rotation != 0.0 || layer.blur != 0.0;
        if has_effects {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button(RichText::new(crate::ui::i18n::text(ui, "design-reset")).size(10.5).color(theme.text_muted))
                    .on_hover_text(crate::ui::i18n::text(ui, "design-reset-rotation-and-blur"))
                    .clicked()
                {
                    layer.rotation = 0.0;
                    layer.blur = 0.0;
                }
            });
        }
        ui.add_space(2.0);
        property_slider(ui, "design-rotation", &mut layer.rotation, -180.0..=180.0, "°", 0, theme);
        ui.add_space(2.0);
        property_slider(ui, "design-blur", &mut layer.blur, 0.0..=60.0, "", 1, theme);

        ui.add_space(6.0);
        section_header(ui, "design-outline-and-shadow", theme);
        ui.checkbox(&mut layer.stroke.enabled, RichText::new(crate::ui::i18n::text(ui, "design-outline")).size(11.5));
        if layer.stroke.enabled {
            ui.indent("stroke_indent", |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                property_slider(ui, "design-thickness", &mut layer.stroke.width, 0.5..=30.0, " px", 1, theme);
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [68.0, 20.0],
                        egui::Label::new(RichText::new(crate::ui::i18n::text(ui, "common-color")).size(11.5).color(theme.text_secondary)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        crate::ui::components::color_swatch_u8_4(
                            ui, "layer_stroke_color", &mut layer.stroke.color, true, theme,
                        );
                    });
                });
            });
            ui.add_space(2.0);
        }
        ui.checkbox(&mut layer.shadow.enabled, RichText::new(crate::ui::i18n::text(ui, "design-shadow")).size(11.5));
        if layer.shadow.enabled {
            ui.indent("shadow_indent", |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                property_slider(ui, "design-blur", &mut layer.shadow.blur, 0.0..=80.0, "", 1, theme);
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [68.0, 20.0],
                        egui::Label::new(RichText::new(crate::ui::i18n::text(ui, "common-color")).size(11.5).color(theme.text_secondary)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        crate::ui::components::color_swatch_u8_4(
                            ui, "layer_shadow_color", &mut layer.shadow.color, true, theme,
                        );
                    });
                });
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [68.0, 20.0],
                        egui::Label::new(RichText::new(crate::ui::i18n::text(ui, "design-position")).size(11.0).color(theme.text_muted)),
                    );
                    let rem = ui.available_width();
                    let drag_w = ((rem - 6.0) * 0.5).max(40.0);
                    ui.add_sized([drag_w, 20.0], egui::DragValue::new(&mut layer.shadow.x).speed(0.5).prefix("X: "));
                    ui.add_sized([drag_w, 20.0], egui::DragValue::new(&mut layer.shadow.y).speed(0.5).prefix("Y: "));
                });
            });
        }
    });
}
