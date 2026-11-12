use super::*;
use crate::core::preferences::ServiceProfile;
use crate::services::db::AppDb;

#[test]
fn last_clicked_theme_survives_database_reload_without_overwriting_the_event_default() {
    let db = AppDb::open_in_memory().unwrap();
    let mut state = ControlPanelState::default();
    let default_theme = state.settings.service().theme_id.clone();
    let live_theme = state.live_theme_id.clone();
    state.selected_theme_index = 2;
    let selected_theme = state.preview_theme().id.clone();
    state.capture_profiles();
    assert_eq!(
        state.live_theme_id, live_theme,
        "Saving preview must not change live output"
    );
    assert_eq!(state.settings.service().theme_id, default_theme);
    db.save_settings(&state.settings).unwrap();

    let mut restored = ControlPanelState {
        settings: db.load_settings().unwrap().unwrap(),
        ..Default::default()
    };
    // IDs, not catalogue indices, must survive reordered theme lists.
    restored.themes.reverse();
    restored.apply_profiles();
    assert_eq!(restored.preview_theme().id, selected_theme);
    assert_eq!(restored.settings.service().theme_id, default_theme);
}

#[test]
fn theme_selection_is_independent_per_event_and_does_not_replace_live() {
    let mut state = ControlPanelState::default();
    let first_service = state.settings.active_service.clone();
    let second = ServiceProfile {
        id: "theme_second_service".into(),
        theme_id: state.themes[1].id.clone(),
        ..Default::default()
    };
    state.settings.services.push(second.clone());
    let live_theme = state.live_theme_id.clone();
    state.selected_theme_index = 2;
    let first_selection = state.preview_theme().id.clone();
    state.switch_service(&second.id);
    assert_eq!(state.preview_theme().id, second.theme_id);
    state.selected_theme_index = 3;
    let second_selection = state.preview_theme().id.clone();
    state.switch_service(&first_service);
    assert_eq!(state.preview_theme().id, first_selection);
    state.switch_service(&second.id);
    assert_eq!(state.preview_theme().id, second_selection);
    assert_eq!(state.live_theme_id, live_theme);
}

#[test]
fn deleted_selection_falls_back_to_the_event_default() {
    let mut state = ControlPanelState::default();
    let service = &mut state.settings.services[0];
    service.theme_id = state.themes[1].id.clone();
    service.last_selected_theme_id = Some("deleted_theme".into());
    service.last_selected_theme_default_id = Some(service.theme_id.clone());
    let expected = service.theme_id.clone();
    state.apply_profiles();
    assert_eq!(state.preview_theme().id, expected);
}

#[test]
fn legacy_profiles_still_restore_their_configured_theme() {
    let mut state = ControlPanelState::default();
    let mut json = serde_json::to_value(ServiceProfile::default()).unwrap();
    json.as_object_mut()
        .unwrap()
        .remove("last_selected_theme_id");
    json.as_object_mut().unwrap().remove("last_selected_theme_default_id");
    json["theme_id"] = state.themes[1].id.clone().into();
    let service: ServiceProfile = serde_json::from_value(json).unwrap();
    assert!(service.last_selected_theme_id.is_none());
    state.settings.services[0] = service.clone();
    state.settings.active_service = service.id;
    state.apply_profiles();
    assert_eq!(state.preview_theme().id, service.theme_id);
}

#[test]
fn explicitly_changing_the_event_default_replaces_the_remembered_selection() {
    let mut state = ControlPanelState {
        selected_theme_index: 2,
        ..Default::default()
    };
    state.capture_profiles();
    let new_default = state.themes[1].id.clone();
    state.event_editor_draft = Some(events::EventEditorDraft {
        id: Some(state.settings.active_service.clone()),
        name: state.settings.service().name.clone(),
        theme_id: new_default.clone(),
        ..Default::default()
    });
    state.save_event_from_draft();
    state.apply_profiles();
    assert_eq!(state.preview_theme().id, new_default);
    assert_eq!(state.settings.service().theme_id, new_default);
}
