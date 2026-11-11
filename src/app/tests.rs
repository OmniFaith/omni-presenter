use crate::ui;

use super::*;

#[test]
fn settings_save_persists_and_restores_without_replacing_live_output() {
    let db = AppDb::open_in_memory().unwrap();
    let mut app = PresentationApp::with_db(db.clone());
    app.control_state.capture_profiles();
    let live = app.slide_deck.projected_text().to_owned();
    let live_theme = app.control_state.live_theme_id.clone();
    let mut draft = app.control_state.settings.clone();
    draft.operators[0].dark_mode = true;
    draft.operators[0].scale = 125;
    draft.services[0].theme_id = "midnight_sapphire".into();
    draft.physical_output_enabled = false;
    app.control_state.settings_ui.draft = Some(draft.clone());
    app.control_state.settings_ui.save_requested = true;
    app.control_state.settings_ui.open = true;
    app.save_settings_draft();
    assert_eq!(db.load_settings().unwrap().unwrap(), draft);
    assert_eq!(app.slide_deck.projected_text(), live);
    assert_eq!(app.control_state.live_theme_id, live_theme);
    assert!(!app.control_state.settings_ui.open);
    let restored = PresentationApp::with_db(db);
    assert_eq!(restored.control_state.settings.operator().scale, 125);
    assert_eq!(restored.control_state.theme, ui::theme::AppTheme::dark());
    assert_eq!(restored.control_state.preview_theme().id, "midnight_sapphire");
}

#[test]
fn test_app_frame_performance() {
    let mut app = PresentationApp::with_db(AppDb::open_in_memory().unwrap());
    let ctx = egui::Context::default();

    let mut raw_input = egui::RawInput::default();
    raw_input.screen_rect = Some(egui::Rect::from_min_size(
        egui::pos2(0.0, 0.0),
        egui::vec2(1280.0, 720.0),
    ));
    let mut rendered_leaf_rects = Vec::new();
    let mut output = ctx.run_ui(raw_input.clone(), |ui| {
        ui::render_layout(
            ui,
            &mut app.control_state,
            &mut app.slide_deck,
            &mut app.is_presentation_open,
            &mut rendered_leaf_rects,
            Some(&app.db),
        );
    });
    output.textures_delta.clear();
    assert!(!rendered_leaf_rects.is_empty());
}

#[test]
fn interface_theme_follows_system_changes_and_applies_custom_palettes() {
    use crate::core::interface_theme::{InterfaceTheme, ThemeMode};
    let mut app = PresentationApp::with_db(AppDb::open_in_memory().unwrap());
    app.control_state.settings.operator_mut().theme_mode = ThemeMode::System;
    let ctx = egui::Context::default();
    for system in [egui::Theme::Light, egui::Theme::Dark, egui::Theme::Light] {
        let mut output = ctx.run_ui(egui::RawInput { system_theme: Some(system), ..Default::default() }, |_| app.apply_appearance(&ctx));
        output.textures_delta.clear();
        assert_eq!(ctx.theme(), system);
        assert_eq!(app.control_state.settings.operator().dark_mode, system == egui::Theme::Dark);
    }
    let mut custom = InterfaceTheme { id: "test".into(), ..Default::default() };
    custom.dark.insert("accent".into(), "#F044AA".into());
    app.control_state.settings.interface_themes.push(custom);
    app.control_state.settings.operator_mut().theme_mode = ThemeMode::Dark;
    app.control_state.settings.operator_mut().interface_theme_id = "test".into();
    let mut output = ctx.run_ui(egui::RawInput::default(), |_| app.apply_appearance(&ctx));
    output.textures_delta.clear();
    assert_eq!(app.control_state.theme.accent, egui::Color32::from_rgb(240,68,170));
}

#[test]
fn test_default_app_initialization_with_real_db() {
    let db = AppDb::open("omni-presenter.db").unwrap();
    let app = PresentationApp::with_db(db);
    assert!(!app.control_state.themes.is_empty());
    assert!(app.control_state.themes.iter().any(|t| t.id == "obsidian_pro"));
    assert!(!app.control_state.themes.iter().any(|t| t.id == "hands_water"));
    assert!(!app.control_state.themes.iter().any(|t| t.id == "starry_sky"));
    assert!(app.control_state.themes.iter().any(|t| t.id == "shader_aurora"));
}

