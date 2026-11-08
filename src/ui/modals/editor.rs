#[path = "editor_versions.rs"]
mod editing;
#[path = "editor_actions.rs"]
pub mod actions;
#[path = "editor_menu.rs"]
pub mod menu;
#[path = "editor_selection.rs"]
pub mod selection;
#[path = "syntax_guide.rs"]
mod syntax_guide;

use crate::core::models::{PresentationTheme, Song};
use crate::core::slides::{SlideDeck, parse_slides_from_text};
use crate::core::state::{ControlPanelState, EditorSnapshot};
use crate::render::{paint_theme_bg, render_theme_lyrics};
use crate::services::db::AppDb;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, RichText, Stroke, StrokeKind, Vec2};
use egui_phosphor::regular::{ARROW_U_UP_LEFT, ARROW_U_UP_RIGHT, EYE, EYE_SLASH, FLOPPY_DISK, LINK};

const TOOLBAR_HEIGHT: f32 = 44.0;
const SIDEBAR_MIN: f32 = 260.0;
const SIDEBAR_MAX: f32 = 340.0;

pub fn render_editor_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    slide_deck: &mut SlideDeck,
    is_text_focused: &mut bool,
    db: Option<&AppDb>,
) {
    if !state.song_editor.open {
        return;
    }

    if state.song_editor.theme.is_none() {
        state.song_editor.theme = Some(state.preview_theme().clone());
        state.song_editor.theme_dirty = false;
        state.song_editor.selected_slide = None;
        state.song_editor.history.clear();
        state.song_editor.future.clear();
        state.song_editor.show_versions = false;
        state.song_editor.versions_query.clear();
        state.song_editor.selection = (0, 0);
        state.song_editor.linked_versions = db
            .and_then(|db| db.song_version_ids_of(&state.song_editor.title).ok())
            .unwrap_or_default();
    }

    let screen_rect = ctx.input(|i| i.viewport_rect());
    let tokens = state.theme.clone();
    let mut work_theme = state.song_editor.theme.clone().unwrap_or_default();

    let mut should_save = false;
    let mut should_cancel = false;
    let mut project_slide: Option<usize> = None;

    let overlay_id = egui::Id::unique("song_editor_overlay");
    egui::Area::new(overlay_id)
        .order(egui::Order::Foreground)
        .fixed_pos(screen_rect.min)
        .show(ctx, |ui| {
            if ui.input(|i| i.key_pressed(egui::Key::Escape))
                && !state.song_editor.show_versions
            {
                should_cancel = true;
            }
            if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
                should_save = true;
            }

            let _ = ui.allocate_rect(screen_rect, egui::Sense::click_and_drag());
            ui.painter()
                .rect_filled(screen_rect, 0.0, Color32::from_black_alpha(130));

            let panel_rect = screen_rect.shrink2(Vec2::new(26.0, 20.0));
            let shadow = egui::epaint::Shadow {
                offset: [0, 12],
                blur: 28,
                spread: 0,
                color: Color32::from_black_alpha(70),
            };
            ui.painter().add(shadow.as_shape(panel_rect, 12.0));
            ui.painter().rect_filled(panel_rect, 12.0, tokens.bg_panel);
            ui.painter().rect_stroke(
                panel_rect,
                12.0,
                Stroke::new(1.0, tokens.border_default),
                StrokeKind::Inside,
            );

            let inner = panel_rect.shrink(12.0);
            let toolbar_rect =
                egui::Rect::from_min_size(inner.min, Vec2::new(inner.width(), TOOLBAR_HEIGHT));
            ui.painter().line_segment(
                [
                    egui::pos2(inner.min.x, toolbar_rect.max.y + 8.0),
                    egui::pos2(inner.max.x, toolbar_rect.max.y + 8.0),
                ],
                Stroke::new(1.0, tokens.border_subtle),
            );

            let body_top = toolbar_rect.max.y + 14.0;
            let body_rect = egui::Rect::from_min_max(egui::pos2(inner.min.x, body_top), inner.max);
            let sidebar_w = (body_rect.width() * 0.28).clamp(SIDEBAR_MIN, SIDEBAR_MAX);
            let sidebar_rect =
                egui::Rect::from_min_size(body_rect.min, Vec2::new(sidebar_w, body_rect.height()));
            let divider_x = sidebar_rect.max.x + 10.0;
            ui.painter().line_segment(
                [
                    egui::pos2(divider_x, body_rect.min.y),
                    egui::pos2(divider_x, body_rect.max.y),
                ],
                Stroke::new(1.0, tokens.border_subtle),
            );
            let grid_rect = egui::Rect::from_min_max(
                egui::pos2(divider_x + 12.0, body_rect.min.y),
                body_rect.max,
            );

            {
                let mut tui = ui.new_child(egui::UiBuilder::new().max_rect(toolbar_rect));
                let changed = render_toolbar(
                    &mut tui,
                    state,
                    &mut work_theme,
                    &tokens,
                    &mut should_save,
                    &mut should_cancel,
                );
                if changed {
                    state.song_editor.theme_dirty = true;
                }
            }

            {
                let mut sui = ui.new_child(egui::UiBuilder::new().max_rect(sidebar_rect));
                sui.set_clip_rect(sidebar_rect.intersect(ui.clip_rect()));
                render_sidebar(
                    &mut sui,
                    state,
                    &tokens,
                    is_text_focused,
                    slide_deck.library(),
                    db,
                );
            }

            {
                let mut gui = ui.new_child(egui::UiBuilder::new().max_rect(grid_rect));
                gui.set_clip_rect(grid_rect.intersect(ui.clip_rect()));
                render_grid(&mut gui, state, &work_theme, &tokens, &mut project_slide);
            }
        });

    syntax_guide::render_syntax_guide_modal(ctx, state, &tokens);

    state.song_editor.theme = Some(work_theme);

    if let Some(index) = project_slide {
        state.song_editor.selected_slide = Some(index);
        slide_deck.load_custom_content(
            state.song_editor.title.clone(),
            state.song_editor.artist.clone(),
            state.song_editor.text.clone(),
        );
        slide_deck.select_preview_slide(index);
        state.notify(state.settings.language.format("workspace-stanza-loaded-in-preview", &[("name", &(index + 1).to_string())]));
    }

    if should_save {
        save_song(state, slide_deck, db);
        state.song_editor.theme = None;
        state.song_editor.open = false;
    } else if should_cancel || !state.song_editor.open {
        state.song_editor.theme = None;
        state.song_editor.open = false;
    }
}

