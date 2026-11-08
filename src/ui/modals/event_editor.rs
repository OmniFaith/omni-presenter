use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use crate::ui::components::dialog;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, RichText, Stroke};
use egui_phosphor::regular::*;

pub fn render_event_editor_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    _db: Option<&AppDb>,
) {
    if !state.show_event_editor_modal {
        return;
    }

    let Some(ref mut draft) = state.event_editor_draft else {
        state.show_event_editor_modal = false;
        return;
    };

    let theme = state.theme.clone();
    let is_editing = draft.id.is_some();
    let title_key = if is_editing {
        "workspace-edit-event"
    } else {
        "workspace-new-event"
    };

    let mut should_save = false;
    let mut should_cancel = false;

    let close = dialog::show(
        ctx,
        "event_editor_modal",
        title_key,
        "workspace-event-settings-desc",
        510.0,
        &theme,
        |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);

            // Nome do Evento
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "workspace-event-name"))
                        .size(12.0)
                        .strong()
                        .color(theme.text_primary),
                );
                ui.add_space(2.0);
                let text_res = ui.add(
                    egui::TextEdit::singleline(&mut draft.name)
                        .hint_text(crate::ui::i18n::text(ui, "workspace-event-name-placeholder"))
                        .desired_width(ui.available_width()),
                );
                if draft.name.is_empty() && !is_editing {
                    text_res.request_focus();
                }
            });

            ui.add_space(4.0);

            // Tipo de Recorrência
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "workspace-recurrence"))
                        .size(12.0)
                        .strong()
                        .color(theme.text_primary),
                );
                ui.add_space(2.0);

                let patterns = [
                    ("none", "workspace-recurrence-none"),
                    ("weekly", "workspace-recurrence-weekly"),
                    ("monthly", "workspace-recurrence-monthly"),
                    ("daily", "workspace-recurrence-daily"),
                ];

                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 4.0);
                    for (pat, label_key) in patterns {
                        let selected = draft.pattern == pat;
                        let btn = egui::Button::new(
                            RichText::new(crate::ui::i18n::text(ui, label_key))
                                .size(11.5)
                                .color(if selected {
                                    theme.accent
                                } else {
                                    theme.text_secondary
                                }),
                        )
                        .fill(if selected {
                            theme.accent_soft
                        } else {
                            theme.bg_pill
                        })
                        .stroke(Stroke::new(
                            1.0,
                            if selected {
                                theme.accent
                            } else {
                                theme.border_default
                            },
                        ))
                        .corner_radius(6.0)
                        .min_size(egui::vec2(0.0, 24.0));

                        if ui.add(btn).clicked() {
                            draft.pattern = pat.to_string();
                        }
                    }
                });
            });

            // Opções específicas da recorrência
            match draft.pattern.as_str() {
                "none" => {
                    ui.add_space(2.0);
                    if draft.specific_dates.is_empty() {
                        if !draft.date.trim().is_empty() {
                            draft.specific_dates.push(draft.date.trim().to_string());
                        } else {
                            draft.specific_dates.push(String::new());
                        }
                    }

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(crate::ui::i18n::text(ui, "workspace-event-date"))
                                .size(11.5)
                                .strong()
                                .color(theme.text_secondary),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button(
                                    RichText::new(crate::ui::i18n::text(
                                        ui,
                                        "workspace-sequential-days",
                                    ))
                                    .size(10.5),
                                )
                                .clicked()
                            {
                                let last_val =
                                    draft.specific_dates.last().cloned().unwrap_or_default();
                                let next_d = add_day_to_date_str(&last_val);
                                draft.specific_dates.push(next_d);
                            }
                            if ui
                                .button(
                                    RichText::new(crate::ui::i18n::text(
                                        ui,
                                        "workspace-add-date",
                                    ))
                                    .size(10.5),
                                )
                                .clicked()
                            {
                                draft.specific_dates.push(String::new());
                            }
                        });
                    });

                    ui.add_space(2.0);

                    let mut to_remove = None;
                    let dates_count = draft.specific_dates.len();
                    for (idx, date_val) in draft.specific_dates.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("{}.", idx + 1))
                                    .size(11.0)
                                    .color(theme.text_muted),
                            );
                            ui.add(
                                egui::TextEdit::singleline(date_val)
                                    .hint_text("Ex: 15/11/2026")
                                    .desired_width(140.0),
                            );
                            if dates_count > 1 {
                                if ui
                                    .button(
                                        RichText::new(TRASH)
                                            .size(11.0)
                                            .color(theme.status_danger),
                                    )
                                    .clicked()
                                {
                                    to_remove = Some(idx);
                                }
                            }
                        });
                    }
                    if let Some(idx) = to_remove {
                        draft.specific_dates.remove(idx);
                    }
                    if let Some(first) = draft.specific_dates.first() {
                        draft.date = first.clone();
                    }

                    ui.add_space(4.0);
                    render_time_picker(ui, &mut draft.time, &mut draft.all_day, &theme);
                }
                "weekly" => {
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(crate::ui::i18n::text(ui, "workspace-days-of-week"))
                            .size(11.5)
                            .color(theme.text_secondary),
                    );
                    render_weekdays_selector(ui, &mut draft.days_of_week, &theme);
                    ui.add_space(4.0);
                    render_time_picker(ui, &mut draft.time, &mut draft.all_day, &theme);
                }
                "monthly" => {
                    // 1. Semana do mês (ocorrência)
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(crate::ui::i18n::text(ui, "workspace-week-of-month"))
                            .size(11.5)
                            .color(theme.text_secondary),
                    );
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                        let week_options = [
                            (0u8, "workspace-week-any"),
                            (1u8, "workspace-week-1"),
                            (2u8, "workspace-week-2"),
                            (3u8, "workspace-week-3"),
                            (4u8, "workspace-week-4"),
                            (5u8, "workspace-week-last"),
                        ];
                        for (w_val, label_key) in week_options {
                            let is_sel = draft.week_of_month == w_val;
                            let btn = egui::Button::new(
                                RichText::new(crate::ui::i18n::text(ui, label_key))
                                    .size(11.0)
                                    .color(if is_sel {
                                        theme.accent
                                    } else {
                                        theme.text_secondary
                                    }),
                            )
                            .fill(if is_sel {
                                theme.accent_soft
                            } else {
                                theme.bg_pill
                            })
                            .stroke(Stroke::new(
                                1.0,
                                if is_sel {
                                    theme.accent
                                } else {
                                    theme.border_default
                                },
                            ))
                            .corner_radius(4.0);
                            if ui.add(btn).clicked() {
                                draft.week_of_month = w_val;
                            }
                        }
                    });

                    // 2. Dias da semana
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(crate::ui::i18n::text(ui, "workspace-days-of-week"))
                            .size(11.5)
                            .color(theme.text_secondary),
                    );
                    render_weekdays_selector(ui, &mut draft.days_of_week, &theme);

                    // 3. Meses do ano
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(crate::ui::i18n::text(ui, "workspace-months"))
                                .size(11.5)
                                .color(theme.text_secondary),
                        );
                        let all_months_sel = draft.months.is_empty();
                        let all_btn = egui::Button::new(
                            RichText::new(crate::ui::i18n::text(ui, "workspace-all-months"))
                                .size(10.5)
                                .color(if all_months_sel {
                                    theme.accent
                                } else {
                                    theme.text_secondary
                                }),
                        )
                        .fill(if all_months_sel {
                            theme.accent_soft
                        } else {
                            theme.bg_pill
                        })
                        .stroke(Stroke::new(
                            1.0,
                            if all_months_sel {
                                theme.accent
                            } else {
                                theme.border_default
                            },
                        ))
                        .corner_radius(4.0);
                        if ui.add(all_btn).clicked() {
                            draft.months.clear();
                        }
                    });

                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(3.0, 3.0);
                        for m in 1u8..=12u8 {
                            let m_key = format!("workspace-month-{m}");
                            let is_sel = draft.months.contains(&m);
                            let btn = egui::Button::new(
                                RichText::new(crate::ui::i18n::text(ui, &m_key))
                                    .size(10.5)
                                    .color(if is_sel {
                                        theme.accent
                                    } else {
                                        theme.text_secondary
                                    }),
                            )
                            .fill(if is_sel {
                                theme.accent_soft
                            } else {
                                theme.bg_pill
                            })
                            .stroke(Stroke::new(
                                1.0,
                                if is_sel {
                                    theme.accent
                                } else {
                                    theme.border_default
                                },
                            ))
                            .corner_radius(4.0)
                            .min_size(egui::vec2(28.0, 20.0));
                            if ui.add(btn).clicked() {
                                if is_sel {
                                    draft.months.retain(|&x| x != m);
                                } else {
                                    draft.months.push(m);
                                    draft.months.sort();
                                }
                            }
                        }
                    });

                    ui.add_space(4.0);
                    render_time_picker(ui, &mut draft.time, &mut draft.all_day, &theme);
                }
                _ => {
                    render_time_picker(ui, &mut draft.time, &mut draft.all_day, &theme);
                }
            }

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Visualização e Seleção de Temas
            let language = state.settings.language;
            let themes = &state.themes;

            render_visual_theme_selector(
                ui,
                "event_default_theme_strip",
                "workspace-event-default-theme",
                &mut draft.theme_id,
                themes,
                language,
                &theme,
            );

            ui.add_space(6.0);

            render_visual_theme_selector(
                ui,
                "event_default_wallpaper_strip",
                "workspace-event-default-wallpaper",
                &mut draft.idle_theme_id,
                themes,
                language,
                &theme,
            );

            // Se for novo evento, opção de copiar itens da lista atual
            if !is_editing {
                ui.add_space(4.0);
                ui.checkbox(
                    &mut draft.copy_current_playlist,
                    RichText::new(crate::ui::i18n::text(
                        ui,
                        "workspace-event-copy-current-playlist",
                    ))
                    .size(11.5)
                    .color(theme.text_primary),
                );
            }

            ui.add_space(8.0);
            ui.separator();

            // Botões de Ação
            ui.horizontal(|ui| {
                if dialog::action(
                    ui,
                    crate::ui::i18n::text(ui, "common-cancel"),
                    false,
                    &theme,
                )
                .clicked()
                {
                    should_cancel = true;
                }

                let can_save = !draft.name.trim().is_empty();
                ui.add_enabled_ui(can_save, |ui| {
                    let action_label = if is_editing {
                        crate::ui::i18n::text(ui, "workspace-save-event")
                    } else {
                        crate::ui::i18n::text(ui, "workspace-create-event")
                    };
                    if dialog::action(ui, action_label, true, &theme).clicked() {
                        should_save = true;
                    }
                });
            });
        },
    );

    if close || should_cancel {
        state.event_editor_draft = None;
        state.show_event_editor_modal = false;
    } else if should_save {
        state.save_event_from_draft();
    }
}