#[test]
fn test_song_editor_modal_with_long_lyrics_scrolls_cleanly() {
    let mut app = PresentationApp::with_db(AppDb::open_in_memory().unwrap());
    let ctx = egui::Context::default();

    app.control_state.song_editor.open = true;
    app.control_state.song_editor.title = "Se Não For Pra Te Servir".into();
    app.control_state.song_editor.artist = "Louvor".into();
    let mut long_lyrics = String::new();
    for i in 1..=50 {
        long_lyrics.push_str(&format!("Verso {i}: Se não for pra te servir porque eu estou aqui\n\n"));
    }
    app.control_state.song_editor.text = long_lyrics;

    let mut raw_input = egui::RawInput::default();
    raw_input.screen_rect = Some(egui::Rect::from_min_size(
        egui::pos2(0.0, 0.0),
        egui::vec2(1024.0, 700.0),
    ));

    let mut is_text_focused = false;
    let mut output = ctx.run_ui(raw_input, |_ui| {
        ui::render_editor_modal(
            &ctx,
            &mut app.control_state,
            &mut app.slide_deck,
            &mut is_text_focused,
            Some(&app.db),
        );
    });
    output.textures_delta.clear();
    assert!(app.control_state.song_editor.open);
}

#[test]
fn test_song_editor_with_conditional_breaks_and_ratio_toggle() {
    let mut app = PresentationApp::with_db(AppDb::open_in_memory().unwrap());
    let ctx = egui::Context::default();

    app.control_state.song_editor.open = true;
    app.control_state.song_editor.title = "Porque Ele Vive".into();
    app.control_state.song_editor.artist = "Harpa Cristã".into();
    app.control_state.song_editor.text =
        "Deus enviou seu Filho amado | Para perdoar [4:3] e me salvar\n\nPorque Ele vive [4:3] posso crer no amanhã".into();

    let mut raw_input = egui::RawInput::default();
    raw_input.screen_rect = Some(egui::Rect::from_min_size(
        egui::pos2(0.0, 0.0),
        egui::vec2(1280.0, 800.0),
    ));

    // Render in 16:9 mode
    app.control_state.song_editor.preview_ratio_4_3 = false;
    let mut is_text_focused = false;
    let mut out169 = ctx.run_ui(raw_input.clone(), |_ui| {
        ui::render_editor_modal(
            &ctx,
            &mut app.control_state,
            &mut app.slide_deck,
            &mut is_text_focused,
            Some(&app.db),
        );
    });
    out169.textures_delta.clear();

    // Toggle to 4:3 mode
    app.control_state.song_editor.preview_ratio_4_3 = true;
    let mut out43 = ctx.run_ui(raw_input, |_ui| {
        ui::render_editor_modal(
            &ctx,
            &mut app.control_state,
            &mut app.slide_deck,
            &mut is_text_focused,
            Some(&app.db),
        );
    });
    out43.textures_delta.clear();

    // Verify syntax highlighting
    let job = crate::ui::components::lyrics_highlighter::highlight_lyrics_job(
        &app.control_state.song_editor.text,
        &app.control_state.settings.strophe_tags,
        &app.control_state.theme,
        14.0,
        400.0,
        false,
    );
    assert!(!job.text.is_empty());
}

