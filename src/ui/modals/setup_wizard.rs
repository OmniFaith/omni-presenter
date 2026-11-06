use crate::core::i18n::Language;
use crate::core::interface_theme::ThemeMode;
use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use crate::services::monitor::MonitorInfo;
use crate::ui::modals::settings::components::{appearance_card, card, setting_toggle, styled_button, ButtonTone};
use crate::ui::modals::settings::screen_card::{render_screen_card, ScreenCardAction, ScreenCardProps};
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, RichText, Stroke};
use egui_phosphor::regular::{
    ARROW_LEFT, ARROW_RIGHT, CHECK, CHECK_CIRCLE, DESKTOP, GLOBE, MONITOR, PAINT_BRUSH,
    SPARKLE, TEXT_AA,
};

pub fn render_setup_wizard_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    monitors: &[MonitorInfo],
    db: Option<&AppDb>,
) -> Option<usize> {
    if !state.show_setup_wizard {
        return None;
    }

    let tokens = state.theme.clone();
    let language = state.settings.language;
    let mut monitor_target = None;

    let frame = egui::Frame::new()
        .fill(tokens.bg_panel)
        .stroke(Stroke::new(1.0, tokens.border_default))
        .corner_radius(16.0)
        .inner_margin(24)
        .shadow(egui::epaint::Shadow {
            offset: [0, 14],
            blur: 48,
            spread: 0,
            color: Color32::from_black_alpha(75),
        });

    egui::Modal::new(egui::Id::unique("initial_setup_wizard_modal"))
        .frame(frame)
        .backdrop_color(Color32::from_black_alpha(140))
        .show(ctx, |ui| {
            let max_w = 620.0_f32.min((ctx.content_rect().width() - 48.0).max(320.0));
            ui.set_width(max_w);

            let visuals = &mut ui.style_mut().visuals;
            visuals.widgets.inactive.weak_bg_fill = tokens.bg_pill;
            visuals.widgets.inactive.bg_fill = tokens.bg_pill;
            visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, tokens.border_default);
            visuals.widgets.hovered.weak_bg_fill = tokens.bg_card_hover;
            visuals.widgets.hovered.bg_fill = tokens.bg_card_hover;
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);

            // Centered Stepper Navigation Header
            render_stepper_header(ui, state, &tokens, language);
            ui.add_space(14.0);

            // Content Area depending on active step (centered & clean without nested card cascades)
            match state.setup_wizard.step {
                0 => render_step_welcome(ui, state, &tokens, language),
                1 => render_step_appearance(ui, state, &tokens, language),
                2 => render_step_screens(ui, state, monitors, &tokens, language),
                3 => render_step_breaks(ui, state, &tokens, language),
                _ => render_step_ready(ui, state, monitors, &tokens, language),
            }

            ui.add_space(18.0);

            // Bottom Navigation Footer
            render_wizard_footer(ui, state, db, &mut monitor_target, &tokens, language);
        });

    monitor_target
}

