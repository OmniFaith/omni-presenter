use super::components::{ButtonTone, card, setting_toggle, styled_button};
use crate::{
    core::{
        i18n::Language,
        plugins::{InstalledPlugin, PluginRequest},
        state::ControlPanelState,
    },
    ui::{
        components::{package_io, plugin_fields},
        theme::AppTheme,
    },
};
use eframe::egui::{self, RichText};
use egui_phosphor::regular::{PLAY, PLUS, PUZZLE_PIECE, STACK, TRASH};

pub(super) fn render(
    ui: &mut egui::Ui,
    plugin: &mut InstalledPlugin,
    state: &mut ControlPanelState,
    language: Language,
    theme: &AppTheme,
) -> bool {
    let mut remove = false;
    let installed = state.settings.plugins.iter().any(|saved| {
        saved.enabled
            && saved.package == plugin.package
            && plugin.package.settings.iter().all(|field| {
                saved.settings.get(&field.id).unwrap_or(&field.default)
                    == plugin.settings.get(&field.id).unwrap_or(&field.default)
            })
    });
    card(ui, theme, |ui| {
        ui.horizontal_top(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(42.0, 42.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 10.0, theme.accent_soft);
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                PUZZLE_PIECE,
                egui::FontId::proportional(22.0),
                theme.accent,
            );
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(plugin.package.localized_name(language))
                        .size(17.0)
                        .strong()
                        .color(theme.text_primary),
                );
                ui.label(
                    RichText::new(format!(
                        "{} · v{}",
                        plugin.package.author, plugin.package.version
                    ))
                    .size(11.0)
                    .color(theme.text_muted),
                );
            });
        });
        ui.add_space(4.0);
        ui.label(
            RichText::new(plugin.package.localized_description(language))
                .size(11.5)
                .color(theme.text_secondary),
        );
        setting_toggle(
            ui,
            &mut plugin.enabled,
            language.text("plugins-enable-this"),
            "",
            theme,
        );
        ui.separator();
        let mut tabs = vec![(0, "plugins-overview")];
        if !plugin.package.settings.is_empty() {
            tabs.push((1, "plugins-configuration"));
        }
        if !plugin.package.components.is_empty() {
            tabs.push((2, "plugins-components"));
        }
        if !plugin.package.commands.is_empty() {
            tabs.push((3, "plugins-tools"));
        }
        if !plugin.package.shaders.is_empty() {
            tabs.push((4, "plugins-shaders"));
        }
        if !plugin.package.templates.is_empty() {
            tabs.push((5, "plugins-templates"));
        }
        if !plugin.package.interface_themes.is_empty() {
            tabs.push((6, "appearance-title"));
        }
        let tab_id = egui::Id::unique(("plugin_detail_tab", &plugin.package.id));
        let mut selected = ui
            .ctx()
            .data(|data| data.get_temp::<u8>(tab_id))
            .filter(|tab| tabs.iter().any(|(id, _)| tab == id))
            .unwrap_or(0);
        ui.horizontal_wrapped(|ui| {
            for (tab, key) in &tabs {
                if ui
                    .selectable_label(selected == *tab, language.text(key))
                    .clicked()
                {
                    selected = *tab;
                }
            }
        });
        ui.ctx().data_mut(|data| data.insert_temp(tab_id, selected));
        ui.add_space(8.0);
        match selected {
            1 => {
                plugin_fields::fields(
                    ui,
                    &plugin.package.settings,
                    &mut plugin.settings,
                    language,
                    theme,
                );
            }
            2 => {
                for component in &plugin.package.components {
                    ui.strong(component.localized_name(language));
                    if !component.description.is_empty() {
                        ui.label(
                            RichText::new(component.localized_description(language))
                                .small()
                                .color(theme.text_secondary),
                        );
                    }
                    ui.label(
                        RichText::new(
                            component
                                .fields
                                .iter()
                                .filter(|field| !field.hidden)
                                .map(|field| field.label.as_str())
                                .collect::<Vec<_>>()
                                .join(" · "),
                        )
                        .small()
                        .color(theme.text_muted),
                    );
                    ui.add_space(8.0);
                }
                ui.label(
                    RichText::new(language.text("plugins-components-hint"))
                        .small()
                        .color(theme.text_muted),
                );
            }
            3 => {
                for command in &plugin.package.commands {
                    ui.strong(command.localized_name(language));
                    ui.label(
                        RichText::new(command.localized_description(language))
                            .small()
                            .color(theme.text_secondary),
                    );
                    ui.add_enabled_ui(installed && plugin.enabled, |ui| {
                        if styled_button(
                            ui,
                            Some(PLAY),
                            language.text("plugins-run"),
                            ButtonTone::Secondary,
                            theme,
                        )
                        .clicked()
                        {
                            state.settings_ui.plugin_request = Some(PluginRequest::Command {
                                plugin: plugin.package.id.clone(),
                                command: command.id.clone(),
                            });
                        }
                    });
                    ui.add_space(8.0);
                }
            }
            4 => {
                for shader in &plugin.package.shaders {
                    ui.strong(shader.localized_name(language));
                    ui.label(
                        RichText::new(shader.localized_description(language))
                            .small()
                            .color(theme.text_secondary),
                    );
                    let mut preview = crate::core::models::PresentationTheme::default();
                    preview.bg_type = "shader".into();
                    preview.options.shader.custom = Some(shader.clone());
                    let width = ui.available_width();
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(width, (width / 2.4).clamp(72.0, 140.0)),
                        egui::Sense::hover(),
                    );
                    crate::render::paint_theme_bg(ui.painter(), rect, &preview, 8.0);
                }
                ui.label(
                    RichText::new(language.text("plugins-shaders-hint"))
                        .small()
                        .color(theme.text_muted),
                );
            }
            5 => {
                for (index, template) in plugin.package.templates.iter().enumerate() {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!("{STACK}  {}", template.name));
                        ui.add_enabled_ui(installed && plugin.enabled, |ui| {
                            if styled_button(
                                ui,
                                Some(PLUS),
                                language.text("plugins-use-template"),
                                ButtonTone::Secondary,
                                theme,
                            )
                            .clicked()
                            {
                                state.settings_ui.plugin_request = Some(PluginRequest::Template {
                                    plugin: plugin.package.id.clone(),
                                    index,
                                });
                            }
                        });
                    });
                }
            }
            6 => {
                for definition in &plugin.package.interface_themes {
                    ui.label(definition.localized_name(language));
                }
            }
            _ => {
                ui.label(
                    RichText::new(language.text("plugins-includes"))
                        .size(12.0)
                        .strong(),
                );
                for (count, key, hint) in [
                    (
                        plugin.package.components.len(),
                        "plugins-component-count",
                        "plugins-components-hint",
                    ),
                    (
                        plugin.package.commands.len(),
                        "plugins-tool-count",
                        "plugins-tools-hint",
                    ),
                    (
                        plugin.package.shaders.len(),
                        "plugins-shader-count",
                        "plugins-shaders-hint",
                    ),
                    (
                        plugin.package.templates.len(),
                        "plugins-template-count",
                        "plugins-templates-hint",
                    ),
                    (
                        plugin.package.interface_themes.len(),
                        "plugins-theme-count",
                        "plugins-themes-hint",
                    ),
                ] {
                    if count == 0 {
                        continue;
                    }
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(language.count(key, key, count))
                            .color(theme.text_primary),
                    );
                    ui.label(
                        RichText::new(language.text(hint))
                            .size(10.5)
                            .color(theme.text_muted),
                    );
                }
            }
        }
        ui.add_space(10.0);
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            package_io::export(
                ui,
                "export_plugin",
                language.text("plugins-export"),
                &format!("{}.omni-presenter-plugin.json", plugin.package.id),
                &plugin.package,
            );
            if styled_button(
                ui,
                Some(TRASH),
                language.text("plugins-remove"),
                ButtonTone::Ghost,
                theme,
            )
            .clicked()
            {
                remove = true;
            }
        });
    });
    remove
}
