use crate::core::state::ControlPanelState;
use crate::ui::AppTheme;
use eframe::egui::{self, RichText};
use egui_phosphor::regular::{
    CARET_DOWN, CARET_LEFT, CARET_RIGHT, CHECK, CHURCH, COPY_SIMPLE, GEAR, PLUS, REPEAT, TRASH,
};

fn split_date_tag(date_str: &str) -> (&str, Option<&str>) {
    if let Some(idx) = date_str.find(" (") {
        (&date_str[..idx], Some(date_str[idx..].trim()))
    } else {
        (date_str, None)
    }
}

fn make_date_time_layout_job(
    date_text: &str,
    time_text: &str,
    extra_tag: Option<&str>,
    items_text: Option<&str>,
    theme: &AppTheme,
    font_size: f32,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();

    // 1. Data (texto primário com bom contraste)
    job.append(
        date_text,
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::proportional(font_size),
            color: theme.text_primary,
            ..Default::default()
        },
    );

    // 2. Horário em DESTAQUE no meio do texto com cor de acento
    let trimmed_time = time_text.trim();
    if !trimmed_time.is_empty() {
        let prefix = if trimmed_time.starts_with('·')
            || trimmed_time.starts_with("às")
            || trimmed_time.starts_with("at")
        {
            format!("  {trimmed_time}")
        } else {
            format!(" · {trimmed_time}")
        };
        job.append(
            &prefix,
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::proportional(font_size),
                color: theme.accent,
                ..Default::default()
            },
        );
    }

    // 3. Tag de proximidade (ex: "(em 2 dias)")
    if let Some(tag) = extra_tag {
        if !tag.is_empty() {
            job.append(
                &format!("  {tag}"),
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::proportional(font_size * 0.9),
                    color: theme.text_secondary,
                    ..Default::default()
                },
            );
        }
    }

    // 4. Contagem de itens (ex: "(0 itens)")
    if let Some(items) = items_text {
        if !items.is_empty() {
            job.append(
                &format!("  ({items})"),
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::proportional(font_size * 0.9),
                    color: theme.text_secondary,
                    ..Default::default()
                },
            );
        }
    }

    job
}

