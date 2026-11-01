//! Application catalogue adapter for shader contributions.
use crate::core::state::ControlPanelState;

pub fn sync_shader_themes(state: &mut ControlPanelState) {
    let selected_id = state
        .themes
        .get(state.selected_theme_index)
        .map(|theme| theme.id.clone());

    // Never keep auto-injected plugin_shader dummy themes in the presentation list
    state
        .themes
        .retain(|theme| !theme.id.starts_with("plugin_shader:"));

    // If an existing theme (created/saved by the user) uses a plugin shader, sync the custom shader WGSL/GLSL
    for theme in &mut state.themes {
        if theme.bg_type == "shader" {
            for plugin in state.settings.plugins.iter().filter(|p| p.enabled) {
                for shader in &plugin.package.shaders {
                    let kind_id = format!("{}:{}", plugin.package.id, shader.id);
                    if theme.options.shader.kind == kind_id {
                        theme.options.shader.custom = Some(shader.clone());
                    }
                }
            }
        }
    }

    if let Some(id) = selected_id {
        state.selected_theme_index = state
            .themes
            .iter()
            .position(|theme| theme.id == id)
            .unwrap_or(0);
    } else if state.selected_theme_index >= state.themes.len() {
        state.selected_theme_index = state.themes.len().saturating_sub(1);
    }

    state.live_theme_index = state
        .themes
        .iter()
        .position(|theme| theme.id == state.live_theme_id)
        .unwrap_or(0);
    if let Some(live_theme) = state.themes.get(state.live_theme_index) {
        state.live_theme_id = live_theme.id.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::PresentationTheme;

    #[test]
    fn plugin_shader_themes_are_not_auto_recreated_and_legacy_ones_are_purged() {
        let mut state = ControlPanelState::default();
        state.settings.plugins = crate::core::plugins::bundled();

        // 1. Initial state: normal user themes
        let initial_count = state.themes.len();

        // 2. Add an auto-injected theme as if from an older session
        let mut old_theme = PresentationTheme::default();
        old_theme.id = "plugin_shader:omni-presenter.stage-tools:plasma".into();
        old_theme.name = "Plasma · Ferramentas de palco".into();
        state.themes.push(old_theme);
        assert_eq!(state.themes.len(), initial_count + 1);

        // 3. sync_shader_themes must purge it and NOT recreate it
        sync_shader_themes(&mut state);
        assert_eq!(state.themes.len(), initial_count);
        assert!(!state.themes.iter().any(|t| t.id.starts_with("plugin_shader:")));

        // 4. Multiple calls must stay clean and never recreate deleted theme
        sync_shader_themes(&mut state);
        assert_eq!(state.themes.len(), initial_count);
        assert!(!state.themes.iter().any(|t| t.id.starts_with("plugin_shader:")));

        // 5. User-created theme that references the plugin shader has its shader definition synchronized
        let mut custom_theme = PresentationTheme::default();
        custom_theme.id = "my_plasma_theme".into();
        custom_theme.bg_type = "shader".into();
        custom_theme.options.shader.kind = "omni-presenter.stage-tools:plasma".into();
        assert!(custom_theme.options.shader.custom.is_none());
        state.themes.push(custom_theme);

        sync_shader_themes(&mut state);
        let updated = state.themes.iter().find(|t| t.id == "my_plasma_theme").unwrap();
        assert!(updated.options.shader.custom.is_some());
        assert_eq!(updated.options.shader.custom.as_ref().unwrap().id, "plasma");
    }
}