fn snapshot(state: &ControlPanelState, theme: &PresentationTheme) -> EditorSnapshot {
    EditorSnapshot {
        title: state.song_editor.title.clone(),
        artist: state.song_editor.artist.clone(),
        text: state.song_editor.text.clone(),
        theme: theme.clone(),
    }
}

fn push_history(state: &mut ControlPanelState, snap: EditorSnapshot) {
    state.song_editor.history.push(snap);
    if state.song_editor.history.len() > 120 {
        state.song_editor.history.remove(0);
    }
    state.song_editor.future.clear();
}

fn restore(
    state: &mut ControlPanelState,
    snap: EditorSnapshot,
    work_theme: &mut PresentationTheme,
) {
    state.song_editor.title = snap.title;
    state.song_editor.artist = snap.artist;
    state.song_editor.text = snap.text;
    *work_theme = snap.theme;
}

fn render_toolbar(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    work: &mut PresentationTheme,
    tokens: &AppTheme,
    should_save: &mut bool,
    should_cancel: &mut bool,
) -> bool {
    let before = snapshot(state, work);
    let mut changed = false;
    let mut history_action = false;

    ui.horizontal_centered(|ui| {
        let actions_w = 196.0;
        let tools_w = (ui.available_width() - actions_w).max(240.0);

        ui.allocate_ui_with_layout(
            Vec2::new(tools_w, TOOLBAR_HEIGHT),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                egui::ScrollArea::horizontal()
                    .id_salt("song_editor_toolbar_scroll")
                    .show(ui, |ui| {
                        ui.horizontal_centered(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;

                            let undo_ok = !state.song_editor.history.is_empty();
                            if tool_button(ui, ARROW_U_UP_LEFT, false, "workspace-undo", tokens, undo_ok)
                                && let Some(prev) = state.song_editor.history.pop() {
                                    state.song_editor.future.push(snapshot(state, work));
                                    restore(state, prev, work);
                                    history_action = true;
                                }
                            let redo_ok = !state.song_editor.future.is_empty();
                            if tool_button(ui, ARROW_U_UP_RIGHT, false, "workspace-redo", tokens, redo_ok)
                                && let Some(next) = state.song_editor.future.pop() {
                                    state.song_editor.history.push(snapshot(state, work));
                                    restore(state, next, work);
                                    history_action = true;
                                }

                            changed |= selection::render_editor_tools(ui, state, tokens);
                        });
                    });
            },
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            if primary_button(ui, FLOPPY_DISK, crate::ui::i18n::text(ui, "common-save-item"), tokens).clicked() {
                *should_save = true;
            }
            if ghost_button(ui, crate::ui::i18n::text(ui, "common-cancel"), tokens).clicked() {
                *should_cancel = true;
            }
        });
    });

    if changed && !history_action {
        push_history(state, before);
    }

    changed
}

