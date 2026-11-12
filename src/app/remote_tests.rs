use super::PresentationApp;
use crate::{
    core::{models::PlaylistItemKind, preferences::AppSettings},
    services::{
        control::{ControlCommand as Command, ResourceKind},
        db::AppDb,
    },
};

fn fixture() -> (PresentationApp, i64, i64) {
    let db = AppDb::open_in_memory().unwrap();
    let first = db
        .add_song(
            "Canção de Esperança",
            "Equipe",
            "Primeira estrofe\n\nSegunda estrofe\n\nÚltima estrofe",
        )
        .unwrap();
    let second = db
        .add_song("Próxima música", "Banda", "Outro começo\n\nOutro final")
        .unwrap();
    db.save_settings(&AppSettings {
        server_enabled: false,
        chat_enabled: false,
        physical_output_enabled: false,
        ..Default::default()
    })
    .unwrap();
    (PresentationApp::with_db(db), first, second)
}

#[test]
fn web_layout_is_the_native_layout_and_rejects_missing_panels() {
    let (mut app, _, _) = fixture();
    let mut layout = app.control_state.layout.clone();
    if let crate::core::layout::LayoutNode::Split { fraction, .. } = &mut layout.root {
        *fraction = 0.31;
    }
    app.apply_web_command(Command::WorkspaceLayout {
        bible: false,
        layout: layout.clone(),
    })
    .unwrap();
    assert_eq!(app.control_state.layout, layout);
    assert_eq!(
        app.db.load_settings().unwrap().unwrap().operator().layout,
        layout
    );
    let invalid = crate::core::layout::LayoutTree {
        root: crate::core::layout::LayoutNode::Leaf(crate::core::layout::ContainerKind::Selection),
    };
    assert!(
        app.apply_web_command(Command::WorkspaceLayout {
            bible: false,
            layout: invalid
        })
        .is_err()
    );
    assert_eq!(app.control_state.layout, layout);
}

#[test]
fn controller_preview_and_live_use_independent_real_viewer_outputs() {
    let (mut app, first, second) = fixture();
    app.apply_web_command(Command::Select {
        kind: ResourceKind::Song,
        id: first.to_string(),
        project: true,
    })
    .unwrap();
    app.apply_web_command(Command::Select {
        kind: ResourceKind::Song,
        id: second.to_string(),
        project: false,
    })
    .unwrap();
    let preview: serde_json::Value = serde_json::from_slice(
        &app.web_control
            .preview
            .lock()
            .unwrap()
            .preview_response("/api/state/__control_preview")
            .2,
    )
    .unwrap();
    let live: serde_json::Value = serde_json::from_slice(
        &app.web_control
            .live
            .lock()
            .unwrap()
            .preview_response("/api/state/__control_live")
            .2,
    )
    .unwrap();
    assert_eq!(preview["text"], "Próxima música\nBanda");
    assert_eq!(live["text"], "Canção de Esperança\nEquipe");
    assert!(
        preview["fontUrl"]
            .as_str()
            .unwrap()
            .starts_with("/control-preview/font?")
    );
    assert!(
        live["fontUrl"]
            .as_str()
            .unwrap()
            .starts_with("/control-live/font?")
    );
    assert_eq!(app.slide_deck.live_title(), "Canção de Esperança");
}

#[test]
fn single_bible_verse_keeps_the_same_chapter_navigation_as_the_native_app() {
    let (mut app, _, _) = fixture();
    app.apply_web_command(Command::Bible {
        reference: "João 3:16".into(),
        version: "ARA".into(),
        merge: false,
        project: true,
        playlist: false,
    })
    .unwrap();
    assert_eq!(app.slide_deck.live_title(), "João 3");
    assert_eq!(app.slide_deck.live_index(), 15);
    assert!(app.slide_deck.live_slides().len() > 16);
    app.apply_web_command(Command::Navigate {
        direction: "next".into(),
    })
    .unwrap();
    assert!(app.slide_deck.live_slide_text().ends_with("João 3.17"));
}

