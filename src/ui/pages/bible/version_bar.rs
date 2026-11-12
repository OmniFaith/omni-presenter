use crate::core::bible::version_code;
use crate::core::state::ControlPanelState;
use eframe::egui::{self, Color32, FontId, Rect, RichText, Stroke};
use egui_phosphor::regular::{BOOK_OPEN, BOOKS, CHECK_CIRCLE, GLOBE, MAGNIFYING_GLASS, PLUS, STAR, X};

pub(super) fn version_bar(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    theme: &crate::ui::theme::AppTheme,
) {
    egui::Frame::new()
        .fill(theme.bg_pill)
        .corner_radius(9.0)
        .inner_margin(3)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            egui::ScrollArea::horizontal()
                .id_salt("bible_versions")
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 3.0;
                        let catalog = state.bible.library.catalog();
                        let current = version_code(&state.bible_version);
                        let shortcuts = state.bible_shortcuts.clone();
                        let mut reorder = None;
                        for version in &shortcuts {
                            let version = version.clone();
                            let Some((_, name)) = catalog.iter().find(|(id, _)| id == &version)
                            else {
                                continue;
                            };
                            let name = name.clone();
                            let active = current == version;
                            let id = ui.make_persistent_id(("bible_ver_btn", &version));
                            let label = if crate::core::bible::bundled_meta(&version).is_some() {
                                version.split_once('_')
                                    .filter(|(prefix, _)| prefix.len() == 2)
                                    .map_or(version.as_str(), |(_, abbreviation)| abbreviation)
                            } else {
                                version.as_str()
                            };
                            let font = egui::FontId::proportional(11.0);
                            let text_width = ui.painter().layout_no_wrap(
                                label.to_owned(), font.clone(), theme.text_primary,
                            ).size().x;
                            let (rect, response) = ui
                                .allocate_exact_size(egui::vec2((text_width + 18.0).max(36.0), 27.0), egui::Sense::click_and_drag());
                            let response = response
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .on_hover_text(name.clone());

                            crate::ui::components::drag_drop::source(ui, &response, || {
                                crate::ui::components::drag_drop::DragPayload::new(
                                    crate::ui::components::drag_drop::DragSource::BibleVersions(vec![version.clone()]),
                                    label,
                                    &name,
                                    BOOK_OPEN,
                                )
                            });

                            if let Some((payload, after)) = crate::ui::components::drag_drop::reorder_target(
                                ui,
                                response.rect,
                                theme,
                                true,
                                |source| matches!(source, crate::ui::components::drag_drop::DragSource::BibleVersions(ids) if !ids.contains(&version)),
                            ) && let crate::ui::components::drag_drop::DragSource::BibleVersions(ids) = &payload.source {
                                reorder = Some((ids.clone(), version.clone(), after));
                            }

                            let hover = ui.ctx().animate_bool(id.with("hover"), response.hovered());
                            let down = ui.ctx().animate_bool(
                                id.with("down"),
                                response.is_pointer_button_down_on(),
                            );

                            let fill = if active {
                                theme.bg_card
                            } else if down > 0.05 {
                                theme
                                    .bg_card_active
                                    .lerp_to_gamma(theme.accent_soft, down * 0.45)
                            } else if hover > 0.05 {
                                egui::Color32::TRANSPARENT.lerp_to_gamma(theme.bg_card_hover, hover)
                            } else {
                                egui::Color32::TRANSPARENT
                            };

                            let stroke = if active {
                                Stroke::new(1.0, theme.border_default)
                            } else if down > 0.05 {
                                Stroke::new(1.0, theme.accent.linear_multiply(0.7))
                            } else if hover > 0.05 {
                                Stroke::new(
                                    1.0,
                                    theme
                                        .border_subtle
                                        .lerp_to_gamma(theme.accent.linear_multiply(0.55), hover),
                                )
                            } else {
                                Stroke::NONE
                            };

                            let text_color = if active {
                                theme.accent
                            } else if hover > 0.05 {
                                theme.text_primary.lerp_to_gamma(theme.accent, hover * 0.75)
                            } else {
                                theme.text_primary
                            };

                            ui.painter()
                                .rect(rect, 7.0, fill, stroke, egui::StrokeKind::Inside);
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                label,
                                font,
                                text_color,
                            );

                            response.context_menu(|ui| {
                                if ui.button(crate::ui::i18n::text(ui, "workspace-unpin-item")).clicked() {
                                    state.bible_shortcuts.retain(|id| id != &version);
                                    ui.close();
                                }
                            });
                            if response.clicked() {
                                state.bible_version = version;
                                state.bible.preview_key.clear();
                                state.bible.search_key.clear();
                                state.bible.scroll_to_verse = true;
                            }
                        }

                        if let Some((ids, target, after)) = reorder {
                            crate::core::interaction::move_items(
                                &mut state.bible_shortcuts,
                                &ids,
                                &target,
                                after,
                                |id| id,
                            );
                            if let Some(service) = state
                                .settings
                                .services
                                .iter_mut()
                                .find(|s| s.id == state.settings.active_service)
                            {
                                service.bible_shortcuts = state.bible_shortcuts.clone();
                            }
                        }
                        let picker_open_id = ui.make_persistent_id("bible_version_picker_window_open");
                        let mut is_open = ui
                            .ctx()
                            .data_mut(|d| d.get_temp::<bool>(picker_open_id).unwrap_or(false));

                        let plus_id = ui.make_persistent_id("bible_ver_plus");
                        let (plus_rect, plus) =
                            ui.allocate_exact_size(egui::vec2(28.0, 27.0), egui::Sense::click());
                        let plus = plus
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .on_hover_text(crate::ui::i18n::text(ui, "workspace-pin-translation-to-quick-bar"));

                        if plus.clicked() {
                            is_open = !is_open;
                            ui.ctx().data_mut(|d| d.insert_temp(picker_open_id, is_open));
                        }

                        let plus_hover =
                            ui.ctx().animate_bool(plus_id.with("hover"), plus.hovered());
                        let plus_down = ui
                            .ctx()
                            .animate_bool(plus_id.with("down"), plus.is_pointer_button_down_on());

                        let plus_fill = if is_open {
                            theme.accent_soft
                        } else if plus_down > 0.05 {
                            theme
                                .bg_card_active
                                .lerp_to_gamma(theme.accent_soft, plus_down * 0.45)
                        } else if plus_hover > 0.05 {
                            egui::Color32::TRANSPARENT
                                .lerp_to_gamma(theme.bg_card_hover, plus_hover)
                        } else {
                            egui::Color32::TRANSPARENT
                        };
                        let plus_stroke = if is_open {
                            Stroke::new(1.0, theme.accent)
                        } else if plus_hover > 0.05 {
                            Stroke::new(1.0, theme.border_subtle)
                        } else {
                            Stroke::NONE
                        };
                        let plus_color = if is_open || plus_hover > 0.05 {
                            theme.accent
                        } else {
                            theme.text_primary
                        };

                        ui.painter().rect(
                            plus_rect,
                            7.0,
                            plus_fill,
                            plus_stroke,
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().text(
                            plus_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            PLUS,
                            egui::FontId::proportional(12.0),
                            plus_color,
                        );

                        if is_open {
                            render_catalog_window(ui, state, theme, &catalog, &mut is_open);
                            ui.ctx().data_mut(|d| d.insert_temp(picker_open_id, is_open));
                        }
                    });
                });
        });
}

