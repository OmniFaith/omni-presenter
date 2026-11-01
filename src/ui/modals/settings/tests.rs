use super::components::setting_toggle;
use super::outputs::outputs;
use super::render_settings_modal;
use crate::core::state::{ControlPanelState, SettingsTab};
use crate::services::monitor::MonitorInfo;
use crate::ui::theme::AppTheme;
use eframe::egui;

#[test]
fn web_control_keeps_mdns_first_and_opens_its_advertised_url() {
    for port in [80, 5000] {
        let ctx = egui::Context::default();
        let mut state = ControlPanelState::default();
        state.settings.server_port = port;
        state.settings.allow_lan = true;
        state.settings_ui.lan_address = "192.168.1.10".into();
        state.settings_ui.discovery_url =
            crate::services::web::http_url("omni-presenter.local", port, "/control");
        let mut draft = state.settings.clone();
        // Unsaved connection settings must not change the running server's links.
        draft.server_port = 8080;
        draft.allow_lan = false;
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(900.0, 1000.0),
            )),
            ..Default::default()
        };
        let render = |ui: &mut egui::Ui, draft: &mut crate::core::preferences::AppSettings| {
            super::connections::connections(ui, draft, &state, &state.theme);
        };
        let mut output = ctx.run_ui(raw.clone(), |ui| render(ui, &mut draft));
        let position = output
            .shapes
            .iter()
            .find_map(|shape| {
                if let egui::Shape::Text(text) = &shape.shape
                    && (text.galley.job.text.contains("Abrir controle web") || text.galley.job.text.contains(draft.language.text("control-open")))
                {
                    Some(text.pos + text.galley.rect.center().to_vec2())
                } else {
                    None
                }
            })
            .expect("automatic discovery button should be visible first");
        output.textures_delta.clear();
        let mut click = raw;
        click.events.push(egui::Event::PointerMoved(position));
        for pressed in [true, false] {
            click.events.push(egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
        }
        let mut output = ctx.run_ui(click, |ui| render(ui, &mut draft));
        let opened: Vec<_> = output
            .platform_output
            .commands
            .iter()
            .filter_map(|command| {
                if let egui::OutputCommand::OpenUrl(url) = command {
                    Some(url.url.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(opened, [state.settings_ui.discovery_url.as_str()]);
        output.textures_delta.clear();
    }
}

#[test]
fn output_cards_select_detected_monitor_and_create_virtual_screen() {
    let ctx = egui::Context::default();
    let mut state = ControlPanelState::default();
    let mut draft = state.settings.clone();
    let monitors: Vec<_> = ["Notebook", "Projetor"]
        .into_iter()
        .enumerate()
        .map(|(index, name)| MonitorInfo {
            winit_index: index,
            name: name.into(),
            display_name: name.into(),
            resolution: (1920, 1080),
            position: (index as i32 * 1920, 0),
            scale_factor: 1.0,
        })
        .collect();
    draft.monitor_name = "Notebook".into();
    let theme = state.theme.clone();
    let raw = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(900.0, 900.0),
        )),
        ..Default::default()
    };
    let mut first = ctx.run_ui(raw.clone(), |ui| {
        outputs(ui, &mut draft, &mut state, &monitors, &theme, None)
    });
    let label_position = |output: &egui::FullOutput, label: &str| {
        output
            .shapes
            .iter()
            .find_map(|shape| {
                if let egui::Shape::Text(text) = &shape.shape {
                    if text.galley.job.text == label {
                        return Some(text.pos + text.galley.rect.center().to_vec2());
                    }
                }
                None
            })
            .unwrap()
    };
    let monitor_position = label_position(&first, "2");
    first.textures_delta.clear();
    let click = |position: egui::Pos2| {
        let mut input = raw.clone();
        input.events.push(egui::Event::PointerMoved(position));
        for pressed in [true, false] {
            input.events.push(egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
        }
        input
    };
    let mut second = ctx.run_ui(click(monitor_position), |ui| {
        outputs(ui, &mut draft, &mut state, &monitors, &theme, None)
    });
    assert!(!draft.physical_monitor_enabled("Projetor"));
    assert!(draft.physical_monitor_enabled("Notebook"));
    let add_position = label_position(&second, egui_phosphor::regular::PLUS);
    second.textures_delta.clear();
    let original_id = draft.screens[0].id.clone();
    let mut third = ctx.run_ui(click(add_position), |ui| {
        outputs(ui, &mut draft, &mut state, &monitors, &theme, None)
    });
    third.textures_delta.clear();
    assert_eq!(draft.screens.len(), 2);
    assert_eq!(draft.screens[0].id, original_id);
    assert_ne!(draft.screens[0].id, draft.screens[1].id);
}

#[test]
fn switch_rows_toggle_on_click() {
    let ctx = egui::Context::default();
    let theme = AppTheme::light();
    let mut value = false;
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(600.0, 200.0),
        )),
        ..Default::default()
    };
    let mut output = ctx.run_ui(input.clone(), |ui| {
        setting_toggle(
            ui,
            &mut value,
            "Compartilhar",
            "Visualizar na rede local.",
            &theme,
        )
    });
    output.textures_delta.clear();
    let mut clicked = input;
    let position = egui::pos2(28.0, 20.0);
    clicked.events.push(egui::Event::PointerMoved(position));
    for pressed in [true, false] {
        clicked.events.push(egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
    }
    let mut output = ctx.run_ui(clicked, |ui| {
        setting_toggle(
            ui,
            &mut value,
            "Compartilhar",
            "Visualizar na rede local.",
            &theme,
        )
    });
    output.textures_delta.clear();
    assert!(value);
}