fn render_stepper_header(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
    language: Language,
) {
    let steps = [
        (0, SPARKLE, language.text("settings-wizard-step-welcome")),
        (1, PAINT_BRUSH, language.text("settings-wizard-step-appearance")),
        (2, DESKTOP, language.text("settings-wizard-step-monitors")),
        (3, TEXT_AA, language.text("settings-wizard-step-breaks")),
        (4, CHECK_CIRCLE, language.text("settings-wizard-step-ready")),
    ];

    let gap = 8.0;
    let pill_widths: Vec<f32> = steps
        .iter()
        .map(|(_, icon, label)| {
            let text = format!("{icon} {label}");
            let text_layout = ui.painter().layout_no_wrap(
                text,
                egui::FontId::proportional(11.0),
                tokens.text_primary,
            );
            (text_layout.size().x + 20.0).max(64.0)
        })
        .collect();

    let total_w: f32 = pill_widths.iter().sum::<f32>() + gap * (steps.len() - 1) as f32;
    let offset_x = ((ui.available_width() - total_w) * 0.5).max(0.0);

    ui.horizontal(|ui| {
        if offset_x > 0.0 {
            ui.add_space(offset_x);
        }
        let current = state.setup_wizard.step;
        for (idx, (step_idx, icon, label)) in steps.iter().enumerate() {
            let is_active = *step_idx == current;
            let is_past = *step_idx < current;

            let (bg_color, text_color, border_color) = if is_active {
                (tokens.accent, Color32::WHITE, tokens.accent)
            } else if is_past {
                (tokens.accent_soft, tokens.accent, tokens.accent.linear_multiply(0.4))
            } else {
                (tokens.bg_pill, tokens.text_muted, tokens.border_default)
            };

            let text = format!("{icon} {label}");
            let pill_w = pill_widths[idx];
            let (rect, response) = ui.allocate_exact_size(egui::vec2(pill_w, 28.0), egui::Sense::click());

            if response.hovered() && !is_active {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if response.clicked() {
                state.setup_wizard.step = *step_idx;
            }

            ui.painter().rect(
                rect,
                14.0,
                if response.hovered() && !is_active { tokens.bg_card_hover } else { bg_color },
                Stroke::new(1.0, border_color),
                egui::StrokeKind::Inside,
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(11.0),
                text_color,
            );
            if idx + 1 < steps.len() {
                ui.add_space(gap);
            }
        }
    });
}

fn render_step_welcome(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
    language: Language,
) {
    ui.vertical_centered(|ui| {
        ui.add_space(10.0);
        let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(56.0, 56.0), egui::Sense::hover());
        ui.painter().rect_filled(icon_rect, 14.0, tokens.accent_soft);
        ui.painter().text(
            icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            SPARKLE,
            egui::FontId::proportional(30.0),
            tokens.accent,
        );
        ui.add_space(8.0);
        ui.label(
            RichText::new(language.text("settings-wizard-welcome"))
                .size(22.0)
                .strong()
                .color(tokens.text_primary),
        );
        ui.add_space(2.0);
        ui.label(
            RichText::new(language.text("settings-wizard-welcome-desc"))
                .size(13.0)
                .color(tokens.text_secondary),
        );
        ui.add_space(22.0);

        ui.label(
            RichText::new(format!("{GLOBE}  {}", language.text("settings-language")))
                .size(13.0)
                .strong()
                .color(tokens.text_primary),
        );
        ui.add_space(10.0);

        let languages = [
            (Language::Portuguese, "Português (Brasil)", "🇧🇷"),
            (Language::English, "English", "🇺🇸"),
            (Language::Spanish, "Español", "🇪🇸"),
        ];

        let gap = 12.0;
        let total_avail = ui.available_width();
        let item_w = ((total_avail - gap * (languages.len() - 1) as f32) / languages.len() as f32).max(110.0);

        ui.horizontal(|ui| {
            for (idx, (lang_opt, name, flag)) in languages.iter().enumerate() {
                let selected = state.settings.language == *lang_opt;
                let (rect, response) = ui.allocate_exact_size(egui::vec2(item_w, 48.0), egui::Sense::click());
                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if response.clicked() {
                    state.settings.language = *lang_opt;
                }

                let fill = if selected {
                    tokens.accent_soft
                } else if response.hovered() {
                    tokens.bg_card_hover
                } else {
                    tokens.bg_pill
                };

                let stroke = if selected {
                    Stroke::new(1.5, tokens.accent)
                } else {
                    Stroke::new(1.0, tokens.border_default)
                };

                ui.painter().rect(rect, 10.0, fill, stroke, egui::StrokeKind::Inside);
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{flag}  {name}"),
                    egui::FontId::proportional(13.0),
                    if selected { tokens.accent } else { tokens.text_primary },
                );

                if idx + 1 < languages.len() {
                    ui.add_space(gap);
                }
            }
        });

        ui.add_space(14.0);
    });
}

