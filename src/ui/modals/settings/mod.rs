use crate::core::state::{ControlPanelState, SettingsTab};
use crate::services::db::AppDb;
use crate::services::monitor::MonitorInfo;
use eframe::egui::{self, Color32, RichText, Stroke};
use egui_phosphor::regular::{
    ARROWS_DOWN_UP, BOOK_OPEN, BOOKS, GEAR_SIX, GLOBE, INFO, KEYBOARD, PALETTE, PUZZLE_PIECE, USER, X,
};

mod about;
pub(crate) mod transfer;
mod appearance;
mod bibles;
pub mod components;
mod connections;
pub mod floating_preview;
mod interface;
pub mod outputs;
pub mod physical_background;
mod profiles;
mod plugins;
mod plugin_details;
pub mod screen_card;
pub mod shortcuts;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod extension_tests;

pub use floating_preview::render_floating_screen_preview;

pub fn render_settings_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    monitors: &[MonitorInfo],
    db: Option<&AppDb>,
) {
    let id = egui::Id::unique("settings_modal_overlay");
    if !state.settings_ui.open {
        crate::ui::components::package_io::cancel_import::<crate::core::plugins::PluginPackage>(ctx, "import_plugin");
        crate::ui::components::package_io::cancel_import::<crate::core::interface_theme::InterfaceTheme>(ctx, "import_interface_theme");
        state.settings_ui.draft = None;
        ctx.data_mut(|d| d.remove_temp::<u64>(id));
        ctx.animate_bool_with_time(id.with("enter"), false, 0.0);
        return;
    }
    if state.settings_ui.draft.is_none() {
        state.capture_profiles();
        state.settings_ui.draft = Some(state.settings.clone());
        state.settings_ui.error.clear();
    }
    let frame = ctx.cumulative_frame_nr();
    let opened = ctx.data_mut(|d| *d.get_temp_mut_or_insert_with(id, || frame));
    let mut draft = state.settings_ui.draft.take().unwrap();
    let (theme, _) = crate::ui::AppTheme::resolve(&draft, ctx);
    state.theme = theme.clone();
    let mut close = state.settings_ui.shortcut_capture.command.is_none()
        && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
    let language = draft.language;
    crate::ui::i18n::install(ctx, language);
    let mut save = false;
    let screen = ctx.input(|i| i.viewport_rect());
    let size = egui::vec2(
        980.0_f32.min(screen.width() - 32.0),
        680.0_f32.min(screen.height() - 32.0),
    );
    let rect = egui::Rect::from_center_size(screen.center(), size);

    egui::Area::new(id)
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            components::apply_settings_style(ui, &theme);
            ui.allocate_rect(screen, egui::Sense::click_and_drag());
            let progress = ctx.animate_bool_with_time(
                id.with("enter"),
                true,
                if state.settings.operator().reduce_motion {
                    0.0
                } else {
                    0.16
                },
            );

            // Backdrop dimming
            ui.painter().rect_filled(
                screen,
                0.0,
                Color32::from_black_alpha((110.0 * progress) as u8),
            );

            let floating_rect = ctx.data(|d| d.get_temp::<egui::Rect>(egui::Id::unique("floating_preview_rect")));
            if frame > opened
                && !egui::Popup::is_any_open(ctx)
                && ui.input(|i| {
                    i.pointer.any_click()
                        && i.pointer.interact_pos().is_some_and(|p| {
                            !rect.contains(p) && floating_rect.is_none_or(|fr| !fr.contains(p))
                        })
                })
            {
                close = true;
            }

            // Elevation shadow
            let shadow = egui::epaint::Shadow {
                offset: [0, 16],
                blur: 48,
                spread: 0,
                color: Color32::from_black_alpha(55),
            };
            ui.painter().add(shadow.as_shape(rect, 14.0));

            // Modal Background
            ui.painter().rect_filled(rect, 14.0, theme.bg_card);

            // Left Sidebar Layout
            let sidebar_width = 210.0_f32;
            let sidebar_rect = egui::Rect::from_min_max(
                rect.min,
                egui::pos2(rect.min.x + sidebar_width, rect.max.y),
            );

            // Sidebar contrasting panel fill
            ui.painter().rect_filled(sidebar_rect, 14.0, theme.bg_app);

            // Vertical divider between sidebar and content
            ui.painter().line_segment(
                [
                    egui::pos2(rect.min.x + sidebar_width, rect.min.y),
                    egui::pos2(rect.min.x + sidebar_width, rect.max.y),
                ],
                Stroke::new(1.0, theme.border_default),
            );

            // Outer modal border stroke
            ui.painter().rect_stroke(
                rect,
                14.0,
                Stroke::new(1.0, theme.border_default),
                egui::StrokeKind::Inside,
            );

            // --- Sidebar Content ---
            let nav_inner_rect = egui::Rect::from_min_max(
                sidebar_rect.min + egui::vec2(12.0, 18.0),
                sidebar_rect.max - egui::vec2(12.0, 18.0),
            );
            let mut nav = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt("settings_nav")
                    .max_rect(nav_inner_rect),
            );

            // Sidebar Header Branding
            nav.horizontal(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), egui::Sense::hover());
                ui.painter().rect_filled(icon_rect, 6.0, theme.accent_soft);
                ui.painter().text(
                    icon_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    GEAR_SIX,
                    egui::FontId::proportional(15.0),
                    theme.accent,
                );

                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(language.text("settings-title"))
                            .size(13.5)
                            .strong()
                            .color(theme.text_primary),
                    );
                    ui.label(
                        RichText::new("Omni Presenter v0.7 alpha")
                            .size(10.0)
                            .color(theme.text_muted),
                    );
                });
            });

            nav.add_space(14.0);
            nav.separator();
            nav.add_space(8.0);

            // Navigation Items
            for (tab, icon, label) in [
                (
                    SettingsTab::Outputs,
                    BOOK_OPEN,
                    language.text("settings-outputs"),
                ),
                (
                    SettingsTab::Interface,
                    PALETTE,
                    language.text("settings-interface"),
                ),
                (
                    SettingsTab::Shortcuts,
                    KEYBOARD,
                    language.text("shortcuts-title"),
                ),
                (
                    SettingsTab::Plugins,
                    PUZZLE_PIECE,
                    language.text("plugins-title"),
                ),
                (SettingsTab::Bible, BOOKS, language.text("settings-bibles")),
                (
                    SettingsTab::Connections,
                    GLOBE,
                    language.text("settings-connections"),
                ),
                (
                    SettingsTab::Profiles,
                    USER,
                    language.text("settings-profiles"),
                ),
                (SettingsTab::Transfer, ARROWS_DOWN_UP, language.text("transfer-title")),
                (SettingsTab::About, INFO, language.text("settings-about")),
            ] {
                let active = state.settings_ui.tab == tab;
                let (item_rect, response) = nav.allocate_exact_size(
                    egui::vec2(nav.available_width(), 38.0),
                    egui::Sense::click(),
                );
                let hover = ctx.animate_bool(response.id.with("hover"), response.hovered());
                let selected = ctx.animate_bool(response.id.with("selected"), active);

                let bg_color = if selected > 0.0 {
                    theme.bg_card.lerp_to_gamma(theme.accent_soft, selected)
                } else if hover > 0.0 {
                    theme.bg_pill.lerp_to_gamma(theme.bg_card_hover, hover)
                } else {
                    Color32::TRANSPARENT
                };
                nav.painter().rect_filled(item_rect, 8.0, bg_color);

                if selected > 0.05 {
                    nav.painter().rect_filled(
                        egui::Rect::from_min_size(
                            item_rect.left_top() + egui::vec2(2.0, 8.0),
                            egui::vec2(3.5, item_rect.height() - 16.0),
                        ),
                        1.75,
                        theme.accent,
                    );
                }

                let item_color = theme.text_secondary
                    .lerp_to_gamma(theme.text_primary, hover)
                    .lerp_to_gamma(theme.accent, selected);

                nav.painter().text(
                    item_rect.min + egui::vec2(18.0, 19.0),
                    egui::Align2::CENTER_CENTER,
                    icon,
                    egui::FontId::proportional(15.5),
                    item_color,
                );
                nav.painter().text(
                    item_rect.min + egui::vec2(38.0, 19.0),
                    egui::Align2::LEFT_CENTER,
                    label,
                    if active {
                        egui::FontId::proportional(13.0)
                    } else {
                        egui::FontId::proportional(12.5)
                    },
                    item_color,
                );

                if response.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    state.settings_ui.tab = tab;
                }
                nav.add_space(3.0);
            }

            // --- Right Content Container ---
            let header_height = 60.0_f32;
            let footer_height = 54.0_f32;
            let content_left = rect.min.x + sidebar_width;
            let content_right = rect.max.x;

            // Header Area
            let content_header_rect = egui::Rect::from_min_max(
                egui::pos2(content_left + 24.0, rect.min.y + 16.0),
                egui::pos2(content_right - 24.0, rect.min.y + header_height),
            );
            let mut header_ui = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt("settings_header")
                    .max_rect(content_header_rect),
            );

            let (active_icon, active_title, active_subtitle) = match state.settings_ui.tab {
                SettingsTab::Transfer => (ARROWS_DOWN_UP, language.text("transfer-title"), language.text("transfer-subtitle")),
                SettingsTab::Outputs => (
                    BOOK_OPEN,
                    language.text("settings-outputs"),
                    language.text("settings-manage-presentation-monitors-and-virtual-screens-for-obs-and-the"),
                ),
                SettingsTab::Interface => (
                    PALETTE,
                    language.text("settings-interface"),
                    language.text("settings-customize-the-appearance-language-text-scale-and-layout"),
                ),
                SettingsTab::Shortcuts => (
                    KEYBOARD,
                    language.text("shortcuts-title"),
                    language.text("settings-keyboard-shortcuts-for-quick-presentation-control"),
                ),
                SettingsTab::Plugins => (
                    PUZZLE_PIECE,
                    language.text("plugins-title"),
                    language.text("plugins-hint"),
                ),
                SettingsTab::Bible => (
                    BOOKS,
                    language.text("settings-bibles"),
                    language.text("settings-bible-translations-installed-in-the-local-database"),
                ),
                SettingsTab::Connections => (
                    GLOBE,
                    language.text("settings-connections"),
                    language.text("settings-local-web-server-connection-ports-and-lan-sharing"),
                ),
                SettingsTab::Profiles => (
                    USER,
                    language.text("settings-profiles"),
                    language.text("settings-operator-profiles-and-saved-service-schedules"),
                ),
                SettingsTab::About => (
                    INFO,
                    language.text("settings-about"),
                    language.text("settings-system-diagnostics-version-and-runtime-information"),
                ),
            };

            header_ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    let (h_icon_rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
                    ui.painter().rect_filled(h_icon_rect, 6.0, theme.accent_soft);
                    ui.painter().text(
                        h_icon_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        active_icon,
                        egui::FontId::proportional(16.0),
                        theme.accent,
                    );

                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(active_title)
                                .size(17.0)
                                .strong()
                                .color(theme.text_primary),
                        );
                        ui.label(
                            RichText::new(language.text(active_subtitle))
                                .size(11.0)
                                .color(theme.text_secondary),
                        );
                    });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if theme
                        .icon_button(
                            ui,
                            X,
                            language.text("common-close"),
                            false,
                            egui::Vec2::splat(28.0),
                        )
                        .clicked()
                    {
                        close = true;
                    }
                });
            });

            // Header separator line
            ui.painter().line_segment(
                [
                    egui::pos2(content_left, rect.min.y + header_height),
                    egui::pos2(content_right, rect.min.y + header_height),
                ],
                Stroke::new(1.0, theme.border_default),
            );

            // Fixed Footer Area at bottom
            let footer_y = rect.max.y - footer_height;
            ui.painter().line_segment(
                [
                    egui::pos2(content_left, footer_y),
                    egui::pos2(content_right, footer_y),
                ],
                Stroke::new(1.0, theme.border_default),
            );

            let footer_rect = egui::Rect::from_min_max(
                egui::pos2(content_left + 24.0, footer_y + 8.0),
                egui::pos2(content_right - 24.0, rect.max.y - 8.0),
            );
            let mut footer_ui = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt("settings_footer")
                    .max_rect(footer_rect),
            );

            footer_ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(crate::ui::i18n::format(
                            ui,
                            "settings-active-profiles",
                            &[
                                ("operator", &draft.operator().name),
                                ("service", &draft.service().display_name(draft.language)),
                            ],
                        ))
                        .size(11.5)
                        .color(theme.text_secondary),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if crate::ui::components::dialog::action(
                        ui,
                        "common-save",
                        true,
                        &theme,
                    )
                    .clicked()
                    {
                        save = true;
                    }

                    ui.add_space(8.0);

                    if crate::ui::components::dialog::action(
                        ui,
                        "common-cancel",
                        false,
                        &theme,
                    )
                    .clicked()
                    {
                        close = true;
                    }
                });
            });

            // Scrollable Content Area between Header and Footer
            let body_rect = egui::Rect::from_min_max(
                egui::pos2(content_left + 24.0, rect.min.y + header_height + 12.0),
                egui::pos2(content_right - 24.0, footer_y - 8.0),
            );
            let mut content = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt("settings_content")
                    .max_rect(body_rect),
            );
            components::apply_settings_style(&mut content, &theme);
            content.set_max_width(body_rect.width());
            content.set_clip_rect(content.clip_rect().intersect(body_rect));
            content.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

            if !state.settings_ui.error.is_empty() {
                content.colored_label(theme.status_danger, &state.settings_ui.error);
                content.add_space(6.0);
            }

            egui::ScrollArea::vertical()
                .id_salt(("settings_scroll", state.settings_ui.tab as usize))
                .auto_shrink([false, false])
                .show(&mut content, |ui| {
                    ui.set_width((body_rect.width() - 16.0).max(1.0));
                    ui.set_clip_rect(ui.clip_rect().intersect(body_rect));
                    match state.settings_ui.tab {
                    SettingsTab::Transfer => transfer::render(ui, &draft, db),
                    SettingsTab::Outputs => {
                        outputs::outputs(ui, &mut draft, state, monitors, &theme, db)
                    }
                    SettingsTab::Interface => interface::interface(ui, &mut draft, state, &theme),
                    SettingsTab::Shortcuts => shortcuts::shortcuts(
                        ui,
                        &mut draft,
                        &mut state.settings_ui.shortcut_capture,
                        &theme,
                    ),
                    SettingsTab::Plugins => plugins::render(ui, &mut draft, state, &theme),
                    SettingsTab::Bible => bibles::bibles(ui, &mut draft, state, db, &theme),
                    SettingsTab::Connections => {
                        connections::connections(ui, &mut draft, state, &theme)
                    }
                    SettingsTab::Profiles => profiles::profiles(ui, &mut draft, &theme),
                    SettingsTab::About => about::about(ui, state, &theme),
                    }
                });
        });

    if save {
        draft.normalize();
        let invalid_theme = draft.interface_themes.iter().find_map(|theme| theme.validate().err());
        let invalid_plugin_settings = draft.plugins.iter().flat_map(|plugin| plugin.package.settings.iter().map(move |field| (plugin, field)))
            .find(|(plugin, field)| plugin.settings.get(&field.id).is_some_and(|value| !field.accepts(value)));
        if draft.operators.iter().any(|p| p.name.trim().is_empty())
            || draft.services.iter().any(|p| p.name.trim().is_empty())
            || draft.screens.iter().any(|s| s.name.trim().is_empty())
        {
            state.settings_ui.error = language.text("settings-names-required").into();
        } else if let Some(error) = invalid_theme {
            state.settings_ui.error = language.text(&error).to_owned();
        } else if invalid_plugin_settings.is_some() {
            state.settings_ui.error = language.text("plugins-invalid-output").into();
        } else {
            // The application persists the draft before applying it or moving the output.
            state.settings_ui.save_requested = true;
        }
    }
    if close {
        state.settings_ui.open = false;
        state.settings_ui.shortcut_capture = Default::default();
        state.settings_ui.draft = None;
        state.settings_ui.save_requested = false;
        state.settings_ui.error.clear();
        ctx.data_mut(|d| d.remove_temp::<u64>(id));
        ctx.animate_bool_with_time(id.with("enter"), false, 0.0);
        let (saved_theme, saved_dark) = crate::ui::AppTheme::resolve(&state.settings, ctx);
        state.theme = saved_theme;
        let mut visuals = if saved_dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        state.theme.apply_to_visuals(&mut visuals);
        ctx.set_visuals(visuals);
        ctx.request_repaint();
    } else {
        let (preview_theme, _) = crate::ui::AppTheme::resolve(&draft, ctx);
        state.theme = preview_theme;
        state.settings_ui.draft = Some(draft);
    }
}
