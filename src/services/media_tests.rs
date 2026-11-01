use super::{db::AppDb, media};
use crate::core::{slides::SlideDeck, state::ControlPanelState};

#[test]
fn media_preview_and_live_do_not_create_themes_and_keep_independent_paths() {
    let mut state = ControlPanelState::default();
    let mut deck = SlideDeck::new("Letra ao vivo");
    let themes = state.themes.clone();
    let selected = state.selected_theme_index;
    let first_path = std::env::temp_dir().join(format!("{}.jpg", crate::core::preferences::new_id("media-first")));
    let second_path = std::env::temp_dir().join(format!("{}.jpg", crate::core::preferences::new_id("media-second")));
    std::fs::write(&first_path, []).unwrap();
    std::fs::write(&second_path, []).unwrap();
    let first = first_path.to_str().unwrap();
    let second = second_path.to_str().unwrap();
    media::activate(
        &mut state,
        &mut deck,
        "Primeira imagem",
        first,
        "image/jpeg",
        true,
    )
    .unwrap();
    media::activate(
        &mut state,
        &mut deck,
        "Segunda imagem",
        second,
        "image/jpeg",
        false,
    )
    .unwrap();
    assert_eq!(state.themes, themes);
    assert_eq!(state.selected_theme_index, selected);
    assert_eq!(media::live_theme(&state, &deck).bg_image_path, first);
    assert_eq!(media::preview_theme(&state, &deck).bg_image_path, second);
    deck.load_custom_content("Aviso".into(), String::new(), "Texto".into());
    assert!(deck.preview_media.is_none());
    assert_eq!(
        media::preview_theme(&state, &deck),
        state.preview_theme().clone()
    );
    assert_eq!(media::live_theme(&state, &deck).bg_image_path, first);
    deck.project_preview();
    assert!(deck.live_media.is_none());
    assert_eq!(media::live_theme(&state, &deck), state.live_theme().clone());
    std::fs::remove_file(first_path).unwrap();
    std::fs::remove_file(second_path).unwrap();
}

#[test]
fn selecting_a_broken_image_defers_validation_to_the_background_loader() {
    let path = std::env::temp_dir().join(format!(
        "omni-presenter-invalid-image-{}.jpg",
        crate::core::preferences::new_id("test")
    ));
    std::fs::write(&path, b"not a valid jpeg").unwrap();
    let mut state = ControlPanelState::default();
    let mut deck = SlideDeck::new("Conteúdo ao vivo");
    let themes = state.themes.clone();
    assert!(
        media::activate(
            &mut state,
            &mut deck,
            "Imagem",
            &path.to_string_lossy(),
            "image/jpeg",
            false
        )
        .is_ok()
    );
    assert_eq!(state.themes, themes);
    assert_eq!(deck.live_slide_text(), "Conteúdo ao vivo");
    let ctx = eframe::egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        if matches!(
            super::images::request(&ctx, &path.to_string_lossy(), 320),
            super::images::ImageLoad::Failed(_)
        ) {
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn media_import_only_populates_the_media_catalog() {
    let db = AppDb::open_in_memory().unwrap();
    db.add_media_item("Imagem", "example.jpg", "image", "")
        .unwrap();
    let mut state = ControlPanelState::default();
    let themes = state.themes.clone();
    state.reload_media_items(&db);
    assert_eq!(state.media_items.len(), 1);
    assert_eq!(state.themes, themes);
}