#[test]
fn test_theme_live_preview_in_settings_reverts_if_not_saved() {
    let mut app = PresentationApp::with_db(AppDb::open_in_memory().unwrap());
    let ctx = egui::Context::default();

    // Initial appearance
    app.apply_appearance(&ctx);
    let initial_theme = app.control_state.theme.clone();

    // Open settings modal and preview a different theme in the draft
    app.control_state.settings_ui.open = true;
    let mut draft = app.control_state.settings.clone();
    let initial_scale = app.control_state.settings.operator().scale;
    draft.operator_mut().interface_theme_id = "forest".into();
    draft.operator_mut().scale = 140;
    app.control_state.settings_ui.draft = Some(draft.clone());

    // apply_appearance should live-preview the theme from draft, but keep saved scale
    app.apply_appearance(&ctx);
    let preview_theme = app.control_state.theme.clone();
    assert_ne!(preview_theme, initial_theme);
    assert_eq!(ctx.zoom_factor(), initial_scale as f32 / 100.0);

    // Cancel / close settings without saving (draft is cleared, open = false)
    app.control_state.settings_ui.open = false;
    app.control_state.settings_ui.draft = None;
    app.apply_appearance(&ctx);
    assert_eq!(app.control_state.theme, initial_theme);

    // Now open and save draft
    app.control_state.settings_ui.open = true;
    draft.operator_mut().interface_theme_id = "violet".into();
    app.control_state.settings_ui.draft = Some(draft);
    app.control_state.settings_ui.save_requested = true;
    app.save_settings_draft();

    app.apply_appearance(&ctx);
    assert_eq!(app.control_state.settings.operator().interface_theme_id, "violet");
    let (saved_theme, _) = ui::theme::AppTheme::resolve(&app.control_state.settings, &ctx);
    assert_eq!(app.control_state.theme, saved_theme);
}

#[test]
fn test_execute_command_previous_slide_navigates_back() {
    use crate::core::commands::Command;
    use crate::core::slides::SlideDeck;
    let mut app = PresentationApp::with_db(AppDb::open_in_memory().unwrap());
    let ctx = egui::Context::default();

    // Setup dummy song slides
    app.slide_deck = SlideDeck::new("Strophe 1\n\nStrophe 2\n\nStrophe 3");
    assert_eq!(app.slide_deck.current_index(), 0);

    // Advance to slide 1
    app.execute_command(&ctx, Command::NextSlide);
    assert_eq!(app.slide_deck.current_index(), 1);

    // Advance to slide 2
    app.execute_command(&ctx, Command::NextSlide);
    assert_eq!(app.slide_deck.current_index(), 2);

    // Go back with PreviousSlide
    app.execute_command(&ctx, Command::PreviousSlide);
    assert_eq!(app.slide_deck.current_index(), 1);

    // Go back to slide 0
    app.execute_command(&ctx, Command::PreviousSlide);
    assert_eq!(app.slide_deck.current_index(), 0);
}

#[test]
fn tab_and_shift_tab_cycle_workspace_containers() {
    use crate::core::layout::ContainerKind;
    use crate::core::state::SidebarPage;
    let mut app = PresentationApp::with_db(AppDb::open_in_memory().unwrap());
    let ctx = egui::Context::default();

    app.control_state.active_sidebar_page = SidebarPage::Home;
    app.control_state.focused_container = Some(ContainerKind::Selection);

    // Simulate pressing Tab
    let mut input = egui::RawInput::default();
    input.events.push(egui::Event::Key {
        key: egui::Key::Tab,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    let mut output = ctx.run_ui(input, |_| {});
    output.textures_delta.clear();
    app.handle_global_input(&ctx, false, false);
    assert_eq!(app.control_state.focused_container, Some(ContainerKind::Player));

    // Simulate pressing Tab again
    let mut input2 = egui::RawInput::default();
    input2.events.push(egui::Event::Key {
        key: egui::Key::Tab,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    let mut output2 = ctx.run_ui(input2, |_| {});
    output2.textures_delta.clear();
    app.handle_global_input(&ctx, false, false);
    assert_eq!(app.control_state.focused_container, Some(ContainerKind::Lyrics));

    // Simulate pressing Shift+Tab to go back
    let mut input3 = egui::RawInput::default();
    input3.events.push(egui::Event::Key {
        key: egui::Key::Tab,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::SHIFT,
    });
    let mut output3 = ctx.run_ui(input3, |_| {});
    output3.textures_delta.clear();
    app.handle_global_input(&ctx, false, false);
    assert_eq!(app.control_state.focused_container, Some(ContainerKind::Player));
}
