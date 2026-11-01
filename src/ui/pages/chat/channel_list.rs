use crate::{
    core::state::ControlPanelState,
    services::chat::{Channel, Snapshot},
    ui::{
        AppTheme,
        components::{
            context_menu_danger_item, context_menu_header, context_menu_item,
            context_menu_separator,
            drag_drop::{self, DragPayload, DragSource},
            interaction,
        },
    },
};
use eframe::egui;
use egui_phosphor::regular::{CHAT_CIRCLE_TEXT, CHECKS, PENCIL_SIMPLE, PLUS, TRASH};

pub(super) fn selection_bar(ui: &mut egui::Ui, state: &mut ControlPanelState, theme: &AppTheme) {
    let count = state.chat.channel_selection.selected.len();
    if count < 2 {
        return;
    }
    ui.horizontal(|ui| {
        if interaction::selection_summary(ui, count, theme).clicked() {
            state.chat.channel_selection.clear();
        }
        if theme
            .icon_button(
                ui,
                CHECKS,
                state.settings.language.text("chat-read"),
                false,
                egui::Vec2::splat(24.0),
            )
            .clicked()
        {
            apply(
                state,
                Action::Read(
                    state
                        .chat
                        .channel_selection
                        .selected
                        .iter()
                        .cloned()
                        .collect(),
                ),
            );
        }
        if theme
            .icon_button(
                ui,
                TRASH,
                state.settings.language.text("chat-delete"),
                false,
                egui::Vec2::splat(24.0),
            )
            .clicked()
        {
            apply(
                state,
                Action::Delete(
                    state
                        .chat
                        .channel_selection
                        .selected
                        .iter()
                        .filter(|id| id.as_str() != "general")
                        .cloned()
                        .collect(),
                ),
            );
        }
    });
}

enum Action {
    Read(Vec<String>),
    Delete(Vec<String>),
    Move(Vec<String>, String, bool),
}

