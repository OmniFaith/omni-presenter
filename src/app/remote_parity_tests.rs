use super::*;

#[test]
fn bible_version_options_and_shortcuts_sync_without_replacing_live_output() {
    let (mut app, first, _) = fixture();
    app.apply_web_command(Command::Select {
        kind: ResourceKind::Song,
        id: first.to_string(),
        project: true,
    })
    .unwrap();
    let live = app.slide_deck.live_slides().to_vec();
    app.apply_web_command(Command::Bible {
        reference: "João 3:16-18".into(),
        version: "ARA".into(),
        merge: false,
        project: false,
        playlist: false,
    })
    .unwrap();
    app.apply_web_command(Command::BibleVersion { id: "KJV".into() })
        .unwrap();
    assert_eq!(app.control_state.bible_chapter, 3);
    assert_eq!(app.control_state.bible_verse, 16);
    assert_eq!(app.control_state.bible.verse_end, 18);
    assert_eq!(app.slide_deck.live_slides(), live);
    app.apply_web_command(Command::BibleOptions {
        merge: true,
        cards: true,
    })
    .unwrap();
    assert!(app.control_state.bible.cards_layout);
    app.apply_web_command(Command::BibleShortcut {
        id: "KJV".into(),
        operation: "add".into(),
        target: None,
    })
    .unwrap();
    app.apply_web_command(Command::BibleShortcut {
        id: "KJV".into(),
        operation: "remove".into(),
        target: None,
    })
    .unwrap();
    assert!(!app.control_state.bible_shortcuts.contains(&"KJV".into()));
    let saved = app.db.load_settings().unwrap().unwrap();
    assert!(saved.operator().bible_cards_layout);
    assert_eq!(saved.service().bible_version, "KJV");
    assert!(
        app.web_control.snapshot()["bible_versions"]
            .as_array()
            .unwrap()
            .len()
            > 1
    );
    assert!(
        app.apply_web_command(Command::BibleVersion {
            id: "missing-version".into()
        })
        .is_err()
    );
}

#[test]
fn song_editor_preserves_metadata_for_legacy_clients_and_keeps_live_independent() {
    let (mut app, first, _) = fixture();
    app.apply_web_command(Command::Select {
        kind: ResourceKind::Song,
        id: first.to_string(),
        project: true,
    })
    .unwrap();
    app.apply_web_command(Command::SaveSong {
        id: Some(first),
        title: "Edited".into(),
        artist: "Author".into(),
        content: "#*refrão\n//c: #FFF\n**New** | lyric".into(),
        folder: Some("Worship".into()),
        tags: Some("faith,hope".into()),
    })
    .unwrap();
    app.apply_web_command(Command::SaveSong {
        id: Some(first),
        title: "Edited again".into(),
        artist: "Author".into(),
        content: "Two\n\nStanzas".into(),
        folder: None,
        tags: None,
    })
    .unwrap();
    let song = app
        .db
        .get_all_songs()
        .unwrap()
        .into_iter()
        .find(|s| s.id == Some(first))
        .unwrap();
    assert_eq!(song.folder, "Worship");
    assert_eq!(song.tags, "faith,hope");
    assert_eq!(app.slide_deck.live_title(), "Canção de Esperança");
    assert_eq!(app.slide_deck.title(), "Edited again");
}

#[test]
fn playlist_selection_batch_moves_insertion_and_save_use_native_state_and_ids() {
    let (mut app, first, second) = fixture();
    for id in [first, second, first] {
        app.apply_web_command(Command::PlaylistAdd {
            kind: ResourceKind::Song,
            id: id.to_string(),
        })
        .unwrap();
    }
    let ids: Vec<_> = app
        .control_state
        .playlist
        .iter()
        .map(|i| i.id.clone())
        .collect();
    app.apply_web_command(Command::PlaylistSelection {
        id: ids[0].clone(),
        range: false,
        additive: false,
    })
    .unwrap();
    app.apply_web_command(Command::PlaylistSelection {
        id: ids[1].clone(),
        range: true,
        additive: false,
    })
    .unwrap();
    assert_eq!(app.control_state.playlist_selection.selected.len(), 2);
    let focused = ids[1].clone();
    app.apply_web_command(Command::PlaylistBatch {
        operation: "move".into(),
        ids: ids[..2].to_vec(),
        target: Some(ids[2].clone()),
        after: true,
        played: false,
    })
    .unwrap();
    assert_eq!(app.control_state.playlist[0].id, ids[2]);
    assert_eq!(
        app.control_state.playlist[app.control_state.selected_playlist_index.unwrap()].id,
        focused
    );
    app.apply_web_command(Command::PlaylistInsert {
        kind: ResourceKind::Song,
        id: first.to_string(),
        target: Some(focused.clone()),
        after: false,
    })
    .unwrap();
    assert_eq!(
        app.control_state.playlist[app.control_state.selected_playlist_index.unwrap()].id,
        focused
    );
    let before = app.control_state.playlist.clone();
    assert!(
        app.apply_web_command(Command::PlaylistBatch {
            operation: "remove".into(),
            ids: vec!["unknown-id".into()],
            target: None,
            after: false,
            played: false
        })
        .is_err()
    );
    assert_eq!(app.control_state.playlist, before);
    app.apply_web_command(Command::PlaylistBatch {
        operation: "save".into(),
        ids: Vec::new(),
        target: None,
        after: false,
        played: false,
    })
    .unwrap();
    assert_eq!(
        app.db.load_settings().unwrap().unwrap().service().playlist,
        app.control_state.playlist
    );
    assert!(!app.is_presentation_open);
}