fn add_day_to_date_str(date_str: &str) -> String {
    let trimmed = date_str.trim();
    if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%d/%m/%Y") {
        (d + chrono::Duration::days(1)).format("%d/%m/%Y").to_string()
    } else if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        (d + chrono::Duration::days(1)).format("%Y-%m-%d").to_string()
    } else {
        let today = chrono::Local::now().date_naive();
        (today + chrono::Duration::days(1)).format("%d/%m/%Y").to_string()
    }
}

fn render_weekdays_selector(ui: &mut egui::Ui, days_of_week: &mut Vec<u8>, theme: &AppTheme) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
        let day_keys = [
            ("workspace-day-sun", 0u8),
            ("workspace-day-mon", 1u8),
            ("workspace-day-tue", 2u8),
            ("workspace-day-wed", 3u8),
            ("workspace-day-thu", 4u8),
            ("workspace-day-fri", 5u8),
            ("workspace-day-sat", 6u8),
        ];
        for (day_key, day_u8) in day_keys {
            let label = crate::ui::i18n::text(ui, day_key);
            let is_selected = days_of_week.contains(&day_u8);
            let btn = egui::Button::new(
                RichText::new(label)
                    .size(11.0)
                    .color(if is_selected {
                        theme.accent
                    } else {
                        theme.text_secondary
                    }),
            )
            .fill(if is_selected {
                theme.accent_soft
            } else {
                theme.bg_pill
            })
            .stroke(Stroke::new(
                1.0,
                if is_selected {
                    theme.accent
                } else {
                    theme.border_default
                },
            ))
            .corner_radius(4.0)
            .min_size(egui::vec2(34.0, 24.0));

            if ui.add(btn).clicked() {
                if is_selected {
                    days_of_week.retain(|&d| d != day_u8);
                } else {
                    days_of_week.push(day_u8);
                    days_of_week.sort();
                }
            }
        }
    });
}