#[test]
fn all_settings_tabs_render_in_both_themes() {
    for dark in [false, true] {
        let ctx = egui::Context::default();
        let mut state = ControlPanelState::default();
        state.settings.operators[0].dark_mode = dark;
        state.apply_profiles();
        state.settings_ui.open = true;
        for tab in [
            SettingsTab::Outputs,
            SettingsTab::Interface,
            SettingsTab::Shortcuts,
            SettingsTab::Plugins,
            SettingsTab::Bible,
            SettingsTab::Connections,
            SettingsTab::Profiles,
            SettingsTab::About,
        ] {
            state.settings_ui.tab = tab;
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(980.0, 600.0),
                )),
                ..Default::default()
            };
            let mut output = ctx.run_ui(input.clone(), |ui| {
                render_settings_modal(ui.ctx(), &mut state, &[], None)
            });
            output.textures_delta.clear();
            assert!(state.settings_ui.open);
            assert!(state.settings_ui.draft.is_some());
        }
    }
}

#[test]
fn escape_discards_draft_without_changing_preferences() {
    let ctx = egui::Context::default();
    let mut state = ControlPanelState::default();
    state.settings_ui.open = true;
    let raw = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1280.0, 760.0),
        )),
        ..Default::default()
    };
    let mut output = ctx.run_ui(raw.clone(), |ui| {
        render_settings_modal(ui.ctx(), &mut state, &[], None)
    });
    output.textures_delta.clear();
    state.settings_ui.draft.as_mut().unwrap().operators[0].dark_mode = true;
    let mut input = raw;
    input.events.push(egui::Event::Key {
        key: egui::Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    let mut output = ctx.run_ui(input.clone(), |ui| {
        render_settings_modal(ui.ctx(), &mut state, &[], None)
    });
    output.textures_delta.clear();
    assert!(!state.settings_ui.open);
    assert!(state.settings_ui.draft.is_none());
    assert!(!state.settings.operator().dark_mode);
}

#[test]
fn test_screen_card_and_floating_preview() {
    let ctx = egui::Context::default();
    let mut state = ControlPanelState::default();
    let monitors = vec![MonitorInfo {
        winit_index: 0,
        name: "Projetor Principal".into(),
        display_name: "Projetor Principal".into(),
        resolution: (1920, 1080),
        position: (0, 0),
        scale_factor: 1.0,
    }];
    let theme = AppTheme::dark();

    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(600.0, 400.0),
        )),
        ..Default::default()
    };
    let mut output = ctx.run_ui(input.clone(), |ui| {
        let (resp, action) = super::screen_card::render_screen_card(
            ui,
            super::screen_card::ScreenCardProps {
                id_salt: "test_card",
                label: "1",
                name: "Projetor",
                subtitle: Some("1920 × 1080"),
                selected: true,
                enabled: true,
                is_virtual: false,
                theme: &theme,
            },
        );
        assert_eq!(resp.rect.size(), egui::vec2(142.0, 88.0));
        assert_eq!(action, super::screen_card::ScreenCardAction::None);
    });
    output.textures_delta.clear();

    state.settings_ui.floating_preview = Some(crate::core::state::FloatingPreviewState {
        open: true,
        target: crate::core::state::FloatingPreviewTarget::Physical {
            monitor_name: "Projetor Principal".into(),
            index: 1,
        },
        aspect_ratio: 16.0 / 9.0,
        show_info: true,
    });

    let mut preview_out = ctx.run_ui(input, |ui| {
        super::floating_preview::render_floating_screen_preview(
            ui.ctx(),
            &mut state,
            &monitors,
            Some("Texto de teste da projeção"),
            Some("Hino 123"),
            Some("Próxima estrofe"),
            None,
        );
    });
    preview_out.textures_delta.clear();
    assert!(state.settings_ui.floating_preview.is_some());
}