#[test]
fn remote_preview_does_not_replace_live_and_commands_update_the_actual_deck() {
    let (mut app, first, second) = fixture();
    app.apply_web_command(Command::Select {
        kind: ResourceKind::Song,
        id: first.to_string(),
        project: true,
    })
    .unwrap();
    app.apply_web_command(Command::Navigate {
        direction: "next".into(),
    })
    .unwrap();
    app.apply_web_command(Command::Select {
        kind: ResourceKind::Song,
        id: second.to_string(),
        project: false,
    })
    .unwrap();
    assert_eq!(app.slide_deck.live_title(), "Canção de Esperança");
    assert_eq!(app.slide_deck.live_index(), 1);
    assert_eq!(app.slide_deck.title(), "Próxima música");
    app.apply_web_command(Command::ProjectPreview { index: Some(2) })
        .unwrap();
    assert_eq!(app.slide_deck.live_slide_text(), "Outro final");
    app.apply_web_command(Command::Visibility { hidden: true })
        .unwrap();
    app.apply_web_command(Command::Visibility { hidden: true })
        .unwrap();
    assert!(app.slide_deck.is_content_hidden());
    assert_eq!(app.slide_deck.projected_text(), "");
    assert!(
        app.apply_web_command(Command::LiveSlide { index: 50 })
            .is_err()
    );
    assert_eq!(app.slide_deck.live_index(), 2);
    app.apply_web_command(Command::Stop).unwrap();
    assert!(!app.is_presentation_open);
}

#[test]
fn remote_playlist_reordering_pins_and_status_survive_settings_roundtrip() {
    let (mut app, first, second) = fixture();
    for id in [first, second] {
        app.apply_web_command(Command::PlaylistAdd {
            kind: ResourceKind::Song,
            id: id.to_string(),
        })
        .unwrap();
    }
    let a = app.control_state.playlist[0].id.clone();
    let b = app.control_state.playlist[1].id.clone();
    app.apply_web_command(Command::PlaylistSelect {
        id: b.clone(),
        project: true,
    })
    .unwrap();
    app.apply_web_command(Command::PlaylistMove {
        id: b.clone(),
        target: a.clone(),
        after: false,
    })
    .unwrap();
    assert_eq!(app.control_state.playlist[0].id, b);
    assert_eq!(app.control_state.selected_playlist_index, Some(0));
    assert!(app.control_state.playlist[0].played);
    for _ in 0..2 {
        app.apply_web_command(Command::Pin {
            kind: ResourceKind::Song,
            id: first.to_string(),
        })
        .unwrap();
    }
    assert_eq!(app.control_state.quick_bar.items.len(), 1);
    app.apply_web_command(Command::PlaylistRemove { id: a })
        .unwrap();
    assert_eq!(
        app.db
            .load_settings()
            .unwrap()
            .unwrap()
            .service()
            .playlist
            .len(),
        1
    );
    assert_eq!(
        app.db
            .load_settings()
            .unwrap()
            .unwrap()
            .operator()
            .quick_items
            .len(),
        1
    );
    let snapshot = app.web_control.snapshot();
    assert_eq!(snapshot["playlist"][0]["id"], b);
    assert!(snapshot["player"].is_null());
}

#[test]
fn remote_bible_ranges_notices_and_song_edits_use_real_content_and_keep_live_independent() {
    let (mut app, first, _) = fixture();
    app.apply_web_command(Command::Select {
        kind: ResourceKind::Song,
        id: first.to_string(),
        project: true,
    })
    .unwrap();
    app.apply_web_command(Command::Bible {
        reference: "Jo 3:16-18".into(),
        version: "ARA".into(),
        merge: false,
        project: false,
        playlist: false,
    })
    .unwrap();
    assert_eq!(app.slide_deck.slides().len(), 3);
    let first_verse = crate::core::slides::strip_directive_commands(&app.slide_deck.slides()[0]);
    assert!(first_verse.starts_with("[16] “"));
    assert!(!app.slide_deck.slides()[0].contains("//ref-pos:"));
    assert!(app.slide_deck.slides()[2].contains("João 3.16–18"));
    assert_eq!(app.slide_deck.live_title(), "Canção de Esperança");
    app.apply_web_command(Command::Bible {
        reference: "Sl119:01-07".into(),
        version: "ARA".into(),
        merge: true,
        project: false,
        playlist: true,
    })
    .unwrap();
    assert_eq!(app.control_state.playlist[0].kind, PlaylistItemKind::Bible);
    // This endpoint stores the canonical BibleTranslation passage, not the view-formatted reference.
    assert!(
        app.control_state.playlist[0]
            .content
            .contains("Salmos 119:1–7")
    );
    app.apply_web_command(Command::Text {
        title: "Aviso".into(),
        content: "Primeiro aviso\n\nSegundo aviso".into(),
        project: true,
        playlist: false,
    })
    .unwrap();
    assert_eq!(app.slide_deck.live_slides().len(), 2);
    assert_eq!(app.slide_deck.live_title(), "Aviso");
    app.apply_web_command(Command::SaveSong {
        id: Some(first),
        title: "Título editado".into(),
        artist: "Equipe".into(),
        content: "Letra nova".into(),
    })
    .unwrap();
    assert_eq!(app.slide_deck.title(), "Título editado");
    assert_eq!(app.slide_deck.live_title(), "Aviso");
    assert!(
        app.db
            .get_all_songs()
            .unwrap()
            .iter()
            .any(|song| song.id == Some(first) && song.text == "Letra nova")
    );
}

