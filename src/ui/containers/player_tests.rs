use super::*;

#[test]
fn test_player_column_renders_empty_state() {
    let mut state = ControlPanelState::default();
    let mut deck = SlideDeck::default();
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        render_player_column(ui, &mut state, &mut deck, &mut false);
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());
}

#[test]
fn test_player_column_renders_with_audio_and_video() {
    let mut state = ControlPanelState::default();
    let mut deck = SlideDeck::default();
    state.active_player_media = Some(crate::core::state::ActivePlayerMedia {
        title: "Trilha Sonora".into(),
        path: "/path/to/music.mp3".into(),
        kind: PlayerMediaKind::Audio,
        duration_secs: 180.0,
        is_playing: true,
        is_projecting: false,
        video_start_time: std::time::Instant::now(),
        video_paused_at: None,
        video_fit: "contain".into(),
    });

    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        render_player_column(ui, &mut state, &mut deck, &mut false);
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());

    state.active_player_media = Some(crate::core::state::ActivePlayerMedia {
        title: "Background Video".into(),
        path: "/path/to/video.mp4".into(),
        kind: PlayerMediaKind::Video,
        duration_secs: 60.0,
        is_playing: true,
        is_projecting: true,
        video_start_time: std::time::Instant::now(),
        video_paused_at: None,
        video_fit: "contain".into(),
    });
    deck.live_media = Some(crate::core::slides::MediaOutput {
        path: "/path/to/video.mp4".into(),
        kind: "video".into(),
    });

    let mut output2 = ctx.run_ui(egui::RawInput::default(), |ui| {
        render_player_column(ui, &mut state, &mut deck, &mut true);
    });
    output2.textures_delta.clear();
    assert!(!output2.shapes.is_empty());
}

#[test]
fn test_player_stop_closes_player_and_unprojects() {
    let mut state = ControlPanelState::default();
    let mut deck = SlideDeck::default();
    let video_path = "/path/to/video.mp4";
    let video_title = "Meu Video";

    deck.load_custom_content(video_title.into(), "Mídia".into(), String::new());
    deck.preview_media = Some(crate::core::slides::MediaOutput {
        path: video_path.into(),
        kind: "video".into(),
    });
    deck.project_preview();

    state.active_player_media = Some(crate::core::state::ActivePlayerMedia {
        title: video_title.into(),
        path: video_path.into(),
        kind: PlayerMediaKind::Video,
        duration_secs: 60.0,
        is_playing: true,
        is_projecting: true,
        video_start_time: std::time::Instant::now(),
        video_paused_at: None,
        video_fit: "contain".into(),
    });

    let is_open = true;

    assert!(deck.live_media.is_some());
    assert_eq!(deck.live_title(), video_title);
    assert!(state.active_player_media.is_some());

    // Simulate clicking stop button
    let ctx = egui::Context::default();
    let media = state.active_player_media.clone().unwrap();
    crate::services::video::stop(&ctx, &media.path);
    if deck.live_media.as_ref().map(|m| &m.path) == Some(&media.path) {
        deck.live_media = None;
        deck.preview_media = None;
        if deck.live_title() == media.title {
            deck.clear_live();
        }
        state.apply_theme_to_live();
    }
    state.close_player();

    assert!(state.active_player_media.is_none());
    assert!(deck.live_media.is_none());
    assert_eq!(deck.live_title(), "");
    assert_eq!(deck.live_slides(), &[""]);
    assert_eq!(deck.projected_text(), "");
    assert!(is_open);
}

#[test]
fn test_player_video_finishes_and_auto_closes_with_transition() {
    let mut state = ControlPanelState::default();
    let mut deck = SlideDeck::default();
    let video_path = "/path/to/short_video.mp4";
    let video_title = "Short Clip";

    state.active_player_media = Some(crate::core::state::ActivePlayerMedia {
        title: video_title.into(),
        path: video_path.into(),
        kind: PlayerMediaKind::Video,
        duration_secs: 2.0,
        is_playing: true,
        is_projecting: true,
        video_start_time: std::time::Instant::now(),
        video_paused_at: None,
        video_fit: "contain".into(),
    });
    deck.live_media = Some(crate::core::slides::MediaOutput {
        path: video_path.into(),
        kind: "video".into(),
    });

    let mut is_open = true;
    let ctx = egui::Context::default();
    // Frame 1: playing video, animation primed to 1.0
    let mut input1 = egui::RawInput::default();
    input1.time = Some(0.0);
    let mut output1 = ctx.run_ui(input1, |ui| {
        render_player_column(ui, &mut state, &mut deck, &mut is_open);
    });
    output1.textures_delta.clear();
    assert!(state.active_player_media.is_some());

    // Frame 2: video ends, closing transition starts
    if let Some(media) = &mut state.active_player_media {
        media.video_start_time = std::time::Instant::now()
            .checked_sub(std::time::Duration::from_secs(3))
            .unwrap();
    }
    let mut input2 = egui::RawInput::default();
    input2.time = Some(0.1);
    let mut output2 = ctx.run_ui(input2, |ui| {
        render_player_column(ui, &mut state, &mut deck, &mut is_open);
    });
    output2.textures_delta.clear();
    assert!(state.active_player_media.is_some(), "Should be in fade-out transition");

    // Frame 3: transition completes (0.6s later)
    let mut input3 = egui::RawInput::default();
    input3.time = Some(1.0);
    let mut output3 = ctx.run_ui(input3, |ui| {
        render_player_column(ui, &mut state, &mut deck, &mut is_open);
    });
    output3.textures_delta.clear();

    assert!(state.active_player_media.is_none());
    assert!(deck.live_media.is_none());
    assert!(!is_open, "Presentation must be closed when video finishes");
}

#[test]
fn test_hero_play_button_uses_text_on_accent_contrast() {
    let theme_darker_pro = crate::ui::theme::AppTheme::from_definition(
        &crate::core::interface_theme::resolve_builtin("darker_pro").unwrap(),
        true,
    );
    assert_ne!(theme_darker_pro.accent, theme_darker_pro.text_on_accent);
    assert_eq!(theme_darker_pro.text_on_accent, Color32::WHITE);
}


