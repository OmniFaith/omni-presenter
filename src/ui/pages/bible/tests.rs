use super::navigation::toggle_layout;
use super::search::{QueryParse, number_is_complete, parse_query, split_reference};
use super::selection::{
    add_to_playlist, is_in_selection, pin_passage, project, select_reference, sync_preview,
};
use super::*;
use crate::core::models::PlaylistItemKind;
use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use eframe::egui;

#[test]
fn bible_search_accepts_leading_zeroes_in_chapters_and_verses() {
    for (query, digits, expected_search, selected) in [
        ("Atos 0", "2", "Atos 2:", None),
        ("Atos 2:0", "2", "", Some((2, 2))),
        ("Salmos 0", "19", "Salmos 19:", None),
        ("Salmos 0", "02", "Salmos 2:", None),
        ("Salmos 119:0", "19", "", Some((119, 19))),
        ("Ester 0", "2", "Ester 2:", None),
        ("Ester 10:0", "2", "", Some((10, 2))),
    ] {
        let mut state = ControlPanelState {
            active_sidebar_page: crate::core::state::SidebarPage::Bible,
            ..Default::default()
        };
        state.bible.show_search = true;
        state.bible.search_focus = true;
        state.bible.search = query.into();
        let mut deck = SlideDeck::default();
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 1024.0),
            )),
            ..Default::default()
        };
        for _ in 0..2 {
            let mut output = ctx.run_ui(input.clone(), |ui| {
                render_bible_search_overlay(ui.ctx(), &mut state, &mut deck)
            });
            output.textures_delta.clear();
        }
        assert_eq!(
            state.bible.search, query,
            "zero must wait for the next digit"
        );
        assert!(state.bible.show_search);

        let field_id = egui::Id::unique("bible_search_overlay").with("field");
        let mut edit = egui::TextEdit::load_state(&ctx, field_id).unwrap();
        let end = egui::text::CCursor::new(state.bible.search.chars().count());
        edit.cursor
            .set_char_range(Some(egui::text::CCursorRange::one(end)));
        edit.store(&ctx, field_id);
        for (index, digit) in digits.char_indices() {
            let mut typed = input.clone();
            typed.events.push(egui::Event::Text(digit.to_string()));
            let mut output = ctx.run_ui(typed, |ui| {
                render_bible_search_overlay(ui.ctx(), &mut state, &mut deck)
            });
            output.textures_delta.clear();
            let mut output = ctx.run_ui(input.clone(), |ui| {
                render_bible_search_overlay(ui.ctx(), &mut state, &mut deck)
            });
            output.textures_delta.clear();
            if index + 1 < digits.len() {
                assert!(state.bible.show_search);
                assert_eq!(
                    state.bible.search,
                    format!("{query}{}", &digits[..=index]),
                    "wait for all digit positions"
                );
            }
        }
        assert_eq!(state.bible.search, expected_search);
        if let Some((chapter, verse)) = selected {
            assert!(!state.bible.show_search);
            assert_eq!((state.bible_chapter, state.bible_verse), (chapter, verse));
            assert!(
                deck.preview_slide_text()
                    .ends_with(&format!("{} {chapter}:{verse}", state.bible_book))
            );
        } else {
            assert!(state.bible.show_search);
        }
    }
}

#[test]
fn searching_completes_book_chapter_and_verse() {
    let state = ControlPanelState::default();
    let bible = state.bible.library.get(&state.bible_version).unwrap();
    let name = |parsed: &QueryParse| parsed.book.map(|b| bible.books[b].name.clone());

    let parsed = parse_query(&bible, "gn 1");
    assert!(parsed.book_complete);
    assert_eq!(name(&parsed).as_deref(), Some("Gênesis"));
    assert_eq!(parsed.chapter, Some(1));

    let parsed = parse_query(&bible, "1 sa 3");
    assert_eq!(name(&parsed).as_deref(), Some("1 Samuel"));
    assert_eq!(parsed.chapter, Some(3));

    let parsed = parse_query(&bible, "ester 10:2");
    assert_eq!(name(&parsed).as_deref(), Some("Ester"));
    assert_eq!(
        (parsed.chapter, parsed.verse, parsed.has_separator),
        (Some(10), Some(2), true)
    );

    let parsed = parse_query(&bible, "jo");
    assert!(parsed.matches.len() >= 2);

    let parsed = parse_query(&bible, "3 jo");
    assert_eq!(name(&parsed).as_deref(), Some("3 João"));

    assert_eq!(
        split_reference("1 samuel 3:16"),
        ("1samuel".into(), "3:16".into())
    );
}

