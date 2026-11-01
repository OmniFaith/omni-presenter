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