fn render_catalog_window(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    theme: &crate::ui::theme::AppTheme,
    catalog: &[(String, String)],
    is_open: &mut bool,
) {
    // Collect unique languages with counts
    let mut lang_counts: std::collections::BTreeMap<&str, usize> =
        std::collections::BTreeMap::new();
    for (id, _) in catalog {
        let lang = crate::core::bible::bible_language(id);
        *lang_counts.entry(lang).or_default() += 1;
    }

    // Sort order: Portuguese first, then English, then Spanish, then alphabetical
    let mut ordered_langs: Vec<(&str, usize)> = Vec::new();
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

    let selected_lang_id = ui.make_persistent_id("version_bar_selected_bible_lang");
    let mut selected_lang = ui.ctx().data_mut(|d| {
        d.get_temp::<String>(selected_lang_id)
            .unwrap_or_else(|| "Português".to_string())
    });

    if !lang_counts.contains_key(selected_lang.as_str()) {
        selected_lang = "Português".to_string();
    }

    let search_id = ui.make_persistent_id("bible_version_search");
    let mut search_query = ui
        .ctx()
        .data_mut(|d| d.get_temp::<String>(search_id).unwrap_or_default());

    let query_clean = crate::core::bible::fold(&search_query);

    let filtered_catalog: Vec<_> = catalog
        .iter()
        .filter(|(id, name)| {
            if query_clean.is_empty() {
                true
            } else {
                let lang = crate::core::bible::bible_language(id);
                let id_norm = crate::core::bible::fold(id);
                let name_norm = crate::core::bible::fold(name);
                let lang_norm = crate::core::bible::fold(lang);
                id_norm.contains(&query_clean)
                    || name_norm.contains(&query_clean)
                    || lang_norm.contains(&query_clean)
            }
        })
        .collect();

    let mut close_window = false;
    let mut select_version: Option<String> = None;
    let center = ui.ctx().input(|i| i.viewport_rect().center());

    egui::Window::new("bible_version_selector")
        .id(ui.make_persistent_id("bible_version_selector_window"))
        .open(is_open)
        .title_bar(false)
        .collapsible(false)
        .resizable(true)
        .default_size(egui::vec2(760.0, 520.0))
        .min_size(egui::vec2(480.0, 320.0))
        .max_size(egui::vec2(1600.0, 1100.0))
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(center)
        .frame(
            egui::Frame::new()
                .fill(theme.bg_panel)
                .stroke(Stroke::new(1.0, theme.border_default))
                .corner_radius(12.0)
                .inner_margin(12.0)
                .shadow(egui::epaint::Shadow {
                    offset: [0, 8],
                    blur: 32,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(80),
                }),
        )
        .show(ui.ctx(), |ui| {
            if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
                close_window = true;
            }

            // Compact unified header
            render_catalog_header(
                ui,
                theme,
                catalog.len(),
                ordered_langs.len(),
                &mut close_window,
            );

            // Search Bar
            ui.add_space(6.0);
            render_search_bar(ui, theme, &mut search_query, search_id);

            ui.add_space(6.0);
            let sep_y = ui.cursor().top();
            ui.painter().hline(
                ui.available_rect_before_wrap().x_range(),
                sep_y,
                Stroke::new(1.0, theme.border_subtle),
            );
            ui.add_space(6.0);

            let content_h = ui.available_height().max(180.0);

            if query_clean.is_empty() {
                // Master-Detail Two Column View
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), content_h),
                    egui::Layout::left_to_right(egui::Align::Min),
                    |ui| {
                        // Left column: Languages
                        let left_w = 190.0;
                        ui.vertical(|ui| {
                            ui.set_width(left_w);
                            ui.set_height(content_h);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(GLOBE).size(12.0).color(theme.accent));
                                ui.label(
                                    RichText::new(format!("Idiomas ({})", ordered_langs.len()))
                                        .size(11.5)
                                        .strong()
                                        .color(theme.text_secondary),
                                );
                            });
                            ui.add_space(4.0);

                            let lang_scroll_h = (content_h - 26.0).max(100.0);
                            egui::Frame::new()
                                .fill(theme.bg_pill)
                                .corner_radius(8.0)
                                .inner_margin(4.0)
                                .stroke(Stroke::new(1.0, theme.border_subtle))
                                .show(ui, |ui| {
                                    egui::ScrollArea::vertical()
                                        .id_salt("version_bar_lang_scroll")
                                        .max_height(lang_scroll_h)
                                        .auto_shrink([false, false])
                                        .show(ui, |ui| {
                                            ui.spacing_mut().item_spacing.y = 2.0;
                                            for (lang, count) in &ordered_langs {
                                                let is_selected = selected_lang.as_str() == *lang;
                                                let resp = render_lang_item(
                                                    ui,
                                                    theme,
                                                    lang,
                                                    *count,
                                                    is_selected,
                                                );
                                                if resp.clicked() {
                                                    selected_lang = (*lang).to_string();
                                                    ui.ctx().data_mut(|d| {
                                                        d.insert_temp(selected_lang_id, selected_lang.clone())
                                                    });
                                                }
                                            }
                                        });
                                });
                        });

                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // Right column: Versions in selected language
                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(content_h);
                            let current_versions: Vec<_> = catalog
                                .iter()
                                .filter(|(id, _)| crate::core::bible::bible_language(id) == selected_lang.as_str())
                                .collect();

                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("Traduções em {selected_lang}"))
                                        .size(12.5)
                                        .strong()
                                        .color(theme.text_primary),
                                );
                                ui.label(
                                    RichText::new(format!("({} disponíveis)", current_versions.len()))
                                        .size(11.0)
                                        .color(theme.text_muted),
                                );
                            });

                            ui.add_space(4.0);

                            let ver_scroll_h = (content_h - 26.0).max(100.0);
                            egui::ScrollArea::vertical()
                                .id_salt("version_bar_versions_scroll")
                                .max_height(ver_scroll_h)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing.y = 3.0;
                                    for (id, name) in current_versions {
                                        render_version_row(
                                            ui,
                                            theme,
                                            state,
                                            id,
                                            name,
                                            &selected_lang,
                                            false,
                                            &mut select_version,
                                        );
                                    }
                                });
                        });
                    },
                );
            } else {
                // Search Results across all languages
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), content_h),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("Resultados da busca ({})", filtered_catalog.len()))
                                    .size(12.0)
                                    .strong()
                                    .color(theme.accent),
                            );
                        });
                        ui.add_space(4.0);

                        let search_scroll_h = (content_h - 26.0).max(100.0);
                        egui::ScrollArea::vertical()
                            .id_salt("version_bar_search_scroll")
                            .max_height(search_scroll_h)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                if filtered_catalog.is_empty() {
                                    ui.add_space(32.0);
                                    ui.vertical_centered(|ui| {
                                        ui.label(
                                            RichText::new(MAGNIFYING_GLASS)
                                                .size(24.0)
                                                .color(theme.text_muted),
                                        );
                                        ui.add_space(6.0);
                                        ui.label(
                                            RichText::new("Nenhuma tradução encontrada")
                                                .size(12.0)
                                                .color(theme.text_muted),
                                        );
                                    });
                                    ui.add_space(32.0);
                                    return;
                                }

                                ui.spacing_mut().item_spacing.y = 3.0;
                                for (id, name) in &filtered_catalog {
                                    let lang = crate::core::bible::bible_language(id);
                                    render_version_row(
                                        ui,
                                        theme,
                                        state,
                                        id,
                                        name,
                                        lang,
                                        true,
                                        &mut select_version,
                                    );
                                }
                            });
                    },
                );
            }
        });

    if let Some(id) = select_version {
        if !state.bible_shortcuts.contains(&id) {
            state.bible_shortcuts.push(id.clone());
        }
        state.bible_version = id;
        state.bible.preview_key.clear();
        state.bible.search_key.clear();
        state.bible.scroll_to_verse = true;
        close_window = true;
    }

    if close_window {
        *is_open = false;
    }
}

