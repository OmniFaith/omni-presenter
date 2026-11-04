use super::*;

#[test]
fn test_selection_column_renders() {
    let mut state = ControlPanelState::default();
    let mut deck = SlideDeck::default();
    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(280.0, 700.0),
        )),
        ..Default::default()
    };
    let mut output = ctx.run_ui(raw_input, |ui| {
        render_selection_column(ui, &mut state, &mut deck, &mut false, None);
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());
}

#[test]
fn test_multiselect_state_and_batch_actions() {
    let mut state = ControlPanelState::default();
    assert!(state.selected_song_ids.is_empty());
    state.selected_song_ids.extend([1, 2, 3]);
    assert_eq!(state.selected_song_ids.len(), 3);
    assert!(state.is_song_selected(2));
    assert!(!state.is_song_selected(4));
    state.clear_song_selection();
    assert!(state.selected_song_ids.is_empty());
}

#[test]
fn test_multiselect_move_and_delete_in_db() {
    let db = AppDb::open_in_memory().unwrap();
    let initial_count = db.get_all_songs().unwrap().len();
    let id1 = db.add_song("Song 1", "Artist 1", "Verse 1").unwrap();
    let id2 = db.add_song("Song 2", "Artist 2", "Verse 2").unwrap();
    let id3 = db.add_song("Song 3", "Artist 3", "Verse 3").unwrap();
    for id in [id1, id2] {
        db.move_song_to_folder(id, "Specials").unwrap();
    }
    let songs = db.get_all_songs().unwrap();
    assert_eq!(
        songs.iter().find(|s| s.id == Some(id1)).unwrap().folder,
        "Specials"
    );
    assert_eq!(
        songs.iter().find(|s| s.id == Some(id2)).unwrap().folder,
        "Specials"
    );
    assert_eq!(songs.iter().find(|s| s.id == Some(id3)).unwrap().folder, "");
    for id in [id1, id2] {
        db.delete_song(id).unwrap();
    }
    let remaining = db.get_all_songs().unwrap();
    assert_eq!(remaining.len(), initial_count + 1);
    assert!(remaining.iter().any(|s| s.id == Some(id3)));
    assert!(!remaining.iter().any(|s| s.id == Some(id1)));
    assert!(!remaining.iter().any(|s| s.id == Some(id2)));
}

#[test]
fn test_render_all_tabs() {
    let mut state = ControlPanelState::default();
    let mut deck = SlideDeck::default();
    let db = AppDb::open_in_memory().unwrap();
    state.reload_all_selection_items(&db);
    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(280.0, 700.0),
        )),
        ..Default::default()
    };
    for tab in 0..=4 {
        state.selected_tab = tab;
        let mut output = ctx.run_ui(raw_input.clone(), |ui| {
            render_selection_column(ui, &mut state, &mut deck, &mut false, Some(&db));
        });
        output.textures_delta.clear();
        assert!(!output.shapes.is_empty(), "Tab {tab} produced no shapes");
    }
}

#[test]
fn test_media_crud_and_folders_in_db() {
    let db = AppDb::open_in_memory().unwrap();
    db.add_folder_to_category("media", "Backgrounds").unwrap();
    assert!(
        db.get_folders_by_category("media")
            .unwrap()
            .contains(&"Backgrounds".into())
    );
    let id = db
        .add_media_item("Cross background", "/path/to/cross.jpg", "image", "General")
        .unwrap();
    let items = db.get_all_media_items().unwrap();
    let found = items.iter().find(|item| item.id == Some(id)).unwrap();
    assert_eq!(found.title, "Cross background");
    assert_eq!(found.folder, "General");
    db.move_media_to_folder(id, "Backgrounds").unwrap();
    assert_eq!(
        db.get_all_media_items()
            .unwrap()
            .iter()
            .find(|item| item.id == Some(id))
            .unwrap()
            .folder,
        "Backgrounds"
    );
    db.delete_media_item(id).unwrap();
    assert!(
        !db.get_all_media_items()
            .unwrap()
            .iter()
            .any(|item| item.id == Some(id))
    );
}

