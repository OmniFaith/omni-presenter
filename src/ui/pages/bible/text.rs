use crate::core::bible::version_code;
use crate::core::layout::ContainerKind;
use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use crate::ui::components::{centered_horizontal, render_header_bar};
use eframe::egui::{self, RichText, Stroke};
use egui_phosphor::regular::{CARET_LEFT, CARET_RIGHT, LIST_PLUS, PLAY, PLUS, PUSH_PIN};

use super::helpers::current_book;
use super::import::open_bible_settings;
use super::selection::{
    add_to_playlist, is_in_selection, pin_passage, project, select_reference, selected_passage,
    selection_passages, selection_toolbar, sync_preview,
};

fn verse_row(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    number: &str,
    text: &str,
    selected: bool,
    live: bool,
    theme: &crate::ui::theme::AppTheme,
) -> egui::Response {
    ui.push_id(id, |ui| {
        let font_size = 14.0;
        let galley = ui.painter().layout(
            text.into(),
            egui::FontId::proportional(font_size),
            if selected {
                theme.text_primary
            } else {
                theme.text_secondary
            },
            (ui.available_width() - 46.0).max(40.0),
        );
        let height = galley.size().y + 26.0;
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), height.max(42.0)),
            egui::Sense::click_and_drag(),
        );
        let hover = ui
            .ctx()
            .animate_bool(response.id.with("hover"), response.hovered());
        let active = ui
            .ctx()
            .animate_bool(response.id.with("selected"), selected);
        let fill = theme
            .bg_card
            .lerp_to_gamma(theme.bg_card_hover, hover)
            .lerp_to_gamma(theme.bg_card_active, active);
        ui.painter().rect_filled(rect, 10.0, fill);
        if live {
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    rect.min + egui::vec2(0.0, 10.0),
                    egui::vec2(3.0, (height - 20.0).max(10.0)),
                ),
                2.0,
                theme.status_success_fg,
            );
        }
        let top = 13.0;
        ui.painter().text(
            rect.min + egui::vec2(10.0, top),
            egui::Align2::LEFT_TOP,
            number,
            egui::FontId::proportional(font_size),
            if selected {
                theme.accent
            } else {
                theme.text_secondary
            },
        );
        ui.painter()
            .galley(rect.min + egui::vec2(34.0, top), galley, theme.text_primary);
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    })
    .inner
}

fn chapter_nav_button(
    ui: &mut egui::Ui,
    id_source: &str,
    icon: &str,
    enabled: bool,
    tooltip: &str,
    theme: &crate::ui::theme::AppTheme,
) -> egui::Response {
    let id = ui.make_persistent_id(id_source);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(26.0, 24.0),
        if enabled {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        },
    );
    if !enabled {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(12.0),
            theme.text_muted.linear_multiply(0.28),
        );
        return response;
    }

    let response = response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(crate::ui::i18n::text(ui, tooltip));

    let hover = ui.ctx().animate_bool(id.with("h"), response.hovered());
    let down = ui
        .ctx()
        .animate_bool(id.with("d"), response.is_pointer_button_down_on());

    let fill = if down > 0.05 {
        theme
            .bg_card_active
            .lerp_to_gamma(theme.accent_soft, down * 0.45)
    } else if hover > 0.05 {
        egui::Color32::TRANSPARENT.lerp_to_gamma(theme.bg_card_hover, hover)
    } else {
        egui::Color32::TRANSPARENT
    };

    let stroke = if down > 0.05 {
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

    let is_pressed = response.is_pointer_button_down_on();
    let text_color = if is_pressed {
        theme.accent
    } else if hover > 0.05 {
        theme.text_primary.lerp_to_gamma(theme.accent, hover * 0.75)
    } else {
        theme.text_primary
    };

    let draw_rect = rect.shrink(down * 0.75);
    ui.painter()
        .rect(draw_rect, 7.0, fill, stroke, egui::StrokeKind::Inside);
    ui.painter().text(
        draw_rect.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        if is_pressed {
            egui::FontId::proportional(12.5)
        } else {
            egui::FontId::proportional(12.0)
        },
        text_color,
    );

    response
}