fn render_step_appearance(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
    language: Language,
) {
    ui.vertical_centered(|ui| {
        let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::hover());
        ui.painter().rect_filled(icon_rect, 10.0, tokens.accent_soft);
        ui.painter().text(
            icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            PAINT_BRUSH,
            egui::FontId::proportional(22.0),
            tokens.accent,
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new(language.text("appearance-title"))
                .size(19.0)
                .strong()
                .color(tokens.text_primary),
        );
        ui.label(
            RichText::new(language.text("appearance-hint"))
                .size(12.0)
                .color(tokens.text_secondary),
        );
        ui.add_space(14.0);

        // Centered 3 equal-width mode choices
        let operator = state.settings.operator_mut();
        let selected_mode = if operator.theme_mode == ThemeMode::Legacy {
            if operator.dark_mode { ThemeMode::Dark } else { ThemeMode::Light }
        } else {
            operator.theme_mode
        };

        let modes = [
            (ThemeMode::System, "appearance-system", "💻"),
            (ThemeMode::Light, "appearance-light", "☀️"),
            (ThemeMode::Dark, "appearance-dark", "🌙"),
        ];

        let gap = 12.0;
        let total_avail = ui.available_width();
        let btn_w = ((total_avail - gap * (modes.len() - 1) as f32) / modes.len() as f32).max(100.0);

        ui.horizontal(|ui| {
            for (idx, (mode, label_key, icon)) in modes.iter().enumerate() {
                let is_sel = selected_mode == *mode;
                let (rect, response) = ui.allocate_exact_size(egui::vec2(btn_w, 42.0), egui::Sense::click());
                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if response.clicked() {
                    let op = state.settings.operator_mut();
                    op.theme_mode = *mode;
                    if *mode != ThemeMode::System {
                        op.dark_mode = *mode == ThemeMode::Dark;
                    }
                    state.apply_profiles();
                }

                let fill = if is_sel {
                    tokens.accent_soft
                } else if response.hovered() {
                    tokens.bg_card_hover
                } else {
                    tokens.bg_pill
                };

                let stroke = if is_sel {
                    Stroke::new(1.5, tokens.accent)
                } else {
                    Stroke::new(1.0, tokens.border_default)
                };

                ui.painter().rect(rect, 8.0, fill, stroke, egui::StrokeKind::Inside);
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{icon}  {}", language.text(label_key)),
                    egui::FontId::proportional(12.5),
                    if is_sel { tokens.accent } else { tokens.text_primary },
                );

                if idx + 1 < modes.len() {
                    ui.add_space(gap);
                }
            }
        });
    });

    ui.add_space(16.0);

    // Palettes section
    card(ui, tokens, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(language.text("settings-palette"))
                    .size(12.5)
                    .strong()
                    .color(tokens.text_primary),
            );
        });
        ui.add_space(6.0);

        let mut catalogue = crate::core::interface_theme::builtins();
        catalogue.extend(state.settings.interface_themes.clone());
        for plugin in state.settings.plugins.iter().filter(|p| p.enabled) {
            catalogue.extend(plugin.package.interface_themes.clone());
        }

        let selected = state.settings.operator().interface_theme_id.clone();
        let mut choice = selected.clone();
        let dark = state.settings.operator().theme_mode.is_dark(
            ui.ctx().system_theme() == Some(egui::Theme::Dark),
            state.settings.operator().dark_mode,
        );

        let gap = 8.0;
        let available = ui.available_width();
        let columns = ((available + gap) / 140.0).floor().clamp(1.0, 4.0) as usize;
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
                            definition.localized_name_for_mode(language, dark),
                            choice == definition.id,
                            width,
                            tokens,
                        ) {
                            choice = definition.id.clone();
                        }
                    });
                }
            });
            ui.add_space(gap);
        }

        if choice != selected {
            state.settings.operator_mut().interface_theme_id = choice;
            state.apply_profiles();
        }
    });
}

