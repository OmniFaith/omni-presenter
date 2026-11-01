use crate::{
    core::{slides::SlideDeck, state::ControlPanelState},
    services::{
        db::AppDb,
        lrclib::{self, LrcLibTrack},
        tasks::BackgroundTask,
    },
    ui::{components::dialog, theme::AppTheme},
};
use eframe::egui::{self, Color32, FontId, RichText, Stroke, Vec2};
use egui_phosphor::regular::*;
use std::time::{Duration, Instant};

const ICON_MUSIC: &str = "\u{E001}"; // assets/icons/music-note-2.svg

pub fn render_web_lyrics_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    db: Option<&AppDb>,
) {
    if !state.web_lyrics_search.open {
        return;
    }

    // 1. Poll running background search task
    if let Some(task) = &state.web_lyrics_search.task
        && let Some(result) = task.take() {
            state.web_lyrics_search.task = None;
            state.web_lyrics_search.is_loading = false;
            match result {
                Ok(tracks) => {
                    state.web_lyrics_search.results = tracks;
                    state.web_lyrics_search.error_message = None;
                    state.web_lyrics_search.selected_index =
                        if state.web_lyrics_search.results.is_empty() {
                            None
                        } else {
                            Some(0)
                        };
                }
                Err(err) => {
                    state.web_lyrics_search.results.clear();
                    state.web_lyrics_search.selected_index = None;
                    state.web_lyrics_search.error_message = Some(err);
                }
            }
        }

    let tokens = state.theme.clone();
    let language = state.settings.language;

    let mut should_cancel = false;
    let mut should_import = false;
    let mut should_import_and_edit = false;
    let mut trigger_search = false;

    // 2. Dynamic live search debounce (400ms)
    let query_trimmed = state.web_lyrics_search.query.trim().to_string();
    if let Some(keystroke) = state.web_lyrics_search.last_keystroke {
        if keystroke.elapsed() >= Duration::from_millis(400) {
            state.web_lyrics_search.last_keystroke = None;
            if query_trimmed.len() >= 2
                && query_trimmed != state.web_lyrics_search.last_searched_query
            {
                trigger_search = true;
            }
        } else {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }

    let folders = db.and_then(|d| d.get_folders().ok()).unwrap_or_default();

    let close = dialog::show(
        ctx,
        "web_lyrics_modal",
        "lrclib-title",
        "lrclib-subtitle",
        820.0,
        &tokens,
        |ui| {
            // 1. Search bar row (strictly matching 36px height)
            let field_h = 36.0;
            let btn_w = 110.0;
            let gap = 8.0;
            let search_box_width = (ui.available_width() - btn_w - gap).max(180.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;

                let (input_rect, input_resp) = ui.allocate_exact_size(
                    egui::vec2(search_box_width, field_h),
                    egui::Sense::click(),
                );

                let input_id = ui.make_persistent_id("lrclib_search_field_input");
                let is_focused = ui.ctx().memory(|m| m.has_focus(input_id));
                let stroke_c = if is_focused {
                    tokens.accent
                } else {
                    tokens.border_default
                };

                ui.painter().rect_filled(input_rect, 7.0, tokens.bg_pill);
                ui.painter().rect_stroke(
                    input_rect,
                    7.0,
                    Stroke::new(1.0, stroke_c),
                    egui::StrokeKind::Inside,
                );

                let icon_color = if is_focused {
                    tokens.accent
                } else {
                    tokens.text_muted
                };
                ui.painter().text(
                    egui::pos2(input_rect.min.x + 16.0, input_rect.center().y),
                    egui::Align2::CENTER_CENTER,
                    MAGNIFYING_GLASS,
                    FontId::proportional(14.0),
                    icon_color,
                );

                let clear_pad = if state.web_lyrics_search.query.is_empty() { 12.0 } else { 32.0 };
                let text_rect = egui::Rect::from_min_max(
                    egui::pos2(input_rect.min.x + 32.0, input_rect.min.y),
                    egui::pos2(input_rect.max.x - clear_pad, input_rect.max.y),
                );

                let edit = egui::TextEdit::singleline(&mut state.web_lyrics_search.query)
                    .id(input_id)
                    .frame(egui::Frame::NONE)
                    .hint_text(RichText::new(language.text("lrclib-live-search-hint")).color(tokens.text_muted).size(13.0))
                    .font(FontId::proportional(13.0))
                    .margin(egui::Margin::symmetric(0, 8))
                    .align(egui::Align2::LEFT_CENTER);

                let text_res = ui.put(text_rect, edit);

                if text_res.changed() {
                    state.web_lyrics_search.last_keystroke = Some(Instant::now());
                }

                if !state.web_lyrics_search.query.is_empty() {
                    let clear_rect = egui::Rect::from_center_size(
                        egui::pos2(input_rect.right() - 16.0, input_rect.center().y),
                        Vec2::splat(20.0),
                    );
                    let clear_btn = ui.put(
                        clear_rect,
                        egui::Button::new(RichText::new(X).size(11.0).color(tokens.text_muted)).frame(false),
                    );
                    if clear_btn.on_hover_text(language.text("lrclib-clear")).clicked() {
                        state.web_lyrics_search.reset_search();
                        state.web_lyrics_search.query.clear();
                        text_res.request_focus();
                    }
                }

                if input_resp.clicked() && !text_res.has_focus() {
                    text_res.request_focus();
                }

                if text_res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && !query_trimmed.is_empty() {
                        trigger_search = true;
                    }

                // Search action button (exact 36.0 px)
                let can_search = !query_trimmed.is_empty() && !state.web_lyrics_search.is_loading;
                let (fill_c, text_c, stroke) = if can_search {
                    (tokens.accent, tokens.text_on_accent, Stroke::NONE)
                } else {
                    (tokens.bg_pill, tokens.text_muted, Stroke::new(1.0, tokens.border_default))
                };

                let btn = egui::Button::new(
                    RichText::new(format!("{MAGNIFYING_GLASS}  {}", language.text("lrclib-search-button")))
                        .size(13.0)
                        .strong()
                        .color(text_c),
                )
                .fill(fill_c)
                .stroke(stroke)
                .corner_radius(7.0)
                .min_size(Vec2::new(btn_w, field_h));

                if ui.add_enabled(can_search, btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    trigger_search = true;
                }
            });

            ui.add_space(8.0);

            // Error notice
            if let Some(ref err) = state.web_lyrics_search.error_message {
                let err_text = if err.starts_with("lrclib-") || err.starts_with("lrclib.") {
                    language.text(err).to_string()
                } else {
                    language.format("lrclib-error-prefix", &[("error", err)])
                };
                ui.horizontal(|ui| {
                    ui.label(RichText::new(WARNING_CIRCLE).size(15.0).color(tokens.status_danger));
                    ui.label(RichText::new(err_text).size(12.0).color(tokens.status_danger));
                    if ui.small_button(language.text("lrclib-retry")).clicked() {
                        trigger_search = true;
                    }
                });
                ui.add_space(4.0);
            }

            // 2. BODY CONTAINER: exactly 360.0 px height
            let body_h = 360.0;
            if state.web_lyrics_search.is_loading {
                render_loading_state(ui, body_h, &tokens, language);
            } else if state.web_lyrics_search.results.is_empty() {
                if query_trimmed.is_empty() {
                    render_welcome_state(ui, body_h, &tokens, language);
                } else {
                    render_no_results_state(ui, &query_trimmed, body_h, &tokens, language);
                }
            } else {
                let list_w = 310.0;
                let available_preview_w = (ui.available_width() - list_w - 12.0).max(200.0);

                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;

                    // Results List (Left)
                    egui::Frame::new()
                        .fill(tokens.bg_card)
                        .stroke(Stroke::new(1.0, tokens.border_default))
                        .corner_radius(tokens.radius_md)
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.set_width(list_w);
                            ui.set_height(body_h);

                            ui.vertical(|ui| {
                                egui::ScrollArea::vertical()
                                    .id_salt("lrclib_results_scroll")
                                    .max_height(body_h - 16.0)
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        ui.set_width(ui.available_width());
                                        for (idx, track) in state.web_lyrics_search.results.iter().enumerate() {
                                            let is_selected = state.web_lyrics_search.selected_index == Some(idx);
                                            if render_song_result_card(ui, track, is_selected, &tokens, language) {
                                                state.web_lyrics_search.selected_index = Some(idx);
                                            }
                                            ui.add_space(5.0);
                                        }
                                    });
                            });
                        });

                    // Strophe Preview (Right)
                    egui::Frame::new()
                        .fill(tokens.bg_card)
                        .stroke(Stroke::new(1.0, tokens.border_default))
                        .corner_radius(tokens.radius_md)
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            ui.set_width(available_preview_w);
                            ui.set_height(body_h);

                            ui.vertical(|ui| {
                                if let Some(track) = state.web_lyrics_search.selected_index.and_then(|idx| state.web_lyrics_search.results.get(idx)) {
                                    render_lyrics_strophes_preview(ui, track, available_preview_w - 24.0, body_h - 24.0, &tokens, language);
                                } else {
                                    render_right_empty_state(ui, &tokens, language);
                                }
                            });
                        });
                });
            }

            ui.add_space(10.0);

            // 3. FOOTER: Destination Folder Picker & Action Buttons
            ui.horizontal(|ui| {
                ui.label(RichText::new(language.text("lrclib-target-folder")).size(12.0).color(tokens.text_secondary));

                let folder_display = if state.web_lyrics_search.target_folder.is_empty() {
                    format!("{FOLDER}  {}", language.text("common-root"))
                } else {
                    format!("{FOLDER}  {}", state.web_lyrics_search.target_folder)
                };

                ui.scope(|ui| {
                    ui.spacing_mut().button_padding = egui::vec2(10.0, 6.0);
                    let visuals = &mut ui.style_mut().visuals.widgets;
                    visuals.inactive.corner_radius = egui::CornerRadius::same(7);
                    visuals.hovered.corner_radius = egui::CornerRadius::same(7);
                    visuals.active.corner_radius = egui::CornerRadius::same(7);
                    visuals.inactive.bg_fill = tokens.bg_pill;
                    visuals.inactive.bg_stroke = Stroke::new(1.0, tokens.border_default);

                    egui::ComboBox::from_id_salt("lrclib_folder_picker")
                        .width(150.0)
                        .selected_text(RichText::new(folder_display).size(12.0).color(tokens.text_primary))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.web_lyrics_search.target_folder, String::new(), language.text("common-root"));
                            for f in &folders {
                                ui.selectable_value(&mut state.web_lyrics_search.target_folder, f.clone(), f);
                            }
                        });
                });

                let results_count = state.web_lyrics_search.results.len();
                if results_count > 0 {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(language.format("lrclib-results-count", &[("count", &results_count.to_string())]))
                            .size(11.5)
                            .color(tokens.text_muted),
                    );
                }

                // Actions on right (34px height)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;

                    let has_selected = state.web_lyrics_search.selected_index.and_then(|idx| state.web_lyrics_search.results.get(idx)).is_some();

                    if has_selected {
                        let import_btn = egui::Button::new(
                            RichText::new(format!("{LIST_PLUS}  {}", language.text("lrclib-import-button")))
                                .size(12.5)
                                .strong()
                                .color(tokens.text_on_accent),
                        )
                        .fill(tokens.accent)
                        .stroke(Stroke::NONE)
                        .corner_radius(7.0)
                        .min_size(Vec2::new(140.0, 34.0));

                        if ui.add(import_btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            should_import = true;
                        }

                        let edit_btn = egui::Button::new(
                            RichText::new(format!("{PENCIL_SIMPLE}  {}", language.text("lrclib-import-and-edit")))
                                .size(12.0)
                                .color(tokens.text_primary),
                        )
                        .fill(tokens.bg_pill)
                        .stroke(Stroke::new(1.0, tokens.border_default))
                        .corner_radius(7.0)
                        .min_size(Vec2::new(135.0, 34.0));

                        if ui.add(edit_btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            should_import_and_edit = true;
                        }
                    }

                    let cancel_btn = egui::Button::new(
                        RichText::new(language.text("common-cancel"))
                            .size(12.0)
                            .color(tokens.text_secondary),
                    )
                    .fill(tokens.bg_pill)
                    .stroke(Stroke::new(1.0, tokens.border_default))
                    .corner_radius(7.0)
                    .min_size(Vec2::new(80.0, 34.0));

                    if ui.add(cancel_btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        should_cancel = true;
                    }
                });
            });
        },
    );

    // Launch search task
    if trigger_search && !query_trimmed.is_empty() {
        state.web_lyrics_search.last_searched_query = query_trimmed.clone();
        state.web_lyrics_search.is_loading = true;
        state.web_lyrics_search.error_message = None;

        let q = query_trimmed;
        let task = BackgroundTask::spawn_with_repaint(ctx, move || {
            lrclib::search_lyrics(&q, None)
        });
        state.web_lyrics_search.task = Some(task);
    }

    if close || should_cancel {
        state.web_lyrics_search.open = false;
        state.web_lyrics_search.task = None;
    } else if should_import || should_import_and_edit {
        if let Some(track) = state
            .web_lyrics_search
            .selected_index
            .and_then(|idx| state.web_lyrics_search.results.get(idx).cloned())
        {
            let title = track.title().to_string();
            let artist = track.artist_name.clone();
            let lyrics = track.lyrics_text();
            let folder = state.web_lyrics_search.target_folder.clone();

            if should_import_and_edit {
                state.song_editor = Default::default();
                state.song_editor.title = title;
                state.song_editor.artist = artist;
                state.song_editor.text = lyrics;
                state.song_editor.folder = folder;
                state.song_editor.open = true;
            } else {
                if let Some(app_db) = db {
                    let _ = app_db.add_song_full(&title, &artist, &lyrics, &folder, "");
                    deck.reload_from_db(app_db);
                } else {
                    deck.add_or_update_song(title.clone(), artist, lyrics);
                }
                state.selected_tab = 0;
                state.notify(language.format("lrclib-imported-success", &[("title", &title)]));
            }
        }
        state.web_lyrics_search.open = false;
        state.web_lyrics_search.task = None;
    }
}

