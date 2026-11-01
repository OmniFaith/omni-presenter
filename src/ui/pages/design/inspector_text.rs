use crate::{
    core::design::{DesignLayer, LayerContent},
    ui::theme::AppTheme,
};
use eframe::egui::{self, RichText};
use egui_phosphor::regular::*;

use super::inspector::{property_slider, section_header};

pub(super) fn render_text_tab(
    ui: &mut egui::Ui,
    layer: &mut DesignLayer,
    index: usize,
    theme_font: Option<&str>,
    theme: &AppTheme,
    runtime_changed: &mut bool,
) {
    if matches!(
        layer.content,
        LayerContent::Image { .. } | LayerContent::Shape { .. } | LayerContent::Vector { .. }
    ) {
        ui.label(
            RichText::new(crate::ui::i18n::text(ui, "design-this-layer-has-no-editable-text"))
                .size(11.0)
                .color(theme.text_muted),
        );
        return;
    }
    ui.add_enabled_ui(!layer.locked, |ui| {
        match &mut layer.content {
            LayerContent::Plugin(instance) => {
                section_header(ui, "design-plugin-content-and-text", theme);
                let language = crate::ui::i18n::language(ui.ctx());
                *runtime_changed |=
                    crate::ui::components::plugin_fields::component_operational(ui, instance, language, theme);
                ui.add_space(4.0);
            }
            LayerContent::Text(text) => {
                section_header(ui, "design-text-content", theme);
                ui.add(
                    egui::TextEdit::multiline(text)
                        .desired_rows(3)
                        .desired_width(f32::INFINITY)
                        .margin(egui::vec2(9.0, 7.0))
                        .hint_text(crate::ui::i18n::text(ui, "design-enter-text-here")),
                );
                ui.add_space(4.0);
            }
            LayerContent::Countdown(timer) => {
                section_header(ui, "design-text-when-finished", theme);
                ui.add(
                    egui::TextEdit::singleline(&mut timer.finished_text)
                        .desired_width(f32::INFINITY)
                        .hint_text(crate::ui::i18n::text(ui, "design-e-g-let-s-begin")),
                );
                ui.add_space(4.0);
            }
            LayerContent::Questions { items, current, .. } => {
                section_header(ui, "design-questions-and-answers", theme);
                if !items.is_empty() {
                    let c = (*current).min(items.len() - 1);
                    ui.add(
                        egui::TextEdit::multiline(&mut items[c].question)
                            .desired_rows(2)
                            .desired_width(f32::INFINITY)
                            .hint_text(crate::ui::i18n::text(ui, "design-question")),
                    );
                    ui.add(
                        egui::TextEdit::multiline(&mut items[c].answer)
                            .desired_rows(2)
                            .desired_width(f32::INFINITY)
                            .hint_text(crate::ui::i18n::text(ui, "design-answer-label")),
                    );
                }
                ui.add_space(4.0);
            }
            LayerContent::Clock { seconds, date } => {
                section_header(ui, "design-display-options", theme);
                ui.horizontal(|ui| {
                    ui.checkbox(seconds, crate::ui::i18n::text(ui, "design-seconds"));
                    ui.checkbox(date, crate::ui::i18n::text(ui, "design-date"));
                });
                ui.add_space(4.0);
            }
            _ => {}
        }

        section_header(ui, "design-typography-and-style", theme);
        property_slider(ui, "design-size", &mut layer.font_size, 12.0..=240.0, " px", 0, theme);
        ui.add_space(4.0);
        crate::ui::components::font_picker::font_picker_with_theme(ui, ("design_font", index), &mut layer.font_family, ui.available_width(), theme);
        ui.add_space(4.0);
        let effective_font = if layer.font_family == crate::core::models::fonts::THEME_FONT || layer.font_family.is_empty() {
            theme_font.unwrap_or("Montserrat")
        } else {
            &layer.font_family
        };
        let mut weight = layer.effective_font_weight();
        if crate::ui::components::font_picker::font_weight_picker(ui, ("design_weight", index), effective_font, &mut weight, theme) {
            layer.set_font_weight(weight);
        }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_sized(
                [68.0, 20.0],
                egui::Label::new(RichText::new(crate::ui::i18n::text(ui, "design-align")).size(11.5).color(theme.text_secondary)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("design_text_align")
                    .selected_text(match layer.align.as_str() {
                        "left" => crate::ui::i18n::text(ui, "design-left"),
                        "right" => crate::ui::i18n::text(ui, "design-right"),
                        _ => crate::ui::i18n::text(ui, "design-center"),
                    })
                    .width((ui.available_width() - 4.0).clamp(80.0, 160.0))
                    .show_ui(ui, |ui| {
                        for (value, key) in [("left", "design-left"), ("center", "design-center"), ("right", "design-right")] {
                            ui.selectable_value(&mut layer.align, value.into(), crate::ui::i18n::text(ui, key));
                        }
                    });
            });
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let normal_text = format!("{} {}", TEXT_AA, crate::ui::i18n::text(ui, "font-normal"));
            ui.selectable_label(!layer.text_style.italic, normal_text)
                .on_hover_text(crate::ui::i18n::text(ui, "font-normal"));
        });
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            if ui.checkbox(&mut layer.bold, RichText::new(crate::ui::i18n::text(ui, "font-bold")).size(11.0)).changed() {
                layer.set_font_weight(if layer.bold { 700 } else { 400 });
            }
            ui.checkbox(&mut layer.text_style.italic, RichText::new(crate::ui::i18n::text(ui, "font-italic")).size(11.0));
        });
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.checkbox(&mut layer.text_style.underline, RichText::new(crate::ui::i18n::text(ui, "font-underline")).size(11.0));
            ui.checkbox(&mut layer.text_style.strike, RichText::new(crate::ui::i18n::text(ui, "font-strikethrough")).size(11.0));
        });
        ui.add_space(4.0);
        egui::ComboBox::from_id_salt("design_text_case")
            .selected_text(match layer.text_style.case.as_str() {
                "upper" => crate::ui::i18n::text(ui, "design-uppercase"),
                "lower" => crate::ui::i18n::text(ui, "design-lowercase"),
                "title" => crate::ui::i18n::text(ui, "design-title-case"),
                _ => crate::ui::i18n::text(ui, "design-original"),
            })
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for (value, key) in [
                    ("original", "design-original"),
                    ("upper", "design-uppercase"),
                    ("lower", "design-lowercase"),
                    ("title", "design-title-case"),
                ] {
                    ui.selectable_value(&mut layer.text_style.case, value.into(), crate::ui::i18n::text(ui, key));
                }
            });
        ui.add_space(2.0);
        property_slider(ui, "design-line-spacing", &mut layer.text_style.line_height, 0.5..=3.0, "×", 2, theme);
        ui.add_space(2.0);
        property_slider(ui, "design-letter-spacing", &mut layer.text_style.letter_spacing, -5.0..=20.0, " px", 1, theme);
        ui.add_space(2.0);
        property_slider(ui, "design-paragraph-spacing", &mut layer.text_style.paragraph_spacing, 0.0..=100.0, " px", 0, theme);
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.checkbox(&mut layer.text_style.wrap, RichText::new(crate::ui::i18n::text(ui, "design-wrap-text")).size(11.0));
            ui.checkbox(&mut layer.text_style.auto_fit, RichText::new(crate::ui::i18n::text(ui, "design-auto-fit")).size(11.0))
                .on_hover_text(crate::ui::i18n::text(ui, "design-automatically-reduce-the-font-size-when-text-exceeds-the-layer"));
        });
    });
}