pub fn render_event_footer(ui: &mut egui::Ui, state: &mut ControlPanelState, theme: &AppTheme) {
    let lang = crate::ui::i18n::language(ui.ctx());
    let active_service_id = state.settings.active_service.clone();
    let (event_name, active_time, schedule_info, item_count) = {
        let active = state.settings.service();
        (
            active.name.clone(),
            active.recurrence.time.trim().to_string(),
            active.schedule_label_localized(lang),
            state.playlist.len(),
        )
    };
    let current_date = state.current_service_date();
    let date_display =
        crate::core::state::events::format_date_display_with_time(&current_date, &active_time, lang);

    let avail_w = ui.available_width().max(40.0);
    let actions_w = 48.0;
    let gap = 4.0;
    let row_h = 24.0;
    ui.spacing_mut().item_spacing = egui::vec2(gap, 3.0);

    // Linha 1: Perfil do Evento / Culto + Ações Rápidas [ + │ ⚙ ]
    let event_btn_w = (avail_w - actions_w - gap).max(40.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(gap, 0.0);

        let (rect, resp) =
            ui.allocate_exact_size(egui::vec2(event_btn_w, row_h), egui::Sense::click());
        let is_hovered = resp.hovered();
        let is_pressed = resp.is_pointer_button_down_on();

        let bg = if is_pressed {
            theme.bg_card_active
        } else if is_hovered {
            theme.bg_card_hover
        } else {
            theme.bg_pill
        };
        let stroke = egui::Stroke::new(
            1.0,
            if is_hovered || is_pressed {
                theme.border_active
            } else {
                theme.border_default
            },
        );

        ui.painter().rect_filled(rect, 6.0, bg);
        ui.painter().rect_stroke(rect, 6.0, stroke, egui::StrokeKind::Inside);

        ui.painter().text(
            rect.left_center() + egui::vec2(7.0, 0.0),
            egui::Align2::LEFT_CENTER,
            CHURCH,
            egui::FontId::proportional(11.5),
            theme.accent,
        );

        ui.painter().text(
            rect.right_center() - egui::vec2(7.0, 0.0),
            egui::Align2::RIGHT_CENTER,
            CARET_DOWN,
            egui::FontId::proportional(9.0),
            theme.text_secondary,
        );

        let text_avail_w = (event_btn_w - 38.0).max(10.0);
        let font_id = egui::FontId::proportional(11.0);
        let full_galley = ui.painter().layout_no_wrap(event_name.clone(), font_id.clone(), theme.text_primary);
        let display_name = if full_galley.size().x <= text_avail_w {
            event_name.clone()
        } else {
            let mut truncated = event_name.clone();
            while !truncated.is_empty() {
                truncated.pop();
                let candidate = format!("{truncated}…");
                let g = ui.painter().layout_no_wrap(candidate.clone(), font_id.clone(), theme.text_primary);
                if g.size().x <= text_avail_w {
                    truncated = candidate;
                    break;
                }
            }
            if truncated.is_empty() {
                "…".into()
            } else {
                truncated
            }
        };

        let text_rect = egui::Rect::from_min_max(
            rect.min + egui::vec2(22.0, 0.0),
            rect.max - egui::vec2(18.0, 0.0),
        );
        ui.painter().text(
            text_rect.left_center(),
            egui::Align2::LEFT_CENTER,
            display_name,
            font_id,
            theme.text_primary,
        );

        let tooltip_text = if !active_time.is_empty() && !schedule_info.is_empty() {
            format!("{event_name} · {active_time}\n{REPEAT}  {schedule_info}")
        } else if !schedule_info.is_empty() {
            format!("{event_name}\n{REPEAT}  {schedule_info}")
        } else {
            event_name.clone()
        };
        let resp = resp.on_hover_text(tooltip_text);

        egui::Popup::menu(&resp).show(|ui| {
            ui.set_max_width(280.0);
            ui.label(
                RichText::new(crate::ui::i18n::text(ui, "workspace-events"))
                    .size(11.0)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.separator();

            let services_info: Vec<(String, String, String, String, usize)> = state
                .settings
                .services
                .iter()
                .map(|s| {
                    let count = if s.id == active_service_id {
                        state.playlist.len()
                    } else {
                        s.playlist.len()
                    };
                    let time = s.recurrence.time.trim().to_string();
                    (
                        s.id.clone(),
                        s.name.clone(),
                        time,
                        s.schedule_label_localized(lang),
                        count,
                    )
                })
                .collect();

            let mut switch_to = None;
            for (id, name, time, sched, count) in services_info {
                let is_active = id == active_service_id;
                let check = if is_active {
                    format!("{CHECK} ")
                } else {
                    "   ".to_string()
                };
                let time_suffix = if time.is_empty() {
                    String::new()
                } else {
                    format!(" · {time}")
                };
                let mut btn =
                    ui.selectable_label(is_active, format!("{check}{name}{time_suffix} ({count})"));
                if !sched.is_empty() {
                    btn = btn.on_hover_text(&sched);
                }
                if btn.clicked() && !is_active {
                    switch_to = Some(id);
                    ui.close();
                }
                if !sched.is_empty() {
                    ui.label(
                        RichText::new(format!("   {sched}"))
                            .size(9.5)
                            .color(theme.text_secondary),
                    );
                }
            }

            if let Some(target_id) = switch_to {
                state.switch_service(&target_id);
            }

            ui.separator();

            if ui
                .button(format!(
                    "{PLUS}  {}",
                    crate::ui::i18n::text(ui, "workspace-new-event")
                ))
                .clicked()
            {
                state.open_new_event_modal();
                ui.close();
            }

            if ui
                .button(format!(
                    "{GEAR}  {}",
                    crate::ui::i18n::text(ui, "workspace-edit-event")
                ))
                .clicked()
            {
                state.open_edit_event_modal(&active_service_id);
                ui.close();
            }

            if ui
                .button(format!(
                    "{COPY_SIMPLE}  {}",
                    crate::ui::i18n::text(ui, "workspace-duplicate-event")
                ))
                .clicked()
            {
                state.duplicate_event(&active_service_id);
                ui.close();
            }

            if state.settings.services.len() > 1 {
                if ui
                    .button(format!(
                        "{TRASH}  {}",
                        crate::ui::i18n::text(ui, "workspace-delete-event")
                    ))
                    .clicked()
                {
                    state.delete_event(&active_service_id);
                    ui.close();
                }
            }
        });

        // Pílula de ações rápidas agrupadas [ + │ ⚙ ]
        let (act_rect, _) = ui.allocate_exact_size(egui::vec2(actions_w, row_h), egui::Sense::hover());
        ui.painter().rect_filled(act_rect, 6.0, theme.bg_pill);
        ui.painter().rect_stroke(act_rect, 6.0, egui::Stroke::new(1.0, theme.border_default), egui::StrokeKind::Inside);

        let half_w = actions_w * 0.5;
        let plus_rect = egui::Rect::from_min_size(act_rect.min, egui::vec2(half_w, row_h));
        let gear_rect = egui::Rect::from_min_size(egui::pos2(act_rect.min.x + half_w, act_rect.min.y), egui::vec2(half_w, row_h));

        let div_x = act_rect.min.x + half_w;
        ui.painter().line_segment(
            [egui::pos2(div_x, act_rect.min.y + 4.0), egui::pos2(div_x, act_rect.max.y - 4.0)],
            egui::Stroke::new(1.0, theme.border_subtle.linear_multiply(0.45)),
        );

        let plus_resp = ui.interact(plus_rect, ui.make_persistent_id("footer_plus_btn"), egui::Sense::click());
        if plus_resp.hovered() {
            ui.painter().rect_filled(plus_rect, 5.0, theme.bg_card_hover);
        }
        ui.painter().text(
            plus_rect.center(),
            egui::Align2::CENTER_CENTER,
            PLUS,
            egui::FontId::proportional(11.5),
            if plus_resp.hovered() { theme.text_primary } else { theme.text_secondary },
        );
        if plus_resp.on_hover_text(crate::ui::i18n::text(ui, "workspace-new-event")).clicked() {
            state.open_new_event_modal();
        }

        let gear_resp = ui.interact(gear_rect, ui.make_persistent_id("footer_gear_btn"), egui::Sense::click());
        if gear_resp.hovered() {
            ui.painter().rect_filled(gear_rect, 5.0, theme.bg_card_hover);
        }
        ui.painter().text(
            gear_rect.center(),
            egui::Align2::CENTER_CENTER,
            GEAR,
            egui::FontId::proportional(11.5),
            if gear_resp.hovered() { theme.text_primary } else { theme.text_secondary },
        );
        if gear_resp.on_hover_text(crate::ui::i18n::text(ui, "workspace-edit-event")).clicked() {
            state.open_edit_event_modal(&active_service_id);
        }
    });

    // Linha 2: Pílula Segmentada Integrada de Data: [ ‹ │ Dom, 01/Nov · 19:30 ▾ │ › ]
    let (date_bar_rect, _) = ui.allocate_exact_size(egui::vec2(avail_w, row_h), egui::Sense::hover());
    ui.painter().rect_filled(date_bar_rect, 6.0, theme.bg_pill);
    ui.painter().rect_stroke(date_bar_rect, 6.0, egui::Stroke::new(1.0, theme.border_default), egui::StrokeKind::Inside);

    let arrow_w = 26.0;
    let left_arrow_rect = egui::Rect::from_min_size(date_bar_rect.min, egui::vec2(arrow_w, row_h));
    let right_arrow_rect = egui::Rect::from_min_size(
        egui::pos2(date_bar_rect.max.x - arrow_w, date_bar_rect.min.y),
        egui::vec2(arrow_w, row_h),
    );
    let center_date_rect = egui::Rect::from_min_max(
        egui::pos2(date_bar_rect.min.x + arrow_w, date_bar_rect.min.y),
        egui::pos2(date_bar_rect.max.x - arrow_w, date_bar_rect.max.y),
    );

    // Divisores verticais sutis
    ui.painter().line_segment(
        [egui::pos2(date_bar_rect.min.x + arrow_w, date_bar_rect.min.y + 4.0), egui::pos2(date_bar_rect.min.x + arrow_w, date_bar_rect.max.y - 4.0)],
        egui::Stroke::new(1.0, theme.border_subtle.linear_multiply(0.45)),
    );
    ui.painter().line_segment(
        [egui::pos2(date_bar_rect.max.x - arrow_w, date_bar_rect.min.y + 4.0), egui::pos2(date_bar_rect.max.x - arrow_w, date_bar_rect.max.y - 4.0)],
        egui::Stroke::new(1.0, theme.border_subtle.linear_multiply(0.45)),
    );

    // Botão Anterior ‹
    let left_resp = ui.interact(left_arrow_rect, ui.make_persistent_id("footer_date_prev"), egui::Sense::click());
    if left_resp.hovered() {
        ui.painter().rect_filled(left_arrow_rect, 5.0, theme.bg_card_hover);
    }
    ui.painter().text(
        left_arrow_rect.center(),
        egui::Align2::CENTER_CENTER,
        CARET_LEFT,
        egui::FontId::proportional(11.5),
        if left_resp.hovered() { theme.accent } else { theme.text_secondary },
    );
    if left_resp.on_hover_text(crate::ui::i18n::text(ui, "workspace-previous-service-date")).clicked() {
        state.step_service_date(-1);
    }

    // Botão Próximo ›
    let right_resp = ui.interact(right_arrow_rect, ui.make_persistent_id("footer_date_next"), egui::Sense::click());
    if right_resp.hovered() {
        ui.painter().rect_filled(right_arrow_rect, 5.0, theme.bg_card_hover);
    }
    ui.painter().text(
        right_arrow_rect.center(),
        egui::Align2::CENTER_CENTER,
        CARET_RIGHT,
        egui::FontId::proportional(11.5),
        if right_resp.hovered() { theme.accent } else { theme.text_secondary },
    );
    if right_resp.on_hover_text(crate::ui::i18n::text(ui, "workspace-next-service-date")).clicked() {
        state.step_service_date(1);
    }

    // Botão Central de Data
    let center_resp = ui.interact(center_date_rect, ui.make_persistent_id("footer_date_center"), egui::Sense::click());
    if center_resp.hovered() {
        ui.painter().rect_filled(center_date_rect, 0.0, theme.bg_card_hover);
    }

    let today_str = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
    let is_today = current_date == today_str;

    let date_short_raw =
        crate::core::state::events::format_date_short_localized(&current_date, lang);
    let (d_base, _) = split_date_tag(&date_short_raw);

    // Se for hoje exibe "Hoje · {horário}". Se não for hoje, exibe sempre o dia e o horário completos ("Dom, 01/Nov · 19:30")
    let display_date_label = if is_today {
        lang.text("workspace-date-today")
    } else {
        d_base
    };

    let mut job = egui::text::LayoutJob::default();
    job.append(
        display_date_label,
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::proportional(10.5),
            color: if is_today { theme.accent } else { theme.text_primary },
            ..Default::default()
        },
    );
    if !active_time.trim().is_empty() {
        job.append(
            &format!(" · {active_time}"),
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::proportional(10.5),
                color: theme.accent,
                ..Default::default()
            },
        );
    }
    let final_galley = ui.fonts_mut(|f| f.layout_job(job));

        let center_w = center_date_rect.width();
        let galley_w = final_galley.size().x;
        let text_pos = if galley_w + 16.0 <= center_w {
            let start_x = center_date_rect.center().x - (galley_w + 12.0) * 0.5;
            ui.painter().text(
                egui::pos2(start_x + galley_w + 6.0, center_date_rect.center().y),
                egui::Align2::LEFT_CENTER,
                CARET_DOWN,
                egui::FontId::proportional(9.0),
                theme.text_secondary,
            );
            egui::pos2(start_x, center_date_rect.center().y - final_galley.size().y * 0.5)
        } else {
            ui.painter().text(
                center_date_rect.right_center() - egui::vec2(5.0, 0.0),
                egui::Align2::RIGHT_CENTER,
                CARET_DOWN,
                egui::FontId::proportional(9.0),
                theme.text_secondary,
            );
            egui::pos2(center_date_rect.min.x + 6.0, center_date_rect.center().y - final_galley.size().y * 0.5)
        };
        ui.painter().galley(text_pos, final_galley, theme.text_primary);

        let date_tooltip =
            lang.format("workspace-selected-date-tooltip", &[("date", &date_display)]);
        let center_resp = center_resp.on_hover_text(date_tooltip);

        egui::Popup::menu(&center_resp).show(|ui| {
            ui.set_width(310.0);
            ui.set_max_width(330.0);

            egui::ScrollArea::vertical()
                .id_salt("date_popup_scroll")
                .max_height(260.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(4.0, 5.0);

                    // Ação de ir para o culto de hoje
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(crate::ui::i18n::text(ui, "workspace-nearest-service"))
                                    .size(11.5)
                                    .strong(),
                            )
                            .fill(theme.bg_card)
                            .stroke(egui::Stroke::new(1.0, theme.border_default))
                            .corner_radius(6.0)
                            .min_size(egui::vec2(ui.available_width(), 26.0)),
                        )
                        .clicked()
                    {
                        state.auto_select_nearest_service();
                        ui.close();
                    }

                    // Opção de copiar do culto anterior
                    let cur = state.current_service_date();
                    let prev_d = state.settings.service().step_date(&cur, -1);
                    let prev_has = state
                        .settings
                        .service()
                        .get_session(&prev_d)
                        .is_some_and(|s| !s.playlist.is_empty());
                    if prev_has {
                        let prev_fmt =
                            crate::core::state::events::format_date_display_localized(&prev_d, lang);
                        let copy_label = lang
                            .format("workspace-copy-previous-with-date", &[("date", &prev_fmt)]);
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(format!("{COPY_SIMPLE}  {copy_label}")).size(11.0),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .min_size(egui::vec2(ui.available_width(), 24.0)),
                            )
                            .clicked()
                        {
                            state.copy_items_from_previous_service();
                            ui.close();
                        }
                    }

                    // Histórico / Sessões salvas deste perfil
                    let sessions = state.settings.service().sessions.clone();
                    let mut switch_date = None;
                    let mut switch_service_to = None;

                    let past_sessions: Vec<_> =
                        sessions.iter().filter(|s| s.date < current_date).collect();
                    if !past_sessions.is_empty() {
                        ui.separator();
                        ui.label(
                            RichText::new(crate::ui::i18n::text(ui, "workspace-history-sessions"))
                                .size(10.5)
                                .strong()
                                .color(theme.text_secondary),
                        );
                        for s in past_sessions.iter().rev().take(4) {
                            let base_fmt =
                                crate::core::state::events::format_date_display_localized(&s.date, lang);
                            let (date_part, tag_opt) = split_date_tag(&base_fmt);
                            let count_str = s.playlist.len().to_string();
                            let items_lbl =
                                lang.format("workspace-items-count", &[("count", &count_str)]);
                            let job = make_date_time_layout_job(
                                date_part,
                                &active_time,
                                tag_opt,
                                Some(&items_lbl),
                                theme,
                                11.5,
                            );
                            if ui
                                .add(
                                    egui::Button::new(job)
                                        .fill(egui::Color32::TRANSPARENT)
                                        .min_size(egui::vec2(ui.available_width(), 24.0)),
                                )
                                .clicked()
                            {
                                switch_date = Some(s.date.clone());
                                ui.close();
                            }
                        }
                    }

                    // Data atual em destaque (Card destacado com borda e contraste perfeito)
                    ui.separator();
                    let active_badge = lang.text("workspace-active-badge");
                    let count_str = item_count.to_string();
                    let items_lbl = lang.format("workspace-items-count", &[("count", &count_str)]);
                    let (active_base, active_tag) = split_date_tag(&date_display);
                    let active_summary = format!("{active_badge} · {items_lbl}");

                    egui::Frame::new()
                        .fill(theme.bg_card_active)
                        .stroke(egui::Stroke::new(1.0, theme.accent.linear_multiply(0.4)))
                        .corner_radius(6.0)
                        .inner_margin(egui::Margin::symmetric(8, 6))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(CHECK)
                                        .size(12.0)
                                        .color(theme.accent)
                                        .strong(),
                                );
                                let job = make_date_time_layout_job(
                                    active_base,
                                    &active_time,
                                    active_tag,
                                    Some(&active_summary),
                                    theme,
                                    11.5,
                                );
                                ui.label(job);
                            });
                        });

                    // Outros cultos/eventos agendados para a mesma data
                    let other_same_day: Vec<_> = state
                        .settings
                        .services
                        .iter()
                        .filter(|s| s.id != active_service_id && s.occurs_on_date(&current_date))
                        .collect();
                    if !other_same_day.is_empty() {
                        ui.separator();
                        ui.label(
                            RichText::new(crate::ui::i18n::text(ui, "workspace-same-day-events"))
                                .size(10.5)
                                .strong()
                                .color(theme.accent),
                        );
                        for s in &other_same_day {
                            let s_count = s.get_session(&current_date).map_or(0, |sess| sess.playlist.len());
                            let count_str = s_count.to_string();
                            let items_lbl = lang.format("workspace-items-count", &[("count", &count_str)]);
                            let job = make_date_time_layout_job(
                                &s.name,
                                &s.recurrence.time,
                                None,
                                Some(&items_lbl),
                                theme,
                                11.5,
                            );
                            if ui
                                .add(
                                    egui::Button::new(job)
                                        .fill(egui::Color32::TRANSPARENT)
                                        .min_size(egui::vec2(ui.available_width(), 24.0)),
                                )
                                .clicked()
                            {
                                switch_service_to = Some(s.id.clone());
                                ui.close();
                            }
                        }
                    }

                    // Próximos cultos planejados
                    ui.separator();
                    ui.label(
                        RichText::new(crate::ui::i18n::text(ui, "workspace-upcoming-sessions"))
                            .size(10.5)
                            .strong()
                            .color(theme.text_secondary),
                    );

                    for step in 1..=3 {
                        let up_date = state.settings.service().step_date(&current_date, step);
                        let base_fmt =
                            crate::core::state::events::format_date_display_localized(&up_date, lang);
                        let (date_part, tag_opt) = split_date_tag(&base_fmt);
                        let up_count = state
                            .settings
                            .service()
                            .get_session(&up_date)
                            .map_or(0, |s| s.playlist.len());
                        let up_count_str = up_count.to_string();
                        let up_items_lbl =
                            lang.format("workspace-items-count", &[("count", &up_count_str)]);
                        let job = make_date_time_layout_job(
                            date_part,
                            &active_time,
                            tag_opt,
                            Some(&up_items_lbl),
                            theme,
                            11.5,
                        );
                        if ui
                            .add(
                                egui::Button::new(job)
                                    .fill(egui::Color32::TRANSPARENT)
                                    .min_size(egui::vec2(ui.available_width(), 24.0)),
                            )
                            .clicked()
                        {
                            switch_date = Some(up_date);
                            ui.close();
                        }
                    }

                    if let Some(target_d) = switch_date {
                        state.switch_service_date(&target_d);
                    }
                    if let Some(target_s) = switch_service_to {
                        state.switch_service(&target_s);
                    }
                });
        });
}

