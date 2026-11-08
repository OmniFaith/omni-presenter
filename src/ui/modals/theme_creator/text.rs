use super::widgets::{
    color_row_rgb, color_row_srgba, property_slider, property_slider_pct, section_title,
    toggle_switch_row,
};
use crate::core::state::ControlPanelState;
use eframe::egui::{self, Color32, RichText, Vec2};
use egui_phosphor::regular::*;

const FONT_WEIGHT_OPTIONS: &[(u16, &str)] = &[
    (100, "settings-100-thin"),
    (200, "settings-200-extra-light"),
    (300, "settings-300-light"),
    (400, "settings-400-regular"),
    (500, "settings-500-medium"),
    (600, "settings-600-semi-bold"),
    (700, "settings-700-bold"),
    (800, "settings-800-extra-bold"),
    (900, "settings-900-black"),
];

pub(super) fn text_tab(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    _sidebar_width: f32,
) -> bool {
    let mut push_snapshot = false;

    // --- 1. APARÊNCIA ---
    section_title(ui, "design-appearance", &state.theme);

    let mut rgb = [
        state.theme_editor.editing.text_color.r() as f32 / 255.0,
        state.theme_editor.editing.text_color.g() as f32 / 255.0,
        state.theme_editor.editing.text_color.b() as f32 / 255.0,
    ];
    if color_row_rgb(ui, "common-color", &mut rgb, &state.theme) {
        state.theme_editor.editing.text_color = Color32::from_rgb(
            (rgb[0] * 255.0) as u8,
            (rgb[1] * 255.0) as u8,
            (rgb[2] * 255.0) as u8,
        );
        push_snapshot = true;
    }
    ui.add_space(2.0);

    let mut emphasis = state.theme_editor.editing.options.emphasis_color;
    let mut emph_rgb = [
        emphasis[0] as f32 / 255.0,
        emphasis[1] as f32 / 255.0,
        emphasis[2] as f32 / 255.0,
    ];
    if color_row_rgb(ui, "settings-emphasis", &mut emph_rgb, &state.theme) {
        emphasis = [
            (emph_rgb[0] * 255.0) as u8,
            (emph_rgb[1] * 255.0) as u8,
            (emph_rgb[2] * 255.0) as u8,
        ];
        state.theme_editor.editing.options.emphasis_color = emphasis;
        push_snapshot = true;
    }
    ui.add_space(2.0);

    if property_slider_pct(
        ui,
        "design-opacity",
        &mut state.theme_editor.editing.options.text_opacity,
        0.0,
        100.0,
        &state.theme,
    ) {
        push_snapshot = true;
    }

    // --- 2. EFEITOS ---
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(crate::ui::i18n::text(ui, "design-effects"))
                .size(11.5)
                .color(state.theme.text_secondary),
        );
        let has_effects = state.theme_editor.editing.options.text_rotation != 0.0
            || state.theme_editor.editing.options.text_glow != 0.0;
        if has_effects {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button(RichText::new(crate::ui::i18n::text(ui, "design-reset")).size(10.5).color(state.theme.text_muted))
                    .on_hover_text(crate::ui::i18n::text(ui, "settings-reset-rotation-and-brightness"))
                    .clicked()
                {
                    state.theme_editor.editing.options.text_rotation = 0.0;
                    state.theme_editor.editing.options.text_glow = 0.0;
                    push_snapshot = true;
                }
            });
        }
    });
    ui.add_space(2.0);

    if property_slider(
        ui,
        "design-rotation",
        &mut state.theme_editor.editing.options.text_rotation,
        -30.0..=30.0,
        "°",
        0,
        0.5,
        &state.theme,
    ) {
        push_snapshot = true;
    }
    ui.add_space(2.0);

    if property_slider(
        ui,
        "settings-brightness",
        &mut state.theme_editor.editing.options.text_glow,
        0.0..=30.0,
        "",
        1,
        0.5,
        &state.theme,
    ) {
        push_snapshot = true;
    }
    ui.add_space(2.0);

    if toggle_switch_row(
        ui,
        "design-show-song-effects",
        &mut state.theme_editor.editing.options.show_song_effects,
        &state.theme,
    ) {
        push_snapshot = true;
    }

    // --- 3. CONTORNO E SOMBRA ---
    ui.add_space(6.0);
    section_title(ui, "design-outline-and-shadow", &state.theme);

    if ui
        .checkbox(
            &mut state.theme_editor.editing.outline_enabled,
            RichText::new(crate::ui::i18n::text(ui, "design-outline")).size(11.5),
        )
        .changed()
    {
        push_snapshot = true;
    }

    if state.theme_editor.editing.outline_enabled {
        ui.indent("theme_outline_indent", |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            if property_slider(
                ui,
                "design-thickness",
                &mut state.theme_editor.editing.outline_width,
                0.5..=12.0,
                " px",
                1,
                0.1,
                &state.theme,
            ) {
                push_snapshot = true;
            }
            let mut out_rgb = [
                state.theme_editor.editing.outline_color.r() as f32 / 255.0,
                state.theme_editor.editing.outline_color.g() as f32 / 255.0,
                state.theme_editor.editing.outline_color.b() as f32 / 255.0,
            ];
            if color_row_rgb(ui, "common-color", &mut out_rgb, &state.theme) {
                state.theme_editor.editing.outline_color = Color32::from_rgb(
                    (out_rgb[0] * 255.0) as u8,
                    (out_rgb[1] * 255.0) as u8,
                    (out_rgb[2] * 255.0) as u8,
                );
                push_snapshot = true;
            }
        });
        ui.add_space(2.0);
    }

    if ui
        .checkbox(
            &mut state.theme_editor.editing.shadow_enabled,
            RichText::new(crate::ui::i18n::text(ui, "design-shadow")).size(11.5),
        )
        .changed()
    {
        push_snapshot = true;
    }

    if state.theme_editor.editing.shadow_enabled {
        ui.indent("theme_shadow_indent", |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            if property_slider(
                ui,
                "design-blur",
                &mut state.theme_editor.editing.shadow_blur,
                0.0..=24.0,
                "",
                1,
                0.2,
                &state.theme,
            ) {
                push_snapshot = true;
            }
            if color_row_srgba(
                ui,
                "common-color",
                &mut state.theme_editor.editing.shadow_color,
                &state.theme,
            ) {
                push_snapshot = true;
            }
            ui.horizontal(|ui| {
                ui.add_sized(
                    [68.0, 20.0],
                    egui::Label::new(
                        RichText::new(crate::ui::i18n::text(ui, "design-position"))
                            .size(11.0)
                            .color(state.theme.text_muted),
                    ),
                );
                let rem = ui.available_width();
                let drag_w = ((rem - 6.0) * 0.5).max(40.0);
                if ui
                    .add_sized(
                        [drag_w, 20.0],
                        egui::DragValue::new(&mut state.theme_editor.editing.shadow_offset_x)
                            .speed(0.2)
                            .prefix("X: "),
                    )
                    .changed()
                {
                    push_snapshot = true;
                }
                if ui
                    .add_sized(
                        [drag_w, 20.0],
                        egui::DragValue::new(&mut state.theme_editor.editing.shadow_offset_y)
                            .speed(0.2)
                            .prefix("Y: "),
                    )
                    .changed()
                {
                    push_snapshot = true;
                }
            });
        });
    }

    // --- 4. TEXTO ---
    ui.add_space(6.0);
    section_title(ui, "common-text", &state.theme);

    if property_slider(
        ui,
        "design-size",
        &mut state.theme_editor.editing.font_size,
        10.0..=160.0,
        " px",
        0,
        1.0,
        &state.theme,
    ) {
        push_snapshot = true;
    }
    ui.add_space(4.0);

    if crate::ui::components::font_picker(
        ui,
        "theme_font_picker",
        &mut state.theme_editor.editing.font_family,
        ui.available_width(),
        &state.theme,
    ) {
        push_snapshot = true;
    }
    ui.add_space(3.0);

    ui.horizontal(|ui| {
        ui.add_sized(
            [68.0, 20.0],
            egui::Label::new(
                RichText::new(crate::ui::i18n::text(ui, "font-weight-label"))
                    .size(11.5)
                    .color(state.theme.text_secondary),
            ),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let available = crate::core::models::fonts::available_font_weights(&state.theme_editor.editing.font_family);
            if !available.is_empty() && !available.contains(&state.theme_editor.editing.font_weight)
                && let Some(&closest) = available
                    .iter()
                    .min_by_key(|&&w| (w as i32 - state.theme_editor.editing.font_weight as i32).abs())
                {
                    state.theme_editor.editing.font_weight = closest;
                    state.theme_editor.editing.font_bold = closest >= 600;
                    push_snapshot = true;
                }
            let cur_weight_label = FONT_WEIGHT_OPTIONS
                .iter()
                .find(|(w, _)| *w == state.theme_editor.editing.font_weight)
                .map(|(_, l)| *l)
                .unwrap_or("settings-700-bold");
            egui::ComboBox::from_id_salt("theme_font_weight_combo")
                .selected_text(crate::ui::i18n::text(ui, cur_weight_label))
                .width((ui.available_width() - 4.0).clamp(100.0, 180.0))
                .show_ui(ui, |ui| {
                    for (w, label) in FONT_WEIGHT_OPTIONS {
                        if available.contains(w)
                            && ui
                                .selectable_value(
                                    &mut state.theme_editor.editing.font_weight,
                                    *w,
                                    crate::ui::i18n::text(ui, label),
                                )
                                .clicked()
                            {
                                state.theme_editor.editing.font_bold = *w >= 600;
                                push_snapshot = true;
                            }
                    }
                });
        });
    });
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;

        let style_btn =
            |ui: &mut egui::Ui, active: bool, text: RichText, tooltip: &str| -> bool {
                ui.add(
                    egui::Button::new(text)
                        .min_size(Vec2::new(26.0, 24.0))
                        .selected(active),
                )
                .on_hover_text(crate::ui::i18n::text(ui, tooltip))
                .clicked()
            };

        if style_btn(
            ui,
            state.theme_editor.editing.font_bold,
            RichText::new("B").strong(),
            "font-bold",
        ) {
            state.theme_editor.editing.font_bold = !state.theme_editor.editing.font_bold;
            state.theme_editor.editing.font_weight = if state.theme_editor.editing.font_bold {
                700
            } else {
                400
            };
            push_snapshot = true;
        }
        if style_btn(
            ui,
            state.theme_editor.editing.font_italic,
            RichText::new("I").italics(),
            "font-italic",
        ) {
            state.theme_editor.editing.font_italic = !state.theme_editor.editing.font_italic;
            push_snapshot = true;
        }
        if style_btn(
            ui,
            state.theme_editor.editing.font_underline,
            RichText::new("U").underline(),
            "font-underline",
        ) {
            state.theme_editor.editing.font_underline = !state.theme_editor.editing.font_underline;
            push_snapshot = true;
        }
        if style_btn(
            ui,
            state.theme_editor.editing.font_strikethrough,
            RichText::new("S").strikethrough(),
            "settings-strikethrough",
        ) {
            state.theme_editor.editing.font_strikethrough =
                !state.theme_editor.editing.font_strikethrough;
            push_snapshot = true;
        }

        ui.add_space(3.0);
        ui.separator();
        ui.add_space(3.0);

        for (val, icon, tip) in [
            ("left", TEXT_ALIGN_LEFT, "workspace-align-left"),
            ("center", TEXT_ALIGN_CENTER, "workspace-center"),
            ("right", TEXT_ALIGN_RIGHT, "workspace-align-right"),
            ("justify", TEXT_ALIGN_JUSTIFY, "workspace-justify"),
        ] {
            let active = state.theme_editor.editing.align_h == val;
            if ui
                .add(
                    egui::Button::new(RichText::new(icon).size(14.0))
                        .min_size(Vec2::new(26.0, 24.0))
                        .selected(active),
                )
                .on_hover_text(crate::ui::i18n::text(ui, tip))
                .clicked()
            {
                state.theme_editor.editing.align_h = val.into();
                push_snapshot = true;
            }
        }
    });

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_sized(
            [68.0, 20.0],
            egui::Label::new(
                RichText::new(crate::ui::i18n::text(ui, "workspace-vertical"))
                    .size(11.5)
                    .color(state.theme.text_secondary),
            ),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            egui::ComboBox::from_id_salt("theme_text_vertical")
                .selected_text(match state.theme_editor.editing.align_v.as_str() {
                    "top" => crate::ui::i18n::text(ui, "settings-top"),
                    "bottom" => crate::ui::i18n::text(ui, "settings-bottom"),
                    _ => crate::ui::i18n::text(ui, "design-center"),
                })
                .width((ui.available_width() - 4.0).clamp(80.0, 140.0))
                .show_ui(ui, |ui| {
                    for (value, label) in [
                        ("top", "settings-top"),
                        ("center", "design-center"),
                        ("bottom", "settings-bottom"),
                    ] {
                        if ui
                            .selectable_value(
                                &mut state.theme_editor.editing.align_v,
                                value.into(),
                                crate::ui::i18n::text(ui, label),
                            )
                            .changed()
                        {
                            push_snapshot = true;
                        }
                    }
                });
        });
    });

    // --- 5. MAIS OPÇÕES DE TEXTO ---
    ui.add_space(4.0);
    egui::CollapsingHeader::new(
        RichText::new(crate::ui::i18n::text(ui, "settings-more-text-options"))
            .size(11.5)
            .color(state.theme.text_secondary),
    )
    .id_salt("theme_text_advanced")
    .show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 3.0;

        if property_slider(
            ui,
            "design-line",
            &mut state.theme_editor.editing.line_spacing,
            0.65..=3.0,
            "x",
            2,
            0.02,
            &state.theme,
        ) {
            push_snapshot = true;
        }

        if property_slider(
            ui,
            "settings-spacing",
            &mut state.theme_editor.editing.letter_spacing,
            -10.0..=30.0,
            " px",
            1,
            0.2,
            &state.theme,
        ) {
            push_snapshot = true;
        }

        if property_slider(
            ui,
            "settings-paragraphs",
            &mut state.theme_editor.editing.options.paragraph_spacing,
            0.0..=60.0,
            " px",
            1,
            0.5,
            &state.theme,
        ) {
            push_snapshot = true;
        }

        ui.horizontal(|ui| {
            ui.add_sized(
                [68.0, 20.0],
                egui::Label::new(
                    RichText::new(crate::ui::i18n::text(ui, "settings-text-box"))
                        .size(11.5)
                        .color(state.theme.text_secondary),
                ),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("theme_text_case")
                    .selected_text(match state.theme_editor.editing.text_case.as_str() {
                        "uppercase" => crate::ui::i18n::text(ui, "design-uppercase"),
                        "lowercase" => crate::ui::i18n::text(ui, "design-lowercase"),
                        "titlecase" => crate::ui::i18n::text(ui, "control-title-slide"),
                        _ => crate::ui::i18n::text(ui, "design-original"),
                    })
                    .width((ui.available_width() - 4.0).clamp(80.0, 140.0))
                    .show_ui(ui, |ui| {
                        for (value, label) in [
                            ("original", "design-original"),
                            ("uppercase", "design-uppercase"),
                            ("lowercase", "design-lowercase"),
                            ("titlecase", "control-title-slide"),
                        ] {
                            if ui
                                .selectable_value(
                                    &mut state.theme_editor.editing.text_case,
                                    value.into(),
                                    crate::ui::i18n::text(ui, label),
                                )
                                .changed()
                            {
                                push_snapshot = true;
                            }
                        }
                    });
            });
        });

        ui.horizontal_wrapped(|ui| {
            if ui
                .checkbox(
                    &mut state.theme_editor.editing.options.wrap_text,
                    RichText::new(crate::ui::i18n::text(ui, "settings-wrap-lines")).size(11.0),
                )
                .changed()
            {
                push_snapshot = true;
            }
            if ui
                .checkbox(
                    &mut state.theme_editor.editing.options.auto_fit,
                    RichText::new(crate::ui::i18n::text(ui, "design-auto-fit")).size(11.0),
                )
                .on_hover_text(crate::ui::i18n::text(ui, "settings-automatically-reduce-the-font-size-when-text-exceeds-the-box"))
                .changed()
            {
                push_snapshot = true;
            }
        });
    });

    push_snapshot
}
