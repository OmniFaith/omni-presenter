use super::{ChatService, MAX_UPLOAD, SendMessage};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::Read,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

pub fn tls_port(port: u16) -> u16 {
    port.checked_add(1).unwrap_or(65534)
}

pub struct SecureChatServer {
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl SecureChatServer {
    pub fn start(chat: ChatService, port: u16, lan: bool) -> Result<Self, String> {
        let (certificate, private_key) = chat.certificate(&crate::services::web::local_address())?;
        let server = tiny_http::Server::https(
            (if lan { "0.0.0.0" } else { "127.0.0.1" }, port),
            tiny_http::SslConfig {
                certificate: certificate.into_bytes(),
                private_key: private_key.into_bytes(),
            },
        )
        .map_err(|e| e.to_string())?;
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let thread = std::thread::spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                if let Ok(Some(request)) = server.recv_timeout(Duration::from_millis(100)) {
                    let chat = chat.clone();
                    std::thread::spawn(move || serve(request, &chat));
                }
            }
        });
        Ok(Self {
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for SecureChatServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub fn decode(value: &str) -> String {
    let mut bytes = Vec::new();
    let input = value.as_bytes();
    let mut index = 0;
    while index < input.len() {
        if input[index] == b'%'
            && index + 2 < input.len()
            && let Some(hex) = value.get(index + 1..index + 3)
            && let Ok(byte) = u8::from_str_radix(hex, 16)
        {
            bytes.push(byte);
            index += 3;
            continue;
        }
        bytes.push(if input[index] == b'+' {
            b' '
        } else {
            input[index]
        });
        index += 1;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn token(request: &tiny_http::Request) -> String {
    for header in request.headers() {
        if header.field.equiv("Authorization")
            && let Some(token) = header.value.as_str().strip_prefix("Bearer ")
        {
            return token.into();
        }
        if header.field.equiv("Cookie") {
            for part in header.value.as_str().split(';') {
                let part = part.trim();
                if let Some(value) = part
                    .strip_prefix("omni_chat=")
                    .or_else(|| part.strip_prefix("omni_control="))
                {
                    return value.into();
                }
            }
        }
    }
    String::new()
}

fn json_body(request: &mut tiny_http::Request) -> Result<Value, String> {
    let mut bytes = Vec::new();
    request
        .as_reader()
        .take(65537)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 65536 {
        return Err("services-message-body-is-too-large".into());
    }
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

fn reply(
    request: tiny_http::Request,
    status: u16,
    mime: &str,
    body: Vec<u8>,
    extra: Vec<(&str, String)>,
) {
    let mut response = tiny_http::Response::from_data(body).with_status_code(status);
    for (key, value) in [
        ("Content-Type", mime.to_owned()),
        ("Cache-Control", "no-store".into()),
        ("X-Content-Type-Options", "nosniff".into()),
        ("Access-Control-Allow-Origin", "*".into()),
        (
            "Access-Control-Allow-Headers",
            "Authorization, Content-Type".into(),
        ),
        (
            "Access-Control-Allow-Methods",
            "GET, POST, DELETE, OPTIONS".into(),
        ),
    ]
    .into_iter()
    .chain(extra)
    {
        if let Ok(header) = tiny_http::Header::from_bytes(key, value) {
            response.add_header(header);
        }
    }
    let _ = request.respond(response);
}

pub fn serve(mut request: tiny_http::Request, chat: &ChatService) {
    let interface_language = crate::services::i18n::language(&request, None);
    let url = request.url().to_owned();
    let (path, query) = url.split_once('?').unwrap_or((&url, ""));
    let params: HashMap<String, String> = query
        .split('&')
        .filter_map(|p| p.split_once('='))
        .map(|(k, v)| (decode(k), decode(v)))
        .collect();
    let method = request.method().as_str().to_owned();
    if method == "OPTIONS" {
        reply(request, 204, "text/plain", Vec::new(), Vec::new());
        return;
    }
    let asset = match path {
        "/" | "/chat" | "/chat/" => Some((
            "text/html; charset=utf-8",
            include_bytes!("../../../assets/web/chat/chat.html").as_slice(),
        )),
        "/chat.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/chat/chat.js").as_slice(),
        )),
        "/chat.css" => Some((
            "text/css; charset=utf-8",
            include_bytes!("../../../assets/web/chat/chat.css").as_slice(),
        )),
        "/chat-workspace.css" => Some((
            "text/css; charset=utf-8",
            include_bytes!("../../../assets/web/chat/chat-workspace.css").as_slice(),
        )),
        "/chat-picker.css" => Some((
            "text/css; charset=utf-8",
            include_bytes!("../../../assets/web/chat/chat-picker.css").as_slice(),
        )),
        "/chat-cards.css" => Some((
            "text/css; charset=utf-8",
            include_bytes!("../../../assets/web/chat/chat-cards.css").as_slice(),
        )),
        "/chat-icons.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/chat/chat-icons.js").as_slice(),
        )),
        "/chat-emoji.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/chat/chat-emoji.js").as_slice(),
        )),
        "/chat-attachments.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/chat/chat-attachments.js").as_slice(),
        )),
        "/chat-bible.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/chat/chat-bible.js").as_slice(),
        )),
        "/chat-bootstrap.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/chat/chat-bootstrap.js").as_slice(),
        )),
        "/chat-channels.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/chat/chat-channels.js").as_slice(),
        )),
        "/chat-channels.css" => Some((
            "text/css; charset=utf-8",
            include_bytes!("../../../assets/web/chat/chat-channels.css").as_slice(),
        )),
        "/api/chat/emoji-font" => Some((
            "font/ttf",
            include_bytes!("../../../assets/fonts/NotoEmoji.ttf").as_slice(),
        )),
        "/api/chat/icon-font" => Some((
            "font/ttf",
            include_bytes!("../../../assets/fonts/custom-icons.ttf").as_slice(),
        )),
        _ => None,
    };
    if method == "GET"
        && let Some((mime, body)) = asset
    {
        reply(request, 200, mime, body.to_vec(), Vec::new());
        return;
    }
    if method == "GET" && path == "/api/chat/ui-font" {
        let fonts = eframe::egui::FontDefinitions::default();
        if let Some(font) = fonts.families.get(&eframe::egui::FontFamily::Proportional)
            .and_then(|names| names.first()).and_then(|name| fonts.font_data.get(name)) {
            reply(request, 200, "font/ttf", font.bytes().to_vec(), Vec::new());
        } else {
            reply(request, 404, "text/plain", Vec::new(), Vec::new());
        }
        return;
    }
    if method == "GET" && path == "/api/chat/icons" {
        use egui_phosphor::regular::*;
        let icons = json!(std::collections::BTreeMap::from([
            ("chat", "\u{E003}"),
            ("bible", "\u{E002}"),
            ("book", "\u{E002}"),
            ("song", "\u{E001}"),
            ("music", "\u{E001}"),
            ("attach", PLUS),
            ("plus", PLUS),
            ("bell", BELL),
            ("notify", BELL),
            ("menu", LIST),
            ("list", LIST),
            ("search", MAGNIFYING_GLASS),
            ("record", MICROPHONE),
            ("mic", MICROPHONE),
            ("send", PAPER_PLANE_RIGHT),
            ("smile", SMILEY),
            ("play", PLAY),
            ("pause", PAUSE),
            ("stop", STOP),
            ("trash", TRASH),
            ("link", LINK_SIMPLE),
            ("download", DOWNLOAD_SIMPLE),
            ("file", FILE),
            ("image", IMAGE),
            ("video", VIDEO),
            ("playlist", "\u{E2F8}"),
            ("shortcut", HEART),
            ("pin", HEART),
            ("close", X),
            ("edit", PENCIL_SIMPLE),
            ("more", DOTS_SIX_VERTICAL),
        ]));
        reply(request, 200, "application/json", icons.to_string().into_bytes(), Vec::new());
        return;
    }
    if method == "GET" && path == "/api/chat/certificate" {
        match chat.certificate(&crate::services::web::local_address()) {
            Ok((certificate, _)) => reply(
                request,
                200,
                "application/x-pem-file",
                certificate.into_bytes(),
                vec![(
                    "Content-Disposition",
                    "attachment; filename=\"omni-presenter-chat.pem\"".into(),
                )],
            ),
            Err(error) => reply(
                request,
                500,
                "application/json",
                json!({"error":interface_language.message(&error)}).to_string().into_bytes(),
                Vec::new(),
            ),
        }
        return;
    }
    if method == "GET"
        && let Some(profile) = path.strip_prefix("/api/chat/avatars/")
    {
        if let Some(bytes) = chat.avatar(&decode(profile)) {
            let mut response = tiny_http::Response::from_data(bytes);
            for (key, value) in [
                ("Content-Type", "image/jpeg"),
                ("Cache-Control", "public, max-age=3600"),
                ("Access-Control-Allow-Origin", "*"),
                ("X-Content-Type-Options", "nosniff"),
            ] {
                response.add_header(tiny_http::Header::from_bytes(key, value).unwrap());
            }
            let _ = request.respond(response);
        } else {
            reply(request, 404, "text/plain", Vec::new(), Vec::new());
        }
        return;
    }
    let session = token(&request);
    if method == "GET"
        && let Some(id) = path.strip_prefix("/api/chat/files/")
    {
        match chat.download(&session, id) {
            Ok((name, mime, data)) => {
                let inline = matches!(
                    mime.as_str(),
                    "image/png"
                        | "image/jpeg"
                        | "image/gif"
                        | "image/webp"
                        | "audio/webm"
                        | "audio/wav"
                        | "audio/ogg"
                        | "audio/mp4"
                        | "audio/mpeg"
                        | "video/mp4"
                        | "video/webm"
                );
                let disposition = format!(
                    "{}; filename=\"{}\"",
                    if inline { "inline" } else { "attachment" },
                    name.replace(['"', '\\'], "_")
                );
                reply(
                    request,
                    200,
                    &mime,
                    data,
                    vec![("Content-Disposition", disposition)],
                );
            }
            Err(error) => reply(
                request,
                403,
                "application/json",
                json!({"error":interface_language.message(&error)}).to_string().into_bytes(),
                Vec::new(),
            ),
        }
        return;
    }
    let mut cookies = Vec::new();
    let result = (|| -> Result<Value, String> {
        if method == "POST"
            && let Some(id) = path
                .strip_prefix("/api/chat/messages/")
                .and_then(|p| p.strip_suffix("/reactions"))
        {
            let body = json_body(&mut request)?;
            return Ok(json!(chat.react(
                &session,
                id.parse().map_err(|_| "services-invalid-id")?,
                body["emoji"].as_str().unwrap_or("")
            )?));
        }
        if method == "DELETE"
            && let Some(id) = path.strip_prefix("/api/chat/messages/")
        {
            return Ok(json!(chat.delete_message(
                &session,
                id.parse().map_err(|_| "services-invalid-id")?
            )?));
        }
        if method == "POST"
            && let Some(id) = path
                .strip_prefix("/api/chat/messages/")
                .and_then(|p| p.strip_suffix("/actions"))
        {
            let id = id.parse().map_err(|_| "services-invalid-message-id")?;
            let body = json_body(&mut request)?;
            return Ok(json!(chat.request_action(
                &session,
                id,
                body["action"].as_str().unwrap_or("")
            )?));
        }
        if method == "GET"
            && let Some(id) = path.strip_prefix("/api/chat/actions/")
        {
            return Ok(json!(chat.action_status(&session, id)?));
        }
        if method == "GET"
            && let Some(id) = path.strip_prefix("/api/chat/messages/")
        {
            return Ok(json!(chat.message(
                &session,
                id.parse().map_err(|_| "services-invalid-message-id")?
            )?));
        }
        match (method.as_str(), path) {
            ("GET", "/api/chat/translations") => {
                Ok(json!(interface_language.interface_messages()))
            }
            ("GET", "/api/chat/emojis") => {
                let language = crate::core::i18n::Language::from_code(
                    params.get("lang").map(String::as_str).unwrap_or("pt-BR"),
                );
                Ok(json!(crate::core::emoji::catalogue().iter().map(|category| json!({"name":category.label(language),"icon":category.icon,"emojis":category.emojis.iter().map(|emoji| json!({"value":emoji.value,"name":emoji.label(language)})).collect::<Vec<_>>()})).collect::<Vec<_>>()))
            }
            ("GET", "/api/chat/notifications") => Ok(json!(chat.notifications(&session)?)),
            ("POST", "/api/chat/read") => {
                let body = json_body(&mut request)?;
                Ok(json!(chat.mark_read(
                    &session,
                    body["channel_id"].as_str(),
                    body["peer_id"].as_str(),
                    body["upto"].as_u64()
                )?))
            }
            ("GET", "/api/chat/profiles") => Ok(json!(chat.profiles())),
            ("POST", "/api/chat/login") => {
                let body = json_body(&mut request)?;
                let token = chat.login(
                    body["profile_id"].as_str().unwrap_or(""),
                    body["password"].as_str().or_else(|| body["code"].as_str()).unwrap_or(""),
                )?;
                cookies.push(format!(
                    "omni_chat={token}; Path=/; HttpOnly; SameSite=Strict"
                ));
                cookies.push(format!(
                    "omni_control={token}; Path=/; HttpOnly; SameSite=Strict"
                ));
                Ok(json!({"token":token,"profile_id":body["profile_id"]}))
            }
            ("POST", "/api/chat/session") => {
                // Exchange an existing bearer token for a browser cookie.
                let profile_id = chat.session_profile(&session)?;
                cookies.push(format!(
                    "omni_chat={session}; Path=/; HttpOnly; SameSite=Strict"
                ));
                cookies.push(format!(
                    "omni_control={session}; Path=/; HttpOnly; SameSite=Strict"
                ));
                Ok(json!({"ok":true,"profile_id":profile_id}))
            }
            ("POST", "/api/chat/logout") => {
                chat.logout(&session);
                cookies.push("omni_chat=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0".into());
                cookies.push("omni_control=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0".into());
                Ok(json!({"ok":true}))
            }
            ("POST", "/api/chat/heartbeat") => {
                chat.heartbeat(&session)?;
                Ok(json!({"ok":true}))
            }
            ("GET", "/api/chat/state") => {
                let mut snapshot = chat.snapshot(
                    &session,
                    params.get("channel").map(String::as_str),
                    params.get("peer").map(String::as_str),
                    params.get("q").map(String::as_str).unwrap_or(""),
                    params.get("before").and_then(|v| v.parse().ok()),
                )?;
                let language = crate::core::i18n::Language::from_code(
                    params.get("lang").map(String::as_str).unwrap_or("pt-BR"),
                );
                for channel in &mut snapshot.channels {
                    if channel.id == "general" {
                        channel.name = language.text("chat-general").into();
                    }
                }
                Ok(json!(snapshot))
            }
            ("POST", "/api/chat/channels/update") => {
                let body = json_body(&mut request)?;
                chat.update_channel(
                    &session,
                    body["id"].as_str().unwrap_or(""),
                    body["name"].as_str().unwrap_or(""),
                    body["tool"].as_str().unwrap_or("general"),
                    body["description"].as_str().unwrap_or(""),
                )?;
                Ok(json!({"ok":true}))
            }
            ("POST", "/api/chat/channels/reorder") => {
                let body = json_body(&mut request)?;
                let ids: Vec<String> =
                    serde_json::from_value(body["ids"].clone()).map_err(|e| e.to_string())?;
                chat.move_channels(
                    &session,
                    &ids,
                    body["target"].as_str().unwrap_or(""),
                    body["after"].as_bool().unwrap_or(false),
                )?;
                Ok(json!({"ok":true}))
            }
            ("POST", "/api/chat/channels/read") | ("DELETE", "/api/chat/channels") => {
                let body = json_body(&mut request)?;
                let ids: Vec<String> =
                    serde_json::from_value(body["ids"].clone()).map_err(|e| e.to_string())?;
                if method == "DELETE" {
                    chat.delete_channels(&session, &ids)?;
                } else {
                    chat.mark_channels_read(&session, &ids)?;
                }
                Ok(json!({"ok":true}))
            }
            ("POST", "/api/chat/channels") => {
                let body = json_body(&mut request)?;
                Ok(json!(chat.create_channel(
                    &session,
                    body["name"].as_str().unwrap_or(""),
                    body["tool"].as_str().unwrap_or("general"),
                    body["description"].as_str().unwrap_or("")
                )?))
            }
            ("POST", "/api/chat/messages") => {
                let body: SendMessage =
                    serde_json::from_value(json_body(&mut request)?).map_err(|e| e.to_string())?;
                Ok(json!(chat.send(&session, body)?))
            }
            ("GET", "/api/chat/songs") => {
                chat.songs(&session, params.get("q").map(String::as_str).unwrap_or(""))
            }
            ("GET", "/api/chat/versions") => chat.versions(&session),
            ("GET", "/api/chat/bible/search") => chat.bible_search(
                &session,
                params.get("version").map(String::as_str).unwrap_or(""),
                params.get("q").map(String::as_str).unwrap_or(""),
            ),
            ("POST", "/api/chat/uploads") => {
                chat.heartbeat(&session)?;
                if request.body_length().is_some_and(|len| len > MAX_UPLOAD) {
                    return Err("Arquivo maior que 256 MiB".into());
                }
                let mime = request
                    .headers()
                    .iter()
                    .find(|h| h.field.equiv("Content-Type"))
                    .map(|h| h.value.as_str().to_owned())
                    .unwrap_or("application/octet-stream".into());
                let mut data = Vec::new();
                request
                    .as_reader()
                    .take(MAX_UPLOAD as u64 + 1)
                    .read_to_end(&mut data)
                    .map_err(|e| e.to_string())?;
                Ok(json!(chat.upload(
                    &session,
                    params.get("name").map(String::as_str).unwrap_or("arquivo"),
                    &mime,
                    &data
                )?))
            }
            _ => Err("services-endpoint-not-found".into()),
        }
    })();
    match result {
        Ok(value) => reply(
            request,
            200,
            "application/json; charset=utf-8",
            value.to_string().into_bytes(),
            cookies.into_iter().map(|v| ("Set-Cookie", v)).collect(),
        ),
        Err(error) => {
            let status = if error.starts_with("services-session")
                || error.starts_with("services-invalid-profile")
                || error.starts_with("chat-invalid-profile")
                || error.starts_with("Sessão")
                || error.starts_with("Perfil ou código")
                || error.starts_with("Perfil ou senha") {
                401
            } else if error == "services-endpoint-not-found"
                || error.starts_with("Endpoint") {
                404
            } else {
                400
            };
            reply(
                request,
                status,
                "application/json; charset=utf-8",
                json!({"error":interface_language.message(&error)}).to_string().into_bytes(),
                Vec::new(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    fn request(
        chat: &ChatService,
        method: &str,
        path: &str,
        auth: &str,
        mime: &str,
        body: &[u8],
    ) -> (String, Vec<u8>) {
        let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
        let address = server.server_addr().to_ip().unwrap();
        let chat = chat.clone();
        let thread = std::thread::spawn(move || serve(server.recv().unwrap(), &chat));
        let mut socket = std::net::TcpStream::connect(address).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        write!(socket,"{method} {path} HTTP/1.0\r\nHost: localhost\r\nAuthorization: Bearer {auth}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\n\r\n",body.len()).unwrap();
        socket.write_all(body).unwrap();
        let mut response = Vec::new();
        socket.read_to_end(&mut response).unwrap();
        thread.join().unwrap();
        let boundary = response.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
        (
            String::from_utf8_lossy(&response[..boundary]).into_owned(),
            response[boundary + 4..].to_vec(),
        )
    }

    #[test]
    fn api_handles_login_channels_binary_uploads_and_authenticated_downloads() {
        let db = crate::services::db::AppDb::open_in_memory().unwrap();
        let profile = crate::core::preferences::OperatorProfile::default();
        let chat = ChatService::new(db, std::slice::from_ref(&profile)).unwrap();
        let login = json!({"profile_id":profile.id,"code":chat.access_code(&profile.id).unwrap()})
            .to_string();
        let (headers, body) = request(
            &chat,
            "POST",
            "/api/chat/login",
            "",
            "application/json",
            login.as_bytes(),
        );
        assert!(headers.starts_with("HTTP/1.0 200"));
        assert!(headers.to_lowercase().contains("httponly"));
        let token = serde_json::from_slice::<Value>(&body).unwrap()["token"]
            .as_str()
            .unwrap()
            .to_owned();
        let (headers, _) = request(
            &chat,
            "GET",
            "/api/chat/state?channel=general",
            "invalid",
            "application/json",
            b"",
        );
        assert!(headers.starts_with("HTTP/1.0 401"));
        let (headers, body) = request(
            &chat,
            "POST",
            "/api/chat/channels",
            &token,
            "application/json",
            br#"{"name":"som","tool":"media"}"#,
        );
        assert!(headers.starts_with("HTTP/1.0 200"));
        let channel: Value = serde_json::from_slice(&body).unwrap();
        let binary = [0, 255, 0, 127, 42];
        let (headers, body) = request(
            &chat,
            "POST",
            "/api/chat/uploads?name=arquivo%20da%20equipe.bin",
            &token,
            "application/octet-stream",
            &binary,
        );
        assert!(headers.starts_with("HTTP/1.0 200"));
        let file: Value = serde_json::from_slice(&body).unwrap();
        let message =
            json!({"channel_id":channel["id"],"kind":"file","file_id":file["id"]}).to_string();
        assert!(
            request(
                &chat,
                "POST",
                "/api/chat/messages",
                &token,
                "application/json",
                message.as_bytes()
            )
            .0
            .starts_with("HTTP/1.0 200")
        );
        let path = format!("/api/chat/files/{}", file["id"].as_str().unwrap());
        let (headers, body) = request(&chat, "GET", &path, &token, "", b"");
        assert_eq!(body, binary);
        assert!(headers.contains("attachment;"));
        assert!(
            request(&chat, "GET", &path, "", "", b"")
                .0
                .starts_with("HTTP/1.0 403")
        );
        assert!(
            request(&chat, "GET", "/chat", "", "", b"")
                .0
                .starts_with("HTTP/1.0 200")
        );
        for (path, content_type) in [
            ("/chat.js", "application/javascript"),
            ("/chat-icons.js", "application/javascript"),
            ("/chat-emoji.js", "application/javascript"),
            ("/chat-attachments.js", "application/javascript"),
            ("/chat-bible.js", "application/javascript"),
            ("/chat-bootstrap.js", "application/javascript"),
            ("/chat-channels.js", "application/javascript"),
            ("/chat.css", "text/css"),
            ("/chat-workspace.css", "text/css"),
            ("/chat-picker.css", "text/css"),
            ("/chat-cards.css", "text/css"),
            ("/chat-channels.css", "text/css"),
        ] {
            let (status, body) = request(&chat, "GET", path, "", "", b"");
            assert!(status.starts_with("HTTP/1.0 200"), "{path} must be served");
            assert!(!body.is_empty(), "{path} must not be empty");
            assert!(
                status.contains(content_type),
                "{path} must use {content_type}"
            );
        }
        let (_, html) = request(&chat, "GET", "/chat", "", "", b"");
        let html = String::from_utf8(html).unwrap();
        for asset in [
            "chat-icons.js",
            "chat-emoji.js",
            "chat-attachments.js",
            "chat-bible.js",
            "chat-bootstrap.js",
            "chat-channels.js",
            "chat.js",
            "chat-workspace.css",
            "chat-picker.css",
            "chat-cards.css",
            "chat-channels.css",
        ] {
            assert!(html.contains(asset), "chat.html must load {asset}");
        }
        assert!(!html.contains("/api/chat/docs"));
    }

    #[test]
    fn percent_decoding_accepts_unicode_and_malformed_input_without_panicking() {
        assert_eq!(decode("Salmos+119%3A1%2D7"), "Salmos 119:1-7");
        assert_eq!(decode("Ora%C3%A7%C3%A3o"), "Oração");
        assert_eq!(decode("%€"), "%€");
        assert_eq!(decode("%ZZ"), "%ZZ");
    }
}
