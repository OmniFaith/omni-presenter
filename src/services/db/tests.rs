use super::*;
use crate::core::models::PresentationTheme;

#[test]
fn imported_bible_survives_database_roundtrip() {
    let db = AppDb::open_in_memory().unwrap();
    let mut translation = (*crate::core::bible::BibleLibrary::default()
        .get(crate::core::bible::DEFAULT_BIBLE)
        .unwrap())
    .clone();
    translation.id = "TESTE".into();
    translation.name = "Versão de teste".into();
    db.save_bible_translation(&translation).unwrap();
    db.save_bible_translation(&translation).unwrap();
    let restored = db.get_bible_translations().unwrap();
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].id, "TESTE");
    assert_eq!(
        restored[0].passage(16, 10, 2, 2).unwrap().content,
        translation.passage(16, 10, 2, 2).unwrap().content
    );
}

#[test]
fn settings_roundtrip_preserves_profiles_and_virtual_screens() {
    let db = AppDb::open_in_memory().unwrap();
    assert!(db.load_settings().unwrap().is_none());
    let mut settings = crate::core::preferences::AppSettings::default();
    settings.operators[0].name = "Ana · projeção".into();
    settings.operators[0].dark_mode = true;
    settings.operators[0].scale = 125;
    settings.operators[0].layout.root.swap_leaves(
        crate::core::layout::ContainerKind::Lyrics,
        crate::core::layout::ContainerKind::Stage,
    );
    settings.services[0].playlist = crate::core::state::ControlPanelState::default().playlist;
    settings.server_enabled = true;
    settings.screens[0].lyrics_only = true;
    db.save_settings(&settings).unwrap();
    assert_eq!(db.load_settings().unwrap().unwrap(), settings);
    settings.operators[0].scale = 110;
    db.save_settings(&settings).unwrap();
    assert_eq!(db.load_settings().unwrap().unwrap(), settings);
}

#[test]
fn operator_profile_preserves_dock_preview_bg() {
    let db = AppDb::open_in_memory().unwrap();
    let mut state = crate::core::state::ControlPanelState::default();
    assert!(!state.dock_preview_bg);

    // Switch preview mode to Background
    state.dock_preview_bg = true;
    state.capture_profiles();
    db.save_settings(&state.settings).unwrap();

    let loaded = db.load_settings().unwrap().unwrap();
    let mut restored = crate::core::state::ControlPanelState {
        settings: loaded,
        ..Default::default()
    };
    restored.apply_profiles();
    assert!(restored.dock_preview_bg);
}

#[test]
fn settings_preserve_default_and_custom_network_ports() {
    let db = AppDb::open_in_memory().unwrap();
    for port in [80, 3000, 8080, 65535] {
        let settings = crate::core::preferences::AppSettings {
            server_port: port,
            ..Default::default()
        };
        db.save_settings(&settings).unwrap();
        assert_eq!(db.load_settings().unwrap().unwrap().server_port, port);
    }
}

#[test]
fn test_sqlite_song_crud() {
    let db = AppDb::open_in_memory().expect("in-memory db failed");
    let songs = db.get_all_songs().expect("failed to get songs");
    assert_eq!(songs.len(), 0, "fresh db should have 0 songs");

    // Add
    let id = db
        .add_song("Nova Canção", "Artista Teste", "Letra teste\n\nEstrofe 2")
        .expect("add failed");
    assert!(id > 0);

    // Search
    let found = db.search_songs("Nova").expect("search failed");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].title, "Nova Canção");

    // Update
    db.update_song(id, "Canção Atualizada", "Novo Artista", "Texto modificado")
        .expect("update failed");
    let found = db.search_songs("Atualizada").expect("search failed");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].artist, "Novo Artista");

    // Delete
    db.delete_song(id).expect("delete failed");
    let found = db.search_songs("Atualizada").expect("search failed");
    assert!(found.is_empty());

    // Folders test
    let folders = db.get_folders().expect("get_folders failed");
    assert!(!folders.contains(&"Cantor Cristão".to_string()));

    db.add_folder("Jovens").expect("add_folder failed");
    let folders = db.get_folders().expect("get_folders failed");
    assert!(folders.contains(&"Jovens".to_string()));

    let id2 = db
        .add_song_with_folder("Música Jovem", "Banda", "Letra", "Jovens")
        .expect("add song with folder failed");
    let songs = db.get_all_songs().expect("get songs failed");
    let s = songs.iter().find(|s| s.id == Some(id2)).unwrap();
    assert_eq!(s.folder, "Jovens");

    db.move_song_to_folder(id2, "Coral")
        .expect("move song failed");
    let songs = db.get_all_songs().expect("get songs failed");
    let s = songs.iter().find(|s| s.id == Some(id2)).unwrap();
    assert_eq!(s.folder, "Coral");

    db.delete_folder("Jovens").expect("delete_folder failed");
    let folders = db.get_folders().expect("get_folders failed");
    assert!(!folders.contains(&"Jovens".to_string()));
}

