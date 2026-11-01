use eframe::egui;

use super::PresentationApp;

impl PresentationApp {
    /// Persists the pending settings draft and returns the monitor index to move to.
    pub(super) fn save_settings_draft(&mut self) -> Option<usize> {
        if !self.control_state.settings_ui.save_requested {
            return None;
        }
        self.control_state.settings_ui.save_requested = false;
        let draft = self.control_state.settings_ui.draft.clone()?;
        if let Err(error) = self.db.save_settings(&draft) {
            self.control_state.settings_ui.error = self
                .control_state
                .settings
                .language
                .format("workspace-could-not-save", &[("error", &error.to_string())]);
            return None;
        }
        let previous = &self.control_state.settings;
        let restart = (
            previous.server_enabled,
            previous.server_port,
            previous.allow_lan,
            previous.chat_enabled,
        ) != (
            draft.server_enabled,
            draft.server_port,
            draft.allow_lan,
            draft.chat_enabled,
        ) || ((draft.server_enabled || draft.chat_enabled)
            && self.browser_server.is_none());
        self.control_state.settings = draft;
        crate::services::plugins::sync_shader_themes(&mut self.control_state);
        if let Some(service) = &self.control_state.chat.service
            && let Err(error) = service.sync_profiles(&self.control_state.settings.operators) {
                self.control_state.chat.error = error;
            }
        self.control_state.apply_profiles();
        self.control_state.chat.last_sync = None;
        self.last_saved_settings =
            serde_json::to_string(&self.control_state.settings).unwrap_or_default();
        self.settings_changed_at = None;
        if restart {
            self.restart_browser_server();
        }
        if self.control_state.settings.server_enabled
            && !self.control_state.settings_ui.server_running
        {
            self.control_state.settings_ui.error =
                self.control_state.settings_ui.server_status.clone();
        } else {
            self.control_state.settings_ui.open = false;
            self.control_state.settings_ui.draft = None;
            self.control_state.notify(self.control_state.settings.language.text("services-preferences-saved"));
        }
        self.monitors
            .iter()
            .find(|m| m.name == self.control_state.settings.monitor_name)
            .map(|m| m.winit_index)
    }

    /// Debounced autosave of operator/service profiles (700 ms after last change).
    pub(super) fn autosave_profiles(&mut self, ctx: &egui::Context) {
        if self.control_state.settings_ui.open {
            return;
        }
        self.control_state.capture_profiles();
        let json = serde_json::to_string(&self.control_state.settings).unwrap_or_default();
        if json == self.last_saved_settings {
            self.settings_changed_at = None;
            return;
        }
        if self.pending_settings_json != json {
            self.pending_settings_json = json.clone();
            self.settings_changed_at = Some(std::time::Instant::now());
        }
        if self
            .settings_changed_at
            .is_some_and(|t| t.elapsed().as_millis() >= 700)
        {
            match self.db.save_settings(&self.control_state.settings) {
                Ok(()) => {
                    self.last_saved_settings = json;
                    self.settings_changed_at = None;
                }
                Err(error) => {
                    self.control_state.notify(
                        self.control_state.settings.language.format(
                            "services-could-not-save-preferences",
                            &[("error", &error.to_string())],
                        ),
                    );
                    self.settings_changed_at = Some(std::time::Instant::now());
                }
            }
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(750));
    }
}
