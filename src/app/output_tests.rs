use super::PresentationApp;
use crate::{
    core::{
        models::PresentationTheme,
        preferences::{AppSettings, PhysicalBackground},
    },
    services::{db::AppDb, monitor::MonitorInfo},
};
use eframe::{
    App,
    egui::{self, Color32},
};
use std::sync::{Arc, Mutex};

#[test]
fn physical_windows_reuse_ids_and_only_show_wallpaper_in_standby() {
    let db = AppDb::open_in_memory().unwrap();
    db.save_settings(&AppSettings {
        server_enabled: false,
        chat_enabled: false,
        ..Default::default()
    })
    .unwrap();
    let mut app = PresentationApp::with_db(db.clone());
    app.monitors = ["Projector", "Stage"]
        .iter()
        .enumerate()
        .map(|(index, name)| MonitorInfo {
            winit_index: index,
            name: name.to_string(),
            display_name: name.to_string(),
            resolution: (1280, 720),
            position: (index as i32 * 1280, 0),
            scale_factor: 1.0,
        })
        .collect();
    app.control_state.settings.monitor_name = "Projector".into();
    app.control_state.settings.physical_transition_ms = 0;
    let live_color = Color32::from_rgb(180, 40, 20);
    let wallpaper_color = Color32::from_rgb(20, 40, 180);
    app.control_state.themes = vec![PresentationTheme {
        id: "live".into(),
        bg_type: "color".into(),
        bg_top: live_color,
        bg_bottom: live_color,
        is_gradient: false,
        ..Default::default()
    }];
    app.control_state.live_theme_id = "live".into();
    app.control_state.live_theme_index = 0;
    app.control_state.settings.physical_background =
        PhysicalBackground::Color { rgb: [20, 40, 180] };
    let song = db.add_song("Song", "", "Lyrics").unwrap();
    app.slide_deck = crate::core::slides::SlideDeck::from_db(&db);
    app.apply_web_command(crate::services::control::ControlCommand::Select {
        kind: crate::services::control::ResourceKind::Song,
        id: song.to_string(),
        project: false,
    })
    .unwrap();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let frames = captured.clone();
    egui::Context::set_immediate_viewport_renderer(move |ctx, mut viewport| {
        let mut raw = egui::RawInput {
            viewport_id: viewport.ids.this,
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 720.0),
            )),
            ..Default::default()
        };
        raw.viewports.insert(
            viewport.ids.this,
            egui::ViewportInfo {
                parent: Some(viewport.ids.parent),
                ..Default::default()
            },
        );
        let mut out = ctx.run_ui(raw, |ui| (viewport.viewport_ui_cb)(ui));
        out.textures_delta.clear();
        let colors: Vec<_> = out
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Rect(rect) if rect.fill.a() > 0 => Some(rect.fill),
                _ => None,
            })
            .collect();
        frames
            .lock()
            .unwrap()
            .push((viewport.ids.this, viewport.builder, colors));
    });
    let ctx = egui::Context::default();
    let overlay = crate::services::windowing::projection_window_mode()
        == crate::services::windowing::ProjectionWindowMode::Overlay;
    ctx.set_embed_viewports(false);
    let draw = |app: &mut PresentationApp| {
        captured.lock().unwrap().clear();
        let input = egui::RawInput {
            time: Some(ctx.input(|input| input.time) + 0.1),
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| app.draw_presentation_viewport(ui.ctx()));
        output.textures_delta.clear();
        captured.lock().unwrap().clone()
    };
    let standby = draw(&mut app);
    assert_eq!(standby.len(), 2);
    assert_ne!(standby[0].0, standby[1].0);
    assert!(
        standby
            .iter()
            .all(|(_, builder, colors)| builder.transparent == Some(true)
                && colors.contains(&wallpaper_color)
                && !colors.contains(&live_color))
    );
    app.is_presentation_open = true;
    let live = draw(&mut app);
    if overlay {
        for (id, builder, _) in &live {
            let monitor = app
                .monitors
                .iter()
                .find(|monitor| crate::ui::presentation::viewport_id(&monitor.name) == *id)
                .unwrap();
            let expected = monitor.viewport_rect(ctx.zoom_factor());
            assert_eq!(builder.position, Some(expected.min));
            assert_eq!(builder.inner_size, Some(expected.size()));
            assert_eq!(builder.fullscreen, Some(false));
            assert_eq!(builder.monitor, None);
        }
    }
    assert_eq!(
        standby.iter().map(|frame| frame.0).collect::<Vec<_>>(),
        live.iter().map(|frame| frame.0).collect::<Vec<_>>()
    );
    assert!(
        live.iter().all(
            |(_, _, colors)| colors.contains(&live_color) && !colors.contains(&wallpaper_color)
        )
    );
    app.is_presentation_open = false;
    assert!(
        draw(&mut app)
            .iter()
            .all(|(_, _, colors)| colors.contains(&wallpaper_color))
    );
    app.control_state.settings.physical_background = PhysicalBackground::Transparent;
    assert!(
        draw(&mut app)
            .iter()
            .all(|(_, builder, colors)| colors.is_empty()
                && builder.fullscreen == Some(false)
                && builder.visible == Some(overlay)),
        "Transparent standby must not paint an opaque black rectangle"
    );
    assert_eq!(
        crate::native_options().viewport.transparent,
        Some(true),
        "eframe enables alpha-capable backbuffers using the ROOT flag"
    );
    assert_eq!(app.clear_color(&egui::Visuals::dark()), [0.0; 4]);
    app.control_state.settings.persistent_physical_output = false;
    assert!(draw(&mut app).is_empty());
    app.is_presentation_open = true;
    assert_eq!(draw(&mut app).len(), 2);
    app.control_state.settings.toggle_physical_monitor("Stage");
    assert_eq!(draw(&mut app).len(), 1);
    app.control_state
        .settings
        .toggle_physical_monitor("Projector");
    assert!(draw(&mut app).is_empty());
    app.control_state.settings.toggle_physical_monitor("Stage");
    let only_stage = draw(&mut app);
    assert_eq!(only_stage.len(), 1);
    assert_eq!(
        only_stage[0].0,
        crate::ui::presentation::viewport_id("Stage")
    );

    // Overlays crossfade to the desktop without changing native window mode.
    // The pure-Wayland fallback keeps its live background opaque instead.
    app.control_state.settings.persistent_physical_output = true;
    app.control_state.settings.physical_transition_ms = 350;
    app.is_presentation_open = false;
    let mut previous_alpha = 255;
    for frame in 0..3 {
        let exiting = draw(&mut app);
        if overlay {
            assert_eq!(exiting[0].1.fullscreen, Some(false));
            assert_eq!(exiting[0].1.monitor, None);
            let alpha = exiting[0]
                .2
                .iter()
                .map(|color| color.a())
                .max()
                .unwrap_or(0);
            if frame > 0 {
                assert!(alpha < previous_alpha);
            }
            assert!(!exiting[0].2.contains(&Color32::BLACK));
            previous_alpha = alpha;
        } else {
            assert!(exiting[0].2.contains(&live_color));
        }
    }
    draw(&mut app);
    let idle = draw(&mut app);
    assert_eq!(idle[0].1.fullscreen, Some(false));
    assert!(idle[0].2.is_empty());
    app.is_presentation_open = true;
    let mut previous_alpha = 0;
    for frame in 0..4 {
        let entering = draw(&mut app);
        if overlay {
            assert_eq!(entering[0].1.fullscreen, Some(false));
            assert_eq!(entering[0].1.monitor, None);
            let alpha = entering[0]
                .2
                .iter()
                .map(|color| color.a())
                .max()
                .unwrap_or(0);
            if frame > 0 {
                assert!(alpha > previous_alpha);
            }
            assert!(!entering[0].2.contains(&Color32::BLACK));
            previous_alpha = alpha;
        } else {
            assert!(entering[0].2.contains(&live_color));
        }
    }

    // Designs follow the same window lifecycle and use their own background.
    let design_color = Color32::from_rgb(30, 130, 80);
    app.slide_deck.live_design = Some(crate::core::design::DesignDocument {
        background_mode: "color".into(),
        background: [30, 130, 80, 255],
        ..Default::default()
    });
    app.physical_transition = Default::default();
    for _ in 0..5 {
        let design = draw(&mut app);
        if !overlay {
            assert!(design[0].2.contains(&design_color));
        }
        if overlay {
            assert_eq!(design[0].1.fullscreen, Some(false));
        }
        assert!(!design[0].2.contains(&live_color));
    }
    if overlay {
        let rendered =
            crate::render::design::frame(app.slide_deck.live_design.as_ref().unwrap(), 160);
        assert_eq!(
            *rendered.image.get_pixel(0, 0),
            image::Rgba([30, 130, 80, 255])
        );
    }

    // An uncached image must finish decoding before projection begins.
    let path = std::env::temp_dir().join(format!(
        "omni-presenter-first-frame-{}.png",
        crate::core::design::new_id()
    ));
    image::RgbaImage::from_pixel(80, 60, image::Rgba([120, 80, 30, 255]))
        .save(&path)
        .unwrap();
    app.slide_deck.live_design = None;
    app.control_state.themes[0].bg_type = "image".into();
    app.control_state.themes[0].bg_image_path = path.to_string_lossy().into_owned();
    app.physical_transition = Default::default();
    let waiting = draw(&mut app);
    assert_eq!(waiting[0].1.fullscreen, Some(false));
    assert_eq!(waiting[0].1.mouse_passthrough, Some(true));
    assert!(waiting[0].2.is_empty());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if draw(&mut app)[0].1.mouse_passthrough == Some(false) {
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    std::fs::remove_file(path).unwrap();
    app.control_state.settings.physical_output_enabled = false;
    assert!(draw(&mut app).is_empty());
}

