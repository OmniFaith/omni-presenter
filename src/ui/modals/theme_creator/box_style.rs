use super::widgets::{color_row_srgba, geo_field, property_slider, section_title};
use crate::core::state::ControlPanelState;
use eframe::egui::{self, RichText, Stroke};
use egui_phosphor::regular::*;

pub(super) fn box_tab(ui: &mut egui::Ui, state: &mut ControlPanelState) -> bool {
    let mut push_snapshot = false;

    section_title(ui, "design-position-and-size", &state.theme);

    let cell_gap = 6.0;
    let cell_w = ((ui.available_width() - cell_gap) * 0.5).max(60.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = cell_gap;
        if geo_field(
            ui,
            "X",
            &mut state.theme_editor.editing.box_x,
            false,
            cell_w,
            &state.theme,
        ) {
            push_snapshot = true;
        }
        if geo_field(
            ui,
            "Y",
            &mut state.theme_editor.editing.box_y,
            false,
            cell_w,
            &state.theme,
        ) {
            push_snapshot = true;
        }
    });

    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = cell_gap;
        if geo_field(
            ui,
            "common-geo-width",
            &mut state.theme_editor.editing.box_w,
            true,
            cell_w,
            &state.theme,
        ) {
            push_snapshot = true;
        }
        if geo_field(
            ui,
            "common-geo-height",
            &mut state.theme_editor.editing.box_h,
            true,
            cell_w,
            &state.theme,
        ) {
            push_snapshot = true;
        }
    });

    state.theme_editor.editing.box_x = state
        .theme_editor
        .editing
        .box_x
        .min(1.0 - state.theme_editor.editing.box_w);
    state.theme_editor.editing.box_y = state
        .theme_editor
        .editing
        .box_y
        .min(1.0 - state.theme_editor.editing.box_h);

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = cell_gap;
        let btn_w = ((ui.available_width() - cell_gap) * 0.5).max(50.0);
        if ui
            .add_sized(
                [btn_w, 24.0],
                egui::Button::new(RichText::new(crate::ui::i18n::text(ui, "design-center-horizontally")).size(11.0)),
            )
            .clicked()
        {
            state.theme_editor.editing.box_x =
                ((1.0 - state.theme_editor.editing.box_w) / 2.0).clamp(0.0, 1.0);
            state.theme_editor.is_text_selected = true;
            push_snapshot = true;
        }
        if ui
            .add_sized(
                [btn_w, 24.0],
                egui::Button::new(RichText::new(crate::ui::i18n::text(ui, "design-center-vertically")).size(11.0)),
            )
            .clicked()
        {
            state.theme_editor.editing.box_y =
                ((1.0 - state.theme_editor.editing.box_h) / 2.0).clamp(0.0, 1.0);
            state.theme_editor.is_text_selected = true;
            push_snapshot = true;
        }
    });

    ui.add_space(6.0);
    section_title(ui, "settings-text-box-templates", &state.theme);

    let preset_btn = |ui: &mut egui::Ui,
                      icon: &str,
                      text: &str,
                      tooltip: &str,
                      btn_w: f32,
                      theme: &crate::ui::theme::AppTheme|
     -> egui::Response {
        ui.add_sized(
            [btn_w, 26.0],
            egui::Button::new(
                RichText::new(format!("{icon} {}", crate::ui::i18n::text(ui, text)))
                    .size(11.0)
                    .color(theme.text_primary),
            )
            .fill(theme.bg_pill)
            .stroke(Stroke::new(1.0, theme.border_default))
            .corner_radius(6.0),
        )
        .on_hover_text(crate::ui::i18n::text(ui, tooltip))
    };

    let p_btn_w = ((ui.available_width() - cell_gap) * 0.5).max(50.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = cell_gap;
        if preset_btn(
            ui,
            DESKTOP,
            "settings-full-screen",
            "settings-fill-the-screen-with-safe-margins",
            p_btn_w,
            &state.theme,
        )
        .clicked()
        {
            state.theme_editor.editing.box_x = 0.08;
            state.theme_editor.editing.box_y = 0.12;
            state.theme_editor.editing.box_w = 0.84;
            state.theme_editor.editing.box_h = 0.76;
            state.theme_editor.is_text_selected = true;
            push_snapshot = true;
        }
        if preset_btn(
            ui,
            TELEVISION,
            "settings-lower-third",
            "settings-ideal-for-captions-obs-and-streaming",
            p_btn_w,
            &state.theme,
        )
        .clicked()
        {
            state.theme_editor.editing.box_x = 0.08;
            state.theme_editor.editing.box_y = 0.68;
            state.theme_editor.editing.box_w = 0.84;
            state.theme_editor.editing.box_h = 0.26;
            state.theme_editor.is_text_selected = true;
            push_snapshot = true;
        }
    });
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = cell_gap;
        if preset_btn(
            ui,
            ARROWS_IN,
            "settings-centered",
            "settings-centered-text-in-a-medium-box",
            p_btn_w,
            &state.theme,
        )
        .clicked()
        {
            state.theme_editor.editing.box_x = 0.14;
            state.theme_editor.editing.box_y = 0.22;
            state.theme_editor.editing.box_w = 0.72;
            state.theme_editor.editing.box_h = 0.56;
            state.theme_editor.is_text_selected = true;
            push_snapshot = true;
        }
        if preset_btn(
            ui,
            ARROW_LINE_UP,
            "settings-upper-banner",
            "settings-text-in-the-upper-third",
            p_btn_w,
            &state.theme,
        )
        .clicked()
        {
            state.theme_editor.editing.box_x = 0.08;
            state.theme_editor.editing.box_y = 0.06;
            state.theme_editor.editing.box_w = 0.84;
            state.theme_editor.editing.box_h = 0.28;
            push_snapshot = true;
        }
    });

    ui.add_space(6.0);
    section_title(ui, "design-appearance", &state.theme);

    if ui
        .checkbox(
            &mut state.theme_editor.editing.box_bg_enabled,
            RichText::new(crate::ui::i18n::text(ui, "settings-box-background")).size(11.5),
        )
        .changed()
    {
        push_snapshot = true;
    }

    if state.theme_editor.editing.box_bg_enabled {
        ui.indent("box_bg_indent", |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            if color_row_srgba(
                ui,
                "common-color",
                &mut state.theme_editor.editing.box_bg_color,
                &state.theme,
            ) {
                push_snapshot = true;
            }
            if property_slider(
                ui,
                "design-corners",
                &mut state.theme_editor.editing.box_radius,
                0.0..=60.0,
                " px",
                0,
                0.5,
                &state.theme,
            ) {
                push_snapshot = true;
            }
        });
        ui.add_space(2.0);
    }

    ui.add_space(4.0);
    section_title(ui, "settings-outline-and-border", &state.theme);

    if ui
        .checkbox(
            &mut state.theme_editor.editing.box_border_enabled,
            RichText::new(crate::ui::i18n::text(ui, "settings-box-border")).size(11.5),
        )
        .changed()
    {
        push_snapshot = true;
    }

    if state.theme_editor.editing.box_border_enabled {
        ui.indent("box_border_indent", |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            if property_slider(
                ui,
                "design-thickness",
                &mut state.theme_editor.editing.box_border_width,
                0.5..=20.0,
                " px",
                1,
                0.2,
                &state.theme,
            ) {
                push_snapshot = true;
            }
            if color_row_srgba(
                ui,
                "common-color",
                &mut state.theme_editor.editing.box_border_color,
                &state.theme,
            ) {
                push_snapshot = true;
            }
        });
    }

    push_snapshot
}