fn render_step_screens(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    monitors: &[MonitorInfo],
    tokens: &AppTheme,
    language: Language,
) {
    ui.vertical_centered(|ui| {
        let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::hover());
        ui.painter().rect_filled(icon_rect, 10.0, tokens.accent_soft);
        ui.painter().text(
            icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            DESKTOP,
            egui::FontId::proportional(22.0),
            tokens.accent,
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new(language.text("settings-presentation-screens"))
                .size(19.0)
                .strong()
                .color(tokens.text_primary),
        );
        ui.label(
            RichText::new(language.text("settings-send-the-presentation-to-all-enabled-screens-above-click-a"))
                .size(12.0)
                .color(tokens.text_secondary),
        );
        ui.add_space(10.0);
    });

    card(ui, tokens, |ui| {
        if monitors.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(6.0);
                ui.label(
                    RichText::new(language.text("workspace-no-monitors-detected"))
                        .color(tokens.text_secondary)
                        .size(12.0),
                );
                ui.add_space(6.0);
            });
        } else {
            egui::ScrollArea::horizontal()
                .id_salt("wizard_physical_screens")
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 10.0;
                        for (index, monitor) in monitors.iter().enumerate() {
                            let enabled = state.settings.physical_monitor_enabled(&monitor.name);
                            let label = (index + 1).to_string();
                            let res_str = format!("{} × {}", monitor.resolution.0, monitor.resolution.1);

                            let (_resp, action) = render_screen_card(
                                ui,
                                ScreenCardProps {
                                    id_salt: &format!("wiz_screen_{}", monitor.name),
                                    label: &label,
                                    name: &monitor.display_name,
                                    subtitle: Some(&res_str),
                                    selected: enabled,
                                    enabled: enabled && state.settings.physical_output_enabled,
                                    is_virtual: false,
                                    theme: tokens,
                                },
                            );

                            if matches!(action, ScreenCardAction::Select | ScreenCardAction::ToggleEnabled) {
                                state.settings.toggle_physical_monitor(&monitor.name);
                                if state.settings.physical_monitor_enabled(&monitor.name) {
                                    state.settings.monitor_name = monitor.name.clone();
                                }
                            }
                        }
                    });
                });
        }

        ui.add_space(6.0);
        setting_toggle(
            ui,
            &mut state.settings.physical_output_enabled,
            "workspace-open-the-projection-window-on-the-monitor",
            "settings-send-the-presentation-to-all-enabled-screens-above-click-a",
            tokens,
        );
        setting_toggle(
            ui,
            &mut state.settings.persistent_physical_output,
            "settings-keep-the-presentation-window-ready",
            "settings-keep-the-output-ready-show-the-standby-wallpaper-when-no",
            tokens,
        );
    });

    ui.add_space(8.0);

    // Aspect ratio selection - Centered horizontal row
    card(ui, tokens, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{MONITOR}  {}", language.text("settings-screen-aspect-ratio")))
                    .size(12.5)
                    .strong()
                    .color(tokens.text_primary),
            );
        });
        ui.add_space(6.0);

        #[derive(Clone, Copy, PartialEq)]
        enum AspectOption {
            Wide16_9,
            Wide16_10,
            Standard4_3,
            Ultra21_9,
        }

        let current_aspect = state.setup_wizard.presentation_aspect_ratio;
        let selected_opt = if (current_aspect - 16.0 / 9.0).abs() < 0.05 {
            AspectOption::Wide16_9
        } else if (current_aspect - 16.0 / 10.0).abs() < 0.05 {
            AspectOption::Wide16_10
        } else if (current_aspect - 4.0 / 3.0).abs() < 0.05 {
            AspectOption::Standard4_3
        } else if (current_aspect - 21.0 / 9.0).abs() < 0.05 {
            AspectOption::Ultra21_9
        } else {
            AspectOption::Wide16_9
        };

        let aspect_list = [
            (AspectOption::Wide16_9, "16:9 Widescreen", 16.0 / 9.0),
            (AspectOption::Wide16_10, "16:10", 16.0 / 10.0),
            (AspectOption::Standard4_3, "4:3 Clássico", 4.0 / 3.0),
            (AspectOption::Ultra21_9, "21:9 Ultrawide", 21.0 / 9.0),
        ];

        let gap = 8.0;
        let total_avail = ui.available_width();
        let btn_w = ((total_avail - gap * (aspect_list.len() - 1) as f32) / aspect_list.len() as f32).max(80.0);

        ui.horizontal(|ui| {
            for (idx, (opt, label, ratio_val)) in aspect_list.iter().enumerate() {
                let is_sel = selected_opt == *opt;
                let (rect, response) = ui.allocate_exact_size(egui::vec2(btn_w, 36.0), egui::Sense::click());
                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if response.clicked() {
                    state.setup_wizard.presentation_aspect_ratio = *ratio_val;
                    state.presentation_aspect_ratio = *ratio_val;
                }

                let fill = if is_sel {
                    tokens.accent_soft
                } else if response.hovered() {
                    tokens.bg_card_hover
                } else {
                    tokens.bg_pill
                };

                let stroke = if is_sel {
                    Stroke::new(1.5, tokens.accent)
                } else {
                    Stroke::new(1.0, tokens.border_default)
                };

                ui.painter().rect(rect, 8.0, fill, stroke, egui::StrokeKind::Inside);
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    *label,
                    egui::FontId::proportional(11.5),
                    if is_sel { tokens.accent } else { tokens.text_primary },
                );

                if idx + 1 < aspect_list.len() {
                    ui.add_space(gap);
                }
            }
        });
    });
}