fn render_welcome_state(
    ui: &mut egui::Ui,
    height: f32,
    tokens: &AppTheme,
    language: crate::core::i18n::Language,
) {
    egui::Frame::new()
        .fill(tokens.bg_card)
        .stroke(Stroke::new(1.0, tokens.border_subtle))
        .corner_radius(tokens.radius_md)
        .inner_margin(egui::Margin::symmetric(24, 20))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(height);
            ui.vertical_centered(|ui| {
                let top_space = (height * 0.12).clamp(16.0, 40.0);
                ui.add_space(top_space);

                let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(58.0), egui::Sense::hover());
                ui.painter().circle_filled(icon_rect.center(), 29.0, tokens.accent.linear_multiply(0.12));
                ui.painter().text(
                    icon_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    ICON_MUSIC,
                    FontId::proportional(26.0),
                    tokens.accent,
                );

                ui.add_space(14.0);
                ui.label(
                    RichText::new(language.text("lrclib-ready-title"))
                        .size(16.0)
                        .strong()
                        .color(tokens.text_primary),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(language.text("lrclib-ready-subtitle"))
                        .size(12.5)
                        .color(tokens.text_secondary),
                );

                ui.add_space(16.0);
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "workspace-shortcut-ctrl-shift-w"))
                        .size(11.0)
                        .color(tokens.text_muted),
                );
            });
        });
}

