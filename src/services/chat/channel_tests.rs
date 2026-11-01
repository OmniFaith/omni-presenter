use super::*;

#[test]
fn starts_with_only_general_and_localizes_its_display_name() {
    let (_, service, tokens) = tests::fixture();
    let snapshot = service.snapshot(&tokens[0], None, None, "", None).unwrap();
    assert_eq!(snapshot.channels.len(), 1);
    assert!(snapshot.messages.is_empty());
    let general = &snapshot.channels[0];
    assert_eq!(general.id, "general");
    assert_eq!(
        general.display_name(crate::core::i18n::Language::English),
        "General"
    );
    assert_eq!(
        general.display_name(crate::core::i18n::Language::Portuguese),
        "Geral"
    );
    assert_eq!(
        general.display_name(crate::core::i18n::Language::Spanish),
        "General"
    );
}

#[test]
fn batch_channel_operations_authenticate_persist_and_keep_general() {
    let (db, service, tokens) = tests::fixture();
    let a = service
        .create_channel(&tokens[0], "Sound", "general", "Team")
        .unwrap();
    let b = service
        .create_channel(&tokens[0], "Video", "media", "")
        .unwrap();
    let c = service
        .create_channel(&tokens[0], "Music", "songs", "")
        .unwrap();
    assert!(
        service
            .move_channels("invalid", &[a.id.clone()], &c.id, true)
            .is_err()
    );
    service
        .move_channels(&tokens[0], &[a.id.clone(), b.id.clone()], &c.id, true)
        .unwrap();
    let snapshot = service.snapshot(&tokens[1], None, None, "", None).unwrap();
    assert_eq!(
        snapshot
            .channels
            .iter()
            .map(|channel| channel.id.as_str())
            .collect::<Vec<_>>(),
        ["general", &c.id, &a.id, &b.id]
    );
    service
        .send(
            &tokens[0],
            SendMessage {
                channel_id: Some(a.id.clone()),
                text: "Sound check".into(),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(service.notifications(&tokens[1]).unwrap().total, 1);
    service
        .mark_channels_read(&tokens[1], &[a.id.clone(), b.id.clone()])
        .unwrap();
    assert_eq!(service.notifications(&tokens[1]).unwrap().total, 0);
    assert_eq!(service.notifications(&tokens[2]).unwrap().total, 1);
    assert!(
        service
            .delete_channels(&tokens[0], &[a.id.clone(), "general".into()])
            .is_err()
    );
    assert!(
        service
            .snapshot(&tokens[0], None, None, "", None)
            .unwrap()
            .channels
            .iter()
            .any(|channel| channel.id == a.id)
    );
    service
        .update_channel(&tokens[0], &a.id, "Audio", "general", "Updated")
        .unwrap();
    service
        .delete_channels(&tokens[0], &[a.id.clone(), b.id.clone()])
        .unwrap();
    assert_eq!(service.notifications(&tokens[2]).unwrap().total, 0);
    let stored: Store = serde_json::from_str(&db.chat_load().unwrap().unwrap()).unwrap();
    assert_eq!(
        stored
            .channels
            .iter()
            .map(|channel| channel.id.as_str())
            .collect::<Vec<_>>(),
        ["general", &c.id]
    );
    assert!(stored.messages.is_empty());
}

#[test]
fn legacy_cleanup_preserves_real_conversations_and_custom_channels() {
    let (db, service, tokens) = tests::fixture();
    {
        let mut runtime = service.runtime.lock().unwrap();
        runtime.store.channels.extend([
            Channel {
                id: "songs".into(),
                name: "louvor".into(),
                tool: "songs".into(),
                description: String::new(),
            },
            Channel {
                id: "media".into(),
                name: "mídias".into(),
                tool: "media".into(),
                description: String::new(),
            },
            Channel {
                id: "custom".into(),
                name: "My channel".into(),
                tool: "general".into(),
                description: String::new(),
            },
        ]);
    }
    service
        .send(
            &tokens[0],
            SendMessage {
                channel_id: Some("songs".into()),
                text: "Real conversation".into(),
                ..Default::default()
            },
        )
        .unwrap();
    let operators = (0..3)
        .map(|index| crate::core::preferences::OperatorProfile {
            id: format!("person_{index}"),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    let restored = ChatService::new(db, &operators).unwrap();
    let stored = &restored.runtime.lock().unwrap().store;
    assert!(stored.channels.iter().any(|channel| channel.id == "songs"));
    assert!(stored.channels.iter().any(|channel| channel.id == "custom"));
    assert!(!stored.channels.iter().any(|channel| channel.id == "media"));
    assert_eq!(stored.messages[0].text, "Real conversation");
}
