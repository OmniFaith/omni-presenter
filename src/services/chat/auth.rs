use super::*;
use crate::core::preferences::OperatorProfile;
use std::{
    hash::{Hash, Hasher},
    time::{Duration, Instant},
};

impl ChatService {
    pub fn new(db: AppDb, operators: &[OperatorProfile]) -> Result<Self, String> {
        db.chat_setup()?;
        let mut store: Store = db
            .chat_load()?
            .map(|v| serde_json::from_str(&v).map_err(|e| e.to_string()))
            .transpose()?
            .unwrap_or_default();
        // Only remove untouched legacy examples; channels with real conversations belong to the user.
        store.channels.retain(|channel| {
            let example = [
                ("bible", "bíblia"),
                ("songs", "louvor"),
                ("media", "mídias"),
            ]
            .iter()
            .any(|(id, name)| {
                channel.id == *id && channel.name == *name && channel.description.is_empty()
            });
            !example
                || store
                    .messages
                    .iter()
                    .any(|message| message.channel_id.as_deref() == Some(&channel.id))
        });
        if !store.channels.iter().any(|channel| channel.id == "general") {
            store.channels.insert(
                0,
                Channel {
                    id: "general".into(),
                    name: "Geral".into(),
                    tool: "general".into(),
                    description: String::new(),
                },
            );
        }
        let service = Self {
            db,
            runtime: Arc::new(Mutex::new(Runtime {
                store,
                sessions: HashMap::new(),
                actions: VecDeque::new(),
                results: HashMap::new(),
            })),
        };
        service.sync_profiles(operators)?;
        service.save(&service.runtime.lock().unwrap().store)?;
        Ok(service)
    }

    pub(super) fn save(&self, store: &Store) -> Result<(), String> {
        self.db
            .chat_save(&serde_json::to_string(store).map_err(|e| e.to_string())?)
    }

    pub fn sync_profiles(&self, operators: &[OperatorProfile]) -> Result<(), String> {
        let mut runtime = self.runtime.lock().unwrap();
        let mut next = runtime.store.clone();
        next.profiles
            .retain(|p| operators.iter().any(|o| o.id == p.id));
        for operator in operators {
            if let Some(profile) = next.profiles.iter_mut().find(|p| p.id == operator.id) {
                if profile.password_hash != operator.web_password_hash {
                    runtime.sessions.retain(|_, session| session.profile_id != operator.id);
                }
                profile.name = operator.name.clone();
                profile.avatar = operator.avatar.clone();
                profile.password_hash = operator.web_password_hash.clone();
            } else {
                next.profiles.push(Profile {
                    id: operator.id.clone(),
                    name: operator.name.clone(),
                    password_hash: operator.web_password_hash.clone(),
                    avatar: operator.avatar.clone(),
                });
            }
        }
        if serde_json::to_string(&next.profiles).ok()
            != serde_json::to_string(&runtime.store.profiles).ok()
        {
            self.save(&next)?;
            runtime.store = next;
        }
        let ids: Vec<_> = runtime
            .store
            .profiles
            .iter()
            .map(|p| p.id.clone())
            .collect();
        runtime.sessions.retain(|_, s| {
            ids.contains(&s.profile_id) && s.seen.elapsed() < Duration::from_secs(86400)
        });
        Ok(())
    }

    pub fn profiles(&self) -> Vec<ProfileView> {
        let rt = self.runtime.lock().unwrap();
        Self::profile_views(&rt)
    }

    pub(super) fn profile_views(rt: &Runtime) -> Vec<ProfileView> {
        rt.store
            .profiles
            .iter()
            .map(|p| ProfileView {
                id: p.id.clone(),
                name: p.name.clone(),
                avatar_url: p.avatar.as_ref().map(|bytes| {
                    let mut hash = std::collections::hash_map::DefaultHasher::new();
                    bytes.hash(&mut hash);
                    format!("/api/chat/avatars/{}?v={:x}", p.id, hash.finish())
                }),
                online: rt
                    .sessions
                    .values()
                    .any(|s| s.profile_id == p.id && s.seen.elapsed() < Duration::from_secs(30)),
            })
            .collect()
    }

    #[cfg(test)]
    pub fn access_code(&self, profile: &str) -> Option<String> {
        self.runtime
            .lock()
            .unwrap()
            .store
            .profiles
            .iter()
            .find(|p| p.id == profile)
            .map(|_| crate::services::credentials::DEFAULT_PASSWORD.into())
    }

    pub fn avatar(&self, profile: &str) -> Option<Vec<u8>> {
        self.runtime
            .lock()
            .unwrap()
            .store
            .profiles
            .iter()
            .find(|p| p.id == profile)
            .and_then(|p| p.avatar.clone())
    }

    pub fn login(&self, profile_id: &str, password: &str) -> Result<String, String> {
        let hash = self.runtime.lock().unwrap()
            .store
            .profiles
            .iter()
            .find(|p| p.id == profile_id)
            .map(|p| p.password_hash.clone());
        if !hash.is_some_and(|hash| crate::services::credentials::verify(&hash, password)) {
            return Err("services-invalid-profile-or-password".into());
        }
        self.login_local(profile_id)
    }

    /// Native operator sessions do not need to know the browser password.
    pub(crate) fn login_local(&self, profile_id: &str) -> Result<String, String> {
        let mut rt = self.runtime.lock().unwrap();
        if !rt.store.profiles.iter().any(|p| p.id == profile_id) {
            return Err("services-profile-not-found".into());
        }
        let token = secret()?;
        rt.sessions.insert(
            token.clone(),
            Session {
                profile_id: profile_id.into(),
                seen: Instant::now(),
            },
        );
        Ok(token)
    }

    pub fn logout(&self, token: &str) {
        self.runtime.lock().unwrap().sessions.remove(token);
    }

    pub(super) fn authenticate(rt: &mut Runtime, token: &str) -> Result<String, String> {
        let session = rt
            .sessions
            .get_mut(token)
            .ok_or("services-session-expired-sign-in-again")?;
        if session.seen.elapsed() > Duration::from_secs(86400) {
            return Err("services-session-expired-sign-in-again".into());
        }
        session.seen = Instant::now();
        Ok(session.profile_id.clone())
    }

    pub fn heartbeat(&self, token: &str) -> Result<(), String> {
        Self::authenticate(&mut self.runtime.lock().unwrap(), token).map(|_| ())
    }

    pub fn session_profile(&self, token: &str) -> Result<String, String> {
        Self::authenticate(&mut self.runtime.lock().unwrap(), token)
    }

    pub fn certificate(&self, host: &str) -> Result<(String, String), String> {
        self.db.chat_certificate(host)
    }
}