fn render_sidebar(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
    is_text_focused: &mut bool,
    library: &[Song],
    db: Option<&AppDb>,
) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 8.0;
        field_label(ui, crate::ui::i18n::text(ui, "workspace-title"), tokens);
        pill_frame(tokens).show(ui, |ui| {
            let res = ui.add(
                egui::TextEdit::singleline(&mut state.song_editor.title)
                    .frame(egui::Frame::NONE)
                    .desired_width(f32::INFINITY)
                    .hint_text(crate::ui::i18n::text(ui, "workspace-song-title"))
                    .font(egui::FontId::proportional(14.0)),
            );
            if res.has_focus() {
                *is_text_focused = true;
            }
        });

        field_label(ui, crate::ui::i18n::text(ui, "workspace-songwriter-band"), tokens);
        pill_frame(tokens).show(ui, |ui| {
            let res = ui.add(
                egui::TextEdit::singleline(&mut state.song_editor.artist)
                    .frame(egui::Frame::NONE)
                    .desired_width(f32::INFINITY)
                    .hint_text(crate::ui::i18n::text(ui, "workspace-songwriter-or-band"))
                    .font(egui::FontId::proportional(14.0)),
            );
            if res.has_focus() {
                *is_text_focused = true;
            }
        });

        field_label(ui, crate::ui::i18n::text(ui, "workspace-tags"), tokens);
        pill_frame(tokens).show(ui, |ui| {
            let res = ui.add(
                egui::TextEdit::singleline(&mut state.song_editor.tags)
                    .frame(egui::Frame::NONE)
                    .desired_width(f32::INFINITY)
                    .hint_text(crate::ui::i18n::text(ui, "workspace-separated-by-commas"))
                    .font(egui::FontId::proportional(13.0)),
            );
            if res.has_focus() {
                *is_text_focused = true;
            }
        });

        let versions_label = crate::ui::i18n::format(ui, "workspace-link-versions", &[("name", LINK)]);
        let versions_btn = egui::Button::new(
            RichText::new(versions_label)
                .size(12.5)
                .color(if state.song_editor.show_versions {
                    tokens.accent
                } else {
                    tokens.text_primary
                }),
        )
        .fill(if state.song_editor.show_versions {
            tokens.accent_soft
        } else {
            tokens.bg_pill
        })
        .stroke(Stroke::new(
            1.0,
            if state.song_editor.show_versions {
                tokens.accent
            } else {
                tokens.border_default
            },
        ))
        .corner_radius(8.0)
        .min_size(Vec2::new(ui.available_width(), 32.0));
        if ui.add(versions_btn).clicked() {
            state.song_editor.show_versions = !state.song_editor.show_versions;
        }
        if state.song_editor.show_versions {
            editing::versions_panel(ui, state, tokens, library, db);
        }

        field_label(ui, "workspace-text", tokens);
        let remaining = (ui.available_height() - 4.0).max(60.0);
        egui::Frame::new()
            .fill(tokens.bg_pill)
            .stroke(Stroke::new(1.0, tokens.border_default))
            .corner_radius(8.0)
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                let content_h = (remaining - 16.0).max(40.0);
                ui.set_max_height(content_h);
                let tags_ref = &state.settings.strophe_tags;
                let tokens_ref = tokens;
                let hide_syntax_markers = state.song_editor.hide_syntax_markers;
                let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
                    let job = crate::ui::components::lyrics_highlighter::highlight_lyrics_job(
                        buf.as_str(),
                        tags_ref,
                        tokens_ref,
                        14.0,
                        wrap_width,
                        hide_syntax_markers,
                    );
                    ui.fonts_mut(|f| f.layout_job(job))
                };
                egui::ScrollArea::vertical()
                    .id_salt("song_editor_text_scroll")
                    .max_height(content_h)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let output = egui::TextEdit::multiline(&mut state.song_editor.text)
                            .id(egui::Id::unique(editing::TEXT_EDIT_ID))
                            .frame(egui::Frame::NONE)
                            .desired_width(f32::INFINITY)
                            .hint_text(
                                crate::ui::i18n::text(ui, "workspace-enter-the-lyrics-here-separate-stanzas-with-a-blank-line"),
                            )
                            .font(egui::FontId::proportional(14.0))
                            .layouter(&mut layouter)
                            .show(ui);
                        if output.response.has_focus() {
                            *is_text_focused = true;
                            if let Some(range) = output.cursor_range {
                                let range = range.as_sorted_char_range();
                                state.song_editor.selection = (range.start.0, range.end.0);
                            }
                        }
                    });
            });
    });
}

