use crate::core::layout::ContainerKind;
use crate::core::state::{ChatState, ControlPanelState};
use crate::services::chat::{ChatService, SendMessage};
use eframe::egui::{self, RichText, Vec2};
use egui_phosphor::regular::{CHAT_CIRCLE_TEXT, GLOBE, USERS_THREE};

mod channel_list;
mod channels;
mod composer;
mod dialogs;
mod emoji;
mod messages;
mod notice;
#[cfg(test)]
mod tests;

use dialogs::dialogs;
use emoji::emoji_picker;

pub use channels::render_chat_channels;
pub use messages::render_chat_messages;
pub use notice::render_chat_notice;

#[derive(Default)]
pub(super) struct MessageActions {
    download: Option<String>,
    play: Option<String>,
    action: Option<(u64, String)>,
    seek: Option<f32>,
    react: Option<(u64, String)>,
    open_reactions: Option<(u64, egui::Rect)>,
    delete: Option<u64>,
}
pub(super) struct MessageContext<'a> {
    operator: &'a str,
    tokens: &'a crate::ui::theme::AppTheme,
    profiles: &'a [crate::core::preferences::OperatorProfile],
    audio: &'a AudioStatus,
}

pub(super) fn panel(tokens: &crate::ui::theme::AppTheme) -> egui::Frame {
    egui::Frame::new()
        .fill(tokens.bg_panel)
        .stroke(egui::Stroke::new(1.0, tokens.border_default))
        .corner_radius(tokens.radius_lg)
        .inner_margin(14)
}
#[derive(Default)]
pub(super) struct AudioStatus {
    file: Option<String>,
    playing: bool,
    position: f32,
    duration: f32,
}

pub(super) fn primary_button(
    ui: &mut egui::Ui,
    label: impl Into<egui::WidgetText>,
    tokens: &crate::ui::theme::AppTheme,
) -> egui::Response {
    ui.add(
        egui::Button::new(label)
            .fill(tokens.accent)
            .stroke(egui::Stroke::NONE)
            .corner_radius(8.0)
            .min_size(Vec2::new(0.0, 26.0)),
    )
    .on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub(super) fn soft_button(
    ui: &mut egui::Ui,
    label: impl Into<egui::WidgetText>,
    tokens: &crate::ui::theme::AppTheme,
    primary: bool,
) -> egui::Response {
    ui.add(
        egui::Button::new(label)
            .fill(if primary {
                tokens.accent_soft
            } else {
                tokens.bg_panel
            })
            .stroke(egui::Stroke::new(
                1.0,
                if primary {
                    tokens.accent_soft
                } else {
                    tokens.border_default
                },
            ))
            .corner_radius(9.0)
            .min_size(Vec2::new(0.0, 30.0)),
    )
}

pub(super) fn size_label(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f32 / 1048576.0)
    } else {
        format!("{} KB", bytes.div_ceil(1024))
    }
}
pub(super) fn time_label(seconds: f32) -> String {
    let seconds = seconds.max(0.0) as u64;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

pub(super) fn target(chat: &ChatState) -> SendMessage {
    SendMessage {
        channel_id: if chat.peer.is_none() {
            Some(if chat.channel.is_empty() {
                "general".into()
            } else {
                chat.channel.clone()
            })
        } else {
            None
        },
        recipient_id: chat.peer.clone(),
        ..Default::default()
    }
}

pub(super) fn send(chat: &mut ChatState, service: &ChatService, message: SendMessage) -> bool {
    match service.send(&chat.token, message) {
        Ok(_) => {
            chat.error.clear();
            chat.history_before = None;
            true
        }
        Err(error) => {
            chat.error = error;
            false
        }
    }
}

pub(super) fn upload(
    chat: &mut ChatState,
    service: &ChatService,
    name: &str,
    mime: &str,
    bytes: &[u8],
    mut destination: SendMessage,
    audio: bool,
) {
    match service.upload(&chat.token, name, mime, bytes) {
        Ok(file) => {
            destination.kind = if audio { "audio" } else { "file" }.into();
            destination.file_id = Some(file.id);
            send(chat, service, destination);
        }
        Err(error) => chat.error = error,
    }
}

pub(super) fn open_browser(ui: &egui::Ui, state: &ControlPanelState, service: &ChatService) {
    let host = if state.settings.allow_lan {
        crate::services::web::local_address()
    } else {
        "127.0.0.1".into()
    };
    let secure = state.chat.https_error.is_none();
    let port = if secure {
        crate::services::chat::tls_port(state.settings.server_port)
    } else {
        state.settings.server_port
    };
    if let Ok(token) = service.login_local(&state.settings.active_operator) {
        let url = if secure {
            crate::services::web::https_url(&host, port, "/chat")
        } else if state.settings.allow_lan && !state.settings_ui.discovery_chat_url.is_empty() {
            state.settings_ui.discovery_chat_url.clone()
        } else {
            crate::services::web::http_url(&host, port, "/chat")
        };
        ui.ctx().open_url(egui::OpenUrl::new_tab(format!(
            "{url}#token={token}&profile={}",
            state.settings.active_operator
        )));
    }
}

fn render_chat_top_bar(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    snapshot: &crate::services::chat::Snapshot,
    tokens: &crate::ui::theme::AppTheme,
) -> egui::Response {
    let language = state.settings.language;
    tokens
        .container_frame()
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (badge, _) = ui.allocate_exact_size(Vec2::splat(30.0), egui::Sense::hover());
                ui.painter().rect_filled(badge, 7.0, tokens.accent_soft);
                ui.painter().text(
                    badge.center(),
                    egui::Align2::CENTER_CENTER,
                    CHAT_CIRCLE_TEXT,
                    egui::FontId::proportional(17.0),
                    tokens.accent,
                );
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.label(
                        RichText::new(language.text("nav-chat"))
                            .size(13.0)
                            .strong()
                            .color(tokens.text_primary),
                    );
                    ui.label(
                        RichText::new(
                            language.text("chat-channels-direct-messages-and-files-on-the-local-network"),
                        )
                        .size(10.5)
                        .color(tokens.text_muted),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if soft_button(
                        ui,
                        RichText::new(GLOBE).size(15.0).color(tokens.accent),
                        tokens,
                        true,
                    )
                    .on_hover_text(language.text("chat-network-access"))
                    .clicked()
                    {
                        state.chat.show_access = !state.chat.show_access;
                    }
                    let online_count = snapshot.profiles.iter().filter(|p| p.online).count();
                    ui.label(
                        RichText::new(language.format(
                            "services-online-label",
                            &[("icon", USERS_THREE), ("name", &online_count.to_string())],
                        ))
                        .size(11.5)
                        .color(tokens.status_success_fg),
                    );
                });
            });
        })
        .response
}

