use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

impl ChatService {
    pub fn react(&self, token: &str, id: u64, emoji: &str) -> Result<Message, String> {
        if !crate::core::emoji::valid(emoji) {
            return Err("services-choose-an-emoji-from-the-picker".into());
        }
        let mut rt = self.runtime.lock().unwrap();
        let profile = Self::authenticate(&mut rt, token)?;
        let mut next = rt.store.clone();
        let message = next
            .messages
            .iter_mut()
            .find(|m| m.id == id && Self::visible(m, &profile) && !m.deleted)
            .ok_or("services-message-not-found".to_owned())?;
        if let Some(reaction) = message.reactions.iter_mut().find(|r| r.emoji == emoji) {
            if reaction.users.contains(&profile) {
                reaction.users.retain(|p| p != &profile);
            } else {
                reaction.users.push(profile);
            }
        } else {
            message.reactions.push(Reaction {
                emoji: emoji.into(),
                users: vec![profile],
            });
        }
        message.reactions.retain(|r| !r.users.is_empty());
        let result = message.clone();
        self.save(&next)?;
        rt.store = next;
        Ok(result)
    }

    pub fn delete_message(&self, token: &str, id: u64) -> Result<Message, String> {
        let mut rt = self.runtime.lock().unwrap();
        let profile = Self::authenticate(&mut rt, token)?;
        let mut next = rt.store.clone();
        let message = next
            .messages
            .iter_mut()
            .find(|m| m.id == id && m.author_id == profile)
            .ok_or("services-you-can-only-delete-your-own-messages".to_owned())?;
        message.deleted = true;
        message.title.clear();
        message.text = "services-message-deleted".into();
        message.file = None;
        message.shared = None;
        message.reactions.clear();
        let result = message.clone();
        self.save(&next)?;
        rt.store = next;
        let cancelled: Vec<_> = rt
            .actions
            .iter()
            .filter(|a| a.message_id == id)
            .map(|a| a.id.clone())
            .collect();
        rt.actions.retain(|a| a.message_id != id);
        for id in cancelled {
            if let Some((_, result, _)) = rt.results.get_mut(&id) {
                result.status = "error".into();
                result.error = Some("services-the-message-has-been-deleted".into());
            }
        }
        Ok(result)
    }

    pub fn snapshot(
        &self,
        token: &str,
        channel: Option<&str>,
        peer: Option<&str>,
        query: &str,
        before: Option<u64>,
    ) -> Result<Snapshot, String> {
        let mut rt = self.runtime.lock().unwrap();
        let profile = Self::authenticate(&mut rt, token)?;
        let query = crate::core::bible::fold(query);
        let mut messages: Vec<_> = rt
            .store
            .messages
            .iter()
            .rev()
            .filter(|m| {
                Self::visible(m, &profile)
                    && before.is_none_or(|id| m.id < id)
                    && if let Some(peer) = peer {
                        m.channel_id.is_none()
                            && ((m.author_id == profile && m.recipient_id.as_deref() == Some(peer))
                                || (m.author_id == peer
                                    && m.recipient_id.as_deref() == Some(&profile)))
                    } else {
                        channel.is_none_or(|c| m.channel_id.as_deref() == Some(c))
                    }
            })
            .filter(|m| {
                query.is_empty()
                    || crate::core::bible::fold(&format!("{} {} {}", m.author, m.title, m.text))
                        .contains(&query)
            })
            .take(100)
            .cloned()
            .collect();
        messages.reverse();
        let latest_id = rt
            .store
            .messages
            .iter()
            .rev()
            .find(|m| Self::visible(m, &profile))
            .map(|m| m.id)
            .unwrap_or(0);
        Ok(Snapshot {
            profiles: Self::profile_views(&rt),
            channels: rt.store.channels.clone(),
            messages,
            latest_id,
            notifications: Self::unread(&rt.store, &profile),
        })
    }