pub fn render_bible_text(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    presenting: &mut bool,
) {
    let enter_pressed = !ui.ctx().egui_wants_keyboard_input()
        && !state.bible.show_search
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));

    if enter_pressed && !selection_passages(state).is_empty() {
        project(state, deck, presenting);
        ui.ctx().request_repaint();
    }

    let theme = state.theme.clone();
    sync_preview(state, deck, false);
    render_header_bar(ui, ContainerKind::Lyrics, "workspace-text-preview", &theme);
    selection_toolbar(ui, state, deck, presenting, &theme);
    ui.add_space(8.0);
    let Some(bible) = state.bible.library.get(&state.bible_version) else {
        ui.add_space(30.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(crate::ui::i18n::format(ui, "workspace-add-version", &[("version", &version_code(&state.bible_version))])).size(16.0).strong());
            ui.add_space(8.0);
            ui.label(RichText::new(crate::ui::i18n::text(ui, "workspace-import-this-translation-s-json-file-to-look-up-and")).size(12.0).color(theme.text_secondary));
            ui.add_space(12.0);
            if ui.button(crate::ui::i18n::format(ui, "workspace-import-in-settings", &[("icon", PLUS)])).clicked() { open_bible_settings(state); }
        });
        return;
    };
    let book = current_book(state, &bible);
    let searching = !state.bible.search.trim().is_empty()
        && bible.reference(&state.bible.search).is_none()
        && state.bible.search.trim().chars().count() >= 2;
    centered_horizontal(ui, "bible_chapter_title_banner", 160.0, |ui| {
        if !searching {
            let previous = state.bible_chapter > 1;
            if chapter_nav_button(
                ui,
                "bible_prev_chapter",
                CARET_LEFT,
                previous,
                "workspace-previous-chapter",
                &theme,
            )
            .clicked()
            {
                select_reference(state, book, state.bible_chapter - 1, 1, 1);
            }
        }
        let title = if searching {
            let count_str = state.bible.search_results.len().to_string();
            crate::ui::i18n::format(ui, "workspace-results", &[("count", &count_str)])
        } else if state.bible_verse != state.bible.verse_end {
            format!(
                "{} {}:{}–{}",
                state.bible_book,
                state.bible_chapter,
                state.bible_verse.min(state.bible.verse_end),
                state.bible_verse.max(state.bible.verse_end)
            )
        } else {
            selected_passage(state)
                .map(|p| p.reference)
                .unwrap_or_else(|| format!("{} {}", state.bible_book, state.bible_chapter))
        };
        ui.label(
            RichText::new(title)
                .size(13.0)
                .strong()
                .color(theme.text_primary),
        );
        ui.label(RichText::new(&bible.id).size(10.5).color(theme.text_muted))
            .on_hover_text(&bible.name);
        if !searching {
            let next = state.bible_chapter < bible.books[book].chapters.len();
            if chapter_nav_button(
                ui,
                "bible_next_chapter",
                CARET_RIGHT,
                next,
                "workspace-next-chapter",
                &theme,
            )
            .clicked()
            {
                select_reference(state, book, state.bible_chapter + 1, 1, 1);
            }
        }
    });
    ui.add_space(8.0);
    let scroll_height = ui.available_height().max(70.0);
    egui::ScrollArea::vertical().id_salt("bible_verses").max_height(scroll_height).min_scrolled_height(scroll_height).auto_shrink([false, false]).show(ui, |ui| {
        if searching {
            let hits = state.bible.search_results.clone();
            if hits.is_empty() { ui.label(RichText::new(crate::ui::i18n::text(ui, "workspace-no-verses-found-try-another-word-or-a-reference-such")).color(theme.text_secondary)); }
            for hit in hits {
                let reference = format!("{} {}:{}", bible.books[hit.book].name, hit.chapter, hit.verse);
                ui.label(RichText::new(&reference).size(11.0).color(theme.accent));
                let text = &bible.chapter(hit.book, hit.chapter).unwrap()[hit.verse - 1];
                let selected = state.bible_book == bible.books[hit.book].name && state.bible_chapter == hit.chapter && state.bible_verse == hit.verse;
                let response = verse_row(ui, ("search", hit.book, hit.chapter, hit.verse), &format!("{}.", hit.verse), text, selected, false, &theme);
                drag_passage(ui, &response, || bible.passage(hit.book, hit.chapter, hit.verse, hit.verse));
                if response.clicked() || response.double_clicked() {
                    select_reference(state, hit.book, hit.chapter, hit.verse, hit.verse);
                    state.bible.scroll_to_verse = false;
                    sync_preview(state, deck, true);
                    if response.double_clicked() { project(state, deck, presenting); }
                }
                ui.add_space(8.0);
            }
            if state.bible.search_results.len() == 120 { ui.label(RichText::new(crate::ui::i18n::text(ui, "workspace-showing-the-first-120-results-refine-your-search-to-find")).size(11.0).color(theme.text_secondary)); }
        } else if let Some(verses) = bible.chapter(book, state.bible_chapter) {
            for (index, text) in verses.iter().enumerate() {
                let number = index + 1;
                let reference = format!("{} {}:{number}", state.bible_book, state.bible_chapter);
                let selected = number >= state.bible_verse && number <= state.bible.verse_end;
                let live = *presenting && deck.live_artist() == bible.id && deck.live_slide_text().ends_with(&reference);
                let display = if text.is_empty() { crate::ui::i18n::text(ui, "workspace-this-source-has-no-text-for-this-verse") } else { text.as_str() };
                let response = verse_row(ui, ("verse", book, state.bible_chapter, number), &format!("{number}."), display, selected, live, &theme);
                drag_passage(ui, &response, || if selected { selected_passage(state) } else { bible.passage(book, state.bible_chapter, number, number) });
                if selected && state.bible.scroll_to_verse { response.scroll_to_me(Some(egui::Align::Center)); state.bible.scroll_to_verse = false; }
                if !text.is_empty() {
                    if response.clicked() || response.double_clicked() {
                        if ui.input(|i| i.modifiers.shift) {
                            state.bible_verse = state.bible.selection_anchor.min(number);
                            state.bible.verse_end = state.bible.selection_anchor.max(number);
                        } else { state.bible_verse = number; state.bible.verse_end = number; state.bible.selection_anchor = number; }
                        sync_preview(state, deck, true);
                        state.capture_profiles();
                        if response.double_clicked() { project(state, deck, presenting); }
                    }
                    response.context_menu(|ui| {
                        let keep_selection = is_in_selection(state, number);
                        let count = state.bible.verse_end.abs_diff(state.bible_verse) + 1;
                        let action_label = if keep_selection && count > 1 {
                            let count_str = count.to_string();
                            crate::ui::i18n::format(ui, "workspace-present-verses", &[("icon", PLAY), ("count", &count_str)])
                        } else {
                            crate::ui::i18n::format(ui, "workspace-present-this-verse", &[("icon", PLAY)])
                        };
                        if ui.button(action_label).clicked() {
                            if !keep_selection { select_reference(state, book, state.bible_chapter, number, number); }
                            project(state, deck, presenting);
                            ui.close();
                        }
                        if ui.button(crate::ui::i18n::format(ui, "workspace-add-to-playlist-label", &[("icon", LIST_PLUS)])).clicked() {
                            if !keep_selection { select_reference(state, book, state.bible_chapter, number, number); }
                            add_to_playlist(state);
                            ui.close();
                        }
                        if ui.button(crate::ui::i18n::format(ui, "workspace-pin-to-quick-bar-label", &[("icon", PUSH_PIN)])).clicked() {
                            if !keep_selection { select_reference(state, book, state.bible_chapter, number, number); }
                            pin_passage(state);
                            ui.close();
                        }
                    });
                }
                ui.add_space(6.0);
            }
        }
    });
    sync_preview(state, deck, false);
}

fn drag_passage(
    ui: &egui::Ui,
    response: &egui::Response,
    passage: impl FnOnce() -> Option<crate::core::bible::BiblePassage>,
) {
    if response.drag_started()
        && let Some(passage) = passage() {
            use crate::ui::components::drag_drop::{DragPayload, DragSource};
            let mut payload = DragPayload::new(
                DragSource::Library,
                &passage.reference,
                &passage.version,
                egui_phosphor::regular::BOOK_OPEN,
            );
            payload.items.push(crate::core::models::QuickItem {
                id: crate::core::preferences::new_id("verse"),
                title: passage.reference,
                subtitle: passage.version,
                kind: crate::core::models::QuickItemKind::Verse,
                content: passage.content,
            });
            egui::DragAndDrop::set_payload(ui.ctx(), payload);
        }
}
