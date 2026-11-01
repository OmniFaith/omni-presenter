//! LAN chat shared by the native UI, browser and HTTP API.
mod actions;
mod auth;
mod bible;
mod directory;
mod files;
mod http;
mod messages;
mod notifications;
pub mod recording;
mod types;

pub(crate) use bible::suggestions as bible_suggestions;
pub(crate) use http::SecureChatServer;
pub use http::{serve, tls_port};
pub use types::*;

use crate::services::db::AppDb;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::Instant,
};

pub const MAX_UPLOAD: usize = 256 * 1024 * 1024;

fn secret() -> Result<String, String> {
    let mut bytes = [0_u8; 24];
    getrandom::fill(&mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Profile {
    id: String,
    name: String,
    #[serde(default)]
    password_hash: String,
    #[serde(default)]
    avatar: Option<Vec<u8>>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct Store {
    profiles: Vec<Profile>,
    channels: Vec<Channel>,
    messages: Vec<Message>,
    sequence: u64,
    #[serde(default)]
    reads: HashMap<String, HashMap<String, u64>>,
}
struct Session {
    profile_id: String,
    seen: Instant,
}
struct Runtime {
    store: Store,
    sessions: HashMap<String, Session>,
    actions: VecDeque<ChatAction>,
    results: HashMap<String, (String, ActionResult, Instant)>,
}

#[derive(Clone)]
pub struct ChatService {
    db: AppDb,
    runtime: Arc<Mutex<Runtime>>,
}

#[cfg(test)]
mod browser_tests;
#[cfg(test)]
mod channel_tests;
#[cfg(test)]
mod tests;
