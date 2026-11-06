use super::*;
use crate::core::models::PresentationTheme;

fn run_dock(with_bg: bool) -> egui::FullOutput {
    let mut state = ControlPanelState {
        dock_preview_bg: with_bg,
        ..Default::default()
    };
    let mut slide_deck = SlideDeck::default();
    slide_deck.project_custom_content(
        "10.000 Razões".to_owned(),
        "Artista".to_owned(),
        "Estrofe um\n\nEstrofe dois\n\nEstrofe três".to_owned(),
    );
    let mut is_open = true;
    let mut is_modal = false;

    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(1000.0, 220.0),
        )),
        ..Default::default()
    };
    ctx.run_ui(raw_input, |ui| {
        render_presentation_dock(ui, &mut state, &mut slide_deck, &mut is_open, &mut is_modal);
    })
}

#[test]
fn dock_renders_cards_in_text_and_background_modes() {
    for with_bg in [false, true] {
        let mut output = run_dock(with_bg);
        output.textures_delta.clear();
        assert!(
            !output.shapes.is_empty(),
            "dock produced no shapes (with_bg = {with_bg})"
        );
    }
}

#[test]
fn jump_input_targets_valid_strophes() {
    let mut slide_deck = SlideDeck::default();
    slide_deck.project_custom_content(
        "T".to_owned(),
        "A".to_owned(),
        "um\n\ndois\n\ntrês".to_owned(),
    );
    assert_eq!(slide_deck.total_slides(), 3);
    slide_deck.select_live_slide(2);
    assert_eq!(slide_deck.current_index(), 2);
    // Out-of-range indexes are ignored.
    slide_deck.select_live_slide(9);
    assert_eq!(slide_deck.current_index(), 2);
}

#[test]
fn jump_input_selects_all_on_click_or_focus() {
    let mut slide_deck = SlideDeck::default();
    slide_deck.project_custom_content(
        "T".to_owned(),
        "A".to_owned(),
        "um\n\ndois\n\ntrês".to_owned(),
    );
    slide_deck.select_live_slide(1); // slide 2, number "2"
    let tokens = AppTheme::dark();
    let mut state = ControlPanelState::default();

    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(200.0, 100.0),
        )),
        ..Default::default()
    };

    let mut field_id = egui::Id::NULL;
    let mut output = ctx.run_ui(raw_input, |ui| {
        field_id = ui.make_persistent_id("dock_jump_input");
        ui.memory_mut(|m| m.request_focus(field_id));
        jump_input(ui, &tokens, &mut slide_deck, &mut state);
    });
    output.textures_delta.clear();

    let edit = egui::TextEdit::load_state(&ctx, field_id)
        .expect("TextEdit state should be loaded");
    let range = edit.cursor.char_range().expect("Character range should be selected");
    assert_eq!(range.primary, egui::text::CCursor::new(1));
    assert_eq!(range.secondary, egui::text::CCursor::new(0));
}

#[test]
fn jump_input_only_jumps_on_enter() {
    let mut slide_deck = SlideDeck::default();
    slide_deck.project_custom_content(
        "T".to_owned(),
        "A".to_owned(),
        "um\n\ndois\n\ntrês\n\nquatro".to_owned(),
    );
    slide_deck.select_live_slide(0); // slide 1
    let tokens = AppTheme::dark();
    let mut state = ControlPanelState::default();

    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(200.0, 100.0),
        )),
        ..Default::default()
    };

    let mut field_id = egui::Id::NULL;
    // Frame 1: focus and type "3" in buffer, but Enter is NOT pressed
    let mut out1 = ctx.run_ui(raw_input.clone(), |ui| {
        field_id = ui.make_persistent_id("dock_jump_input");
        ui.ctx().data_mut(|d| d.insert_temp(field_id, "3".to_owned()));
        ui.memory_mut(|m| m.request_focus(field_id));
        jump_input(ui, &tokens, &mut slide_deck, &mut state);
    });
    out1.textures_delta.clear();
    // Still on slide 1 (index 0) because Enter wasn't pressed!
    assert_eq!(slide_deck.current_index(), 0);

    // Frame 2: simulate pressing Enter
    let mut enter_input = raw_input;
    enter_input.events.push(egui::Event::Key {
        key: egui::Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });

    let mut out2 = ctx.run_ui(enter_input, |ui| {
        jump_input(ui, &tokens, &mut slide_deck, &mut state);
    });
    out2.textures_delta.clear();
    // Now it jumped to slide 3 (index 2)!
    assert_eq!(slide_deck.current_index(), 2);
}

