use super::{ControlCommand, ResourceKind, WebControl};
use serde_json::{Value, json};
use std::{collections::HashMap, io::Read};

fn decode(value: &str) -> String {
    let value = value.replace('+', " ");
    let mut bytes = Vec::new();
    let mut index = 0;
    while index < value.len() {
        if value.as_bytes()[index] == b'%' && index + 2 < value.len()
            && let Some(byte) = value
                .as_bytes()
                .get(index + 1..index + 3)
                .and_then(|bytes| std::str::from_utf8(bytes).ok())
                .and_then(|hex| u8::from_str_radix(hex, 16).ok())
            {
                bytes.push(byte);
                index += 3;
                continue;
            }
        bytes.push(value.as_bytes()[index]);
        index += 1;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

pub fn web_translations(language: crate::core::i18n::Language) -> serde_json::Value {
    let raw = language.interface_messages();
    let mut messages = serde_json::Map::with_capacity(raw.len() * 5);
    for (key, val) in raw {
        let val_json = serde_json::Value::String(val);
        messages.insert(key.clone(), val_json.clone());
        if let Some(rest) = key.strip_prefix("control-") {
            let underscore = rest.replace('-', "_");
            messages.insert(format!("control.{underscore}"), val_json.clone());
            messages.insert(format!("control.{rest}"), val_json.clone());
            messages.insert(underscore, val_json.clone());
            messages.insert(rest.to_string(), val_json);
        } else if let Some(rest) = key.strip_prefix("chat-") {
            let underscore = rest.replace('-', "_");
            messages.insert(format!("chat.{underscore}"), val_json.clone());
            messages.insert(format!("chat.{rest}"), val_json.clone());
            if let Some(stage) = rest.strip_prefix("bible-stage-") {
                messages.insert(format!("chat.bible_stage.{stage}"), val_json.clone());
                messages.insert(format!("bible_stage.{stage}"), val_json);
            }
        }
    }
    serde_json::Value::Object(messages)
}

pub fn serve(request: tiny_http::Request, control: &WebControl) {
    let language = crate::services::i18n::language(&request, Some(control.language()));
    let url = request.url().to_owned();
    let (path, query) = url.split_once('?').unwrap_or((&url, ""));
    let params: HashMap<_, _> = query
        .split('&')
        .filter_map(|part| part.split_once('='))
        .map(|(key, value)| (decode(key), decode(value)))
        .collect();
    let asset: Option<(&str, &[u8])> = match path {
        "/control" | "/control/" => Some((
            "text/html; charset=utf-8",
            include_bytes!("../../../assets/web/control/control.html"),
        )),
        "/control.css" => Some((
            "text/css; charset=utf-8",
            include_bytes!("../../../assets/web/control/control.css"),
        )),
        "/control-mobile.css" => Some((
            "text/css; charset=utf-8",
            include_bytes!("../../../assets/web/control/control-mobile.css"),
        )),
        "/control-shell.css" => Some((
            "text/css; charset=utf-8",
            include_bytes!("../../../assets/web/control/control-shell.css"),
        )),
        "/control-icons.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/control/control-icons.js"),
        )),
        "/control-ui.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/control/control-ui.js"),
        )),
        "/control-native.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/control/control-native.js"),
        )),
        "/control-docking.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/control/control-docking.js"),
        )),
        "/control-native-bible.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/control/control-native-bible.js"),
        )),
        "/control-native-bible.css" => Some((
            "text/css; charset=utf-8",
            include_bytes!("../../../assets/web/control/control-native-bible.css"),
        )),
        "/control-dock.css" => Some((
            "text/css; charset=utf-8",
            include_bytes!("../../../assets/web/control/control-dock.css"),
        )),
        "/control.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/control/control.js"),
        )),
        "/control-auth.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/control/control-auth.js"),
        )),
        "/control-api.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/control/control-api.js"),
        )),
        "/control-library.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/control/control-library.js"),
        )),
        "/control-song.js" => Some(("application/javascript; charset=utf-8", include_bytes!("../../../assets/web/control/control-song.js"))),
        "/control-versions.js" => Some(("application/javascript; charset=utf-8", include_bytes!("../../../assets/web/control/control-versions.js"))),
        "/control-bible.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/control/control-bible.js"),
        )),
        _ => None,
    };
    if request.method().as_str() == "GET" {
        if path == "/api/control/translations" {
            reply(request, 200, "application/json; charset=utf-8", web_translations(language).to_string().into_bytes());
            return;
        }
        if path == "/api/control/ui-font" {
            let fonts = eframe::egui::FontDefinitions::default();
            if let Some(font) = fonts
                .families
                .get(&eframe::egui::FontFamily::Proportional)
                .and_then(|names| names.first())
                .and_then(|name| fonts.font_data.get(name))
            {
                reply(request, 200, "font/ttf", font.bytes().to_vec());
            } else {
                reply(request, 404, "text/plain", Vec::new());
            }
            return;
        }
        if path == "/api/control/custom-icon-font" || path == "/api/control/icon-font" {
            reply(
                request,
                200,
                "font/ttf",
                include_bytes!("../../../assets/fonts/custom-icons.ttf").to_vec(),
            );
            return;
        }
        if let Some((mime, body)) = asset {
            reply(request, 200, mime, body.to_vec());
            return;
        }
    }
    let Some(mut request) = super::auth::gate(request, control) else { return; };
    if request.method().as_str() == "GET" {
        if path == "/api/control/session" {
            reply(request, 200, "application/json", b"{\"ok\":true}".to_vec());
            return;
        }
        if path == "/api/control/background" {
            if let Some((bytes, mime)) = params.get("id").and_then(|id| control.background(id)) {
                reply(request, 200, mime, bytes);
            } else {
                reply(request, 404, "text/plain", Vec::new());
            }
            return;
        }
    }
    let result = (|| -> Result<Value, String> {
        match (request.method().as_str(), path) {
            ("GET", "/api/control/icons") => {
                use egui_phosphor::regular::*;
                Ok(json!(std::collections::BTreeMap::from([
                    ("operate", "\u{E001}"),
                    ("music", "\u{E001}"),
                    ("bible", "\u{E002}"),
                    ("text", FILE_TEXT),
                    ("media", VIDEO),
                    ("design", STACK),
                    ("verse", "\u{E002}"),
                    ("playlist", LIST_BULLETS),
                    ("addlist", LIST_PLUS),
                    ("playlist_add", LIST_PLUS),
                    ("theme", SQUARES_FOUR),
                    ("search", MAGNIFYING_GLASS),
                    ("folder", FOLDER_PLUS),
                    ("filter", FUNNEL),
                    ("play", PLAY),
                    ("pause", PAUSE),
                    ("stop", STOP),
                    ("next", CARET_RIGHT),
                    ("previous", CARET_LEFT),
                    ("first", SKIP_BACK),
                    ("last", SKIP_FORWARD),
                    ("eye", EYE),
                    ("eyeoff", EYE_SLASH),
                    ("plus", PLUS_CIRCLE),
                    ("close", X),
                    ("more", DOTS_SIX_VERTICAL),
                    ("pin", HEART),
                    ("up", ARROW_UP),
                    ("down", ARROW_DOWN),
                    ("check", CHECK),
                    ("trash", TRASH),
                    ("edit", PENCIL_SIMPLE),
                    ("layout", LAYOUT),
                    ("expand", ARROWS_OUT),
                    ("collapse", ARROWS_IN),
                    ("screen", MONITOR),
                    ("chat", "\u{E003}"),
                    ("grip", DOTS_SIX_VERTICAL),
                    ("grid", SQUARES_FOUR),
                    ("settings", SLIDERS_HORIZONTAL),
                    ("user", USER),
                    ("church", CHURCH),
                    ("calendar", CALENDAR_BLANK),
                    ("refresh", ARROWS_CLOCKWISE),
                    ("sun", SUN),
                    ("moon", MOON),
                    ("circle", CIRCLE),
                    ("check_circle", CHECK_CIRCLE)
                ])))
            }
            ("GET", "/api/control/state") => Ok(control.snapshot()),
            ("GET", "/api/control/library") => {
                let kind: ResourceKind = serde_json::from_value(json!(
                    params.get("kind").map(String::as_str).unwrap_or("song")
                ))
                .map_err(|e| e.to_string())?;
                control.catalog(
                    kind,
                    params.get("q").map(String::as_str).unwrap_or(""),
                    params.get("folder").map(String::as_str),
                )
            }
            ("GET", "/api/control/resource") => {
                let kind: ResourceKind = serde_json::from_value(json!(
                    params.get("kind").map(String::as_str).unwrap_or("song")
                ))
                .map_err(|e| e.to_string())?;
                let item =
                    control.resource(kind, params.get("id").map(String::as_str).unwrap_or(""))?;
                let song = if kind == ResourceKind::Song { control.db.get_all_songs().map_err(|e| e.to_string())?.into_iter().find(|song| song.id.map(|id| id.to_string()).as_deref() == Some(item.id.as_str())) } else { None };
                Ok(
                    json!({"id":item.id,"title":item.title,"artist":item.subtitle,"content":if kind==ResourceKind::Media { "" } else { &item.content },"folder":song.as_ref().map(|s|s.folder.as_str()),"tags":song.as_ref().map(|s|s.tags.as_str())}),
                )
            }
            ("GET", "/api/control/versions") => Ok(json!(
                control
                    .bible
                    .read()
                    .unwrap()
                    .catalog()
                    .into_iter()
                    .map(|(id, name)| json!({"id":id,"name":name}))
                    .collect::<Vec<_>>()
            )),
            ("GET", "/api/control/bible") => {
                let bible = control
                    .bible
                    .read()
                    .unwrap()
                    .get(
                        params
                            .get("version")
                            .map(String::as_str)
                            .unwrap_or(crate::core::bible::DEFAULT_BIBLE),
                    )
                    .ok_or("services-bible-translation-not-found")?;
                Ok(crate::services::chat::bible_suggestions(
                    &bible,
                    params.get("q").map(String::as_str).unwrap_or(""),
                ))
            }
            ("GET", "/api/control/books") => {
                let bible = control
                    .bible
                    .read()
                    .unwrap()
                    .get(
                        params
                            .get("version")
                            .map(String::as_str)
                            .unwrap_or(crate::core::bible::DEFAULT_BIBLE),
                    )
                    .ok_or("services-bible-translation-not-found")?;
                Ok(json!(
                    bible
                        .books
                        .iter()
                        .map(|book| json!({"name":book.name,"chapters":book.chapters.len()}))
                        .collect::<Vec<_>>()
                ))
            }
            ("GET", "/api/control/translations") => Ok(web_translations(language)),
            ("POST", "/api/control/command") => {
                let mime = request
                    .headers()
                    .iter()
                    .find(|header| header.field.equiv("Content-Type"))
                    .map(|header| header.value.as_str())
                    .unwrap_or("");
                if !mime.starts_with("application/json") {
                    return Err("services-send-the-command-as-json".into());
                }
                let mut body = Vec::new();
                request
                    .as_reader()
                    .take(128 * 1024 + 1)
                    .read_to_end(&mut body)
                    .map_err(|e| e.to_string())?;
                if body.len() > 128 * 1024 {
                    return Err("services-the-command-is-too-large".into());
                }
                let command: ControlCommand =
                    serde_json::from_slice(&body).map_err(|e| e.to_string())?;
                control.dispatch(command)
            }
            _ => Err("services-endpoint-not-found".into()),
        }
    })();
    let (status, body) = match result {
        Ok(value) => (200, value),
        Err(error) => (
            if error == "services-endpoint-not-found" || error.starts_with("Endpoint") {
                404
            } else {
                400
            },
            json!({"error":language.message(&error)}),
        ),
    };
    reply(
        request,
        status,
        "application/json; charset=utf-8",
        body.to_string().into_bytes(),
    );
}

