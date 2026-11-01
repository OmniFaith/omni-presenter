use crate::{
    core::state::ControlPanelState,
    ui::theme::AppTheme,
};
use eframe::egui::{self, Color32, RichText, Stroke};
use egui_phosphor::regular::*;

pub struct AspectRatioChoice {
    pub ratio: Option<f32>,
    pub label: &'static str,
    pub description: &'static str,
    pub width: f32,
    pub height: f32,
}

pub fn standard_aspect_ratios(default_ratio: f32) -> Vec<AspectRatioChoice> {
    vec![
        AspectRatioChoice {
            ratio: None,
            label: "design-aspect-ratio-default",
            description: "design-aspect-ratio-default-desc",
            width: default_ratio.clamp(0.5, 3.5),
            height: 1.0,
        },
        AspectRatioChoice {
            ratio: Some(16.0 / 9.0),
            label: "design-aspect-ratio-widescreen",
            description: "design-aspect-ratio-widescreen-desc",
            width: 16.0,
            height: 9.0,
        },
        AspectRatioChoice {
            ratio: Some(16.0 / 10.0),
            label: "design-aspect-ratio-laptop",
            description: "design-aspect-ratio-laptop-desc",
            width: 16.0,
            height: 10.0,
        },
        AspectRatioChoice {
            ratio: Some(4.0 / 3.0),
            label: "design-aspect-ratio-classic",
            description: "design-aspect-ratio-classic-desc",
            width: 4.0,
            height: 3.0,
        },
        AspectRatioChoice {
            ratio: Some(21.0 / 9.0),
            label: "design-aspect-ratio-ultrawide",
            description: "design-aspect-ratio-ultrawide-desc",
            width: 21.0,
            height: 9.0,
        },
        AspectRatioChoice {
            ratio: Some(1.0),
            label: "design-aspect-ratio-square",
            description: "design-aspect-ratio-square-desc",
            width: 1.0,
            height: 1.0,
        },
        AspectRatioChoice {
            ratio: Some(9.0 / 16.0),
            label: "design-aspect-ratio-vertical",
            description: "design-aspect-ratio-vertical-desc",
            width: 9.0,
            height: 16.0,
        },
    ]
}

pub fn aspect_ratio_name(ratio: f32) -> &'static str {
    if (ratio - 16.0 / 9.0).abs() < 0.05 {
        "16:9"
    } else if (ratio - 16.0 / 10.0).abs() < 0.05 {
        "16:10"
    } else if (ratio - 4.0 / 3.0).abs() < 0.05 {
        "4:3"
    } else if (ratio - 21.0 / 9.0).abs() < 0.05 {
        "21:9"
    } else if (ratio - 1.0).abs() < 0.05 {
        "1:1"
    } else if (ratio - 9.0 / 16.0).abs() < 0.05 {
        "9:16"
    } else {
        "design-aspect-ratio-custom"
    }
}