#[test]
fn remote_designs_and_countdowns_use_live_library_documents() {
    let (mut app, _, _) = fixture();
    let mut scene = crate::core::design::DesignDocument::template(2);
    scene.name = "Contagem da reunião".into();
    let id = scene.id.clone();
    app.db.save_design(&scene).unwrap();
    app.apply_web_command(Command::Select {
        kind: ResourceKind::Design,
        id,
        project: true,
    })
    .unwrap();
    assert_eq!(
        app.slide_deck.live_design.as_ref().unwrap().name,
        "Contagem da reunião"
    );
    app.apply_web_command(Command::Timer {
        operation: "pause".into(),
    })
    .unwrap();
    assert_eq!(
        app.web_control.snapshot()["timers"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(
        app.web_control.snapshot()["timers"]
            .as_array()
            .unwrap()
            .iter()
            .all(|timer| timer["running"] == false)
    );
    app.apply_web_command(Command::Timer {
        operation: "start".into(),
    })
    .unwrap();
    assert!(
        app.apply_web_command(Command::Select {
            kind: ResourceKind::Song,
            id: "missing".into(),
            project: true
        })
        .is_err()
    );
}

#[test]
#[ignore = "Requires Playwright and Chromium; run with NODE_PATH pointing to Playwright"]
fn browser_operates_the_real_app_from_web_control() {
    use std::{
        process::{Command as Process, Stdio},
        time::{Duration, Instant},
    };
    let db = AppDb::open_in_memory().unwrap();
    db.add_song(
        "Canção de Esperança",
        "Equipe",
        "Graça e paz\n\nCantamos juntos\n\nÚltima estrofe",
    )
    .unwrap();
    db.add_song("Próxima música", "Banda", "Outro começo\n\nOutro final")
        .unwrap();
    db.add_custom_text("Bem-vindos", "É bom estarmos juntos", "Avisos")
        .unwrap();
    let image_path = std::env::temp_dir().join(format!(
        "{}.png",
        crate::core::preferences::new_id("control-test")
    ));
    image::RgbaImage::from_pixel(8, 8, image::Rgba([60, 100, 180, 255]))
        .save(&image_path)
        .unwrap();
    db.add_media_item(
        "Imagem do encontro",
        &image_path.to_string_lossy(),
        "image",
        "",
    )
    .unwrap();
    let mut scene = crate::core::design::DesignDocument::template(2);
    scene.name = "Cronômetro".into();
    db.save_design(&scene).unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    db.save_settings(&AppSettings {
        server_port: port,
        allow_lan: false,
        server_enabled: true,
        chat_enabled: false,
        physical_output_enabled: false,
        operators: vec![crate::core::preferences::OperatorProfile {
            dark_mode: true,
            scale: 120,
            ..Default::default()
        }],
        ..Default::default()
    })
    .unwrap();
    let mut app = PresentationApp::with_db(db);
    let ctx = eframe::egui::Context::default();
    app.publish_web_control();
    let mut child = Process::new("node")
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/control-web.cjs"))
        .env("CONTROL_URL", format!("http://127.0.0.1:{port}"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let started = Instant::now();
    loop {
        app.sync_web_control(&ctx);
        app.publish_web_control();
        app.publish_browser(&ctx);
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if started.elapsed() > Duration::from_secs(90) {
            child.kill().unwrap();
            panic!("Browser control test timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    let _ = std::fs::remove_file(image_path);
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
