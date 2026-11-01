use super::DesignEditor;
use crate::{
    core::design::DesignLayer,
    ui::{AppTheme, components::*},
};
use eframe::egui;
use egui_phosphor::regular::*;

#[derive(Clone, Copy)]
pub(super) enum LayerAction {
    Edit,
    Visibility,
    Lock,
    Forward,
    Backward,
    Duplicate,
    Delete,
    Group,
    Ungroup,
}

pub(super) fn layer_menu(
    ui: &mut egui::Ui,
    layer: &DesignLayer,
    index: usize,
    count: usize,
    selected_count: usize,
    theme: &AppTheme,
) -> Option<LayerAction> {
    context_menu_header(ui);
    let mut action = None;
    for (value, glyph, label, enabled) in [
        (LayerAction::Edit, PENCIL_SIMPLE, crate::ui::i18n::text(ui, "design-edit-layer"), true),
        (
            LayerAction::Visibility,
            if layer.visible { EYE_SLASH } else { EYE },
            if layer.visible {
                crate::ui::i18n::text(ui, "design-hide-layer")
            } else {
                crate::ui::i18n::text(ui, "design-show-layer")
            },
            true,
        ),
        (
            LayerAction::Lock,
            if layer.locked {
                LOCK_SIMPLE_OPEN
            } else {
                LOCK_SIMPLE
            },
            if layer.locked {
                crate::ui::i18n::text(ui, "design-unlock-layer")
            } else {
                crate::ui::i18n::text(ui, "design-lock-layer")
            },
            true,
        ),
    ] {
        if menu_item(ui, glyph, label, enabled, theme) {
            action = Some(value);
        }
    }
    context_menu_separator(ui);
    for (value, glyph, label, enabled) in [
        (
            LayerAction::Forward,
            ARROW_UP,
            crate::ui::i18n::text(ui, "design-bring-forward"),
            !layer.locked && index + 1 < count,
        ),
        (
            LayerAction::Backward,
            ARROW_DOWN,
            crate::ui::i18n::text(ui, "design-send-backward"),
            !layer.locked && index > 0,
        ),
        (LayerAction::Duplicate, COPY, crate::ui::i18n::text(ui, "design-duplicate-layer"), true),
    ] {
        if menu_item(ui, glyph, label, enabled, theme) {
            action = Some(value);
        }
    }
    if selected_count > 1 || layer.group_id.is_some() {
        context_menu_separator(ui);
        if selected_count > 1 && menu_item(ui, FOLDER_PLUS, crate::ui::i18n::text(ui, "design-group"), true, theme) {
            action = Some(LayerAction::Group);
        }
        if layer.group_id.is_some() && menu_item(ui, FOLDER_MINUS, crate::ui::i18n::text(ui, "design-ungroup"), true, theme) {
            action = Some(LayerAction::Ungroup);
        }
    }
    context_menu_separator(ui);
    let delete_label = crate::ui::i18n::text(ui, "design-delete-layer");
    if !layer.locked {
        if context_menu_danger_item(ui, TRASH, delete_label, theme).clicked() {
            action = Some(LayerAction::Delete);
            ui.close();
        }
    } else {
        context_menu_disabled_item(ui, TRASH, delete_label, theme);
    }
    action
}

fn menu_item(ui: &mut egui::Ui, glyph: &str, label: &str, enabled: bool, theme: &AppTheme) -> bool {
    if !enabled {
        context_menu_disabled_item(ui, glyph, label, theme);
        return false;
    }
    let clicked = context_menu_item(ui, glyph, label, theme).clicked();
    if clicked {
        ui.close();
    }
    clicked
}

pub(super) fn apply_layer(editor: &mut DesignEditor, index: usize, action: LayerAction) {
    let Some(layer) = editor
        .scene
        .as_ref()
        .and_then(|scene| scene.layers.get(index))
    else {
        return;
    };
    if layer.locked
        && matches!(
            action,
            LayerAction::Forward | LayerAction::Backward | LayerAction::Delete
        )
    {
        return;
    }
    if matches!(action, LayerAction::Group) {
        editor.group_selected();
        return;
    }
    if matches!(action, LayerAction::Ungroup) {
        editor.ungroup_selected();
        return;
    }
    editor.selected = Some(index);
    editor.inspector_tab = 1;
    if matches!(action, LayerAction::Edit) {
        return;
    }
    if matches!(action, LayerAction::Forward | LayerAction::Backward) {
        editor.reorder(matches!(action, LayerAction::Forward));
        return;
    }
    let before = editor.scene.clone().unwrap();
    match action {
        LayerAction::Visibility => {
            editor.scene.as_mut().unwrap().layers[index].visible = !before.layers[index].visible
        }
        LayerAction::Lock => {
            editor.scene.as_mut().unwrap().layers[index].locked = !before.layers[index].locked
        }
        LayerAction::Duplicate => editor.duplicate_layer(),
        LayerAction::Delete => editor.delete_layer(),
        _ => {}
    }
    editor.changed(before, true);
}

pub(super) fn new_design_menu(ui: &mut egui::Ui, theme: &AppTheme) -> Option<usize> {
    context_menu_header(ui);
    let mut kind = None;
    let create = context_menu_item(ui, PLUS, "design-new-blank-design", theme);
    if create.clicked() {
        kind = Some(0);
        ui.close();
    }
    context_menu_submenu(ui, STACK, "design-create-from-template", theme, |ui| {
        for (value, glyph, label) in [
            (1, SPARKLE, "design-welcome-label"),
            (2, TIMER, "design-countdown"),
            (3, QUESTION, "design-questions-and-answers"),
        ] {
            if context_menu_item(ui, glyph, label, theme).clicked() {
                kind = Some(value);
                ui.close();
            }
        }
        context_menu_separator(ui);
        for (value, glyph, label) in [
            (4, CLOCK, "design-clock-overlay-label"),
            (5, RECTANGLE, "design-lower-third-overlay"),
        ] {
            if context_menu_item(ui, glyph, label, theme).clicked() {
                kind = Some(value);
                ui.close();
            }
        }
    });
    kind
}
