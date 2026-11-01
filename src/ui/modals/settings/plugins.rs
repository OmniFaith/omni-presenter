use super::plugin_details;
use crate::{
    core::{
        plugins::{InstalledPlugin, PluginPackage, defaults},
        preferences::AppSettings,
        state::ControlPanelState,
    },
    ui::{components::package_io, theme::AppTheme},
};
use eframe::egui::{self, RichText, Stroke};
use egui_phosphor::regular::{DOTS_THREE, MAGNIFYING_GLASS, PLUS, PUZZLE_PIECE};

pub(super) fn render(
    ui: &mut egui::Ui,
    draft: &mut AppSettings,
    state: &mut ControlPanelState,
    theme: &AppTheme,
) {
    let language = draft.language;
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new(language.format(
                "plugins-installed-count",
                &[("count", &draft.plugins.len().to_string())],
            ))
            .size(13.0)
            .strong()
            .color(theme.text_primary),
        );
        ui.add_space(8.0);
        if let Some(result) = package_io::import(
            ui,
            "import_plugin",
            &format!("{PLUS}  {}", language.text("plugins-install")),
            PluginPackage::validate,
        ) {
            match result {
                Ok(package) => {
                    let id = package.id.clone();
                    if let Some(existing) = draft
                        .plugins
                        .iter_mut()
                        .find(|plugin| plugin.package.id == id)
                    {
                        existing.package = package;
                    } else {
                        let settings = defaults(&package.settings);
                        draft.plugins.push(InstalledPlugin {
                            package,
                            enabled: true,
                            settings,
                        });
                    }
                    ui.ctx()
                        .data_mut(|data| data.insert_temp(egui::Id::unique("selected_plugin"), id));
                    state.settings_ui.error.clear();
                }
                Err(error) => state.settings_ui.error = language.text(&error).to_owned(),
            }
        }
        ui.menu_button(DOTS_THREE, |ui| {
            if ui
                .button(language.text("plugins-restore-examples"))
                .clicked()
            {
                for plugin in crate::core::plugins::bundled() {
                    if !draft
                        .plugins
                        .iter()
                        .any(|item| item.package.id == plugin.package.id)
                    {
                        draft.plugins.push(plugin);
                    }
                }
                ui.close();
            }
        });
    });
    ui.add_space(8.0);
    let query_id = egui::Id::unique("plugins_query");
    let mut query = ui
        .ctx()
        .data(|data| data.get_temp::<String>(query_id))
        .unwrap_or_default();
    ui.add(
        egui::TextEdit::singleline(&mut query)
            .hint_text(format!(
                "{MAGNIFYING_GLASS}  {}",
                language.text("plugins-search")
            ))
            .desired_width(ui.available_width()),
    );
    ui.ctx()
        .data_mut(|data| data.insert_temp(query_id, query.clone()));
    ui.add_space(12.0);
    let query = crate::core::bible::fold(&query);
    let filtered: Vec<_> = draft
        .plugins
        .iter()
        .filter(|plugin| {
            query.is_empty()
                || crate::core::bible::fold(&format!(
                    "{} {} {}",
                    plugin.package.name, plugin.package.description, plugin.package.author
                ))
                .contains(&query)
        })
        .map(|plugin| plugin.package.id.clone())
        .collect();
    let selection_id = egui::Id::unique("selected_plugin");
    let mut selected = ui
        .ctx()
        .data(|data| data.get_temp::<String>(selection_id))
        .filter(|id| filtered.contains(id))
        .or_else(|| filtered.first().cloned())
        .unwrap_or_default();
    if filtered.is_empty() {
        egui::Frame::new()
            .fill(theme.bg_panel)
            .stroke(Stroke::new(1.0, theme.border_default))
            .corner_radius(10)
            .inner_margin(24)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(PUZZLE_PIECE)
                        .size(28.0)
                        .color(theme.icon_muted),
                );
                ui.label(language.text(if draft.plugins.is_empty() {
                    "plugins-empty"
                } else {
                    "plugins-no-results"
                }));
            });
        return;
    }
    let width = ui.available_width();
    let mut remove = false;
    if width >= 600.0 {
        let list_width = (width * 0.32).clamp(180.0, 220.0);
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            ui.allocate_ui_with_layout(
                egui::vec2(list_width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(list_width);
                    list(
                        ui,
                        &draft.plugins,
                        &filtered,
                        &mut selected,
                        language,
                        theme,
                    );
                },
            );
            let detail_width = width - list_width - 12.0;
            ui.allocate_ui_with_layout(
                egui::vec2(detail_width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(detail_width);
                    if let Some(plugin) = draft
                        .plugins
                        .iter_mut()
                        .find(|plugin| plugin.package.id == selected)
                    {
                        remove = plugin_details::render(ui, plugin, state, language, theme);
                    }
                },
            );
        });
    } else {
        list(
            ui,
            &draft.plugins,
            &filtered,
            &mut selected,
            language,
            theme,
        );
        ui.add_space(12.0);
        if let Some(plugin) = draft
            .plugins
            .iter_mut()
            .find(|plugin| plugin.package.id == selected)
        {
            remove = plugin_details::render(ui, plugin, state, language, theme);
        }
    }
    if remove {
        draft.plugins.retain(|plugin| plugin.package.id != selected);
    }
    ui.ctx()
        .data_mut(|data| data.insert_temp(selection_id, selected));
    ui.add_space(12.0);
    ui.label(
        RichText::new(language.text("plugins-save-hint"))
            .size(10.5)
            .color(theme.text_muted),
    );
}

