use super::ChatService;
use crate::{
    core::preferences::{AppSettings, OperatorProfile},
    services::{db::AppDb, web::BrowserServer},
};

#[test]
#[ignore = "Requires Playwright and Chromium; run with NODE_PATH pointing to Playwright"]
fn browser_can_load_profiles_share_lyrics_and_search_bible_like_the_app() {
    let db = AppDb::open_in_memory().unwrap();
    db.add_song(
        "Canção de Esperança",
        "Equipe",
        "Graça e paz para todos\n\nCantamos juntos",
    )
    .unwrap();
    let profiles = vec![
        OperatorProfile::default(),
        OperatorProfile {
            id: "sound".into(),
            name: "Mesa de som".into(),
            ..Default::default()
        },
    ];
    let chat = ChatService::new(db, &profiles).unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let settings = AppSettings {
        server_port: port,
        allow_lan: false,
        ..Default::default()
    };
    let _server = BrowserServer::start_with_chat(&settings, Some(chat.clone())).unwrap();
    let output = std::process::Command::new("node")
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/chat-web.cjs"))
        .env("CHAT_URL", format!("http://127.0.0.1:{port}"))
        .env("CHAT_PROFILE", &profiles[0].id)
        .env("CHAT_CODE", chat.access_code(&profiles[0].id).unwrap())
        .output()
        .expect("Node is required for the browser test");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