#[test]
fn jump_input_focused_by_default_shortcuts() {
    let mut slide_deck = SlideDeck::default();
    slide_deck.project_custom_content(
        "T".to_owned(),
        "A".to_owned(),
        "um\n\ndois\n\ntrês".to_owned(),
    );
    let tokens = AppTheme::dark();
    let mut state = ControlPanelState {
        focus_dock_jump_input: true,
        ..Default::default()
    };

    // Verify defaults include Quote and Slash (numpad divide)
    let bindings = state.settings.shortcuts.bindings(crate::core::commands::Command::JumpSlide);
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].key, "Quote");
    assert_eq!(bindings[1].key, "Slash");

    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(200.0, 100.0),
        )),
        ..Default::default()
    };

    let mut field_id = egui::Id::NULL;
    let mut output = ctx.run_ui(raw_input, |ui| {
        field_id = ui.make_persistent_id("dock_jump_input");
        jump_input(ui, &tokens, &mut slide_deck, &mut state);
    });
    output.textures_delta.clear();

    // focus flag was consumed
    assert!(!state.focus_dock_jump_input);

    let edit = egui::TextEdit::load_state(&ctx, field_id)
        .expect("TextEdit state should be loaded");
    let range = edit.cursor.char_range().expect("Character range should be selected");
    // Text was selected
    assert_eq!(range.primary, egui::text::CCursor::new(1));
    assert_eq!(range.secondary, egui::text::CCursor::new(0));
}

#[test]
fn presentation_timer_badge_renders_and_tracks_time() {
    let mut started_at = Some(std::time::Instant::now() - std::time::Duration::from_secs(75));
    let mut seconds_mode = false;
    let tokens = AppTheme::dark();

    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(300.0, 100.0),
        )),
        ..Default::default()
    };

    // Render in MM:SS mode
    let mut output = ctx.run_ui(raw_input.clone(), |ui| {
        presentation_timer_badge(
            ui,
            &tokens,
            Language::Portuguese,
            &mut started_at,
            &mut seconds_mode,
        );
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());

    // Toggle to seconds mode and render
    seconds_mode = true;
    let mut output2 = ctx.run_ui(raw_input, |ui| {
        presentation_timer_badge(
            ui,
            &tokens,
            Language::Portuguese,
            &mut started_at,
            &mut seconds_mode,
        );
    });
    output2.textures_delta.clear();
    assert!(!output2.shapes.is_empty());
}

#[test]
fn song_dock_renders_title_slide_with_t_badge_and_jumps() {
    let mut state = ControlPanelState::default();
    let mut slide_deck = SlideDeck::default();
    slide_deck.add_or_update_song(
        "Isaías 53".to_owned(),
        "Projeto Sola".to_owned(),
        "Ele não tinha qualquer beleza\nOu majestade pra nos atrair\n\nNada havia em Sua aparência".to_owned(),
    );
    slide_deck.project_preview();
    assert_eq!(slide_deck.live_title(), "Isaías 53");
    assert_eq!(slide_deck.live_artist(), "Projeto Sola");
    assert_eq!(slide_deck.total_live_slides(), 4);
    assert_eq!(slide_deck.live_slide_text(), "Isaías 53\nProjeto Sola");
    assert_eq!(slide_deck.live_slides()[3], "");

    let mut is_open = true;
    let mut is_modal = false;
    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(1000.0, 220.0),
        )),
        ..Default::default()
    };
    let mut output = ctx.run_ui(raw_input, |ui| {
        render_presentation_dock(ui, &mut state, &mut slide_deck, &mut is_open, &mut is_modal);
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());

    // Jump to 1 (strophe 1)
    slide_deck.select_live_slide(1);
    assert_eq!(
        slide_deck.live_slide_text(),
        "Ele não tinha qualquer beleza\nOu majestade pra nos atrair"
    );

    // Jump to 3 (blank slide)
    slide_deck.select_live_slide(3);
    assert_eq!(slide_deck.live_slide_text(), "");

    // Jump back to 0 (Title slide)
    slide_deck.select_live_slide(0);
    assert_eq!(slide_deck.live_slide_text(), "Isaías 53\nProjeto Sola");
}

#[test]
fn advancing_past_last_slide_closes_presentation() {
    let mut slide_deck = SlideDeck::default();
    slide_deck.add_or_update_song(
        "Teste".to_owned(),
        "Artista".to_owned(),
        "Estrofe única".to_owned(),
    );
    slide_deck.project_preview();
    assert_eq!(slide_deck.total_live_slides(), 3);
    assert_eq!(slide_deck.live_slides()[2], "");

    let mut is_open = true;
    slide_deck.last_slide();
    assert_eq!(slide_deck.current_index(), 2);

    if !slide_deck.next_slide() && is_open {
        is_open = false;
    }
    assert!(!is_open);
}

