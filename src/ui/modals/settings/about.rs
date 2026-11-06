use crate::core::state::ControlPanelState;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, RichText};
use egui_phosphor::regular::SPARKLE;

use super::components::{card, section, styled_button, ButtonTone};

pub(super) fn about(ui: &mut egui::Ui, state: &mut ControlPanelState, theme: &AppTheme) {
    section(
        ui,
        "settings-about-title",
        "settings-application-information-and-installed-version",
        theme,
    );

    // --- Hero Brand & Version Card ---
    card(ui, theme, |ui| {
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::hover());
            ui.painter().rect_filled(icon_rect, 8.0, theme.accent_soft);
            ui.painter().text(
                icon_rect.center(),
                egui::Align2::CENTER_CENTER,
                SPARKLE,
                egui::FontId::proportional(22.0),
                theme.accent,
            );

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Omni Presenter")
                            .size(16.0)
                            .strong()
                            .color(theme.text_primary),
                    );

                    let version_text = "v0.7 alpha";
                    egui::Frame::new()
                        .fill(theme.accent_soft)
                        .corner_radius(6.0)
                        .inner_margin(egui::vec2(6.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(version_text)
                                    .size(11.0)
                                    .strong()
                                    .color(theme.accent),
                            );
                        });

                    egui::Frame::new()
                        .fill(theme.bg_pill)
                        .corner_radius(6.0)
                        .inner_margin(egui::vec2(6.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("Rust 2024 · Linux")
                                    .size(11.0)
                                    .color(theme.text_secondary),
                            );
                        });
                });

                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "settings-a-modern-workspace-to-prepare-operate-and-present-services-efficiently"))
                        .size(11.5)
                        .color(theme.text_secondary),
                );
            });
        });
    });

    ui.add_space(8.0);

    // --- Initial Setup Wizard Re-run Card ---
    card(ui, theme, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(crate::ui::i18n::text(ui, "settings-wizard-title"))
                    .size(13.5)
                    .strong()
                    .color(theme.text_primary),
            );
        });
        ui.add_space(4.0);
        ui.label(
            RichText::new(crate::ui::i18n::text(ui, "settings-wizard-subtitle"))
                .size(11.5)
                .color(theme.text_secondary),
        );
        ui.add_space(6.0);
        if styled_button(
            ui,
            Some(SPARKLE),
            "settings-wizard-reopen",
            ButtonTone::Secondary,
            theme,
        )
        .clicked()
        {
            state.show_setup_wizard = true;
            state.setup_wizard.step = 0;
            state.settings_ui.open = false;
        }
    });
}