fn render_grid(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    work: &PresentationTheme,
    tokens: &AppTheme,
    project_slide: &mut Option<usize>,
) {
    let slides = parse_slides_from_text(&state.song_editor.text);
    let zoom_row_h = 34.0;
    let scroll_rect = egui::Rect::from_min_max(
        ui.max_rect().min,
        egui::pos2(ui.max_rect().max.x, ui.max_rect().max.y - zoom_row_h),
    );
    let zoom_rect = egui::Rect::from_min_max(
        egui::pos2(ui.max_rect().min.x, ui.max_rect().max.y - zoom_row_h),
        ui.max_rect().max,
    );

    {
        let mut sui = ui.new_child(egui::UiBuilder::new().max_rect(scroll_rect));
        egui::ScrollArea::vertical()
            .id_salt("song_editor_grid_scroll")
            .auto_shrink([false, false])
            .show(&mut sui, |ui| {
                let columns = 4usize;
                let gap = 14.0;
                let avail = ui.available_width();
                let base_w = ((avail - gap * (columns as f32 - 1.0)) / columns as f32).max(80.0);
                let is_4_3 = state.song_editor.preview_ratio_4_3;
                let card_w = base_w * state.song_editor.zoom;
                let card_h = if is_4_3 {
                    card_w * 3.0 / 4.0
                } else {
                    card_w * 9.0 / 16.0
                };

                let total = slides.len();
                let rows = total.div_ceil(columns);
                for row in 0..rows {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        for col in 0..columns {
                            let index = row * columns + col;
                            if index >= total {
                                break;
                            }
                            let text = slides[index].clone();
                            let selected = state.song_editor.selected_slide == Some(index);
                            let (rect, response) = ui.allocate_exact_size(
                                Vec2::new(card_w, card_h),
                                egui::Sense::click(),
                            );
                            paint_card_bg(ui, rect, work, tokens);
                            let border = if selected {
                                Stroke::new(2.0, tokens.accent)
                            } else {
                                Stroke::new(1.0, tokens.border_default)
                            };
                            ui.painter()
                                .rect_stroke(rect, 10.0, border, StrokeKind::Inside);
                            let display_text = if state.song_editor.hide_syntax_markers {
                                let stripped = crate::core::slides::strip_comments(&text);
                                let aspect = if is_4_3 { 4.0 / 3.0 } else { 16.0 / 9.0 };
                                crate::core::breaks::resolve_conditional_breaks(&stripped, aspect)
                            } else {
                                text.clone()
                            };
                            render_theme_lyrics(ui.painter(), rect.shrink(6.0), work, &display_text);

                            let badge = egui::Rect::from_min_size(
                                rect.min + Vec2::new(8.0, 8.0),
                                Vec2::new(22.0, 18.0),
                            );
                            ui.painter()
                                .rect_filled(badge, 5.0, Color32::from_black_alpha(150));
                            ui.painter().text(
                                badge.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("{}", index + 1),
                                egui::FontId::proportional(11.0),
                                Color32::WHITE,
                            );

                            let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
                            if response.clicked() {
                                state.song_editor.selected_slide = Some(index);
                            }
                            if response.double_clicked() {
                                *project_slide = Some(index);
                            }
                            let _ = response.on_hover_text(
                                crate::ui::i18n::text(ui, "workspace-click-to-select-double-click-to-preview"),
                            );
                        }
                    });
                    ui.add_space(gap);
                }
            });
    }

    {
        let mut zui = ui.new_child(egui::UiBuilder::new().max_rect(zoom_rect));
        zui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            ui.label(
                RichText::new(crate::ui::i18n::text(ui, "workspace-preview-ratio"))
                    .size(12.0)
                    .color(tokens.text_secondary),
            );
            let is_4_3 = state.song_editor.preview_ratio_4_3;
            if ui
                .selectable_label(!is_4_3, "16:9")
                .on_hover_text(crate::ui::i18n::text(ui, "workspace-preview-ratio-16-9-tip"))
                .clicked()
            {
                state.song_editor.preview_ratio_4_3 = false;
            }
            if ui
                .selectable_label(is_4_3, "4:3")
                .on_hover_text(crate::ui::i18n::text(ui, "workspace-preview-ratio-4-3-tip"))
                .clicked()
            {
                state.song_editor.preview_ratio_4_3 = true;
            }

            let hide_markers = state.song_editor.hide_syntax_markers;
            let (eye_icon, eye_tip) = if hide_markers {
                (EYE_SLASH, "Ocultar Marcadores: Ativo (Modo Leitura Limpa). Clique para exibir //, |, [4:3]")
            } else {
                (EYE, "Exibir Marcadores: Visíveis. Clique para ocultar //, |, [4:3] para leitura limpa")
            };
            if ui
                .selectable_label(hide_markers, format!("{eye_icon} Leitura Limpa"))
                .on_hover_text(eye_tip)
                .clicked()
            {
                state.song_editor.hide_syntax_markers = !hide_markers;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(
                    egui::Slider::new(&mut state.song_editor.zoom, 0.5..=1.6)
                        .show_value(false)
                        .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
                )
                .on_hover_text(crate::ui::i18n::text(ui, "workspace-stanza-grid-zoom"));
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "workspace-zoom"))
                        .size(12.0)
                        .color(tokens.text_secondary),
                );
            });
        });
    }
}