fn render_step_breaks(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
    language: Language,
) {
    ui.vertical_centered(|ui| {
        let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::hover());
        ui.painter().rect_filled(icon_rect, 10.0, tokens.accent_soft);
        ui.painter().text(
            icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            TEXT_AA,
            egui::FontId::proportional(22.0),
            tokens.accent,
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new(language.text("settings-wizard-step-breaks"))
                .size(19.0)
                .strong()
                .color(tokens.text_primary),
        );
        ui.label(
            RichText::new("Ajuste a formatação automática e quebras de linhas nas letras e versículos.")
                .size(12.0)
                .color(tokens.text_secondary),
        );
        ui.add_space(10.0);
    });

    card(ui, tokens, |ui| {
        setting_toggle(
            ui,
            &mut state.setup_wizard.wrap_text,
            "settings-wizard-auto-wrap",
            "settings-wizard-auto-wrap-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.setup_wizard.auto_fit,
            "settings-wizard-auto-fit",
            "settings-wizard-auto-fit-desc",
            tokens,
        );

        setting_toggle(
            ui,
            &mut state.setup_wizard.dock_preview_bg,
            "settings-thumbnails-with-theme-background",
            "settings-show-the-theme-background-colors-and-fonts-in-stanza-cards",
            tokens,
        );
    });

    ui.add_space(8.0);

    // Centered guide for smart breaks syntax
    card(ui, tokens, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Quebras Inteligentes & Comentários no Editor")
                    .size(12.5)
                    .strong()
                    .color(tokens.text_primary),
            );
        });
        ui.add_space(6.0);

        let guide_items = [
            ("|", "Quebra Suave", "Quebra equilibrada no verso"),
            ("|16:9| / |4:3|", "Quebra por Formato", "Apenas na tela correspondente"),
            ("// nota", "Comentário", "Oculto na projeção"),
            ("---", "Novo Slide", "Divide em novo slide"),
        ];

        let gap = 8.0;
        let total_avail = ui.available_width();
        let col_w = ((total_avail - gap * (guide_items.len() - 1) as f32) / guide_items.len() as f32).max(70.0);

        ui.horizontal(|ui| {
            for (idx, (syntax, title, desc)) in guide_items.iter().enumerate() {
                egui::Frame::new()
                    .fill(tokens.bg_pill)
                    .corner_radius(8.0)
                    .inner_margin(egui::vec2(8.0, 8.0))
                    .show(ui, |ui| {
                        ui.set_width(col_w - 16.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(*syntax).strong().size(12.0).color(tokens.accent));
                            ui.label(RichText::new(*title).strong().size(10.5).color(tokens.text_primary));
                            ui.label(RichText::new(*desc).size(9.5).color(tokens.text_secondary));
                        });
                    });

                if idx + 1 < guide_items.len() {
                    ui.add_space(gap);
                }
            }
        });
    });
}