fn render_time_picker(
    ui: &mut egui::Ui,
    time: &mut String,
    all_day: &mut bool,
    theme: &AppTheme,
) {
    ui.horizontal(|ui| {
        ui.checkbox(
            all_day,
            RichText::new(crate::ui::i18n::text(ui, "workspace-all-day"))
                .size(11.5)
                .color(if *all_day {
                    theme.accent
                } else {
                    theme.text_primary
                }),
        );

        if !*all_day {
            ui.add_space(6.0);
            ui.label(
                RichText::new(crate::ui::i18n::text(ui, "workspace-event-time"))
                    .size(11.5)
                    .color(theme.text_secondary),
            );
            ui.add_space(2.0);
            ui.add(
                egui::TextEdit::singleline(time)
                    .hint_text("19:00")
                    .desired_width(65.0),
            );

            let quick_times = ["09:00", "10:00", "19:00", "19:30", "20:00"];
            for qt in quick_times {
                let is_sel = time.as_str() == qt;
                let btn = egui::Button::new(
                    RichText::new(qt)
                        .size(11.0)
                        .color(if is_sel {
                            theme.accent
                        } else {
                            theme.text_secondary
                        }),
                )
                .fill(if is_sel {
                    theme.accent_soft
                } else {
                    theme.bg_pill
                })
                .stroke(Stroke::new(
                    1.0,
                    if is_sel {
                        theme.accent
                    } else {
                        theme.border_default
                    },
                ))
                .corner_radius(4.0)
                .min_size(egui::vec2(0.0, 20.0));

                if ui.add(btn).clicked() {
                    *time = qt.to_string();
                }
            }
        }
    });
}

