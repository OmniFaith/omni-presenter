use crate::core::layout::ContainerKind;
use crate::core::state::ControlPanelState;
use crate::services::chat::{Message, SharedContent};
use crate::ui::components::render_header_bar;
use eframe::egui::{self, RichText, Vec2};
use egui_phosphor::regular::{
    BOOK_OPEN, DOWNLOAD_SIMPLE, FILE, IMAGE, LINK_SIMPLE, MAGNIFYING_GLASS, MUSIC_NOTES, PAUSE,
    PLAY, PLUS, PUSH_PIN, SMILEY, TRASH, VIDEO,
};

use super::channels::avatar;
use super::composer::composer;
use super::{AudioStatus, MessageActions, MessageContext, size_label, soft_button, time_label};

pub fn render_chat_messages(ui: &mut egui::Ui, state: &mut ControlPanelState) {
    let tokens = state.theme.clone();
    let language = state.settings.language;
    let Some(service) = state.chat.service.clone() else {
        return;
    };
    let Some(snapshot) = state.chat.cached_snapshot.clone() else {
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new(language.text("chat-loading-messages")).color(tokens.text_muted),
            );
        });
        return;
    };

    let title = if let Some(peer) = &state.chat.peer {
        snapshot
            .profiles
            .iter()
            .find(|p| &p.id == peer)
            .map(|p| format!("@ {}", p.name))
    } else {
        snapshot
            .channels
            .iter()
            .find(|c| c.id == state.chat.channel)
            .map(|c| format!("# {}", c.display_name(state.settings.language)))
    }
    .unwrap_or_else(|| "chat-messages".into());

    render_header_bar(ui, ContainerKind::ChatMessages, &title, &tokens);
    ui.add_space(2.0);

    ui.horizontal(|ui| {
        let width = (ui.available_width() * 0.35).clamp(100.0, 300.0);
        ui.label(
            RichText::new(language.text(if state.chat.peer.is_some() {
                "chat-direct-conversation"
            } else {
                "chat-messages-and-shared-content"
            }))
            .size(10.5)
            .color(tokens.text_muted),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let search = tokens.icon_button(
                ui,
                MAGNIFYING_GLASS,
                if state.chat.search_open {
                    language.text("chat-close-search")
                } else {
                    language.text("chat-search-messages")
                },
                state.chat.search_open,
                Vec2::splat(26.0),
            );
            if search.clicked() {
                state.chat.search_open = !state.chat.search_open;
                state.chat.search_focus = state.chat.search_open;
                if !state.chat.search_open {
                    state.chat.search.clear();
                    state.chat.search_all = false;
                    state.chat.history_before = None;
                }
            }
            if state.chat.search_open {
                if ui
                    .checkbox(&mut state.chat.search_all, language.text("chat-all"))
                    .on_hover_text(language.text("chat-search-all-your-conversations"))
                    .changed()
                {
                    state.chat.history_before = None;
                }
                let field = ui.add(
                    egui::TextEdit::singleline(&mut state.chat.search)
                        .hint_text(format!(
                            "{MAGNIFYING_GLASS}  {}",
                            language.text("chat-search")
                        ))
                        .desired_width(width),
                );
                if state.chat.search_focus {
                    field.request_focus();
                    state.chat.search_focus = false;
                }
                if field.changed() {
                    state.chat.history_before = None;
                }
            }
        });
    });
    ui.add_space(4.0);
    ui.separator();

    let compose_h = if state.chat.recorder.is_some() {
        112.0
    } else {
        78.0
    };
    let messages_h = (ui.available_height() - compose_h).max(60.0);
    let mut actions = MessageActions::default();
    let audio = state
        .chat
        .audio_player
        .as_ref()
        .map(|player| AudioStatus {
            file: state.chat.audio_file_id.clone(),
            playing: player.is_playing(),
            position: player.position_secs(),
            duration: player
                .current_track()
                .map(|t| t.duration_secs)
                .unwrap_or(0.0),
        })
        .unwrap_or_default();
    let scroll = egui::ScrollArea::vertical()
        .id_salt((
            "chat_messages",
            &state.chat.channel,
            &state.chat.peer,
            &state.chat.search,
            state.chat.history_before,
        ))
        .max_height(messages_h)
        .min_scrolled_height(messages_h)
        .auto_shrink([false, false])
        .stick_to_bottom(state.chat.history_before.is_none())
        .show(ui, |ui| {
            if snapshot.messages.len() == 100
                && ui.button(language.text("chat-older-messages")).clicked()
            {
                state.chat.history_before = snapshot.messages.first().map(|m| m.id);
            }
            if state.chat.history_before.is_some()
                && ui
                    .button(language.text("chat-back-to-recent-messages"))
                    .clicked()
            {
                state.chat.history_before = None;
            }
            if snapshot.messages.is_empty() {
                ui.add_space(25.0);
                ui.colored_label(
                    tokens.text_muted,
                    language.text(if state.chat.search.is_empty() {
                        "chat-this-is-the-beginning-of-the-conversation"
                    } else {
                        "chat-no-messages-found"
                    }),
                );
            }
            let transcript = ui.available_width().min(940.0);
            let padding = ((ui.available_width() - transcript) * 0.5).max(0.0);
            ui.horizontal_top(|ui| {
                if padding > 0.0 {
                    ui.add_space(padding);
                }
                ui.vertical(|ui| {
                    ui.set_width(transcript);
                    for message in &snapshot.messages {
                        let context = MessageContext {
                            operator: &state.settings.active_operator,
                            tokens: &tokens,
                            profiles: &state.settings.operators,
                            audio: &audio,
                        };
                        message_bubble(ui, message, &context, &mut actions);
                        ui.add_space(14.0);
                    }
                });
            });
        });
    let at_bottom =
        scroll.content_size.y - scroll.state.clamped_offset().y - scroll.inner_rect.height() < 30.0;
    if at_bottom
        && ui.ctx().input(|i| i.focused)
        && state.chat.search.is_empty()
        && !state.chat.search_all
        && state.chat.history_before.is_none()
    {
        let unread = if let Some(peer) = &state.chat.peer {
            snapshot.notifications.private.get(peer)
        } else {
            snapshot.notifications.channels.get(&state.chat.channel)
        }
        .copied()
        .unwrap_or(0);
        let peer = state.chat.peer.as_deref();
        if unread > 0
            && let Ok(notifications) = service.mark_read(
                &state.chat.token,
                if peer.is_none() {
                    Some(&state.chat.channel)
                } else {
                    None
                },
                peer,
                snapshot.messages.last().map(|message| message.id),
            )
        {
            state.chat.notifications = notifications;
        }
    }
    if let Some((id, emoji)) = actions.react {
        let _ = service.react(&state.chat.token, id, &emoji);
    }
    if let Some((id, rect)) = actions.open_reactions {
        state.chat.reacting_to = Some(id);
        state.chat.emoji_anchor = Some(rect);
        state.chat.emoji_opened_frame = ui.ctx().cumulative_frame_nr();
    }
    if let Some(id) = actions.delete {
        let _ = service.delete_message(&state.chat.token, id);
    }
    if let Some((id, action)) = actions.action {
        let _ = service.request_action(&state.chat.token, id, &action);
    }
    if let Some(seek) = actions.seek
        && let Some(player) = state.chat.audio_player.as_mut() {
            player.seek(seek);
        }
    if let Some(id) = actions.play {
        if state.chat.audio_file_id.as_deref() == Some(&id)
            && let Some(player) = state.chat.audio_player.as_mut()
        {
            if player.is_playing() {
                player.pause();
            } else {
                player.resume();
            }
        } else {
            match service.download(&state.chat.token, &id) {
                Ok((name, _content_type, bytes)) => {
                    let ext = name.rsplit('.').next().unwrap_or("wav");
                    let path = std::env::temp_dir().join(format!("omni-presenter-chat-{id}.{ext}"));
                    let result = std::fs::write(&path, bytes)
                        .map_err(|e| e.to_string())
                        .and_then(|_| {
                            state
                                .chat
                                .audio_player
                                .get_or_insert_with(crate::services::audio::AudioPlayer::new)
                                .play_file(
                                    name,
                                    crate::services::media::audio_path(&path.to_string_lossy())?,
                                )
                        });
                    if let Err(error) = result {
                        state.chat.error = format!(
                            "{error}. {}",
                            language.text("chat-you-can-also-listen-in-the-browser-chat")
                        );
                    } else {
                        state.chat.audio_file_id = Some(id.clone());
                    }
                }
                Err(error) => state.chat.error = error,
            }
        }
    }
    ui.add_space(5.0);
    let compose_width = ui.available_width().min(940.0);
    let padding = ((ui.available_width() - compose_width) * 0.5).max(0.0);
    ui.horizontal_top(|ui| {
        if padding > 0.0 {
            ui.add_space(padding);
        }
        ui.vertical(|ui| {
            ui.set_width(compose_width);
            composer(ui, state, &service);
        });
    });
}

