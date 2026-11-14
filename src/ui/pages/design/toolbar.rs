use super::{aspect_ratio, theme_preview, DesignEditor};
use crate::{
    core::state::ControlPanelState,
    ui::theme::AppTheme,
};
use eframe::egui::{self, Stroke, Vec2};
use egui_phosphor::regular::*;

pub(super) fn canvas_toolbar(
    editor: &mut DesignEditor,
    ui: &mut egui::Ui,
    theme: &AppTheme,
    state: &mut ControlPanelState,
) {
    let compact = ui.available_width() < 520.0;
    let can_undo = editor.scene.is_some() && !editor.undo.is_empty();
    let can_redo = editor.scene.is_some() && !editor.redo.is_empty();
    let can_snap = editor.scene.is_some();

    crate::ui::components::centered_horizontal(ui, "design_canvas_toolbar", 420.0, |ui| {
        ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);

        if toolbar_pill_button(
            ui,
            ARROW_U_UP_LEFT,
            None,
            "design-undo-ctrl-z",
            false,
            can_undo,
            theme,
        )
        .clicked()
        {
            editor.history(false);
        }

        if toolbar_pill_button(
            ui,
            ARROW_U_UP_RIGHT,
            None,
            "design-redo-ctrl-shift-z",
            false,
            can_redo,
            theme,
        )
        .clicked()
        {
            editor.history(true);
        }

        ui.add_space(2.0);

        let snap_label = if compact { None } else { Some(crate::ui::i18n::text(ui, "design-guides-enabled")) };
        if toolbar_pill_button(
            ui,
            MAGNET,
            snap_label,
            "design-snap-to-the-center-screen-edges-and-other-layers",
            !editor.disable_snap,
            can_snap,
            theme,
        )
        .clicked()
        {
            editor.disable_snap = !editor.disable_snap;
        }

        ui.add_space(2.0);

        let popup_id = ui.make_persistent_id("design_theme_preview_dropdown");
        let is_open = egui::Popup::is_id_open(ui.ctx(), popup_id);
        let has_preview = editor.preview_theme_id.is_some();
        let preview_active = has_preview || is_open;
        let preview_label = if compact {
            None
        } else if let Some(preview_id) = &editor.preview_theme_id {
            let name = state
                .themes
                .iter()
                .find(|t| &t.id == preview_id)
                .map(|t| t.name.as_str())
                .unwrap_or_else(|| crate::ui::i18n::text(ui, "design-theme"));
            Some(format!("{}: {name} {CARET_DOWN}", crate::ui::i18n::text(ui, "design-theme")))
        } else {
            Some(format!("{} {CARET_DOWN}", crate::ui::i18n::text(ui, "design-preview-theme")))
        };

        let resp = toolbar_pill_button(
            ui,
            EYE,
            preview_label.as_deref(),
            "design-preview-theme-tooltip",
            preview_active,
            true,
            theme,
        );

        theme_preview::show_dropdown(
            &resp,
            popup_id,
            &mut editor.preview_theme_id,
            &mut editor.theme_search,
            &state.themes,
            theme,
        );

        ui.add_space(2.0);

        let aspect_popup_id = ui.make_persistent_id("design_aspect_ratio_dropdown");
        let aspect_is_open = egui::Popup::is_id_open(ui.ctx(), aspect_popup_id);
        let has_custom_aspect = editor.preview_aspect_ratio.is_some();
        let aspect_active = has_custom_aspect || aspect_is_open;
        let current_aspect = editor
            .preview_aspect_ratio
            .unwrap_or(state.presentation_aspect_ratio);
        let ratio_name = aspect_ratio::aspect_ratio_name(current_aspect);
        let aspect_label = if compact {
            None
        } else if has_custom_aspect {
            Some(format!("{}: {ratio_name} {CARET_DOWN}", crate::ui::i18n::text(ui, "design-aspect-ratio")))
        } else {
            Some(format!("{} ({ratio_name}) {CARET_DOWN}", crate::ui::i18n::text(ui, "design-aspect-ratio")))
        };

        let aspect_resp = toolbar_pill_button(
            ui,
            RECTANGLE,
            aspect_label.as_deref(),
            "design-aspect-ratio-tooltip",
            aspect_active,
            true,
            theme,
        );

        aspect_ratio::show_dropdown(
            &aspect_resp,
            aspect_popup_id,
            &mut editor.preview_aspect_ratio,
            state,
            theme,
        );
    });
}