    pub fn send(&self, token: &str, request: SendMessage) -> Result<Message, String> {
        let mut rt = self.runtime.lock().unwrap();
        let profile = Self::authenticate(&mut rt, token)?;
        match (&request.channel_id, &request.recipient_id) {
            (Some(channel), None) if rt.store.channels.iter().any(|c| &c.id == channel) => {}
            (None, Some(peer))
                if rt.store.profiles.iter().any(|p| &p.id == peer) && peer != &profile => {}
            _ => return Err("services-choose-a-valid-channel-or-recipient".into()),
        }
        let kind = if request.kind.is_empty() {
            "text"
        } else {
            &request.kind
        };
        let mut shared = None;
        let (title, text, file) = match kind {
            "text" | "emoji" if !request.text.trim().is_empty() && request.text.len() <= 32000 => {
                (String::new(), request.text.trim().into(), None)
            }
            "bible" => {
                let bible = self.bible_library()?;
                let version = if request.version.is_empty() {
                    crate::core::bible::DEFAULT_BIBLE
                } else {
                    &request.version
                };
                let bible = bible.get(version).ok_or("services-bible-translation-not-found")?;
                let (book, chapter, start, end) = bible
                    .reference(&request.reference)
                    .ok_or("services-invalid-bible-reference")?;
                let passage = bible
                    .passage(book, chapter, start, end)
                    .ok_or("services-passage-unavailable")?;
                let content = if request.merge {
                    passage.content.clone()
                } else {
                    (start..=end)
                        .filter_map(|verse| bible.passage(book, chapter, verse, verse))
                        .map(|p| p.content)
                        .collect::<Vec<_>>()
                        .join("\n\n")
                };
                shared = Some(SharedContent::Bible {
                    reference: passage.reference.clone(),
                    version: passage.version.clone(),
                    content,
                });
                (
                    format!("{} · {}", passage.reference, passage.version),
                    passage.content,
                    None,
                )
            }
            "song" => {
                let song = self
                    .db
                    .get_all_songs()
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .find(|s| s.id == request.song_id && s.id.is_some())
                    .ok_or("services-song-not-found")?;
                let plain = crate::core::markup::strip(&song.text);
                shared = Some(SharedContent::Song {
                    id: song.id.unwrap(),
                    title: song.title.clone(),
                    artist: song.artist.clone(),
                    content: plain.clone(),
                });
                (format!("{} · {}", song.title, song.artist), plain, None)
            }
            "file" | "audio" => {
                let id = request.file_id.as_deref().ok_or("services-attach-a-file")?;
                let (owner, name, mime, size) = self
                    .db
                    .chat_file_info(id)?
                    .ok_or("services-file-not-found")?;
                if owner != profile {
                    return Err("services-this-file-belongs-to-another-profile".into());
                }
                if kind == "audio" && !mime.starts_with("audio/") {
                    return Err("services-invalid-audio-format".into());
                }
                let attachment = Attachment {
                    id: id.into(),
                    name: name.clone(),
                    mime: mime.clone(),
                    size,
                };
                shared = Some(SharedContent::Media { file: attachment });
                (
                    name.clone(),
                    request.text.chars().take(32000).collect(),
                    Some(Attachment {
                        id: id.into(),
                        name,
                        mime,
                        size,
                    }),
                )
            }
            _ => return Err("services-empty-message-or-invalid-type".into()),
        };
        let message = Message {
            id: rt.store.sequence + 1,
            author_id: profile.clone(),
            author: rt
                .store
                .profiles
                .iter()
                .find(|p| p.id == profile)
                .unwrap()
                .name
                .clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            channel_id: request.channel_id,
            recipient_id: request.recipient_id,
            kind: kind.into(),
            title,
            text,
            file,
            shared,
            reactions: Vec::new(),
            deleted: false,
        };
        let mut next = rt.store.clone();
        next.sequence = message.id;
        next.messages.push(message.clone());
        self.save(&next)?;
        rt.store = next;
        Ok(message)
    }

    pub fn message(&self, token: &str, id: u64) -> Result<Message, String> {
        let mut rt = self.runtime.lock().unwrap();
        let profile = Self::authenticate(&mut rt, token)?;
        rt.store
            .messages
            .iter()
            .find(|m| m.id == id && Self::visible(m, &profile))
            .cloned()
            .ok_or("services-message-not-found".into())
    }
}