fn render_step_ready(
    ui: &mut egui::Ui,
    state: &ControlPanelState,
    monitors: &[MonitorInfo],
    tokens: &AppTheme,
    language: Language,
) {
    ui.vertical_centered(|ui| {
        ui.add_space(6.0);
        let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(52.0, 52.0), egui::Sense::hover());
        ui.painter().rect_filled(icon_rect, 13.0, tokens.status_success_bg);
        ui.painter().text(
            icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            CHECK,
            egui::FontId::proportional(28.0),
            tokens.status_success_fg,
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new(language.text("settings-wizard-ready-title"))
                .size(21.0)
                .strong()
                .color(tokens.text_primary),
        );
        ui.add_space(2.0);
        ui.label(
            RichText::new(language.text("settings-wizard-ready-desc"))
                .size(12.5)
                .color(tokens.text_secondary),
        );
        ui.add_space(14.0);
    });

    // Clean, structured summary card
    card(ui, tokens, |ui| {
        let lang_name = match state.settings.language {
            Language::Portuguese => "Português 🇧🇷",
            Language::English => "English 🇺🇸",
            Language::Spanish => "Español 🇪🇸",
        };

        let screen_name = if monitors.is_empty() {
            "Monitor Principal".to_string()
        } else if let Some(first) = monitors.iter().find(|m| state.settings.physical_monitor_enabled(&m.name)) {
            first.display_name.clone()
        } else {
            "Projeção em Janela".to_string()
        };

        let aspect_name = if (state.setup_wizard.presentation_aspect_ratio - 16.0 / 9.0).abs() < 0.05 {
            "16:9 Widescreen"
        } else if (state.setup_wizard.presentation_aspect_ratio - 16.0 / 10.0).abs() < 0.05 {
            "16:10"
        } else if (state.setup_wizard.presentation_aspect_ratio - 4.0 / 3.0).abs() < 0.05 {
            "4:3 Clássico"
        } else if (state.setup_wizard.presentation_aspect_ratio - 21.0 / 9.0).abs() < 0.05 {
            "21:9 Ultrawide"
        } else {
            "16:9 Widescreen"
        };

        let rows = [
            ("🌐  Idioma da Interface", lang_name),
            ("🖥️  Saída de Projeção", screen_name.as_str()),
            ("📐  Proporção da Tela", aspect_name),
            ("🔄  Quebra Automática de Linhas", if state.setup_wizard.wrap_text { "Ativada" } else { "Desativada" }),
            ("🔍  Ajuste Automático (Auto-fit)", if state.setup_wizard.auto_fit { "Ativado" } else { "Desativado" }),
        ];

        for (idx, (label, val)) in rows.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(RichText::new(*label).size(12.0).color(tokens.text_secondary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::Frame::new()
                        .fill(tokens.bg_pill)
                        .corner_radius(6.0)
                        .inner_margin(egui::vec2(10.0, 3.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(*val).size(12.0).strong().color(tokens.text_primary));
                        });
                });
            });
            if idx + 1 < rows.len() {
                ui.add_space(4.0);
            }
        }
    });

    ui.add_space(10.0);
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new("Dica: você pode alterar todas as opções acima a qualquer momento nas Preferências do sistema.")
                .size(11.5)
                .color(tokens.text_secondary),
        );
    });
}