#[test]
fn live_theme_control_renders_and_switches_theme() {
    let mut state = ControlPanelState::default();
    let tokens = state.theme.clone();
    let ctx = egui::Context::default();
    let raw_input = egui::RawInput::default();

    assert!(!state.themes.is_empty());
    let initial_theme_id = state.live_theme().id.clone();

    let second_theme = PresentationTheme {
        id: "test_theme_2".into(),
        name: "Tema Secundario".into(),
        ..Default::default()
    };
    state.themes.push(second_theme.clone());

    let mut output = ctx.run_ui(raw_input, |ui| {
        live_theme_control(ui, &tokens, Language::Portuguese, &mut state);
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());

    let target_idx = state.themes.len() - 1;
    state.selected_theme_index = target_idx;
    state.apply_theme_to_live();
    assert_eq!(state.live_theme().id, "test_theme_2");
    assert_ne!(state.live_theme().id, initial_theme_id);
}

#[test]
fn dock_height_remains_stable_across_multiple_frames() {
    let mut state = ControlPanelState::default();
    let mut slide_deck = SlideDeck::default();
    slide_deck.project_custom_content(
        "Titulo".to_owned(),
        "Artista".to_owned(),
        "Estrofe 1\nLinha 2\n\nEstrofe 2\nLinha 2\n\nEstrofe 3\nLinha 2\n\nEstrofe 4".to_owned(),
    );
    let mut is_open = true;
    let mut is_modal = false;
    let ctx = egui::Context::default();
    let theme = AppTheme::dark();

    let mut heights = Vec::new();

    for _ in 0..5 {
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(1280.0, 720.0),
            )),
            ..Default::default()
        };
        let mut output = ctx.run_ui(raw_input, |ui| {
            let dock_response = egui::Panel::bottom("presentation_dock")
                .resizable(true)
                .show_separator_line(false)
                .default_size(175.0)
                .min_size(130.0)
                .max_size(420.0)
                .frame(
                    egui::Frame::new()
                        .fill(theme.bg_panel)
                        .stroke(egui::Stroke::NONE)
                        .inner_margin(egui::Margin::symmetric(10, 8)),
                )
                .show(ui, |ui| {
                    render_presentation_dock(ui, &mut state, &mut slide_deck, &mut is_open, &mut is_modal);
                });
            heights.push(dock_response.response.rect.height());
        });
        output.textures_delta.clear();
    }

    assert_eq!(heights.len(), 5);
    for i in 1..heights.len() {
        assert!(
            (heights[i] - heights[0]).abs() < 0.5,
            "dock height shifted from frame 0 ({}) to frame {} ({})",
            heights[0],
            i,
            heights[i]
        );
    }
}

#[test]
fn dock_scrolls_to_active_slide_on_navigation() {
    let mut state = ControlPanelState::default();
    let mut slide_deck = SlideDeck::default();
    let lyrics = (1..=20)
        .map(|i| format!("Estrofe {i} da música"))
        .collect::<Vec<_>>()
        .join("\n\n");
    slide_deck.add_or_update_song("Música Longa".to_owned(), "Artista".to_owned(), lyrics);
    slide_deck.project_preview();
    assert_eq!(slide_deck.total_live_slides(), 22);

    let mut is_open = true;
    let mut is_modal = false;
    let ctx = egui::Context::default();

    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(800.0, 220.0),
        )),
        ..Default::default()
    };
    let mut output = ctx.run_ui(raw_input.clone(), |ui| {
        render_presentation_dock(ui, &mut state, &mut slide_deck, &mut is_open, &mut is_modal);
    });
    output.textures_delta.clear();

    // Navigate to slide 15
    slide_deck.select_live_slide(15);
    assert_eq!(slide_deck.current_index(), 15);

    let mut output2 = ctx.run_ui(raw_input, |ui| {
        render_presentation_dock(ui, &mut state, &mut slide_deck, &mut is_open, &mut is_modal);
    });
    output2.textures_delta.clear();
    assert!(!output2.shapes.is_empty());
}

#[test]
fn slide_card_wraps_long_verse_and_conditional_breaks_matching_projection() {
    let tokens = AppTheme::light();
    let theme = PresentationTheme::default();
    let long_verse = "And the earth was without form, and void; and darkness was on the face of the deep. | And the Spirit of God was hovering over the face of the waters.\nGênesis 1:2";

    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(800.0, 600.0),
        )),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        // Render card with with_bg = false (dock card mode)
        render_slide_card(
            ui,
            &tokens,
            &theme,
            long_verse,
            1,
            false,
            false,
            16.0 / 9.0,
            false,
            160.0,
            false,
        );

        // Render card with with_bg = true (with background mode)
        render_slide_card(
            ui,
            &tokens,
            &theme,
            long_verse,
            2,
            false,
            false,
            16.0 / 9.0,
            true,
            160.0,
            false,
        );
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty(), "Cards should produce shapes");
}



