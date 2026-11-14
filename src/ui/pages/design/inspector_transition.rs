use crate::{
    core::design::{AnimationEffect, DesignLayer, LayerContent},
    ui::theme::AppTheme,
};
use eframe::egui::{self, RichText};

use super::{controls, inspector::section_header, slide_controls};

#[allow(clippy::too_many_arguments)]
pub(super) fn render_transition_tab(
    ui: &mut egui::Ui,
    layer: &mut DesignLayer,
    transition: &mut String,
    transition_ms: &mut u32,
    transition_strength: &mut f32,
    transition_easing: &mut String,
    theme: &AppTheme,
    runtime_changed: &mut bool,
) {
    ui.add_enabled_ui(!layer.locked, |ui| {
        // --- 1. Layer Entrance Animation ---
        section_header(ui, "design-in-animation", theme);
        ui.horizontal(|ui| {
            ui.label(RichText::new(crate::ui::i18n::text(ui, "design-effect")).size(12.0).color(theme.text_secondary));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("layer_anim_in_effect")
                    .selected_text(crate::ui::i18n::text(ui, layer.animation.in_effect.label()))
                    .width((ui.available_width() - 4.0).clamp(110.0, 180.0))
                    .show_ui(ui, |ui| {
                        for eff in AnimationEffect::ALL {
                            ui.selectable_value(
                                &mut layer.animation.in_effect,
                                eff,
                                crate::ui::i18n::text(ui, eff.label()),
                            );
                        }
                    });
            });
        });

        if layer.animation.in_effect != AnimationEffect::None {
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(crate::ui::i18n::text(ui, "design-duration-ms")).size(12.0).color(theme.text_secondary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(
                        egui::DragValue::new(&mut layer.animation.in_duration_ms)
                            .range(100..=5000)
                            .speed(25)
                            .suffix(" ms"),
                    );
                });
            });
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(crate::ui::i18n::text(ui, "design-delay")).size(12.0).color(theme.text_secondary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(
                        egui::DragValue::new(&mut layer.animation.in_delay_ms)
                            .range(0..=10000)
                            .speed(50)
                            .suffix(" ms"),
                    );
                });
            });
        }

        ui.add_space(6.0);

        // --- 2. Layer Exit Animation ---
        section_header(ui, "design-out-animation", theme);
        ui.horizontal(|ui| {
            ui.label(RichText::new(crate::ui::i18n::text(ui, "design-effect")).size(12.0).color(theme.text_secondary));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("layer_anim_out_effect")
                    .selected_text(crate::ui::i18n::text(ui, layer.animation.out_effect.label()))
                    .width((ui.available_width() - 4.0).clamp(110.0, 180.0))
                    .show_ui(ui, |ui| {
                        for eff in AnimationEffect::ALL {
                            ui.selectable_value(
                                &mut layer.animation.out_effect,
                                eff,
                                crate::ui::i18n::text(ui, eff.label()),
                            );
                        }
                    });
            });
        });

        if layer.animation.out_effect != AnimationEffect::None {
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(crate::ui::i18n::text(ui, "design-duration-ms")).size(12.0).color(theme.text_secondary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(
                        egui::DragValue::new(&mut layer.animation.out_duration_ms)
                            .range(100..=5000)
                            .speed(25)
                            .suffix(" ms"),
                    );
                });
            });
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(crate::ui::i18n::text(ui, "design-delay")).size(12.0).color(theme.text_secondary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(
                        egui::DragValue::new(&mut layer.animation.out_delay_ms)
                            .range(0..=5000)
                            .speed(50)
                            .suffix(" ms"),
                    );
                });
            });
        }

        if layer.animation.is_animated() {
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(crate::ui::i18n::text(ui, "design-easing")).size(12.0).color(theme.text_secondary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("layer_anim_easing")
                        .selected_text(crate::ui::i18n::easing(ui, &layer.animation.easing))
                        .width((ui.available_width() - 4.0).clamp(90.0, 160.0))
                        .show_ui(ui, |ui| {
                            for easing in ["smooth", "linear", "ease_in", "ease_out", "ease_in_out"] {
                                ui.selectable_value(&mut layer.animation.easing, easing.into(), crate::ui::i18n::easing(ui, easing));
                            }
                        });
                });
            });
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // --- 3. Plugin / Slide Transition ---
        let is_plugin = matches!(&layer.content, LayerContent::Plugin(_));
        if let LayerContent::Plugin(instance) = &mut layer.content {
            let has_trans = instance.definition.fields.iter().any(|f| f.is_transition());
            if has_trans {
                section_header(ui, "design-transition-between-texts", theme);
                let curr_trans = instance
                    .values
                    .get("transition")
                    .and_then(|v| v.as_str())
                    .unwrap_or(if transition.as_str() != "cut" { transition.as_str() } else { "fade" })
                    .to_string();
                let mut trans_val = curr_trans.clone();
                let prev_trans = trans_val.clone();
                ui.horizontal(|ui| {
                    ui.label(RichText::new(crate::ui::i18n::text(ui, "design-effect")).size(12.0).color(theme.text_secondary));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        egui::ComboBox::from_id_salt("plugin_trans_select")
                            .selected_text(match trans_val.as_str() {
                                "slide_h" | "slide" | "slide_left" | "slide_right" | "wipe" => crate::ui::i18n::text(ui, "design-slide-horizontally"),
                                "slide_v" | "slide_up" | "slide_down" => crate::ui::i18n::text(ui, "design-slide-vertically"),
                                "zoom" | "zoom_out" => crate::ui::i18n::text(ui, "design-gentle-zoom"),
                                "cut" | "none" => crate::ui::i18n::text(ui, "design-cut"),
                                _ => crate::ui::i18n::text(ui, "design-dissolve-fade"),
                            })
                            .width((ui.available_width() - 4.0).clamp(110.0, 180.0))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut trans_val, "fade".into(), crate::ui::i18n::text(ui, "design-dissolve-fade"));
                                ui.selectable_value(&mut trans_val, "slide_h".into(), crate::ui::i18n::text(ui, "design-slide-horizontally"));
                                ui.selectable_value(&mut trans_val, "slide_v".into(), crate::ui::i18n::text(ui, "design-slide-vertically"));
                                ui.selectable_value(&mut trans_val, "zoom".into(), crate::ui::i18n::text(ui, "design-gentle-zoom"));
                                ui.selectable_value(&mut trans_val, "cut".into(), crate::ui::i18n::text(ui, "design-cut"));
                            });
                    });
                });
                if trans_val != prev_trans || !instance.values.contains_key("transition") {
                    instance.values.insert("transition".into(), serde_json::Value::String(trans_val));
                    *runtime_changed = true;
                }
                let curr_ms = instance
                    .values
                    .get("transition_ms")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32)
                    .unwrap_or(if *transition_ms > 0 { *transition_ms } else { 800 });
                let mut ms_val = curr_ms;
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(crate::ui::i18n::text(ui, "design-duration-ms")).size(12.0).color(theme.text_secondary));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(
                            egui::DragValue::new(&mut ms_val)
                                .range(100..=5000)
                                .speed(50)
                                .suffix(" ms"),
                        ).changed() {
                            instance.values.insert("transition_ms".into(), serde_json::json!(ms_val));
                            *runtime_changed = true;
                        }
                    });
                });
                if !instance.values.contains_key("transition_ms") {
                    instance.values.insert("transition_ms".into(), serde_json::json!(ms_val));
                }
                ui.add_space(2.0);
                controls::note(
                    ui,
                    "design-transition-applied-automatically-when-switching-this-component-s-texts-and",
                    theme,
                );
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
            }
        }

        let prev_scene_trans = transition.clone();
        let prev_scene_ms = *transition_ms;

        slide_controls::render_transition_controls(
            ui,
            transition,
            transition_ms,
            transition_strength,
            transition_easing,
            theme,
        );

        if (*transition != prev_scene_trans || *transition_ms != prev_scene_ms) && is_plugin
            && let LayerContent::Plugin(instance) = &mut layer.content
                && instance.definition.fields.iter().any(|f| f.is_transition()) {
                    let plugin_trans = match transition.as_str() {
                        "slide_left" | "slide_right" | "wipe" => "slide_h",
                        "slide_up" | "slide_down" => "slide_v",
                        "zoom" | "zoom_out" => "zoom",
                        "cut" => "cut",
                        _ => "fade",
                    };
                    instance.values.insert("transition".into(), serde_json::Value::String(plugin_trans.into()));
                    instance.values.insert("transition_ms".into(), serde_json::json!(*transition_ms));
                    *runtime_changed = true;
                }
    });
}