#[test]
fn number_completes_when_no_longer_a_prefix_of_a_valid_one() {
    let state = ControlPanelState::default();
    let bible = state.bible.library.get(&state.bible_version).unwrap();
    let salmos = bible
        .books
        .iter()
        .position(|book| book.name == "Salmos")
        .unwrap();
    let chapters = bible.books[salmos].chapters.len();
    assert_eq!(chapters, 150);

    assert!(!number_is_complete("0", chapters));
    assert!(!number_is_complete("00", chapters));
    assert!(!number_is_complete("2", chapters));
    assert!(!number_is_complete("01", chapters));
    assert!(!number_is_complete("02", chapters));
    assert!(number_is_complete("019", chapters));
    assert!(number_is_complete("002", chapters));
    assert!(number_is_complete("02", 28));
    assert!(!number_is_complete("1", chapters));
    assert!(!number_is_complete("15", chapters));
    assert!(number_is_complete("25", chapters));
    assert!(number_is_complete("119", chapters));
    assert!(number_is_complete("150", chapters));

    let psalms_119 = bible.books[salmos].chapters[118].len();
    assert_eq!(psalms_119, 176);
    assert!(!number_is_complete("01", psalms_119));
    assert!(number_is_complete("019", psalms_119));
    assert!(!number_is_complete("17", psalms_119));
    assert!(number_is_complete("105", psalms_119));
    assert!(number_is_complete("176", psalms_119));
}

#[test]
fn cards_layout_toggles_and_renders_without_panicking() {
    let mut state = ControlPanelState::default();
    assert!(!state.bible.cards_layout);
    toggle_layout(&mut state);
    assert!(state.bible.cards_layout);

    let mut deck = SlideDeck::default();
    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(260.0, 700.0),
        )),
        ..Default::default()
    };
    let mut output = ctx.run_ui(raw_input, |ui| {
        render_bible_navigation(ui, &mut state, &mut deck);
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty(), "cards layout rendered nothing");

    let mut rects = Vec::new();
    let mut presenting = false;
    let list_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(1000.0, 700.0),
        )),
        ..Default::default()
    };
    toggle_layout(&mut state);
    assert!(!state.bible.cards_layout);
    let mut list_output = ctx.run_ui(list_input.clone(), |ui| {
        render_bible_page(ui, &mut state, &mut deck, &mut presenting, &mut rects, None);
    });
    list_output.textures_delta.clear();
    assert!(
        !rects.is_empty(),
        "list layout still uses the docking containers"
    );
}

#[test]
fn preview_range_and_playlist_keep_live_content_independent() {
    let mut state = ControlPanelState::default();
    let mut deck = SlideDeck::default();
    let live = deck.projected_text().to_owned();
    let library_count = deck.library().len();
    sync_preview(&mut state, &mut deck, false);
    assert!(deck.preview_slide_text().ends_with("Gênesis 1:1"));
    assert_eq!(deck.projected_text(), live);
    state.bible.merge_selection = true;
    state.bible_verse = 1;
    state.bible.verse_end = 3;
    sync_preview(&mut state, &mut deck, true);
    assert_eq!(deck.slides().len(), 1);
    assert!(deck.preview_slide_text().ends_with("Gênesis 1:1–3"));
    add_to_playlist(&mut state);
    add_to_playlist(&mut state);
    assert_eq!(
        state
            .playlist
            .iter()
            .filter(|p| p.kind == PlaylistItemKind::Bible)
            .count(),
        1
    );
    let item = state.playlist.last().unwrap().clone();
    assert!(item.content.ends_with("Gênesis 1:1–3"));
    let mut presenting = false;
    project(&mut state, &mut deck, &mut presenting);
    assert!(presenting);
    assert_eq!(deck.projected_text(), item.content);
    assert_eq!(
        deck.library().len(),
        library_count,
        "Verses must not become fake songs"
    );
}

#[test]
fn shift_selection_without_merge_creates_one_slide_per_verse() {
    let mut state = ControlPanelState::default();
    let mut deck = SlideDeck::default();
    state.bible.merge_selection = false;
    state.bible_verse = 1;
    state.bible.verse_end = 3;
    sync_preview(&mut state, &mut deck, true);
    assert_eq!(
        deck.slides().len(),
        3,
        "each selected verse is its own slide"
    );
    for (offset, slide) in deck.slides().iter().enumerate() {
        assert!(slide.ends_with(&format!("Gênesis 1:{}", offset + 1)));
    }

    let playlist_before = state.playlist.len();
    let quick_before = state.quick_bar.items.len();
    add_to_playlist(&mut state);
    pin_passage(&mut state);
    assert_eq!(
        state.playlist.len(),
        playlist_before + 1,
        "the range is queued as one entry"
    );
    assert_eq!(
        state.quick_bar.items.len(),
        quick_before + 1,
        "the range is pinned as one shortcut"
    );
    assert_eq!(state.playlist.last().unwrap().title, "Gênesis 1:1–3");
    assert_eq!(state.quick_bar.items.last().unwrap().title, "Gênesis 1:1–3");

    let mut presenting = false;
    project(&mut state, &mut deck, &mut presenting);
    assert!(presenting);
    assert_eq!(deck.projected_text(), deck.slides()[0]);
}

