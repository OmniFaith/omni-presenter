use super::*;
use crate::ui::components::drag_drop::{self, DragPayload};

pub(super) fn handle_design_click(
    response: &egui::Response,
    ui: &egui::Ui,
    state: &mut ControlPanelState,
    scene: &DesignDocument,
    visible_ids: &[String],
    actions: &mut DesignSelectionActions,
) {
    let (shift, ctrl) = ui.input(|input| {
        (
            input.modifiers.shift,
            input.modifiers.command || input.modifiers.ctrl,
        )
    });
    if response.double_clicked() {
        actions.project = Some(scene.id.clone());
    } else if response.clicked() {
        crate::core::interaction::select(
            &mut state.selected_design_ids,
            &mut state.design_selection_anchor,
            scene.id.clone(),
            visible_ids,
            shift,
            ctrl,
        );
        if !shift && !ctrl {
            actions.preview = Some(scene.id.clone());
        }
    }
    if response.secondary_clicked() && !state.selected_design_ids.contains(&scene.id) {
        crate::core::interaction::select(
            &mut state.selected_design_ids,
            &mut state.design_selection_anchor,
            scene.id.clone(),
            visible_ids,
            false,
            false,
        );
    }
}

pub(super) fn source(
    ui: &egui::Ui,
    response: &egui::Response,
    state: &ControlPanelState,
    deck: &SlideDeck,
    scene: &DesignDocument,
) {
    drag_drop::source(ui, response, || {
        let mut payload = DragPayload::for_design(scene);
        if state.selected_design_ids.contains(&scene.id) {
            payload.items = deck
                .designs
                .iter()
                .filter(|scene| state.selected_design_ids.contains(&scene.id))
                .flat_map(|scene| DragPayload::for_design(scene).items)
                .collect();
            payload.count = payload.items.len();
        }
        payload
    });
}
