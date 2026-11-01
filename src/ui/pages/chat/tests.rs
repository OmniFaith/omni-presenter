use super::dialogs::dialogs;
use super::*;
use egui_phosphor::regular::{MAGNIFYING_GLASS, MICROPHONE, PAPER_PLANE_RIGHT};

#[test]
fn channels_support_additive_selection_and_a_batch_context_menu() {
    let mut state = ControlPanelState::default();
    state.settings.language = crate::core::i18n::Language::Portuguese;
    let service = ChatService::new(
        crate::services::db::AppDb::open_in_memory().unwrap(),
        &state.settings.operators,
    )
    .unwrap();
    let profile = &state.settings.active_operator;
    state.chat.token = service
        .login(profile, &service.access_code(profile).unwrap())
        .unwrap();
    let alpha = service
        .create_channel(&state.chat.token, "Alpha", "general", "")
        .unwrap();
    let beta = service
        .create_channel(&state.chat.token, "Beta", "general", "")
        .unwrap();
    state.chat.cached_snapshot = Some(
        service
            .snapshot(&state.chat.token, None, None, "", None)
            .unwrap(),
    );
    state.chat.service = Some(service);
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            Vec2::new(280.0, 700.0),
        )),
        ..Default::default()
    };
    let render = |state: &mut ControlPanelState, input| {
        ctx.run_ui(input, |ui| render_chat_channels(ui, state))
    };
    for _ in 0..2 {
        let mut output = render(&mut state, input.clone());
        output.textures_delta.clear();
    }
    let mut output = render(&mut state, input.clone());
    let position = |name: &str| {
        output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text == name => {
                    Some(text.pos + text.galley.rect.center().to_vec2())
                }
                _ => None,
            })
            .unwrap()
    };
    let alpha_position = position("#  Alpha");
    let beta_position = position("#  Beta");
    output.textures_delta.clear();
    let click = |state: &mut ControlPanelState, point, button, modifiers| {
        let mut output = None;
        for pressed in [true, false] {
            let mut event = input.clone();
            event.events = vec![
                egui::Event::ModifiersChanged(modifiers),
                egui::Event::PointerMoved(point),
                egui::Event::PointerButton {
                    pos: point,
                    button,
                    pressed,
                    modifiers,
                },
            ];
            let mut frame = render(state, event);
            frame.textures_delta.clear();
            output = Some(frame);
        }
        output.unwrap()
    };
    click(
        &mut state,
        alpha_position,
        egui::PointerButton::Primary,
        egui::Modifiers::NONE,
    );
    click(
        &mut state,
        beta_position,
        egui::PointerButton::Primary,
        egui::Modifiers::CTRL,
    );
    assert_eq!(
        state.chat.channel_selection.selected,
        [alpha.id.clone(), beta.id.clone()].into_iter().collect()
    );
    assert_eq!(state.chat.channel, alpha.id);
    let mut settled = render(&mut state, input.clone());
    settled.textures_delta.clear();
    let beta_position = settled
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == "#  Beta" => {
                Some(text.pos + text.galley.rect.center().to_vec2())
            }
            _ => None,
        })
        .unwrap();
    click(
        &mut state,
        beta_position,
        egui::PointerButton::Secondary,
        egui::Modifiers::NONE,
    );
    let mut opened = input.clone();
    opened.events = vec![
        egui::Event::ModifiersChanged(egui::Modifiers::NONE),
        egui::Event::PointerMoved(beta_position),
    ];
    let output = render(&mut state, opened);
    let texts: Vec<_> = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        texts.contains(&"Excluir canais"),
        "rendered texts: {texts:?}"
    );
}

#[test]
fn general_and_channel_navigation_use_the_selected_language() {
    for (language, heading) in [
        (crate::core::i18n::Language::English, "Channels"),
        (crate::core::i18n::Language::Spanish, "Canales"),
    ] {
        let mut state = ControlPanelState::default();
        state.settings.language = language;
        let service = ChatService::new(
            crate::services::db::AppDb::open_in_memory().unwrap(),
            &state.settings.operators,
        )
        .unwrap();
        let profile = &state.settings.active_operator;
        let token = service
            .login(profile, &service.access_code(profile).unwrap())
            .unwrap();
        state.chat.cached_snapshot = Some(service.snapshot(&token, None, None, "", None).unwrap());
        let ctx = egui::Context::default();
        crate::ui::i18n::install(&ctx, language);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            render_chat_channels(ui, &mut state)
        });
        let texts: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
                _ => None,
            })
            .collect();
        assert!(texts.contains(&"#  General"));
        assert!(texts.contains(&heading));
        assert!(!texts.contains(&"Canais"));
        output.textures_delta.clear();
    }
}

fn render_chat_page_test(ui: &mut egui::Ui, state: &mut ControlPanelState) {
    let mut deck = crate::core::slides::SlideDeck::default();
    let mut open = false;
    let mut rects = Vec::new();
    render_chat_page(ui, state, &mut deck, &mut open, &mut rects, None);
}