#[test]
fn test_physical_background_per_monitor_and_recent_images() {
    use crate::core::preferences::{AppSettings, PhysicalBackground};

    let mut settings = AppSettings::default();
    assert_eq!(settings.wallpaper_for_monitor("HDMI-1"), &PhysicalBackground::Transparent);

    // Record recent wallpapers
    settings.record_recent_wallpaper("assets/test1.png");
    settings.record_recent_wallpaper("assets/test2.png");
    settings.record_recent_wallpaper("assets/test1.png");
    assert_eq!(settings.recent_wallpapers, vec!["assets/test1.png", "assets/test2.png"]);

    // Set custom wallpaper for HDMI-1
    *settings.wallpaper_for_monitor_mut("HDMI-1") = PhysicalBackground::Color { rgb: [255, 0, 0] };
    assert!(settings.has_monitor_wallpaper("HDMI-1"));
    assert_eq!(settings.wallpaper_for_monitor("HDMI-1"), &PhysicalBackground::Color { rgb: [255, 0, 0] });
    assert_eq!(settings.wallpaper_for_monitor("DP-1"), &PhysicalBackground::Transparent);

    let ctx = egui::Context::default();
    let theme = AppTheme::dark();
    let monitors = vec![
        MonitorInfo {
            winit_index: 0,
            name: "HDMI-1".into(),
            display_name: "Projetor 1".into(),
            resolution: (1920, 1080),
            position: (0, 0),
            scale_factor: 1.0,
        },
        MonitorInfo {
            winit_index: 1,
            name: "DP-1".into(),
            display_name: "Retorno 2".into(),
            resolution: (1920, 1080),
            position: (1920, 0),
            scale_factor: 1.0,
        },
    ];

    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800.0, 600.0),
        )),
        ..Default::default()
    };
    let mut output = ctx.run_ui(input.clone(), |ui| {
        super::physical_background::render(
            ui,
            &mut settings,
            &[],
            &theme,
            None,
            &monitors,
        );
    });
    output.textures_delta.clear();

    // Set the same wallpaper for two monitors
    let shared_bg = PhysicalBackground::Image { path: "assets/church_logo.png".into() };
    *settings.wallpaper_for_monitor_mut("HDMI-1") = shared_bg.clone();
    *settings.wallpaper_for_monitor_mut("DP-1") = shared_bg.clone();
    assert_eq!(settings.wallpaper_for_monitor("HDMI-1"), settings.wallpaper_for_monitor("DP-1"));
    assert_eq!(settings.wallpaper_for_monitor("HDMI-1"), &shared_bg);

    // Pre-populate target monitor in temp state
    let target_id = egui::Id::NULL.with("physical_wallpaper_target_monitor");
    ctx.data_mut(|d| d.insert_temp(target_id, "HDMI-1".to_string()));

    let mut output2 = ctx.run_ui(input, |ui| {
        super::physical_background::render(
            ui,
            &mut settings,
            &[],
            &theme,
            None,
            &monitors,
        );
    });
    output2.textures_delta.clear();

    settings.remove_monitor_wallpaper("HDMI-1");
    settings.remove_monitor_wallpaper("DP-1");
    assert!(!settings.has_monitor_wallpaper("HDMI-1"));
    assert!(!settings.has_monitor_wallpaper("DP-1"));
    assert_eq!(settings.wallpaper_for_monitor("HDMI-1"), &PhysicalBackground::Transparent);
}

