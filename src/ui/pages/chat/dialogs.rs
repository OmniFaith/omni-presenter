use crate::core::i18n::Language;
use crate::core::state::ControlPanelState;
use crate::services::chat::ChatService;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, RichText, Vec2};
use egui_phosphor::regular::{BOOK_OPEN, CHAT_CIRCLE_TEXT, COPY, FILE, MUSIC_NOTES, QR_CODE};

use super::{open_browser, send, target};

#[derive(Clone, Default)]
struct SongSearchResults {
    query: Option<String>,
    songs: std::sync::Arc<Vec<serde_json::Value>>,
}

pub(super) fn mime(name: &str) -> &'static str {
    match name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    }
}

fn access_link(ui: &mut egui::Ui, url: &str, language: Language, theme: &AppTheme) {
    ui.horizontal_wrapped(|ui| {
        ui.hyperlink(url);
        if theme
            .icon_button(
                ui,
                COPY,
                language.text("workspace-copy-link"),
                false,
                Vec2::splat(24.0),
            )
            .clicked()
        {
            ui.ctx().copy_text(url.to_owned());
        }

        if theme
            .icon_button(
                ui,
                QR_CODE,
                "QR Code",
                false,
                Vec2::splat(24.0),
            )
            .on_hover_text("Mostrar QR Code para escanear com a câmera")
            .clicked()
        {
            crate::ui::components::open_qr_dialog(ui.ctx(), "Chat da Equipe", url);
        }
    });
}

