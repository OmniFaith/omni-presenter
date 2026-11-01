use crate::core::bible::{BiblePassage, BibleTranslation, version_code};
use crate::core::models::{PlaylistItem, PlaylistItemKind, QuickItemKind};
use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use crate::ui::components::{centered_horizontal, toolbar_icon_button};
use eframe::egui;
use egui_phosphor::regular::{HEART, LIST_PLUS, PLAY, STACK};

use super::helpers::current_book;

pub fn selected_passage(state: &ControlPanelState) -> Option<BiblePassage> {
    let bible = state.bible.library.get(&state.bible_version)?;
    bible.passage(
        current_book(state, &bible),
        state.bible_chapter,
        state.bible_verse,
        state.bible.verse_end.max(state.bible_verse),
    )
}

/// Every passage the current selection represents.
///
/// With "juntar em um slide" enabled (or a single verse selected) this is one passage with the whole
/// range; otherwise each selected verse becomes its own slide. The repertory and shortcuts keep
/// these slides together in a single entry.
pub fn selection_passages(state: &ControlPanelState) -> Vec<BiblePassage> {
    let Some(bible) = state.bible.library.get(&state.bible_version) else {
        return Vec::new();
    };
    let book = current_book(state, &bible);
    let start = state.bible_verse.min(state.bible.verse_end).max(1);
    let end = state.bible_verse.max(state.bible.verse_end);
    if state.bible.merge_selection || start == end {
        bible
            .passage(book, state.bible_chapter, start, end)
            .into_iter()
            .collect()
    } else {
        (start..=end)
            .filter_map(|verse| bible.passage(book, state.bible_chapter, verse, verse))
            .collect()
    }
}

pub(super) fn is_in_selection(state: &ControlPanelState, number: usize) -> bool {
    number >= state.bible_verse && number <= state.bible.verse_end
}

/// The reference, version and raw text stored for the repertory/shortcuts.
///
/// A multi-verse range stays a single entry (title = the whole reference) but its text keeps one
/// verse per slide unless "juntar em um slide" is enabled.
fn selection_entry(state: &ControlPanelState) -> Option<(String, String, String)> {
    let passage = selected_passage(state)?;
    let content = selection_passages(state)
        .into_iter()
        .map(|part| part.content)
        .collect::<Vec<_>>()
        .join("\n\n");
    Some((passage.reference, passage.version, content))
}

/// Builds one slide per verse in `start..=end`, skipping empty source verses.
fn chapter_slides(
    state: &ControlPanelState,
    bible: &BibleTranslation,
    book: usize,
    chapter: usize,
    start: usize,
    end: usize,
) -> Vec<(usize, String)> {
    let Some(verses) = bible.chapter(book, chapter) else {
        return Vec::new();
    };
    (start..=end)
        .filter(|&verse| verse >= 1 && verse <= verses.len() && !verses[verse - 1].is_empty())
        .map(|verse| {
            (
                verse,
                format!(
                    "{}\n{} {}:{}",
                    verses[verse - 1],
                    state.bible_book,
                    chapter,
                    verse
                ),
            )
        })
        .collect()
}

