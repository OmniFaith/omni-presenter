use super::WebControl;
use crate::{
    core::preferences::OperatorProfile,
    services::{chat::ChatService, credentials, db::AppDb},
};
use std::io::{Read, Write};

fn request(control: &WebControl, method: &str, path: &str, cookie: &str, body: &str) -> String {
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let address = server.server_addr().to_ip().unwrap();
    let control = control.clone();
    let path_owned = path.to_owned();
    let worker = std::thread::spawn(move || {
        let request = server.recv().unwrap();
        if path_owned.contains("__control_") || path_owned.starts_with("/control-preview/") {
            super::serve_preview(request, &control);
        } else {
            super::serve(request, &control);
        }
    });
    let mut socket = std::net::TcpStream::connect(address).unwrap();
    socket
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .unwrap();
    write!(socket, "{method} {path} HTTP/1.0\r\nHost: localhost\r\nCookie: {cookie}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}", body.len()).unwrap();
    let mut response = String::new();
    socket.read_to_string(&mut response).unwrap();
    worker.join().unwrap();
    response
}

#[test]
fn password_login_protects_control_and_previews_and_password_changes_revoke_sessions() {
    let db = AppDb::open_in_memory().unwrap();
    let mut profile = OperatorProfile::default();
    let chat = ChatService::new(db.clone(), &[profile.clone()]).unwrap();
    let control = WebControl::new(db);
    control.set_auth(Some(chat.clone()));
    for path in [
        "/api/control/state",
        "/api/control/library",
        "/api/control/background?id=x",
        "/api/state/__control_preview",
        "/screen/__control_live",
        "/control-preview/video",
    ] {
        assert!(
            request(&control, "GET", path, "", "").starts_with("HTTP/1.0 401"),
            "{path}"
        );
    }
    assert!(
        request(
            &control,
            "POST",
            "/api/control/command",
            "",
            r#"{"action":"stop"}"#
        )
        .starts_with("HTTP/1.0 401")
    );
    assert!(control.drain().is_empty());
    let public = request(&control, "GET", "/api/control/profiles", "", "");
    assert!(public.contains(&profile.name));
    assert!(!public.contains("password_hash"));
    let login =
        |password| serde_json::json!({"profile_id":profile.id,"password":password}).to_string();
    assert!(
        request(&control, "POST", "/api/control/login", "", &login("wrong"))
            .starts_with("HTTP/1.0 401")
    );
    let response = request(
        &control,
        "POST",
        "/api/control/login",
        "",
        &login("omni-presenter"),
    );
    assert!(response.starts_with("HTTP/1.0 200"));
    let cookie = response
        .lines()
        .find_map(|line| line.strip_prefix("Set-Cookie: "))
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    assert!(request(&control, "GET", "/api/control/state", cookie, "").starts_with("HTTP/1.0 200"));

    let cookies: Vec<&str> = response
        .lines()
        .filter_map(|line| line.strip_prefix("Set-Cookie: "))
        .map(|c| c.split(';').next().unwrap())
        .collect();
    assert!(cookies.iter().any(|c| c.starts_with("omni_control=")));
    assert!(cookies.iter().any(|c| c.starts_with("omni_chat=")));
    let chat_cookie = cookies.iter().find(|c| c.starts_with("omni_chat=")).unwrap();
    assert!(request(&control, "GET", "/api/control/state", chat_cookie, "").starts_with("HTTP/1.0 200"));
    profile.web_password_hash = credentials::hash("uma senha personalizada").unwrap();
    chat.sync_profiles(&[profile.clone()]).unwrap();
    assert!(request(&control, "GET", "/api/control/state", cookie, "").starts_with("HTTP/1.0 401"));
    assert!(chat.login(&profile.id, "omni-presenter").is_err());
    let token = chat.login(&profile.id, "uma senha personalizada").unwrap();
    assert!(chat.heartbeat(&token).is_ok());
    let response = request(
        &control,
        "POST",
        "/api/control/login",
        "",
        &login("uma senha personalizada"),
    );
    let cookie = response
        .lines()
        .find_map(|line| line.strip_prefix("Set-Cookie: "))
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    assert!(
        request(&control, "POST", "/api/control/logout", cookie, "{}").starts_with("HTTP/1.0 200")
    );
    assert!(request(&control, "GET", "/api/control/state", cookie, "").starts_with("HTTP/1.0 401"));
    // Persisted credentials work after restart without storing the plaintext password.
    let serialized = serde_json::to_string(&profile).unwrap();
    assert!(!serialized.contains("uma senha personalizada"));
    let profile = serde_json::from_str::<OperatorProfile>(&serialized).unwrap();
    let restored = ChatService::new(AppDb::open_in_memory().unwrap(), &[profile.clone()]).unwrap();
    assert!(
        restored
            .login(&profile.id, "uma senha personalizada")
            .is_ok()
    );
}
