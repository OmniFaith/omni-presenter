use crate::core::bible::{BibleTranslation, SearchHit, fold};
use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use eframe::egui::{self, RichText, Stroke};

use super::selection::{select_reference, sync_preview};

pub(super) fn prepare_search(state: &mut ControlPanelState) {
    let key = format!("{}:{}", state.bible_version, state.bible.search);
    if state.bible.search_key == key {
        return;
    }
    state.bible.search_key = key;
    state.bible.search_results.clear();
    let Some(bible) = state.bible.library.get(&state.bible_version) else {
        return;
    };
    let query = state.bible.search.trim();
    if query.is_empty() {
        return;
    }
    if let Some((book, chapter, start, end)) = bible.reference(query) {
        select_reference(state, book, chapter, start, end);
        return;
    }
    if !matching_books(&bible, query).is_empty() || query.chars().count() < 2 {
        return;
    }
    let needle: Vec<char> = fold(query).chars().collect();
    'books: for (book, data) in bible.books.iter().enumerate() {
        for (chapter, verses) in data.chapters.iter().enumerate() {
            for (verse, text) in verses.iter().enumerate() {
                if contains_folded(text, &needle) {
                    state.bible.search_results.push(SearchHit {
                        book,
                        chapter: chapter + 1,
                        verse: verse + 1,
                    });
                    if state.bible.search_results.len() >= 120 {
                        break 'books;
                    }
                }
            }
        }
    }
}

fn fold_char(c: char) -> char {
    match c {
        'á' | 'à' | 'ã' | 'â' | 'ä' | 'Á' | 'À' | 'Ã' | 'Â' | 'Ä' => 'a',
        'é' | 'ê' | 'è' | 'ë' | 'É' | 'Ê' | 'È' | 'Ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'i',
        'ó' | 'ô' | 'õ' | 'ò' | 'ö' | 'Ó' | 'Ô' | 'Õ' | 'Ò' | 'Ö' => 'o',
        'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
        'ç' | 'Ç' => 'c',
        other => other.to_ascii_lowercase(),
    }
}

/// Case/accent-insensitive `contains` that never allocates, so the whole Bible can be
/// scanned on every keystroke without stuttering.
fn contains_folded(haystack: &str, needle: &[char]) -> bool {
    if needle.is_empty() {
        return true;
    }
    let first = needle[0];
    let mut matched = 0;
    for c in haystack.chars() {
        let folded = fold_char(c);
        if folded == needle[matched] {
            matched += 1;
            if matched == needle.len() {
                return true;
            }
        } else if folded == first {
            matched = 1;
        } else {
            matched = 0;
        }
    }
    false
}

pub(super) use crate::core::bible_search::{matching_books, number_is_complete, parse_query, split_reference};
#[cfg(test)]
pub(super) use crate::core::bible_search::QueryParse;

pub(super) fn close_bible_search(state: &mut ControlPanelState) {
    state.bible.show_search = false;
    state.bible.search_focus = false;
    state.bible.search.clear();
    state.bible.search_results.clear();
    state.bible.search_key.clear();
    state.bible.suggestion_index = 0;
}

#[derive(Clone)]
enum SearchAction {
    Book(usize),
    Chapter(usize, usize),
    Apply(usize, usize, usize, usize),
}

fn run_search_action(
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    bible: &BibleTranslation,
    action: SearchAction,
) {
    state.bible.suggestion_index = 0;
    state.bible.search_key.clear();
    state.bible.search_focus = true;
    match action {
        SearchAction::Book(book) => {
            state.bible.search = if bible.books[book].chapters.len() <= 1 {
                format!("{} 1:", bible.books[book].name)
            } else {
                format!("{} ", bible.books[book].name)
            };
        }
        SearchAction::Chapter(book, chapter) => {
            state.bible.search = format!("{} {}:", bible.books[book].name, chapter);
        }
        SearchAction::Apply(book, chapter, start, end) => {
            state.active_sidebar_page = crate::core::state::SidebarPage::Bible;
            select_reference(state, book, chapter, start, end);
            close_bible_search(state);
            sync_preview(state, deck, true);
        }
    }
}