fn render_catalog_header(
    ui: &mut egui::Ui,
    theme: &crate::ui::theme::AppTheme,
    total_versions: usize,
    total_langs: usize,
    close_window: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.label(RichText::new(BOOKS).size(13.5).color(theme.accent));
        ui.label(
            RichText::new("Traduções da Bíblia")
                .size(13.0)
                .strong()
                .color(theme.text_primary),
        );
        ui.label(
            RichText::new(format!("• {total_versions} versões em {total_langs} idiomas"))
                .size(10.5)
                .color(theme.text_muted),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let (close_rect, close_resp) =
                ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
            let close_hover =
                ui.ctx().animate_bool(close_resp.id.with("hover"), close_resp.hovered());
            if close_hover > 0.05 {
                ui.painter().rect_filled(
                    close_rect,
                    5.0,
                    theme.bg_card_hover.gamma_multiply(close_hover),
                );
            }
            ui.painter().text(
                close_rect.center(),
                egui::Align2::CENTER_CENTER,
                X,
                FontId::proportional(11.5),
                if close_hover > 0.05 {
                    theme.text_primary
                } else {
                    theme.text_muted
                },
            );
            if close_resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                *close_window = true;
            }
        });
    });
}

fn render_search_bar(
    ui: &mut egui::Ui,
    theme: &crate::ui::theme::AppTheme,
    search_query: &mut String,
    search_id: egui::Id,
) {
    let search_h = 32.0;
    let (search_rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), search_h),
        egui::Sense::hover(),
    );

    let text_id = search_id.with("text");
    let is_search_focused = ui.ctx().memory(|m| m.has_focus(text_id));

    let border_color = if is_search_focused {
        theme.accent
    } else {
        theme.border_default
    };
    ui.painter().rect(
        search_rect,
        8.0,
        theme.bg_pill,
        Stroke::new(1.0, border_color),
        egui::StrokeKind::Inside,
    );

    let icon_x = search_rect.min.x + 10.0;
    let center_y = search_rect.center().y.round();

    // Magnifying glass icon perfectly aligned with center_y
    ui.painter().text(
        egui::pos2(icon_x, center_y),
        egui::Align2::LEFT_CENTER,
        MAGNIFYING_GLASS,
        FontId::proportional(12.0),
        if is_search_focused {
            theme.accent
        } else {
            theme.text_muted
        },
    );

    let clear_w = if !search_query.is_empty() { 26.0 } else { 0.0 };
    if !search_query.is_empty() {
        let clear_rect = Rect::from_center_size(
            egui::pos2(search_rect.max.x - 16.0, center_y),
            egui::vec2(20.0, 20.0),
        );
        let clear_resp = ui.interact(
            clear_rect,
            search_id.with("clear"),
            egui::Sense::click(),
        );
        let clear_hover = ui.ctx().animate_bool(clear_resp.id.with("hover"), clear_resp.hovered());
        if clear_hover > 0.05 {
            ui.painter().rect_filled(
                clear_rect,
                4.0,
                theme.bg_card_hover.gamma_multiply(clear_hover),
            );
        }
        ui.painter().text(
            clear_rect.center(),
            egui::Align2::CENTER_CENTER,
            X,
            FontId::proportional(11.0),
            if clear_hover > 0.05 {
                theme.text_primary
            } else {
                theme.text_muted
            },
        );
        if clear_resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            search_query.clear();
            ui.ctx().data_mut(|d| d.insert_temp(search_id, String::new()));
        }
    }

    // TextEdit placed symmetrically and vertically centered on center_y
    let edit_h = 24.0;
    let edit_y = (center_y - edit_h / 2.0).round();
    let text_rect = Rect::from_min_max(
        egui::pos2(icon_x + 20.0, edit_y),
        egui::pos2(search_rect.max.x - clear_w - 6.0, edit_y + edit_h),
    );
    let edit_resp = ui.put(
        text_rect,
        egui::TextEdit::singleline(search_query)
            .id(text_id)
            .frame(egui::Frame::NONE)
            .align(egui::Align2::LEFT_CENTER)
            .margin(egui::Margin::symmetric(0, 2))
            .hint_text(
                RichText::new("Buscar tradução, sigla ou idioma (ex: ARA, NVI, KJV, Alemão)...")
                    .size(11.5)
                    .color(theme.text_muted),
            )
            .font(FontId::proportional(11.5))
            .text_color(theme.text_primary),
    );
    if edit_resp.changed() {
        ui.ctx().data_mut(|d| d.insert_temp(search_id, search_query.clone()));
    }
}