pub fn show_dropdown(
    response: &egui::Response,
    popup_id: egui::Id,
    preview_aspect_ratio: &mut Option<f32>,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
) {
    egui::Popup::menu(response)
        .id(popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            let popup_w = 264.0;
            ui.set_width(popup_w);
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new(RECTANGLE).size(13.0).color(tokens.accent));
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "design-preview-aspect-ratio"))
                        .size(12.0)
                        .strong()
                        .color(tokens.text_primary),
                );
            });
            ui.label(
                RichText::new(crate::ui::i18n::text(ui, "design-preview-aspect-ratio-desc"))
                    .size(10.0)
                    .color(tokens.text_muted),
            );
            ui.add_space(2.0);

            let default_ratio = state.presentation_aspect_ratio.clamp(0.5, 3.5);
            let choices = standard_aspect_ratios(default_ratio);

            egui::ScrollArea::vertical()
                .id_salt("design_aspect_ratio_scroll")
                .max_height(310.0)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 3.0);
                    for item in choices {
                        let is_selected = match (preview_aspect_ratio.as_ref(), item.ratio) {
                            (None, None) => true,
                            (Some(curr), Some(target)) => (curr - target).abs() < 0.02,
                            _ => false,
                        };

                        let item_h = 38.0;
                        let (rect, resp) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), item_h),
                            egui::Sense::click(),
                        );

                        let hover = resp.hovered();
                        let bg_color = if is_selected {
                            tokens.accent_soft
                        } else if hover {
                            tokens.bg_card_hover
                        } else {
                            Color32::TRANSPARENT
                        };

                        if bg_color != Color32::TRANSPARENT {
                            ui.painter().rect_filled(rect, 6.0, bg_color);
                        }
                        if is_selected {
                            ui.painter().rect_stroke(
                                rect,
                                6.0,
                                Stroke::new(1.0, tokens.accent.linear_multiply(0.4)),
                                egui::StrokeKind::Inside,
                            );
                        }

                        // Miniatura geométrica da silhueta da proporção
                        let icon_box = egui::Rect::from_center_size(
                            egui::pos2(rect.min.x + 18.0, rect.center().y),
                            egui::vec2(22.0, 22.0),
                        );
                        let max_dim = 18.0;
                        let (box_w, box_h) = if item.width >= item.height {
                            (max_dim, (max_dim * (item.height / item.width)).max(6.0))
                        } else {
                            ((max_dim * (item.width / item.height)).max(6.0), max_dim)
                        };
                        let shape_rect = egui::Rect::from_center_size(
                            icon_box.center(),
                            egui::vec2(box_w, box_h),
                        );
                        ui.painter().rect(
                            shape_rect,
                            2.0,
                            if is_selected {
                                tokens.accent.linear_multiply(0.18)
                            } else {
                                tokens.bg_pill
                            },
                            Stroke::new(
                                1.0,
                                if is_selected {
                                    tokens.accent
                                } else {
                                    tokens.border_default
                                },
                            ),
                            egui::StrokeKind::Inside,
                        );

                        // Rótulos de texto
                        let localized_label = crate::ui::i18n::text(ui, item.label);
                        let localized_desc = crate::ui::i18n::text(ui, item.description);
                        let label_text = if item.ratio.is_none() {
                            let def_name = crate::ui::i18n::text(ui, aspect_ratio_name(default_ratio));
                            format!("{localized_label} ({def_name})")
                        } else {
                            localized_label.to_string()
                        };

                        let label_galley = ui.painter().layout_no_wrap(
                            label_text,
                            egui::FontId::proportional(11.5),
                            if is_selected {
                                tokens.accent
                            } else {
                                tokens.text_primary
                            },
                        );
                        let sub_galley = ui.painter().layout_no_wrap(
                            localized_desc.to_string(),
                            egui::FontId::proportional(9.5),
                            tokens.text_muted,
                        );

                        ui.painter().galley(
                            egui::pos2(rect.min.x + 36.0, rect.min.y + 4.0),
                            label_galley,
                            if is_selected {
                                tokens.accent
                            } else {
                                tokens.text_primary
                            },
                        );
                        ui.painter().galley(
                            egui::pos2(rect.min.x + 36.0, rect.min.y + 20.0),
                            sub_galley,
                            tokens.text_muted,
                        );

                        if is_selected {
                            ui.painter().text(
                                egui::pos2(rect.max.x - 12.0, rect.center().y),
                                egui::Align2::CENTER_CENTER,
                                CHECK,
                                egui::FontId::proportional(13.0),
                                tokens.accent,
                            );
                        }

                        if resp.clicked() {
                            *preview_aspect_ratio = item.ratio;
                            ui.close();
                        }
                    }
                });

            // Se estiver simulando uma proporção customizada, permite definir como padrão ou resetar
            if let Some(custom) = *preview_aspect_ratio {
                ui.add_space(2.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(crate::ui::i18n::text(ui, "design-set-as-default"))
                                    .size(10.5)
                                    .color(tokens.text_primary),
                            )
                            .fill(tokens.bg_pill),
                        )
                        .on_hover_text(crate::ui::i18n::text(ui, "design-set-as-default-tooltip"))
                        .clicked()
                    {
                        state.presentation_aspect_ratio = custom;
                        state.settings.service_mut().aspect_ratio = custom;
                        *preview_aspect_ratio = None;
                        ui.close();
                    }

                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(crate::ui::i18n::text(ui, "design-restore-default"))
                                    .size(10.5)
                                    .color(tokens.text_muted),
                            )
                            .fill(Color32::TRANSPARENT),
                        )
                        .on_hover_text(crate::ui::i18n::text(ui, "design-restore-default-tooltip"))
                        .clicked()
                    {
                        *preview_aspect_ratio = None;
                        ui.close();
                    }
                });
            }
        });
}
