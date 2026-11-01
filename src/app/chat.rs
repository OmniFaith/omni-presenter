use super::PresentationApp;
use eframe::egui;

impl PresentationApp {
    pub(super) fn sync_chat(&mut self, ctx: &egui::Context) {
        let chat = &mut self.control_state.chat;
        let Some(service) = chat.service.clone() else {
            return;
        };
        self.process_chat_actions(&service);
        let chat = &mut self.control_state.chat;
        let profile = &self.control_state.settings.active_operator;
        // Credentials are also used by web control when chat is disabled.
        if (chat.last_sync.is_none_or(|last| last.elapsed().as_secs() >= 5)
            || (self.control_state.settings.chat_enabled && chat.profile_id != *profile))
            && let Err(error) = service.sync_profiles(&self.control_state.settings.operators) {
                chat.error = error;
            }
        if !self.control_state.settings.chat_enabled {
            chat.last_sync = Some(std::time::Instant::now());
            chat.notice = None;
            chat.notifications = Default::default();
            if let Some(recorder) = chat.recorder.take() {
                recorder.stop();
            }
            chat.recording_target = None;
            chat.recording_started = None;
            if let Some(mut player) = chat.audio_player.take() {
                player.close();
            }
            if !chat.token.is_empty() {
                service.logout(&chat.token);
                chat.token.clear();
                chat.profile_id.clear();
            }
            return;
        }
        let switching = chat.profile_id != *profile;
        if switching
            || chat
                .last_sync
                .is_none_or(|last| last.elapsed().as_secs() >= 5)
        {
            let result = (|| -> Result<(), String> {
                if switching || chat.token.is_empty() || service.heartbeat(&chat.token).is_err() {
                    service.logout(&chat.token);
                    if let Some(mut player) = chat.audio_player.take() {
                        player.close();
                    }
                    if let Some(recorder) = chat.recorder.take() {
                        recorder.stop();
                    }
                    chat.recording_target = None;
                    chat.input.clear();
                    chat.search.clear();
                    chat.peer = None;
                    chat.channel = "general".into();
                    chat.search_all = false;
                    chat.search_open = false;
                    chat.notification_cursor = None;
                    chat.last_notification_poll = None;
                    chat.notice = None;
                    chat.notifications = Default::default();
                    chat.show_emoji = false;
                    chat.reacting_to = None;
                    chat.history_before = None;
                    chat.token = service.login_local(profile)?;
                    chat.profile_id = profile.clone();
                }
                service.heartbeat(&chat.token)
            })();
            if let Err(error) = result {
                chat.error = error;
            }
            chat.last_sync = Some(std::time::Instant::now());
        }
        if !self.control_state.settings.operator().chat_notifications {
            chat.notice = None;
        }
        if chat
            .last_notification_poll
            .is_none_or(|last| last.elapsed().as_millis() >= 800)
        {
            if let Ok(notifications) = service.notifications(&chat.token) {
                if chat.notice.as_ref().is_some_and(|notice| {
                    !notifications
                        .recent
                        .iter()
                        .any(|item| item.id == notice.item.id)
                }) {
                    chat.notice = None;
                }
                if let Some(cursor) = chat.notification_cursor
                    && let Some(item) = notifications.recent.iter().find(|item| item.id > cursor)
                {
                    let viewing = self.control_state.active_sidebar_page
                        == crate::core::state::SidebarPage::Chat
                        && ctx.input(|i| i.focused)
                        && chat.search.is_empty()
                        && chat.history_before.is_none()
                        && if let Some(channel) = &item.channel_id {
                            chat.peer.is_none() && &chat.channel == channel
                        } else {
                            chat.peer == item.peer_id
                        };
                    if !viewing && self.control_state.settings.operator().chat_notifications {
                        chat.notice = Some(crate::core::state::ChatNotice {
                            item: item.clone(),
                            since: std::time::Instant::now(),
                        });
                        if !ctx.input(|i| i.focused) {
                            ctx.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
                                egui::UserAttentionType::Informational,
                            ));
                        }
                    }
                }
                chat.notification_cursor = Some(
                    chat.notification_cursor
                        .unwrap_or(0)
                        .max(notifications.latest_id),
                );
                chat.notifications = notifications;
            }
            chat.last_notification_poll = Some(std::time::Instant::now());
        }
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }

    fn process_chat_actions(&mut self, service: &crate::services::chat::ChatService) {
        for command in service.pending_actions() {
            if !self.control_state.settings.chat_enabled {
                service.finish_action(
                    &command.id,
                    Err(self
                        .control_state
                        .settings
                        .language
                        .text("services-chat-is-disabled")
                        .into()),
                );
                continue;
            }
            let result = self.apply_chat_action(service, &command);
            if let Err(error) = &result {
                self.control_state.notify(
                    self.control_state
                        .settings
                        .language
                        .format("services-chat", &[("error", error)]),
                );
            }
            service.finish_action(&command.id, result);
        }
    }

    fn apply_chat_action(
        &mut self,
        service: &crate::services::chat::ChatService,
        command: &crate::services::chat::ChatAction,
    ) -> Result<(), String> {
        use crate::core::models::{PlaylistItem, PlaylistItemKind, QuickItem, QuickItemKind};
        use crate::services::chat::SharedContent;
        if !self
            .control_state
            .settings
            .operators
            .iter()
            .any(|p| p.id == command.profile_id)
        {
            return Err("O perfil de origem não existe mais".into());
        }
        let (title, subtitle, kind, quick_kind, content) = match &command.content {
            SharedContent::Bible {
                reference,
                version,
                content,
            } => (
                reference.clone(),
                version.clone(),
                PlaylistItemKind::Bible,
                QuickItemKind::Verse,
                content.clone(),
            ),
            SharedContent::Song {
                title,
                artist,
                content,
                ..
            } => (
                title.clone(),
                artist.clone(),
                PlaylistItemKind::Song,
                QuickItemKind::Song,
                crate::core::markup::strip(content),
            ),
            SharedContent::Media { file } => (
                file.name.clone(),
                file.mime.clone(),
                PlaylistItemKind::Media,
                QuickItemKind::Media,
                service.materialize(file)?,
            ),
        };
        match command.action.as_str() {
            "shortcut" => {
                if self.control_state.settings.active_operator == command.profile_id {
                    self.control_state
                        .add_quick_item(title.clone(), subtitle, quick_kind, content);
                } else {
                    let profile = self
                        .control_state
                        .settings
                        .operators
                        .iter_mut()
                        .find(|p| p.id == command.profile_id)
                        .unwrap();
                    if let Some(item) = profile
                        .quick_items
                        .iter_mut()
                        .find(|i| i.title == title && i.kind == quick_kind)
                    {
                        item.subtitle = subtitle;
                        item.content = content;
                    } else {
                        profile.quick_items.push(QuickItem {
                            id: crate::core::preferences::new_id("chat"),
                            title: title.clone(),
                            subtitle,
                            kind: quick_kind,
                            content,
                        });
                    }
                }
                self.control_state.capture_profiles();
                self.db
                    .save_settings(&self.control_state.settings)
                    .map_err(|e| e.to_string())?;
                self.control_state.notify(
                    self.control_state.settings.language.format(
                        "services-pinned-to-the-profile-s-quick-bar",
                        &[("title", &title)],
                    ),
                );
            }
            "playlist" => {
                self.control_state.playlist.push(PlaylistItem {
                    id: format!("chat_{}", command.id),
                    title: title.clone(),
                    subtitle,
                    kind,
                    played: false,
                    content,
                });
                self.control_state.selected_playlist_index =
                    Some(self.control_state.playlist.len() - 1);
                self.control_state.capture_profiles();
                self.db
                    .save_settings(&self.control_state.settings)
                    .map_err(|e| e.to_string())?;
                self.control_state.notify(
                    self.control_state.settings.language.format(
                        "services-added-to-the-playlist",
                        &[("title", &title)],
                    ),
                );
            }
            "project" => {
                if kind == PlaylistItemKind::Media {
                    if crate::services::media::activate(
                        &mut self.control_state,
                        &mut self.slide_deck,
                        &title,
                        &content,
                        &subtitle,
                        true,
                    )? {
                        self.is_presentation_open = true;
                    }
                } else {
                    self.slide_deck
                        .project_custom_content(title, subtitle, content);
                    self.control_state.apply_theme_to_live();
                    self.is_presentation_open = true;
                    self.control_state.show_presentation_dock =
                        self.control_state.settings.operator().show_dock;
                }
            }
            _ => return Err("Ação inválida".into()),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn incoming_messages_update_sidebar_badge_and_notification_without_marking_unopened_chats_read()
    {
        let mut app =
            PresentationApp::with_db(crate::services::db::AppDb::open_in_memory().unwrap());
        app.control_state
            .settings
            .operators
            .push(crate::core::preferences::OperatorProfile {
                id: "sound".into(),
                name: "Mesa de som".into(),
                ..Default::default()
            });
        let ctx = egui::Context::default();
        app.sync_chat(&ctx);
        let service = app.control_state.chat.service.clone().unwrap();
        let sender = service
            .login("sound", &service.access_code("sound").unwrap())
            .unwrap();
        service
            .send(
                &sender,
                crate::services::chat::SendMessage {
                    recipient_id: Some(app.control_state.settings.active_operator.clone()),
                    text: "Prontos para começar".into(),
                    ..Default::default()
                },
            )
            .unwrap();
        app.control_state.chat.last_notification_poll = None;
        app.sync_chat(&ctx);
        assert_eq!(app.control_state.chat.notifications.total, 1);
        assert_eq!(
            app.control_state.chat.notice.as_ref().unwrap().item.author,
            "Mesa de som"
        );
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(60.0, 700.0),
                )),
                ..Default::default()
            },
            |ui| {
                crate::ui::components::icon_sidebar::render_icon_sidebar(ui, &mut app.control_state)
            },
        );
        output.textures_delta.clear();
        assert!(output.shapes.iter().any(
            |shape| matches!(&shape.shape,egui::Shape::Text(text)if text.galley.job.text=="1")
        ));
        assert_eq!(
            service
                .notifications(&app.control_state.chat.token)
                .unwrap()
                .total,
            1
        );
    }
    #[test]
    fn shared_links_execute_real_projection_playlist_and_per_profile_shortcut_actions() {
        use crate::services::chat::SendMessage;
        let db = crate::services::db::AppDb::open_in_memory().unwrap();
        let mut app = PresentationApp::with_db(db.clone());
        app.control_state
            .settings
            .operators
            .push(crate::core::preferences::OperatorProfile {
                id: "sound".into(),
                name: "Mesa de som".into(),
                ..Default::default()
            });
        let ctx = egui::Context::default();
        app.sync_chat(&ctx);
        let service = app.control_state.chat.service.clone().unwrap();
        let token = app.control_state.chat.token.clone();
        let sound = service
            .login("sound", &service.access_code("sound").unwrap())
            .unwrap();
        let bible = service
            .send(
                &token,
                SendMessage {
                    channel_id: Some("general".into()),
                    kind: "bible".into(),
                    reference: "Salmos 119:1-7".into(),
                    ..Default::default()
                },
            )
            .unwrap();
        let command = service.request_action(&token, bible.id, "project").unwrap();
        app.sync_chat(&ctx);
        assert_eq!(
            service.action_status(&token, &command.id).unwrap().status,
            "applied"
        );
        assert!(app.is_presentation_open);
        assert_eq!(app.slide_deck.total_live_slides(), 7);
        assert!(app.slide_deck.live_slide_text().ends_with("Salmos 119:1"));
        service
            .request_action(&sound, bible.id, "shortcut")
            .unwrap();
        app.sync_chat(&ctx);
        assert!(
            app.control_state.quick_bar.items.is_empty(),
            "remote profile shortcuts do not modify the selected operator"
        );
        let persisted = db.load_settings().unwrap().unwrap();
        let saved = &persisted
            .operators
            .iter()
            .find(|p| p.id == "sound")
            .unwrap()
            .quick_items[0];
        assert_eq!(saved.title, "Salmos 119:1–7");
        assert_eq!(
            crate::core::slides::parse_slides_from_text(&saved.content).len(),
            7
        );
        service
            .request_action(&token, bible.id, "playlist")
            .unwrap();
        app.sync_chat(&ctx);
        let queued = app.control_state.playlist.last().unwrap();
        queued.preview(&mut app.slide_deck).unwrap();
        assert_eq!(app.slide_deck.slides().len(), 7);
        let song_id = db
            .add_song("Louvor compartilhado", "Equipe", "Estrofe A\n\nEstrofe B")
            .unwrap();
        let song = service
            .send(
                &token,
                SendMessage {
                    channel_id: Some("general".into()),
                    kind: "song".into(),
                    song_id: Some(song_id),
                    ..Default::default()
                },
            )
            .unwrap();
        service.request_action(&token, song.id, "project").unwrap();
        app.sync_chat(&ctx);
        assert_eq!(app.slide_deck.live_title(), "Louvor compartilhado");
        assert_eq!(app.slide_deck.total_live_slides(), 2);
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(2, 2)
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        let file = service
            .upload(&token, "fundo.png", "image/png", png.get_ref())
            .unwrap();
        let image = service
            .send(
                &token,
                SendMessage {
                    channel_id: Some("general".into()),
                    kind: "file".into(),
                    file_id: Some(file.id),
                    ..Default::default()
                },
            )
            .unwrap();
        service
            .request_action(&token, image.id, "playlist")
            .unwrap();
        app.sync_chat(&ctx);
        let media = app.control_state.playlist.last().unwrap().clone();
        assert!(std::path::Path::new(&media.content).is_file());
        assert_eq!(std::fs::read(&media.content).unwrap(), png.into_inner());
        let command = service.request_action(&token, image.id, "project").unwrap();
        app.sync_chat(&ctx);
        assert_eq!(
            service.action_status(&token, &command.id).unwrap().status,
            "applied"
        );
        let output = crate::services::media::live_theme(&app.control_state, &app.slide_deck);
        assert_eq!(output.bg_type, "image");
        assert_eq!(output.bg_image_path, media.content);
    }

    #[test]
    fn switching_operators_updates_chat_identity_and_presence() {
        let mut app =
            PresentationApp::with_db(crate::services::db::AppDb::open_in_memory().unwrap());
        let ctx = egui::Context::default();
        app.sync_chat(&ctx);
        let old = app.control_state.chat.token.clone();
        let second = crate::core::preferences::OperatorProfile {
            id: "operator_second".into(),
            name: "Mesa de som".into(),
            ..Default::default()
        };
        app.control_state.settings.operators.push(second);
        app.control_state.settings.active_operator = "operator_second".into();
        app.control_state.chat.input = "Rascunho do primeiro perfil".into();
        app.sync_chat(&ctx);
        let service = app.control_state.chat.service.as_ref().unwrap();
        assert!(service.heartbeat(&old).is_err());
        assert_eq!(app.control_state.chat.profile_id, "operator_second");
        assert!(app.control_state.chat.input.is_empty());
        assert!(
            service
                .profiles()
                .iter()
                .any(|p| p.id == "operator_second" && p.online)
        );
    }
}