fn render_lang_item(
    ui: &mut egui::Ui,
    theme: &crate::ui::theme::AppTheme,
    lang: &str,
    count: usize,
    is_selected: bool,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 28.0),
        egui::Sense::click(),
    );
    let hover = ui.ctx().animate_bool(resp.id.with("hover"), resp.hovered());

    let fill = if is_selected {
        theme.accent_soft
    } else if hover > 0.05 {
        theme.bg_card_hover.gamma_multiply(hover)
    } else {
        Color32::TRANSPARENT
    };

    let stroke = if is_selected {
        Stroke::new(1.0, theme.accent)
    } else if hover > 0.05 {
        Stroke::new(1.0, theme.border_subtle.gamma_multiply(hover))
    } else {
        Stroke::NONE
    };

    ui.painter().rect(rect, 6.0, fill, stroke, egui::StrokeKind::Inside);

    let center_y = rect.center().y.round();

    let text_color = if is_selected {
        theme.accent
    } else {
        theme.text_primary
    };

    ui.painter().text(
        egui::pos2(rect.min.x + 8.0, center_y),
        egui::Align2::LEFT_CENTER,
        lang,
        FontId::proportional(11.5),
        text_color,
    );

    let count_color = if is_selected {
        theme.accent
    } else {
        theme.text_muted
    };
    ui.painter().text(
        egui::pos2(rect.max.x - 8.0, center_y),
        egui::Align2::RIGHT_CENTER,
        format!("{count}"),
        FontId::proportional(10.5),
        count_color,
    );

    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn render_version_row(
    ui: &mut egui::Ui,
    theme: &crate::ui::theme::AppTheme,
    state: &mut ControlPanelState,
    id: &str,
    name: &str,
    lang: &str,
    show_lang_suffix: bool,
    select_version: &mut Option<String>,
) {
    let is_pinned = state.bible_shortcuts.iter().any(|x| x == id);
    let is_current = state.bible_version == id;
    let row_h = 36.0;
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), row_h),
        egui::Sense::click(),
    );
    let hover = ui.ctx().animate_bool(resp.id.with("hover"), resp.hovered());

    let bg_color = if is_current {
        theme.accent_soft
    } else if hover > 0.05 {
        theme.bg_card_hover.gamma_multiply(hover)
    } else {
        Color32::TRANSPARENT
    };

    let stroke = if is_current {
        Stroke::new(1.0, theme.accent.linear_multiply(0.4))
    } else if hover > 0.05 {
        Stroke::new(1.0, theme.border_subtle.gamma_multiply(hover))
    } else {
        Stroke::NONE
    };

    if is_current || hover > 0.05 {
        ui.painter().rect(rect, 7.0, bg_color, stroke, egui::StrokeKind::Inside);
    }

    let center_y = rect.center().y.round();

    // ID Badge - vertically centered in row
    let badge_text_color = if is_current {
        theme.text_on_accent
    } else {
        theme.text_primary
    };
    let id_galley = ui.painter().layout_no_wrap(
        id.to_string(),
        FontId::proportional(11.0),
        badge_text_color,
    );
    let badge_w = (id_galley.size().x + 14.0).max(44.0);
    let badge_h = 22.0;
    let badge_rect = Rect::from_min_size(
        egui::pos2(rect.min.x + 8.0, (center_y - badge_h / 2.0).round()),
        egui::vec2(badge_w, badge_h),
    );
    let badge_bg = if is_current { theme.accent } else { theme.bg_pill };
    let badge_stroke = if is_current {
        Stroke::NONE
    } else {
        Stroke::new(1.0, theme.border_default)
    };

    ui.painter().rect(badge_rect, 5.0, badge_bg, badge_stroke, egui::StrokeKind::Inside);
    ui.painter().galley(
        egui::pos2(
            (badge_rect.center().x - id_galley.size().x / 2.0).round(),
            (badge_rect.center().y - id_galley.size().y / 2.0).round(),
        ),
        id_galley,
        badge_text_color,
    );

    // Right action star button - vertically centered in row
    let star_w = 30.0;
    let star_h = 24.0;
    let star_rect = Rect::from_min_size(
        egui::pos2(rect.max.x - star_w - 6.0, (center_y - star_h / 2.0).round()),
        egui::vec2(star_w, star_h),
    );
    let star_resp = ui.interact(star_rect, resp.id.with("star"), egui::Sense::click());
    let star_hover = ui.ctx().animate_bool(star_resp.id.with("hover"), star_resp.hovered());

    // Em uso indicator if active - vertically centered in row
    let in_use_w = if is_current { 72.0 } else { 0.0 };
    if is_current {
        let in_use_h = 20.0;
        let in_use_rect = Rect::from_min_size(
            egui::pos2(star_rect.min.x - in_use_w - 4.0, (center_y - in_use_h / 2.0).round()),
            egui::vec2(in_use_w, in_use_h),
        );
        ui.painter().rect(in_use_rect, 5.0, theme.status_success_bg, Stroke::NONE, egui::StrokeKind::Inside);
        ui.painter().text(
            in_use_rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("{CHECK_CIRCLE} Em uso"),
            FontId::proportional(10.5),
            theme.status_success_fg,
        );
    }

    // Star icon
    let star_color = if is_pinned {
        theme.accent
    } else if star_hover > 0.05 {
        theme.accent
    } else {
        theme.text_muted
    };
    let star_bg = if star_hover > 0.05 { theme.bg_pill } else { Color32::TRANSPARENT };
    if star_hover > 0.05 {
        ui.painter().rect(star_rect, 5.0, star_bg, Stroke::new(1.0, theme.border_subtle), egui::StrokeKind::Inside);
    }
    ui.painter().text(
        star_rect.center(),
        egui::Align2::CENTER_CENTER,
        STAR,
        FontId::proportional(13.0),
        star_color,
    );

    let star_resp = star_resp
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(if is_pinned {
            "Remover atalho da barra rápida"
        } else {
            "Fixar atalho na barra rápida"
        });

    if star_resp.clicked() {
        if is_pinned {
            state.bible_shortcuts.retain(|x| x != id);
        } else {
            state.bible_shortcuts.push(id.to_string());
        }
    } else if resp.clicked() {
        *select_version = Some(id.to_string());
    }

    // Name + optional language suffix with clip rect - perfectly centered vertically!
    let name_x = badge_rect.max.x + 10.0;
    let max_right = if is_current {
        star_rect.min.x - in_use_w - 10.0
    } else {
        star_rect.min.x - 8.0
    };
    let name_clip = Rect::from_min_max(
        egui::pos2(name_x, rect.min.y),
        egui::pos2(max_right.max(name_x + 20.0), rect.max.y),
    );
    let text_color = if is_current { theme.accent } else { theme.text_primary };
    let display_text = if show_lang_suffix {
        format!("{name}  •  {lang}")
    } else {
        name.to_string()
    };
    ui.painter().with_clip_rect(name_clip).text(
        egui::pos2(name_x, center_y),
        egui::Align2::LEFT_CENTER,
        display_text,
        FontId::proportional(12.5),
        text_color,
    );

    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(format!("{name} ({lang}) - Clique para usar"));
}