#[test]
fn asynchronous_media_import_preserves_the_destination_and_refreshes_the_library() {
    let db = AppDb::open_in_memory().unwrap();
    let mut state = ControlPanelState::default();
    let ctx = egui::Context::default();
    state.media_import_task = Some((
        "Fundos".into(),
        crate::services::tasks::BackgroundTask::spawn(|| {
            Ok(vec![std::path::PathBuf::from(
                "example.jpg",
            )])
        }),
    ));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while state.media_import_task.is_some() {
        super::poll_media_import(&ctx, &mut state, Some(&db));
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(state.media_items.len(), 1);
    assert_eq!(state.media_items[0].folder, "Fundos");
    assert_eq!(state.media_items[0].file_type, "image");
    assert!(state.open_media_folders.contains("Fundos"));
    assert_eq!(
        db.get_all_media_items().unwrap()[0].file_path,
        "example.jpg"
    );
}

#[test]
fn conversion_preference_is_enabled_for_old_settings_and_can_be_saved() {
    let old: crate::core::preferences::AppSettings = serde_json::from_str("{}").unwrap();
    assert!(old.ask_media_conversion);
    let mut settings = old;
    settings.ask_media_conversion = false;
    let restored: crate::core::preferences::AppSettings = serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
    assert!(!restored.ask_media_conversion);
}

#[test]
fn test_custom_text_crud_and_folders_in_db() {
    let db = AppDb::open_in_memory().unwrap();
    db.add_folder_to_category("text", "Liturgy").unwrap();
    assert!(
        db.get_folders_by_category("text")
            .unwrap()
            .contains(&"Liturgy".into())
    );
    let id = db
        .add_custom_text("Prayer", "Prayer meeting at 7", "General")
        .unwrap();
    assert_eq!(
        db.get_all_custom_texts()
            .unwrap()
            .iter()
            .find(|item| item.id == Some(id))
            .unwrap()
            .title,
        "Prayer"
    );
    db.update_custom_text(id, "Tuesday prayer", "Tuesday at 7", "Liturgy")
        .unwrap();
    let texts = db.get_all_custom_texts().unwrap();
    let updated = texts.iter().find(|item| item.id == Some(id)).unwrap();
    assert_eq!(updated.title, "Tuesday prayer");
    assert_eq!(updated.folder, "Liturgy");
    db.delete_custom_text(id).unwrap();
    assert!(
        !db.get_all_custom_texts()
            .unwrap()
            .iter()
            .any(|item| item.id == Some(id))
    );
}

#[test]
fn test_saved_verse_crud_and_folders_in_db() {
    let db = AppDb::open_in_memory().unwrap();
    db.add_folder_to_category("bible", "Favorites").unwrap();
    assert!(
        db.get_folders_by_category("bible")
            .unwrap()
            .contains(&"Favorites".into())
    );
    let id = db
        .add_saved_verse("Genesis 1:1", "In the beginning…", "NVI", "General")
        .unwrap();
    assert_eq!(
        db.get_all_saved_verses()
            .unwrap()
            .iter()
            .find(|item| item.id == Some(id))
            .unwrap()
            .reference,
        "Genesis 1:1"
    );
    db.move_verse_to_folder(id, "Favorites").unwrap();
    assert_eq!(
        db.get_all_saved_verses()
            .unwrap()
            .iter()
            .find(|item| item.id == Some(id))
            .unwrap()
            .folder,
        "Favorites"
    );
    db.delete_saved_verse(id).unwrap();
    assert!(
        !db.get_all_saved_verses()
            .unwrap()
            .iter()
            .any(|item| item.id == Some(id))
    );
}

#[test]
fn test_multiselect_state_for_media_text_verse() {
    let mut state = ControlPanelState::default();
    state.selected_media_ids.extend([10, 20]);
    assert!(state.is_media_selected(10));
    assert!(!state.is_media_selected(30));
    state.clear_media_selection();
    assert!(state.selected_media_ids.is_empty());
    state.selected_text_ids.insert(100);
    assert!(state.is_text_selected(100));
    state.clear_text_selection();
    assert!(state.selected_text_ids.is_empty());
    state.selected_verse_ids.insert(200);
    assert!(state.is_verse_selected(200));
    state.clear_verse_selection();
    assert!(state.selected_verse_ids.is_empty());
    state.selected_song_ids.insert(1);
    state.selected_media_ids.insert(2);
    state.selected_text_ids.insert(3);
    state.selected_verse_ids.insert(4);
    state.clear_all_selections();
    assert!(state.selected_song_ids.is_empty());
    assert!(state.selected_media_ids.is_empty());
    assert!(state.selected_text_ids.is_empty());
    assert!(state.selected_verse_ids.is_empty());
}

#[test]
fn test_media_import_action_and_persistence() {
    let db = AppDb::open_in_memory().unwrap();
    let mut state = ControlPanelState::default();
    db.add_media_item("Root image", "/path/to/img.png", "image", "")
        .unwrap();
    db.add_media_item(
        "Background video",
        "/path/to/bg.mp4",
        "video",
        "Backgrounds",
    )
    .unwrap();
    state.reload_media_items(&db);
    assert_eq!(state.media_items.len(), 2);
    assert_eq!(
        state
            .media_items
            .iter()
            .find(|item| item.title == "Root image")
            .unwrap()
            .folder,
        ""
    );
    assert_eq!(
        state
            .media_items
            .iter()
            .find(|item| item.title == "Background video")
            .unwrap()
            .folder,
        "Backgrounds"
    );
}

#[test]
fn range_selection_uses_visible_order_and_preserves_additive_selection() {
    let mut state = ControlPanelState::default();
    model::select_range(
        LibraryKind::Song,
        &mut state,
        4,
        &[4, 2, 8, 1],
        false,
        false,
    );
    model::select_range(LibraryKind::Song, &mut state, 8, &[4, 2, 8, 1], true, false);
    assert_eq!(state.selected_song_ids, [4, 2, 8].into_iter().collect());
    model::select_range(LibraryKind::Song, &mut state, 1, &[4, 2, 8, 1], false, true);
    assert_eq!(state.selected_song_ids.len(), 4);
}

#[test]
fn batch_actions_keep_pins_and_playlist_separate_and_persist_moves() {
    let db = AppDb::open_in_memory().unwrap();
    let first = db.add_custom_text("First", "First slide", "").unwrap();
    let second = db.add_custom_text("Second", "Second slide", "").unwrap();
    let mut state = ControlPanelState::default();
    let mut deck = SlideDeck::default();
    state.reload_custom_texts(&db);
    let entries = model::entries(LibraryKind::Text, &state, &deck);
    actions::execute(
        LibraryAction::Pin(vec![first, second]),
        LibraryKind::Text,
        &entries,
        &mut state,
        &mut deck,
        &mut false,
        Some(&db),
    );
    assert_eq!(state.quick_bar.items.len(), 2);
    assert!(state.playlist.is_empty());
    actions::execute(
        LibraryAction::Move(vec![first, second], "Slides".into()),
        LibraryKind::Text,
        &entries,
        &mut state,
        &mut deck,
        &mut false,
        Some(&db),
    );
    assert!(
        db.get_all_custom_texts()
            .unwrap()
            .iter()
            .all(|item| item.folder == "Slides")
    );
    actions::execute(
        LibraryAction::Delete(vec![first, second]),
        LibraryKind::Text,
        &entries,
        &mut state,
        &mut deck,
        &mut false,
        Some(&db),
    );
    assert!(db.get_all_custom_texts().unwrap().is_empty());
}

#[test]
fn test_web_lyrics_button_in_toolbar() {
    let mut state = ControlPanelState::default();
    let deck = SlideDeck::default();
    let ctx = egui::Context::default();
    assert!(!state.web_lyrics_search.open);

    let raw = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(300.0, 600.0),
        )),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw, |ui| {
        toolbar::toolbar(ui, &mut state, &deck, None);
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());
}

#[test]
fn test_web_media_button_in_toolbar() {
    let mut state = ControlPanelState::default();
    state.selected_tab = 1; // Media tab
    let deck = SlideDeck::default();
    let ctx = egui::Context::default();
    assert!(!state.web_media_modal.open);

    let raw = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(300.0, 600.0),
        )),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw, |ui| {
        toolbar::toolbar(ui, &mut state, &deck, None);
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());
}

#[test]
fn test_drag_and_drop_song_to_folder() {
    let mut state = ControlPanelState::default();
    let mut deck = SlideDeck::default();
    let mut song = crate::core::models::Song::new("Se não for para te Adorar", "Fernandinho", "Letra");
    song.id = Some(101);
    deck.library_mut().push(song);

    let entries = model::entries(LibraryKind::Song, &state, &deck);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].folder, "");

    let mut presenting = false;
    actions::execute(
        LibraryAction::Move(vec![101], "Cantor Cristão".into()),
        LibraryKind::Song,
        &entries,
        &mut state,
        &mut deck,
        &mut presenting,
        None,
    );

    assert_eq!(deck.library()[0].folder, "Cantor Cristão");
    assert!(state.open_folders.contains("Cantor Cristão"));
}
