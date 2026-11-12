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
fn new_services_use_language_defaults_but_saved_versions_are_preserved() {
    use crate::core::i18n::Language;

    for (language, expected) in [
        (Language::Portuguese, "ARA"),
        (Language::English, "KJV"),
        (Language::Spanish, "RVR1960"),
    ] {
        let mut state = ControlPanelState::default();
        state.settings.language = language;
        state.apply_profiles();
        assert_eq!(state.bible_version, expected);
        assert_eq!(state.bible_shortcuts, vec![expected]);
        assert!(state.bible.library.get(expected).is_some());

        state.settings.service_mut().bible_version = "NKJV".into();
        state.apply_profiles();
        assert_eq!(state.bible_version, "NKJV");
    }

    let lib = crate::core::bible::BibleLibrary::default();
    assert!(lib.get("EN_KJV").is_some());
    assert!(lib.get("KJV").is_some());
}

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
fn unpinned_active_bible_translation_does_not_return_on_render_or_profile_reload() {
    let mut state = ControlPanelState::default();
    let current = crate::core::bible::version_code(&state.bible_version);
    state.bible_shortcuts = vec![current.clone()];
    state.bible_shortcuts.retain(|id| id != &current);
    let ctx = egui::Context::default();
    for _ in 0..2 {
        let theme = state.theme.clone();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            super::version_bar::version_bar(ui, &mut state, &theme);
        });
        output.textures_delta.clear();
        assert!(state.bible_shortcuts.is_empty());
        assert_eq!(crate::core::bible::version_code(&state.bible_version), current);
    }
    state.capture_profiles();
    state.apply_profiles();
    assert!(state.bible_shortcuts.is_empty());
    assert_eq!(crate::core::bible::version_code(&state.bible_version), current);
}

#[test]
fn bible_summarized_reference_numbers_stay_outside_quotes() {
    use crate::core::bible::{BibleVerseNumberStyle, format_verse_number};
    for style in [BibleVerseNumberStyle::Bracket, BibleVerseNumberStyle::Superscript, BibleVerseNumberStyle::Dot] {
        for split in [false, true] {
            let mut state = ControlPanelState::default();
            state.bible_verse = 1;
            state.bible.verse_end = 3;
            let settings = &mut state.settings.bible_settings;
            settings.range_number_only = true;
            settings.quotes_around_verse = true;
            settings.verse_number_style = style;
            settings.auto_split_long_verses = split;
            settings.max_chars_per_slide = 40;
            let passages = selection_passages(&state);
            assert!(passages.len() >= 3);
            for passage in &passages {
                let visible = crate::core::slides::strip_directive_commands(&passage.content);
                assert!(!visible.starts_with('“'), "Number must precede the quote: {visible}");
                assert!(visible.contains(" “"), "Verse must still be quoted: {visible}");
                for verse in 1..=3 {
                    let number = format_verse_number(verse, style);
                    assert!(!visible.contains(&format!("“{number}")), "Number is inside quotes: {visible}");
                }
            }
            let first = crate::core::slides::strip_directive_commands(&passages[0].content);
            assert!(first.starts_with(&format!("{} “", format_verse_number(1, style))));
            assert!(!passages[0].content.contains("//ref-pos:"));
            assert!(passages.last().unwrap().content.contains("1:1–3"));
        }
    }
}

#[test]
fn bible_cards_layout_is_saved_and_restored_per_operator() {
    let mut state = ControlPanelState::default();
    let first = state.settings.active_operator.clone();
    let mut second = crate::core::preferences::OperatorProfile::default();
    second.id = "second_operator".into();
    state.settings.operators.push(second);
    super::navigation::toggle_layout(&mut state);
    assert!(state.bible.cards_layout);
    state.capture_profiles();
    let json = serde_json::to_string(&state.settings).unwrap();
    state.settings = serde_json::from_str(&json).unwrap();
    state.settings.active_operator = "second_operator".into();
    state.apply_profiles();
    assert!(!state.bible.cards_layout);
    state.settings.active_operator = first;
    state.apply_profiles();
    assert!(state.bible.cards_layout);
    super::navigation::toggle_layout(&mut state);
    state.capture_profiles();
    state.apply_profiles();
    assert!(!state.bible.cards_layout);
    let legacy: crate::core::preferences::OperatorProfile =
        serde_json::from_str(r#"{"id":"legacy"}"#).unwrap();
    assert!(!legacy.bible_cards_layout);
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

#[test]
fn test_pressing_enter_projects_selected_verse() {
    let mut state = ControlPanelState::default();
    let mut deck = SlideDeck::default();
    state.bible_verse = 1;
    state.bible.verse_end = 1;
    let mut presenting = false;

    let ctx = egui::Context::default();
    let mut raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1280.0, 1024.0),
        )),
        ..Default::default()
    };
    raw_input.events.push(egui::Event::Key {
        key: egui::Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });

    let mut output = ctx.run_ui(raw_input, |ui| {
        super::text::render_bible_text(ui, &mut state, &mut deck, &mut presenting);
    });
    output.textures_delta.clear();

    assert!(presenting, "Pressing enter must project the selected verse");
    assert!(!deck.projected_text().is_empty(), "Projected text should not be empty");
}

