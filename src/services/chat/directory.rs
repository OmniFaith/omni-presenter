use super::*;

impl ChatService {
    pub fn update_channel(
        &self,
        token: &str,
        id: &str,
        name: &str,
        tool: &str,
        description: &str,
    ) -> Result<(), String> {
        let mut runtime = self.runtime.lock().unwrap();
        Self::authenticate(&mut runtime, token)?;
        if id == "general" {
            return Err("chat-general-protected".into());
        }
        validate_channel(name, tool)?;
        let mut next = runtime.store.clone();
        let channel = next
            .channels
            .iter_mut()
            .find(|channel| channel.id == id)
            .ok_or("chat-channel-missing")?;
        channel.name = name.trim().into();
        channel.tool = tool.into();
        channel.description = description.chars().take(500).collect();
        self.save(&next)?;
        runtime.store = next;
        Ok(())
    }

    pub fn delete_channels(&self, token: &str, ids: &[String]) -> Result<(), String> {
        let mut runtime = self.runtime.lock().unwrap();
        Self::authenticate(&mut runtime, token)?;
        if ids.iter().any(|id| id == "general") {
            return Err("chat-general-protected".into());
        }
        if ids.iter().any(|id| {
            !runtime
                .store
                .channels
                .iter()
                .any(|channel| &channel.id == id)
        }) {
            return Err("chat-channel-missing".into());
        }
        let mut next = runtime.store.clone();
        let removed_messages: Vec<_> = next
            .messages
            .iter()
            .filter(|message| {
                message
                    .channel_id
                    .as_ref()
                    .is_some_and(|id| ids.contains(id))
            })
            .map(|message| message.id)
            .collect();
        next.channels.retain(|channel| !ids.contains(&channel.id));
        next.messages
            .retain(|message| !removed_messages.contains(&message.id));
        for reads in next.reads.values_mut() {
            for id in ids {
                reads.remove(&format!("channel:{id}"));
            }
        }
        self.save(&next)?;
        runtime.store = next;
        let cancelled: Vec<_> = runtime
            .actions
            .iter()
            .filter(|action| removed_messages.contains(&action.message_id))
            .map(|action| action.id.clone())
            .collect();
        runtime
            .actions
            .retain(|action| !cancelled.contains(&action.id));
        for id in cancelled {
            if let Some((_, result, _)) = runtime.results.get_mut(&id) {
                result.status = "error".into();
                result.error = Some("chat-channel-deleted".into());
            }
        }
        Ok(())
    }

    pub fn move_channels(
        &self,
        token: &str,
        ids: &[String],
        target: &str,
        after: bool,
    ) -> Result<(), String> {
        let mut runtime = self.runtime.lock().unwrap();
        Self::authenticate(&mut runtime, token)?;
        if ids
            .iter()
            .chain(std::iter::once(&target.to_owned()))
            .any(|id| {
                !runtime
                    .store
                    .channels
                    .iter()
                    .any(|channel| &channel.id == id)
            })
        {
            return Err("chat-channel-missing".into());
        }
        let mut next = runtime.store.clone();
        if crate::core::interaction::move_items(
            &mut next.channels,
            ids,
            &target.to_owned(),
            after,
            |channel| &channel.id,
        ) {
            self.save(&next)?;
            runtime.store = next;
        }
        Ok(())
    }

    pub fn mark_channels_read(&self, token: &str, ids: &[String]) -> Result<(), String> {
        let mut runtime = self.runtime.lock().unwrap();
        let profile = Self::authenticate(&mut runtime, token)?;
        if ids.iter().any(|id| {
            !runtime
                .store
                .channels
                .iter()
                .any(|channel| &channel.id == id)
        }) {
            return Err("chat-channel-missing".into());
        }
        let mut next = runtime.store.clone();
        for id in ids {
            let latest = next
                .messages
                .iter()
                .filter(|message| message.channel_id.as_ref() == Some(id))
                .map(|message| message.id)
                .max()
                .unwrap_or(0);
            next.reads
                .entry(profile.clone())
                .or_default()
                .insert(format!("channel:{id}"), latest);
        }
        self.save(&next)?;
        runtime.store = next;
        Ok(())
    }
    pub fn create_channel(
        &self,
        token: &str,
        name: &str,
        tool: &str,
        description: &str,
    ) -> Result<Channel, String> {
        let mut rt = self.runtime.lock().unwrap();
        Self::authenticate(&mut rt, token)?;
        validate_channel(name, tool)?;
        let channel = Channel {
            id: crate::core::preferences::new_id("channel"),
            name: name.trim().into(),
            tool: tool.into(),
            description: description.chars().take(500).collect(),
        };
        let mut next = rt.store.clone();
        next.channels.push(channel.clone());
        self.save(&next)?;
        rt.store = next;
        Ok(channel)
    }

    pub fn songs(&self, token: &str, query: &str) -> Result<serde_json::Value, String> {
        self.heartbeat(token)?;
        let songs = self.db.get_all_songs().map_err(|e| e.to_string())?;
        let query = crate::core::search::SearchQuery::new(query);
        Ok(serde_json::Value::Array(
            songs
                .into_iter()
                .filter(|song| {
                    query.matches(&crate::core::bible::fold(&format!(
                        "{} {} {}",
                        song.title, song.artist, song.text
                    )))
                })
                .take(100)
                .map(|s| serde_json::json!({"id":s.id,"title":s.title,"artist":s.artist}))
                .collect(),
        ))
    }

    pub fn versions(&self, token: &str) -> Result<serde_json::Value, String> {
        self.heartbeat(token)?;
        let bible = self.bible_library()?;
        Ok(serde_json::Value::Array(
            bible
                .catalog()
                .into_iter()
                .map(|(id, name)| serde_json::json!({"id":id,"name":name}))
                .collect(),
        ))
    }
}

fn validate_channel(name: &str, tool: &str) -> Result<(), String> {
    if name.trim().is_empty() || name.chars().count() > 80 {
        return Err("chat-channel-name-required".into());
    }
    if !["general", "bible", "songs", "media"].contains(&tool) {
        return Err("chat-invalid-tool".into());
    }
    Ok(())
}