#[test]
fn test_sqlite_theme_crud() {
    let db = AppDb::open_in_memory().expect("in-memory db failed");
    let themes = db.get_all_themes().expect("failed to get themes");
    assert!(!themes.is_empty(), "seed should insert default themes");

    let obsidian_theme = themes.iter().find(|t| t.id == "obsidian_pro");
    assert!(obsidian_theme.is_some(), "obsidian_pro theme should be seeded");
    assert!(!themes.iter().any(|t| t.id == "hands_water" || t.id == "starry_sky"));

    // Save a custom theme
    let mut custom = PresentationTheme::default();
    custom.id = "custom_test_1".to_string();
    custom.name = "Tema Teste Culto".to_string();
    custom.font_family = "Montserrat".to_string();
    custom.font_size = 50.0;
    custom.font_weight = 800;
    custom.outline_enabled = true;
    custom.outline_width = 3.5;
    custom.shadow_offset_x = 4.0;
    custom.shadow_offset_y = 5.0;
    custom.box_x = 0.15;
    custom.box_y = 0.25;
    custom.box_bg_enabled = true;
    custom.box_radius = 12.0;
    custom.options.shader.kind = "fluid".into();
    custom.options.shader.speed = 0.75;
    custom.options.transition_ms = 650;
    custom.options.wrap_text = false;
    custom.options.video.path = "/tmp/test-video.mp4".into();

    db.save_theme(&custom).expect("failed to save theme");
    let themes_after = db.get_all_themes().expect("failed to query themes");
    let saved = themes_after
        .iter()
        .find(|t| t.id == "custom_test_1")
        .expect("saved theme not found");
    assert_eq!(saved.name, "Tema Teste Culto");
    assert_eq!(saved.font_size, 50.0);
    assert_eq!(saved.font_weight, 800);
    assert!(saved.outline_enabled);
    assert_eq!(saved.outline_width, 3.5);
    assert_eq!(saved.shadow_offset_x, 4.0);
    assert_eq!(saved.shadow_offset_y, 5.0);
    assert_eq!(saved.box_x, 0.15);
    assert!(saved.box_bg_enabled);
    assert_eq!(saved.box_radius, 12.0);
    assert_eq!(saved.options, custom.options);

    // Update favorite
    db.update_theme_favorite("custom_test_1", true)
        .expect("failed to toggle favorite");
    let favs = db.get_all_themes().expect("failed to query themes");
    let updated = favs.iter().find(|t| t.id == "custom_test_1").unwrap();
    assert!(updated.is_favorite);

    // Delete
    db.delete_theme("custom_test_1")
        .expect("failed to delete theme");
    let final_themes = db.get_all_themes().expect("failed to query themes");
    assert!(
        final_themes
            .iter()
            .find(|t| t.id == "custom_test_1")
            .is_none()
    );
}

#[test]
fn linked_song_versions_roundtrip_and_cleanup() {
    let db = AppDb::open_in_memory().unwrap();
    let a = db.add_song("Cristo Vive A", "Equipe", "A").unwrap();
    let b = db.add_song("Cristo Vive B", "Equipe", "B").unwrap();
    let c = db.add_song("Cristo Vive C", "Equipe", "C").unwrap();

    db.set_song_versions(a, &[b, c]).unwrap();
    let versions = db.song_versions(a).unwrap();
    assert_eq!(versions.len(), 3);
    assert_eq!(db.song_version_ids(b).unwrap().len(), 2);
    assert_eq!(db.song_version_ids_of("Cristo Vive A").unwrap().len(), 2);

    db.set_song_versions(a, &[]).unwrap();
    assert!(db.song_versions(a).unwrap().is_empty());
    assert_eq!(db.song_version_ids(b).unwrap().len(), 1);

    db.delete_song(b).unwrap();
    assert!(db.song_versions(c).unwrap().len() <= 1);
}

#[test]
fn test_default_themes_upgrade_and_restore() {
    let db = AppDb::open_in_memory().unwrap();
    db.upgrade_default_presentation_themes().unwrap();
    let themes = db.get_all_themes().unwrap();
    assert!(themes.iter().any(|t| t.id == "obsidian_pro"));
    assert!(!themes.iter().any(|t| t.id == "hands_water"));
    assert!(!themes.iter().any(|t| t.id == "starry_sky"));
    assert!(themes.iter().any(|t| t.id == "midnight_sapphire"));
    assert!(themes.iter().any(|t| t.id == "warm_ember"));
    assert!(themes.iter().any(|t| t.id == "deep_emerald"));
    assert!(themes.iter().any(|t| t.id == "scripture_serif"));
    assert!(themes.iter().any(|t| t.id == "clean_alabaster"));
    assert!(themes.iter().any(|t| t.id == "broadcast_lower_third"));
    assert!(themes.iter().any(|t| t.id == "frosted_glass_card"));
    assert!(themes.iter().any(|t| t.id == "shader_aurora"));

    // Legacy low contrast IDs should have been cleaned up
    assert!(!themes.iter().any(|t| t.id == "amber_gold"));
    assert!(!themes.iter().any(|t| t.id == "warm_orange"));

    // Delete one default theme and test restore
    db.delete_theme("obsidian_pro").unwrap();
    assert!(!db.get_all_themes().unwrap().iter().any(|t| t.id == "obsidian_pro"));

    db.restore_default_presentation_themes().unwrap();
    assert!(db.get_all_themes().unwrap().iter().any(|t| t.id == "obsidian_pro"));
}