#[test]
fn test_bible_settings_reference_position_and_formatting() {
    use crate::core::bible::{
        BibleReferencePosition, BibleSeparator, BibleSettings, format_bible_slide,
    };

    let mut settings = BibleSettings::default();
    let sample = "No princípio criou Deus os céus e a terra.".to_string();

    // Default: Bottom position, no version tag, full name, colon separator
    let (reference, content) = format_bible_slide(
        &settings,
        "Gênesis",
        "Gn",
        "ARA",
        1,
        1,
        1,
        std::slice::from_ref(&sample),
    );
    assert_eq!(reference, "Gênesis 1:1");
    let alignment = "//valign: center\n//alinhar: centro\n";
    assert_eq!(content, format!("{alignment}//ref-pos: bottom\n//ref-align: right\n{sample}\nGênesis 1:1"));

    // Top position
    settings.reference_position = BibleReferencePosition::Top;
    let (_, content_top) = format_bible_slide(
        &settings,
        "Gênesis",
        "Gn",
        "ARA",
        1,
        1,
        1,
        std::slice::from_ref(&sample),
    );
    assert_eq!(content_top, format!("{alignment}//ref-pos: top\n//ref-align: right\nGênesis 1:1\n{sample}"));

    // Hidden position
    settings.reference_position = BibleReferencePosition::Hidden;
    let (_, content_hidden) = format_bible_slide(
        &settings,
        "Gênesis",
        "Gn",
        "ARA",
        1,
        1,
        1,
        std::slice::from_ref(&sample),
    );
    assert_eq!(content_hidden, format!("{alignment}{sample}"));

    // BelowVerse position
    settings.reference_position = BibleReferencePosition::BelowVerse;
    let (_, content_below) = format_bible_slide(
        &settings,
        "Gênesis",
        "Gn",
        "ARA",
        1,
        1,
        1,
        std::slice::from_ref(&sample),
    );
    assert_eq!(content_below, format!("{alignment}//ref-pos: below\n//ref-align: right\n{sample}\nGênesis 1:1"));

    // EndOfVerse position
    settings.reference_position = BibleReferencePosition::EndOfVerse;
    let (_, content_end) = format_bible_slide(
        &settings,
        "Gênesis",
        "Gn",
        "ARA",
        1,
        1,
        1,
        std::slice::from_ref(&sample),
    );
    assert_eq!(content_end, format!("{alignment}{sample} (Gênesis 1:1)"));

    // Abbreviation and Version Tag
    settings.reference_position = BibleReferencePosition::Bottom;
    settings.use_abbreviation = true;
    settings.show_version = true;
    settings.separator = BibleSeparator::Dot;
    let (ref_abbrev, content_abbrev) = format_bible_slide(
        &settings,
        "Gênesis",
        "Gn",
        "ARA",
        1,
        1,
        1,
        std::slice::from_ref(&sample),
    );
    assert_eq!(ref_abbrev, "Gn 1.1 (ARA)");
    assert_eq!(content_abbrev, format!("{alignment}//ref-pos: bottom\n//ref-align: right\n{sample}\nGn 1.1 (ARA)"));

    // Multi-verse numbering
    let v2 = "E a terra era sem forma e vazia.".to_string();
    let verses = vec![sample.clone(), v2.clone()];
    settings.show_verse_numbers = true;
    let (ref_multi, content_multi) = format_bible_slide(
        &settings,
        "Gênesis",
        "Gn",
        "ARA",
        1,
        1,
        2,
        &verses,
    );
    assert_eq!(ref_multi, "Gn 1.1–2 (ARA)");
    assert_eq!(content_multi, format!("{alignment}//ref-pos: bottom\n//ref-align: right\n[1] {sample}\n[2] {v2}\nGn 1.1–2 (ARA)"));
    // Quotes and Uppercase reference
    settings.quotes_around_verse = true;
    settings.uppercase_reference = true;
    settings.version_style = crate::core::bible::BibleVersionStyle::Hyphen;
    settings.verse_number_style = crate::core::bible::BibleVerseNumberStyle::Superscript;
    let (ref_stylized, content_stylized) = format_bible_slide(
        &settings,
        "Gênesis",
        "Gn",
        "ara",
        1,
        1,
        2,
        &verses,
    );
    assert_eq!(ref_stylized, "GN 1.1–2 - ARA");
    assert!(content_stylized.contains("GN 1.1–2 - ARA"));
    assert!(content_stylized.contains("//ref-pos:"));
    assert!(content_stylized.contains("“¹ No princípio"));
    assert!(content_stylized.contains("² E a terra"));

    settings.reference_size = crate::core::bible::BibleReferenceSize::Compact;
    let (_, content_compact) = format_bible_slide(
        &settings,
        "Gênesis",
        "Gn",
        "ara",
        1,
        1,
        2,
        &verses,
    );
    assert!(content_compact.contains("//ref-size: compact"));

    // Range number only (intermediate slides display verse number; last displays full range)
    settings.range_number_only = true;
    settings.verse_number_style = crate::core::bible::BibleVerseNumberStyle::Bracket;
    let num_16 = crate::core::bible::format_verse_number(16, settings.verse_number_style);
    assert_eq!(num_16, "[16]");
    let full_range = crate::core::bible::format_bible_reference(&settings, "João", "Jo", "NVI", 3, 16, 20);
    assert_eq!(full_range, "JO 3.16–20 - NVI");

    // Version codes with language prefix must be cleaned (EN_KJV -> KJV, ES_RVR1960 -> RVR1960)
    let en_ref = crate::core::bible::format_bible_reference(&settings, "João", "Jo", "EN_KJV", 3, 16, 16);
    assert_eq!(en_ref, "JO 3.16 - KJV");
    let es_ref = crate::core::bible::format_bible_reference(&settings, "João", "Jo", "ES_RVR1960", 3, 16, 16);
    assert_eq!(es_ref, "JO 3.16 - RVR1960");

    // Single verse never hides reference even if range_number_only is true
    let mut state = ControlPanelState::default();
    state.settings.bible_settings.range_number_only = true;
    state.bible_verse = 3;
    state.bible.verse_end = 3;
    let single_passages = super::selection::selection_passages(&state);
    assert_eq!(single_passages.len(), 1);
    assert!(single_passages[0].content.contains("1:3"));
    assert!(!single_passages[0].content.starts_with("[3]"));

    // Multi-verse with range_number_only: intermediate hides ref and shows [verse], last shows full range
    state.bible_verse = 1;
    state.bible.verse_end = 3;
    let multi_passages = super::selection::selection_passages(&state);
    assert_eq!(multi_passages.len(), 3);
    assert!(multi_passages[0].content.contains("[1]"));
    assert!(!multi_passages[0].content.contains("//ref-pos:")); // ref omitted
    assert!(multi_passages[1].content.contains("[2]"));
    assert!(!multi_passages[1].content.contains("//ref-pos:")); // ref omitted
    assert!(multi_passages[2].content.contains("[3]"));
    assert!(multi_passages[2].content.contains("//ref-pos:")); // full range on last slide
    assert!(multi_passages[2].content.contains("1:1–3"));
}