fn message_bubble(
    ui: &mut egui::Ui,
    message: &Message,
    context: &MessageContext<'_>,
    actions: &mut MessageActions,
) {
    let operator = context.operator;
    let language = crate::ui::i18n::language(ui.ctx());
    let tokens = context.tokens;
    let audio = context.audio;
    let photo = context
        .profiles
        .iter()
        .find(|p| p.id == message.author_id)
        .and_then(|p| p.avatar.as_deref());
    let own = message.author_id == operator;
    let available = ui.available_width();
    let max_width = (available - 58.0)
        .max(80.0)
        .min(if message.shared.is_some() {
            420.0
        } else {
            520.0
        });
    let width = if message.shared.is_some() {
        max_width
    } else {
        let text = ui.painter().layout(
            message.text.clone(),
            egui::FontId::proportional(13.5),
            tokens.text_primary,
            (max_width - 28.0).max(40.0),
        );
        (text.size().x + 28.0).max(170.0).min(max_width)
    };
    ui.push_id(("message", message.id), |ui| {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            if own {
                ui.add_space((available - width - 58.0).max(0.0));
            }
            if !own {
                avatar(ui, &message.author, tokens, false, photo);
            }
            ui.vertical(|ui| {
                ui.set_width(width);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(if own {
                            language.text("chat-you")
                        } else {
                            &message.author
                        })
                        .size(11.5)
                        .strong()
                        .color(tokens.text_secondary),
                    );
                    let seconds = message.timestamp / 1000;
                    ui.label(
                        RichText::new(format!(
                            "{:02}:{:02}",
                            (seconds / 3600) % 24,
                            (seconds / 60) % 60
                        ))
                        .size(10.0)
                        .color(tokens.text_muted),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if !message.deleted {
                            if own
                                && tokens
                                    .icon_button(
                                        ui,
                                        TRASH,
                                        language.text("chat-delete-message"),
                                        false,
                                        Vec2::splat(20.0),
                                    )
                                    .clicked()
                            {
                                actions.delete = Some(message.id);
                            }
                            let reaction = tokens.icon_button(
                                ui,
                                SMILEY,
                                language.text("chat-react-to-message"),
                                false,
                                Vec2::splat(20.0),
                            );
                            if reaction.clicked() {
                                actions.open_reactions = Some((message.id, reaction.rect));
                            }
                        }
                    });
                });
                ui.add_space(2.0);
                egui::Frame::new()
                    .fill(if own {
                        tokens.accent_soft
                    } else {
                        tokens.bg_panel
                    })
                    .stroke(egui::Stroke::new(
                        1.0,
                        if own {
                            tokens.accent_soft
                        } else {
                            tokens.border_default
                        },
                    ))
                    .corner_radius(14.0)
                    .inner_margin(14)
                    .show(ui, |ui| {
                        ui.set_width((width - 30.0).max(40.0));
                        if let Some(shared) = &message.shared {
                            match shared {
                                SharedContent::Bible {
                                    reference, version, ..
                                } => {
                                    shared_row(ui, message, tokens, actions, |ui| {
                                        resource_header(
                                            ui,
                                            BOOK_OPEN,
                                            "chat-bible-passage",
                                            reference,
                                            &format!("{} · {version}", language.text("nav-bible")),
                                            tokens,
                                        );
                                    });
                                    content_details(ui, message, "chat-view-passage", tokens);
                                }
                                SharedContent::Song { title, artist, .. } => {
                                    shared_row(ui, message, tokens, actions, |ui| {
                                        resource_header(
                                            ui,
                                            MUSIC_NOTES,
                                            "chat-song",
                                            title,
                                            artist,
                                            tokens,
                                        );
                                    });
                                    content_details(ui, message, "chat-view-lyrics", tokens);
                                }
                                SharedContent::Media { file } => {
                                    if file.mime.starts_with("audio/") {
                                        let active = audio.file.as_deref() == Some(&file.id);
                                        let mut play = None;
                                        let mut seek = None;
                                        ui.horizontal_top(|ui| {
                                            let (rect, response) = ui.allocate_exact_size(
                                                Vec2::splat(42.0),
                                                egui::Sense::click(),
                                            );
                                            ui.painter().circle_filled(
                                                rect.center(),
                                                21.0,
                                                tokens.accent,
                                            );
                                            ui.painter().text(
                                                rect.center(),
                                                egui::Align2::CENTER_CENTER,
                                                if active && audio.playing { PAUSE } else { PLAY },
                                                egui::FontId::proportional(20.0),
                                                tokens.text_on_accent,
                                            );
                                            if response
                                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                                .on_hover_text(language.text(
                                                    if active && audio.playing {
                                                        "chat-pause-voice-message"
                                                    } else {
                                                        "chat-listen-to-voice-message"
                                                    },
                                                ))
                                                .clicked()
                                            {
                                                play = Some(file.id.clone());
                                            }
                                            ui.vertical(|ui| {
                                                ui.horizontal_top(|ui| {
                                                    ui.label(
                                                        RichText::new(
                                                            language.text("chat-voice-message"),
                                                        )
                                                        .size(13.0)
                                                        .strong()
                                                        .color(tokens.text_primary),
                                                    );
                                                    let width = ui.available_width().max(0.0);
                                                    ui.allocate_ui_with_layout(
                                                        Vec2::new(width, 28.0),
                                                        egui::Layout::right_to_left(
                                                            egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            shared_actions(
                                                                ui, message, tokens, actions,
                                                            );
                                                        },
                                                    );
                                                });
                                                ui.add_space(2.0);
                                                let (track, response) = ui.allocate_exact_size(
                                                    Vec2::new(
                                                        (ui.available_width() - 2.0).max(40.0),
                                                        12.0,
                                                    ),
                                                    egui::Sense::click(),
                                                );
                                                let progress = if active && audio.duration > 0.0 {
                                                    (audio.position / audio.duration)
                                                        .clamp(0.0, 1.0)
                                                } else {
                                                    0.0
                                                };
                                                let line = egui::Rect::from_min_size(
                                                    egui::pos2(
                                                        track.left(),
                                                        track.center().y - 2.0,
                                                    ),
                                                    Vec2::new(track.width(), 4.0),
                                                );
                                                ui.painter().rect_filled(
                                                    line,
                                                    2.0,
                                                    tokens.border_active,
                                                );
                                                if progress > 0.0 {
                                                    ui.painter().rect_filled(
                                                        egui::Rect::from_min_size(
                                                            line.min,
                                                            Vec2::new(line.width() * progress, 4.0),
                                                        ),
                                                        2.0,
                                                        tokens.accent,
                                                    );
                                                }
                                                if active
                                                    && audio.duration > 0.0
                                                    && response.clicked()
                                                    && let Some(pos) =
                                                        response.interact_pointer_pos()
                                                {
                                                    seek = Some(
                                                        ((pos.x - track.left()) / track.width())
                                                            .clamp(0.0, 1.0)
                                                            * audio.duration,
                                                    );
                                                }
                                                let caption = if active && audio.duration > 0.0 {
                                                    format!(
                                                        "{} / {}",
                                                        time_label(audio.position),
                                                        time_label(audio.duration)
                                                    )
                                                } else {
                                                    format!(
                                                        "{} · {}",
                                                        language.text("chat-audio"),
                                                        size_label(file.size)
                                                    )
                                                };
                                                ui.label(
                                                    RichText::new(caption)
                                                        .size(10.5)
                                                        .color(tokens.text_muted),
                                                )
                                                .on_hover_text(&file.name);
                                            });
                                        });
                                        if let Some(id) = play {
                                            actions.play = Some(id);
                                        }
                                        if let Some(seconds) = seek {
                                            actions.seek = Some(seconds);
                                        }
                                    } else {
                                        let kind =
                                            crate::core::models::MediaItem::detect_type(&file.name);
                                        let (icon, label) = if file.mime.starts_with("video/")
                                            || kind == "video"
                                        {
                                            (VIDEO, "chat-video")
                                        } else if file.mime.starts_with("image/") || kind == "image"
                                        {
                                            (IMAGE, "chat-image")
                                        } else {
                                            (FILE, "chat-file")
                                        };
                                        shared_row(ui, message, tokens, actions, |ui| {
                                            resource_header(
                                                ui,
                                                icon,
                                                label,
                                                &file.name,
                                                &format!(
                                                    "{} · {}",
                                                    size_label(file.size),
                                                    file.name
                                                        .rsplit('.')
                                                        .next()
                                                        .unwrap_or("arquivo")
                                                        .to_uppercase()
                                                ),
                                                tokens,
                                            );
                                        });
                                    }
                                    if !message.text.is_empty() {
                                        ui.add_space(6.0);
                                        ui.add(
                                            egui::Label::new(
                                                RichText::new(&message.text)
                                                    .size(13.0)
                                                    .color(tokens.text_primary),
                                            )
                                            .wrap(),
                                        );
                                    }
                                }
                            }
                        } else {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(if message.deleted {
                                        language.text("chat-message-deleted")
                                    } else {
                                        &message.text
                                    })
                                    .size(13.5)
                                    .color(
                                        if message.deleted {
                                            tokens.text_muted
                                        } else {
                                            tokens.text_primary
                                        },
                                    ),
                                )
                                .wrap(),
                            );
                        }
                    });
                if !message.reactions.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        for reaction in &message.reactions {
                            let selected = reaction.users.iter().any(|p| p == operator);
                            let people = reaction
                                .users
                                .iter()
                                .map(|id| {
                                    context
                                        .profiles
                                        .iter()
                                        .find(|p| &p.id == id)
                                        .map(|p| p.name.as_str())
                                        .unwrap_or(id)
                                })
                                .collect::<Vec<_>>()
                                .join(", ");
                            let button = egui::Button::new(
                                RichText::new(format!(
                                    "{} {}",
                                    reaction.emoji,
                                    reaction.users.len()
                                ))
                                .size(13.0)
                                .color(if selected {
                                    tokens.accent
                                } else {
                                    tokens.text_secondary
                                }),
                            )
                            .fill(if selected {
                                tokens.accent_soft
                            } else {
                                tokens.bg_pill
                            })
                            .stroke(egui::Stroke::new(
                                1.0,
                                if selected {
                                    tokens.accent
                                } else {
                                    tokens.border_default
                                },
                            ))
                            .corner_radius(12.0);
                            if ui.add(button).on_hover_text(people).clicked() {
                                actions.react = Some((message.id, reaction.emoji.clone()));
                            }
                        }
                    });
                }
            });
            if own {
                avatar(ui, &message.author, tokens, true, photo);
            }
        })
    });
}

