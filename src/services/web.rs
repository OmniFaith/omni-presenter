use crate::core::models::PresentationTheme;
use crate::core::preferences::AppSettings;
use crate::core::slides::SlideDeck;
use serde_json::json;
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
#[path = "web_design.rs"]
mod design_output;
#[path = "web_text.rs"]
mod native_text;
#[cfg(test)]
#[path = "web_typography_tests.rs"]
mod typography_tests;

pub fn local_address() -> String {
    std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| {
            s.connect("192.0.2.1:80")?;
            s.local_addr()
        })
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|_| "127.0.0.1".into())
}

/// Builds an HTTP URL and omits the port when it is the default one.
pub fn http_url(host: &str, port: u16, path: &str) -> String {
    network_url("http", host, port, 80, path)
}

pub fn https_url(host: &str, port: u16, path: &str) -> String {
    network_url("https", host, port, 443, path)
}

fn network_url(scheme: &str, host: &str, port: u16, default_port: u16, path: &str) -> String {
    if port == default_port {
        format!("{scheme}://{host}{path}")
    } else {
        format!("{scheme}://{host}:{port}{path}")
    }
}

#[derive(Clone, Default)]
struct PublishedScreen {
    id: String,
    name: String,
    slug: String,
    state: String,
}

fn percent_decode(s: &str) -> String {
    let mut bytes = Vec::with_capacity(s.len());
    let mut chars = s.as_bytes().iter().copied();
    while let Some(b) = chars.next() {
        if b == b'%' {
            if let (Some(h1), Some(h2)) = (chars.next(), chars.next())
                && let Ok(hex) = u8::from_str_radix(std::str::from_utf8(&[h1, h2]).unwrap_or(""), 16) {
                    bytes.push(hex);
                    continue;
                }
        } else if b == b'+' {
            bytes.push(b' ');
            continue;
        }
        bytes.push(b);
    }
    String::from_utf8(bytes).unwrap_or_else(|_| s.to_string())
}

