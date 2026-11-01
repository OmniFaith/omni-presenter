use crate::core::i18n::Language;
use crate::core::preferences::AppSettings;
use crate::core::state::ControlPanelState;
use crate::ui::components::qr_code::render_qr_code;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, RichText, Stroke};
use egui_phosphor::regular::{
    ARROW_SQUARE_OUT, CHECK_CIRCLE, COPY, GLOBE, QR_CODE, SLIDERS_HORIZONTAL, WIFI_HIGH,
};

use super::components::{
    card, card_header, section, setting_toggle, styled_button, ButtonTone,
};

pub(super) fn connections(
    ui: &mut egui::Ui,
    draft: &mut AppSettings,
    state: &ControlPanelState,
    theme: &AppTheme,
) {
    let language = draft.language;
    let host = if draft.allow_lan && !state.settings_ui.lan_address.is_empty() {
        state.settings_ui.lan_address.as_str()
    } else {
        "127.0.0.1"
    };

    // --- Header Section ---
    section(
        ui,
        "settings-web-server-and-viewers",
        "settings-stream-live-lyrics-and-slides-to-obs-studio-browsers-and",
        theme,
    );

    // --- Card 1: Web Control Connections ---
    card(ui, theme, |ui| {
        card_header(
            ui,
            SLIDERS_HORIZONTAL,
            "control-title",
            "settings-access-the-full-control-panel-in-a-browser-on-phones",
            theme,
        );

        ui.add_space(8.0);

        let running_host = if state.settings.allow_lan && !state.settings_ui.lan_address.is_empty()
        {
            state.settings_ui.lan_address.as_str()
        } else {
            "127.0.0.1"
        };
        let url =
            crate::services::web::http_url(running_host, state.settings.server_port, "/control");

        let automatic_url = &state.settings_ui.discovery_url;
        let automatic_available = state.settings.allow_lan && automatic_url.ends_with("/control");

        let available_w = ui.available_width();
        let qr_panel_w = 140.0;
        let gap = 16.0;

        // Persistent QR code target: "mdns" or "ip"
        let qr_target_id = ui.make_persistent_id("settings_connections_qr_target");
        let mut qr_target: String = ui.ctx().data_mut(|d| {
            d.get_temp(qr_target_id).unwrap_or_else(|| {
                if automatic_available {
                    "mdns".to_string()
                } else {
                    "ip".to_string()
                }
            })
        });
        if !automatic_available && qr_target == "mdns" {
            qr_target = "ip".to_string();
        }

        let (selected_qr_url, selected_qr_label) = if qr_target == "mdns" && automatic_available {
            (automatic_url.as_str(), "mDNS")
        } else {
            (url.as_str(), "IP")
        };

        let render_endpoints = |ui: &mut egui::Ui| {
            if automatic_available {
                connection_endpoint_card(
                    ui,
                    theme,
                    language,
                    GLOBE,
                    language.text("discovery-title"),
                    Some("Recomendado"),
                    automatic_url,
                    true,
                );
                ui.add_space(8.0);
            }

            connection_endpoint_card(
                ui,
                theme,
                language,
                WIFI_HIGH,
                language.text("discovery-ip-alternative"),
                if !automatic_available && state.settings.allow_lan {
                    Some("Ativo")
                } else {
                    None
                },
                &url,
                !automatic_available,
            );

            ui.add_space(8.0);

            if automatic_available {
                ui.label(
                    RichText::new(language.text("discovery-host-hint"))
                        .size(10.5)
                        .color(theme.text_muted),
                );
            } else {
                let hint = if !state.settings.allow_lan {
                    language.text("discovery-lan-required").to_owned()
                } else if !state.settings_ui.discovery_error.is_empty() {
                    language.format(
                        "discovery-error",
                        &[("error", &state.settings_ui.discovery_error)],
                    )
                } else {
                    language.text("discovery-inactive").to_owned()
                };
                ui.label(RichText::new(hint).size(10.5).color(theme.text_muted));
            }
        };

        let render_qr_box = |ui: &mut egui::Ui, qr_target: &mut String| {
            egui::Frame::new()
                .fill(theme.bg_card)
                .corner_radius(10.0)
                .stroke(Stroke::new(1.0, theme.border_default))
                .inner_margin(egui::Margin::symmetric(10, 10))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        // Title
                        ui.label(
                            RichText::new(format!("{QR_CODE} Celular"))
                                .size(11.5)
                                .strong()
                                .color(theme.text_secondary),
                        );

                        // If both mDNS and IP are available, show mini tab switcher
                        if automatic_available {
                            ui.add_space(4.0);
                            let switch_rect = ui
                                .allocate_exact_size(egui::vec2(104.0, 24.0), egui::Sense::hover())
                                .0;
                            ui.painter().rect_filled(switch_rect, 6.0, theme.bg_pill);
                            ui.painter().rect_stroke(
                                switch_rect,
                                6.0,
                                Stroke::new(1.0, theme.border_subtle),
                                egui::StrokeKind::Inside,
                            );

                            let half_w = switch_rect.width() / 2.0;
                            let mdns_rect = egui::Rect::from_min_size(
                                switch_rect.min,
                                egui::vec2(half_w, switch_rect.height()),
                            );
                            let ip_rect = egui::Rect::from_min_size(
                                egui::pos2(switch_rect.min.x + half_w, switch_rect.min.y),
                                egui::vec2(half_w, switch_rect.height()),
                            );

                            let is_mdns = qr_target.as_str() == "mdns";

                            let mdns_resp = ui.interact(
                                mdns_rect,
                                ui.make_persistent_id("qr_tab_mdns"),
                                egui::Sense::click(),
                            );
                            let ip_resp = ui.interact(
                                ip_rect,
                                ui.make_persistent_id("qr_tab_ip"),
                                egui::Sense::click(),
                            );

                            if is_mdns {
                                ui.painter().rect_filled(mdns_rect, 5.0, theme.accent_soft);
                            } else {
                                ui.painter().rect_filled(ip_rect, 5.0, theme.accent_soft);
                            }

                            ui.painter().text(
                                mdns_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "mDNS",
                                egui::FontId::proportional(11.0),
                                if is_mdns {
                                    theme.accent
                                } else {
                                    theme.text_muted
                                },
                            );
                            ui.painter().text(
                                ip_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "IP",
                                egui::FontId::proportional(11.0),
                                if !is_mdns {
                                    theme.accent
                                } else {
                                    theme.text_muted
                                },
                            );

                            if mdns_resp.clicked() {
                                *qr_target = "mdns".to_string();
                                ui.ctx().data_mut(|d| {
                                    d.insert_temp(qr_target_id, qr_target.clone());
                                });
                            }
                            if ip_resp.clicked() {
                                *qr_target = "ip".to_string();
                                ui.ctx().data_mut(|d| {
                                    d.insert_temp(qr_target_id, qr_target.clone());
                                });
                            }
                        }

                        ui.add_space(6.0);

                        // QR Code inside white rounded card
                        let qr_size = 96.0;
                        let qr_frame = egui::Frame::new()
                            .fill(Color32::WHITE)
                            .corner_radius(8.0)
                            .inner_margin(6)
                            .stroke(Stroke::new(1.0, theme.border_default))
                            .show(ui, |ui| {
                                render_qr_code(ui, selected_qr_url, qr_size)
                            });

                        let qr_click = qr_frame
                            .response
                            .interact(egui::Sense::click())
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .on_hover_text("Clique para ampliar");

                        if qr_click.clicked() {
                            crate::ui::components::open_qr_dialog(
                                ui.ctx(),
                                format!("Controle Web ({selected_qr_label})"),
                                selected_qr_url,
                            );
                        }

                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("Escaneie para conectar")
                                .size(10.0)
                                .color(theme.text_muted),
                        );
                    });
                });
        };

        // If LAN is enabled and we have enough width, show 2-column side-by-side with exact top alignment
        if state.settings.allow_lan && available_w >= 520.0 {
            let left_w = (available_w - qr_panel_w - gap).max(300.0);
            let start_y = ui.cursor().top();
            let cursor_x = ui.cursor().min.x;

            let left_rect = egui::Rect::from_min_size(
                egui::pos2(cursor_x, start_y),
                egui::vec2(left_w, 600.0),
            );
            let right_rect = egui::Rect::from_min_size(
                egui::pos2(cursor_x + left_w + gap, start_y),
                egui::vec2(qr_panel_w, 600.0),
            );

            let mut left_ui = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt("connections_left_col")
                    .max_rect(left_rect),
            );
            left_ui.set_max_width(left_w);
            render_endpoints(&mut left_ui);
            let left_used = left_ui.min_rect();

            let mut right_ui = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt("connections_right_col")
                    .max_rect(right_rect),
            );
            right_ui.set_max_width(qr_panel_w);
            render_qr_box(&mut right_ui, &mut qr_target);
            let right_used = right_ui.min_rect();

            let total_h = left_used.height().max(right_used.height());
            ui.advance_cursor_after_rect(egui::Rect::from_min_size(
                egui::pos2(cursor_x, start_y),
                egui::vec2(available_w, total_h),
            ));
        } else {
            // Narrow screen or LAN disabled: Staged cleanly without breaking boundaries
            render_endpoints(ui);

            if state.settings.allow_lan {
                ui.add_space(10.0);
                ui.vertical_centered(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(qr_panel_w, 0.0),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            ui.set_max_width(qr_panel_w);
                            render_qr_box(ui, &mut qr_target);
                        },
                    );
                });
            }
        }
    });

    ui.add_space(12.0);

    // --- Card 2: Configuration Settings ---
    card(ui, theme, |ui| {
        setting_toggle(
            ui,
            &mut draft.server_enabled,
            "settings-browser-screens-http-server",
            "settings-provide-live-web-viewers-while-running",
            theme,
        );
        ui.separator();
        setting_toggle(
            ui,
            &mut draft.allow_lan,
            "settings-share-on-the-local-network-wi-fi-lan",
            "settings-allow-computers-tablets-phones-and-obs-on-the-same-network",
            theme,
        );
        ui.separator();

        // Server Port Configuration
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "settings-server-port"))
                        .size(12.5)
                        .strong()
                        .color(theme.text_primary),
                );
                ui.label(
                    RichText::new(crate::ui::i18n::text(
                        ui,
                        "settings-tcp-port-used-by-browsers-and-obs-studio",
                    ))
                    .size(11.0)
                    .color(theme.text_secondary),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_sized(
                    egui::vec2(78.0, 32.0),
                    egui::DragValue::new(&mut draft.server_port)
                        .range(1..=65535)
                        .speed(1),
                );

                ui.add_space(8.0);

                // Preset pills
                for preset in [8080, 8088, 5000, 3000] {
                    let is_sel = draft.server_port == preset;
                    let response = styled_button(
                        ui,
                        None,
                        &preset.to_string(),
                        if is_sel {
                            ButtonTone::Primary
                        } else {
                            ButtonTone::Secondary
                        },
                        theme,
                    );
                    if response.clicked() {
                        draft.server_port = preset;
                    }
                }
            });
        });

        ui.label(
            RichText::new(language.text("discovery-port-hint"))
                .size(10.5)
                .color(theme.text_muted),
        );

        ui.add_space(2.0);
        ui.separator();
        ui.add_space(2.0);

        // Detected LAN Address Box
        ui.horizontal(|ui| {
            let (icon_rect, _) =
                ui.allocate_exact_size(egui::vec2(26.0, 26.0), egui::Sense::hover());
            ui.painter().rect_filled(icon_rect, 6.0, theme.accent_soft);
            ui.painter().text(
                icon_rect.center(),
                egui::Align2::CENTER_CENTER,
                WIFI_HIGH,
                egui::FontId::proportional(14.0),
                theme.accent,
            );

            ui.vertical(|ui| {
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "settings-local-network-ip-address"))
                        .size(12.5)
                        .strong()
                        .color(theme.text_primary),
                );
                ui.label(
                    RichText::new(host)
                        .monospace()
                        .size(11.5)
                        .color(theme.text_secondary),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if styled_button(
                    ui,
                    Some(COPY),
                    crate::ui::i18n::text(ui, "settings-copy-ip"),
                    ButtonTone::Secondary,
                    theme,
                )
                .on_hover_text(crate::ui::i18n::text(
                    ui,
                    "settings-copy-ip-address-to-clipboard",
                ))
                .clicked()
                {
                    ui.ctx().copy_text(host.to_string());
                }
            });
        });
    });
}

