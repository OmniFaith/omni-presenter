use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, RichText, Stroke, StrokeKind, Vec2};


#[allow(dead_code)]
pub(super) fn toggle_icon_button(
    ui: &mut egui::Ui,
    icon: &str,
    is_active: bool,
    tooltip: &str,
    btn_size: Vec2,
    theme: &AppTheme,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(btn_size, egui::Sense::click());

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let bg_color = if is_active {
        theme.accent
    } else if resp.hovered() {
        theme.bg_card_hover
    } else {
        theme.bg_pill
    };

    let border_color = if is_active {
        theme.accent
    } else if resp.hovered() {
        theme.border_active
    } else {
        theme.border_default
    };

    let text_color = if is_active {
        theme.text_on_accent
    } else {
        theme.icon_default
    };

    ui.painter().rect_filled(rect, 6.0, bg_color);
    ui.painter().rect_stroke(
        rect,
        6.0,
        Stroke::new(1.0, border_color),
        StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional((btn_size.y * 0.48).max(11.0)),
        text_color,
    );

    let clicked = resp.clicked();
    resp.on_hover_text(crate::ui::i18n::text(ui, tooltip));
    clicked
}

pub(super) use crate::ui::pages::design::controls::{caption as section_title, segments};

pub(super) fn collapsible_section_title(
    ui: &mut egui::Ui,
    id_salt: &str,
    label: &str,
    default_open: bool,
    theme: &AppTheme,
) -> bool {
    let id = ui.make_persistent_id(id_salt);
    let mut is_open = ui.ctx().data_mut(|d| *d.get_temp_mut_or_insert_with(id, || default_open));
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 26.0),
        egui::Sense::click(),
    );
    let hover = ui.ctx().animate_bool(resp.id.with("h"), resp.hovered());
    let open_anim = ui.ctx().animate_bool(resp.id.with("open"), is_open);

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    if resp.clicked() {
        is_open = !is_open;
        ui.ctx().data_mut(|d| d.insert_temp(id, is_open));
    }

    // Hover background pill
    if hover > 0.0 {
        ui.painter().rect_filled(
            rect,
            6.0,
            theme.bg_card.lerp_to_gamma(theme.bg_card_hover, hover),
        );
    }

    // Caret icon on the left
    let caret = if is_open {
        egui_phosphor::regular::CARET_DOWN
    } else {
        egui_phosphor::regular::CARET_RIGHT
    };
    ui.painter().text(
        rect.left_center() + egui::vec2(6.0, 0.0),
        egui::Align2::LEFT_CENTER,
        caret,
        egui::FontId::proportional(11.0),
        theme.text_secondary.lerp_to_gamma(theme.text_primary, hover),
    );

    // Section title
    ui.painter().text(
        rect.left_center() + egui::vec2(22.0, 0.0),
        egui::Align2::LEFT_CENTER,
        crate::ui::i18n::text(ui, label),
        egui::FontId::proportional(11.5),
        if is_open {
            theme.text_primary
        } else {
            theme.text_secondary.lerp_to_gamma(theme.text_primary, hover)
        },
    );

    // Mini toggle switch on the right side
    let switch_rect = egui::Rect::from_center_size(
        rect.right_center() + egui::vec2(-16.0, 0.0),
        egui::vec2(26.0, 14.0),
    );
    let switch_bg = theme.border_default.lerp_to_gamma(theme.accent, open_anim);
    ui.painter().rect_filled(switch_rect, 7.0, switch_bg);
    let dot_x = switch_rect.left() + 7.0 + 12.0 * open_anim;
    ui.painter().circle_filled(
        egui::pos2(dot_x, switch_rect.center().y),
        4.5,
        Color32::WHITE,
    );

    resp.on_hover_text(if is_open {
        crate::ui::i18n::text(ui, "settings-hide-shader-properties")
    } else {
        crate::ui::i18n::text(ui, "settings-show-shader-properties")
    });

    is_open
}

pub(super) fn geo_field(
    ui: &mut egui::Ui,
    label: &str,
    val: &mut f32,
    is_size: bool,
    width: f32,
    theme: &AppTheme,
) -> bool {
    let mut changed = false;
    let mut percent = (*val * 100.0).round();
    ui.allocate_ui_with_layout(
        Vec2::new(width, 22.0),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let lbl_w = 12.0;
            ui.add_sized(
                [lbl_w, 20.0],
                egui::Label::new(
                    RichText::new(crate::ui::i18n::text(ui, label))
                        .size(11.0)
                        .strong()
                        .color(theme.text_muted),
                ),
            );
            let drag_w = (width - lbl_w - 4.0).max(30.0);
            let drag = egui::DragValue::new(&mut percent)
                .range(if is_size { 1.0..=100.0 } else { 0.0..=100.0 })
                .fixed_decimals(1)
                .suffix("%")
                .speed(0.25);
            if ui.add_sized([drag_w, 20.0], drag).changed() {
                *val = (percent / 100.0).clamp(0.0, 1.0);
                changed = true;
            }
        },
    );
    changed
}

pub(super) fn transition_tile(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    is_active: bool,
    size: Vec2,
    theme: &AppTheme,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let bg_color = if is_active {
        theme.accent
    } else if resp.hovered() {
        theme.bg_card_hover
    } else {
        theme.bg_pill
    };

    let stroke = if is_active {
        Stroke::new(1.0, theme.accent)
    } else if resp.hovered() {
        Stroke::new(1.0, theme.border_active)
    } else {
        Stroke::new(1.0, theme.border_default)
    };

    let text_color = if is_active {
        theme.text_on_accent
    } else if resp.hovered() {
        theme.text_primary
    } else {
        theme.text_secondary
    };

    ui.painter().rect_filled(rect, 7.0, bg_color);
    ui.painter().rect_stroke(rect, 7.0, stroke, StrokeKind::Inside);

    let label_text = crate::ui::i18n::text(ui, label);
    let display_text = format!("{icon} {label_text}");
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        display_text,
        egui::FontId::proportional(11.0),
        text_color,
    );

    resp.clicked()
}

