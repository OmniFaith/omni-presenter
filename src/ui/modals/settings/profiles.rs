use crate::core::preferences::{AppSettings, OperatorProfile, ServiceProfile, new_id};
use crate::ui::theme::AppTheme;
use eframe::egui::{self, RichText};
use egui_phosphor::regular::{
    ARROW_COUNTER_CLOCKWISE, BOOK_OPEN, CAMERA, COPY_SIMPLE, PLUS, TRASH, USER,
};

use super::components::{
    card, profile_card, section, styled_button, text_input, ButtonTone,
};

pub(super) fn profiles(ui: &mut egui::Ui, draft: &mut AppSettings, theme: &AppTheme) {
    let language = draft.language;
    section(
        ui,
        "workspace-who-is-operating",
        "workspace-each-person-has-their-own-appearance-shortcuts-and-panel-layout",
        theme,
    );

    card(ui, theme, |ui| {
        egui::ScrollArea::horizontal()
            .id_salt("operator_profile_cards")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for p in &draft.operators {
                        ui.push_id(&p.id, |ui| {
                            let detail = format!(
                                "{} · {}%",
                                language.text(if p.dark_mode { "appearance-dark" } else { "appearance-light" }),
                                p.scale
                            );
                            if profile_card(
                                ui,
                                &p.name,
                                &detail,
                                USER,
                                p.avatar.as_deref(),
                                p.id == draft.active_operator,
                                theme,
                            ) {
                                draft.active_operator = p.id.clone();
                            }
                        });
                    }
                });
            });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        let index = draft
            .operators
            .iter()
            .position(|p| p.id == draft.active_operator)
            .unwrap();

        // Operator Header & Avatar Section
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(60.0, 60.0), egui::Sense::hover());
            crate::ui::components::avatar::paint(
                ui,
                rect,
                draft.operators[index].avatar.as_deref(),
                USER,
                theme,
                true,
            );

            ui.add_space(6.0);

            ui.vertical(|ui| {
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "settings-operator-name"))
                        .size(11.5)
                        .strong()
                        .color(theme.text_secondary),
                );
                ui.add_space(2.0);

                text_input(
                    ui,
                    "operator_name_input",
                    &mut draft.operators[index].name,
                    crate::ui::i18n::text(ui, "settings-operator-name"),
                    Some(260.0),
                    theme,
                );

                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    let photo_picker_id = ui.make_persistent_id(("profile_photo_picker", index));
                    let photo_task = ui
                        .ctx()
                        .data_mut(|data| data.get_temp::<crate::services::tasks::BackgroundTask<Result<Vec<u8>, String>>>(photo_picker_id));
                    if let Some(task) = &photo_task {
                        ui.ctx()
                            .request_repaint_after(std::time::Duration::from_millis(33));
                        if let Some(result) = task.take() {
                            ui.ctx()
                                .data_mut(|data| data.remove::<crate::services::tasks::BackgroundTask<Result<Vec<u8>, String>>>(photo_picker_id));
                            match result {
                                Ok(Ok(photo)) => {
                                    draft.operators[index].avatar = Some(photo);
                                    ui.ctx().data_mut(|d| {
                                        d.remove_temp::<String>(egui::Id::unique("profile_photo_error"))
                                    });
                                }
                                Ok(Err(error)) | Err(error) => {
                                    ui.ctx().data_mut(|d| {
                                        d.insert_temp(egui::Id::unique("profile_photo_error"), error)
                                    });
                                }
                            }
                        }
                    }
                    let clicked = ui.add_enabled_ui(photo_task.is_none(), |ui| {
                        styled_button(
                            ui,
                            Some(CAMERA),
                            language.text("workspace-choose-photo"),
                            ButtonTone::Secondary,
                            theme,
                        )
                    }).inner.clicked();
                    if clicked {
                        let filter = language.text("workspace-photo").to_owned();
                        let too_large = language.text("workspace-choose-an-image-up-to-10-mb").to_owned();
                        let task = crate::services::tasks::BackgroundTask::spawn_with_repaint(ui.ctx(), move || {
                            let Some(path) = rfd::FileDialog::new()
                                .add_filter(
                                    filter,
                                    &["png", "jpg", "jpeg", "webp", "gif", "bmp"],
                                )
                                .pick_file()
                            else {
                                return Ok(Err(String::new()));
                            };
                            let meta = std::fs::metadata(&path).map_err(|e| e.to_string())?;
                            if meta.len() > 10 * 1024 * 1024 {
                                return Ok(Err(too_large));
                            }
                            let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
                            crate::services::profiles::prepare_photo(&bytes).map(Ok)
                        });
                        ui.ctx().data_mut(|data| data.insert_temp(photo_picker_id, task));
                    }
                    if photo_task.is_some() {
                        ui.spinner();
                    }

                    if draft.operators[index].avatar.is_some()
                        && styled_button(
                            ui,
                            Some(TRASH),
                            language.text("workspace-remove-photo"),
                            ButtonTone::Danger,
                            theme,
                        )
                        .clicked()
                    {
                        draft.operators[index].avatar = None;
                    }
                });

                ui.label(
                    RichText::new(language.text("workspace-used-in-your-profile-and-team-messages"))
                        .size(11.0)
                        .color(theme.text_muted),
                );
            });
        });

        if let Some(error) = ui
            .ctx()
            .data_mut(|d| d.get_temp::<String>(egui::Id::unique("profile_photo_error")))
        {
            ui.colored_label(theme.status_warning_fg, error);
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        ui.label(RichText::new(crate::ui::i18n::text(ui, "settings-web-access-password")).strong());
        ui.label(crate::ui::i18n::text(ui, "settings-used-to-sign-in-to-chat-and-web-control-with"));
        if draft.operators[index].web_password_hash.is_empty() {
            ui.label(RichText::new(crate::ui::i18n::text(ui, "settings-default-password")).color(theme.text_secondary));
        }
        let password_id = ui.make_persistent_id(("web_password", &draft.operators[index].id));
        let mut password = ui.ctx().data_mut(|d| d.get_temp::<String>(password_id)).unwrap_or_default();
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut password).password(true)
                .hint_text(crate::ui::i18n::text(ui, "settings-new-password")).desired_width(220.0));
            if ui.add_enabled(!password.is_empty(), egui::Button::new(crate::ui::i18n::text(ui, "settings-set-password"))).clicked() {
                match crate::services::credentials::hash(&password) {
                    Ok(hash) => {
                        draft.operators[index].web_password_hash = hash;
                        password.clear();
                    }
                    Err(error) => { ui.colored_label(theme.status_warning_fg, error); }
                }
            }
        });
        ui.ctx().data_mut(|d| d.insert_temp(password_id, password));
        ui.label(RichText::new(crate::ui::i18n::text(ui, "settings-set-the-password-and-click-save-changes-to-apply-it"))
            .small().color(theme.text_muted));
        draft.operators[index].chat_notifications = true;

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Operator action toolbar
        ui.horizontal(|ui| {
            if styled_button(
                ui,
                Some(PLUS),
                language.text("workspace-new-operator-label"),
                ButtonTone::Primary,
                theme,
            )
            .clicked()
            {
                let p = OperatorProfile {
                    id: new_id("operator"),
                    name: language.format(
                        "workspace-operator",
                        &[("number", &(draft.operators.len() + 1).to_string())],
                    ),
                    ..Default::default()
                };
                draft.active_operator = p.id.clone();
                draft.operators.push(p);
            }

            if styled_button(
                ui,
                Some(COPY_SIMPLE),
                language.text("workspace-duplicate-preferences"),
                ButtonTone::Secondary,
                theme,
            )
            .clicked()
            {
                let mut p = draft.operators[index].clone();
                p.id = new_id("operator");
                p.name = language.format("workspace-copy-label", &[("name", &p.name)]);
                p.avatar = None;
                draft.active_operator = p.id.clone();
                draft.operators.push(p);
            }

            if draft.operators.len() > 1 {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if styled_button(
                        ui,
                        Some(TRASH),
                        language.text("workspace-delete-operator"),
                        ButtonTone::Danger,
                        theme,
                    )
                    .clicked()
                    {
                        draft.operators.remove(index);
                        draft.active_operator = draft.operators[0].id.clone();
                    }
                });
            }
        });
    });

    ui.add_space(10.0);

    // --- Culto / Service Profile ---
    section(
        ui,
        "workspace-service-profile",
        "workspace-playlist-initial-theme-idle-background-format-and-bible-version",
        theme,
    );

    card(ui, theme, |ui| {
        egui::ScrollArea::horizontal()
            .id_salt("service_profile_cards")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for p in &draft.services {
                        ui.push_id(&p.id, |ui| {
                            if profile_card(
                                ui,
                                &p.display_name(language),
                                &language.format(
                                    "settings-items-in-repertoire",
                                    &[("count", &p.playlist.len().to_string())],
                                ),
                                BOOK_OPEN,
                                None,
                                p.id == draft.active_service,
                                theme,
                            ) {
                                draft.active_service = p.id.clone();
                            }
                        });
                    }
                });
            });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        let index = draft
            .services
            .iter()
            .position(|p| p.id == draft.active_service)
            .unwrap();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "settings-service-name"))
                        .size(11.5)
                        .strong()
                        .color(theme.text_secondary),
                );
                ui.add_space(2.0);
                let mut current_name = draft.services[index].display_name(language);
                if text_input(
                    ui,
                    "service_name_input",
                    &mut current_name,
                    language.text("settings-e-g-family-service"),
                    Some(260.0),
                    theme,
                )
                .changed()
                {
                    draft.services[index].name = current_name;
                }
            });

            ui.add_space(8.0);

            ui.vertical(|ui| {
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "settings-active-playlist"))
                        .size(11.5)
                        .strong()
                        .color(theme.text_secondary),
                );
                ui.add_space(6.0);
                egui::Frame::new()
                    .fill(theme.bg_pill)
                    .corner_radius(6.0)
                    .inner_margin(egui::vec2(8.0, 4.0))
                    .stroke(egui::Stroke::new(1.0, theme.border_default))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(language.format(
                                "settings-songs-and-verses",
                                &[("count", &draft.services[index].playlist.len().to_string())],
                            ))
                            .size(11.5)
                            .strong()
                            .color(theme.text_primary),
                        );
                    });
            });
        });

        ui.add_space(4.0);

        if styled_button(
            ui,
            Some(ARROW_COUNTER_CLOCKWISE),
            language.text("workspace-mark-the-entire-playlist-as-not-presented"),
            ButtonTone::Secondary,
            theme,
        )
        .clicked()
        {
            let active = draft
                .services
                .iter_mut()
                .find(|p| p.id == draft.active_service)
                .unwrap();
            for item in &mut active.playlist {
                item.played = false;
            }
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Service action toolbar
        ui.horizontal(|ui| {
            if styled_button(
                ui,
                Some(PLUS),
                language.text("workspace-new-service-label"),
                ButtonTone::Primary,
                theme,
            )
            .clicked()
            {
                let p = ServiceProfile {
                    id: new_id("service"),
                    name: language.format(
                        "workspace-service",
                        &[("number", &(draft.services.len() + 1).to_string())],
                    ),
                    ..Default::default()
                };
                draft.active_service = p.id.clone();
                draft.services.push(p);
            }

            if styled_button(
                ui,
                Some(COPY_SIMPLE),
                language.text("workspace-duplicate-service"),
                ButtonTone::Secondary,
                theme,
            )
            .clicked()
            {
                let mut p = draft.services[index].clone();
                p.id = new_id("service");
                p.name = language.format("workspace-copy-label", &[("name", &p.name)]);
                for item in &mut p.playlist {
                    item.played = false;
                }
                draft.active_service = p.id.clone();
                draft.services.push(p);
            }

            if draft.services.len() > 1 {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if styled_button(
                        ui,
                        Some(TRASH),
                        language.text("workspace-delete-service"),
                        ButtonTone::Danger,
                        theme,
                    )
                    .clicked()
                    {
                        draft.services.remove(index);
                        draft.active_service = draft.services[0].id.clone();
                    }
                });
            }
        });
    });

    ui.add_space(4.0);
    ui.label(
        RichText::new(language.text("workspace-you-can-also-switch-operator-and-service-in-the-top"))
            .size(11.5)
            .color(theme.text_secondary),
    );
}
