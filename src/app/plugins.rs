use super::PresentationApp;
use crate::core::{
    models::{PlaylistItem, PlaylistItemKind},
    plugins::{
        PluginRequest, defaults,
        runtime::{self, HostAction},
    },
    preferences::new_id,
};

impl PresentationApp {
    pub(super) fn run_plugin_request(&mut self) {
        let Some(request) = self.control_state.settings_ui.plugin_request.take() else {
            return;
        };
        if let Err(error) = self.execute_plugin_request(request) {
            let language = self.control_state.settings.language;
            self.control_state
                .notify(language.format("plugins-runtime-error", &[("error", &error)]));
        }
    }

    fn execute_plugin_request(&mut self, request: PluginRequest) -> Result<(), String> {
        let id = match &request {
            PluginRequest::Command { plugin, .. }
            | PluginRequest::Template { plugin, .. }
            | PluginRequest::OpenPanel { plugin, .. } => {
                plugin
            }
        };
        let plugin = self
            .control_state
            .settings
            .plugins
            .iter()
            .find(|plugin| plugin.enabled && &plugin.package.id == id)
            .cloned()
            .ok_or("plugins-unavailable")?;
        if let PluginRequest::Template { index, .. } = request {
            let mut scene = plugin
                .package
                .templates
                .get(index)
                .cloned()
                .ok_or("plugins-unavailable")?;
            scene.id = new_id("design");
            for layer in &mut scene.layers {
                layer.id = new_id("layer");
            }
            self.db.save_design(&scene).map_err(|e| e.to_string())?;
            self.slide_deck.designs.push(scene.clone());
            self.slide_deck.preview_design = Some(scene);
            self.control_state.active_sidebar_page = crate::core::state::SidebarPage::Design;
            return Ok(());
        }
        let PluginRequest::Command { command, .. } = request else {
            unreachable!()
        };
        let command = plugin
            .package
            .commands
            .iter()
            .find(|item| item.id == command)
            .ok_or("plugins-unavailable")?;
        let mut props = defaults(&plugin.package.settings);
        for field in &plugin.package.settings {
            if let Some(value) = plugin
                .settings
                .get(&field.id)
                .filter(|value| field.accepts(value))
            {
                props.insert(field.id.clone(), value.clone());
            }
        }
        let title = if self.slide_deck.title().trim().is_empty() {
            self.control_state.settings.language.text("library-texts")
        } else {
            self.slide_deck.title()
        };
        let context = serde_json::json!({"title": title, "text": self.slide_deck.raw_text(),
            "operator": self.control_state.settings.operator().name, "service": self.control_state.settings.service().name,
            "language": self.control_state.settings.language});
        let actions = runtime::actions(command, &props, &context)?;
        // Validate the full result before applying any actions.
        for action in &actions {
            let valid = match action {
                HostAction::Notify { text } => text.len() <= 16_384,
                HostAction::PreviewText { title, text }
                | HostAction::SaveText { title, text }
                | HostAction::QueueText { title, text } => {
                    !title.trim().is_empty() && title.len() <= 200 && text.len() <= 65_536
                }
            };
            if !valid {
                return Err("plugins-invalid-output".into());
            }
        }
        for action in actions {
            match action {
                HostAction::Notify { text } => self.control_state.notify(text),
                HostAction::PreviewText { title, text } => {
                    self.slide_deck
                        .load_custom_content(title, plugin.package.name.clone(), text)
                }
                HostAction::SaveText { title, text } => {
                    self.db
                        .add_custom_text(&title, &text, "")
                        .map_err(|e| e.to_string())?;
                    self.control_state.reload_all_selection_items(&self.db);
                }
                HostAction::QueueText { title, text } => {
                    self.control_state.playlist.push(PlaylistItem {
                        id: new_id("plugin_text"),
                        title,
                        subtitle: plugin.package.name.clone(),
                        kind: PlaylistItemKind::Text,
                        played: false,
                        content: text,
                    })
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::db::AppDb;
    #[test]
    fn plugin_commands_execute_host_actions_without_replacing_live_content() {
        let mut app = PresentationApp::with_db(AppDb::open_in_memory().unwrap());
        app.slide_deck
            .load_custom_content("Aviso".into(), "".into(), "Olá equipe".into());
        let live = app.slide_deck.projected_text().to_owned();
        let initial = app.control_state.playlist.len();
        // The reset-scoreboard command only mutates plugin state and must not touch the live output.
        app.execute_plugin_request(PluginRequest::Command {
            plugin: "omni-presenter.stage-tools".into(),
            command: "reset-scoreboard".into(),
        })
        .unwrap();
        assert_eq!(app.slide_deck.projected_text(), live);
        assert_eq!(app.control_state.playlist.len(), initial);
        app.control_state
            .settings
            .plugins
            .iter_mut()
            .find(|plugin| plugin.package.id == "omni-presenter.stage-tools")
            .unwrap()
            .enabled = false;
        assert!(
            app.execute_plugin_request(PluginRequest::Command {
                plugin: "omni-presenter.stage-tools".into(),
                command: "reset-scoreboard".into()
            })
            .is_err()
        );
    }
}