#[test]
fn saved_bible_ranges_preserve_slide_boundaries_for_playlist_and_shortcuts() {
    for (end, merged, count) in [(7, false, 7), (7, true, 1), (1, false, 1)] {
        let mut state = ControlPanelState::default();
        let bible = state.bible.library.get(&state.bible_version).unwrap();
        let book = bible
            .books
            .iter()
            .position(|book| book.name == "Salmos")
            .unwrap();
        select_reference(&mut state, book, 119, 1, end);
        state.bible.merge_selection = merged;
        let reference = if end == 1 {
            "Salmos 119:1"
        } else {
            "Salmos 119:1–7"
        };

        let playlist_before = state.playlist.len();
        let quick_before = state.quick_bar.items.len();
        add_to_playlist(&mut state);
        pin_passage(&mut state);
        assert_eq!(state.playlist.len(), playlist_before + 1);
        assert_eq!(state.quick_bar.items.len(), quick_before + 1);
        let item = state.playlist.last().unwrap();
        let quick = state.quick_bar.items.last().unwrap();
        assert_eq!(item.title, reference);
        assert_eq!(quick.title, reference);
        assert_eq!(item.content, quick.content);

        // Use the same loading paths as the playlist and shortcut buttons.
        let mut deck = SlideDeck::default();
        item.preview(&mut deck).unwrap();
        deck.project_preview();
        assert_eq!(deck.total_live_slides(), count);
        let playlist_slides = deck.live_slides().to_vec();
        deck.project_custom_content(
            quick.title.clone(),
            quick.subtitle.clone(),
            quick.content.clone(),
        );
        assert_eq!(deck.live_slides(), playlist_slides);
        if merged {
            assert!(deck.live_slides()[0].ends_with(reference));
        } else {
            for (index, slide) in deck.live_slides().iter().enumerate() {
                assert!(slide.ends_with(&format!("Salmos 119:{}", index + 1)));
            }
        }
    }
}

#[test]
fn verse_inside_selection_keeps_every_selected_verse() {
    let mut state = ControlPanelState {
        bible_verse: 1,
        ..Default::default()
    };
    state.bible.verse_end = 3;
    state.bible.selection_anchor = 1;
    assert!(is_in_selection(&state, 1));
    assert!(is_in_selection(&state, 2));
    assert!(is_in_selection(&state, 3));
    assert!(!is_in_selection(&state, 4));

    let before = state.playlist.len();
    add_to_playlist(&mut state);
    assert_eq!(
        state.playlist.len(),
        before + 1,
        "context action queues the selection as one entry"
    );
    assert_eq!(state.playlist.last().unwrap().title, "Gênesis 1:1–3");
}

#[test]
fn test_last_searched_bible_verse_persists_across_profiles() {
    let mut state = ControlPanelState::default();
    assert_eq!(state.bible_book, "Gênesis");
    assert_eq!(state.bible_chapter, 1);
    assert_eq!(state.bible_verse, 1);

    let bible = state
        .bible
        .library
        .get(crate::core::bible::DEFAULT_BIBLE)
        .unwrap();
    let (book, ch, v, _) = bible.reference("jo 3:16").unwrap();
    select_reference(&mut state, book, ch, v, v);
    assert_eq!(state.bible_book, "João");
    assert_eq!(state.bible_chapter, 3);
    assert_eq!(state.bible_verse, 16);

    state.capture_profiles();
    let mut restored = ControlPanelState {
        settings: state.settings.clone(),
        ..Default::default()
    };
    restored.apply_profiles();
    assert_eq!(restored.bible_book, "João");
    assert_eq!(restored.bible_chapter, 3);
    assert_eq!(restored.bible_verse, 16);
}

#[test]
fn test_floating_bible_search_overlay_renders_from_any_page() {
    use crate::core::commands::Command;
    let mut state = ControlPanelState {
        active_sidebar_page: crate::core::state::SidebarPage::Home,
        ..Default::default()
    };
    state.bible.show_search = true;
    state.bible.search_focus = true;
    state.bible.search = "Sl 23:1".into();

    let mut deck = SlideDeck::default();
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1280.0, 1024.0),
        )),
        ..Default::default()
    };
    let mut output = ctx.run_ui(input, |ui| {
        render_bible_search_overlay(ui.ctx(), &mut state, &mut deck);
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty(), "floating search overlay rendered shapes");
    assert_eq!(Command::BibleSearch.label_key(), "command-bible-search");
}