fn render_loading_state(
    ui: &mut egui::Ui,
    height: f32,
    tokens: &AppTheme,
    language: crate::core::i18n::Language,
) {
    egui::Frame::new()
        .fill(tokens.bg_card)
        .stroke(Stroke::new(1.0, tokens.border_subtle))
        .corner_radius(tokens.radius_md)
        .inner_margin(egui::Margin::symmetric(24, 20))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(height);
            ui.vertical_centered(|ui| {
                ui.add_space((height * 0.35).max(30.0));
                ui.spinner();
                ui.add_space(14.0);
                ui.label(
                    RichText::new(language.text("lrclib-searching"))
                        .size(14.0)
                        .strong()
                        .color(tokens.accent),
                );
            });
        });
}

fn render_no_results_state(
    ui: &mut egui::Ui,
    query: &str,
    height: f32,
    tokens: &AppTheme,
    language: crate::core::i18n::Language,
) {
    egui::Frame::new()
        .fill(tokens.bg_card)
        .stroke(Stroke::new(1.0, tokens.border_subtle))
        .corner_radius(tokens.radius_md)
        .inner_margin(egui::Margin::symmetric(24, 20))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(height);
            ui.vertical_centered(|ui| {
                ui.add_space((height * 0.25).max(24.0));
                ui.label(RichText::new(MAGNIFYING_GLASS).size(34.0).color(tokens.text_muted));
                ui.add_space(12.0);
                ui.label(
                    RichText::new(language.format("lrclib-no-results", &[("query", query)]))
                        .size(14.5)
                        .strong()
                        .color(tokens.text_primary),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "workspace-try-simpler-search-terms-just-the-song-title-or-the"))
                        .size(12.0)
                        .color(tokens.text_secondary),
                );
            });
        });
}

