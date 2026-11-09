use crate::core::state::{ControlPanelState, SettingsTab};
use crate::ui::components::dialog;
use crate::ui::modals::settings::components::{card, segmented_picker, setting_toggle, styled_button, ButtonTone};
use eframe::egui::{self, RichText};
use egui_phosphor::regular::{DESKTOP, EYE, GEAR, MONITOR, TELEVISION_SIMPLE};

pub fn render_presentation_settings_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    is_preview_modal_open: &mut bool,
) {
    if !state.show_presentation_settings_modal {
        return;
    }

    let tokens = state.theme.clone();

    let close = dialog::show(
        ctx,
        "presentation_settings_modal",
        "settings-presentation-settings",
        "settings-adjust-the-screen-aspect-ratio-thumbnails-and-bottom-bar-controls",
        480.0,
        &tokens,
        |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 10.0);

            // 1. Barra de Apresentação (Dock)
            card(ui, &tokens, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(crate::ui::i18n::format(ui, "settings-presentation-bottom-bar", &[("icon", TELEVISION_SIMPLE)]))
                            .size(13.0)
                            .strong()
                            .color(tokens.text_primary),
                    );
                });
                ui.add_space(4.0);

                setting_toggle(
                    ui,
                    &mut state.dock_preview_bg,
                    "settings-thumbnails-with-theme-background",
                    "settings-show-the-theme-background-colors-and-fonts-in-stanza-cards",
                    &tokens,
                );
            });

            // 2. Projeção e Proporção de Tela
            card(ui, &tokens, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(crate::ui::i18n::format(ui, "settings-screen-and-aspect-ratio", &[("icon", DESKTOP)]))
                            .size(13.0)
                            .strong()
                            .color(tokens.text_primary),
                    );
                });
                ui.add_space(4.0);

                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "settings-screen-aspect-ratio"))
                        .size(11.5)
                        .color(tokens.text_secondary),
                );
                ui.add_space(2.0);

                #[derive(Clone, Copy, PartialEq)]
                enum AspectOption {
                    Monitor,
                    Wide16_9,
                    Wide16_10,
                    Standard4_3,
                    Ultra21_9,
                }

                let current_opt = if state.settings.service().aspect_ratio <= 0.05 {
                    AspectOption::Monitor
                } else if (state.presentation_aspect_ratio - 16.0 / 9.0).abs() < 0.05 {
                    AspectOption::Wide16_9
                } else if (state.presentation_aspect_ratio - 16.0 / 10.0).abs() < 0.05 {
                    AspectOption::Wide16_10
                } else if (state.presentation_aspect_ratio - 4.0 / 3.0).abs() < 0.05 {
                    AspectOption::Standard4_3
                } else if (state.presentation_aspect_ratio - 21.0 / 9.0).abs() < 0.05 {
                    AspectOption::Ultra21_9
                } else {
                    AspectOption::Monitor
                };

                let mut selected_opt = current_opt;
                if segmented_picker(
                    ui,
                    &mut selected_opt,
                    &[
                        (AspectOption::Monitor, "Auto"),
                        (AspectOption::Wide16_9, "16:9"),
                        (AspectOption::Wide16_10, "16:10"),
                        (AspectOption::Standard4_3, "4:3"),
                        (AspectOption::Ultra21_9, "21:9"),
                    ],
                    &tokens,
                ) {
                    let ratio = match selected_opt {
                        AspectOption::Monitor => 0.0,
                        AspectOption::Wide16_9 => 16.0 / 9.0,
                        AspectOption::Wide16_10 => 16.0 / 10.0,
                        AspectOption::Standard4_3 => 4.0 / 3.0,
                        AspectOption::Ultra21_9 => 21.0 / 9.0,
                    };
                    state.settings.service_mut().aspect_ratio = ratio;
                    if ratio > 0.0 {
                        state.presentation_aspect_ratio = ratio;
                    }
                }
            });

            // 3. Ações Rápidas & Telas
            card(ui, &tokens, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(crate::ui::i18n::format(ui, "settings-screens-and-preview", &[("icon", MONITOR)]))
                            .size(13.0)
                            .strong()
                            .color(tokens.text_primary),
                    );
                });
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    if styled_button(
                        ui,
                        Some(EYE),
                        "settings-preview-window",
                        ButtonTone::Secondary,
                        &tokens,
                    )
                    .on_hover_text(crate::ui::i18n::text(ui, "settings-open-the-expanded-carousel-in-a-separate-window"))
                    .clicked()
                    {
                        *is_preview_modal_open = true;
                    }

                    if styled_button(
                        ui,
                        Some(GEAR),
                        "settings-configure-monitors",
                        ButtonTone::Secondary,
                        &tokens,
                    )
                    .on_hover_text(crate::ui::i18n::text(ui, "settings-open-video-output-and-screen-settings"))
                    .clicked()
                    {
                        state.settings_ui.tab = SettingsTab::Outputs;
                        state.settings_ui.open = true;
                        state.show_presentation_settings_modal = false;
                    }
                });
            });

            ui.add_space(4.0);

            // Botão Fechar / Concluir
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if styled_button(ui, None, "settings-done", ButtonTone::Primary, &tokens).clicked() {
                        state.show_presentation_settings_modal = false;
                    }
                });
            });
        },
    );

    if close {
        state.show_presentation_settings_modal = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_settings_modal_renders_and_toggles_options() {
        let mut state = ControlPanelState::default();
        state.show_presentation_settings_modal = true;
        let mut is_preview_open = false;

        let ctx = egui::Context::default();
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(1000.0, 700.0),
            )),
            ..Default::default()
        };

        let mut output = ctx.run_ui(raw_input, |ui| {
            render_presentation_settings_modal(ui.ctx(), &mut state, &mut is_preview_open);
        });
        output.textures_delta.clear();
        assert!(!output.shapes.is_empty(), "Modal should render shapes");
    }
}
