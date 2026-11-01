use crate::core::state::ControlPanelState;
use crate::services::chat::ChatService;
use eframe::egui::{self, RichText, Vec2};
use egui_phosphor::regular::{
    BOOK_OPEN, MICROPHONE, MUSIC_NOTES, PAPER_PLANE_RIGHT, PLUS, SMILEY, STOP,
};

use super::dialogs::mime;
use super::{send, target, upload};

pub(super) fn composer(ui: &mut egui::Ui, state: &mut ControlPanelState, service: &ChatService) {
    let tokens = state.theme.clone();
    let language = state.settings.language;
    if let Some(recorder) = &state.chat.recorder {
        ui.horizontal(|ui| {
            ui.colored_label(
                tokens.status_warning_fg,
                format!(
                    "● {}",
                    language.format(
                        "chat-recording-s",
                        &[(
                            "seconds",
                            &state
                                .chat
                                .recording_started
                                .map(|time| time.elapsed().as_secs())
                                .unwrap_or(0)
                                .to_string()
                        )]
                    )
                ),
            );
            if ui.button(language.text("common-cancel")).clicked() {
                state.chat.recording_cancelled = true;
                recorder.stop();
            }
            if ui
                .button(format!("{STOP} {}", language.text("chat-stop-and-send")))
                .clicked()
            {
                recorder.stop();
            }
        });
    }
    egui::Frame::new()
        .fill(tokens.bg_pill)
        .stroke(egui::Stroke::new(1.0, tokens.border_default))
        .corner_radius(16.0)
        .inner_margin(10)
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                let chat_picker_id = ui.make_persistent_id("chat_composer_file_picker");
                let chat_task = ui
                    .ctx()
                    .data_mut(|data| data.get_temp::<crate::services::tasks::BackgroundTask<Option<std::path::PathBuf>>>(chat_picker_id));
                if let Some(task) = &chat_task {
                    ui.ctx().request_repaint_after(std::time::Duration::from_millis(33));
                    if let Some(result) = task.take() {
                        ui.ctx().data_mut(|data| data.remove::<crate::services::tasks::BackgroundTask<Option<std::path::PathBuf>>>(chat_picker_id));
                        if let Ok(Some(path)) = result {
                            let destination = target(&state.chat);
                            let result = std::fs::metadata(&path)
                                .map_err(|e| e.to_string())
                                .and_then(|m| {
                                    if m.len() > crate::services::chat::MAX_UPLOAD as u64 {
                                        Err(language.text("chat-file-exceeds-256-mib").into())
                                    } else {
                                        std::fs::read(&path).map_err(|e| e.to_string())
                                    }
                                });
                            match result {
                                Ok(bytes) => {
                                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                                    upload(
                                        &mut state.chat,
                                        service,
                                        &name,
                                        mime(&name),
                                        &bytes,
                                        destination,
                                        false,
                                    );
                                }
                                Err(error) => state.chat.error = error,
                            }
                        }
                    }
                }
                if tokens
                    .icon_button(
                        ui,
                        PLUS,
                        language.text("chat-send-file-or-media"),
                        false,
                        Vec2::splat(28.0),
                    )
                    .clicked()
                    && chat_task.is_none()
                {
                    let task = crate::services::tasks::BackgroundTask::spawn_with_repaint(ui.ctx(), move || {
                        Ok(rfd::FileDialog::new().pick_file())
                    });
                    ui.ctx().data_mut(|data| data.insert_temp(chat_picker_id, task));
                }
                let width = (ui.available_width() - 152.0).max(40.0);
                let input = ui.add_enabled(
                    state.chat.recorder.is_none(),
                    egui::TextEdit::singleline(&mut state.chat.input)
                        .hint_text(language.text("chat-write-a-message"))
                        .desired_width(width)
                        .frame(egui::Frame::NONE),
                );
                if tokens
                    .icon_button(
                        ui,
                        BOOK_OPEN,
                        language.text("chat-send-bible-passage"),
                        state.chat.show_bible,
                        Vec2::splat(28.0),
                    )
                    .clicked()
                {
                    state.chat.show_bible = !state.chat.show_bible;
                }
                if tokens
                    .icon_button(
                        ui,
                        MUSIC_NOTES,
                        language.text("chat-send-a-song-from-the-library"),
                        state.chat.show_songs,
                        Vec2::splat(28.0),
                    )
                    .clicked()
                {
                    state.chat.show_songs = !state.chat.show_songs;
                }
                let emoji_button = tokens.icon_button(
                    ui,
                    SMILEY,
                    language.text("services-emojis"),
                    state.chat.show_emoji,
                    Vec2::splat(28.0),
                );
                if emoji_button.clicked() {
                    state.chat.show_emoji = !state.chat.show_emoji;
                    state.chat.reacting_to = None;
                    state.chat.emoji_anchor = Some(emoji_button.rect);
                    state.chat.emoji_opened_frame = ui.ctx().cumulative_frame_nr();
                }
                let has_text = !state.chat.input.trim().is_empty();
                let pressed = ui
                    .add(
                        egui::Button::new(
                            RichText::new(if has_text {
                                PAPER_PLANE_RIGHT
                            } else {
                                MICROPHONE
                            })
                            .size(19.0)
                            .color(tokens.text_on_accent),
                        )
                        .fill(tokens.accent)
                        .stroke(egui::Stroke::NONE)
                        .corner_radius(12.0)
                        .min_size(Vec2::splat(36.0)),
                    )
                    .on_hover_text(language.text(if has_text {
                        "chat-send-message"
                    } else {
                        "chat-record-audio"
                    }))
                    .clicked();
                if has_text
                    && (pressed
                        || (input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                {
                    let mut request = target(&state.chat);
                    request.text = state.chat.input.trim().into();
                    if send(&mut state.chat, service, request) {
                        state.chat.input.clear();
                        input.request_focus();
                    }
                } else if pressed && state.chat.recorder.is_none() {
                    state.chat.recording_target = Some(target(&state.chat));
                    state.chat.recording_cancelled = false;
                    state.chat.recorder = Some(crate::services::chat::recording::Recorder::start());
                    state.chat.recording_started = Some(std::time::Instant::now());
                }
            });
        });
}