fn render_song_result_card(
    ui: &mut egui::Ui,
    track: &LrcLibTrack,
    is_selected: bool,
    tokens: &AppTheme,
    language: crate::core::i18n::Language,
) -> bool {
    let card_w = ui.available_width();
    let card_h = 48.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(card_w, card_h), egui::Sense::click());

    let hover = ui.ctx().animate_bool(response.id.with("hover"), response.hovered());
    let bg = if is_selected {
        tokens.accent_soft
    } else {
        tokens.bg_card.lerp_to_gamma(tokens.bg_card_hover, hover)
    };
    let stroke = if is_selected {
        Stroke::new(1.0, tokens.accent.linear_multiply(0.45))
    } else {
        Stroke::new(1.0, tokens.border_subtle)
    };

    ui.painter().rect_filled(rect, 7.0, bg);
    ui.painter().rect_stroke(rect, 7.0, stroke, egui::StrokeKind::Inside);

    let content_rect = rect.shrink2(egui::vec2(10.0, 6.0));

    // 1. Right badge pill
    let (badge_text, badge_color) = if track.instrumental {
        (language.text("lrclib-instrumental-badge"), tokens.status_warning_fg)
    } else if track.synced_lyrics.is_some() {
        (language.text("lrclib-synced-badge"), tokens.accent)
    } else {
        (language.text("lrclib-plain-badge"), tokens.text_muted)
    };
    let badge_gal = ui.painter().layout_no_wrap(
        badge_text.to_string(),
        egui::FontId::proportional(10.0),
        badge_color,
    );
    let badge_w = (badge_gal.size().x + 14.0).max(36.0);
    let badge_h = 18.0;
    let badge_rect = egui::Rect::from_min_max(
        egui::pos2(content_rect.max.x - badge_w, content_rect.center().y - badge_h / 2.0),
        egui::pos2(content_rect.max.x, content_rect.center().y + badge_h / 2.0),
    );
    ui.painter().rect_filled(badge_rect, 5.0, badge_color.linear_multiply(0.12));
    ui.painter().text(
        badge_rect.center(),
        egui::Align2::CENTER_CENTER,
        badge_text,
        egui::FontId::proportional(10.0),
        badge_color,
    );

    // 2. Icon + Title & Subtitle on the left, bounded before the badge
    let text_max_x = (badge_rect.min.x - 8.0).max(content_rect.min.x + 30.0);
    let text_rect = egui::Rect::from_min_max(
        content_rect.min,
        egui::pos2(text_max_x, content_rect.max.y),
    );
    let mut text_ui = ui.new_child(egui::UiBuilder::new().max_rect(text_rect));
    text_ui.horizontal(|ui| {
        ui.label(
            RichText::new(ICON_MUSIC)
                .size(13.0)
                .color(if is_selected {
                    tokens.accent
                } else {
                    tokens.icon_muted
                }),
        );
        ui.add_space(4.0);

        let max_label_w = ui.available_width().max(20.0);
        ui.vertical(|ui| {
            ui.set_max_width(max_label_w);
            ui.add(
                egui::Label::new(
                    RichText::new(track.title())
                        .size(12.5)
                        .strong()
                        .color(if is_selected {
                            tokens.accent
                        } else {
                            tokens.text_primary
                        }),
                )
                .truncate(),
            );

            let subtitle = if track.artist_name.is_empty() {
                crate::ui::i18n::text(ui, "workspace-unknown-artist").to_string()
            } else {
                track.artist_name.clone()
            };

            let dur = track.formatted_duration();
            let desc = if dur.is_empty() {
                subtitle
            } else {
                format!("{subtitle} · {dur}")
            };

            ui.add(
                egui::Label::new(RichText::new(desc).size(11.0).color(tokens.text_secondary))
                    .truncate(),
            );
        });
    });

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    response.clicked()
}

