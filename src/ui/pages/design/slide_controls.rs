use super::controls;
use crate::core::{
    design::{DesignDocument, LayerContent},
    models::PresentationTheme,
};
use crate::ui::theme::AppTheme;
use eframe::egui::{self, RichText, Vec2};
use egui_phosphor::regular::*;

pub(super) enum Action {
    None,
    Duplicate,
    Export,
    Delete,
}

pub(super) fn render(
    ui: &mut egui::Ui,
    scene: &mut DesignDocument,
    themes: &[PresentationTheme],
    theme: &AppTheme,
) -> Action {
    controls::caption(ui, "design-design-name", theme);
    ui.add(
        egui::TextEdit::singleline(&mut scene.name)
            .desired_width(f32::INFINITY)
            .margin(egui::vec2(10.0, 8.0))
            .hint_text(crate::ui::i18n::text(ui, "design-name-this-slide")),
    );
    ui.add_space(8.0);
    controls::caption(ui, "design-in-the-presentation", theme);
    if let Some(mode) =
        controls::segments(ui, &["design-slide", "design-overlay"], usize::from(scene.overlay), theme)
    {
        scene.overlay = mode == 1;
        if scene.overlay {
            scene.background_mode = "transparent".into();
        } else if scene.background_mode == "transparent" {
            scene.background_mode = "theme".into();
            if scene.theme_id.is_none() {
                scene.theme_id = themes.first().map(|t| t.id.clone());
            }
        }
    }
    controls::note(
        ui,
        if scene.overlay {
            "design-appears-over-the-current-live-content"
        } else {
            "design-present-this-design-as-a-standalone-slide"
        },
        theme,
    );
    render_transition_controls(
        ui,
        &mut scene.transition,
        &mut scene.transition_ms,
        &mut scene.transition_strength,
        &mut scene.transition_easing,
        theme,
    );
    ui.add_space(8.0);
    controls::caption(ui, "design-background", theme);
    if scene.overlay {
        egui::Frame::new()
            .fill(theme.accent_soft)
            .corner_radius(8.0)
            .inner_margin(10)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(crate::ui::i18n::format(ui, "design-transparent", &[("icon", STACK)]))
                        .size(12.0)
                        .color(theme.accent),
                );
            });
    } else {
        let active = match scene.background_mode.as_str() {
            "theme" => 1,
            "color" => 2,
            _ => 0,
        };
        if let Some(mode) = controls::segments(ui, &["design-no-background", "design-theme", "common-color"], active, theme) {
            scene.background_mode = ["transparent", "theme", "color"][mode].into();
            if mode == 1 {
                if scene.theme_id.is_none() {
                    scene.theme_id = themes.first().map(|theme| theme.id.clone());
                }
                if let Some(selected) = themes
                    .iter()
                    .find(|theme| Some(&theme.id) == scene.theme_id.as_ref())
                {
                    for layer in &mut scene.layers {
                        if !matches!(
                            layer.content,
                            LayerContent::Image { .. } | LayerContent::Shape { .. }
                        ) {
                            layer.font_family = selected.font_family.clone();
                            layer.set_font_weight(selected.font_weight);
                            layer.color = selected.text_color.to_srgba_unmultiplied();
                        }
                    }
                }
            }
        }
        ui.add_space(4.0);
        match scene.background_mode.as_str() {
            "color" => {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(crate::ui::i18n::text(ui, "design-background-color"))
                            .size(12.0)
                            .color(theme.text_secondary),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        crate::ui::components::color_swatch_u8_4(
                            ui,
                            "scene_bg_color",
                            &mut scene.background,
                            true,
                            theme,
                        );
                    });
                });
            }
            "theme" => {
                if scene.theme_id.is_none() {
                    scene.theme_id = themes.first().map(|theme| theme.id.clone());
                }
                let selected = themes
                    .iter()
                    .find(|theme| Some(&theme.id) == scene.theme_id.as_ref());
                let (rect, response) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), 94.0),
                    egui::Sense::click(),
                );
                if let Some(selected) = selected {
                    crate::render::paint_theme_bg(ui.painter(), rect, selected, 8.0);
                    let mut sample = selected.clone();
                    sample.font_size = 160.0;
                    crate::render::render_theme_lyrics(ui.painter(), rect, &sample, "Aa");
                } else {
                    ui.painter().rect_filled(rect, 8.0, theme.bg_app);
                }
                let label = egui::Rect::from_min_max(
                    egui::pos2(rect.left(), rect.bottom() - 29.0),
                    rect.max,
                );
                ui.painter()
                    .rect_filled(label, 0.0, egui::Color32::from_black_alpha(165));
                let mut child = ui.new_child(
                    egui::UiBuilder::new().max_rect(label.shrink2(egui::vec2(9.0, 4.0))),
                );
                child.horizontal(|ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(
                                selected.map_or(
                                    crate::ui::i18n::text(ui, "design-choose-theme"),
                                    |theme| crate::ui::i18n::text(ui, &theme.name),
                                ),
                            )
                            .size(11.0)
                            .color(egui::Color32::WHITE),
                        )
                        .truncate(),
                    );
                });
                ui.painter().rect_stroke(
                    rect,
                    8.0,
                    egui::Stroke::new(1.0, theme.border_default),
                    egui::StrokeKind::Inside,
                );
                response
                    .clone()
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text(crate::ui::i18n::text(ui, "design-click-to-choose-another-theme"));
                egui::Popup::menu(&response).show(|ui| {
                    ui.set_width(210.0);
                    egui::ScrollArea::vertical()
                        .max_height(270.0)
                        .show(ui, |ui| {
                            for item in themes {
                                if ui
                                    .selectable_label(
                                        Some(&item.id) == scene.theme_id.as_ref(),
                                        &item.name,
                                    )
                                    .clicked()
                                {
                                    scene.theme_id = Some(item.id.clone());
                                    for layer in &mut scene.layers {
                                        if !matches!(
                                            layer.content,
                                            LayerContent::Image { .. } | LayerContent::Shape { .. }
                                        ) {
                                            layer.font_family = item.font_family.clone();
                                            layer.set_font_weight(item.font_weight);
                                            layer.color = item.text_color.to_srgba_unmultiplied();
                                        }
                                    }
                                    ui.close();
                                }
                            }
                        });
                });
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new(crate::ui::i18n::format(ui, "design-apply-theme-font-and-color", &[("icon", TEXT_T)]))
                                .size(11.0)
                                .color(theme.accent),
                        )
                        .frame(false),
                    )
                    .clicked()
                    && let Some(selected) = themes
                        .iter()
                        .find(|theme| Some(&theme.id) == scene.theme_id.as_ref())
                {
                    for layer in &mut scene.layers {
                        if !matches!(
                            layer.content,
                            LayerContent::Image { .. } | LayerContent::Shape { .. }
                        ) {
                            layer.font_family = selected.font_family.clone();
                            layer.set_font_weight(selected.font_weight);
                            layer.color = selected.text_color.to_srgba_unmultiplied();
                        }
                    }
                }
            }
            _ => controls::note(ui, "design-preserves-transparency-between-layers", theme),
        }
    }
    ui.add_space(12.0);
    ui.separator();
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new(ARROWS_OUT).size(13.0).color(theme.icon_muted));
        ui.label(
            RichText::new(crate::ui::i18n::text(ui, "design-responsive-screen-layout"))
                .size(10.5)
                .color(theme.text_muted),
        );
    });
    let mut action = Action::None;
    ui.menu_button(
        RichText::new(crate::ui::i18n::format(ui, "design-more-options", &[("icon", DOTS_THREE)]))
            .size(11.0)
            .color(theme.text_secondary),
        |ui| {
            if ui.button(crate::ui::i18n::format(ui, "design-duplicate-design", &[("icon", COPY)])).clicked() {
                action = Action::Duplicate;
                ui.close();
            }
            if ui
                .button(crate::ui::i18n::format(ui, "design-export-layers-as-png", &[("icon", DOWNLOAD_SIMPLE)]))
                .clicked()
            {
                action = Action::Export;
                ui.close();
            }
            ui.separator();
            if ui
                .button(
                    RichText::new(crate::ui::i18n::format(ui, "design-delete-design-label", &[("icon", TRASH)])).color(theme.status_danger),
                )
                .clicked()
            {
                action = Action::Delete;
                ui.close();
            }
        },
    );
    action
}