#[test]
fn test_bible_settings_modal_renders_without_panicking() {
    let mut state = ControlPanelState::default();
    state.bible.show_settings = true;
    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1280.0, 1024.0),
        )),
        ..Default::default()
    };
    let mut output = ctx.run_ui(raw_input, |_ui| {
        super::render_bible_settings_modal(&ctx, &mut state);
    });
    output.textures_delta.clear();
    assert!(state.bible.show_settings);
}

#[test]
fn test_bible_vertical_alignment_directives_and_parsing() {
    use crate::core::bible::{BibleSettings, BibleVerticalAlignment, format_bible_slide};
    use crate::core::markup::parse_slide_directives;

    let mut settings = BibleSettings::default();
    let sample = "No princípio criou Deus os céus e a terra.".to_string();

    // Center is explicit, so a theme cannot silently override Bible alignment.
    let (_, content_center) = format_bible_slide(&settings, "Gênesis", "Gn", "ARA", 1, 1, 1, std::slice::from_ref(&sample));
    assert!(content_center.starts_with("//valign: center\n"));
    let dirs_center = parse_slide_directives(&content_center);
    assert_eq!(dirs_center.valign.as_deref(), Some("center"));
    assert_eq!(dirs_center.align.as_deref(), Some("center"));

    // Top emits //valign: top
    settings.vertical_alignment = BibleVerticalAlignment::Top;
    let (_, content_top) = format_bible_slide(&settings, "Gênesis", "Gn", "ARA", 1, 1, 1, std::slice::from_ref(&sample));
    assert!(content_top.starts_with("//valign: top\n"));
    let dirs_top = parse_slide_directives(&content_top);
    assert_eq!(dirs_top.valign.as_deref(), Some("top"));

    // Bottom emits //valign: bottom
    settings.vertical_alignment = BibleVerticalAlignment::Bottom;
    let (_, content_bottom) = format_bible_slide(&settings, "Gênesis", "Gn", "ARA", 1, 1, 1, std::slice::from_ref(&sample));
    assert!(content_bottom.starts_with("//valign: bottom\n"));
    let dirs_bottom = parse_slide_directives(&content_bottom);
    assert_eq!(dirs_bottom.valign.as_deref(), Some("bottom"));

    // Portuguese aliases in markup
    let custom_pt = "//valinhar: terço inferior\nTexto";
    let dirs_pt = parse_slide_directives(custom_pt);
    assert_eq!(dirs_pt.valign.as_deref(), Some("bottom"));
}