#[test]
fn test_song_title_and_artist_projected_only_on_first_slide() {
    use crate::core::slides::SlideDeck;
    let mut deck = SlideDeck::default();
    deck.add_or_update_song(
        "Porque Ele Vive".into(),
        "Harpa Cristã".into(),
        "Deus enviou seu Filho amado\n\nPorque Ele vive posso crer no amanhã".into(),
    );
    deck.project_preview();
    assert!(deck.is_live_song());
    assert_eq!(deck.live_index(), 0);

    // After moving to second slide
    assert!(deck.total_live_slides() > 1);
    deck.select_live_slide(1);
    assert_eq!(deck.live_index(), 1);

    // Custom content like bible is not a song
    deck.load_custom_content("Gênesis 1".into(), "NVI".into(), "No princípio...".into());
    deck.project_preview();
    assert!(!deck.is_live_song());
}

#[test]
fn test_draw_toast_renders_and_dismisses() {
    let db = AppDb::open_in_memory().unwrap();
    let mut app = PresentationApp::with_db(db);
    let ctx = egui::Context::default();
    let theme = crate::ui::AppTheme::default();

    // No toast initially
    app.draw_toast(&ctx, &theme);
    assert!(app.control_state.toast.is_none());

    // Trigger toast
    app.control_state.notify("Test Notification");
    assert!(app.control_state.toast.is_some());

    // Draw toast with active message
    let mut out1 = ctx.run_ui(Default::default(), |ui| {
        app.draw_toast(ui.ctx(), &theme);
    });
    out1.textures_delta.clear();
    assert!(app.control_state.toast.is_some());

    // Expire toast
    if let Some((msg, _)) = app.control_state.toast.take() {
        app.control_state.toast = Some((msg, std::time::Instant::now() - std::time::Duration::from_secs(4)));
    }
    let mut out2 = ctx.run_ui(Default::default(), |ui| {
        app.draw_toast(ui.ctx(), &theme);
    });
    out2.textures_delta.clear();
    assert!(app.control_state.toast.is_none());
}