pub(super) fn property_slider<T: egui::emath::Numeric>(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut T,
    range: std::ops::RangeInclusive<T>,
    suffix: &str,
    decimals: usize,
    speed: impl Into<f64>,
    theme: &AppTheme,
) -> bool {
    let mut changed = false;
    let label_w = 68.0;
    let val_w = 54.0;
    let spacing = ui.spacing().item_spacing.x;

    ui.horizontal(|ui| {
        ui.add_sized(
            [label_w, 20.0],
            egui::Label::new(
                RichText::new(crate::ui::i18n::text(ui, label))
                    .size(11.5)
                    .color(theme.text_secondary),
            )
            .truncate(),
        );

        let avail_for_slider = (ui.available_width() - val_w - spacing).max(30.0);
        ui.spacing_mut().slider_width = avail_for_slider;
        let slider = egui::Slider::new(value, range.clone())
            .show_value(false)
            .fixed_decimals(decimals);
        if ui.add(slider).changed() {
            changed = true;
        }

        let dv = egui::DragValue::new(value)
            .range(range)
            .fixed_decimals(decimals)
            .suffix(suffix)
            .speed(speed);
        if ui.add_sized([val_w, 20.0], dv).changed() {
            changed = true;
        }
    });
    changed
}

pub(super) fn property_slider_pct(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    min_pct: f32,
    max_pct: f32,
    theme: &AppTheme,
) -> bool {
    let mut changed = false;
    let label_w = 68.0;
    let val_w = 54.0;
    let spacing = ui.spacing().item_spacing.x;
    let mut pct = (*value * 100.0).round();

    ui.horizontal(|ui| {
        ui.add_sized(
            [label_w, 20.0],
            egui::Label::new(
                RichText::new(crate::ui::i18n::text(ui, label))
                    .size(11.5)
                    .color(theme.text_secondary),
            )
            .truncate(),
        );

        let avail_for_slider = (ui.available_width() - val_w - spacing).max(30.0);
        ui.spacing_mut().slider_width = avail_for_slider;
        let slider = egui::Slider::new(&mut pct, min_pct..=max_pct)
            .show_value(false)
            .fixed_decimals(0);
        if ui.add(slider).changed() {
            *value = pct / 100.0;
            changed = true;
        }

        let dv = egui::DragValue::new(&mut pct)
            .range(min_pct..=max_pct)
            .fixed_decimals(0)
            .suffix("%")
            .speed(1.0);
        if ui.add_sized([val_w, 20.0], dv).changed() {
            *value = pct / 100.0;
            changed = true;
        }
    });
    changed
}

pub(super) fn color_row_srgba(
    ui: &mut egui::Ui,
    label: &str,
    color: &mut Color32,
    theme: &AppTheme,
) -> bool {
    let mut changed = false;
    let label_w = 68.0;
    ui.horizontal(|ui| {
        ui.add_sized(
            [label_w, 20.0],
            egui::Label::new(
                RichText::new(crate::ui::i18n::text(ui, label))
                    .size(11.5)
                    .color(theme.text_secondary),
            )
            .truncate(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if crate::ui::components::color_swatch_button(ui, label, color, true, theme) {
                changed = true;
            }
        });
    });
    changed
}

pub(super) fn color_row_rgb(
    ui: &mut egui::Ui,
    label: &str,
    rgb: &mut [f32; 3],
    theme: &AppTheme,
) -> bool {
    let mut changed = false;
    let label_w = 68.0;
    let mut c32 = Color32::from_rgb(
        (rgb[0] * 255.0).round() as u8,
        (rgb[1] * 255.0).round() as u8,
        (rgb[2] * 255.0).round() as u8,
    );
    ui.horizontal(|ui| {
        ui.add_sized(
            [label_w, 20.0],
            egui::Label::new(
                RichText::new(crate::ui::i18n::text(ui, label))
                    .size(11.5)
                    .color(theme.text_secondary),
            )
            .truncate(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if crate::ui::components::color_swatch_button(ui, label, &mut c32, false, theme) {
                rgb[0] = c32.r() as f32 / 255.0;
                rgb[1] = c32.g() as f32 / 255.0;
                rgb[2] = c32.b() as f32 / 255.0;
                changed = true;
            }
        });
    });
    changed
}

pub(super) fn color_row_solid(
    ui: &mut egui::Ui,
    label: &str,
    color: &mut Color32,
    theme: &AppTheme,
) -> bool {
    let mut changed = false;
    let label_w = 68.0;
    ui.horizontal(|ui| {
        ui.add_sized(
            [label_w, 20.0],
            egui::Label::new(
                RichText::new(crate::ui::i18n::text(ui, label))
                    .size(11.5)
                    .color(theme.text_secondary),
            )
            .truncate(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if crate::ui::components::color_swatch_button(ui, label, color, false, theme) {
                changed = true;
            }
        });
    });
    changed
}

#[allow(unused_imports)]
pub(super) use crate::ui::components::{
    color_row_u8_3, color_swatch_button, color_swatch_button_sized, color_swatch_button_tooltip,
    color_swatch_u8_3, color_swatch_u8_3_sized, color_swatch_u8_4, color_swatch_u8_4_sized,
    parse_hex_color,
};