pub(super) fn toolbar_pill_button(
    ui: &mut egui::Ui,
    icon: &str,
    label: Option<&str>,
    tooltip: &str,
    active: bool,
    enabled: bool,
    theme: &AppTheme,
) -> egui::Response {
    let font = if label.is_some() {
        egui::FontId::proportional(12.0)
    } else {
        egui::FontId::proportional(15.0)
    };
    let text = match label {
        Some(lbl) => format!("{icon} {lbl}"),
        None => icon.to_string(),
    };
    let galley = ui
        .painter()
        .layout_no_wrap(text.clone(), font.clone(), theme.text_primary);
    let min_w = if label.is_some() {
        (galley.size().x + 18.0).max(50.0)
    } else {
        32.0
    };
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(min_w, 28.0),
        if enabled {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        },
    );

    let hover = if enabled {
        ui.ctx().animate_bool(resp.id.with("h"), resp.hovered())
    } else {
        0.0
    };
    let down = if enabled {
        ui.ctx()
            .animate_bool(resp.id.with("d"), resp.is_pointer_button_down_on())
    } else {
        0.0
    };
    let selected = ui.ctx().animate_bool(resp.id.with("s"), active);

    let bg = if !enabled {
        theme.bg_pill.linear_multiply(0.4)
    } else if active {
        if down > 0.05 {
            theme
                .accent_soft
                .lerp_to_gamma(theme.bg_card_active, down * 0.35)
        } else {
            theme.bg_pill.lerp_to_gamma(theme.accent_soft, selected)
        }
    } else if down > 0.05 {
        theme
            .bg_card_hover
            .lerp_to_gamma(theme.accent_soft, down * 0.45)
    } else if hover > 0.05 {
        theme.bg_pill.lerp_to_gamma(theme.bg_card_hover, hover)
    } else {
        theme.bg_pill
    };

    let stroke = if !enabled {
        Stroke::new(1.0, theme.border_subtle.linear_multiply(0.3))
    } else if active {
        if down > 0.05 {
            Stroke::new(1.3, theme.accent)
        } else {
            Stroke::new(1.0, theme.accent.linear_multiply(0.65))
        }
    } else if down > 0.05 {
        Stroke::new(1.2, theme.accent.linear_multiply(0.85))
    } else if hover > 0.05 {
        Stroke::new(
            1.0,
            theme
                .border_default
                .lerp_to_gamma(theme.accent.linear_multiply(0.5), hover),
        )
    } else {
        Stroke::new(1.0, theme.border_default)
    };

    let draw_rect = rect.shrink(down * 0.75);
    ui.painter().rect(
        draw_rect,
        theme.radius_sm,
        bg,
        stroke,
        egui::StrokeKind::Inside,
    );

    let text_color = if !enabled {
        theme.text_muted.linear_multiply(0.45)
    } else if active || down > 0.05 {
        theme.accent
    } else if hover > 0.5 {
        theme.text_primary
    } else {
        theme.icon_default
    };

    let galley = ui.painter().layout_no_wrap(text, font, text_color);
    ui.painter().galley(
        egui::pos2(
            draw_rect.center().x - galley.size().x / 2.0,
            draw_rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        text_color,
    );

    let resp = if enabled {
        resp.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        resp
    };
    resp.on_hover_text(crate::ui::i18n::text(ui, tooltip))
}
