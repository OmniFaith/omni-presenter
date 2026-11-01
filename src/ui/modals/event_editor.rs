use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use crate::ui::components::dialog;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, RichText, Stroke};

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
        420.0,
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
                "weekly" => {
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(crate::ui::i18n::text(ui, "workspace-days-of-week"))
                            .size(11.5)
                            .color(theme.text_secondary),
                    );
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
                            let is_selected = draft.days_of_week.contains(&day_u8);
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
                                    draft.days_of_week.retain(|&d| d != day_u8);
                                } else {
                                    draft.days_of_week.push(day_u8);
                                    draft.days_of_week.sort();
                                }
                            }
                        }
                    });

                    render_time_picker(ui, &mut draft.time, &theme);
                }
                "none" => {
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(crate::ui::i18n::text(ui, "workspace-event-date"))
                                    .size(11.5)
                                    .color(theme.text_secondary),
                            );
                            ui.add_space(2.0);
                            ui.add(
                                egui::TextEdit::singleline(&mut draft.date)
                                    .hint_text("Ex: 15/11/2026")
                                    .desired_width(160.0),
                            );
                        });

                        ui.add_space(8.0);

                        ui.vertical(|ui| {
                            render_time_picker(ui, &mut draft.time, &theme);
                        });
                    });
                }
                _ => {
                    render_time_picker(ui, &mut draft.time, &theme);
                }
            }

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

fn render_time_picker(ui: &mut egui::Ui, time: &mut String, theme: &AppTheme) {
    ui.label(
        RichText::new(crate::ui::i18n::text(ui, "workspace-event-time"))
            .size(11.5)
            .color(theme.text_secondary),
    );
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(time)
                .hint_text("19:00")
                .desired_width(70.0),
        );

        let quick_times = ["09:00", "10:00", "19:00", "19:30", "20:00"];
        for qt in quick_times {
            let is_sel = time == qt;
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
    });
}

