use crate::core::bible::BibleTranslation;
use crate::core::preferences::AppSettings;
use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, FontId, RichText, Stroke};
use egui_phosphor::regular::{
    CARET_DOWN, CARET_RIGHT, CHECK_CIRCLE, DOWNLOAD_SIMPLE, FILE_PLUS, FILE_TEXT, GLOBE, STAR, X,
};

use super::components::{
    card, section, styled_button, text_input, ButtonTone,
};

pub(super) fn bibles(
    ui: &mut egui::Ui,
    draft: &mut AppSettings,
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
    theme: &AppTheme,
) {
    let language = draft.language;
    section(
        ui,
        "workspace-available-translations",
        "settings-manage-bible-translations-and-presentation-shortcuts",
        theme,
    );
    let catalog = state.bible.library.catalog();
    let mut use_version = None;
    let service = draft
        .services
        .iter_mut()
        .find(|profile| profile.id == draft.active_service)
        .unwrap();

    // Collect all languages with counts
    let mut lang_counts: std::collections::BTreeMap<&'static str, usize> =
        std::collections::BTreeMap::new();
    for (id, _) in &catalog {
        let lang = crate::core::bible::bible_language(id);
        *lang_counts.entry(lang).or_default() += 1;
    }

    // Sort order: Portuguese first, then English, then Spanish, then alphabetical
    let mut ordered_langs: Vec<(&'static str, usize)> = Vec::new();
    if let Some(&count) = lang_counts.get("Português") {
        ordered_langs.push(("Português", count));
    }
    if let Some(&count) = lang_counts.get("Inglês") {
        ordered_langs.push(("Inglês", count));
    }
    if let Some(&count) = lang_counts.get("Espanhol") {
        ordered_langs.push(("Espanhol", count));
    }
    for (&lang, &count) in &lang_counts {
        if lang != "Português" && lang != "Inglês" && lang != "Espanhol" {
            ordered_langs.push((lang, count));
        }
    }

    let search_id = ui.make_persistent_id("settings_bible_search");
    let mut search_query = ui
        .ctx()
        .data_mut(|d| d.get_temp::<String>(search_id).unwrap_or_default());

    let selected_lang_id = ui.make_persistent_id("settings_selected_bible_lang");
    let mut selected_lang = ui.ctx().data_mut(|d| {
        d.get_temp::<String>(selected_lang_id)
            .unwrap_or_else(|| "Português".to_string())
    });

    if selected_lang != "ALL" && !lang_counts.contains_key(selected_lang.as_str()) {
        selected_lang = "Português".to_string();
    }

    let query_clean = crate::core::bible::fold(&search_query);

    // Main Card: Full-Width Translations List with Filter Toolbar
    card(ui, theme, |ui| {
        // Toolbar: Search Box on Left, Language Dropdown on Right
        ui.horizontal(|ui| {
            let resp = text_input(
                ui,
                search_id,
                &mut search_query,
                "Buscar tradução, sigla ou idioma...",
                Some(320.0),
                theme,
            );
            if resp.changed() {
                ui.ctx().data_mut(|d| d.insert_temp(search_id, search_query.clone()));
            }

            if !search_query.is_empty() {
                if styled_button(ui, Some(X), "", ButtonTone::Ghost, theme)
                    .on_hover_text("Limpar busca")
                    .clicked()
                {
                    search_query.clear();
                    ui.ctx().data_mut(|d| d.insert_temp(search_id, String::new()));
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let combo_label = if selected_lang == "ALL" {
                    format!("Todas as línguas ({})", catalog.len())
                } else {
                    let count = lang_counts.get(selected_lang.as_str()).copied().unwrap_or(0);
                    format!("{selected_lang} ({count})")
                };

                egui::ComboBox::from_id_salt("settings_bibles_lang_combo")
                    .selected_text(RichText::new(format!("{GLOBE}  {combo_label}")).size(12.0))
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(
                                selected_lang == "ALL",
                                format!("Todas as línguas ({})", catalog.len()),
                            )
                            .clicked()
                        {
                            selected_lang = "ALL".to_string();
                            ui.ctx().data_mut(|d| d.insert_temp(selected_lang_id, selected_lang.clone()));
                        }
                        ui.separator();
                        for (lang, count) in &ordered_langs {
                            if ui
                                .selectable_label(
                                    selected_lang.as_str() == *lang,
                                    format!("{lang} ({count})"),
                                )
                                .clicked()
                            {
                                selected_lang = (*lang).to_string();
                                ui.ctx().data_mut(|d| d.insert_temp(selected_lang_id, selected_lang.clone()));
                            }
                        }
                    });
            });
        });

        // Quick Language Filter Pills (when not searching)
        if query_clean.is_empty() {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let pt_count = lang_counts.get("Português").copied().unwrap_or(0);
                let en_count = lang_counts.get("Inglês").copied().unwrap_or(0);
                let es_count = lang_counts.get("Espanhol").copied().unwrap_or(0);

                for (lang_key, label) in [
                    ("Português", format!("Português ({pt_count})")),
                    ("Inglês", format!("Inglês ({en_count})")),
                    ("Espanhol", format!("Espanhol ({es_count})")),
                    ("ALL", format!("Todas ({})", catalog.len())),
                ] {
                    let is_active = selected_lang == lang_key;
                    let tone = if is_active {
                        ButtonTone::Primary
                    } else {
                        ButtonTone::Secondary
                    };
                    if styled_button(ui, None, &label, tone, theme).clicked() {
                        selected_lang = lang_key.to_string();
                        ui.ctx().data_mut(|d| d.insert_temp(selected_lang_id, selected_lang.clone()));
                    }
                }
            });
        }

        ui.add_space(6.0);
        let sep_y = ui.cursor().top();
        ui.painter().hline(
            ui.available_rect_before_wrap().x_range(),
            sep_y,
            Stroke::new(1.0, theme.border_subtle),
        );
        ui.add_space(6.0);

        // Filter versions
        let (list_title, filtered_versions): (String, Vec<_>) = if !query_clean.is_empty() {
            let results: Vec<_> = catalog
                .iter()
                .filter(|(id, name)| {
                    let lang = crate::core::bible::bible_language(id);
                    let id_norm = crate::core::bible::fold(id);
                    let name_norm = crate::core::bible::fold(name);
                    let lang_norm = crate::core::bible::fold(lang);
                    id_norm.contains(&query_clean)
                        || name_norm.contains(&query_clean)
                        || lang_norm.contains(&query_clean)
                })
                .collect();
            (
                format!("Resultados da busca ({} encontrados)", results.len()),
                results,
            )
        } else if selected_lang == "ALL" {
            (
                format!("Todas as traduções ({})", catalog.len()),
                catalog.iter().collect(),
            )
        } else {
            let results: Vec<_> = catalog
                .iter()
                .filter(|(id, _)| crate::core::bible::bible_language(id) == selected_lang.as_str())
                .collect();
            let count = results.len();
            (
                format!("Traduções em {selected_lang} ({count} disponíveis)"),
                results,
            )
        };

        ui.label(
            RichText::new(&list_title)
                .size(12.5)
                .strong()
                .color(theme.text_secondary),
        );
        ui.add_space(4.0);

        // Scrollable List of Translations - Full Width & Generous Height (Shows 8-10 items at once)
        egui::ScrollArea::vertical()
            .id_salt("settings_bibles_versions_scroll")
            .max_height(340.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if filtered_versions.is_empty() {
                    ui.add_space(24.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Nenhuma tradução encontrada")
                                .size(12.0)
                                .color(theme.text_muted),
                        );
                    });
                    ui.add_space(24.0);
                    return;
                }

                ui.spacing_mut().item_spacing.y = 4.0;
                for (id, name) in filtered_versions {
                    let is_current = state.bible_version == *id;
                    let is_shortcut = service.bible_shortcuts.iter().any(|item| item == id);
                    let lang = crate::core::bible::bible_language(id);

                    let (row_rect, row_resp) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 38.0),
                        egui::Sense::click(),
                    );
                    let hover = ui.ctx().animate_bool(row_resp.id.with("hover"), row_resp.hovered());

                    let bg_color = if is_current {
                        theme.accent_soft
                    } else if hover > 0.05 {
                        theme.bg_card_hover.gamma_multiply(hover)
                    } else {
                        Color32::TRANSPARENT
                    };

                    let stroke = if is_current {
                        Stroke::new(1.0, theme.accent.linear_multiply(0.35))
                    } else if hover > 0.05 {
                        Stroke::new(1.0, theme.border_subtle.gamma_multiply(hover))
                    } else {
                        Stroke::NONE
                    };

                    if is_current || hover > 0.05 {
                        ui.painter().rect(row_rect, 7.0, bg_color, stroke, egui::StrokeKind::Inside);
                    }

                    let center_y = row_rect.center().y.round();

                    // ID Badge
                    let id_galley = ui.painter().layout_no_wrap(
                        id.to_string(),
                        FontId::proportional(11.0),
                        if is_current {
                            theme.text_on_accent
                        } else {
                            theme.text_primary
                        },
                    );
                    let badge_w = (id_galley.size().x + 14.0).max(46.0);
                    let badge_h = 22.0;
                    let badge_rect = egui::Rect::from_min_size(
                        egui::pos2(row_rect.min.x + 8.0, (center_y - badge_h / 2.0).round()),
                        egui::vec2(badge_w, badge_h),
                    );
                    let badge_bg = if is_current {
                        theme.accent
                    } else {
                        theme.bg_pill
                    };
                    let badge_stroke = if is_current {
                        Stroke::NONE
                    } else {
                        Stroke::new(1.0, theme.border_default)
                    };
                    ui.painter().rect(
                        badge_rect,
                        5.0,
                        badge_bg,
                        badge_stroke,
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().galley(
                        egui::pos2(
                            (badge_rect.center().x - id_galley.size().x / 2.0).round(),
                            (badge_rect.center().y - id_galley.size().y / 2.0).round(),
                        ),
                        id_galley,
                        if is_current {
                            theme.text_on_accent
                        } else {
                            theme.text_primary
                        },
                    );

                    // Right controls: Star button & Active button
                    let star_w = 28.0;
                    let star_h = 26.0;
                    let star_rect = egui::Rect::from_min_size(
                        egui::pos2(row_rect.max.x - star_w - 6.0, (center_y - star_h / 2.0).round()),
                        egui::vec2(star_w, star_h),
                    );
                    let star_resp = ui.interact(star_rect, row_resp.id.with("star"), egui::Sense::click());
                    let star_hover = ui.ctx().animate_bool(star_resp.id.with("hover"), star_resp.hovered());

                    let star_color = if is_shortcut {
                        theme.accent
                    } else if star_hover > 0.05 {
                        theme.accent
                    } else {
                        theme.text_muted
                    };
                    let star_bg = if is_shortcut {
                        theme.accent_soft
                    } else if star_hover > 0.05 {
                        theme.bg_pill
                    } else {
                        Color32::TRANSPARENT
                    };
                    let star_stroke = if is_shortcut {
                        Stroke::new(1.0, theme.accent.linear_multiply(0.5))
                    } else if star_hover > 0.05 {
                        Stroke::new(1.0, theme.border_subtle)
                    } else {
                        Stroke::NONE
                    };
                    ui.painter().rect(star_rect, 6.0, star_bg, star_stroke, egui::StrokeKind::Inside);
                    ui.painter().text(
                        star_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        STAR,
                        FontId::proportional(13.0),
                        star_color,
                    );

                    let star_resp = star_resp
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text(if is_shortcut {
                            "Remover atalho da barra rápida"
                        } else {
                            "Fixar atalho na barra rápida"
                        });

                    if star_resp.clicked() {
                        if is_shortcut {
                            service.bible_shortcuts.retain(|item| item != id);
                        } else {
                            service.bible_shortcuts.push((*id).clone());
                        }
                    }

                    // Action: In use indicator or Use button
                    let action_w = if is_current { 76.0 } else { 58.0 };
                    let action_rect = egui::Rect::from_min_size(
                        egui::pos2(star_rect.min.x - action_w - 6.0, (center_y - 12.0).round()),
                        egui::vec2(action_w, 24.0),
                    );

                    if is_current {
                        ui.painter().rect(
                            action_rect,
                            5.0,
                            theme.status_success_bg,
                            Stroke::NONE,
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().text(
                            action_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{CHECK_CIRCLE} Em uso"),
                            FontId::proportional(10.5),
                            theme.status_success_fg,
                        );
                    } else {
                        let use_resp = ui.interact(action_rect, row_resp.id.with("use"), egui::Sense::click());
                        let use_hover = ui.ctx().animate_bool(use_resp.id.with("hover"), use_resp.hovered());
                        let use_bg = if use_hover > 0.05 {
                            theme.bg_card_hover
                        } else {
                            theme.bg_pill
                        };
                        ui.painter().rect(
                            action_rect,
                            5.0,
                            use_bg,
                            Stroke::new(1.0, theme.border_default),
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().text(
                            action_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            language.text("workspace-use"),
                            FontId::proportional(11.0),
                            theme.text_primary,
                        );
                        if use_resp
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .on_hover_text(language.text("workspace-set-as-current-translation"))
                            .clicked()
                            || row_resp.clicked()
                        {
                            use_version = Some((*id).clone());
                        }
                    }

                    // Name + Optional Language tag
                    let name_x = badge_rect.max.x + 10.0;
                    let max_right = action_rect.min.x - 8.0;
                    let name_clip = egui::Rect::from_min_max(
                        egui::pos2(name_x, row_rect.min.y),
                        egui::pos2(max_right.max(name_x + 20.0), row_rect.max.y),
                    );

                    let display_text = if selected_lang == "ALL" || !query_clean.is_empty() {
                        format!("{name}  •  {lang}")
                    } else {
                        name.to_string()
                    };

                    ui.painter().with_clip_rect(name_clip).text(
                        egui::pos2(name_x, center_y),
                        egui::Align2::LEFT_CENTER,
                        display_text,
                        FontId::proportional(12.5),
                        if is_current {
                            theme.accent
                        } else {
                            theme.text_primary
                        },
                    );
                }
            });
    });

    if let Some(id) = use_version {
        state.bible_version = id.clone();
        state.bible.preview_key.clear();
        state.bible.search_key.clear();
        state.bible.scroll_to_verse = true;
        let service = draft
            .services
            .iter_mut()
            .find(|profile| profile.id == draft.active_service)
            .unwrap();
        service.bible_version = id.clone();
        if !service.bible_shortcuts.contains(&id) {
            service.bible_shortcuts.push(id.clone());
        }
        if !state.bible_shortcuts.contains(&id) {
            state.bible_shortcuts.push(id);
        }
    }

    ui.add_space(6.0);

    // Collapsible Import Card - Clean & Unobtrusive (takes 1 line by default, doesn't crowd out the bibles)
    let is_importing_active =
        state.bible.import_path.is_some() || !state.bible.import_error.is_empty();
    let import_open_id = ui.make_persistent_id("settings_import_bible_open");
    let mut import_open = ui
        .ctx()
        .data_mut(|d| d.get_temp::<bool>(import_open_id).unwrap_or(false))
        || is_importing_active;

    let mut import = false;

    card(ui, theme, |ui| {
        ui.horizontal(|ui| {
            let icon = if import_open { CARET_DOWN } else { CARET_RIGHT };
            let toggle_resp = ui
                .add(
                    egui::Button::new(
                        RichText::new(format!(
                            "{icon}  {}",
                            language.text("workspace-import-a-bible-translation")
                        ))
                        .size(13.0)
                        .strong()
                        .color(theme.text_primary),
                    )
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::NONE),
                )
                .on_hover_cursor(egui::CursorIcon::PointingHand);

            if toggle_resp.clicked() {
                import_open = !import_open;
                ui.ctx().data_mut(|d| d.insert_temp(import_open_id, import_open));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(language.text(
                        "workspace-the-translation-is-saved-in-the-app-and-available-offline",
                    ))
                    .size(11.0)
                    .color(theme.text_muted),
                );
            });
        });

        if import_open {
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Inputs: Sigla and Nome
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(crate::ui::i18n::text(ui, "settings-abbreviation-code"))
                            .size(11.5)
                            .strong()
                            .color(theme.text_secondary),
                    );
                    ui.add_space(2.0);
                    text_input(
                        ui,
                        "bible_import_sigla",
                        &mut state.bible.import_code,
                        crate::ui::i18n::text(ui, "settings-e-g-kjv-asv"),
                        Some(120.0),
                        theme,
                    );
                });

                ui.add_space(8.0);

                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(crate::ui::i18n::text(ui, "workspace-translation-name"))
                            .size(11.5)
                            .strong()
                            .color(theme.text_secondary),
                    );
                    ui.add_space(2.0);
                    text_input(
                        ui,
                        "bible_import_name",
                        &mut state.bible.import_name,
                        crate::ui::i18n::text(ui, "settings-e-g-king-james-version"),
                        None,
                        theme,
                    );
                });
            });

            ui.add_space(4.0);

            // File selection row
            let bible_picker_id = ui.make_persistent_id("bibles_file_picker");
            let bible_task = ui.ctx().data_mut(|data| {
                data.get_temp::<crate::services::tasks::BackgroundTask<Option<std::path::PathBuf>>>(
                    bible_picker_id,
                )
            });
            if let Some(task) = &bible_task {
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(33));
                if let Some(result) = task.take() {
                    ui.ctx().data_mut(|data| {
                        data.remove::<crate::services::tasks::BackgroundTask<Option<std::path::PathBuf>>>(
                            bible_picker_id,
                        )
                    });
                    if let Ok(Some(path)) = result {
                        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Bible");
                        if state.bible.import_code.is_empty() {
                            state.bible.import_code = stem
                                .chars()
                                .filter(|c| c.is_alphanumeric())
                                .take(12)
                                .collect::<String>()
                                .to_uppercase();
                        }
                        if state.bible.import_name.is_empty() {
                            state.bible.import_name = stem.into();
                        }
                        state.bible.import_path = Some(path);
                    }
                }
            }
            ui.horizontal(|ui| {
                if ui
                    .add_enabled_ui(bible_task.is_none(), |ui| {
                        styled_button(
                            ui,
                            Some(FILE_PLUS),
                            language.text("workspace-select-json-file"),
                            ButtonTone::Secondary,
                            theme,
                        )
                    })
                    .inner
                    .clicked()
                {
                    let filter = language.text("workspace-bible-json").to_owned();
                    let task = crate::services::tasks::BackgroundTask::spawn_with_repaint(
                        ui.ctx(),
                        move || {
                            Ok(rfd::FileDialog::new()
                                .add_filter(filter, &["json"])
                                .pick_file())
                        },
                    );
                    ui.ctx()
                        .data_mut(|data| data.insert_temp(bible_picker_id, task));
                }
                if bible_task.is_some() {
                    ui.spinner();
                }

                let mut clear_path = false;
                if let Some(path) = &state.bible.import_path {
                    egui::Frame::new()
                        .fill(theme.bg_pill)
                        .corner_radius(6.0)
                        .inner_margin(egui::vec2(8.0, 6.0))
                        .stroke(Stroke::new(1.0, theme.border_default))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!(
                                        "{FILE_TEXT} {}",
                                        path.file_name().unwrap_or_default().to_string_lossy()
                                    ))
                                    .size(11.5)
                                    .color(theme.text_primary),
                                );

                                if styled_button(ui, Some(X), "", ButtonTone::Ghost, theme).clicked() {
                                    clear_path = true;
                                }
                            });
                        });
                }
                if clear_path {
                    state.bible.import_path = None;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if state.bible.import_path.is_some() {
                        if styled_button(
                            ui,
                            Some(DOWNLOAD_SIMPLE),
                            language.text("workspace-import"),
                            ButtonTone::Primary,
                            theme,
                        )
                        .clicked()
                        {
                            import = true;
                        }
                    } else {
                        styled_button(
                            ui,
                            Some(DOWNLOAD_SIMPLE),
                            language.text("workspace-import"),
                            ButtonTone::Ghost,
                            theme,
                        );
                    }
                });
            });

            ui.add_space(2.0);

            egui::CollapsingHeader::new(
                RichText::new(language.text("workspace-accepted-format"))
                    .size(12.0)
                    .color(theme.text_secondary),
            )
            .show(ui, |ui| {
                ui.label(
                    RichText::new(
                        language.text(
                            "workspace-json-containing-the-66-books-in-biblical-order-each-book",
                        ),
                    )
                    .size(11.5)
                    .color(theme.text_secondary),
                );
                ui.add_space(2.0);
                ui.code(
                    r#"[{"name":"Genesis","chapters":[["No princípio criou Deus...", "..."]]}, …]"#,
                );
                ui.label(
                    RichText::new(language.text(
                        "workspace-objects-with-books-and-the-scrollmapper-format-are-also-supported",
                    ))
                    .size(11.0)
                    .color(theme.text_muted),
                );
            });

            if !state.bible.import_error.is_empty() {
                ui.colored_label(theme.status_danger, &state.bible.import_error);
            }
        }
    });

    if import {
        state.bible.import_error.clear();
        let result = (|| {
            let path = state
                .bible
                .import_path
                .as_ref()
                .ok_or(language.text("workspace-select-a-json-file"))?;
            if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 40 * 1024 * 1024 {
                return Err(language
                    .text("workspace-the-file-exceeds-the-40-mb-limit")
                    .into());
            }
            let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
            let translation = BibleTranslation::from_json(
                &bytes,
                &state.bible.import_code,
                &state.bible.import_name,
            )?;
            let db = db.ok_or(
                language.text("workspace-the-database-is-unavailable-to-save-this-translation"),
            )?;
            db.save_bible_translation(&translation).map_err(|e| {
                language.format("workspace-could-not-save", &[("error", &e.to_string())])
            })?;
            Ok::<_, String>(translation)
        })();
        match result {
            Ok(translation) => {
                let id = translation.id.clone();
                state.bible.library.insert(translation.clone());
                state.bible_version = id.clone();
                state.bible.preview_key.clear();
                state.bible.search_key.clear();
                state.bible.scroll_to_verse = true;
                state.bible.import_code.clear();
                state.bible.import_name.clear();
                state.bible.import_path = None;
                if !state.bible_shortcuts.contains(&id) {
                    state.bible_shortcuts.push(id.clone());
                }
                let service = draft
                    .services
                    .iter_mut()
                    .find(|profile| profile.id == draft.active_service)
                    .unwrap();
                service.bible_version = id.clone();
                if !service.bible_shortcuts.contains(&id) {
                    service.bible_shortcuts.push(id);
                }
                state.notify(language.format("workspace-imported", &[("name", &translation.name)]));
            }
            Err(error) => state.bible.import_error = error,
        }
    }
}
