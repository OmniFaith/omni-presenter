use super::components::{appearance_card, card, section};
use crate::{
    core::{
        interface_theme::{COLOR_KEYS, InterfaceTheme, ThemeMode},
        preferences::{AppSettings, new_id},
    },
    ui::{components::package_io, theme::AppTheme},
};
use eframe::egui;

pub(super) fn render(ui: &mut egui::Ui, draft: &mut AppSettings, theme: &AppTheme) {
    let language = draft.language;
    section(
        ui,
        language.text("appearance-title"),
        language.text("appearance-hint"),
        theme,
    );
    card(ui, theme, |ui| {
        let operator = draft.operator_mut();
        ui.horizontal_wrapped(|ui| {
            for (mode, label) in [
                (ThemeMode::System, "appearance-system"),
                (ThemeMode::Light, "appearance-light"),
                (ThemeMode::Dark, "appearance-dark"),
            ] {
                let selected = if operator.theme_mode == ThemeMode::Legacy {
                    if operator.dark_mode {
                        ThemeMode::Dark
                    } else {
                        ThemeMode::Light
                    }
                } else {
                    operator.theme_mode
                };
                if ui
                    .selectable_label(selected == mode, language.text(label))
                    .clicked()
                {
                    operator.theme_mode = mode;
                    if mode != ThemeMode::System {
                        operator.dark_mode = mode == ThemeMode::Dark;
                    }
                }
            }
        });
        ui.add_space(8.0);
        let mut catalogue = crate::core::interface_theme::builtins();
        catalogue.extend(draft.interface_themes.clone());
        for plugin in draft.plugins.iter().filter(|plugin| plugin.enabled) {
            catalogue.extend(plugin.package.interface_themes.clone());
        }
        if draft.operator().interface_theme_id == "midnight" {
            draft.operator_mut().interface_theme_id = "darker_pro".into();
        }
        let selected = draft.operator().interface_theme_id.clone();
        let mut choice = selected.clone();
        let dark = draft.operator().theme_mode.is_dark(
            ui.ctx().system_theme() == Some(egui::Theme::Dark),
            draft.operator().dark_mode,
        );
        let gap = 10.0;
        let available = ui.available_width();
        let columns = ((available + gap) / 170.0).floor().clamp(1.0, 3.0) as usize;
        let width = ((available - gap * (columns - 1) as f32) / columns as f32).max(1.0);
        for row in catalogue.chunks(columns) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for definition in row {
                    ui.push_id(&definition.id, |ui| {
                        let palette = AppTheme::from_definition(definition, dark);
                        if appearance_card(
                            ui,
                            &palette,
                            definition.localized_name(language),
                            choice == definition.id,
                            width,
                            theme,
                        ) {
                            choice = definition.id.clone();
                        }
                    });
                }
            });
            ui.add_space(gap);
        }
        draft.operator_mut().interface_theme_id = choice.clone();
        let definition = catalogue
            .iter()
            .find(|theme| theme.id == choice)
            .cloned()
            .unwrap_or_default();
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if let Some(result) = package_io::import(
                ui,
                "import_interface_theme",
                language.text("appearance-import"),
                InterfaceTheme::validate,
            ) {
                match result {
                    Ok(mut imported) => {
                        ui.ctx().data_mut(|data| {
                            data.remove::<String>(egui::Id::unique("appearance_error"))
                        });
                        if crate::core::interface_theme::builtins()
                            .iter()
                            .any(|theme| theme.id == imported.id)
                        {
                            imported.id = new_id("ui_theme");
                        }
                        draft.operator_mut().interface_theme_id = imported.id.clone();
                        draft
                            .interface_themes
                            .retain(|theme| theme.id != imported.id);
                        draft.interface_themes.push(imported);
                    }
                    Err(error) => {
                        ui.ctx().data_mut(|data| {
                            data.insert_temp(egui::Id::unique("appearance_error"), error)
                        });
                    }
                }
            }
            if ui.button(language.text("appearance-create")).clicked() {
                let mut copy = definition.clone();
                copy.id = new_id("ui_theme");
                copy.name = format!("{} · {}", copy.name, language.text("appearance-copy"));
                draft.operator_mut().interface_theme_id = copy.id.clone();
                draft.interface_themes.push(copy);
            }
            package_io::export(
                ui,
                "export_interface_theme",
                language.text("appearance-export"),
                &format!("{}.omni-presenter-theme.json", definition.id),
                &definition,
            );
        });
        if let Some(error) = ui
            .ctx()
            .data(|data| data.get_temp::<String>(egui::Id::unique("appearance_error")))
        {
            ui.colored_label(theme.status_danger, language.text(&error));
        }
        if let Some(custom) = draft
            .interface_themes
            .iter_mut()
            .find(|theme| theme.id == choice)
        {
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(language.text("appearance-name")).strong());
                ui.add(egui::TextEdit::singleline(&mut custom.name).desired_width(180.0));
                ui.add_space(20.0);
                ui.add(
                    egui::Slider::new(&mut custom.rounding, 0.0..=24.0)
                        .text(language.text("appearance-rounding")),
                );
            });
            ui.add_space(4.0);
            for dark in [false, true] {
                ui.add_space(4.0);
                egui::CollapsingHeader::new(
                    egui::RichText::new(language.text(if dark {
                        "appearance-dark-palette"
                    } else {
                        "appearance-light-palette"
                    }))
                    .size(13.0)
                    .strong()
                    .color(theme.text_primary),
                )
                .id_salt(("palette", dark))
                .show(ui, |ui| {
                    ui.add_space(4.0);
                    let defaults = AppTheme::from_definition(custom, dark);
                    let values = if dark {
                        &mut custom.dark
                    } else {
                        &mut custom.light
                    };
                    let json = defaults.color_values();

                    let gap = 10.0;
                    let avail = ui.available_width();
                    let cols = if avail >= 760.0 { 3 } else if avail >= 420.0 { 2 } else { 1 };
                    let item_w = ((avail - gap * (cols - 1) as f32) / cols as f32).max(140.0);

                    for chunk in COLOR_KEYS.chunks(cols) {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = gap;
                            for key in chunk {
                                egui::Frame::new()
                                    .fill(theme.bg_card)
                                    .stroke(egui::Stroke::new(1.0, theme.border_subtle))
                                    .corner_radius(6.0)
                                    .inner_margin(egui::Margin::symmetric(10, 6))
                                    .show(ui, |ui| {
                                        ui.set_width(item_w - 20.0);
                                        ui.horizontal(|ui| {
                                            let label_key = format!("appearance-color-{}", key.replace('_', "-"));
                                            ui.label(
                                                egui::RichText::new(language.text(&label_key))
                                                    .size(12.0)
                                                    .color(theme.text_primary),
                                            );
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    let mut color = values
                                                        .get(*key)
                                                        .and_then(|value| {
                                                            crate::core::interface_theme::parse_color(value)
                                                        })
                                                        .or_else(|| json.get(*key).copied())
                                                        .unwrap_or([0, 0, 0, 255]);
                                                    if ui
                                                        .color_edit_button_srgba_unmultiplied(&mut color)
                                                        .changed()
                                                    {
                                                        values.insert(
                                                            (*key).into(),
                                                            format!(
                                                                "#{:02X}{:02X}{:02X}{:02X}",
                                                                color[0], color[1], color[2], color[3]
                                                            ),
                                                        );
                                                    }
                                                },
                                            );
                                        });
                                    });
                            }
                        });
                        ui.add_space(2.0);
                    }
                });
            }
            ui.add_space(8.0);
            if ui
                .button(
                    egui::RichText::new(language.text("appearance-remove"))
                        .color(theme.status_danger),
                )
                .clicked()
            {
                draft.interface_themes.retain(|theme| theme.id != choice);
                for operator in &mut draft.operators {
                    if operator.interface_theme_id == choice {
                        operator.interface_theme_id = "classic".into();
                    }
                }
            }
        }
    });
}
