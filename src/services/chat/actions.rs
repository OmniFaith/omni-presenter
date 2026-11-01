use super::*;
use std::time::Duration;

impl ChatService {
    pub fn request_action(
        &self,
        token: &str,
        message_id: u64,
        action: &str,
    ) -> Result<ActionResult, String> {
        let mut rt = self.runtime.lock().unwrap();
        let profile = Self::authenticate(&mut rt, token)?;
        if !["shortcut", "playlist", "project"].contains(&action) {
            return Err("services-invalid-action".into());
        }
        let message = rt
            .store
            .messages
            .iter()
            .find(|m| m.id == message_id && Self::visible(m, &profile))
            .ok_or("services-content-not-found-in-this-conversation")?;
        let content = message
            .shared
            .clone()
            .ok_or("services-this-message-does-not-contain-shared-content")?;
        if rt.actions.len() >= 64 {
            return Err("services-wait-for-previous-actions-to-finish".into());
        }
        let id = secret()?;
        let result = ActionResult {
            id: id.clone(),
            status: "queued".into(),
            error: None,
        };
        rt.actions.push_back(ChatAction {
            id: id.clone(),
            profile_id: profile.clone(),
            action: action.into(),
            content,
            message_id,
        });
        rt.results
            .retain(|_, (_, _, created)| created.elapsed() < Duration::from_secs(3600));
        rt.results
            .insert(id, (profile, result.clone(), Instant::now()));
        Ok(result)
    }

    pub fn action_status(&self, token: &str, id: &str) -> Result<ActionResult, String> {
        let mut rt = self.runtime.lock().unwrap();
        let profile = Self::authenticate(&mut rt, token)?;
        rt.results
            .get(id)
            .filter(|(owner, _, _)| owner == &profile)
            .map(|(_, result, _)| result.clone())
            .ok_or("services-action-not-found".into())
    }

    pub(crate) fn pending_actions(&self) -> Vec<ChatAction> {
        self.runtime.lock().unwrap().actions.drain(..).collect()
    }

    pub(crate) fn finish_action(&self, id: &str, result: Result<(), String>) {
        if let Some((_, status, _)) = self.runtime.lock().unwrap().results.get_mut(id) {
            status.status = if result.is_ok() { "applied" } else { "error" }.into();
            status.error = result.err();
        }
    }
}