#[derive(Default)]
struct Published {
    design: Option<serde_json::Value>,
    design_assets: std::collections::HashMap<String, (Vec<u8>, &'static str)>,
    screens: Vec<PublishedScreen>,
    background: Option<(Vec<u8>, &'static str)>,
    previous_background: Option<(Vec<u8>, &'static str)>,
    background_revision: u64,
    video_path: Option<std::path::PathBuf>,
    font: Vec<u8>,
    font_revision: u64,
}

impl Published {
    fn find_screen(&self, query: &str) -> Option<&PublishedScreen> {
        let clean = query.trim_matches('/');
        let decoded = percent_decode(clean);
        let clean_slug = crate::core::preferences::slugify(clean);
        let decoded_slug = crate::core::preferences::slugify(&decoded);

        self.screens.iter().find(|s| {
            s.slug == clean
                || s.id == clean
                || s.slug == decoded
                || s.id == decoded
                || (!clean_slug.is_empty() && s.slug == clean_slug)
                || (!decoded_slug.is_empty() && s.slug == decoded_slug)
                || s.name.eq_ignore_ascii_case(clean)
                || s.name.eq_ignore_ascii_case(&decoded)
        })
    }
}

pub struct BrowserServer {
    stop: Arc<AtomicBool>,
    data: Arc<RwLock<Published>>,
    thread: Option<std::thread::JoinHandle<()>>,
    background_path: String,
    font_key: String,
    _secure_chat: Option<crate::services::chat::SecureChatServer>,
    pub chat_https_error: Option<String>,
    _discovery: Option<crate::services::discovery::Discovery>,
    pub discovery_url: Option<String>,
    pub discovery_chat_url: Option<String>,
    pub discovery_error: Option<String>,
}

impl BrowserServer {
    pub(crate) fn preview_renderer() -> Self {
        Self {
            stop: Arc::new(AtomicBool::new(false)),
            data: Default::default(),
            thread: None,
            background_path: String::new(),
            font_key: String::new(),
            _secure_chat: None,
            chat_https_error: None,
            _discovery: None,
            discovery_url: None,
            discovery_chat_url: None,
            discovery_error: None,
        }
    }

    pub(crate) fn preview_response(&self, path: &str) -> (u16, &'static str, Vec<u8>) {
        route(
            path.strip_prefix("/control-preview")
                .or_else(|| path.strip_prefix("/control-live"))
                .unwrap_or(path),
            &self.data.read().unwrap(),
        )
    }

    pub(crate) fn preview_video(&self) -> Option<std::path::PathBuf> {
        self.data.read().unwrap().video_path.clone()
    }

    pub(crate) fn rewrite_output_urls(&self, output_prefix: &str) {
        fn rewrite(value: &mut serde_json::Value, output_prefix: &str) {
            match value {
                serde_json::Value::String(text) => {
                    for prefix in ["/background", "/font?", "/video", "/design-assets/"] {
                        if text.starts_with(prefix)
                            || (prefix == "/background" && text.starts_with("url('/background"))
                        {
                            *text = text.replacen(prefix, &format!("{output_prefix}{prefix}"), 1);
                            break;
                        }
                    }
                }
                serde_json::Value::Array(items) => items
                    .iter_mut()
                    .for_each(|value| rewrite(value, output_prefix)),
                serde_json::Value::Object(fields) => fields
                    .iter_mut()
                    .filter(|(key, _)| {
                        ![
                            "text",
                            "title",
                            "name",
                            "next",
                            "nextText",
                            "next_text",
                            "Text",
                            "question",
                            "answer",
                        ]
                        .contains(&key.as_str())
                    })
                    .for_each(|(_, value)| rewrite(value, output_prefix)),
                _ => {}
            }
        }
        let mut data = self.data.write().unwrap();
        for screen in &mut data.screens {
            if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&screen.state) {
                rewrite(&mut value, output_prefix);
                screen.state = value.to_string();
            }
        }
    }
    pub fn start(settings: &AppSettings) -> Result<Self, String> {
        Self::start_with_chat(settings, None)
    }

    pub fn start_with_chat(
        settings: &AppSettings,
        chat: Option<crate::services::chat::ChatService>,
    ) -> Result<Self, String> {
        Self::start_with_control(settings, chat, None)
    }

    pub fn start_with_control(
        settings: &AppSettings,
        chat: Option<crate::services::chat::ChatService>,
        control: Option<crate::services::control::WebControl>,
    ) -> Result<Self, String> {
        let host = if settings.allow_lan {
            "0.0.0.0"
        } else {
            "127.0.0.1"
        };
        let server = super::web_http::HttpServer::bind(host, settings.server_port)?;
        let stop = Arc::new(AtomicBool::new(false));
        let data = Arc::new(RwLock::new(Published::default()));
        let worker_stop = stop.clone();
        let worker_data = data.clone();
        let (secure_chat, chat_https_error) = if let Some(chat) = &chat {
            match crate::services::chat::SecureChatServer::start(
                chat.clone(),
                crate::services::chat::tls_port(settings.server_port),
                settings.allow_lan,
            ) {
                Ok(server) => (Some(server), None),
                Err(error) => (None, Some(error)),
            }
        } else {
            (None, None)
        };
        let (discovery, discovery_error) = if settings.allow_lan {
            let screens: Vec<String> = if settings.server_enabled {
                settings
                    .screens
                    .iter()
                    .filter(|screen| screen.enabled)
                    .map(|screen| {
                        crate::core::preferences::screen_url_slug(screen, &settings.screens)
                    })
                    .collect()
            } else {
                Vec::new()
            };
            let tls_port = secure_chat
                .is_some()
                .then(|| crate::services::chat::tls_port(settings.server_port));
            match crate::services::discovery::Discovery::start(
                server.port,
                control.is_some(),
                chat.is_some(),
                tls_port,
                &screens,
            ) {
                Ok(discovery) => (Some(discovery), None),
                Err(error) => (None, Some(error)),
            }
        } else {
            (None, None)
        };
        let discovery_url = discovery.as_ref().and_then(super::discovery::Discovery::control_url);
        let discovery_chat_url = discovery
            .as_ref()
            .filter(|_| chat.is_some())
            .and_then(crate::services::discovery::Discovery::chat_url);
        let thread = std::thread::spawn(move || {
            while !worker_stop.load(Ordering::Relaxed) {
                let Ok(Some(request)) = server.recv_timeout(Duration::from_millis(100)) else {
                    continue;
                };
                let path = request.url().split('?').next().unwrap_or("/");
                if (path.starts_with("/control-preview/")
                    || path.starts_with("/control-live/")
                    || matches!(
                        path,
                        "/screen/__control_preview"
                            | "/api/state/__control_preview"
                            | "/screen/__control_live"
                            | "/api/state/__control_live"
                            | "/viewer.js"
                            | "/design.js"
                            | "/shader.glsl"
                    ))
                    && let Some(control) = &control
                {
                    let control = control.clone();
                    std::thread::spawn(move || {
                        crate::services::control::serve_preview(request, &control)
                    });
                    continue;
                }
                if (path == "/control"
                    || path == "/control/"
                    || path.starts_with("/control-")
                    || path.starts_with("/control.")
                    || path.starts_with("/api/control/"))
                    && let Some(control) = &control
                {
                    let control = control.clone();
                    std::thread::spawn(move || crate::services::control::serve(request, &control));
                    continue;
                }
                if is_chat_path(path)
                    && let Some(chat) = &chat
                {
                    let chat = chat.clone();
                    std::thread::spawn(move || crate::services::chat::serve(request, &chat));
                    continue;
                }
                if path == "/video" {
                    let video = worker_data.read().unwrap().video_path.clone();
                    if let Some(path) = video {
                        std::thread::spawn(move || serve_video(request, &path));
                        continue;
                    }
                }
                let published = worker_data.read().unwrap();
                let (status, content_type, body) = route(path, &published);
                drop(published);
                let mut response = tiny_http::Response::from_data(body).with_status_code(status);
                response.add_header(
                    tiny_http::Header::from_bytes("Content-Type", content_type).unwrap(),
                );
                response.add_header(
                    tiny_http::Header::from_bytes("Cache-Control", "no-store").unwrap(),
                );
                response.add_header(
                    tiny_http::Header::from_bytes("X-Content-Type-Options", "nosniff").unwrap(),
                );
                response.add_header(
                    tiny_http::Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap(),
                );
                let _ = request.respond(response);
            }
        });
        Ok(Self {
            stop,
            data,
            thread: Some(thread),
            background_path: String::new(),
            font_key: String::new(),
            _secure_chat: secure_chat,
            chat_https_error,
            _discovery: discovery,
            discovery_url,
            discovery_chat_url,
            discovery_error,
        })
    }

    pub fn refresh_discovery(&mut self) {
        if let Some(discovery) = &mut self._discovery {
            discovery.poll();
            self.discovery_url = discovery.control_url();
            self.discovery_chat_url = discovery.chat_url();
            self.discovery_error = discovery.error.clone();
        }
    }

    pub fn publish(
        &mut self,
        settings: &AppSettings,
        deck: &SlideDeck,
        theme: &PresentationTheme,
        active: bool,
        all_themes: &[PresentationTheme],
    ) {
        let design = deck.live_design.as_ref().filter(|_| active);
        let theme = design
            .filter(|scene| scene.background_mode == "theme")
            .and_then(|scene| {
                all_themes
                    .iter()
                    .find(|theme| Some(&theme.id) == scene.theme_id.as_ref())
            })
            .unwrap_or(theme);
        let themed = design.is_none_or(|scene| scene.background_mode == "theme");
        let path = if themed && theme.bg_type == "image" {
            theme.bg_image_path.clone()
        } else {
            String::new()
        };
        let mut data = self.data.write().unwrap();
        let font_key = format!(
            "{}:{}:{}",
            theme.font_family, theme.font_weight, theme.font_italic
        );
        if self.font_key != font_key {
            self.font_key = font_key;
            data.font = crate::core::models::fonts::presentation_font_bytes(
                &theme.font_family,
                theme.font_weight,
                theme.font_italic,
            );
            data.font_revision += 1;
        }
        let video_path = if themed
            && theme.bg_type == "video"
            && !crate::services::video::path(theme).is_empty()
        {
            Some(std::path::PathBuf::from(crate::services::video::path(
                theme,
            )))
        } else {
            None
        };
        if data.video_path != video_path {
            data.video_path = video_path;
            data.background_revision += 1;
        }
        if self.background_path != path {
            self.background_path = path.clone();
            data.background_revision += 1;
            // A viewer may still be fetching the outgoing image during a transition.
            if data.background.is_some() {
                data.previous_background = data.background.take();
            }
            data.background = std::fs::read(path)
                .ok()
                .and_then(|bytes| match image::guess_format(&bytes).ok()? {
                    image::ImageFormat::Png => Some((bytes, "image/png")),
                    image::ImageFormat::Jpeg => Some((bytes, "image/jpeg")),
                    _ => None,
                });
        }
        // Browsers receive layers and render at their own viewport size.
        let extra = deck.live_overlay.as_ref().filter(|_| active);
        let resolve_scene = |scene: &crate::core::design::DesignDocument| {
            scene
                .resolve_theme_fonts(
                    all_themes.iter().find(|theme| Some(&theme.id) == scene.theme_id.as_ref()),
                )
                .into_owned()
        };
        let mut composite = design.map(resolve_scene).or_else(|| {
            extra.map(|extra| {
                let mut scene = resolve_scene(extra);
                scene.background_mode = "transparent".into();
                scene.layers.clear();
                scene
            })
        });
        if let Some(scene) = &mut composite {
            if let Some(extra) = extra {
                scene.layers.extend(resolve_scene(extra).layers);
            }
            data.design = Some(design_output::prepare(scene, &mut data));
        } else {
            data.design = None;
        }
        data.screens = settings
            .screens
            .iter()
            .filter(|s| s.enabled)
            .map(|s| {
                let screen_theme = if let Some(ref tid) = s.custom_theme_id {
                    all_themes.iter().find(|t| &t.id == tid).unwrap_or(theme)
                } else {
                    theme
                };
                let mut options = if !themed { Default::default() } else { screen_theme.options.clone() };
                options.video.path.clear();

                let screen_bg = resolve_screen_bg(s, themed, screen_theme, theme, &data, all_themes);

                let raw_text = if active && design.is_none() { deck.projected_text() } else { "" };
                let text = if s.show_comments {
                    crate::core::slides::strip_directive_commands(raw_text)
                } else {
                    crate::core::slides::strip_directive_commands(&crate::core::slides::strip_comments(raw_text))
                };

                let raw_next = if active { deck.next_projected_text() } else { "" };
                let next_text = if s.show_comments {
                    crate::core::slides::strip_directive_commands(raw_next)
                } else {
                    crate::core::slides::strip_directive_commands(&crate::core::slides::strip_comments(raw_next))
                };

                let is_first_song_slide = deck.is_live_song() && deck.live_index() == 0;
                let full_title = if !deck.live_artist().is_empty() {
                    format!("{} · {}", deck.live_title(), deck.live_artist())
                } else {
                    deck.live_title().to_string()
                };
                let web_title = if design.is_none() && active && !deck.is_content_hidden() {
                    if is_first_song_slide {
                        full_title.as_str()
                    } else if s.show_title {
                        deck.live_title()
                    } else {
                        ""
                    }
                } else {
                    ""
                };

                let state = json!({
                    "name": s.name,
                    "title": web_title,
                    "text": text,
                    "active": active,
                    "hidden": active && deck.is_content_hidden(),
                    "background": screen_bg,
                    "design": data.design,
                    "transparent": s.lyrics_only,
                    "color": css_color(screen_theme.text_color),
                    "font": screen_theme.font_family,
                    "fontUrl": format!("/font?v={}", data.font_revision),
                    "backgroundType": if !themed { "color" } else { &screen_theme.bg_type },
                    "videoUrl": if data.video_path.is_some() && screen_theme.id == theme.id { format!("/video?v={}", data.background_revision) } else { String::new() },
                    "shaderIndex": crate::services::shaders::kind_index(&screen_theme.options.shader.kind),
                    "shaderFragment": screen_theme.options.shader.custom.as_ref().map(crate::core::plugins::shader::glsl),
                    "clock": if themed && screen_theme.bg_type == "shader" { crate::services::shaders::clock() } else { 0.0 },
                    "options": options,
                    "transition": screen_theme.transition,
                    "weight": screen_theme.font_weight,
                    "italic": screen_theme.font_italic,
                    "underline": screen_theme.font_underline,
                    "strike": screen_theme.font_strikethrough,
                    "size": screen_theme.font_size * s.font_scale as f32 / 100.0,
                    "lineHeight": screen_theme.line_spacing,
                    "letterSpacing": screen_theme.letter_spacing,
                    "align": screen_theme.align_h,
                    "vertical": screen_theme.align_v,
                    "case": screen_theme.text_case,
                    "box": [screen_theme.box_x, screen_theme.box_y, screen_theme.box_w, screen_theme.box_h],
                    "boxBackground": if screen_theme.box_bg_enabled { css_color(screen_theme.box_bg_color) } else { "transparent".into() },
                    "boxBorder": if screen_theme.box_border_enabled { format!("{}px solid {}", screen_theme.box_border_width, css_color(screen_theme.box_border_color)) } else { "none".into() },
                    "radius": screen_theme.box_radius,
                    "padding": screen_theme.box_padding,
                    "shadow": if screen_theme.shadow_enabled { format!("{}px {}px {}px {}", screen_theme.shadow_offset_x, screen_theme.shadow_offset_y, screen_theme.shadow_blur, css_color(screen_theme.shadow_color)) } else { "none".into() },
                    "outline": if screen_theme.outline_enabled { format!("{}px {}", screen_theme.outline_width, css_color(screen_theme.outline_color)) } else { "0 transparent".into() },
                    "motion": !settings.operator().reduce_motion && screen_theme.transition != "cut" && screen_theme.transition != "none",
                    "aspect": settings.service().aspect_ratio,
                    "showClock": s.show_clock,
                    "showNextPreview": s.show_next_preview,
                    "showComments": s.show_comments,
                    "nextText": next_text,
                });
                let slug = crate::core::preferences::screen_url_slug(s, &settings.screens);
                let mut state = state;
                state["nativeText"] = json!(!s.show_comments && !s.show_next_preview);
                state["textLayer"] = json!(if s.show_comments || s.show_next_preview { None } else { native_text::layer(&mut data, screen_theme, raw_text, s.font_scale, settings.service().aspect_ratio) });
                state["showTitle"] = json!(s.show_title);
                PublishedScreen {
                    id: s.id.clone(),
                    name: s.name.clone(),
                    slug,
                    state: state.to_string(),
                }
            })
            .collect();
    }
}

fn to_base64(data: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for c in data.chunks(3) {
        let n = ((c[0] as usize) << 16) | ((c.get(1).copied().unwrap_or(0) as usize) << 8) | (c.get(2).copied().unwrap_or(0) as usize);
        out.push(T[(n >> 18) & 63] as char);
        out.push(T[(n >> 12) & 63] as char);
        out.push(if c.len() > 1 { T[(n >> 6) & 63] as char } else { '=' });
        out.push(if c.len() > 2 { T[n & 63] as char } else { '=' });
    }
    out
}

fn resolve_screen_bg(
    s: &crate::core::preferences::VirtualScreen,
    themed: bool,
    screen_theme: &PresentationTheme,
    theme: &PresentationTheme,
    data: &Published,
    all_themes: &[PresentationTheme],
) -> String {
    if !themed || s.lyrics_only {
        return "transparent".to_string();
    }
    if let Some(ref bg) = s.custom_background {
        return match bg {
            crate::core::preferences::PhysicalBackground::Transparent => "transparent".to_string(),
            crate::core::preferences::PhysicalBackground::Color { rgb } => format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2]),
            crate::core::preferences::PhysicalBackground::Image { path } => std::fs::read(path).ok().map(|b| {
                let m = if path.ends_with(".png") { "image/png" } else { "image/jpeg" };
                format!("url('data:{};base64,{}') center / cover", m, to_base64(&b))
            }).unwrap_or_else(|| "transparent".to_string()),
            crate::core::preferences::PhysicalBackground::Theme { id } => {
                let th = all_themes.iter().find(|t| &t.id == id).unwrap_or(theme);
                if th.bg_type == "color" || th.bg_type == "solid" { css_color(th.bg_top) } else { theme_gradient(th) }
            }
            _ => "transparent".to_string(),
        };
    }
    if screen_theme.id == theme.id && data.background.is_some() {
        format!("url('/background?v={}') center / cover", data.background_revision)
    } else if screen_theme.bg_type == "color" || screen_theme.bg_type == "solid" {
        css_color(screen_theme.bg_top)
    } else {
        theme_gradient(screen_theme)
    }
}