pub fn sync_preview(state: &mut ControlPanelState, deck: &mut SlideDeck, force: bool) {
    let code = version_code(&state.bible_version);
    let Some(bible) = state.bible.library.get(&code) else {
        let key = format!("missing:{code}");
        if state.bible.preview_key != key {
            deck.load_custom_content(
                state.settings.language.format("workspace-version-label", &[("code", &code)]),
                code,
                String::new(),
            );
            state.bible.preview_key = key;
        }
        return;
    };
    let book = current_book(state, &bible);
    state.bible_book = bible.books[book].name.clone();
    state.bible_chapter = state
        .bible_chapter
        .clamp(1, bible.books[book].chapters.len());
    let chapter = bible.chapter(book, state.bible_chapter).unwrap();
    state.bible_verse = state.bible_verse.clamp(1, chapter.len());
    state.bible.verse_end = state
        .bible
        .verse_end
        .clamp(state.bible_verse, chapter.len());
    let key = format!(
        "{code}:{book}:{}:{}:{}:{}",
        state.bible_chapter,
        state.bible_verse,
        state.bible.verse_end,
        state.bible.merge_selection as u8
    );
    if !force && state.bible.preview_key == key {
        return;
    }
    if state.bible_verse == state.bible.verse_end {
        // A whole chapter is available to the live controls, one verse per slide.
        // Missing source verses are skipped without changing their displayed references.
        let slides = chapter_slides(state, &bible, book, state.bible_chapter, 1, chapter.len());
        let selected = slides
            .iter()
            .position(|(number, _)| *number == state.bible_verse);
        if let Some(index) = selected {
            deck.load_custom_content(
                format!("{} {}", state.bible_book, state.bible_chapter),
                code,
                slides
                    .iter()
                    .map(|(_, text)| text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n\n"),
            );
            deck.select_preview_slide(index);
        } else {
            deck.load_custom_content(String::new(), code, String::new());
        }
    } else if state.bible.merge_selection {
        if let Some(passage) = selected_passage(state) {
            deck.load_custom_content(passage.reference, passage.version, passage.content);
        } else {
            deck.load_custom_content(String::new(), code, String::new());
        }
    } else {
        // Range without merge: one slide per selected verse.
        let start = state.bible_verse.min(state.bible.verse_end).max(1);
        let end = state.bible_verse.max(state.bible.verse_end);
        let slides = chapter_slides(state, &bible, book, state.bible_chapter, start, end);
        if slides.is_empty() {
            deck.load_custom_content(String::new(), code, String::new());
        } else {
            deck.load_custom_content(
                format!("{} {}", state.bible_book, state.bible_chapter),
                code,
                slides
                    .iter()
                    .map(|(_, text)| text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n\n"),
            );
            deck.select_preview_slide(0);
        }
    }
    state.bible.preview_key = key;
}

pub(super) fn select_reference(
    state: &mut ControlPanelState,
    book: usize,
    chapter: usize,
    start: usize,
    end: usize,
) {
    state.bible_book = crate::core::bible::BOOK_NAMES[book].into();
    state.bible_chapter = chapter;
    state.bible_verse = start;
    state.bible.verse_end = end;
    state.bible.selection_anchor = start;
    state.bible.expanded_book = Some(book);
    state.bible.scroll_to_book = true;
    state.bible.scroll_to_verse = true;
    state.bible.preview_key.clear();
    state.capture_profiles();
}

pub(super) fn selection_toolbar(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    presenting: &mut bool,
    theme: &crate::ui::theme::AppTheme,
) {
    let available = !selection_passages(state).is_empty();
    centered_horizontal(ui, "bible_text_toolbar", 136.0, |ui| {
        ui.add_enabled_ui(available, |ui| {
            if toolbar_icon_button(ui, PLAY, crate::ui::i18n::text(ui, "workspace-present-selected-passage"), theme).clicked() {
                project(state, deck, presenting);
            }
        });
        let merged = state.bible.merge_selection;
        let merge = state.theme.icon_button(
            ui,
            STACK,
            crate::ui::i18n::text(ui, "workspace-combine-multiple-verses-in-one-slide"),
            merged,
            egui::vec2(28.0, 28.0),
        );
        if merge.clicked() {
            state.bible.merge_selection = !merged;
            state.bible.preview_key.clear();
            sync_preview(state, deck, true);
        }
        ui.add_enabled_ui(available, |ui| {
            if toolbar_icon_button(ui, HEART, crate::ui::i18n::text(ui, "workspace-pin-selection-to-quick-bar"), theme).clicked() {
                pin_passage(state);
            }
            if toolbar_icon_button(
                ui,
                LIST_PLUS,
                crate::ui::i18n::text(ui, "workspace-add-selection-to-playlist"),
                theme,
            )
            .clicked()
            {
                add_to_playlist(state);
            }
        });
    });
}

pub(super) fn add_to_playlist(state: &mut ControlPanelState) {
    let Some((reference, version, content)) = selection_entry(state) else {
        return;
    };
    let id = format!("bible:{version}:{reference}");
    if state.playlist.iter().any(|item| item.id == id) {
        state.notify(state.settings.language.text("workspace-the-selection-is-already-in-the-playlist"));
        return;
    }
    state.playlist.push(PlaylistItem {
        id,
        title: reference.clone(),
        subtitle: version,
        kind: PlaylistItemKind::Bible,
        played: false,
        content,
    });
    state.selected_playlist_index = Some(state.playlist.len() - 1);
    state.notify(state.settings.language.format("workspace-added-to-playlist", &[("reference", &reference)]));
}

pub(super) fn pin_passage(state: &mut ControlPanelState) {
    let Some((reference, version, content)) = selection_entry(state) else {
        return;
    };
    state.add_quick_item(reference.clone(), version, QuickItemKind::Verse, content);
    state.notify(state.settings.language.format("workspace-pinned-to-quick-bar", &[("reference", &reference)]));
}

pub(super) fn project(state: &mut ControlPanelState, deck: &mut SlideDeck, presenting: &mut bool) {
    if selection_passages(state).is_empty() {
        return;
    }
    sync_preview(state, deck, true);
    deck.project_preview();
    state.apply_theme_to_live();
    state.show_presentation_dock = state.settings.operator().show_dock;
    *presenting = true;
}