fn paint_card_bg(ui: &egui::Ui, rect: egui::Rect, work: &PresentationTheme, tokens: &AppTheme) {
    match work.bg_type.as_str() {
        "video" | "shader" => {
            ui.painter().rect_filled(rect, 10.0, tokens.bg_pill);
        }
        _ => {
            paint_theme_bg(ui.painter(), rect, work, 10.0);
        }
    }
}

fn field_label(ui: &mut egui::Ui, text: &str, tokens: &AppTheme) {
    let text = crate::ui::i18n::text(ui, text);
    ui.label(
        RichText::new(text)
            .size(10.5)
            .strong()
            .color(tokens.text_muted),
    );
}

fn pill_frame(tokens: &AppTheme) -> egui::Frame {
    egui::Frame::new()
        .fill(tokens.bg_pill)
        .stroke(Stroke::new(1.0, tokens.border_default))
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(10, 8))
}


fn tool_button_resp(
    ui: &mut egui::Ui,
    icon: &str,
    active: bool,
    tooltip: &str,
    tokens: &AppTheme,
    enabled: bool,
) -> egui::Response {
    let size = Vec2::new(30.0, 28.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    if !enabled {
        return response.on_hover_text(crate::ui::i18n::text(ui, tooltip));
    }
    let hover = ui
        .ctx()
        .animate_bool(response.id.with("hover"), response.hovered());
    let selected = ui.ctx().animate_bool(response.id.with("sel"), active);
    let bg = tokens
        .bg_panel
        .lerp_to_gamma(tokens.bg_card_hover, hover)
        .lerp_to_gamma(tokens.accent_soft, selected);
    ui.painter().rect_filled(rect, 7.0, bg);
    let color = tokens
        .icon_default
        .lerp_to_gamma(tokens.accent, selected.max(hover * 0.6));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(16.0),
        color,
    );
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(crate::ui::i18n::text(ui, tooltip))
}


