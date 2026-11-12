//! Compact action-first layout; extensive format/security help stays opt-in.
use super::super::components::{self, ButtonTone};
use crate::{core::i18n::Language, ui::theme::AppTheme};
use eframe::egui::{self, RichText};
use egui_phosphor::regular::{DOWNLOAD_SIMPLE, FILE_TEXT, MUSIC_NOTES, UPLOAD_SIMPLE};

#[derive(Default)]
pub(super) struct Actions {
    pub export: bool,
    pub import: bool,
    pub lyrics: bool,
    pub example: bool,
}

fn heading(ui: &mut egui::Ui, icon: &str, title: &str, description: &str, theme: &AppTheme) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 8.0, theme.accent_soft);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(18.0),
            theme.accent,
        );
        ui.add_space(3.0);
        ui.label(
            RichText::new(title)
                .size(14.0)
                .strong()
                .color(theme.text_primary),
        );
    });
    ui.label(
        RichText::new(description)
            .size(11.5)
            .color(theme.text_secondary),
    );
}

fn backup_card(ui: &mut egui::Ui, export: bool, language: Language, theme: &AppTheme) -> bool {
    let mut clicked = false;
    components::card(ui, theme, |ui| {
        ui.set_min_height(128.0);
        heading(
            ui,
            if export {
                UPLOAD_SIMPLE
            } else {
                DOWNLOAD_SIMPLE
            },
            language.text(if export {
                "transfer-export-card"
            } else {
                "transfer-import-card"
            }),
            language.text(if export {
                "transfer-export-hint"
            } else {
                "transfer-import-hint"
            }),
            theme,
        );
        // Fixed action position keeps the two cards balanced across translations.
        ui.add_space((ui.min_rect().top() + 96.0 - ui.cursor().top()).max(0.0));
        clicked = components::styled_button(
            ui,
            None,
            if export {
                "transfer-export"
            } else {
                "transfer-import"
            },
            ButtonTone::Secondary,
            theme,
        )
        .clicked();
    });
    clicked
}

pub(super) fn show(ui: &mut egui::Ui, language: Language, theme: &AppTheme) -> Actions {
    let mut actions = Actions::default();
    ui.spacing_mut().item_spacing.y = 10.0;
    if ui.available_width() >= 480.0 {
        ui.columns(2, |columns| {
            actions.export = backup_card(&mut columns[0], true, language, theme);
            actions.import = backup_card(&mut columns[1], false, language, theme);
        });
    } else {
        actions.export = backup_card(ui, true, language, theme);
        actions.import = backup_card(ui, false, language, theme);
    }
    components::card(ui, theme, |ui| {
        heading(
            ui,
            MUSIC_NOTES,
            language.text("transfer-lyrics"),
            language.text("transfer-lyrics-hint"),
            theme,
        );
        ui.horizontal_wrapped(|ui| {
            actions.lyrics = components::styled_button(
                ui,
                None,
                "transfer-lyrics-choose",
                ButtonTone::Primary,
                theme,
            )
            .clicked();
            actions.example = components::styled_button(
                ui,
                Some(FILE_TEXT),
                "transfer-example",
                ButtonTone::Ghost,
                theme,
            )
            .clicked();
        });
    });
    actions
}

pub(super) fn details(ui: &mut egui::Ui, language: Language, theme: &AppTheme) {
    egui::CollapsingHeader::new(
        RichText::new(language.text("transfer-details"))
            .size(11.5)
            .color(theme.text_secondary),
    )
    .id_salt("transfer_help")
    .show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 8.0;
        for key in [
            "transfer-scope",
            "transfer-exclusions",
            "transfer-lyrics-formats",
            "transfer-sensitive",
        ] {
            ui.label(
                RichText::new(language.text(key))
                    .size(11.5)
                    .color(theme.text_secondary),
            );
        }
    });
}
