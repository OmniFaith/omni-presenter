use crate::core::preferences::{new_id, screen_url_slug, unique_screen_slug, AppSettings, VirtualScreen};
use crate::core::state::{ControlPanelState, FloatingPreviewState, FloatingPreviewTarget};
use crate::services::monitor::MonitorInfo;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, RichText, Stroke};
use egui_phosphor::regular::{
    ARROW_SQUARE_OUT, CHECK_CIRCLE, COPY, EYE, GEAR_SIX, INFO, PLUS, QR_CODE, TELEVISION_SIMPLE, TRASH, X,
};

use super::components::{card, setting_toggle, styled_button, ButtonTone};
use super::screen_card::{render_screen_card, ScreenCardAction, ScreenCardProps};

pub(super) fn outputs(
    ui: &mut egui::Ui,
    draft: &mut AppSettings,
    state: &mut ControlPanelState,
    monitors: &[MonitorInfo],
    theme: &AppTheme,
    db: Option<&crate::services::db::AppDb>,
) {
    let language = draft.language;
    card(ui, theme, |ui| {
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
            ui.painter().rect_filled(icon_rect, 6.0, theme.accent_soft);
            ui.painter().text(
                icon_rect.center(),
                egui::Align2::CENTER_CENTER,
                egui_phosphor::regular::DESKTOP,
                egui::FontId::proportional(15.0),
                theme.accent,
            );
            ui.label(
                RichText::new(crate::ui::i18n::text(ui, "settings-presentation-screens"))
                    .size(13.5)
                    .strong()
                    .color(theme.text_primary),
            );
            egui::Frame::new()
                .fill(theme.bg_pill)
                .corner_radius(6.0)
                .inner_margin(egui::vec2(6.0, 2.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(crate::ui::i18n::format(ui, "settings-detected", &[("count", &monitors.len().to_string())]))
                            .size(10.5)
                            .color(theme.text_secondary),
                    );
                });
        });
        ui.add_space(8.0);
        if monitors.is_empty() {
            ui.label(
                RichText::new(language.text("workspace-no-monitors-detected")).color(theme.text_secondary),
            );
        }
        egui::ScrollArea::horizontal()
            .id_salt("physical_output_cards")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    for (index, monitor) in monitors.iter().enumerate() {
                        ui.vertical(|ui| {
                        let enabled = draft.physical_monitor_enabled(&monitor.name);
                        let label = (index + 1).to_string();
                        let res_str = format!("{} × {}", monitor.resolution.0, monitor.resolution.1);
                        let (_response, action) = render_screen_card(
                            ui,
                            ScreenCardProps {
                                id_salt: &format!("phys_{}", monitor.name),
                                label: &label,
                                name: &monitor.display_name,
                                subtitle: Some(&res_str),
                                selected: enabled,
                                enabled: enabled && draft.physical_output_enabled,
                                is_virtual: false,
                                theme,
                            },
                        );

                        match action {
                            ScreenCardAction::Select | ScreenCardAction::ToggleEnabled => {
                                draft.toggle_physical_monitor(&monitor.name);
                                if draft.physical_monitor_enabled(&monitor.name) {
                                    draft.monitor_name = monitor.name.clone();
                                }
                            }
                            ScreenCardAction::OpenSettings => {
                                draft.monitor_name = monitor.name.clone();
                                state.settings_ui.show_physical_config =
                                    !state.settings_ui.show_physical_config;
                            }
                            ScreenCardAction::OpenPreview => {
                                state.settings_ui.floating_preview = Some(FloatingPreviewState {
                                    open: true,
                                    target: FloatingPreviewTarget::Physical {
                                        monitor_name: monitor.name.clone(),
                                        index: index + 1,
                                    },
                                    aspect_ratio: 16.0 / 9.0,
                                    show_info: true,
                                });
                            }
                            _ => {}
                        }
                        ui.add_sized([142.0, 18.0], egui::Label::new(
                            RichText::new(&monitor.display_name).size(11.0).color(theme.text_secondary)
                        ).truncate()).on_hover_text(format!("{} · {}", monitor.display_name, res_str));
                        });
                    }
                });
            });
        ui.add_space(4.0);
        setting_toggle(
            ui,
            &mut draft.physical_output_enabled,
            "workspace-open-the-projection-window-on-the-monitor",
            "settings-send-the-presentation-to-all-enabled-screens-above-click-a",
            theme,
        );
        setting_toggle(
            ui,
            &mut draft.persistent_physical_output,
            "settings-keep-the-presentation-window-ready",
            "settings-keep-the-output-ready-show-the-standby-wallpaper-when-no",
            theme,
        );
        ui.horizontal(|ui| {
            ui.label(RichText::new(crate::ui::i18n::text(ui, "settings-entrance-and-exit")).size(12.0).color(theme.text_secondary));
            let selected = match draft.physical_transition_ms {
                0 => crate::ui::i18n::text(ui, "settings-no-transition").to_owned(),
                200 => crate::ui::i18n::text(ui, "settings-fast-200-ms").to_owned(),
                350 => crate::ui::i18n::text(ui, "settings-smooth-350-ms").to_owned(),
                600 => crate::ui::i18n::text(ui, "settings-smooth-600-ms").to_owned(),
                1000 => crate::ui::i18n::text(ui, "settings-slow-1-s").to_owned(),
                ms => crate::ui::i18n::format(ui, "settings-smooth-ms", &[("name", &ms.to_string())]),
            };
            egui::ComboBox::from_id_salt("physical_entrance_transition").selected_text(&selected).show_ui(ui, |ui| {
                for (ms, key) in [
                    (0, "settings-no-transition"),
                    (200, "settings-fast-200-ms"),
                    (350, "settings-smooth-350-ms"),
                    (600, "settings-smooth-600-ms"),
                    (1000, "settings-slow-1-s"),
                ] {
                    ui.selectable_value(&mut draft.physical_transition_ms, ms, crate::ui::i18n::text(ui, key));
                }
            });
        });
    });
    super::physical_background::render(ui, draft, &state.themes, theme, db, monitors);

    // Additional configuration panel for the physical presentation screen
    if state.settings_ui.show_physical_config {
        ui.add_space(6.0);
        let selected_monitor = monitors.iter().find(|m| m.name == draft.monitor_name);
        card(ui, theme, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                let (icon_rect, _) =
                    ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
                ui.painter().rect_filled(icon_rect, 6.0, theme.accent_soft);
                ui.painter().text(
                    icon_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    GEAR_SIX,
                    egui::FontId::proportional(16.0),
                    theme.accent,
                );

                ui.label(
                    RichText::new(language.format(
                        "settings-projection-settings-title",
                        &[("name", selected_monitor.map(|m| m.display_name.as_str()).unwrap_or(crate::ui::i18n::text(ui, "settings-disconnected-monitor")))],
                    ))
                    .size(13.5)
                    .strong()
                    .color(theme.text_primary),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(RichText::new(X).size(13.0).color(theme.text_secondary))
                                .fill(theme.bg_pill)
                                .corner_radius(6.0),
                        )
                        .on_hover_text(language.text("workspace-close-settings-panel"))
                        .clicked()
                    {
                        state.settings_ui.show_physical_config = false;
                    }

                    // Floating preview button in header
                    let prev_btn = egui::Button::new(
                        RichText::new(format!("{EYE} {}", language.text("workspace-view-screen")))
                            .size(11.5)
                            .color(theme.accent),
                    )
                    .fill(theme.accent_soft)
                    .stroke(Stroke::new(1.0, theme.accent))
                    .corner_radius(6.0);

                    if ui
                        .add(prev_btn)
                        .on_hover_text(language.text("workspace-open-viewer-in-a-new-window"))
                        .clicked()
                    {
                        let idx = monitors
                            .iter()
                            .position(|m| m.name == draft.monitor_name)
                            .unwrap_or(0)
                            + 1;
                        state.settings_ui.floating_preview = Some(FloatingPreviewState {
                            open: true,
                            target: FloatingPreviewTarget::Physical {
                                monitor_name: draft.monitor_name.clone(),
                                index: idx,
                            },
                            aspect_ratio: 16.0 / 9.0,
                            show_info: true,
                        });
                    }
                });
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            if let Some(mon) = selected_monitor {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(language.text("workspace-native-resolution"))
                            .size(12.0)
                            .color(theme.text_secondary),
                    );
                    ui.label(
                        RichText::new(format!("{} × {} px", mon.resolution.0, mon.resolution.1))
                            .size(12.0)
                            .strong()
                            .color(theme.text_primary),
                    );

                    ui.add_space(16.0);
                    ui.label(
                        RichText::new(language.text("workspace-position"))
                            .size(12.0)
                            .color(theme.text_secondary),
                    );
                    ui.label(
                        RichText::new(format!("X: {}, Y: {}", mon.position.0, mon.position.1))
                            .size(12.0)
                            .color(theme.text_primary),
                    );

                    ui.add_space(16.0);
                    ui.label(
                        RichText::new(language.text("workspace-monitor-scale"))
                            .size(12.0)
                            .color(theme.text_secondary),
                    );
                    ui.label(
                        RichText::new(format!("{:.0}%", mon.scale_factor * 100.0))
                            .size(12.0)
                            .color(theme.text_primary),
                    );
                });
            } else {
                ui.label(
                    RichText::new(language.text("workspace-the-selected-monitor-is-currently-disconnected-or-virtual"))
                        .size(12.0)
                        .color(theme.text_secondary),
                );
            }
        });
    }

    ui.add_space(8.0);
    let selection_id = ui.make_persistent_id("selected_virtual_output");
    let mut selected = ui.ctx().data_mut(|d| d.get_temp::<String>(selection_id));
    let mut add = false;
    let mut duplicate_screen_id: Option<String> = None;
    let mut delete_screen_id: Option<String> = None;

    card(ui, theme, |ui| {
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
            ui.painter().rect_filled(icon_rect, 6.0, theme.bg_pill);
            ui.painter().text(
                icon_rect.center(),
                egui::Align2::CENTER_CENTER,
                TELEVISION_SIMPLE,
                egui::FontId::proportional(15.0),
                theme.text_secondary,
            );
            ui.label(
                RichText::new(language.text("workspace-virtual-presentation-screens"))
                    .size(13.5)
                    .strong()
                    .color(theme.text_primary),
            );
            egui::Frame::new()
                .fill(theme.bg_pill)
                .corner_radius(6.0)
                .inner_margin(egui::vec2(6.0, 2.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(crate::ui::i18n::format(ui, "settings-configured", &[("count", &draft.screens.len().to_string())]))
                            .size(10.5)
                            .color(theme.text_secondary),
                    );
                });
        });
        ui.add_space(8.0);
        egui::ScrollArea::horizontal()
            .id_salt("virtual_output_cards")
            .show(ui, |ui| {
            let url_slugs: Vec<String> = draft
                .screens
                .iter()
                .map(|s| screen_url_slug(s, &draft.screens))
                .collect();
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                for (index, screen) in draft.screens.iter_mut().enumerate() {
                    ui.vertical(|ui| {
                        let label = format!("v{}", index + 1);
                        let sub = if screen.lyrics_only {
                            Some(language.text("settings-obs-transparent"))
                        } else {
                            None
                        };
                        let (_response, action) = render_screen_card(
                            ui,
                            ScreenCardProps {
                                id_salt: &format!("virt_{}", screen.id),
                                label: &label,
                                name: &screen.name,
                                subtitle: sub,
                                selected: selected.as_deref() == Some(screen.id.as_str()),
                                enabled: screen.enabled,
                                is_virtual: true,
                                theme,
                            },
                        );


                        match action {
                            ScreenCardAction::Select => {
                                selected = if selected.as_deref() == Some(screen.id.as_str()) {
                                    None
                                } else {
                                    Some(screen.id.clone())
                                };
                            }
                            ScreenCardAction::ToggleEnabled => {
                                screen.enabled = !screen.enabled;
                            }
                            ScreenCardAction::OpenSettings => {
                                selected = Some(screen.id.clone());
                            }
                            ScreenCardAction::OpenPreview => {
                                state.settings_ui.floating_preview = Some(FloatingPreviewState {
                                    open: true,
                                    target: FloatingPreviewTarget::Virtual {
                                        id: screen.id.clone(),
                                        name: screen.name.clone(),
                                    },
                                    aspect_ratio: 16.0 / 9.0,
                                    show_info: true,
                                });
                            }
                            ScreenCardAction::CopyLink => {
                                let host = if draft.allow_lan {
                                    &state.settings_ui.lan_address
                                } else {
                                    "127.0.0.1"
                                };
                                let url_slug = url_slugs.get(index).map(|s| s.as_str()).unwrap_or(screen.id.as_str());
                                let url = state.settings_ui.discovery_url
                                    .strip_suffix("/control")
                                    .filter(|_| draft.allow_lan && state.settings.server_port == draft.server_port)
                                    .map(|base| format!("{base}/screen/{url_slug}"))
                                    .unwrap_or_else(|| crate::services::web::http_url(
                                        host, draft.server_port, &format!("/screen/{url_slug}"),
                                    ));
                                ui.ctx().copy_text(url);
                            }
                            ScreenCardAction::ShowQrCode => {
                                let host = if draft.allow_lan {
                                    &state.settings_ui.lan_address
                                } else {
                                    "127.0.0.1"
                                };
                                let url_slug = url_slugs.get(index).map(|s| s.as_str()).unwrap_or(screen.id.as_str());
                                let url = state.settings_ui.discovery_url
                                    .strip_suffix("/control")
                                    .filter(|_| draft.allow_lan && state.settings.server_port == draft.server_port)
                                    .map(|base| format!("{base}/screen/{url_slug}"))
                                    .unwrap_or_else(|| crate::services::web::http_url(
                                        host, draft.server_port, &format!("/screen/{url_slug}"),
                                    ));
                                crate::ui::components::open_qr_dialog(
                                    ui.ctx(),
                                    format!("Telão: {}", screen.name),
                                    url,
                                );
                            }
                            ScreenCardAction::OpenBrowser => {
                                let url_slug = url_slugs.get(index).map(|s| s.as_str()).unwrap_or(screen.id.as_str());
                                let url = crate::services::web::http_url(
                                    "127.0.0.1", draft.server_port, &format!("/screen/{url_slug}"),
                                );
                                ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                            }
                            ScreenCardAction::Duplicate => {
                                duplicate_screen_id = Some(screen.id.clone());
                            }
                            ScreenCardAction::Delete => {
                                delete_screen_id = Some(screen.id.clone());
                            }
                            _ => {}
                        }

                        ui.add_sized(
                            [142.0, 18.0],
                            egui::Label::new(
                                RichText::new(&screen.name)
                                    .size(11.0)
                                    .color(theme.text_secondary),
                            )
                            .truncate(),
                        );
                    });
                }

                // Add button (+)
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::click());
                let hover = ui
                    .ctx()
                    .animate_bool(response.id.with("hover"), response.hovered());
                ui.painter().rect_filled(
                    rect,
                    8.0,
                    theme.bg_pill.lerp_to_gamma(theme.accent_soft, hover),
                );
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    PLUS,
                    egui::FontId::proportional(22.0),
                    theme.icon_default.lerp_to_gamma(theme.accent, hover),
                );
                if response
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text(language.text("workspace-add-a-virtual-presentation-screen"))
                    .clicked()
                {
                    add = true;
                }
            });
        });
    });

    if add {
        let screen = VirtualScreen {
            id: new_id("screen"),
            name: language.format(
                "workspace-screen",
                &[("number", &(draft.screens.len() + 1).to_string())],
            ),
            ..Default::default()
        };
        selected = Some(screen.id.clone());
        draft.screens.push(screen);
    }

    if let Some(dup_id) = duplicate_screen_id
        && let Some(src) = draft.screens.iter().find(|s| s.id == dup_id).cloned() {
            let mut clone = src;
            clone.id = new_id("screen");
            clone.name = format!("{} ({})", clone.name, language.text("appearance-copy"));
            selected = Some(clone.id.clone());
            draft.screens.push(clone);
        }

    if let Some(del_id) = delete_screen_id {
        draft.screens.retain(|s| s.id != del_id);
        if selected.as_deref() == Some(&del_id) {
            selected = None;
        }
    }

    let mut remove = false;
    if let Some(index) = draft
        .screens
        .iter()
        .position(|s| Some(s.id.as_str()) == selected.as_deref())
    {
        ui.add_space(6.0);
        let idle_theme = draft.service().idle_theme_id.clone();
        let other_slugs_before: Vec<String> = draft.screens[..index]
            .iter()
            .map(|s| s.slug())
            .collect();
        let recents = &mut draft.recent_wallpapers;
        let screen = &mut draft.screens[index];
        card(ui, theme, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                let (icon_rect, _) =
                    ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
                ui.painter().rect_filled(icon_rect, 6.0, theme.accent_soft);
                ui.painter().text(
                    icon_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    TELEVISION_SIMPLE,
                    egui::FontId::proportional(16.0),
                    theme.accent,
                );

                ui.add(
                    egui::TextEdit::singleline(&mut screen.name)
                        .margin(egui::vec2(8.0, 5.0))
                        .desired_width(170.0)
                        .font(egui::FontId::proportional(13.5))
                        .hint_text(language.text("workspace-screen-name")),
                );

                let (active_rect, active_resp) =
                    ui.allocate_exact_size(egui::vec2(76.0, 26.0), egui::Sense::click());
                let active_hover = ui
                    .ctx()
                    .animate_bool(active_resp.id.with("act_h"), active_resp.hovered());
                if active_resp.clicked() {
                    screen.enabled = !screen.enabled;
                }
                let pill_bg = if screen.enabled {
                    theme
                        .accent_soft
                        .lerp_to_gamma(theme.accent, active_hover * 0.2)
                } else {
                    theme
                        .bg_pill
                        .lerp_to_gamma(theme.bg_card_hover, active_hover)
                };
                let pill_stroke = if screen.enabled {
                    Stroke::new(1.0, theme.accent)
                } else {
                    Stroke::new(1.0, theme.border_default)
                };
                ui.painter().rect_filled(active_rect, 13.0, pill_bg);
                ui.painter()
                    .rect_stroke(active_rect, 13.0, pill_stroke, egui::StrokeKind::Inside);
                let (dot_color, label_text, text_color) = if screen.enabled {
                    (theme.accent, language.text("workspace-active"), theme.accent)
                } else {
                    (
                        theme.text_muted,
                        language.text("workspace-inactive"),
                        theme.text_secondary,
                    )
                };
                ui.painter().circle_filled(
                    active_rect.left_center() + egui::vec2(10.0, 0.0),
                    3.5,
                    dot_color,
                );
                ui.painter().text(
                    active_rect.left_center() + egui::vec2(20.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    label_text,
                    egui::FontId::proportional(12.0),
                    text_color,
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if theme
                        .icon_button(
                            ui,
                            X,
                            language.text("workspace-close-this-screen-s-panel"),
                            false,
                            egui::Vec2::splat(26.0),
                        )
                        .clicked()
                    {
                        selected = None;
                    }

                    if styled_button(
                        ui,
                        Some(TRASH),
                        language.text("workspace-remove"),
                        ButtonTone::Danger,
                        theme,
                    )
                    .on_hover_text(language.text("workspace-delete-this-virtual-screen"))
                    .clicked()
                    {
                        remove = true;
                    }

                    // Floating preview button in header
                    if styled_button(
                        ui,
                        Some(EYE),
                        language.text("workspace-view-screen"),
                        ButtonTone::Primary,
                        theme,
                    )
                    .on_hover_text(language.text("workspace-view-in-a-new-window"))
                    .clicked()
                    {
                        state.settings_ui.floating_preview = Some(FloatingPreviewState {
                            open: true,
                            target: FloatingPreviewTarget::Virtual {
                                id: screen.id.clone(),
                                name: screen.name.clone(),
                            },
                            aspect_ratio: 16.0 / 9.0,
                            show_info: true,
                        });
                    }
                });
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            super::physical_background::render_virtual_screen_appearance(
                ui,
                screen,
                &state.themes,
                state.live_theme(),
                theme,
                db,
                recents,
                &idle_theme,
            );

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            ui.label(
                RichText::new(language.text("workspace-content-and-stage-display"))
                    .size(12.0)
                    .strong()
                    .color(theme.accent),
            );

            setting_toggle(
                ui,
                &mut screen.show_comments,
                "workspace-show-comments-and-notes",
                "workspace-show-instruction-lines-and-marked-notes",
                theme,
            );

            setting_toggle(
                ui,
                &mut screen.show_next_preview,
                "workspace-next-slide-preview",
                "workspace-show-the-next-stanza-or-verse-in-a-footer-so",
                theme,
            );

            setting_toggle(
                ui,
                &mut screen.show_clock,
                "workspace-show-stage-clock",
                "workspace-show-the-current-time-in-the-corner-of-the-screen",
                theme,
            );

            setting_toggle(
                ui,
                &mut screen.show_title,
                "workspace-show-title-or-reference",
                "workspace-show-the-song-title-or-projected-verse-reference",
                theme,
            );

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            ui.label(
                RichText::new(language.text("workspace-typography"))
                    .size(12.0)
                    .strong()
                    .color(theme.accent),
            );

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(language.text("workspace-text-scale"))
                        .size(12.5)
                        .color(theme.text_secondary),
                );
                ui.add(egui::Slider::new(&mut screen.font_scale, 50..=200).suffix("%"));
                for preset in [80, 100, 120, 150] {
                    if ui
                        .selectable_label(screen.font_scale == preset, format!("{preset}%"))
                        .clicked()
                    {
                        screen.font_scale = preset;
                    }
                }
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            ui.label(
                RichText::new(language.text("workspace-address-and-broadcast"))
                    .size(12.0)
                    .strong()
                    .color(theme.accent),
            );

            let host = if draft.allow_lan {
                &state.settings_ui.lan_address
            } else {
                "127.0.0.1"
            };
            let url_slug = unique_screen_slug(&screen.name, &screen.id, &other_slugs_before);
            let ip_url = crate::services::web::http_url(host, draft.server_port, &format!("/screen/{url_slug}"));
            let url = state.settings_ui.discovery_url
                .strip_suffix("/control")
                .filter(|_| draft.allow_lan && state.settings.server_port == draft.server_port)
                .map(|base| format!("{base}/screen/{url_slug}"))
                .unwrap_or_else(|| ip_url.clone());

            egui::Frame::new()
                .fill(theme.bg_app)
                .stroke(Stroke::new(1.0, theme.border_default))
                .corner_radius(8.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;

                        if !draft.server_enabled {
                            ui.label(
                                RichText::new(format!(
                                    "{INFO} {}",
                                    language.text("workspace-web-server-is-off-enable-it-in-connections-to-allow")
                                ))
                                .size(11.5)
                                .color(Color32::from_rgb(245, 158, 11)),
                            );
                        } else if !screen.enabled {
                            ui.label(
                                RichText::new(format!(
                                    "{INFO} {}",
                                    language.text("workspace-this-screen-is-currently-inactive")
                                ))
                                .size(11.5)
                                .color(theme.text_muted),
                            );
                        } else {
                            ui.label(
                                RichText::new(format!(
                                    "{CHECK_CIRCLE} {}",
                                    language.text("workspace-ready-for-streaming-and-capture")
                                ))
                                .size(11.5)
                                .color(Color32::from_rgb(34, 197, 94)),
                            );
                        }
                    });

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;

                        ui.add(
                            egui::Label::new(
                                RichText::new(&url)
                                    .monospace()
                                    .size(11.5)
                                    .color(theme.text_primary),
                            )
                            .truncate(),
                        );

                        if styled_button(
                            ui,
                            Some(COPY),
                            language.text("workspace-copy-link"),
                            ButtonTone::Secondary,
                            theme,
                        )
                        .on_hover_text(language.text("workspace-copy-url-to-clipboard"))
                        .clicked()
                        {
                            ui.ctx().copy_text(url.clone());
                        }

                        if styled_button(
                            ui,
                            Some(QR_CODE),
                            "QR",
                            ButtonTone::Secondary,
                            theme,
                        )
                        .on_hover_text("Mostrar QR Code para abrir este telão no celular ou tablet")
                        .clicked()
                        {
                            crate::ui::components::open_qr_dialog(
                                ui.ctx(),
                                format!("Telão: {}", screen.name),
                                url.clone(),
                            );
                        }

                        let available = screen.enabled
                            && state.settings_ui.server_running
                            && state.settings.server_port == draft.server_port
                            && state.settings.allow_lan == draft.allow_lan
                            && state
                                .settings
                                .screens
                                .iter()
                                .any(|s| s.id == screen.id && s.enabled);

                        ui.add_enabled_ui(available, |ui| {
                            if styled_button(
                                ui,
                                Some(ARROW_SQUARE_OUT),
                                language.text("workspace-open"),
                                ButtonTone::Primary,
                                theme,
                            )
                            .on_hover_text(language.text("workspace-open-the-viewer-in-a-new-browser-tab"))
                            .clicked()
                            {
                                let url_slug = unique_screen_slug(&screen.name, &screen.id, &other_slugs_before);
                                ui.ctx().open_url(egui::OpenUrl::new_tab(crate::services::web::http_url(
                                    "127.0.0.1", draft.server_port, &format!("/screen/{url_slug}"),
                                )));
                            }
                        });

                        // Quick button to open floating preview right here
                        if styled_button(
                            ui,
                            Some(EYE),
                            language.text("workspace-preview"),
                            ButtonTone::Secondary,
                            theme,
                        )
                        .on_hover_text(language.text("workspace-view-in-a-new-window"))
                        .clicked()
                        {
                            state.settings_ui.floating_preview = Some(FloatingPreviewState {
                                open: true,
                                target: FloatingPreviewTarget::Virtual {
                                    id: screen.id.clone(),
                                    name: screen.name.clone(),
                                },
                                aspect_ratio: 16.0 / 9.0,
                                show_info: true,
                            });
                        }
                    });
                    if url != ip_url {
                        ui.add_space(4.0);
                        ui.label(RichText::new(language.text("discovery-ip-alternative")).size(10.5).color(theme.text_muted));
                        ui.hyperlink(&ip_url);
                    }
                });
        });
        if remove {
            draft.screens.remove(index);
            selected = None;
        }
    }
    ui.ctx().data_mut(|d| {
        if let Some(selected) = selected {
            d.insert_temp(selection_id, selected);
        } else {
            d.remove_temp::<String>(selection_id);
        }
    });
}
