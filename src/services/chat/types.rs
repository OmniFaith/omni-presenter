use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub tool: String,
    pub description: String,
}

impl Channel {
    pub fn display_name(&self, language: crate::core::i18n::Language) -> &str {
        if self.id == "general" {
            language.text("chat-general")
        } else {
            &self.name
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub name: String,
    pub mime: String,
    pub size: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub author_id: String,
    pub author: String,
    pub timestamp: u64,
    pub channel_id: Option<String>,
    pub recipient_id: Option<String>,
    pub kind: String,
    pub title: String,
    pub text: String,
    pub file: Option<Attachment>,
    #[serde(default)]
    pub shared: Option<SharedContent>,
    #[serde(default)]
    pub reactions: Vec<Reaction>,
    #[serde(default)]
    pub deleted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reaction {
    pub emoji: String,
    pub users: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotificationItem {
    pub id: u64,
    pub author_id: String,
    pub author: String,
    pub preview: String,
    pub channel_id: Option<String>,
    pub peer_id: Option<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Notifications {
    pub total: usize,
    pub channels: HashMap<String, usize>,
    pub private: HashMap<String, usize>,
    pub latest_id: u64,
    pub recent: Vec<NotificationItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SharedContent {
    Bible {
        reference: String,
        version: String,
        content: String,
    },
    Song {
        id: i64,
        title: String,
        artist: String,
        content: String,
    },
    Media {
        file: Attachment,
    },
}

#[derive(Clone, Debug)]
pub struct ChatAction {
    pub id: String,
    pub profile_id: String,
    pub action: String,
    pub content: SharedContent,
    pub message_id: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ActionResult {
    pub id: String,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct SendMessage {
    pub channel_id: Option<String>,
    pub recipient_id: Option<String>,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub reference: String,
    #[serde(default)]
    pub version: String,
    pub song_id: Option<i64>,
    pub file_id: Option<String>,
    #[serde(default)]
    pub merge: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfileView {
    pub id: String,
    pub name: String,
    pub online: bool,
    #[serde(default)]
    pub avatar_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub profiles: Vec<ProfileView>,
    pub channels: Vec<Channel>,
    pub messages: Vec<Message>,
    pub latest_id: u64,
    pub notifications: Notifications,
}