pub(super) fn render(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    snapshot: &Snapshot,
    theme: &AppTheme,
) {
    let language = state.settings.language;
    let order: Vec<_> = snapshot
        .channels
        .iter()
        .map(|channel| channel.id.clone())
        .collect();
    state.chat.channel_selection.retain(&order);
    let mut action = None;
    if !ui.ctx().egui_wants_keyboard_input()
        && !egui::DragAndDrop::has_any_payload(ui.ctx())
        && ui.ctx().memory(|memory| memory.top_modal_layer().is_none())
    {
        if ui.rect_contains_pointer(ui.max_rect())
            && ui.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::A))
        {
            state.chat.channel_selection.selected = order.iter().cloned().collect();
            state.chat.channel_selection.anchor = order.first().cloned();
        }
        if !state.chat.channel_selection.selected.is_empty() {
            if ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
                state.chat.channel_selection.clear();
            }
            if ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Delete)) {
                action = Some(Action::Delete(
                    state
                        .chat
                        .channel_selection
                        .selected
                        .iter()
                        .filter(|id| id.as_str() != "general")
                        .cloned()
                        .collect(),
                ));
            }
        }
    }
    for channel in &snapshot.channels {
        let selected = state.chat.channel_selection.selected.contains(&channel.id);
        let response = super::channels::nav_button(
            ui,
            &format!("#  {}", channel.display_name(language)),
            selected || (state.chat.peer.is_none() && state.chat.channel == channel.id),
            None,
            None,
            snapshot
                .notifications
                .channels
                .get(&channel.id)
                .copied()
                .unwrap_or(0),
            theme,
        );
        if response.clicked() {
            let (shift, additive) = ui.input(|input| {
                (
                    input.modifiers.shift,
                    input.modifiers.ctrl || input.modifiers.command,
                )
            });
            state
                .chat
                .channel_selection
                .click(channel.id.clone(), &order, shift, additive);
            if !shift && !additive {
                open(state, &channel.id);
            }
        }
        if response.secondary_clicked() && !selected {
            state
                .chat
                .channel_selection
                .click(channel.id.clone(), &order, false, false);
        }
        if response.drag_started() && !state.chat.channel_selection.selected.contains(&channel.id) {
            state
                .chat
                .channel_selection
                .click(channel.id.clone(), &order, false, false);
        }
        let ids = state.chat.channel_selection.drag_ids(&channel.id, &order);
        drag_drop::source(ui, &response, || {
            let mut payload = DragPayload::new(
                DragSource::Channels(ids.clone()),
                channel.display_name(language),
                &channel.description,
                CHAT_CIRCLE_TEXT,
            );
            payload.count = ids.len();
            payload
        });
        if let Some((payload, after)) = drag_drop::reorder_target(
            ui,
            response.rect,
            theme,
            false,
            |source| matches!(source, DragSource::Channels(ids) if !ids.contains(&channel.id)),
        )
            && let DragSource::Channels(ids) = &payload.source {
                action = Some(Action::Move(ids.clone(), channel.id.clone(), after));
            }
        response
            .on_hover_text(&channel.description)
            .context_menu(|ui| {
                context_menu_header(ui);
                if context_menu_item(ui, CHAT_CIRCLE_TEXT, language.text("chat-open"), theme)
                    .clicked()
                {
                    open(state, &channel.id);
                    ui.close();
                }
                if channel.id != "general"
                    && ids.len() == 1
                    && context_menu_item(ui, PENCIL_SIMPLE, language.text("chat-edit"), theme)
                        .clicked()
                {
                    edit(state, channel);
                    ui.close();
                }
                if context_menu_item(ui, CHECKS, language.text("chat-read"), theme).clicked() {
                    action = Some(Action::Read(ids.clone()));
                    ui.close();
                }
                if context_menu_item(ui, PLUS, language.text("chat-create"), theme).clicked() {
                    create(state);
                    ui.close();
                }
                context_menu_separator(ui);
                if context_menu_item(ui, CHECKS, language.text("chat-select-all"), theme).clicked()
                {
                    state.chat.channel_selection.selected = order.iter().cloned().collect();
                    ui.close();
                }
                let removable: Vec<_> = ids
                    .iter()
                    .filter(|id| id.as_str() != "general")
                    .cloned()
                    .collect();
                if !removable.is_empty()
                    && context_menu_danger_item(ui, TRASH, language.text("chat-delete"), theme)
                        .clicked()
                {
                    action = Some(Action::Delete(removable));
                    ui.close();
                }
            });
    }
    if let Some(action) = action {
        apply(state, action);
    }
}

pub(super) fn create(state: &mut ControlPanelState) {
    state.chat.editing_channel = None;
    state.chat.new_channel.clear();
    state.chat.new_channel_description.clear();
    state.chat.new_channel_tool = "general".into();
    state.chat.error.clear();
    state.chat.show_channels = true;
}

fn edit(state: &mut ControlPanelState, channel: &Channel) {
    state.chat.editing_channel = Some(channel.id.clone());
    state.chat.new_channel = channel.name.clone();
    state.chat.new_channel_description = channel.description.clone();
    state.chat.new_channel_tool = channel.tool.clone();
    state.chat.error.clear();
    state.chat.show_channels = true;
}

fn open(state: &mut ControlPanelState, id: &str) {
    state.chat.channel = id.into();
    state.chat.peer = None;
    state.chat.history_before = None;
    state.chat.search_all = false;
}

fn apply(state: &mut ControlPanelState, action: Action) {
    let Some(service) = state.chat.service.clone() else {
        return;
    };
    let result = match action {
        Action::Read(ids) => service.mark_channels_read(&state.chat.token, &ids),
        Action::Delete(ids) => service.delete_channels(&state.chat.token, &ids).map(|()| {
            if ids.contains(&state.chat.channel) {
                state.chat.channel = "general".into();
                state.chat.peer = None;
            }
            state.chat.channel_selection.clear();
        }),
        Action::Move(ids, target, after) => {
            service.move_channels(&state.chat.token, &ids, &target, after)
        }
    };
    match result {
        Ok(()) => {
            state.chat.error.clear();
            state.chat.last_sync = None;
        }
        Err(error) => state.chat.error = error,
    }
}