fn connection_endpoint_card(
    ui: &mut egui::Ui,
    theme: &AppTheme,
    language: Language,
    icon: &str,
    title: &str,
    badge: Option<&str>,
    url: &str,
    is_primary: bool,
) {
    egui::Frame::new()
        .fill(theme.bg_card)
        .corner_radius(8.0)
        .stroke(Stroke::new(
            1.0,
            if is_primary {
                theme.accent.linear_multiply(0.3)
            } else {
                theme.border_default
            },
        ))
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            // Header line of the endpoint: Title + Badge
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{icon}  {title}"))
                        .size(11.5)
                        .strong()
                        .color(if is_primary {
                            theme.text_primary
                        } else {
                            theme.text_secondary
                        }),
                );

                if let Some(b) = badge {
                    egui::Frame::new()
                        .fill(theme.accent_soft)
                        .corner_radius(4.0)
                        .inner_margin(egui::Margin::symmetric(6, 2))
                        .stroke(Stroke::new(1.0, theme.accent.linear_multiply(0.25)))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(format!("{CHECK_CIRCLE} {b}"))
                                    .size(10.0)
                                    .color(theme.accent),
                            );
                        });
                }
            });

            ui.add_space(5.0);

            // Controls line: Monospace URL on left, action buttons on right
            let row_w = ui.available_width();
            if row_w >= 410.0 {
                ui.horizontal(|ui| {
                    egui::Frame::new()
                        .fill(theme.bg_pill)
                        .corner_radius(5.0)
                        .inner_margin(egui::Margin::symmetric(8, 5))
                        .stroke(Stroke::new(1.0, theme.border_subtle))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(url)
                                    .monospace()
                                    .size(11.5)
                                    .color(if is_primary {
                                        theme.accent
                                    } else {
                                        theme.text_secondary
                                    }),
                            );
                        });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // QR button
                        if styled_button(ui, Some(QR_CODE), "", ButtonTone::Secondary, theme)
                            .on_hover_text("Ampliar QR Code")
                            .clicked()
                        {
                            crate::ui::components::open_qr_dialog(
                                ui.ctx(),
                                format!("Controle Web ({title})"),
                                url,
                            );
                        }

                        // Copy button
                        if styled_button(ui, Some(COPY), "", ButtonTone::Secondary, theme)
                            .on_hover_text(language.text("control-copy-link"))
                            .clicked()
                        {
                            ui.ctx().copy_text(url.to_owned());
                        }

                        // Open button: Must contain language.text("control-open")
                        if styled_button(
                            ui,
                            Some(ARROW_SQUARE_OUT),
                            language.text("control-open"),
                            ButtonTone::Secondary,
                            theme,
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                        {
                            ui.ctx().open_url(egui::OpenUrl::new_tab(url.to_owned()));
                        }
                    });
                });
            } else {
                // Stacked layout for compact widths
                egui::Frame::new()
                    .fill(theme.bg_pill)
                    .corner_radius(5.0)
                    .inner_margin(egui::Margin::symmetric(8, 5))
                    .stroke(Stroke::new(1.0, theme.border_subtle))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(url)
                                .monospace()
                                .size(11.5)
                                .color(if is_primary {
                                    theme.accent
                                } else {
                                    theme.text_secondary
                                }),
                        );
                    });

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    if styled_button(
                        ui,
                        Some(ARROW_SQUARE_OUT),
                        language.text("control-open"),
                        ButtonTone::Secondary,
                        theme,
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                    {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(url.to_owned()));
                    }

                    if styled_button(ui, Some(COPY), "", ButtonTone::Secondary, theme)
                        .on_hover_text(language.text("control-copy-link"))
                        .clicked()
                    {
                        ui.ctx().copy_text(url.to_owned());
                    }

                    if styled_button(ui, Some(QR_CODE), "", ButtonTone::Secondary, theme)
                        .on_hover_text("Ampliar QR Code")
                        .clicked()
                    {
                        crate::ui::components::open_qr_dialog(
                            ui.ctx(),
                            format!("Controle Web ({title})"),
                            url,
                        );
                    }
                });
            }
        });
}