fn shared_row(
    ui: &mut egui::Ui,
    message: &Message,
    tokens: &crate::ui::theme::AppTheme,
    actions: &mut MessageActions,
    add_content: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal_top(|ui| {
        add_content(ui);
        let width = ui.available_width().max(0.0);
        ui.allocate_ui_with_layout(
            Vec2::new(width, 28.0),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                shared_actions(ui, message, tokens, actions);
            },
        );
    });
}

fn shared_actions(
    ui: &mut egui::Ui,
    message: &Message,
    tokens: &crate::ui::theme::AppTheme,
    actions: &mut MessageActions,
) {
    let language = crate::ui::i18n::language(ui.ctx());
    if !message.deleted {
        let response = ui
            .add(
                egui::Button::new(
                    RichText::new(egui_phosphor::regular::DOTS_SIX_VERTICAL)
                        .size(15.0)
                        .color(tokens.text_muted),
                )
                .frame(false)
                .sense(egui::Sense::click_and_drag()),
            )
            .on_hover_text(crate::ui::i18n::language(ui.ctx()).text("drag-pin"));
        crate::ui::components::drag_drop::source(ui, &response, || {
            crate::ui::components::drag_drop::DragPayload::new(
                crate::ui::components::drag_drop::DragSource::ChatMessage(message.id),
                if message.title.is_empty() {
                    message
                        .file
                        .as_ref()
                        .map(|file| file.name.as_str())
                        .unwrap_or(&message.author)
                } else {
                    &message.title
                },
                &message.author,
                match message.kind.as_str() {
                    "bible" => BOOK_OPEN,
                    "song" => MUSIC_NOTES,
                    _ => FILE,
                },
            )
        });
    }
    {
        ui.spacing_mut().item_spacing.x = 4.0;
        let audio = message
            .file
            .as_ref()
            .is_some_and(|f| f.mime.starts_with("audio/"));
        let playable = message.file.as_ref().is_none_or(|f| {
            f.mime.starts_with("audio/")
                || f.mime.starts_with("video/")
                || f.mime.starts_with("image/")
                || matches!(
                    crate::core::models::MediaItem::detect_type(&f.name),
                    "audio" | "video" | "image"
                )
        });
        if playable {
            let label = if audio {
                format!("{MUSIC_NOTES}  {}", language.text("chat-open-in-player"))
            } else {
                format!("{PLAY}  {}", language.text("chat-present"))
            };
            if soft_button(
                ui,
                RichText::new(label).size(11.0).color(tokens.accent),
                tokens,
                true,
            )
            .clicked()
            {
                actions.action = Some((message.id, "project".into()));
            }
        } else if let Some(file) = &message.file
            && soft_button(
                ui,
                RichText::new(format!("{DOWNLOAD_SIMPLE}  {}", language.text("chat-download")))
                    .size(11.0)
                    .color(tokens.accent),
                tokens,
                true,
            )
            .clicked()
        {
            actions.download = Some(file.id.clone());
        }
        for (icon, tooltip, kind) in [
            (PUSH_PIN, language.text("chat-pin-to-quick-bar"), "shortcut"),
            (PLUS, language.text("workspace-add-to-playlist"), "playlist"),
            (LINK_SIMPLE, language.text("chat-copy-content-link"), "copy"),
        ] {
            if tokens
                .icon_button(ui, icon, tooltip, false, Vec2::splat(24.0))
                .clicked()
            {
                actions.action = Some((message.id, kind.into()));
            }
        }
        if playable
            && let Some(file) = &message.file
            && tokens
                .icon_button(
                    ui,
                    DOWNLOAD_SIMPLE,
                    language.text("chat-download-file"),
                    false,
                    Vec2::splat(24.0),
                )
                .clicked()
        {
            actions.download = Some(file.id.clone());
        }
    }
}