fn render_visual_theme_selector(
    ui: &mut egui::Ui,
    id_salt: &str,
    label_key: &str,
    selected_id: &mut String,
    themes: &[crate::core::models::PresentationTheme],
    language: crate::core::i18n::Language,
    theme: &AppTheme,
) {
    let current_label = if selected_id.is_empty() {
        language.text("workspace-none-keep-current").to_string()
    } else if let Some(t) = themes.iter().find(|t| t.id == *selected_id) {
        t.display_name(language)
    } else {
        selected_id.clone()
    };

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(language.text(label_key))
                .size(11.5)
                .strong()
                .color(theme.text_secondary),
        );
        ui.add_space(2.0);
        ui.label(
            RichText::new(format!("— {current_label}"))
                .size(11.0)
                .color(theme.text_muted),
        );
    });
    ui.add_space(2.0);

    let card_w = 98.0;
    let card_h = 56.0;

    egui::ScrollArea::horizontal()
        .id_salt(id_salt)
        .max_height(card_h + 10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);

                // 1. Card "Nenhum / Manter atual"
                let is_none_selected = selected_id.is_empty();
                let (none_rect, none_resp) = ui.allocate_exact_size(
                    egui::vec2(card_w, card_h),
                    egui::Sense::click(),
                );

                if none_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                if none_resp.clicked() {
                    selected_id.clear();
                }

                let none_hover = ui
                    .ctx()
                    .animate_bool(none_resp.id.with("hov"), none_resp.hovered());
                let none_bg = if is_none_selected {
                    theme.bg_card_hover.lerp_to_gamma(theme.accent_soft, 0.35)
                } else if none_hover > 0.0 {
                    theme.bg_card_hover
                } else {
                    theme.bg_pill
                };

                let none_stroke = if is_none_selected {
                    Stroke::new(2.0, theme.accent)
                } else if none_hover > 0.0 {
                    Stroke::new(1.0, theme.border_active)
                } else {
                    Stroke::new(1.0, theme.border_default)
                };

                ui.painter().rect_filled(none_rect, 6.0, none_bg);
                ui.painter().rect_stroke(
                    none_rect,
                    6.0,
                    none_stroke,
                    egui::StrokeKind::Inside,
                );

                let icon_pos = egui::pos2(none_rect.center().x, none_rect.top() + 20.0);
                ui.painter().text(
                    icon_pos,
                    egui::Align2::CENTER_CENTER,
                    PROHIBIT,
                    egui::FontId::proportional(16.0),
                    if is_none_selected {
                        theme.accent
                    } else {
                        theme.text_muted
                    },
                );

                let label_pos = egui::pos2(none_rect.center().x, none_rect.bottom() - 10.0);
                ui.painter().text(
                    label_pos,
                    egui::Align2::CENTER_CENTER,
                    language.text("workspace-none-keep-current"),
                    egui::FontId::proportional(8.5),
                    if is_none_selected {
                        theme.accent
                    } else {
                        theme.text_secondary
                    },
                );

                // 2. Cards para cada tema
                for t in themes {
                    let is_sel = selected_id == &t.id;
                    let (t_rect, t_resp) = ui.allocate_exact_size(
                        egui::vec2(card_w, card_h),
                        egui::Sense::click(),
                    );

                    if t_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }

                    if t_resp.clicked() {
                        *selected_id = t.id.clone();
                    }

                    // Background do tema
                    crate::render::paint_theme_bg(ui.painter(), t_rect, t, 6.0);

                    // Amostra de texto "Aa"
                    let preview_area = egui::Rect::from_min_max(
                        t_rect.min,
                        egui::pos2(t_rect.max.x, t_rect.max.y - 18.0),
                    );
                    crate::render::render_theme_lyrics(ui.painter(), preview_area, t, "Aa");

                    // Barra com o nome do tema
                    let bar_rect = egui::Rect::from_min_max(
                        egui::pos2(t_rect.min.x, t_rect.max.y - 18.0),
                        t_rect.max,
                    );
                    ui.painter().rect_filled(
                        bar_rect,
                        egui::CornerRadius {
                            nw: 0,
                            ne: 0,
                            sw: 6,
                            se: 6,
                        },
                        Color32::from_black_alpha(175),
                    );
                    ui.painter().text(
                        bar_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        t.display_name(language),
                        egui::FontId::proportional(9.0),
                        Color32::WHITE,
                    );

                    // Contorno de seleção ou hover
                    let stroke = if is_sel {
                        Stroke::new(2.0, theme.accent)
                    } else if t_resp.hovered() {
                        Stroke::new(1.5, theme.border_active)
                    } else {
                        Stroke::new(1.0, Color32::from_black_alpha(60))
                    };
                    ui.painter().rect_stroke(
                        t_rect,
                        6.0,
                        stroke,
                        egui::StrokeKind::Inside,
                    );

                    // Badge de seleção
                    if is_sel {
                        let check_pos = egui::pos2(t_rect.max.x - 9.0, t_rect.min.y + 9.0);
                        ui.painter().circle_filled(check_pos, 7.0, theme.accent);
                        ui.painter().text(
                            check_pos,
                            egui::Align2::CENTER_CENTER,
                            CHECK,
                            egui::FontId::proportional(9.0),
                            theme.text_on_accent,
                        );
                    }
                }
            });
        });
}