pub(super) fn reply(request: tiny_http::Request, status: u16, mime: &str, body: Vec<u8>) {
    let mut response = tiny_http::Response::from_data(body).with_status_code(status);
    for (key, value) in [
        ("Content-Type", mime),
        ("Cache-Control", "no-store"),
        ("X-Content-Type-Options", "nosniff"),
    ] {
        response.add_header(tiny_http::Header::from_bytes(key, value).unwrap());
    }
    let _ = request.respond(response);
}

pub fn serve_preview(request: tiny_http::Request, control: &WebControl) {
    let path = request.url().split('?').next().unwrap_or("/").to_owned();
    let asset: Option<(&str, &[u8])> = match path.as_str() {
        "/viewer.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/viewer/viewer.js"),
        )),
        "/design.js" => Some((
            "application/javascript; charset=utf-8",
            include_bytes!("../../../assets/web/viewer/design.js"),
        )),
        "/shader.glsl" => Some((
            "text/plain; charset=utf-8",
            include_bytes!("../../../assets/shaders/background.glsl"),
        )),
        _ => None,
    };
    if let Some((mime, body)) = asset {
        reply(request, 200, mime, body.to_vec());
        return;
    }
    let Some(request) = super::auth::gate(request, control) else { return; };
    let renderer = if path.starts_with("/control-live/") || path.contains("__control_live") {
        &control.live
    } else {
        &control.preview
    };
    if matches!(
        path.as_str(),
        "/control-preview/video" | "/control-live/video"
    ) {
        let video = renderer.lock().unwrap().preview_video();
        if let Some(video) = video {
            crate::services::web::serve_video(request, &video);
            return;
        }
    }
    let (status, mime, body) = renderer.lock().unwrap().preview_response(&path);
    reply(request, status, mime, body);
}