fn theme_gradient(th: &PresentationTheme) -> String {
    if th.bg_type == "shader" && !th.options.shader.colors.is_empty() {
        let c = &th.options.shader.colors;
        format!("linear-gradient(135deg, rgb({},{},{}), rgb({},{},{}), rgb({},{},{}))", c[0][0], c[0][1], c[0][2], c[1][0], c[1][1], c[1][2], c[2][0], c[2][1], c[2][2])
    } else {
        format!("linear-gradient({}, {})", css_color(th.bg_top), css_color(th.bg_bottom))
    }
}

fn css_color(c: eframe::egui::Color32) -> String {
    let [r, g, b, a] = c.to_srgba_unmultiplied();
    format!("rgba({r},{g},{b},{:.3})", a as f32 / 255.0)
}

fn is_chat_path(path: &str) -> bool {
    matches!(path, "/" | "/chat" | "/chat/")
        || path.starts_with("/chat.")
        || path.starts_with("/chat-")
        || path.starts_with("/api/chat/")
}

fn route(path: &str, data: &Published) -> (u16, &'static str, Vec<u8>) {
    if !data.screens.is_empty()
        && let Some(key) = path.strip_prefix("/design-assets/")
        && let Some((bytes, mime)) = data.design_assets.get(key)
    {
        return (200, mime, bytes.clone());
    }
    match path {
        "/design.js" => return (200, "application/javascript; charset=utf-8", include_bytes!("../../assets/web/viewer/design.js").to_vec()),
        "/viewer.js" => return (200, "application/javascript; charset=utf-8", include_bytes!("../../assets/web/viewer/viewer.js").to_vec()),
        "/shader.glsl" => return (200, "text/plain; charset=utf-8", include_bytes!("../../assets/shaders/background.glsl").to_vec()),
        "/font" if !data.font.is_empty() => return (200, "font/ttf", data.font.clone()),
        _ => {}
    }
    if let Some(id) = path.strip_prefix("/screen/") {
        let clean_id = id.trim_matches('/');
        if data.find_screen(clean_id).is_some() {
            return (
                200,
                "text/html; charset=utf-8",
                include_bytes!("../../assets/web/viewer/viewer.html").to_vec(),
            );
        }
    }
    if let Some(id) = path.strip_prefix("/api/state/") {
        let clean_id = id.trim_matches('/');
        if let Some(screen) = data.find_screen(clean_id) {
            return (
                200,
                "application/json; charset=utf-8",
                screen.state.as_bytes().to_vec(),
            );
        }
    }
    if path == "/background" && !data.screens.is_empty()
        && let Some((bytes, mime)) = data
            .background
            .as_ref()
            .or(data.previous_background.as_ref())
        {
            return (200, mime, bytes.clone());
        }
    (
        404,
        "text/plain; charset=utf-8",
        crate::core::i18n::Language::system()
            .text("services-screen-unavailable-check-that-it-is-enabled")
            .as_bytes()
            .to_vec(),
    )
}