fn render_badge_pill(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .fill(color.linear_multiply(0.12))
        .stroke(Stroke::NONE)
        .corner_radius(5.0)
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(10.0).strong().color(color));
        });
}

fn render_right_empty_state(ui: &mut egui::Ui, tokens: &AppTheme, language: crate::core::i18n::Language) {
    ui.vertical_centered(|ui| {
        ui.add_space(60.0);
        ui.label(RichText::new(ICON_MUSIC).size(36.0).color(tokens.text_muted));
        ui.add_space(10.0);
        ui.label(
            RichText::new(language.text("lrclib-preview-placeholder"))
                .size(13.0)
                .color(tokens.text_muted),
        );
    });
}

fn render_lyrics_strophes_preview(
    ui: &mut egui::Ui,
    track: &LrcLibTrack,
    available_width: f32,
    available_height: f32,
    tokens: &AppTheme,
    language: crate::core::i18n::Language,
) {
    ui.vertical(|ui| {
        // Song Header Summary
        ui.horizontal(|ui| {
            let max_header_text_w = (available_width - 90.0).max(100.0);
            ui.vertical(|ui| {
                ui.set_max_width(max_header_text_w);
                ui.add(
                    egui::Label::new(
                        RichText::new(track.title())
                            .size(16.0)
                            .strong()
                            .color(tokens.text_primary),
                    )
                    .truncate(),
                );

                let mut subtitle = track.artist_name.clone();
                if let Some(album) = &track.album_name
                    && !album.is_empty() {
                        subtitle.push_str(&format!(" · {album}"));
                    }
                let dur = track.formatted_duration();
                if !dur.is_empty() {
                    subtitle.push_str(&format!(" ({dur})"));
                }
                ui.add(
                    egui::Label::new(RichText::new(subtitle).size(12.0).color(tokens.text_secondary))
                        .truncate(),
                );
            });

            let strophes = track.strophes();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !strophes.is_empty() {
                    render_badge_pill(
                        ui,
                        &language.format("lrclib-strophes-count", &[("count", &strophes.len().to_string())]),
                        tokens.accent,
                    );
                }
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(6.0);

        let card_w = (available_width - 16.0).max(180.0);
        let scroll_h = (available_height - 60.0).max(100.0);
        egui::ScrollArea::vertical()
            .id_salt("web_lyrics_strophes_scroll")
            .max_height(scroll_h)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(available_width);
                ui.vertical(|ui| {
                    if track.instrumental {
                        ui.add_space(40.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(language.text("lrclib-instrumental"))
                                    .italics()
                                    .color(tokens.text_muted),
                            );
                        });
                    } else {
                        let strophes = track.strophes();
                        if strophes.is_empty() {
                            ui.add_space(40.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new(language.text("lrclib-no-lyrics-available"))
                                        .italics()
                                        .color(tokens.text_muted),
                                );
                            });
                        } else {
                            for (i, strophe) in strophes.iter().enumerate() {
                                egui::Frame::new()
                                    .fill(tokens.bg_pill)
                                    .stroke(Stroke::new(1.0, tokens.border_subtle))
                                    .corner_radius(tokens.radius_md)
                                    .inner_margin(egui::Margin::symmetric(14, 10))
                                    .show(ui, |ui| {
                                        ui.set_width(card_w);
                                        ui.horizontal_top(|ui| {
                                            ui.label(
                                                RichText::new((i + 1).to_string())
                                                    .size(11.0)
                                                    .strong()
                                                    .color(tokens.accent),
                                            );
                                            ui.add_space(8.0);
                                            ui.vertical(|ui| {
                                                for line in strophe.lines() {
                                                    ui.add(
                                                        egui::Label::new(
                                                            RichText::new(line)
                                                                .font(FontId::proportional(13.0))
                                                                .line_height(Some(19.0))
                                                                .color(tokens.text_primary),
                                                        )
                                                        .wrap(),
                                                    );
                                                }
                                            });
                                        });
                                    });
                                ui.add_space(8.0);
                            }
                        }
                    }
                });
            });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_lyrics_modal_renders_and_closed_by_default() {
        let mut state = ControlPanelState::default();
        let mut deck = SlideDeck::default();
        let ctx = egui::Context::default();
        assert!(!state.web_lyrics_search.open);

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_web_lyrics_modal(ui.ctx(), &mut state, &mut deck, None);
        });
        output.textures_delta.clear();
        assert!(output.shapes.is_empty());
    }

    #[test]
    fn test_web_lyrics_modal_renders_with_strophes() {
        let mut state = ControlPanelState::default();
        let mut deck = SlideDeck::default();
        let ctx = egui::Context::default();
        state.web_lyrics_search.open = true;
        state.web_lyrics_search.results.push(LrcLibTrack {
            id: 1,
            name: Some("Amazing Grace".into()),
            track_name: "Amazing Grace".into(),
            artist_name: "John Newton".into(),
            album_name: Some("Hymns".into()),
            duration: Some(180.0),
            instrumental: false,
            plain_lyrics: Some("Amazing grace how sweet the sound\nThat saved a wretch like me\n\nI once was lost but now am found\nWas blind but now I see".into()),
            synced_lyrics: None,
        });
        state.web_lyrics_search.selected_index = Some(0);

        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 800.0),
            )),
            ..Default::default()
        };

        let mut output = ctx.run_ui(raw, |ui| {
            render_web_lyrics_modal(ui.ctx(), &mut state, &mut deck, None);
        });
        output.textures_delta.clear();
        assert!(!output.shapes.is_empty());
    }

    #[test]
    fn test_web_lyrics_import_into_database_and_deck() {
        let mut state = ControlPanelState::default();
        let mut deck = SlideDeck::default();
        let db = AppDb::open_in_memory().unwrap();

        state.web_lyrics_search.open = true;
        state.web_lyrics_search.results.push(LrcLibTrack {
            id: 99,
            name: Some("Oceans".into()),
            track_name: "Oceans".into(),
            artist_name: "Hillsong".into(),
            album_name: None,
            duration: Some(240.0),
            instrumental: false,
            plain_lyrics: Some("You call me out upon the waters\nThe great unknown\nWhere feet may fail".into()),
            synced_lyrics: None,
        });
        state.web_lyrics_search.selected_index = Some(0);

        let track = &state.web_lyrics_search.results[0];
        let id = db
            .add_song_full(
                track.title(),
                &track.artist_name,
                &track.lyrics_text(),
                &state.web_lyrics_search.target_folder,
                "",
            )
            .unwrap();
        assert!(id > 0);

        deck.reload_from_db(&db);
        let found = deck.library().iter().any(|s| s.title == "Oceans");
        assert!(found);
    }
}
