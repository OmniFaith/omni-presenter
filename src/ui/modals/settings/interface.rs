use crate::core::preferences::AppSettings;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, RichText, Stroke};
use egui_phosphor::regular::{ARROW_COUNTER_CLOCKWISE, CHAT_CIRCLE_TEXT, GLOBE, LAYOUT, SLIDERS};

use super::components::{
    card, card_header, section, segmented_picker, setting_toggle, styled_button,
    ButtonTone,
};

pub(super) fn interface(ui: &mut egui::Ui, draft: &mut AppSettings, theme: &AppTheme) {
    let language = draft.language;

    // --- Language Selection ---
    section(
        ui,
        language.text("settings-language"),
        language.text("settings-language-hint"),
        theme,
    );
    card(ui, theme, |ui| {
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
            ui.painter().rect_filled(icon_rect, 6.0, theme.accent_soft);
            ui.painter().text(
                icon_rect.center(),
                egui::Align2::CENTER_CENTER,
                GLOBE,
                egui::FontId::proportional(15.0),
                theme.accent,
            );
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(language.text("settings-language"))
                        .size(13.0)
                        .strong()
                        .color(theme.text_primary),
                );
                ui.label(
                    RichText::new(language.text("settings-language-hint"))
                        .size(11.0)
                        .color(theme.text_secondary),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("interface_language_picker")
                    .selected_text(
                        RichText::new(draft.language.name())
                            .size(12.5)
                            .strong()
                            .color(theme.text_primary),
                    )
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for lang in crate::core::i18n::Language::ALL {
                            let is_selected = draft.language == lang;
                            if ui.selectable_value(&mut draft.language, lang, lang.name()).clicked() {
                                ui.ctx().request_repaint();
                            }
                            if is_selected {
                                ui.scroll_to_cursor(Some(egui::Align::Center));
                            }
                        }
                    });
            });
        });
    });

    card(ui, theme, |ui| {
        setting_toggle(
            ui,
            &mut draft.ask_media_conversion,
            language.text("library-conversion-setting"),
            language.text("library-conversion-setting-hint"),
            theme,
        );
        ui.add_space(8.0);
        setting_toggle(
            ui,
            &mut draft.hardware_acceleration,
            language.text("settings-hardware-acceleration"),
            language.text("settings-hardware-acceleration-hint"),
            theme,
        );
    });

    // --- Appearance & Theme Mode ---
    super::appearance::render(ui, draft, theme);
    let operator = draft
        .operators
        .iter_mut()
        .find(|p| p.id == draft.active_operator)
        .unwrap();

    ui.add_space(10.0);

    // --- Typography & UI Scale ---
    card(ui, theme, |ui| {
        card_header(
            ui,
            SLIDERS,
            language.text("settings-scale"),
            language.text("settings-operator-hint"),
            theme,
        );

        ui.add_space(4.0);

        // Segmented Presets
        let options = [
            (90, language.text("settings-compact")),
            (100, language.text("settings-standard")),
            (110, language.text("settings-comfortable")),
            (125, language.text("settings-large")),
        ];
        segmented_picker(ui, &mut operator.scale, &options, theme);

        ui.add_space(6.0);

        // Full-width fine tuning slider row
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(language.text("settings-fine-tune"))
                    .size(12.0)
                    .color(theme.text_secondary),
            );
            ui.add_space(4.0);

            let slider_w = (ui.available_width() - 56.0).max(100.0);
            ui.spacing_mut().slider_width = slider_w;
            ui.add(
                egui::Slider::new(&mut operator.scale, 80..=150)
                    .step_by(5.0)
                    .show_value(false)
                    .trailing_fill(true),
            );

            egui::Frame::new()
                .fill(theme.accent_soft)
                .corner_radius(6.0)
                .inner_margin(egui::vec2(6.0, 3.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("{}%", operator.scale))
                            .size(11.5)
                            .strong()
                            .color(theme.accent),
                    );
                });
        });

        ui.add_space(4.0);

        // Dynamic typography preview box
        egui::Frame::new()
            .fill(theme.bg_pill)
            .corner_radius(8.0)
            .inner_margin(12)
            .stroke(Stroke::new(1.0, theme.border_default))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let preview_scale = operator.scale as f32 / 100.0;
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Omni Presenter")
                            .size(14.0 * preview_scale)
                            .strong()
                            .color(theme.text_primary),
                    );
                    ui.label(
                        RichText::new("·")
                            .size(13.0 * preview_scale)
                            .color(theme.text_muted),
                    );
                    ui.label(
                        RichText::new(language.text("settings-preview"))
                            .size(12.5 * preview_scale)
                            .color(theme.text_secondary),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        egui::Frame::new()
                            .fill(theme.accent_soft)
                            .corner_radius(5.0)
                            .inner_margin(egui::vec2(7.0, 2.5))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(format!("{:.0} pt · Aa 123", 13.0 * preview_scale))
                                        .size(10.5)
                                        .color(theme.accent)
                                        .strong(),
                                );
                            });
                    });
                });
            });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        setting_toggle(
            ui,
            &mut operator.reduce_motion,
            language.text("settings-reduce-motion"),
            language.text("settings-reduce-motion-hint"),
            theme,
        );
    });

    operator.show_dock = true;
    operator.keyboard_shortcuts = true;
    operator.chat_notifications = true;

    // --- Workspace & Panels Layout Reset ---
    section(
        ui,
        language.text("settings-operation"),
        language.text("settings-operation-hint"),
        theme,
    );
    card(ui, theme, |ui| {
        // Main Workspace
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
            ui.painter().rect_filled(icon_rect, 6.0, theme.bg_pill);
            ui.painter().text(
                icon_rect.center(),
                egui::Align2::CENTER_CENTER,
                LAYOUT,
                egui::FontId::proportional(15.0),
                theme.text_secondary,
            );
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(language.text("settings-reset-layout"))
                        .size(13.0)
                        .strong()
                        .color(theme.text_primary),
                );
                ui.label(
                    RichText::new(language.text("workspace-restores-the-default-workspace-panel-layout"))
                        .size(11.0)
                        .color(theme.text_secondary),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if styled_button(
                    ui,
                    Some(ARROW_COUNTER_CLOCKWISE),
                    "settings-restore",
                    ButtonTone::Secondary,
                    theme,
                )
                .clicked()
                {
                    operator.layout = crate::core::layout::LayoutTree::default();
                }
            });
        });

        ui.add_space(2.0);
        ui.separator();
        ui.add_space(2.0);

        // Bible Workspace
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
            ui.painter().rect_filled(icon_rect, 6.0, theme.bg_pill);
            ui.painter().text(
                icon_rect.center(),
                egui::Align2::CENTER_CENTER,
                SLIDERS,
                egui::FontId::proportional(15.0),
                theme.text_secondary,
            );
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "settings-bible-panel"))
                        .size(13.0)
                        .strong()
                        .color(theme.text_primary),
                );
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "settings-restore-the-book-chapter-and-verse-columns"))
                        .size(11.0)
                        .color(theme.text_secondary),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if styled_button(
                    ui,
                    Some(ARROW_COUNTER_CLOCKWISE),
                    "settings-restore",
                    ButtonTone::Secondary,
                    theme,
                )
                .clicked()
                {
                    operator.bible_layout = crate::core::layout::LayoutTree::bible_workspace();
                }
            });
        });

        ui.add_space(2.0);
        ui.separator();
        ui.add_space(2.0);

        // Chat Workspace
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
            ui.painter().rect_filled(icon_rect, 6.0, theme.bg_pill);
            ui.painter().text(
                icon_rect.center(),
                egui::Align2::CENTER_CENTER,
                CHAT_CIRCLE_TEXT,
                egui::FontId::proportional(15.0),
                theme.text_secondary,
            );
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "settings-team-chat-panel"))
                        .size(13.0)
                        .strong()
                        .color(theme.text_primary),
                );
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "settings-restore-the-default-channel-and-message-layout"))
                        .size(11.0)
                        .color(theme.text_secondary),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if styled_button(
                    ui,
                    Some(ARROW_COUNTER_CLOCKWISE),
                    "settings-restore",
                    ButtonTone::Secondary,
                    theme,
                )
                .clicked()
                {
                    operator.chat_layout = crate::core::layout::LayoutTree::default_chat();
                }
            });
        });
    });
}
