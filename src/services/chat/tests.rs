use super::*;
use crate::core::preferences::OperatorProfile;
use crate::services::db::AppDb;

#[test]
fn reactions_toggle_per_profile_and_private_messages_remain_private() {
    let (_, chat, tokens) = fixture();
    let message = chat
        .send(
            &tokens[0],
            SendMessage {
                channel_id: Some("general".into()),
                text: "Vamos começar".into(),
                ..Default::default()
            },
        )
        .unwrap();
    let first = chat.react(&tokens[1], message.id, "🙏").unwrap();
    assert_eq!(first.reactions[0].users, vec!["person_1"]);
    let both = chat.react(&tokens[0], message.id, "🙏").unwrap();
    assert_eq!(both.reactions[0].users.len(), 2);
    let removed = chat.react(&tokens[1], message.id, "🙏").unwrap();
    assert_eq!(removed.reactions[0].users, vec!["person_0"]);
    assert!(chat.react(&tokens[0], message.id, "not an emoji").is_err());
    let private = chat
        .send(
            &tokens[0],
            SendMessage {
                recipient_id: Some("person_1".into()),
                text: "Reservado".into(),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(chat.react(&tokens[2], private.id, "👍").is_err());
    assert!(chat.delete_message(&tokens[1], message.id).is_err());
    let deleted = chat.delete_message(&tokens[0], message.id).unwrap();
    assert!(deleted.deleted);
    assert_eq!(deleted.text, "services-message-deleted");
    assert!(deleted.reactions.is_empty());
    assert!(chat.react(&tokens[1], message.id, "🙏").is_err());
}

#[test]
fn unread_counts_and_read_receipts_are_scoped_to_profile_and_conversation_and_persist() {
    let (db, chat, tokens) = fixture();
    let public = chat
        .send(
            &tokens[0],
            SendMessage {
                channel_id: Some("general".into()),
                text: "Equipe".into(),
                ..Default::default()
            },
        )
        .unwrap();
    chat.send(
        &tokens[0],
        SendMessage {
            recipient_id: Some("person_1".into()),
            text: "Só você".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(chat.notifications(&tokens[0]).unwrap().total, 0);
    let unread = chat.notifications(&tokens[1]).unwrap();
    assert_eq!(unread.total, 2);
    assert_eq!(unread.channels["general"], 1);
    assert_eq!(unread.private["person_0"], 1);
    assert_eq!(chat.notifications(&tokens[2]).unwrap().total, 1);
    let read = chat
        .mark_read(&tokens[1], Some("general"), None, Some(public.id))
        .unwrap();
    assert_eq!(read.total, 1);
    assert_eq!(chat.notifications(&tokens[2]).unwrap().total, 1);
    assert_eq!(
        chat.mark_read(&tokens[1], None, Some("person_0"), None)
            .unwrap()
            .total,
        0
    );
    let operators: Vec<_> = (0..3)
        .map(|i| OperatorProfile {
            id: format!("person_{i}"),
            name: format!("Pessoa {i}"),
            ..Default::default()
        })
        .collect();
    let restored = ChatService::new(db, &operators).unwrap();
    let token = restored
        .login("person_1", &restored.access_code("person_1").unwrap())
        .unwrap();
    assert_eq!(restored.notifications(&token).unwrap().total, 0);
    chat.delete_message(&tokens[0], public.id).unwrap();
    assert_eq!(chat.notifications(&tokens[2]).unwrap().total, 0);
}

#[test]
fn deleting_a_shared_private_message_removes_attachment_access_and_cancels_pending_actions() {
    let (_, chat, tokens) = fixture();
    let file = chat
        .upload(&tokens[0], "documento.txt", "text/plain", b"privado")
        .unwrap();
    let message = chat
        .send(
            &tokens[0],
            SendMessage {
                recipient_id: Some("person_1".into()),
                kind: "file".into(),
                file_id: Some(file.id.clone()),
                ..Default::default()
            },
        )
        .unwrap();
    let action = chat
        .request_action(&tokens[1], message.id, "playlist")
        .unwrap();
    assert!(chat.download(&tokens[1], &file.id).is_ok());
    chat.delete_message(&tokens[0], message.id).unwrap();
    assert!(chat.download(&tokens[1], &file.id).is_err());
    assert!(chat.pending_actions().is_empty());
    assert_eq!(
        chat.action_status(&tokens[1], &action.id).unwrap().status,
        "error"
    );
    assert!(
        chat.message(&tokens[1], message.id)
            .unwrap()
            .shared
            .is_none()
    );
}

#[test]
fn profile_photos_sync_without_exposing_codes_or_photo_payloads_in_profile_list() {
    let (db, chat, _) = fixture();
    let mut operators: Vec<_> = (0..3)
        .map(|i| OperatorProfile {
            id: format!("person_{i}"),
            name: format!("Pessoa {i}"),
            ..Default::default()
        })
        .collect();
    let mut image = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(24, 40)
        .write_to(&mut image, image::ImageFormat::Png)
        .unwrap();
    let photo = crate::services::profiles::prepare_photo(image.get_ref()).unwrap();
    operators[0].avatar = Some(photo.clone());
    chat.sync_profiles(&operators).unwrap();
    assert_eq!(chat.avatar("person_0").unwrap(), photo);
    let views = serde_json::to_value(chat.profiles()).unwrap();
    assert!(
        views[0]["avatar_url"]
            .as_str()
            .unwrap()
            .starts_with("/api/chat/avatars/")
    );
    assert!(views[0].get("code").is_none());
    assert!(views[0].get("avatar").is_none());
    let restored = ChatService::new(db, &operators).unwrap();
    assert_eq!(restored.avatar("person_0").unwrap(), photo);
}

pub(super) fn fixture() -> (AppDb, ChatService, Vec<String>) {
    let db = AppDb::open_in_memory().unwrap();
    let operators: Vec<_> = (0..3)
        .map(|i| OperatorProfile {
            id: format!("person_{i}"),
            name: format!("Pessoa {i}"),
            ..Default::default()
        })
        .collect();
    let chat = ChatService::new(db.clone(), &operators).unwrap();
    let tokens = operators
        .iter()
        .map(|p| {
            chat.login(&p.id, &chat.access_code(&p.id).unwrap())
                .unwrap()
        })
        .collect();
    (db, chat, tokens)
}

#[test]
fn channels_presence_search_and_private_messages_are_profile_scoped() {
    let (_, chat, tokens) = fixture();
    assert!(chat.login("person_0", "errado").is_err());
    assert!(chat.profiles().iter().all(|p| p.online));
    let channel = chat
        .create_channel(&tokens[0], "Projeção", "bible", "Passagens do culto")
        .unwrap();
    let public = chat
        .send(
            &tokens[0],
            SendMessage {
                channel_id: Some(channel.id.clone()),
                text: "Oração e louvor".into(),
                ..Default::default()
            },
        )
        .unwrap();
    let private = chat
        .send(
            &tokens[0],
            SendMessage {
                recipient_id: Some("person_1".into()),
                text: "Mensagem reservada".into(),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        chat.snapshot(&tokens[1], Some(&channel.id), None, "oracao", None)
            .unwrap()
            .messages[0]
            .id,
        public.id
    );
    assert_eq!(
        chat.snapshot(&tokens[1], None, Some("person_0"), "", None)
            .unwrap()
            .messages[0]
            .id,
        private.id
    );
    assert_eq!(
        chat.snapshot(&tokens[0], None, Some("person_1"), "", None)
            .unwrap()
            .messages
            .len(),
        1
    );
    let outsider = chat
        .snapshot(&tokens[2], None, None, "reservada", None)
        .unwrap();
    assert!(outsider.messages.is_empty());
    assert_eq!(outsider.latest_id, public.id);
    assert!(
        chat.snapshot("not-a-session", None, None, "", None)
            .is_err()
    );
    chat.logout(&tokens[1]);
    assert!(
        !chat
            .profiles()
            .iter()
            .find(|p| p.id == "person_1")
            .unwrap()
            .online
    );
    assert!(chat.heartbeat(&tokens[1]).is_err());
}

#[test]
fn uploads_survive_restart_and_private_files_are_not_downloadable_by_others() {
    let (db, chat, tokens) = fixture();
    let code = chat.access_code("person_1").unwrap();
    let file = chat
        .upload(
            &tokens[0],
            "../../escala.pdf",
            "application/pdf",
            b"PDF bytes",
        )
        .unwrap();
    assert_eq!(file.name, "escala.pdf");
    assert!(chat.download(&tokens[1], &file.id).is_err());
    assert!(
        chat.send(
            &tokens[1],
            SendMessage {
                channel_id: Some("general".into()),
                kind: "file".into(),
                file_id: Some(file.id.clone()),
                ..Default::default()
            }
        )
        .is_err()
    );
    chat.send(
        &tokens[0],
        SendMessage {
            recipient_id: Some("person_1".into()),
            kind: "file".into(),
            file_id: Some(file.id.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(chat.download(&tokens[1], &file.id).unwrap().2, b"PDF bytes");
    assert!(chat.download(&tokens[2], &file.id).is_err());
    drop(chat);
    let operators: Vec<_> = (0..3)
        .map(|i| OperatorProfile {
            id: format!("person_{i}"),
            name: format!("Pessoa {i}"),
            ..Default::default()
        })
        .collect();
    let restored = ChatService::new(db, &operators).unwrap();
    assert!(restored.profiles().iter().all(|p| !p.online));
    let token = restored.login("person_1", &code).unwrap();
    assert_eq!(restored.download(&token, &file.id).unwrap().2, b"PDF bytes");
    assert_eq!(
        restored
            .snapshot(&token, None, Some("person_0"), "", None)
            .unwrap()
            .messages
            .len(),
        1
    );
}

#[test]
fn bible_and_song_messages_use_real_catalog_content_and_history_is_paginated() {
    let (db, chat, tokens) = fixture();
    let shared_channel = chat
        .create_channel(&tokens[0], "Shared content", "general", "")
        .unwrap();
    let song_id = db
        .add_song(
            "Louvor da equipe",
            "Equipe",
            "Primeira estrofe\n\nSegunda estrofe",
        )
        .unwrap();
    let bible = chat
        .send(
            &tokens[0],
            SendMessage {
                channel_id: Some(shared_channel.id.clone()),
                kind: "bible".into(),
                reference: "Salmos 119:1-7".into(),
                version: crate::core::bible::DEFAULT_BIBLE.into(),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(bible.title.starts_with("Salmos 119:1–7"));
    assert!(bible.text.contains("Bem-aventurados"));
    let song = chat
        .send(
            &tokens[0],
            SendMessage {
                channel_id: Some(shared_channel.id.clone()),
                kind: "song".into(),
                song_id: Some(song_id),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(song.text, "Primeira estrofe\n\nSegunda estrofe");
    assert!(
        chat.songs(&tokens[1], "equipe")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["id"] == song_id)
    );
    for number in 0..105 {
        chat.send(
            &tokens[0],
            SendMessage {
                channel_id: Some("general".into()),
                text: format!("Mensagem {number}"),
                ..Default::default()
            },
        )
        .unwrap();
    }
    let latest = chat
        .snapshot(&tokens[1], Some("general"), None, "", None)
        .unwrap();
    assert_eq!(latest.messages.len(), 100);
    let older = chat
        .snapshot(
            &tokens[1],
            Some("general"),
            None,
            "",
            Some(latest.messages[0].id),
        )
        .unwrap();
    assert_eq!(older.messages.len(), 5);
    assert!(older.messages.last().unwrap().id < latest.messages[0].id);
    assert!(
        chat.send(
            &tokens[0],
            SendMessage {
                channel_id: Some("general".into()),
                recipient_id: Some("person_1".into()),
                text: "ambígua".into(),
                ..Default::default()
            }
        )
        .is_err()
    );
}

#[test]
fn sessions_and_operator_switching_do_not_disconnect_other_clients() {
    let (_, chat, tokens) = fixture();
    let other = chat
        .login("person_0", &chat.access_code("person_0").unwrap())
        .unwrap();
    chat.logout(&tokens[0]);
    chat.heartbeat(&other).unwrap();
    let operators = vec![OperatorProfile {
        id: "person_0".into(),
        name: "Renomeado".into(),
        ..Default::default()
    }];
    chat.sync_profiles(&operators).unwrap();
    assert_eq!(chat.profiles()[0].name, "Renomeado");
    assert!(chat.heartbeat(&tokens[1]).is_err());
    assert!(chat.heartbeat(&tokens[2]).is_err());
    chat.heartbeat(&other).unwrap();
}

#[test]
fn shared_content_actions_and_links_obey_private_message_permissions() {
    let (_, chat, tokens) = fixture();
    let message = chat
        .send(
            &tokens[0],
            SendMessage {
                recipient_id: Some("person_1".into()),
                kind: "bible".into(),
                reference: "Salmos 119:1-7".into(),
                ..Default::default()
            },
        )
        .unwrap();
    let SharedContent::Bible { content, .. } = message.shared.as_ref().unwrap() else {
        panic!("missing Bible reference");
    };
    assert_eq!(
        crate::core::slides::parse_slides_from_text(content).len(),
        7
    );
    assert!(chat.message(&tokens[2], message.id).is_err());
    assert!(
        chat.request_action(&tokens[2], message.id, "project")
            .is_err()
    );
    let command = chat
        .request_action(&tokens[1], message.id, "shortcut")
        .unwrap();
    assert_eq!(command.status, "queued");
    assert!(chat.action_status(&tokens[2], &command.id).is_err());
    let pending = chat.pending_actions();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].profile_id, "person_1");
    chat.finish_action(&command.id, Ok(()));
    assert_eq!(
        chat.action_status(&tokens[1], &command.id).unwrap().status,
        "applied"
    );
    let merged = chat
        .send(
            &tokens[0],
            SendMessage {
                channel_id: Some("general".into()),
                kind: "bible".into(),
                reference: "Salmos 119:1-7".into(),
                merge: true,
                ..Default::default()
            },
        )
        .unwrap();
    let SharedContent::Bible { content, .. } = merged.shared.unwrap() else {
        panic!("missing Bible reference");
    };
    assert_eq!(
        crate::core::slides::parse_slides_from_text(&content).len(),
        1
    );
}