pub fn render_chat_page(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    slide_deck: &mut crate::core::slides::SlideDeck,
    is_presentation_open: &mut bool,
    rendered_leaf_rects: &mut Vec<(ContainerKind, egui::Rect)>,
    db: Option<&crate::services::db::AppDb>,
) {
    let tokens = state.theme.clone();
    let language = state.settings.language;
    let Some(service) = state.chat.service.clone() else {
        ui.heading(language.text("nav-chat"));
        ui.label(language.text(&state.chat.error));
        return;
    };
    if state.chat.token.is_empty() {
        ui.heading(language.text("chat-connecting-to-chat"));
        if !state.chat.error.is_empty() {
            ui.label(language.text(&state.chat.error));
        }
        return;
    }
    if state.chat.channel.is_empty() {
        state.chat.channel = "general".into();
    }
    let result = service.snapshot(
        &state.chat.token,
        if state.chat.search_all || state.chat.peer.is_some() {
            None
        } else {
            Some(&state.chat.channel)
        },
        if state.chat.search_all {
            None
        } else {
            state.chat.peer.as_deref()
        },
        &state.chat.search,
        state.chat.history_before,
    );
    let snapshot = match result {
        Ok(snapshot) => snapshot,
        Err(error) => {
            state.chat.error = error;
            ui.label(language.text(&state.chat.error));
            return;
        }
    };
    state.chat.cached_snapshot = Some(snapshot.clone());
    if state.chat.peer.is_none()
        && !snapshot
            .channels
            .iter()
            .any(|channel| channel.id == state.chat.channel)
    {
        state.chat.channel = "general".into();
    }

    // A recording keeps its original recipient even if the selected room changes later.
    if let Some(recording) = state.chat.recorder.as_ref().and_then(|r| r.take()) {
        state.chat.recorder = None;
        state.chat.recording_started = None;
        if !state.chat.recording_cancelled {
            match recording {
                Ok(bytes) => {
                    let destination = state
                        .chat
                        .recording_target
                        .take()
                        .unwrap_or_else(|| target(&state.chat));
                    upload(
                        &mut state.chat,
                        &service,
                        "gravacao.wav",
                        "audio/wav",
                        &bytes,
                        destination,
                        true,
                    );
                }
                Err(error) => state.chat.error = error,
            }
        }
        state.chat.recording_cancelled = false;
    }

    let avail_rect = ui.available_rect_before_wrap().shrink(6.0);
    let top_x = avail_rect.min.x + 2.0;
    let top_w = (avail_rect.width() - 4.0).max(1.0);
    let top_y = avail_rect.min.y + 2.0;

    let mut top_ui = ui.new_child(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
        egui::pos2(top_x, top_y),
        egui::vec2(top_w, 200.0),
    )));
    let top_bar_resp = render_chat_top_bar(&mut top_ui, state, &snapshot, &tokens);

    let mut remaining_top = top_bar_resp.rect.max.y + 4.0;
    if !state.chat.error.is_empty() {
        let err_slot_rect = egui::Rect::from_min_size(
            egui::pos2(avail_rect.min.x, remaining_top),
            egui::vec2(avail_rect.width(), 26.0),
        );
        let err_card_rect = err_slot_rect.shrink(2.0);
        let mut err_ui = ui.new_child(egui::UiBuilder::new().max_rect(err_card_rect));
        err_ui.horizontal_wrapped(|ui| {
            ui.colored_label(tokens.status_warning_fg, &state.chat.error);
            if ui.small_button(state.settings.language.text("common-close")).clicked() {
                state.chat.error.clear();
            }
        });
        remaining_top = err_slot_rect.max.y + 4.0;
    }

    let chat_rect =
        egui::Rect::from_min_max(egui::pos2(avail_rect.min.x, remaining_top), avail_rect.max);
    let mut root = state.chat_layout.root.clone();
    crate::ui::render_node(
        ui,
        &mut root,
        chat_rect,
        "chat_root",
        state,
        slide_deck,
        is_presentation_open,
        rendered_leaf_rects,
        db,
    );
    state.chat_layout.root = root;

    dialogs(ui, state, &service);
    emoji_picker(ui.ctx(), state, &service);
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_millis(500));
    if state
        .chat
        .audio_player
        .as_ref()
        .is_some_and(|p| p.is_playing())
    {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(33));
    }
}