pub(crate) fn render_transition_controls(
    ui: &mut egui::Ui,
    transition: &mut String,
    transition_ms: &mut u32,
    transition_strength: &mut f32,
    transition_easing: &mut String,
    theme: &AppTheme,
) {
    controls::caption(ui, "design-live-transition", theme);
    ui.horizontal(|ui| {
        ui.label(RichText::new(crate::ui::i18n::text(ui, "design-effect")).size(12.0).color(theme.text_secondary));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            egui::ComboBox::from_id_salt("design_transition_effect")
                .selected_text(match transition.as_str() {
                    "slide_left" | "slide" => crate::ui::i18n::text(ui, "design-slide-left"),
                    "slide_right" => crate::ui::i18n::text(ui, "design-slide-right"),
                    "slide_up" => crate::ui::i18n::text(ui, "design-slide-up"),
                    "slide_down" => crate::ui::i18n::text(ui, "design-slide-down"),
                    "zoom" => crate::ui::i18n::text(ui, "design-zoom-in"),
                    "zoom_out" => crate::ui::i18n::text(ui, "design-zoom-out"),
                    "wipe" => crate::ui::i18n::text(ui, "design-wipe"),
                    "cut" => crate::ui::i18n::text(ui, "design-cut"),
                    _ => crate::ui::i18n::text(ui, "design-dissolve-fade"),
                })
                .width((ui.available_width() - 4.0).clamp(110.0, 180.0))
                .show_ui(ui, |ui| {
                    ui.selectable_value(transition, "fade".into(), crate::ui::i18n::text(ui, "design-dissolve-fade"));
                    ui.selectable_value(transition, "slide_left".into(), crate::ui::i18n::text(ui, "design-slide-left"));
                    ui.selectable_value(transition, "slide_right".into(), crate::ui::i18n::text(ui, "design-slide-right"));
                    ui.selectable_value(transition, "slide_up".into(), crate::ui::i18n::text(ui, "design-slide-up"));
                    ui.selectable_value(transition, "slide_down".into(), crate::ui::i18n::text(ui, "design-slide-down"));
                    ui.selectable_value(transition, "zoom".into(), crate::ui::i18n::text(ui, "design-zoom-in"));
                    ui.selectable_value(transition, "zoom_out".into(), crate::ui::i18n::text(ui, "design-zoom-out"));
                    ui.selectable_value(transition, "wipe".into(), crate::ui::i18n::text(ui, "design-wipe"));
                    ui.selectable_value(transition, "cut".into(), crate::ui::i18n::text(ui, "design-cut"));
                });
        });
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new(crate::ui::i18n::text(ui, "design-duration-ms")).size(12.0).color(theme.text_secondary));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(
                egui::DragValue::new(transition_ms)
                    .range(0..=5000)
                    .speed(10)
                    .suffix(" ms"),
            );
        });
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new(crate::ui::i18n::text(ui, "design-intensity")).size(12.0).color(theme.text_secondary));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(
                egui::DragValue::new(transition_strength)
                    .range(0.0..=100.0)
                    .speed(1.0)
                    .suffix(""),
            );
        });
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new(crate::ui::i18n::text(ui, "design-easing")).size(12.0).color(theme.text_secondary));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            egui::ComboBox::from_id_salt("design_transition_easing")
                .selected_text(crate::ui::i18n::easing(ui, transition_easing))
                .width((ui.available_width() - 4.0).clamp(80.0, 160.0))
                .show_ui(ui, |ui| {
                    for easing in ["smooth", "linear", "ease_in", "ease_out", "ease_in_out"] {
                        ui.selectable_value(transition_easing, easing.into(), crate::ui::i18n::easing(ui, easing));
                    }
                });
        });
    });
    controls::note(
        ui,
        "design-live-transition-when-switching-this-dynamic-slide-scoreboard-notices-or",
        theme,
    );
}
