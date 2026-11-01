use super::*;

impl ChatService {
    pub(super) fn visible(message: &Message, profile: &str) -> bool {
        message.channel_id.is_some()
            || message.author_id == profile
            || message.recipient_id.as_deref() == Some(profile)
    }

    pub(super) fn unread(store: &Store, profile: &str) -> Notifications {
        let mut notifications = Notifications::default();
        for message in &store.messages {
            if !Self::visible(message, profile) {
                continue;
            }
            notifications.latest_id = notifications.latest_id.max(message.id);
            if message.deleted || message.author_id == profile {
                continue;
            }
            let key = if let Some(channel) = &message.channel_id {
                format!("channel:{channel}")
            } else {
                format!("dm:{}", message.author_id)
            };
            let read = store
                .reads
                .get(profile)
                .and_then(|r| r.get(&key))
                .copied()
                .unwrap_or(0);
            if message.id <= read {
                continue;
            }
            notifications.total += 1;
            if let Some(channel) = &message.channel_id {
                *notifications.channels.entry(channel.clone()).or_default() += 1;
            } else {
                *notifications
                    .private
                    .entry(message.author_id.clone())
                    .or_default() += 1;
            }
            let preview = match message.kind.as_str() {
                "audio" => crate::core::i18n::Language::system().text("chat-voice-message").into(),
                "bible" | "song" => message.title.clone(),
                "file" => message
                    .file
                    .as_ref()
                    .map(|f| f.name.clone())
                    .unwrap_or_default(),
                _ => message.text.clone(),
            };
            notifications.recent.push(NotificationItem {
                id: message.id,
                author_id: message.author_id.clone(),
                author: message.author.clone(),
                preview: preview.replace('\n', " ").chars().take(100).collect(),
                channel_id: message.channel_id.clone(),
                peer_id: if message.channel_id.is_none() {
                    Some(message.author_id.clone())
                } else {
                    None
                },
            });
            if notifications.recent.len() > 10 {
                notifications.recent.remove(0);
            }
        }
        notifications.recent.reverse();
        notifications
    }

    pub fn notifications(&self, token: &str) -> Result<Notifications, String> {
        let mut rt = self.runtime.lock().unwrap();
        let profile = Self::authenticate(&mut rt, token)?;
        Ok(Self::unread(&rt.store, &profile))
    }

    pub fn mark_read(
        &self,
        token: &str,
        channel: Option<&str>,
        peer: Option<&str>,
        upto: Option<u64>,
    ) -> Result<Notifications, String> {
        let mut rt = self.runtime.lock().unwrap();
        let profile = Self::authenticate(&mut rt, token)?;
        let key = match (channel, peer) {
            (Some(c), None) if rt.store.channels.iter().any(|v| v.id == c) => {
                format!("channel:{c}")
            }
            (None, Some(p)) if rt.store.profiles.iter().any(|v| v.id == p) && p != profile => {
                format!("dm:{p}")
            }
            _ => return Err("chat-invalid-conversation".into()),
        };
        let latest = rt
            .store
            .messages
            .iter()
            .filter(|m| Self::visible(m, &profile))
            .filter(|m| {
                if let Some(c) = channel {
                    m.channel_id.as_deref() == Some(c)
                } else {
                    m.channel_id.is_none()
                        && (m.author_id == peer.unwrap() || m.recipient_id.as_deref() == peer)
                }
            })
            .map(|m| m.id)
            .max()
            .unwrap_or(0);
        let upto = upto.unwrap_or(latest).min(latest);
        if upto
            > rt.store
                .reads
                .get(&profile)
                .and_then(|r| r.get(&key))
                .copied()
                .unwrap_or(0)
        {
            let mut next = rt.store.clone();
            next.reads
                .entry(profile.clone())
                .or_default()
                .insert(key, upto);
            self.save(&next)?;
            rt.store = next;
        }
        Ok(Self::unread(&rt.store, &profile))
    }
}
