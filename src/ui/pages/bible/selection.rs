use crate::core::bible::{
    BiblePassage, BibleTranslation, format_bible_reference, format_bible_slide,
    format_bible_slide_with_custom_ref, format_verse_number, version_code,
};
use crate::core::models::{PlaylistItem, PlaylistItemKind, QuickItemKind};
use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use crate::ui::components::{centered_horizontal, toolbar_icon_button};
use eframe::egui;
use egui_phosphor::regular::{HEART, LIST_PLUS, PLAY, STACK};

use super::helpers::current_book;

pub fn selected_passage(state: &ControlPanelState) -> Option<BiblePassage> {
    let bible = state.bible.library.get(&state.bible_version)?;
    let book = current_book(state, &bible);
    let start = state.bible_verse;
    let end = state.bible.verse_end.max(state.bible_verse);
    if start == 0 || end < start {
        return None;
    }
    let verses = bible.chapter(book, state.bible_chapter)?.get(start - 1..end)?;
    if verses.iter().any(|v| v.is_empty()) {
        return None;
    }
    let (reference, content) = format_bible_slide(
        &state.settings.bible_settings,
        &bible.books[book].name,
        &bible.books[book].abbrev,
        &bible.id,
        state.bible_chapter,
        start,
        end,
        verses,
    );
    Some(BiblePassage {
        reference,
        version: bible.id.clone(),
        content,
    })
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
    let Some(chapter_verses) = bible.chapter(book, state.bible_chapter) else {
        return Vec::new();
    };
    if (state.bible.merge_selection && !state.settings.bible_settings.one_verse_per_slide)
        || start == end
    {
        let Some(verses) = chapter_verses.get(start - 1..end) else {
            return Vec::new();
        };
        if verses.iter().any(|v| v.is_empty()) {
            return Vec::new();
        }

        if start == end
            && state.settings.bible_settings.auto_split_long_verses
            && state.settings.bible_settings.split_action
                == crate::core::bible::BibleSplitAction::MultipleSlides
            && verses[0].chars().count() > state.settings.bible_settings.max_chars_per_slide
        {
            let chunks = crate::core::bible::split_verse_text(
                &verses[0],
                state.settings.bible_settings.max_chars_per_slide,
                state.settings.bible_settings.split_punctuation
                    == crate::core::bible::BibleSplitPunctuation::NaturalPunctuation,
                state.settings.bible_settings.split_ellipsis,
            );
            let total = chunks.len();
            chunks
                .into_iter()
                .enumerate()
                .map(|(i, chunk)| {
                    let (ref_str, content) = crate::core::bible::format_bible_slide(
                        &state.settings.bible_settings,
                        &bible.books[book].name,
                        &bible.books[book].abbrev,
                        &bible.id,
                        state.bible_chapter,
                        start,
                        end,
                        std::slice::from_ref(&chunk),
                    );
                    let reference = if total > 1 {
                        format!("{ref_str} ({}/{})", i + 1, total)
                    } else {
                        ref_str
                    };
                    BiblePassage {
                        reference,
                        version: bible.id.clone(),
                        content,
                    }
                })
                .collect()
        } else {
            let (reference, content) = crate::core::bible::format_bible_slide(
                &state.settings.bible_settings,
                &bible.books[book].name,
                &bible.books[book].abbrev,
                &bible.id,
                state.bible_chapter,
                start,
                end,
                verses,
            );
            vec![BiblePassage {
                reference,
                version: bible.id.clone(),
                content,
            }]
        }
    } else {
        let mut passages = Vec::new();
        let is_multi = state.bible_verse != state.bible.verse_end && start < end;
        let full_range_ref = if is_multi && state.settings.bible_settings.range_number_only {
            Some(format_bible_reference(
                &state.settings.bible_settings,
                &bible.books[book].name,
                &bible.books[book].abbrev,
                &bible.id,
                state.bible_chapter,
                start,
                end,
            ))
        } else {
            None
        };

        for verse in start..=end {
            let Some(v) = chapter_verses.get(verse - 1) else { continue; };
            if v.is_empty() { continue; }

            let is_last_verse = verse == end;
            let (display_ref, verse_body) = if let Some(ref full_ref) = full_range_ref {
                let num_str = format_verse_number(verse, state.settings.bible_settings.verse_number_style);
                let text_with_num = format!("{num_str} {v}");
                if is_last_verse {
                    (full_ref.clone(), text_with_num)
                } else {
                    (String::new(), text_with_num)
                }
            } else {
                let r = format_bible_reference(
                    &state.settings.bible_settings,
                    &bible.books[book].name,
                    &bible.books[book].abbrev,
                    &bible.id,
                    state.bible_chapter,
                    verse,
                    verse,
                );
                (r, v.clone())
            };

            if state.settings.bible_settings.auto_split_long_verses
                && state.settings.bible_settings.split_action
                    == crate::core::bible::BibleSplitAction::MultipleSlides
                && verse_body.chars().count() > state.settings.bible_settings.max_chars_per_slide
            {
                let chunks = crate::core::bible::split_verse_text(
                    &verse_body,
                    state.settings.bible_settings.max_chars_per_slide,
                    state.settings.bible_settings.split_punctuation
                        == crate::core::bible::BibleSplitPunctuation::NaturalPunctuation,
                    state.settings.bible_settings.split_ellipsis,
                );
                let total = chunks.len();
                for (chunk_idx, chunk) in chunks.into_iter().enumerate() {
                    let slide_ref = if !display_ref.is_empty() && total > 1 {
                        format!("{display_ref} ({}/{})", chunk_idx + 1, total)
                    } else {
                        display_ref.clone()
                    };
                    let (_, content) = format_bible_slide_with_custom_ref(
                        &state.settings.bible_settings,
                        slide_ref.clone(),
                        verse,
                        std::slice::from_ref(&chunk),
                    );
                    passages.push(BiblePassage {
                        reference: if slide_ref.is_empty() { format_verse_number(verse, state.settings.bible_settings.verse_number_style) } else { slide_ref },
                        version: bible.id.clone(),
                        content,
                    });
                }
            } else {
                let (_, content) = format_bible_slide_with_custom_ref(
                    &state.settings.bible_settings,
                    display_ref.clone(),
                    verse,
                    std::slice::from_ref(&verse_body),
                );
                passages.push(BiblePassage {
                    reference: if display_ref.is_empty() { format_verse_number(verse, state.settings.bible_settings.verse_number_style) } else { display_ref },
                    version: bible.id.clone(),
                    content,
                });
            }
        }
        passages
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
    let is_multi = state.bible_verse != state.bible.verse_end && start < end;
    let full_range_ref = if is_multi && state.settings.bible_settings.range_number_only {
        Some(format_bible_reference(
            &state.settings.bible_settings,
            &bible.books[book].name,
            &bible.books[book].abbrev,
            &bible.id,
            chapter,
            start,
            end,
        ))
    } else {
        None
    };

    (start..=end)
        .filter(|&verse| verse >= 1 && verse <= verses.len() && !verses[verse - 1].is_empty())
        .map(|verse| {
            let is_last_verse = verse == end;
            let (display_ref, verse_body) = if let Some(ref full_ref) = full_range_ref {
                let num_str = format_verse_number(verse, state.settings.bible_settings.verse_number_style);
                let text_with_num = format!("{num_str} {}", &verses[verse - 1]);
                if is_last_verse {
                    (full_ref.clone(), text_with_num)
                } else {
                    (String::new(), text_with_num)
                }
            } else {
                let r = format_bible_reference(
                    &state.settings.bible_settings,
                    &bible.books[book].name,
                    &bible.books[book].abbrev,
                    &bible.id,
                    chapter,
                    verse,
                    verse,
                );
                (r, verses[verse - 1].clone())
            };

            let (_, content) = format_bible_slide_with_custom_ref(
                &state.settings.bible_settings,
                display_ref,
                verse,
                std::slice::from_ref(&verse_body),
            );
            (verse, content)
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
        "{code}:{book}:{}:{}:{}:{}:{:?}:{:?}:{:?}:{:?}:{:?}:{}:{}:{}:{}:{}",
        state.bible_chapter,
        state.bible_verse,
        state.bible.verse_end,
        state.bible.merge_selection as u8,
        state.settings.bible_settings.reference_position,
        state.settings.bible_settings.reference_alignment,
        state.settings.bible_settings.alignment,
        state.settings.bible_settings.vertical_alignment,
        state.settings.bible_settings.verse_number_style,
        state.settings.bible_settings.quotes_around_verse,
        state.settings.bible_settings.use_abbreviation,
        state.settings.bible_settings.show_version,
        state.settings.bible_settings.range_number_only as u8,
        state.settings.bible_settings.reference_size as u8,
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
    let bible = state.bible.library.get(&state.bible_version);
    if let Some(b) = bible.as_ref().and_then(|lib| lib.books.get(book)) {
        state.bible_book = b.name.clone();
    } else {
        state.bible_book = crate::core::bible::book_names_for_language(state.settings.language)[book].into();
    }
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
