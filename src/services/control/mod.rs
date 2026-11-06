//! Browser commands cross a queue and are applied exclusively by the application thread.
mod catalog;
mod auth;
mod http;
mod types;
#[cfg(test)]
mod auth_tests;
pub use http::{serve, serve_preview, web_translations};
pub use types::{ControlCommand, ResourceKind};

use crate::{core::bible::BibleLibrary, services::db::AppDb};
use serde_json::Value;
use std::sync::{Arc, Mutex, RwLock, mpsc};

pub struct PendingCommand {
    pub command: ControlCommand,
    pub issued: std::time::Instant,
    pub reply: mpsc::SyncSender<Result<Value, String>>,
}

#[derive(Clone)]
pub struct WebControl {
    pub(crate) preview: Arc<Mutex<crate::services::web::BrowserServer>>,
    pub(crate) live: Arc<Mutex<crate::services::web::BrowserServer>>,
    pub(crate) db: AppDb,
    pub(crate) bible: Arc<RwLock<BibleLibrary>>,
    state: Arc<RwLock<Value>>,
    pending: Arc<Mutex<std::collections::VecDeque<PendingCommand>>>,
    wake: Arc<Mutex<Option<Arc<dyn Fn() + Send + Sync>>>>,
    backgrounds: Arc<RwLock<std::collections::HashMap<String, String>>>,
    instance: Arc<String>,
    auth: Arc<RwLock<Option<crate::services::chat::ChatService>>>,
}

impl WebControl {
    pub fn new(db: AppDb) -> Self {
        let mut bible = BibleLibrary::default();
        if let Ok(versions) = db.get_bible_translations() {
            for version in versions {
                bible.insert(version);
            }
        }
        Self {
            preview: Arc::new(Mutex::new(
                crate::services::web::BrowserServer::preview_renderer(),
            )),
            live: Arc::new(Mutex::new(
                crate::services::web::BrowserServer::preview_renderer(),
            )),
            db,
            bible: Arc::new(RwLock::new(bible)),
            state: Arc::new(RwLock::new(serde_json::json!({"ready":false}))),
            pending: Default::default(),
            wake: Default::default(),
            backgrounds: Default::default(),
            instance: Arc::new(crate::core::preferences::new_id("control")),
            auth: Default::default(),
        }
    }

    pub fn snapshot(&self) -> Value {
        self.state.read().unwrap().clone()
    }
    pub(super) fn language(&self) -> crate::core::i18n::Language {
        self.state.read().unwrap()["language"].as_str()
            .map(crate::core::i18n::Language::from_code).unwrap_or_default()
    }
    pub fn set_auth(&self, service: Option<crate::services::chat::ChatService>) {
        *self.auth.write().unwrap() = service;
    }
    pub fn publish_outputs(
        &self,
        deck: &crate::core::slides::SlideDeck,
        theme: &crate::core::models::PresentationTheme,
        live_theme: &crate::core::models::PresentationTheme,
        themes: &[crate::core::models::PresentationTheme],
        aspect: f32,
        active: bool,
    ) {
        use crate::core::preferences::{AppSettings, ServiceProfile, VirtualScreen};
        let mut settings = AppSettings {
            screens: vec![VirtualScreen {
                id: "__control_preview".into(),
                ..Default::default()
            }],
            services: vec![ServiceProfile {
                aspect_ratio: aspect,
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut preview = self.preview.lock().unwrap();
        preview.publish(&settings, &deck.preview_output(), theme, true, themes);
        preview.rewrite_output_urls("/control-preview");
        settings.screens[0].id = "__control_live".into();
        let mut live = self.live.lock().unwrap();
        live.publish(&settings, deck, live_theme, active, themes);
        live.rewrite_output_urls("/control-live");
    }
    pub fn set_bible(&self, bible: BibleLibrary) {
        *self.bible.write().unwrap() = bible;
    }
    pub fn publish(&self, mut value: Value) {
        let mut state = self.state.write().unwrap();
        value["revision"] = serde_json::json!(state["revision"].as_u64().unwrap_or(0) + 1);
        value["instance"] = serde_json::json!(*self.instance);
        *state = value;
    }
    pub fn set_backgrounds(&self, backgrounds: impl IntoIterator<Item = (String, String)>) {
        *self.backgrounds.write().unwrap() = backgrounds.into_iter().collect();
    }
    pub fn background(&self, id: &str) -> Option<(Vec<u8>, &'static str)> {
        let path = self.backgrounds.read().unwrap().get(id)?.clone();
        let bytes = std::fs::read(&path).ok()?;
        let mime = match path
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str()
        {
            "png" => "image/png",
            "webp" => "image/webp",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "bmp" => "image/bmp",
            _ => "image/jpeg",
        };
        Some((bytes, mime))
    }
    pub fn set_waker(&self, wake: impl Fn() + Send + Sync + 'static) {
        let mut callback = self.wake.lock().unwrap();
        if callback.is_none() {
            *callback = Some(Arc::new(wake));
        }
    }
    pub fn drain(&self) -> Vec<PendingCommand> {
        self.pending.lock().unwrap().drain(..).collect()
    }

    pub fn dispatch(&self, command: ControlCommand) -> Result<Value, String> {
        let (reply, receive) = mpsc::sync_channel(1);
        {
            let mut pending = self.pending.lock().unwrap();
            if pending.len() >= 128 {
                return Err(crate::core::i18n::Language::system().text("services-the-application-is-busy-wait-and-try-again").to_string());
            }
            pending.push_back(PendingCommand {
                command,
                reply,
                issued: std::time::Instant::now(),
            });
        }
        if let Some(wake) = self.wake.lock().unwrap().clone() {
            wake();
        }
        receive
            .recv_timeout(std::time::Duration::from_secs(15))
            .map_err(|_| crate::core::i18n::Language::system().text("services-the-application-did-not-respond-to-the-command").to_string())?
    }
}