fn tool_button(
    ui: &mut egui::Ui,
    icon: &str,
    active: bool,
    tooltip: &str,
    tokens: &AppTheme,
    enabled: bool,
) -> bool {
    tool_button_resp(ui, icon, active, tooltip, tokens, enabled).clicked()
}

fn ghost_button(ui: &mut egui::Ui, label: &str, tokens: &AppTheme) -> egui::Response {
    let label = crate::ui::i18n::text(ui, label);
    let button = egui::Button::new(RichText::new(label).size(13.0).color(tokens.text_primary))
        .fill(tokens.bg_pill)
        .stroke(Stroke::new(1.0, tokens.border_default))
        .corner_radius(7.0)
        .min_size(Vec2::new(86.0, 30.0));
    ui.add(button).on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn primary_button(ui: &mut egui::Ui, icon: &str, label: &str, tokens: &AppTheme) -> egui::Response {
    let label = crate::ui::i18n::text(ui, label);
    let button = egui::Button::new(
        RichText::new(format!("{icon} {label}"))
            .size(13.0)
            .strong()
            .color(tokens.text_on_accent),
    )
    .fill(tokens.accent)
    .stroke(Stroke::NONE)
    .corner_radius(7.0)
    .min_size(Vec2::new(96.0, 30.0));
    ui.add(button).on_hover_cursor(egui::CursorIcon::PointingHand)
}


fn save_song(state: &mut ControlPanelState, slide_deck: &mut SlideDeck, db: Option<&AppDb>) {
    let title = if state.song_editor.title.trim().is_empty() {
        state.settings.language.text("workspace-new-song").to_owned()
    } else {
        state.song_editor.title.trim().to_owned()
    };
    let artist = state.song_editor.artist.trim().to_owned();

    if let Some(app_db) = db {
        let existing_id = app_db
            .search_songs(&title)
            .unwrap_or_default()
            .into_iter()
            .find(|s| s.title == title)
            .and_then(|s| s.id);
        if let Some(id) = existing_id {
            let _ = app_db.update_song_full(
                id,
                &title,
                &artist,
                &state.song_editor.text,
                &state.song_editor.folder,
                &state.song_editor.tags,
            );
        } else {
            let _ = app_db.add_song_full(
                &title,
                &artist,
                &state.song_editor.text,
                &state.song_editor.folder,
                &state.song_editor.tags,
            );
        }
        if let Some(id) = app_db
            .search_songs(&title)
            .unwrap_or_default()
            .into_iter()
            .find(|song| song.title == title)
            .and_then(|song| song.id)
        {
            let _ = app_db.set_song_versions(id, &state.song_editor.linked_versions);
        }
        slide_deck.reload_from_db(app_db);
        if let Some(idx) = slide_deck.library().iter().position(|s| s.title == title) {
            let is_live = slide_deck.is_previewing_live_song();
            let cur_preview_idx = slide_deck.preview_slide_index();
            slide_deck.select_song(idx);
            if cur_preview_idx < slide_deck.slides().len() {
                slide_deck.select_preview_slide(cur_preview_idx);
            }
            if is_live {
                slide_deck.project_preview();
            }
        }
    } else {
        slide_deck.add_or_update_song(
            title.clone(),
            artist.clone(),
            state.song_editor.text.clone(),
        );
    }

    for item in &mut state.playlist {
        if item.kind == crate::core::models::PlaylistItemKind::Song && item.title == title {
            item.subtitle = artist.clone();
        }
    }

    if state.song_editor.theme_dirty
        && let Some(theme) = state.song_editor.theme.clone()
            && state.selected_theme_index < state.themes.len() {
                state.themes[state.selected_theme_index] = theme;
            }

    state.notify(state.settings.language.format("workspace-saved", &[("title", &title)]));
    state.song_editor.theme_dirty = false;
}
