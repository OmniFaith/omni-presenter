use crate::core::layout::ContainerKind;
use crate::core::markup;
use crate::core::models::QuickItemKind;
use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use crate::ui::components::presentation_dock::render_slide_card;
use crate::ui::components::{
    centered_horizontal, context_menu_danger_item, context_menu_disabled_item, context_menu_header,
    context_menu_item, context_menu_raw_item, context_menu_separator, context_menu_submenu, render_header_bar, toolbar_icon_button,
};
use eframe::egui::{self, Margin, Stroke};
use egui_phosphor::regular::{
    ARROW_DOWN, ARROW_UP, COPY, EYE, FILE_PLUS, GEAR, HEART, LIST_BULLETS, PENCIL_SIMPLE, PLAY, SQUARES_FOUR, TAG, TRASH,
};

pub fn render_lyrics_column(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    slide_deck: &mut SlideDeck,
    is_presentation_open: &mut bool,
    db: Option<&AppDb>,
) {
    if slide_deck.preview_design.is_some() {
        crate::ui::pages::design::library::render_preview(
            ui,
            state,
            slide_deck,
            is_presentation_open,
        );
        return;
    }
    let theme = state.theme.clone();
    let language = crate::ui::i18n::language(ui.ctx());
    let is_bible = slide_deck.is_bible()
        || state.active_sidebar_page == crate::core::state::SidebarPage::Bible
        || (state.active_sidebar_page == crate::core::state::SidebarPage::Home && state.selected_tab == 3)
        || state
            .selected_playlist_index
            .and_then(|idx| state.playlist.get(idx))
            .is_some_and(|item| item.kind == crate::core::models::PlaylistItemKind::Bible);

    // Barra de título com drag handle
    render_header_bar(ui, ContainerKind::Lyrics, "workspace-lyrics-preview", &theme);
    ui.add_space(4.0);

    // Botões embaixo da barra de título (centralizados)
    centered_horizontal(ui, "lyrics_toolbar", 160.0, |ui| {
        if toolbar_icon_button(ui, PLAY, language.text("workspace-present-song-live"), &theme).clicked() {
            slide_deck.set_is_song(true);
            slide_deck.select_preview_slide(0);
            slide_deck.project_preview();
            state.apply_theme_to_live();
            *is_presentation_open = true;
            state.show_presentation_dock = state.settings.operator().show_dock;
        }
        let cards = state.lyrics_cards_layout;
        let (icon, tooltip) = if cards {
            (LIST_BULLETS, language.text("workspace-list-layout"))
        } else {
            (SQUARES_FOUR, language.text("workspace-card-layout"))
        };
        if theme
            .icon_button(ui, icon, tooltip, cards, egui::vec2(28.0, 28.0))
            .clicked()
        {
            state.lyrics_cards_layout = !cards;
            state.capture_profiles();
        }
        if is_bible {
            if toolbar_icon_button(
                ui,
                FILE_PLUS,
                language.text("workspace-create-text-from-bible"),
                &theme,
            )
            .clicked()
            {
                let clean_slides: Vec<String> = slide_deck
                    .slides()
                    .iter()
                    .map(|slide| {
                        crate::core::slides::strip_directive_commands(
                            &crate::core::slides::strip_comments(slide),
                        )
                        .trim()
                        .to_string()
                    })
                    .filter(|s| !s.is_empty())
                    .collect();
                let clean_content = clean_slides.join("\n\n");
                state.open_text_editor(None, slide_deck.title(), "", &clean_content);
            }
        } else if toolbar_icon_button(
            ui,
            PENCIL_SIMPLE,
            language.text("workspace-edit-this-song-s-lyrics"),
            &theme,
        )
        .clicked()
        {
            state.song_editor.title = slide_deck.title().to_owned();
            state.song_editor.artist = slide_deck.artist().to_owned();
            state.song_editor.text = slide_deck.raw_text().to_owned();
            state.song_editor.folder = slide_deck
                .current_song()
                .map(|s| s.folder.clone())
                .unwrap_or_default();
            state.song_editor.tags = slide_deck
                .current_song()
                .map(|s| s.tags.clone())
                .unwrap_or_default();
            state.song_editor.open = true;
        }
        if toolbar_icon_button(ui, TRASH, language.text("workspace-remove-from-playlist-label"), &theme).clicked() {
            let title = slide_deck.title().to_owned();
            let before = state.playlist.len();
            state.playlist.retain(|item| {
                item.kind != crate::core::models::PlaylistItemKind::Song || item.title != title
            });
            state.selected_playlist_index = None;
            let removed = before - state.playlist.len();
            if removed > 0 {
                state.notify(language.format("workspace-removed-from-the-playlist", &[("title", &title)]));
            } else {
                state.notify(language.format("workspace-is-not-in-the-playlist", &[("title", &title)]));
            }
        }
        if toolbar_icon_button(ui, HEART, language.text("chat-pin-to-quick-bar"), &theme).clicked() {
            let title = slide_deck.title().to_owned();
            let artist = slide_deck.artist().to_owned();
            let text = slide_deck.raw_text().to_owned();
            state.add_quick_item(title.clone(), artist, QuickItemKind::Song, text);
            state.notify(language.format("workspace-pinned-to-the-quick-bar", &[("title", &title)]));
        }
    });

    ui.add_space(6.0);

    // Song Title and Artist banner
    ui.vertical_centered(|ui| {
        ui.add(
            egui::Label::new(
                egui::RichText::new(markup::strip(slide_deck.title()))
                    .strong()
                    .size(13.5)
                    .color(theme.text_primary),
            )
            .wrap(),
        );
        if !slide_deck.artist().is_empty() {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(slide_deck.artist())
                        .size(11.5)
                        .color(theme.text_secondary),
                )
                .wrap(),
            );
        }

        if let Some(db) = db
            && let Some(current) = slide_deck.current_song().and_then(|song| song.id)
            && let Ok(versions) = db.song_versions(current)
            && versions.len() > 1
        {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(crate::ui::i18n::text(ui, "workspace-version-variant-3"))
                        .size(10.5)
                        .color(theme.text_muted),
                );
                let selected = slide_deck.title().to_owned();
                egui::ComboBox::from_id_salt("lyrics_song_version")
                    .selected_text(
                        egui::RichText::new(selected.clone())
                            .size(11.0)
                            .color(theme.text_primary),
                    )
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for version in &versions {
                            let is_selected = version.title == selected;
                            if ui
                                .selectable_label(is_selected, version.title.clone())
                                .clicked()
                                && !is_selected
                                && let Some(index) = slide_deck
                                    .library()
                                    .iter()
                                    .position(|song| song.title == version.title)
                            {
                                slide_deck.select_song(index);
                            }
                        }
                    });
            });
        }
    });

    ui.add_space(10.0);

    // Strophes list: "Parte da letra selecionada" prominently highlighted with right-click context menu
    egui::ScrollArea::vertical()
        .id_salt("lyrics_scroll_area")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let slides = slide_deck.slides().to_vec();
            let preview_idx = slide_deck.preview_slide_index();
            let live_idx = slide_deck.live_index();
            let is_preview_song_live = slide_deck.is_previewing_live_song();
            let cards = state.lyrics_cards_layout;
            let preview_theme = crate::services::media::preview_theme(state, slide_deck);
            let width = ui.available_width().max(1.0);
            let columns = if cards {
                ((width + 12.0) / 192.0).floor().max(1.0) as usize
            } else {
                1
            };
            let cell_width = ((width - 12.0 * (columns - 1) as f32) / columns as f32).max(1.0);

            let mut action_project = None;
            let mut action_preview = None;
            let mut action_edit = None;
            let mut action_tag = None;
            let mut action_move_up = None;
            let mut action_move_down = None;
            let mut action_duplicate = None;
            let mut action_delete = None;

            egui::Grid::new(("lyrics_slides", cards))
                .num_columns(columns)
                .spacing(egui::vec2(12.0, 12.0))
                .min_col_width(cell_width)
                .max_col_width(cell_width)
                .show(ui, |ui| {
                    for (index, strophe) in slides.iter().enumerate() {
                        let is_preview = index == preview_idx;
                        let is_live =
                            index == live_idx && *is_presentation_open && is_preview_song_live;

                        let card_id = ui.make_persistent_id(("strophe", slide_deck.title(), index));
                        let was_hovered = ui
                            .ctx()
                            .data_mut(|d| d.get_temp::<bool>(card_id))
                            .unwrap_or(false);
                        let hover = ui.ctx().animate_bool(card_id.with("hover"), was_hovered);
                        let selected = ui.ctx().animate_bool(card_id.with("selected"), is_preview);
                        let card_bg = if is_preview {
                            theme.bg_card_active
                        } else if is_live {
                            theme.accent_soft
                        } else {
                            theme.bg_card.lerp_to_gamma(theme.bg_card_hover, hover)
                        };
                        let card_bg = card_bg.lerp_to_gamma(theme.bg_card_active, selected);

                        let card_border = if is_live {
                            Stroke::new(1.0, theme.accent)
                        } else {
                            Stroke::NONE
                        };

                        let text_color = if is_preview || is_live {
                            theme.text_primary
                        } else {
                            theme.text_secondary
                        };

                        let strophe_frame = egui::Frame::new()
                            .fill(card_bg)
                            .stroke(card_border)
                            .corner_radius(theme.radius_lg)
                            .inner_margin(Margin::symmetric(14, 10));

                        let is_song_mode = slide_deck.is_song();
                        let interact = if cards {
                            let response = render_slide_card(
                                ui,
                                &theme,
                                &preview_theme,
                                strophe,
                                index,
                                is_preview,
                                false,
                                state.presentation_aspect_ratio.clamp(0.5, 3.5),
                                true,
                                cell_width,
                                is_song_mode,
                            );
                            if is_live {
                                let rect = egui::Rect::from_min_size(
                                    response.rect.min,
                                    egui::vec2(
                                        cell_width,
                                        (cell_width
                                            / state.presentation_aspect_ratio.clamp(0.5, 3.5))
                                        .round(),
                                    ),
                                );
                                ui.painter().rect_stroke(
                                    rect,
                                    theme.radius_md,
                                    Stroke::new(2.0, theme.status_success_fg),
                                    egui::StrokeKind::Inside,
                                );
                            }
                            response
                        } else {
                            let res = strophe_frame.show(ui, |ui| {
                                let available_w = (cell_width - 28.0).max(1.0);
                                ui.set_width(available_w);
                                ui.horizontal_top(|ui| {
                                    if is_song_mode {
                                        if index == 0 {
                                            ui.label(
                                                egui::RichText::new(crate::ui::i18n::text(ui, "workspace-title"))
                                                    .strong()
                                                    .size(10.5)
                                                    .color(theme.accent),
                                            );
                                            ui.add_space(6.0);
                                        } else {
                                            ui.label(
                                                egui::RichText::new(index.to_string())
                                                    .strong()
                                                    .size(11.0)
                                                    .color(theme.text_muted),
                                            );
                                            ui.add_space(8.0);
                                        }
                                    } else {
                                        ui.label(
                                            egui::RichText::new((index + 1).to_string())
                                                .strong()
                                                .size(11.0)
                                                .color(theme.text_muted),
                                        );
                                        ui.add_space(8.0);
                                    }
                                    ui.vertical(|ui| {
                                        let content_w = (available_w - 36.0).max(40.0);
                                        ui.set_max_width(content_w);
                                        let tag_opt = crate::core::slides::extract_strophe_tag(strophe);
                                        if let Some(tag) = &tag_opt {
                                            let [r, g, b] = crate::core::preferences::StropheTagDefinition::color_for(
                                                &state.settings.strophe_tags,
                                                tag,
                                                [100, 116, 139],
                                            );
                                            let badge_color = egui::Color32::from_rgb(r, g, b);
                                            let display_tag = crate::core::preferences::StropheTagDefinition::localize_tag_name(tag, language);
                                            ui.horizontal(|ui| {
                                                let (rect, _) = ui.allocate_exact_size(egui::vec2(0.0, 16.0), egui::Sense::hover());
                                                let text_galley = ui.painter().layout_no_wrap(
                                                    display_tag.to_string(),
                                                    egui::FontId::proportional(10.5),
                                                    badge_color,
                                                );
                                                let badge_w = text_galley.size().x + 12.0;
                                                let badge_rect = egui::Rect::from_min_size(rect.min, egui::vec2(badge_w, 16.0));
                                                ui.painter().rect_filled(
                                                    badge_rect,
                                                    4.0,
                                                    badge_color.linear_multiply(0.15),
                                                );
                                                ui.painter().text(
                                                    badge_rect.center(),
                                                    egui::Align2::CENTER_CENTER,
                                                    display_tag,
                                                    egui::FontId::proportional(10.5),
                                                    badge_color,
                                                );
                                                ui.add_space(badge_w + 4.0);
                                            });
                                            ui.add_space(2.0);
                                        }

                                        let content_w = (available_w - 36.0).max(40.0);
                                        let resolved_strophe = crate::core::breaks::resolve_conditional_breaks(
                                            strophe,
                                            state.presentation_aspect_ratio,
                                        );
                                        for line in resolved_strophe.lines() {
                                            let trimmed = line.trim();
                                            if trimmed.starts_with("#*") {
                                                continue;
                                            }
                                            if crate::core::markup::is_directive_command(trimmed) {
                                                continue;
                                            }
                                            if is_bible && trimmed.starts_with("//") {
                                                continue;
                                            }
                                            if trimmed.starts_with("//") {
                                                ui.add(
                                                    egui::Label::new(
                                                        egui::RichText::new(trimmed)
                                                            .size(11.5)
                                                            .italics()
                                                            .color(theme.text_muted),
                                                    )
                                                    .wrap(),
                                                );
                                            } else {
                                                let (main_text, comment_opt) = match line.find("//") {
                                                    Some(idx) => {
                                                        let m = &line[..idx];
                                                        let c = if is_bible { None } else { Some(&line[idx..]) };
                                                        (m, c)
                                                    }
                                                    None => (line, None),
                                                };

                                                if is_bible && main_text.trim().is_empty() {
                                                    continue;
                                                }

                                                let mut job = egui::text::LayoutJob::default();
                                                job.wrap.max_width = content_w;

                                                let runs = markup::runs(main_text);
                                                for run in runs {
                                                    let mut font_id = egui::FontId::proportional(13.0);
                                                    if run.style.emphasis {
                                                        font_id.family = egui::FontFamily::Name("inter_bold".into());
                                                    }
                                                    let run_color = run.style.color.unwrap_or(text_color);
                                                    let format = egui::TextFormat {
                                                        font_id,
                                                        color: run_color,
                                                        italics: run.style.italic,
                                                        underline: if run.style.underline {
                                                            Stroke::new(1.0, run_color)
                                                        } else {
                                                            Stroke::NONE
                                                        },
                                                        strikethrough: if run.style.strikethrough {
                                                            Stroke::new(1.0, run_color)
                                                        } else {
                                                            Stroke::NONE
                                                        },
                                                        ..Default::default()
                                                    };
                                                    job.append(run.text, 0.0, format);
                                                }

                                                if let Some(comment_text) = comment_opt {
                                                    let comment_format = egui::TextFormat {
                                                        font_id: egui::FontId::proportional(11.5),
                                                        color: theme.text_muted,
                                                        italics: true,
                                                        ..Default::default()
                                                    };
                                                    job.append(comment_text, 0.0, comment_format);
                                                }

                                                ui.label(job);
                                            }
                                        }
                                    });
                                });
                            });

                            res.response.interact(egui::Sense::click())
                        };
                        ui.ctx()
                            .data_mut(|d| d.insert_temp(card_id, interact.hovered()));
                        if interact.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if interact.clicked() {
                            slide_deck.select_preview_slide(index);
                        }
                        if interact.double_clicked() {
                            slide_deck.project_preview_slide(index);
                            state.apply_theme_to_live();
                            *is_presentation_open = true;
                            state.show_presentation_dock = state.settings.operator().show_dock;
                        }

                        // Menu de contexto com clique com botão direito na estrofe
                        interact.context_menu(|ui| {
                            context_menu_header(ui);
                            let is_title_slide = is_song_mode && index == 0;
                            let play_label = if is_title_slide {
                                "workspace-present-title"
                            } else {
                                "workspace-present-this-stanza"
                            };
                            if context_menu_item(ui, PLAY, play_label, &theme).clicked() {
                                action_project = Some(index);
                                ui.close();
                            }
                            if context_menu_item(ui, EYE, "workspace-preview", &theme).clicked()
                            {
                                action_preview = Some(index);
                                ui.close();
                            }

                            context_menu_separator(ui);

                            if is_title_slide {
                                if context_menu_item(
                                    ui,
                                    PENCIL_SIMPLE,
                                    "workspace-edit-song-information",
                                    &theme,
                                )
                                .clicked()
                                {
                                    state.song_editor.title = slide_deck.title().to_owned();
                                    state.song_editor.artist = slide_deck.artist().to_owned();
                                    state.song_editor.text = slide_deck.raw_text().to_owned();
                                    state.song_editor.open = true;
                                    ui.close();
                                }
                            } else {
                                context_menu_submenu(
                                    ui,
                                    TAG,
                                    "workspace-tag-strophe",
                                    &theme,
                                    |ui| {
                                        for tag_def in &state.settings.strophe_tags {
                                            let [r, g, b] = tag_def.color;
                                            let icon_col = Some(egui::Color32::from_rgb(r, g, b));
                                            let display_name = tag_def.localized_name(language);
                                            if context_menu_raw_item(ui, TAG, display_name, icon_col, &theme).clicked() {
                                                action_tag = Some((index, Some(tag_def.name.clone())));
                                                ui.close();
                                            }
                                        }
                                        context_menu_separator(ui);
                                        if context_menu_item(ui, GEAR, "workspace-tag-manage", &theme).clicked() {
                                            state.show_tag_manager_modal = true;
                                            ui.close();
                                        }
                                        if context_menu_item(ui, TRASH, "workspace-tag-remove", &theme).clicked() {
                                            action_tag = Some((index, None));
                                            ui.close();
                                        }
                                    },
                                );

                                if is_bible {
                                    if context_menu_item(
                                        ui,
                                        FILE_PLUS,
                                        "workspace-create-text-from-bible",
                                        &theme,
                                    )
                                    .clicked()
                                    {
                                        let clean_slides: Vec<String> = slide_deck
                                            .slides()
                                            .iter()
                                            .map(|slide| {
                                                crate::core::slides::strip_directive_commands(
                                                    &crate::core::slides::strip_comments(slide),
                                                )
                                                .trim()
                                                .to_string()
                                            })
                                            .filter(|s| !s.is_empty())
                                            .collect();
                                        state.open_text_editor(None, slide_deck.title(), "", &clean_slides.join("\n\n"));
                                        ui.close();
                                    }
                                } else if context_menu_item(
                                    ui,
                                    PENCIL_SIMPLE,
                                    "workspace-edit-this-stanza",
                                    &theme,
                                )
                                .clicked()
                                {
                                    action_edit = Some((index, strophe.clone()));
                                    ui.close();
                                }

                                let can_move_up = if is_song_mode { index > 1 } else { index > 0 };
                                if can_move_up {
                                    if context_menu_item(ui, ARROW_UP, "workspace-move-up", &theme)
                                        .clicked()
                                    {
                                        action_move_up = Some(index);
                                        ui.close();
                                    }
                                } else {
                                    context_menu_disabled_item(ui, ARROW_UP, "workspace-move-up", &theme);
                                }

                                if index + 1 < slides.len() {
                                    if context_menu_item(ui, ARROW_DOWN, "workspace-move-down", &theme)
                                        .clicked()
                                    {
                                        action_move_down = Some(index);
                                        ui.close();
                                    }
                                } else {
                                    context_menu_disabled_item(
                                        ui,
                                        ARROW_DOWN,
                                        "workspace-move-down",
                                        &theme,
                                    );
                                }

                                if context_menu_item(ui, COPY, "workspace-duplicate-stanza", &theme).clicked() {
                                    action_duplicate = Some(index);
                                    ui.close();
                                }

                                let min_slides = if is_song_mode { 2 } else { 1 };
                                if slides.len() > min_slides {
                                    context_menu_separator(ui);
                                    if context_menu_danger_item(ui, TRASH, "workspace-delete-stanza", &theme)
                                        .clicked()
                                    {
                                        action_delete = Some(index);
                                        ui.close();
                                    }
                                }
                            }
                        });

                        if (index + 1) % columns == 0 {
                            ui.end_row();
                        }
                    }
                });

            ui.add_space(16.0);

            if state.focused_container == Some(ContainerKind::Lyrics)
                && ui.memory(|m| m.focused().is_none())
                && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter))
            {
                action_project = Some(slide_deck.preview_slide_index());
            }

            // Executa as ações acionadas pelo menu de contexto
            if let Some(idx) = action_project {
                slide_deck.set_is_song(true);
                slide_deck.project_preview_slide(idx);
                state.apply_theme_to_live();
                *is_presentation_open = true;
                state.show_presentation_dock = state.settings.operator().show_dock;
            }
            if let Some(idx) = action_preview {
                slide_deck.select_preview_slide(idx);
            }
            if let Some((idx, tag_choice)) = action_tag {
                if let Some(current_text) = slides.get(idx) {
                    let updated = crate::core::slides::set_strophe_tag(current_text, tag_choice.as_deref());
                    slide_deck.update_slide_text(idx, &updated);
                    let _ = slide_deck.save_current_song_text(db);
                }
            }
            if let Some((idx, text)) = action_edit {
                state.strophe_editor.index = idx;
                state.strophe_editor.text = text;
                state.strophe_editor.open = true;
            }
            if let Some(idx) = action_move_up {
                slide_deck.move_slide_up(idx);
            }
            if let Some(idx) = action_move_down {
                slide_deck.move_slide_down(idx);
            }
            if let Some(idx) = action_duplicate {
                slide_deck.duplicate_slide(idx);
            }
            if let Some(idx) = action_delete {
                slide_deck.delete_slide(idx);
            }

        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(
        ctx: &egui::Context,
        state: &mut ControlPanelState,
        deck: &mut SlideDeck,
        width: f32,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 800.0),
            )),
            events,
            ..Default::default()
        };
        let mut presenting = false;
        let mut output = ctx.run_ui(input, |ui| {
            render_lyrics_column(ui, state, deck, &mut presenting, None)
        });
        output.textures_delta.clear();
        output
    }

    fn text_rect(output: &egui::FullOutput, label: &str) -> egui::Rect {
        output
            .shapes
            .iter()
            .find_map(|shape| {
                if let egui::Shape::Text(text) = &shape.shape
                    && text.galley.job.text == label
                {
                    Some(text.galley.rect.translate(text.pos.to_vec2()))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| panic!("missing label {label}"))
    }

    fn click(
        ctx: &egui::Context,
        state: &mut ControlPanelState,
        deck: &mut SlideDeck,
        width: f32,
        pos: egui::Pos2,
    ) -> egui::FullOutput {
        for pressed in [true, false] {
            let output = frame(
                ctx,
                state,
                deck,
                width,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            if !pressed {
                return output;
            }
        }
        unreachable!()
    }

    #[test]
    fn layout_button_switches_views_and_cards_select_preview_without_changing_live() {
        for width in [260.0, 640.0] {
            let mut state = ControlPanelState::default();
            let mut deck = SlideDeck::default();
            deck.load_custom_content("Atos 2".into(), "BL".into(), "Primeiro versículo\nAtos 2:1\n\nSegundo versículo\nAtos 2:2\n\nTerceiro versículo\nAtos 2:3".into());
            let slides = deck.slides().to_vec();
            let live = deck.live_slides().to_vec();
            let ctx = egui::Context::default();
            frame(&ctx, &mut state, &mut deck, width, Vec::new());
            let output = frame(&ctx, &mut state, &mut deck, width, Vec::new());
            assert!(!state.lyrics_cards_layout);
            let button = text_rect(&output, SQUARES_FOUR).center();
            click(&ctx, &mut state, &mut deck, width, button);
            assert!(state.lyrics_cards_layout);
            let output = frame(&ctx, &mut state, &mut deck, width, Vec::new());
            let first = text_rect(&output, "1").center();
            let second = text_rect(&output, "2").center();
            if width > 600.0 {
                assert!(second.x > first.x, "cards share a row in wide panels");
            } else {
                assert!(second.y > first.y, "cards wrap in narrow panels");
            }
            let output = click(&ctx, &mut state, &mut deck, width, second);
            assert_eq!(deck.preview_slide_index(), 1);
            assert_eq!(deck.slides(), slides);
            assert_eq!(deck.live_slides(), live);
            let button = text_rect(&output, LIST_BULLETS).center();
            click(&ctx, &mut state, &mut deck, width, button);
            assert!(!state.lyrics_cards_layout);
            assert_eq!(deck.preview_slide_index(), 1);
            assert_eq!(deck.live_slides(), live);
        }
    }
}