fn render_wizard_footer(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
    monitor_target: &mut Option<usize>,
    tokens: &AppTheme,
    _language: Language,
) {
    ui.horizontal(|ui| {
        // Skip Button on the left
        if styled_button(
            ui,
            None,
            "settings-wizard-skip",
            ButtonTone::Ghost,
            tokens,
        )
        .clicked()
        {
            *monitor_target = finish_wizard(state, db);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let step = state.setup_wizard.step;
            let is_last = step >= 4;

            if is_last {
                if styled_button(
                    ui,
                    Some(CHECK),
                    "settings-wizard-finish",
                    ButtonTone::Primary,
                    tokens,
                )
                .clicked()
                {
                    *monitor_target = finish_wizard(state, db);
                }
            } else {
                if styled_button(
                    ui,
                    Some(ARROW_RIGHT),
                    "settings-wizard-next",
                    ButtonTone::Primary,
                    tokens,
                )
                .clicked()
                {
                    state.setup_wizard.step += 1;
                }
            }

            if step > 0 {
                if styled_button(
                    ui,
                    Some(ARROW_LEFT),
                    "settings-wizard-back",
                    ButtonTone::Secondary,
                    tokens,
                )
                .clicked()
                {
                    state.setup_wizard.step -= 1;
                }
            }
        });
    });
}

fn finish_wizard(
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
) -> Option<usize> {
    state.settings.first_run_completed = true;
    state.show_setup_wizard = false;
    state.presentation_aspect_ratio = state.setup_wizard.presentation_aspect_ratio;
    state.dock_preview_bg = state.setup_wizard.dock_preview_bg;

    // Apply wrap_text and auto_fit to all presentation themes
    for theme in &mut state.themes {
        theme.options.wrap_text = state.setup_wizard.wrap_text;
        theme.options.auto_fit = state.setup_wizard.auto_fit;
        if let Some(db) = db {
            let _ = db.save_theme(theme);
        }
    }

    if let Some(db) = db {
        let _ = db.save_settings(&state.settings);
    }
    state.apply_profiles();
    state.apply_theme_to_live();

    let lang = state.settings.language;
    state.notify(lang.text("settings-wizard-ready-title").to_string());

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_wizard_lifecycle_and_steps_rendering() {
        let mut state = ControlPanelState {
            show_setup_wizard: true,
            ..Default::default()
        };
        assert!(!state.settings.first_run_completed);

        let ctx = egui::Context::default();
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(1024.0, 768.0),
            )),
            ..Default::default()
        };

        let monitors = vec![
            MonitorInfo {
                winit_index: 0,
                name: "HDMI-1".to_string(),
                display_name: "Samsung 4K".to_string(),
                resolution: (3840, 2160),
                position: (0, 0),
                scale_factor: 1.0,
            },
            MonitorInfo {
                winit_index: 1,
                name: "DP-1".to_string(),
                display_name: "LG Projector".to_string(),
                resolution: (1920, 1080),
                position: (3840, 0),
                scale_factor: 1.0,
            },
        ];

        // Verify each step renders without panic
        for step in 0..=4 {
            state.setup_wizard.step = step;
            let mut output = ctx.run_ui(raw_input.clone(), |ui| {
                render_setup_wizard_modal(ui.ctx(), &mut state, &monitors, None);
            });
            output.textures_delta.clear();
            assert!(!output.shapes.is_empty(), "Step {step} should produce shapes");
        }

        // Test finish_wizard
        state.setup_wizard.wrap_text = true;
        state.setup_wizard.auto_fit = true;
        finish_wizard(&mut state, None);

        assert!(state.settings.first_run_completed);
        assert!(!state.show_setup_wizard);
        for theme in &state.themes {
            assert!(theme.options.wrap_text);
            assert!(theme.options.auto_fit);
        }
    }
}