fn list(
    ui: &mut egui::Ui,
    plugins: &[InstalledPlugin],
    visible: &[String],
    selected: &mut String,
    language: crate::core::i18n::Language,
    theme: &AppTheme,
) {
    for plugin in plugins
        .iter()
        .filter(|plugin| visible.contains(&plugin.package.id))
    {
        let active = *selected == plugin.package.id;
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 68.0), egui::Sense::click());
        let fill = if active {
            theme.accent_soft
        } else if response.hovered() {
            theme.bg_card_hover
        } else {
            theme.bg_panel
        };
        ui.painter().rect(
            rect,
            9.0,
            fill,
            Stroke::new(
                1.0,
                if active {
                    theme.accent
                } else {
                    theme.border_default
                },
            ),
            egui::StrokeKind::Inside,
        );
        let icon_rect =
            egui::Rect::from_min_size(rect.min + egui::vec2(10.0, 15.0), egui::vec2(34.0, 34.0));
        ui.painter().rect_filled(icon_rect, 8.0, theme.bg_pill);
        ui.painter().text(
            icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            PUZZLE_PIECE,
            egui::FontId::proportional(18.0),
            if active {
                theme.accent
            } else {
                theme.icon_default
            },
        );
        let text_rect = egui::Rect::from_min_max(
            rect.min + egui::vec2(54.0, 11.0),
            rect.max - egui::vec2(10.0, 8.0),
        );
        let mut text = ui.new_child(egui::UiBuilder::new().max_rect(text_rect));
        text.add(
            egui::Label::new(
                RichText::new(&plugin.package.name)
                    .size(12.0)
                    .strong()
                    .color(theme.text_primary),
            )
            .truncate(),
        );
        text.add(
            egui::Label::new(
                RichText::new(format!(
                    "v{} · {}",
                    plugin.package.version,
                    language.text(if plugin.enabled {
                        "plugins-enabled"
                    } else {
                        "plugins-disabled"
                    })
                ))
                .size(10.0)
                .color(theme.text_muted),
            )
            .truncate(),
        );
        if response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            *selected = plugin.package.id.clone();
        }
        ui.add_space(6.0);
    }
    ui.add_space(4.0);
    ui.label(
        RichText::new(language.text("plugins-list-hint"))
            .size(10.0)
            .color(theme.text_muted),
    );
}