pub(super) fn dialogs(ui: &egui::Ui, state: &mut ControlPanelState, service: &ChatService) {
    use crate::ui::components::dialog;
    let tokens = state.theme.clone();
    let language = state.settings.language;
    if state.chat.show_access {
        let close = dialog::show(
            ui.ctx(),
            "chat_access",
            language.text("chat-network-access"),
            language.text("chat-chat-with-the-team-from-your-phone-or-browser"),
            450.0,
            &tokens,
            |ui| {
                egui::Frame::new()
                    .fill(tokens.bg_pill)
                    .corner_radius(10.0)
                    .inner_margin(14)
                    .show(ui, |ui| {
                        ui.label(RichText::new(&state.settings.operator().name).strong());
                        ui.label(
                            language.text("chat-enter-password-from-settings"),
                        );
                        if state.settings.operator().web_password_hash.is_empty() {
                            ui.label(language.text("settings-default-password"));
                        }
                    });
                let host = if state.settings.allow_lan {
                    crate::services::web::local_address()
                } else {
                    "127.0.0.1".into()
                };
                ui.label(
                    RichText::new(language.text("chat-chat-addresses"))
                        .size(12.0)
                        .strong(),
                );
                if state.settings.allow_lan && !state.settings_ui.discovery_chat_url.is_empty() {
                    ui.label(
                        RichText::new(language.text("discovery-title"))
                            .size(11.5)
                            .strong(),
                    );
                    access_link(ui, &state.settings_ui.discovery_chat_url, language, &tokens);
                    ui.add_space(6.0);
                }
                ui.label(
                    RichText::new(language.text("discovery-ip-alternative"))
                        .size(11.5)
                        .color(tokens.text_muted),
                );
                access_link(
                    ui,
                    &crate::services::web::http_url(&host, state.settings.server_port, "/chat"),
                    language,
                    &tokens,
                );
                if state.chat.https_error.is_none() {
                    access_link(
                        ui,
                        &crate::services::web::https_url(
                            &host,
                            crate::services::chat::tls_port(state.settings.server_port),
                            "/chat",
                        ),
                        language,
                        &tokens,
                    );
                }
                ui.label(
                    RichText::new(language.text(
                        "chat-to-record-in-the-browser-use-https-and-trust-the",
                    ))
                    .size(11.0)
                    .color(tokens.text_muted),
                );
                if let Some(error) = &state.chat.https_error {
                    ui.colored_label(tokens.status_warning_fg, error);
                }
                ui.add_space(8.0);
                let primary_chat_url = crate::services::web::http_url(&host, state.settings.server_port, "/chat");
                ui.vertical_centered(|ui| {
                    crate::ui::components::qr_card(
                        ui,
                        &primary_chat_url,
                        120.0,
                        Some("Aponte a câmera do celular para entrar no chat"),
                        &tokens,
                    );
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if dialog::action(ui, language.text("chat-open-in-browser"), true, &tokens).clicked() {
                        open_browser(ui, state, service);
                    }
                    if dialog::action(ui, language.text("common-close"), false, &tokens).clicked() {
                        state.chat.show_access = false;
                    }
                });
            },
        );
        state.chat.show_access &= !close;
    }
    if state.chat.show_channels {
        if state.chat.new_channel_tool.is_empty() {
            state.chat.new_channel_tool = "general".into();
        }
        let close = dialog::show(
            ui.ctx(),
            "chat_channel",
            language.text(if state.chat.editing_channel.is_some() {
                "chat-edit"
            } else {
                "chat-create"
            }),
            language.text("chat-channel-hint"),
            430.0,
            &tokens,
            |ui| {
                ui.label(
                    RichText::new(language.text("chat-channel-name"))
                        .size(12.0)
                        .strong(),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut state.chat.new_channel)
                        .hint_text(language.text("chat-channel-placeholder"))
                        .desired_width(ui.available_width()),
                );
                ui.label(
                    RichText::new(language.text("chat-channel-tool"))
                        .size(12.0)
                        .strong(),
                );
                let tools = [
                    (
                        "general",
                        CHAT_CIRCLE_TEXT,
                        language.text("chat-general-tool"),
                    ),
                    ("bible", BOOK_OPEN, language.text("nav-bible")),
                    ("songs", MUSIC_NOTES, language.text("chat-songs-tool")),
                    ("media", FILE, language.text("chat-media-tool")),
                ];
                ui.columns(2, |cols| {
                    for (index, (key, icon, label)) in tools.iter().enumerate() {
                        let active = state.chat.new_channel_tool == *key;
                        let ui = &mut cols[index % 2];
                        if ui
                            .add_sized(
                                Vec2::new(ui.available_width(), 38.0),
                                egui::Button::new(
                                    RichText::new(format!("{icon}  {label}")).size(12.0).color(
                                        if active {
                                            tokens.accent
                                        } else {
                                            tokens.text_secondary
                                        },
                                    ),
                                )
                                .fill(if active {
                                    tokens.accent_soft
                                } else {
                                    tokens.bg_pill
                                })
                                .stroke(egui::Stroke::new(
                                    1.0,
                                    if active {
                                        tokens.accent
                                    } else {
                                        tokens.border_default
                                    },
                                ))
                                .corner_radius(8.0),
                            )
                            .clicked()
                        {
                            state.chat.new_channel_tool = (*key).into();
                        }
                    }
                });
                ui.label(
                    RichText::new(language.text("chat-channel-description"))
                        .size(12.0)
                        .strong(),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut state.chat.new_channel_description)
                        .hint_text(language.text("chat-channel-description"))
                        .desired_width(ui.available_width()),
                );
                if !state.chat.error.is_empty() {
                    ui.colored_label(tokens.status_warning_fg, language.text(&state.chat.error));
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if dialog::action(ui, language.text("common-cancel"), false, &tokens).clicked()
                    {
                        state.chat.show_channels = false;
                    }
                    ui.add_enabled_ui(!state.chat.new_channel.trim().is_empty(), |ui| {
                        if dialog::action(
                            ui,
                            language.text(if state.chat.editing_channel.is_some() {
                                "common-save"
                            } else {
                                "chat-create"
                            }),
                            true,
                            &tokens,
                        )
                        .clicked()
                        {
                            let result = if let Some(id) = state.chat.editing_channel.as_deref() {
                                service
                                    .update_channel(
                                        &state.chat.token,
                                        id,
                                        &state.chat.new_channel,
                                        &state.chat.new_channel_tool,
                                        &state.chat.new_channel_description,
                                    )
                                    .map(|()| id.to_owned())
                            } else {
                                service
                                    .create_channel(
                                        &state.chat.token,
                                        &state.chat.new_channel,
                                        &state.chat.new_channel_tool,
                                        &state.chat.new_channel_description,
                                    )
                                    .map(|channel| channel.id)
                            };
                            match result {
                                Ok(id) => {
                                    state.chat.channel = id;
                                    state.chat.peer = None;
                                    state.chat.history_before = None;
                                    state.chat.new_channel.clear();
                                    state.chat.new_channel_description.clear();
                                    state.chat.error.clear();
                                    state.chat.show_channels = false;
                                }
                                Err(error) => state.chat.error = error,
                            }
                        }
                    });
                });
            },
        );
        state.chat.show_channels &= !close;
        if !state.chat.show_channels {
            state.chat.editing_channel = None;
        }
    }
    if state.chat.show_bible {
        let close = dialog::show(
            ui.ctx(),
            "chat_bible",
            language.text("chat-share-passage"),
            language.text("chat-send-a-reference-the-team-can-present-or-save"),
            430.0,
            &tokens,
            |ui| {
                if state.chat.bible_version.is_empty() {
                    state.chat.bible_version = state.bible_version.clone();
                }
                ui.label(
                    RichText::new(language.text("chat-bible-translation"))
                        .size(12.0)
                        .strong(),
                );
                {
                    egui::ComboBox::from_id_salt("chat_bible_version")
                        .selected_text(&state.chat.bible_version)
                        .show_ui(ui, |ui| {
                            for (id, name) in state.bible.library.catalog() {
                                ui.selectable_value(&mut state.chat.bible_version, id.clone(), id)
                                    .on_hover_text(name);
                            }
                        });
                }
                ui.label(
                    RichText::new(language.text("chat-reference"))
                        .size(12.0)
                        .strong(),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut state.chat.bible_reference)
                        .hint_text(language.text("services-psalms-119-1-7"))
                        .desired_width(ui.available_width()),
                );
                ui.checkbox(
                    &mut state.bible.merge_selection,
                    language.text("chat-combine-verses-into-one-slide"),
                );
                if !state.chat.error.is_empty() {
                    ui.colored_label(tokens.status_warning_fg, &state.chat.error);
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if dialog::action(ui, language.text("common-cancel"), false, &tokens).clicked() {
                        state.chat.show_bible = false;
                    }
                    if dialog::action(ui, language.text("chat-send-passage"), true, &tokens).clicked() {
                        let mut request = target(&state.chat);
                        request.kind = "bible".into();
                        request.reference = state.chat.bible_reference.clone();
                        request.version = state.chat.bible_version.clone();
                        request.merge = state.bible.merge_selection;
                        if send(&mut state.chat, service, request) {
                            state.chat.show_bible = false;
                        }
                    }
                });
            },
        );
        state.chat.show_bible &= !close;
    }
    if state.chat.show_songs {
        let cache_id = egui::Id::unique("chat_song_search_results");
        let mut results = ui
            .ctx()
            .data(|data| data.get_temp::<SongSearchResults>(cache_id))
            .unwrap_or_default();
        let close = dialog::show(
            ui.ctx(),
            "chat_song",
            language.text("chat-share-song"),
            language.text("chat-choose-lyrics-from-the-app-library"),
            460.0,
            &tokens,
            |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut state.chat.song_search)
                        .hint_text(language.text("chat-search-title-artist-or-lyrics"))
                        .desired_width(ui.available_width()),
                );
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        if results.query.as_ref() != Some(&state.chat.song_search) {
                            match service.songs(&state.chat.token, &state.chat.song_search) {
                                Ok(songs) => {
                                    results.query = Some(state.chat.song_search.clone());
                                    results.songs = std::sync::Arc::new(
                                        songs.as_array().cloned().unwrap_or_default(),
                                    );
                                }
                                Err(error) => state.chat.error = error,
                            }
                        }
                        {
                            let songs = results.songs.as_ref();
                            if songs.is_empty() {
                                ui.label(
                                    RichText::new(language.text("chat-no-songs-found"))
                                        .color(tokens.text_muted),
                                );
                            }
                            for song in songs {
                                let label = format!(
                                    "{MUSIC_NOTES}  {} · {}",
                                    song["title"].as_str().unwrap_or(""),
                                    song["artist"].as_str().unwrap_or("")
                                );
                                if ui
                                    .add_sized(
                                        Vec2::new(ui.available_width(), 38.0),
                                        egui::Button::new(RichText::new(label).size(12.0))
                                            .fill(tokens.bg_pill)
                                            .stroke(egui::Stroke::NONE)
                                            .corner_radius(8.0),
                                    )
                                    .clicked()
                                {
                                    let mut request = target(&state.chat);
                                    request.kind = "song".into();
                                    request.song_id = song["id"].as_i64();
                                    if send(&mut state.chat, service, request) {
                                        state.chat.show_songs = false;
                                    }
                                }
                            }
                        }
                    });
                if !state.chat.error.is_empty() {
                    ui.colored_label(tokens.status_warning_fg, &state.chat.error);
                }
                if dialog::action(ui, language.text("common-close"), false, &tokens).clicked() {
                    state.chat.show_songs = false;
                }
            },
        );
        state.chat.show_songs &= !close;
        if state.chat.show_songs {
            ui.ctx()
                .data_mut(|data| data.insert_temp(cache_id, results));
        } else {
            ui.ctx()
                .data_mut(|data| data.remove::<SongSearchResults>(cache_id));
        }
    } else {
        ui.ctx().data_mut(|data| {
            data.remove::<SongSearchResults>(egui::Id::unique("chat_song_search_results"))
        });
    }
}