#[test]
fn search_is_hidden_until_its_toolbar_button_is_clicked() {
    let mut state = ControlPanelState::default();
    state.settings.language = crate::core::i18n::Language::Portuguese;
    let service = ChatService::new(
        crate::services::db::AppDb::open_in_memory().unwrap(),
        &state.settings.operators,
    )
    .unwrap();
    state.chat.token = service
        .login(
            &state.settings.active_operator,
            &service
                .access_code(&state.settings.active_operator)
                .unwrap(),
        )
        .unwrap();
    state.chat.profile_id = state.settings.active_operator.clone();
    state.chat.service = Some(service);
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            Vec2::new(1280.0, 800.0),
        )),
        ..Default::default()
    };
    for _ in 0..2 {
        let mut output = ctx.run_ui(input.clone(), |ui| render_chat_page_test(ui, &mut state));
        output.textures_delta.clear();
    }
    let mut output = ctx.run_ui(input.clone(), |ui| render_chat_page_test(ui, &mut state));
    output.textures_delta.clear();
    assert!(!output.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t)if t.galley.job.text.contains("Pesquisar mensagens…"))));
    let point = output
        .shapes
        .iter()
        .find_map(|s| {
            if let egui::Shape::Text(t) = &s.shape {
                (t.galley.job.text == MAGNIFYING_GLASS)
                    .then_some(t.pos + t.galley.rect.center().to_vec2())
            } else {
                None
            }
        })
        .unwrap();
    for pressed in [true, false] {
        let mut click = input.clone();
        click.events = vec![
            egui::Event::PointerMoved(point),
            egui::Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            },
        ];
        let mut output = ctx.run_ui(click, |ui| render_chat_page_test(ui, &mut state));
        output.textures_delta.clear();
    }
    assert!(state.chat.search_open);
    let mut output = ctx.run_ui(input, |ui| render_chat_page_test(ui, &mut state));
    output.textures_delta.clear();
    assert!(output.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t)if t.galley.job.text.contains("Pesquisar mensagens…"))));
}

#[test]
fn channel_dialog_uses_app_modal_and_translated_tool_choices() {
    let mut state = ControlPanelState::default();
    state.settings.language = crate::core::i18n::Language::Portuguese;
    let service = ChatService::new(
        crate::services::db::AppDb::open_in_memory().unwrap(),
        &state.settings.operators,
    )
    .unwrap();
    state.chat.show_channels = true;
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                Vec2::new(1200.0, 800.0),
            )),
            ..Default::default()
        },
        |ui| dialogs(ui, &mut state, &service),
    );
    output.textures_delta.clear();
    for _ in 0..2 {
        output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(1200.0, 800.0),
                )),
                ..Default::default()
            },
            |ui| dialogs(ui, &mut state, &service),
        );
        output.textures_delta.clear();
    }
    let texts: Vec<_> = output
        .shapes
        .iter()
        .filter_map(|s| {
            if let egui::Shape::Text(t) = &s.shape {
                Some(t.galley.job.text.clone())
            } else {
                None
            }
        })
        .collect();
    assert!(texts.iter().any(|t| t == "Criar canal"));
    assert!(texts.iter().any(|t| t.contains("Conversa geral")));
    assert!(!texts.iter().any(|t| t == "general"));
}

#[test]
fn shared_cards_stay_compact_in_wide_chat_and_voice_messages_have_a_single_listening_control() {
    let mut state = ControlPanelState::default();
    let service = ChatService::new(
        crate::services::db::AppDb::open_in_memory().unwrap(),
        &state.settings.operators,
    )
    .unwrap();
    let profile = state.settings.active_operator.clone();
    state.chat.token = service
        .login(&profile, &service.access_code(&profile).unwrap())
        .unwrap();
    state.chat.profile_id = profile;
    state.chat.service = Some(service.clone());
    state.chat.channel = "general".into();
    service
        .send(
            &state.chat.token,
            SendMessage {
                channel_id: Some("general".into()),
                kind: "bible".into(),
                reference: "João 3:16".into(),
                ..Default::default()
            },
        )
        .unwrap();
    let file = service
        .upload(
            &state.chat.token,
            "gravacao.wav",
            "audio/wav",
            &crate::services::chat::recording::wav(&[0; 8000], 8000, 1),
        )
        .unwrap();
    service
        .send(
            &state.chat.token,
            SendMessage {
                channel_id: Some("general".into()),
                kind: "audio".into(),
                file_id: Some(file.id),
                ..Default::default()
            },
        )
        .unwrap();
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            Vec2::new(2000.0, 1000.0),
        )),
        ..Default::default()
    };
    let mut output = ctx.run_ui(input, |ui| render_chat_page_test(ui, &mut state));
    output.textures_delta.clear();
    let cards: Vec<_> = output
        .shapes
        .iter()
        .filter_map(|shape| {
            if let egui::Shape::Rect(rect) = &shape.shape {
                (rect.fill == state.theme.accent_soft
                    && rect.stroke.width == 1.0
                    && rect.rect.width() > 250.0)
                    .then_some(rect.rect)
            } else {
                None
            }
        })
        .collect();
    assert!(!cards.is_empty());
    assert!(
        cards.iter().all(|rect| rect.width() <= 421.0),
        "message cards must not stretch across the window: {cards:?}"
    );
    let labels: Vec<_> = output
        .shapes
        .iter()
        .filter_map(|shape| {
            if let egui::Shape::Text(text) = &shape.shape {
                Some(text.galley.job.text.clone())
            } else {
                None
            }
        })
        .collect();
    let voice_label = state.settings.language.text("chat-voice-message");
    let open_player_label = state.settings.language.text("chat-open-in-player");
    assert!(labels.iter().any(|s| s == voice_label));
    assert!(
        !labels
            .iter()
            .any(|s| s.contains("Ouvir áudio") || s.contains("▶ Reproduzir"))
    );
    assert!(labels.iter().any(|s| s.contains(open_player_label)));
    output.textures_delta.clear();
}