fn resource_header(
    ui: &mut egui::Ui,
    icon: &str,
    kind: &str,
    title: &str,
    subtitle: &str,
    tokens: &crate::ui::theme::AppTheme,
) {
    ui.horizontal_top(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(40.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 11.0, tokens.accent_soft);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(21.0),
            tokens.accent,
        );
        ui.vertical(|ui| {
            ui.label(
                RichText::new(crate::ui::i18n::language(ui.ctx()).text(kind))
                    .size(9.0)
                    .strong()
                    .color(tokens.accent),
            );
            ui.add(
                egui::Label::new(
                    RichText::new(title)
                        .size(14.0)
                        .strong()
                        .color(tokens.text_primary),
                )
                .truncate(),
            )
            .on_hover_text(title);
            if !subtitle.is_empty() {
                ui.add(
                    egui::Label::new(RichText::new(subtitle).size(11.0).color(tokens.text_muted))
                        .truncate(),
                );
            }
        });
    });
}

fn content_details(
    ui: &mut egui::Ui,
    message: &Message,
    label: &str,
    tokens: &crate::ui::theme::AppTheme,
) {
    ui.add_space(5.0);
    egui::CollapsingHeader::new(
        RichText::new(crate::ui::i18n::language(ui.ctx()).text(label))
            .size(11.0)
            .color(tokens.text_secondary),
    )
    .id_salt(("shared_message", message.id))
    .show(ui, |ui| {
        ui.add(
            egui::Label::new(
                RichText::new(&message.text)
                    .size(12.5)
                    .color(tokens.text_secondary),
            )
            .wrap(),
        );
    });
}