#[test]
fn test_physical_wallpaper_with_many_monitors_scales_cleanly() {
    use crate::core::preferences::AppSettings;

    let ctx = egui::Context::default();
    let theme = AppTheme::dark();
    let mut settings = AppSettings::default();

    let mut monitors = Vec::new();
    for i in 1..=8 {
        monitors.push(MonitorInfo {
            winit_index: i - 1,
            name: format!("MON-{i}"),
            display_name: format!("Samsung Screen {i}"),
            resolution: (3840, 2160),
            position: ((i as i32 - 1) * 3840, 0),
            scale_factor: 1.0,
        });
    }

    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(900.0, 700.0),
        )),
        ..Default::default()
    };

    let target_id = egui::Id::NULL.with("physical_wallpaper_target_monitor");
    ctx.data_mut(|d| d.insert_temp(target_id, "MON-8".to_string()));

    let mut output = ctx.run_ui(input, |ui| {
        super::physical_background::render(
            ui,
            &mut settings,
            &[],
            &theme,
            None,
            &monitors,
        );
    });
    output.textures_delta.clear();
    assert_eq!(monitors.len(), 8);
}

#[test]
fn test_physical_wallpaper_multi_monitor_simultaneous_selection() {
    use crate::core::preferences::AppSettings;

    let ctx = egui::Context::default();
    let theme = AppTheme::dark();
    let mut settings = AppSettings::default();

    let monitors = vec![
        MonitorInfo {
            winit_index: 0,
            name: "MON-1".to_string(),
            display_name: "Monitor 1".to_string(),
            resolution: (1920, 1080),
            position: (0, 0),
            scale_factor: 1.0,
        },
        MonitorInfo {
            winit_index: 1,
            name: "MON-2".to_string(),
            display_name: "Monitor 2".to_string(),
            resolution: (1920, 1080),
            position: (1920, 0),
            scale_factor: 1.0,
        },
    ];

    let multi_id = egui::Id::NULL.with("physical_wallpaper_selected_monitors");
    ctx.data_mut(|d| d.insert_temp(multi_id, vec!["MON-1".to_string(), "MON-2".to_string()]));

    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(900.0, 700.0))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(input, |ui| {
        super::physical_background::render(
            ui,
            &mut settings,
            &[],
            &theme,
            None,
            &monitors,
        );
    });
    output.textures_delta.clear();

    let selected: Vec<String> = ctx.data_mut(|d| d.get_temp(multi_id)).unwrap_or_default();
    assert_eq!(selected, vec!["MON-1", "MON-2"]);
}

#[test]
fn test_virtual_screen_custom_background_and_presentation_wallpaper() {
    use crate::core::preferences::{AppSettings, PhysicalBackground, VirtualScreen};

    let mut settings = AppSettings::default();
    assert!(!settings.use_wallpaper_in_presentation);
    settings.use_wallpaper_in_presentation = true;
    assert!(settings.use_wallpaper_in_presentation);

    let mut virtual_screen = VirtualScreen {
        id: "stream".into(),
        name: "Transmissão".into(),
        enabled: true,
        custom_background: None,
        ..Default::default()
    };
    assert!(virtual_screen.custom_background.is_none());

    // Switch to custom wallpaper
    virtual_screen.custom_background = Some(PhysicalBackground::Color { rgb: [30, 40, 50] });
    assert!(virtual_screen.custom_background.is_some());

    let ctx = egui::Context::default();
    let theme = AppTheme::dark();
    let live_theme = crate::core::models::PresentationTheme::default();
    let mut recents = vec!["assets/church_logo.png".to_string()];

    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800.0, 600.0),
        )),
        ..Default::default()
    };
    let mut output = ctx.run_ui(input.clone(), |ui| {
        super::physical_background::render_virtual_screen_appearance(
            ui,
            &mut virtual_screen,
            &[],
            &live_theme,
            &theme,
            None,
            &mut recents,
            "obsidian_pro",
        );
    });
    output.textures_delta.clear();

    // Now test with custom_background = None (theme mode)
    virtual_screen.custom_background = None;
    let mut output2 = ctx.run_ui(input, |ui| {
        super::physical_background::render_virtual_screen_appearance(
            ui,
            &mut virtual_screen,
            &[],
            &live_theme,
            &theme,
            None,
            &mut recents,
            "obsidian_pro",
        );
    });
    output2.textures_delta.clear();
}