pub(crate) fn serve_video(request: tiny_http::Request, path: &std::path::Path) {
    use std::io::{Seek, SeekFrom};
    let result = (|| -> std::io::Result<()> {
        let mut file = std::fs::File::open(path)?;
        let size = file.metadata()?.len();
        let range = request
            .headers()
            .iter()
            .find(|h| h.field.equiv("Range"))
            .map(|h| h.value.as_str());
        let parsed = range
            .and_then(|value| value.strip_prefix("bytes="))
            .and_then(|value| value.split_once('-'));
        let (start, end, partial) = if let Some((start, end)) = parsed {
            (
                start.parse::<u64>().unwrap_or(0),
                end.parse::<u64>()
                    .unwrap_or(size.saturating_sub(1))
                    .min(size.saturating_sub(1)),
                true,
            )
        } else {
            (0, size.saturating_sub(1), false)
        };
        if size == 0 || start >= size || end < start {
            let mut response = tiny_http::Response::empty(416);
            response.add_header(
                tiny_http::Header::from_bytes("Content-Range", format!("bytes */{size}")).unwrap(),
            );
            return request.respond(response);
        }
        file.seek(SeekFrom::Start(start))?;
        let length = end - start + 1;
        let mime = if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("webm"))
        {
            "video/webm"
        } else {
            "video/mp4"
        };
        let mut headers = vec![
            tiny_http::Header::from_bytes("Content-Type", mime).unwrap(),
            tiny_http::Header::from_bytes("Accept-Ranges", "bytes").unwrap(),
        ];
        if partial {
            headers.push(
                tiny_http::Header::from_bytes(
                    "Content-Range",
                    format!("bytes {start}-{end}/{size}"),
                )
                .unwrap(),
            );
        }
        let response = tiny_http::Response::new(
            tiny_http::StatusCode(if partial { 206 } else { 200 }),
            headers,
            std::io::Read::take(file, length),
            Some(length as usize),
            None,
        );
        request.respond(response)
    })();
    if let Err(error) = result {
        eprintln!("Erro ao servir vídeo: {error}");
    }
}