/// Full-screen staged search: pick a book, then a chapter, then a verse. Every stage completes
/// the reference and reveals the next one, but a full reference can still be typed at once.
/// `Esc` cancels, `Enter` picks the highlighted tile, the arrows move around the grid.
pub fn render_bible_search_overlay(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
) {
    if !state.bible.show_search {
        return;
    }
    prepare_search(state);
    let theme = state.theme.clone();
    let screen = ctx.content_rect();
    let overlay_id = egui::Id::unique("bible_search_overlay");
    let field_id = overlay_id.with("field");
    egui::Area::new(overlay_id).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
        ui.set_min_size(screen.size());
        ui.set_max_size(screen.size());
        let backdrop = ui.interact(screen, overlay_id.with("backdrop"), egui::Sense::click());
        ui.painter().rect_filled(screen, 0.0, egui::Color32::from_black_alpha(240));
        let close = backdrop.clicked() || ctx.input(|i| i.key_pressed(egui::Key::Escape));
        let (down, up, left, right, enter, tab) = ctx.input(|i| (
            i.key_pressed(egui::Key::ArrowDown), i.key_pressed(egui::Key::ArrowUp),
            i.key_pressed(egui::Key::ArrowLeft), i.key_pressed(egui::Key::ArrowRight),
            i.key_pressed(egui::Key::Enter), i.key_pressed(egui::Key::Tab),
        ));

        let Some(bible) = state
            .bible
            .library
            .get(&state.bible_version)
            .or_else(|| state.bible.library.get(crate::core::bible::DEFAULT_BIBLE))
        else {
            return;
        };
        let query = state.bible.search.trim().to_owned();
        let parsed = parse_query(&bible, &query);

        let stage = if !parsed.book_complete { 0 } else if !parsed.has_separator { 1 } else { 2 };
        let book = parsed.book;
        let chapters = book.map(|b| bible.books[b].chapters.len()).unwrap_or(1).max(1);
        let chapter = parsed.chapter.map(|c| c.clamp(1, chapters)).unwrap_or(1);
        let verses = book.map(|b| bible.books[b].chapters[chapter - 1].len()).unwrap_or(1).max(1);

        let (tiles, columns): (Vec<(String, SearchAction)>, usize) = match stage {
            0 if !parsed.matches.is_empty() => (parsed.matches.iter().map(|&b| (bible.books[b].name.clone(), SearchAction::Book(b))).collect(), 3),
            0 => (state.bible.search_results.iter().map(|h| (format!("{} {}:{}", bible.books[h.book].name, h.chapter, h.verse), SearchAction::Apply(h.book, h.chapter, h.verse, h.verse))).take(60).collect(), 3),
            1 => ((1..=chapters).map(|c| (c.to_string(), SearchAction::Chapter(book.expect("book is set"), c))).collect(), 8),
            _ => ((1..=verses).map(|v| (v.to_string(), SearchAction::Apply(book.expect("book is set"), chapter, v, v))).collect(), 8),
        };
        let len = tiles.len();
        let kind = match stage {
            0 => "workspace-book",
            1 => "workspace-chapter",
            _ => "workspace-verse",
        };
        let hint = match stage {
            0 if parsed.matches.is_empty() => {
                let len_str = len.to_string();
                crate::ui::i18n::format(ui, "workspace-text-results", &[("len", &len_str)])
            }
            0 => {
                let len_str = len.to_string();
                crate::ui::i18n::format(ui, "workspace-books-found", &[("count", &len_str)])
            }
            1 => {
                let ch_str = chapters.to_string();
                crate::ui::i18n::format(ui, "workspace-chapters", &[("chapters", &ch_str)])
            }
            _ => {
                let v_str = verses.to_string();
                crate::ui::i18n::format(ui, "workspace-verses", &[("verses", &v_str)])
            }
        };
        let preview = match stage {
            0 => book.map(|b| bible.books[b].name.clone()).unwrap_or_else(|| query.clone()),
            1 if parsed.chapter.is_some() => format!("{} {}", bible.books[book.unwrap()].name, parsed.chapter.unwrap()),
            1 => bible.books[book.unwrap()].name.clone(),
            _ => match parsed.verse {
                Some(v) => format!("{} {chapter}:{}", bible.books[book.unwrap()].name, v.clamp(1, verses)),
                None => format!("{} {chapter}:", bible.books[book.unwrap()].name),
            },
        };

        // Autocompletes the book name while the user is typing: the completion is computed
        // after the keystroke is applied (below), so backspacing (no `Text` event) never
        // re-adds the name.
        let inserted_text = ctx.input(|i| i.events.iter().any(|event| matches!(event, egui::Event::Text(text) if text.chars().any(char::is_alphabetic))));
        let inserted_digit = ctx.input(|i| i.events.iter().any(|event| matches!(event, egui::Event::Text(text) if text.chars().any(|c| c.is_ascii_digit()))));
        let mut autocomplete: Option<String> = None;
        let mut chapter_complete: Option<(usize, usize)> = None;
        let mut verse_complete: Option<(usize, usize, usize)> = None;

        let mut action: Option<SearchAction> = None;
        let mut index = state.bible.suggestion_index.min(len.saturating_sub(1));
        let typed = match stage {
            1 => parsed.chapter.and_then(|c| c.checked_sub(1)),
            2 => parsed.verse.and_then(|v| v.checked_sub(1)),
            _ => None,
        };
        if let Some(typed) = typed {
            index = typed.min(len.saturating_sub(1));
        }
        let panel_height = 250.0 + (len.div_ceil(columns).min(8) as f32) * 34.0;
        ui.scope_builder(egui::UiBuilder::new().max_rect(screen).layout(egui::Layout::top_down(egui::Align::Center)), |ui| {
            ui.add_space(((screen.height() - panel_height) / 2.0).max(20.0));
            egui::Frame::new()
                .fill(theme.bg_panel)
                .stroke(Stroke::new(1.0, theme.border_default))
                .corner_radius(18.0)
                .inner_margin(24)
                .show(ui, |ui| {
                    ui.set_width(600.0);
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new(crate::ui::i18n::text(ui, "workspace-press-esc-to-cancel")).size(11.5).color(theme.text_muted));
                        ui.add_space(10.0);
                        ui.label(RichText::new(crate::ui::i18n::text(ui, kind)).size(12.5).color(theme.text_secondary));
                        ui.add_space(6.0);
                        let field = egui::Frame::new().fill(theme.bg_card).corner_radius(12.0).inner_margin(egui::Margin { left: 14, right: 14, top: 6, bottom: 6 })
                            .show(ui, |ui| {
                                ui.set_width(560.0);
                                ui.add(
                                    egui::TextEdit::singleline(&mut state.bible.search)
                                        .id(field_id)
                                        .font(egui::FontId::proportional(30.0))
                                        .align(egui::Align2::CENTER_CENTER)
                                        .desired_width(560.0)
                                        .frame(egui::Frame::NONE)
                                        .text_color(theme.text_primary)
                                        .hint_text(RichText::new(crate::ui::i18n::text(ui, "workspace-type-or-choose-below")).size(19.0).color(theme.text_muted)),
                                )
                            }).inner;
                        if state.bible.search_focus {
                            field.request_focus();
                            state.bible.search_focus = false;
                        }
                        if inserted_text {
                            let (book_part, numbers) = split_reference(&state.bible.search);
                            let matches = matching_books(&bible, &book_part);
                            if let [book] = matches[..] {
                                let full = fold(&bible.books[book].name).replace(' ', "");
                                if book_part != full {
                                    let name = bible.books[book].name.clone();
                                    let chapters = bible.books[book].chapters.len();
                                    autocomplete = Some(if !numbers.is_empty() {
                                        format!("{name} {numbers}")
                                    } else if chapters <= 1 {
                                        format!("{name} 1:")
                                    } else {
                                        format!("{name} ")
                                    });
                                }
                            }
                        }
                        if inserted_digit {
                            let (book_part, numbers) = split_reference(&state.bible.search);
                            let matches = matching_books(&bible, &book_part);
                            if let [book] = matches[..] {
                                let chapters = bible.books[book].chapters.len().max(1);
                                if !numbers.contains([':', '.']) {
                                    if let Ok(chapter) = numbers.parse::<usize>()
                                        && number_is_complete(&numbers, chapters)
                                    {
                                        chapter_complete = Some((book, chapter.clamp(1, chapters)));
                                    }
                                } else if let Some((chapter_text, verse_text)) = numbers.split_once([':', '.'])
                                    && let (Ok(chapter), Ok(verse)) = (chapter_text.parse::<usize>(), verse_text.parse::<usize>())
                                {
                                    let chapter = chapter.clamp(1, chapters);
                                    let verses = bible.books[book].chapters[chapter - 1].len().max(1);
                                    if number_is_complete(verse_text, verses) {
                                        verse_complete = Some((book, chapter, verse.clamp(1, verses)));
                                    }
                                }
                            }
                        }
                        ui.add_space(10.0);
                        if !query.is_empty() {
                            ui.label(RichText::new(format!("→ {preview}")).size(18.0).color(theme.accent));
                        }
                        ui.add_space(2.0);
                        ui.label(RichText::new(&hint).size(11.5).color(theme.text_muted));
                        if len > 0 {
                            if down { index = (index + columns).min(len - 1); }
                            if up { index = index.saturating_sub(columns); }
                            if right { index = (index + 1).min(len - 1); }
                            if left { index = index.saturating_sub(1); }
                            if enter || tab { action = Some(tiles[index].1.clone()); }
                            ui.add_space(14.0);
                            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                                for (row, chunk) in tiles.chunks(columns).enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 6.0;
                                        for (column, (label, tile_action)) in chunk.iter().enumerate() {
                                            let position = row * columns + column;
                                            let selected = position == index;
                                            let text = RichText::new(label).size(13.0).color(if selected { theme.accent } else { theme.text_primary });
                                            let button = egui::Button::new(text)
                                                .fill(if selected { theme.accent_soft } else { theme.bg_card })
                                                .stroke(Stroke::new(1.0, if selected { theme.accent } else { theme.border_default }))
                                                .corner_radius(8.0);
                                            let size = if stage == 0 { egui::vec2(188.0, 30.0) } else { egui::vec2(62.0, 30.0) };
                                            let response = ui.add_sized(size, button);
                                            if selected { response.scroll_to_me(None); }
                                            if response.clicked() {
                                                index = position;
                                                action = Some(tile_action.clone());
                                            }
                                        }
                                    });
                                    ui.add_space(6.0);
                                }
                            });
                        }
                    });
                });
        });

        state.bible.suggestion_index = index;
        if let Some((book, chapter, verse)) = verse_complete {
            state.active_sidebar_page = crate::core::state::SidebarPage::Bible;
            select_reference(state, book, chapter, verse, verse);
            close_bible_search(state);
            sync_preview(state, deck, true);
        } else if let Some((book, chapter)) = chapter_complete {
            let name = bible.books[book].name.clone();
            state.bible.search = format!("{name} {chapter}:");
            state.bible.search_key.clear();
            state.bible.suggestion_index = 0;
            state.bible.search_focus = true;
            let restored = egui::TextEdit::load_state(ctx, field_id);
            if let Some(mut edit) = restored {
                let end = egui::text::CCursor::new(state.bible.search.chars().count());
                edit.cursor.set_char_range(Some(egui::text::CCursorRange::one(end)));
                edit.store(ctx, field_id);
            }
        } else if let Some(action) = action {
            let completing = matches!(action, SearchAction::Book(_) | SearchAction::Chapter(_, _));
            run_search_action(state, deck, &bible, action);
            let restored = if completing { egui::TextEdit::load_state(ctx, field_id) } else { None };
            if let Some(mut edit) = restored {
                let end = egui::text::CCursor::new(state.bible.search.chars().count());
                edit.cursor.set_char_range(Some(egui::text::CCursorRange::one(end)));
                edit.store(ctx, field_id);
            }
        } else if close {
            close_bible_search(state);
        } else if let Some(candidate) = autocomplete.filter(|candidate| *candidate != state.bible.search) {
            state.bible.search = candidate;
            state.bible.search_key.clear();
            state.bible.suggestion_index = 0;
            state.bible.search_focus = true;
            let restored = egui::TextEdit::load_state(ctx, field_id);
            if let Some(mut edit) = restored {
                let end = egui::text::CCursor::new(state.bible.search.chars().count());
                edit.cursor.set_char_range(Some(egui::text::CCursorRange::one(end)));
                edit.store(ctx, field_id);
            }
        }
    });
}