#[test]
fn network_chat_renders_real_messages_and_switches_microphone_to_send() {
    let mut state = ControlPanelState::default();
    let service = ChatService::new(
        crate::services::db::AppDb::open_in_memory().unwrap(),
        &state.settings.operators,
    )
    .unwrap();
    let profile = state.settings.active_operator.clone();
    state.chat.token = service
        .login(&profile, &service.access_code(&profile).unwrap())
        .unwrap();
    state.chat.profile_id = profile;
    state.chat.service = Some(service.clone());
    state.chat.channel = "general".into();
    service
        .send(
            &state.chat.token,
            SendMessage {
                channel_id: Some("general".into()),
                text: "Olá da rede".into(),
                ..Default::default()
            },
        )
        .unwrap();
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            Vec2::new(1200.0, 800.0),
        )),
        ..Default::default()
    };
    let mut output = ctx.run_ui(input.clone(), |ui| render_chat_page_test(ui, &mut state));
    let labels = |output: &egui::FullOutput| {
        output
            .shapes
            .iter()
            .filter_map(|s| {
                if let egui::Shape::Text(t) = &s.shape {
                    Some(t.galley.job.text.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };
    let empty = labels(&output);
    output.textures_delta.clear();
    assert!(empty.iter().any(|s| s == MICROPHONE));
    assert!(!empty.iter().any(|s| s == PAPER_PLANE_RIGHT));
    assert!(empty.iter().any(|s| s == "Olá da rede"));
    state.chat.input = "Mensagem pronta".into();
    let mut output = ctx.run_ui(input, |ui| render_chat_page_test(ui, &mut state));
    let typed = labels(&output);
    output.textures_delta.clear();
    assert!(!typed.iter().any(|s| s == MICROPHONE));
    assert!(typed.iter().any(|s| s == PAPER_PLANE_RIGHT));
}

#[test]
fn chat_docking_and_containers_swap_position() {
    use crate::core::layout::DropZone;
    let mut state = ControlPanelState::default();
    assert!(state.chat_layout.root.contains(ContainerKind::ChatChannels));
    assert!(state.chat_layout.root.contains(ContainerKind::ChatMessages));

    // Test swapping positions
    state.chat_layout.dock(
        ContainerKind::ChatChannels,
        ContainerKind::ChatMessages,
        DropZone::Center,
    );
    if let crate::core::layout::LayoutNode::Split { first, second, .. } = &state.chat_layout.root {
        assert_eq!(
            **first,
            crate::core::layout::LayoutNode::Leaf(ContainerKind::ChatMessages)
        );
        assert_eq!(
            **second,
            crate::core::layout::LayoutNode::Leaf(ContainerKind::ChatChannels)
        );
    } else {
        panic!("expected split node");
    }

    // Test docking vertically
    state.chat_layout.dock(
        ContainerKind::ChatChannels,
        ContainerKind::ChatMessages,
        DropZone::Bottom,
    );
    assert!(state.chat_layout.root.contains(ContainerKind::ChatChannels));
    assert!(state.chat_layout.root.contains(ContainerKind::ChatMessages));
}

#[test]
fn private_profiles_appear_even_when_the_selected_operator_is_the_only_online_one() {
    let mut state = ControlPanelState::default();
    let second = crate::core::preferences::OperatorProfile { id: "operator_second".into(), name: "Mesa de som".into(), ..Default::default() };
    state.settings.operators.push(second.clone());
    let service = ChatService::new(crate::services::db::AppDb::open_in_memory().unwrap(), &state.settings.operators).unwrap();
    let profile = &state.settings.active_operator;
    let token = service.login(profile, &service.access_code(profile).unwrap()).unwrap();
    state.chat.cached_snapshot = Some(service.snapshot(&token, Some("general"), None, "", None).unwrap());
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(280.0, 700.0))), ..Default::default() }, |ui| render_chat_channels(ui, &mut state));
    let texts: Vec<_> = output.shapes.iter().filter_map(|shape| match &shape.shape { egui::Shape::Text(text) => Some(text.galley.job.text.as_str()), _ => None }).collect();
    assert!(texts.contains(&"Mesa de som"), "rendered texts: {texts:?}");
    output.textures_delta.clear();
}