impl Drop for BrowserServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_chat_asset_is_routed_to_the_chat_server() {
        for path in [
            "/",
            "/chat",
            "/chat/",
            "/chat.js",
            "/chat.css",
            "/chat-icons.js",
            "/chat-emoji.js",
            "/chat-attachments.js",
            "/chat-bible.js",
            "/chat-bootstrap.js",
            "/chat-channels.js",
            "/chat-picker.css",
            "/chat-cards.css",
            "/chat-workspace.css",
            "/chat-channels.css",
            "/api/chat/profiles",
        ] {
            assert!(is_chat_path(path), "{path} must reach the chat server");
        }
        assert!(!is_chat_path("/design"));
        assert!(!is_chat_path("/viewer.js"));
    }

    #[test]
    fn designs_and_transparent_extras_publish_without_replacing_live_lyrics() {
        use crate::core::design::{DesignDocument, DesignLayer, LayerContent};
        let settings = AppSettings::default();
        let mut deck = SlideDeck::new("Letra ao vivo");
        let mut extra = DesignDocument::default();
        extra.overlay = true;
        let mut shape = DesignLayer::new(LayerContent::Shape { radius: 0.0 });
        shape.rect = [0.0, 0.0, 0.2, 0.2];
        shape.color = [255, 0, 0, 128];
        extra.layers.push(shape);
        deck.live_overlay = Some(extra);
        let mut server = BrowserServer {
            stop: Arc::new(AtomicBool::new(false)),
            data: Arc::new(RwLock::new(Published::default())),
            thread: None,
            background_path: String::new(),
            font_key: String::new(),
            _secure_chat: None,
            chat_https_error: None,
            _discovery: None,
            discovery_url: None,
            discovery_chat_url: None,
            discovery_error: None,
        };
        server.publish(&settings, &deck, &PresentationTheme::default(), true, &[]);
        {
            let data = server.data.read().unwrap();
            let state: serde_json::Value =
                serde_json::from_slice(&route("/api/state/auditorio", &data).2).unwrap();
            assert_eq!(state["text"], "Letra ao vivo");
            assert_eq!(state["design"]["background_mode"], "transparent");
            assert_eq!(state["design"]["layers"][0]["color"][3], 128);
            assert_eq!(state["design"]["layers"][0]["rect"][0], 0.0);
            assert_eq!(route("/design.js", &data).0, 200);
        }
        server.publish(&settings, &deck, &PresentationTheme::default(), false, &[]);
        let data = server.data.read().unwrap();
        assert!(data.design.is_none());
        assert_eq!(route("/design", &data).0, 404);
    }

    #[test]
    fn http_server_serves_viewer_and_reports_port_conflicts() {
        use std::io::{Read, Write};
        let reserved = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = reserved.local_addr().unwrap().port();
        let settings = AppSettings {
            server_port: port,
            allow_lan: false,
            ..Default::default()
        };
        assert!(BrowserServer::start(&settings).is_err());
        drop(reserved);
        // A porta efêmera pode ser reutilizada por outra conexão logo após ser liberada
        // (tiny_http não define SO_REUSEADDR), então tentamos algumas vezes.
        let start_with_retry = || {
            for _ in 0..100 {
                match BrowserServer::start(&settings) {
                    Ok(server) => return server,
                    Err(_) => std::thread::sleep(Duration::from_millis(20)),
                }
            }
            panic!("a porta {port} não ficou livre após liberar o listener");
        };
        let mut server = start_with_retry();
        server.publish(
            &settings,
            &SlideDeck::new("Primeira\n\nSegunda"),
            &PresentationTheme::default(),
            true,
            &[],
        );
        for (path, expected) in [
            ("/screen/auditorio", "text/html"),
            ("/api/state/auditorio", "Primeira"),
        ] {
            let mut socket = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            write!(socket, "GET {path} HTTP/1.0\r\nHost: localhost\r\n\r\n").unwrap();
            let mut response = String::new();
            socket.read_to_string(&mut response).unwrap();
            assert!(response.starts_with("HTTP/1.0 200"));
            assert!(response.contains(expected));
        }
        drop(server);
        let rebind = start_with_retry();
        drop(rebind);
    }

    #[test]
    fn viewer_only_publishes_live_content_and_enabled_screens() {
        let mut settings = AppSettings::default();
        settings
            .screens
            .push(crate::core::preferences::VirtualScreen {
                id: "disabled".into(),
                enabled: false,
                ..Default::default()
            });
        let mut deck = SlideDeck::new("Ao vivo <script>\n\nPróxima estrofe");
        deck.load_custom_content("Prévia".into(), "".into(), "Não publicar".into());
        let mut server = BrowserServer {
            stop: Arc::new(AtomicBool::new(false)),
            data: Arc::new(RwLock::new(Published::default())),
            thread: None,
            background_path: String::new(),
            font_key: String::new(),
            _secure_chat: None,
            chat_https_error: None,
            _discovery: None,
            discovery_url: None,
            discovery_chat_url: None,
            discovery_error: None,
        };
        server.publish(&settings, &deck, &PresentationTheme::default(), true, &[]);
        let data = server.data.read().unwrap();
        let (status, _, body) = route("/api/state/auditorio", &data);
        assert_eq!(status, 200);
        let state: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(state["text"], "Ao vivo <script>");
        assert_eq!(route("/screen/disabled", &data).0, 404);
        assert_eq!(route("/screen/unknown", &data).0, 404);
        drop(data);
        deck.hide_content();
        server.publish(&settings, &deck, &PresentationTheme::default(), true, &[]);
        let data = server.data.read().unwrap();
        let state: serde_json::Value =
            serde_json::from_slice(&route("/api/state/auditorio", &data).2).unwrap();
        assert_eq!(state["text"], "");
        assert_eq!(state["hidden"], true);
    }

    #[test]
    fn test_screen_comments_custom_theme_and_stage_options() {
        let mut settings = AppSettings::default();
        settings.screens = vec![
            crate::core::preferences::VirtualScreen {
                id: "congregacao".into(),
                name: "Telão Principal".into(),
                enabled: true,
                show_comments: false,
                custom_theme_id: None,
                show_clock: false,
                show_next_preview: false,
                ..Default::default()
            },
            crate::core::preferences::VirtualScreen {
                id: "palco".into(),
                name: "Retorno Palco".into(),
                enabled: true,
                show_comments: true,
                custom_theme_id: Some("palco_theme".into()),
                show_clock: true,
                show_next_preview: true,
                ..Default::default()
            },
        ];

        let mut custom_theme = PresentationTheme::default();
        custom_theme.id = "palco_theme".into();
        custom_theme.name = "Tema Palco".into();
        custom_theme.text_color = eframe::egui::Color32::from_rgb(255, 200, 0);

        let deck = SlideDeck::new("// Intro: G D Em\nPorque Deus amou o mundo\n\nSegunda estrofe");
        let mut server = BrowserServer {
            stop: Arc::new(AtomicBool::new(false)),
            data: Arc::new(RwLock::new(Published::default())),
            thread: None,
            background_path: String::new(),
            font_key: String::new(),
            _secure_chat: None, chat_https_error: None,
            _discovery: None, discovery_url: None, discovery_chat_url: None, discovery_error: None,
        };
        server.publish(
            &settings,
            &deck,
            &PresentationTheme::default(),
            true,
            &[custom_theme],
        );

        let data = server.data.read().unwrap();

        // 1. Congregação: comments stripped
        let state_cong: serde_json::Value =
            serde_json::from_slice(&route("/api/state/congregacao", &data).2).unwrap();
        assert_eq!(state_cong["text"], "Porque Deus amou o mundo");
        assert_eq!(state_cong["showClock"], false);
        assert_eq!(state_cong["showNextPreview"], false);

        // 2. Palco: comments kept, custom theme used, clock & next preview enabled
        let state_palco: serde_json::Value =
            serde_json::from_slice(&route("/api/state/palco", &data).2).unwrap();
        assert_eq!(state_palco["text"], "// Intro: G D Em\nPorque Deus amou o mundo");
        assert_eq!(state_palco["color"], "rgba(255,200,0,1.000)");
        assert_eq!(state_palco["showClock"], true);
        assert_eq!(state_palco["showNextPreview"], true);
        assert_eq!(state_palco["nextText"], "Segunda estrofe");

        // 3. Screen URL resolution by slug, decoded name, and original id
        assert_eq!(route("/screen/telao-principal", &data).0, 200);
        assert_eq!(route("/api/state/telao-principal", &data).0, 200);
        assert_eq!(route("/screen/retorno-palco", &data).0, 200);
        assert_eq!(route("/api/state/retorno-palco", &data).0, 200);
        assert_eq!(route("/screen/Tel%C3%A3o%20Principal", &data).0, 200);
        assert_eq!(route("/api/state/congregacao", &data).0, 200);
    }
}
