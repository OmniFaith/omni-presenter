use crate::core::layout::ContainerKind;
use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use crate::ui::components::{centered_horizontal, render_header_bar, toolbar_icon_button};
use eframe::egui::{self, RichText, Stroke};
use egui_phosphor::regular::{FUNNEL, LIST_BULLETS, MAGNIFYING_GLASS, SQUARES_FOUR, X};

use super::cards::cards_body;
use super::search::{close_bible_search, matching_books, prepare_search, split_reference};
use super::selection::{select_reference, sync_preview};
use super::version_bar::version_bar;

pub(super) fn toggle_layout(state: &mut ControlPanelState) {
    state.bible.cards_layout = !state.bible.cards_layout;
    state.bible.scroll_to_book = true;
    state.bible.scroll_to_verse = true;
}

pub fn render_bible_navigation(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
) {
    let theme = state.theme.clone();
    render_header_bar(ui, ContainerKind::Selection, "workspace-bible", &theme);
    centered_horizontal(ui, "bible_navigation_toolbar", 108.0, |ui| {
        let search_active = state.bible.show_search || !state.bible.search.is_empty();
        if theme
            .icon_button(
                ui,
                MAGNIFYING_GLASS,
                crate::ui::i18n::text(ui, "workspace-floating-bible-search-or-enter-a-reference"),
                search_active,
                egui::vec2(28.0, 28.0),
            )
            .clicked()
        {
            if state.bible.show_search {
                close_bible_search(state);
            } else {
                state.bible.show_search = true;
                state.bible.search_focus = true;
            }
        }
        let filter = toolbar_icon_button(ui, FUNNEL, crate::ui::i18n::text(ui, "workspace-filter-books"), &theme);
        egui::Popup::menu(&filter).show(|ui| {
            for (value, label) in [
                (0, "workspace-all-books"),
                (1, "workspace-old-testament"),
                (2, "workspace-new-testament"),
            ] {
                if ui
                    .selectable_value(&mut state.bible.testament, value, crate::ui::i18n::text(ui, label))
                    .clicked()
                {
                    ui.close();
                }
            }
        });
        let cards = state.bible.cards_layout;
        let (icon, tooltip) = if cards {
            (LIST_BULLETS, crate::ui::i18n::text(ui, "workspace-list-layout"))
        } else {
            (SQUARES_FOUR, crate::ui::i18n::text(ui, "workspace-card-layout"))
        };
        if state
            .theme
            .icon_button(ui, icon, tooltip, cards, egui::vec2(28.0, 28.0))
            .clicked()
        {
            toggle_layout(state);
        }
    });
    ui.add_space(8.0);
    version_bar(ui, state, &theme);

    if !state.bible.show_search && !state.bible.search.is_empty() {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let edit = ui.add(
                egui::TextEdit::singleline(&mut state.bible.search)
                    .hint_text(crate::ui::i18n::text(ui, "workspace-search-book-reference-or-text"))
                    .desired_width((ui.available_width() - 32.0).max(40.0)),
            );
            if state.bible.search_focus {
                edit.request_focus();
                state.bible.search_focus = false;
            }
            if ui
                .button(X)
                .on_hover_text(crate::ui::i18n::text(ui, "workspace-clear-search"))
                .clicked()
            {
                close_bible_search(state);
            }
        });
        ui.add_space(4.0);
    }

    if (state.bible.show_search || !state.bible.search.is_empty())
        && ui.input(|i| i.key_pressed(egui::Key::Escape))
    {
        close_bible_search(state);
    }

    if !state.bible.show_search && !ui.ctx().egui_wants_keyboard_input() {
        let typed = ui.input(|i| {
            if i.modifiers.ctrl || i.modifiers.command || i.modifiers.alt {
                return None;
            }
            i.events.iter().find_map(|event| match event {
                egui::Event::Text(text) if text.chars().any(char::is_alphanumeric) => {
                    Some(text.clone())
                }
                _ => None,
            })
        });
        if let Some(text) = typed {
            state.bible.show_search = true;
            state.bible.search = text;
            state.bible.search_key.clear();
            state.bible.suggestion_index = 0;
            state.bible.search_focus = true;
        }
    }
    if state.bible.testament != 0 {
        ui.label(
            RichText::new(if state.bible.testament == 1 {
                crate::ui::i18n::text(ui, "workspace-old-testament")
            } else {
                crate::ui::i18n::text(ui, "workspace-new-testament")
            })
            .size(10.5)
            .color(theme.accent),
        );
    }
    prepare_search(state);
    ui.add_space(10.0);

    let query_trimmed = state.bible.search.trim().to_owned();

    // 1. Text search results
    if !query_trimmed.is_empty() && !state.bible.search_results.is_empty()
        && let Some(bible) = state.bible.library.get(&state.bible_version).or_else(|| {
            state.bible.library.get(crate::core::bible::DEFAULT_BIBLE)
        }) {
            ui.horizontal(|ui| {
                let count_str = state.bible.search_results.len().to_string();
                ui.label(
                    RichText::new(crate::ui::i18n::format(
                        ui,
                        "workspace-verses-found",
                        &[("count", &count_str)],
                    ))
                    .size(11.0)
                    .color(theme.text_secondary),
                );
            });
            ui.add_space(4.0);
            let hits = state.bible.search_results.clone();
            egui::ScrollArea::vertical()
                .id_salt("bible_search_results_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for hit in &hits {
                        let book_name = &bible.books[hit.book].name;
                        let verse_text = bible
                            .chapter(hit.book, hit.chapter)
                            .and_then(|ch| ch.get(hit.verse - 1))
                            .map(|s| s.as_str())
                            .unwrap_or("");

                        let is_selected = state.bible_book == *book_name
                            && state.bible_chapter == hit.chapter
                            && state.bible_verse == hit.verse;

                        let (rect, resp) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 52.0),
                            egui::Sense::click(),
                        );
                        if resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        let fill = if is_selected {
                            theme.accent_soft
                        } else if resp.hovered() {
                            theme.bg_card_hover
                        } else {
                            theme.bg_card
                        };
                        ui.painter().rect_filled(rect, 6.0, fill);
                        let stroke_c = if is_selected {
                            theme.accent
                        } else {
                            theme.border_subtle
                        };
                        ui.painter().rect_stroke(
                            rect,
                            6.0,
                            Stroke::new(1.0, stroke_c),
                            egui::StrokeKind::Inside,
                        );

                        let painter = ui.painter().with_clip_rect(rect.shrink(5.0));
                        painter.text(
                            rect.left_top() + egui::vec2(8.0, 5.0),
                            egui::Align2::LEFT_TOP,
                            format!("{} {}:{}", book_name, hit.chapter, hit.verse),
                            egui::FontId::proportional(12.0),
                            if is_selected {
                                theme.accent
                            } else {
                                theme.text_primary
                            },
                        );
                        let galley = painter.layout(
                            verse_text.to_owned(),
                            egui::FontId::proportional(10.5),
                            theme.text_secondary,
                            rect.width() - 16.0,
                        );
                        painter.galley(
                            rect.left_top() + egui::vec2(8.0, 24.0),
                            galley,
                            theme.text_secondary,
                        );

                        if resp.clicked() {
                            select_reference(state, hit.book, hit.chapter, hit.verse, hit.verse);
                            sync_preview(state, deck, true);
                        }
                        ui.add_space(3.0);
                    }
                });
            return;
        }

    // 2. No results at all when search query is typed and no matching books
    if !query_trimmed.is_empty()
        && let Some(bible) = state.bible.library.get(&state.bible_version).or_else(|| {
            state.bible.library.get(crate::core::bible::DEFAULT_BIBLE)
        }) {
            let (book_part, _) = split_reference(&query_trimmed);
            let matching = matching_books(&bible, &book_part);
            if matching.is_empty() {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(crate::ui::i18n::text(ui, "workspace-no-books-or-verses-found"))
                            .size(12.5)
                            .color(theme.text_muted),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(crate::ui::i18n::text(ui, "workspace-try-a-book-name-e-g-psalms-john-or-a"))
                            .size(11.0)
                            .color(theme.text_secondary),
                    );
                });
                return;
            }
        }

    if state.bible.cards_layout {
        cards_body(ui, state, deck, &theme);
        sync_preview(state, deck, false);
        return;
    }
    let bible = state
        .bible
        .library
        .get(&state.bible_version)
        .unwrap_or_else(|| {
            state
                .bible
                .library
                .get(crate::core::bible::DEFAULT_BIBLE)
                .unwrap()
        });

    let matching_indices = if query_trimmed.is_empty() {
        None
    } else {
        let (book_part, _) = split_reference(&query_trimmed);
        let m = matching_books(&bible, &book_part);
        if m.is_empty() { None } else { Some(m) }
    };
    let visible_books: Vec<usize> = if let Some(ref m) = matching_indices {
        m.clone()
    } else {
        (0..bible.books.len()).collect()
    };

    egui::ScrollArea::vertical()
        .id_salt("bible_books")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for &index in &visible_books {
                if (state.bible.testament == 1 && index >= 39)
                    || (state.bible.testament == 2 && index < 39)
                {
                    continue;
                }
                let book = &bible.books[index];
                let selected = state.bible_book == book.name;
                let expanded = state.bible.expanded_book == Some(index);
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 35.0),
                    egui::Sense::click(),
                );
                let hover = ui
                    .ctx()
                    .animate_bool(response.id.with("hover"), response.hovered());
                let down = ui.ctx().animate_bool(
                    response.id.with("down"),
                    response.is_pointer_button_down_on(),
                );
                let fill = if down > 0.05 {
                    theme
                        .bg_card_hover
                        .lerp_to_gamma(theme.accent_soft, down * 0.45)
                } else {
                    theme.bg_card.lerp_to_gamma(theme.bg_card_hover, hover)
                };
                ui.painter().rect_filled(rect, 7.0, fill);
                ui.painter().text(
                    rect.left_center() + egui::vec2(6.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    format!("{}. {}", index + 1, book.name),
                    egui::FontId::proportional(12.5),
                    if selected {
                        theme.text_primary
                    } else {
                        theme.text_secondary
                    },
                );
                let openness = ui.ctx().animate_bool(response.id.with("open"), expanded);
                let center = rect.right_center() - egui::vec2(12.0, 0.0);
                let direction = if openness > 0.5 { -1.0 } else { 1.0 };
                ui.painter().line_segment(
                    [
                        center + egui::vec2(-4.0, -2.0 * direction),
                        center + egui::vec2(0.0, 2.0 * direction),
                    ],
                    Stroke::new(
                        1.2,
                        if expanded {
                            theme.accent
                        } else {
                            theme.icon_muted
                        },
                    ),
                );
                ui.painter().line_segment(
                    [
                        center + egui::vec2(0.0, 2.0 * direction),
                        center + egui::vec2(4.0, -2.0 * direction),
                    ],
                    Stroke::new(
                        1.2,
                        if expanded {
                            theme.accent
                        } else {
                            theme.icon_muted
                        },
                    ),
                );
                if selected && state.bible.scroll_to_book {
                    response.scroll_to_me(Some(egui::Align::Center));
                    state.bible.scroll_to_book = false;
                }
                let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
                if response.clicked() {
                    state.bible.expanded_book = if expanded { None } else { Some(index) };
                    if !selected {
                        select_reference(state, index, 1, 1, 1);
                        state.bible.scroll_to_book = false;
                    }
                }
                let mut collapse = egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    ui.make_persistent_id(("bible_chapters", index)),
                    expanded,
                );
                collapse.set_open(state.bible.expanded_book == Some(index));
                collapse.show_body_unindented(ui, |ui| {
                    let width = ui.available_width();
                    let min_cell_w = if book.chapters.len() >= 100 {
                        50.0
                    } else {
                        44.0
                    };
                    let btn_h = if book.chapters.len() >= 100 {
                        36.0
                    } else {
                        33.0
                    };
                    let columns = (((width + 5.0) / (min_cell_w + 5.0)).floor() as usize).max(1);
                    let cell = ((width - (columns - 1) as f32 * 5.0) / columns as f32).floor();
                    for row in (0..book.chapters.len()).step_by(columns) {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 5.0;
                            for chapter in row + 1..=(row + columns).min(book.chapters.len()) {
                                let active = selected && state.bible_chapter == chapter;
                                let (rect, response) = ui.allocate_exact_size(
                                    egui::vec2(cell, btn_h),
                                    egui::Sense::click(),
                                );
                                let hover = ui
                                    .ctx()
                                    .animate_bool(response.id.with("h"), response.hovered());
                                let down = ui.ctx().animate_bool(
                                    response.id.with("d"),
                                    response.is_pointer_button_down_on(),
                                );
                                let response =
                                    response.on_hover_cursor(egui::CursorIcon::PointingHand);

                                let fill = if active {
                                    theme.accent_soft
                                } else if down > 0.05 {
                                    theme
                                        .bg_card_hover
                                        .lerp_to_gamma(theme.accent_soft, down * 0.45)
                                } else {
                                    theme.bg_pill.lerp_to_gamma(theme.bg_card_hover, hover)
                                };

                                let stroke = if active {
                                    Stroke::new(1.5, theme.accent)
                                } else if down > 0.05 {
                                    Stroke::new(1.3, theme.accent.linear_multiply(0.85))
                                } else if hover > 0.05 {
                                    Stroke::new(1.2, theme.accent.linear_multiply(0.55))
                                } else {
                                    Stroke::new(1.0, theme.border_default)
                                };

                                ui.painter().rect(
                                    rect,
                                    8.0,
                                    fill,
                                    stroke,
                                    egui::StrokeKind::Inside,
                                );

                                let is_pressed = response.is_pointer_button_down_on();
                                let text_color = if active {
                                    theme.accent
                                } else if is_pressed || hover > 0.5 {
                                    theme.text_primary
                                } else {
                                    theme.text_secondary
                                };

                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    chapter.to_string(),
                                    if active || is_pressed {
                                        egui::FontId::proportional(13.0)
                                    } else {
                                        egui::FontId::proportional(12.5)
                                    },
                                    text_color,
                                );

                                if response.clicked() {
                                    select_reference(state, index, chapter, 1, 1);
                                    state.bible.scroll_to_book = false;
                                }
                            }
                        });
                    }
                    ui.add_space(8.0);
                });
            }
        });
    sync_preview(state, deck, false);
}
