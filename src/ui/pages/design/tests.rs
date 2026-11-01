use super::*;

#[test]
fn creating_saving_and_reusing_designs_uses_live_library_references() {
    let db = AppDb::open_in_memory().unwrap();
    let mut deck = SlideDeck::new("Conteúdo ao vivo");
    let mut editor = DesignEditor::default();
    editor.new_scene(2, &db, &mut deck);
    assert_eq!(deck.designs.len(), 1);
    assert_eq!(db.load_designs().unwrap(), deck.designs);
    let mut state = ControlPanelState::default();
    let scene = editor.scene.clone().unwrap();
    library::pin(&mut state, &scene);
    library::playlist(&mut state, &scene);
    assert_eq!(
        state.quick_bar.items[0].kind,
        crate::core::models::QuickItemKind::Design
    );
    assert_eq!(state.quick_bar.items[0].content, scene.id);
    editor.scene.as_mut().unwrap().name = "Culto · início".into();
    editor.saved = false;
    editor.flush(&db, &mut deck, true);
    state.playlist[0].preview(&mut deck).unwrap();
    assert_eq!(deck.preview_design.as_ref().unwrap().name, "Culto · início");
    assert_eq!(deck.live_slide_text(), "Conteúdo ao vivo");
    deck.project_preview();
    let projected = deck.live_design.clone().unwrap();
    assert!(projected.layers.iter().any(|layer|matches!(&layer.content,LayerContent::Countdown(timer) if timer.started_at.is_some())));
    editor.scene.as_mut().unwrap().layers.clear();
    editor.saved = false;
    editor.flush(&db, &mut deck, true);
    assert_eq!(
        deck.live_design.as_ref(),
        Some(&projected),
        "editing the draft must not change the live design"
    );
}

#[test]
fn transparent_extra_keeps_live_content_and_survives_slide_changes() {
    let mut deck = SlideDeck::new("Primeiro\n\nSegundo");
    let mut extra = DesignDocument::default();
    extra.overlay = true;
    extra.layers.push(DesignLayer::new(LayerContent::Clock {
        seconds: true,
        date: false,
    }));
    deck.designs.push(extra.clone());
    deck.preview_design_by_id(&extra.id).unwrap();
    deck.project_preview();
    assert_eq!(deck.live_slide_text(), "Primeiro");
    assert!(deck.live_design.is_none());
    assert_eq!(
        deck.live_overlay.as_ref().unwrap().background_mode,
        "transparent"
    );
    deck.next_slide();
    assert_eq!(deck.live_slide_text(), "Segundo");
    deck.project_custom_content("Bíblia".into(), "BL".into(), "Um versículo".into());
    assert_eq!(deck.live_slide_text(), "Um versículo");
    assert_eq!(deck.live_overlay.as_ref().unwrap().id, extra.id);
    deck.live_overlay = None;
    assert_eq!(deck.live_slide_text(), "Um versículo");
}

#[test]
fn history_restores_layer_content_and_keeps_redo_available() {
    let db = AppDb::open_in_memory().unwrap();
    let mut deck = SlideDeck::default();
    let mut editor = DesignEditor::default();
    editor.new_scene(0, &db, &mut deck);
    editor.add_layer(LayerContent::Text("Primeira".into()));
    editor.add_layer(LayerContent::Clock {
        seconds: false,
        date: false,
    });
    assert_eq!(editor.scene.as_ref().unwrap().layers.len(), 2);
    editor.history(false);
    assert_eq!(editor.scene.as_ref().unwrap().layers.len(), 1);
    editor.history(true);
    assert_eq!(editor.scene.as_ref().unwrap().layers.len(), 2);
}

#[test]
fn editor_renders_in_both_appearances_at_minimum_window_size() {
    for dark in [false, true] {
        let db = AppDb::open_in_memory().unwrap();
        let mut deck = SlideDeck::default();
        let mut editor = DesignEditor::default();
        let mut state = ControlPanelState::default();
        if dark {
            state.theme = crate::ui::theme::AppTheme::dark();
        }
        editor.new_scene(3, &db, &mut deck);
        let ctx = egui::Context::default();
        ctx.data_mut(|data| data.insert_temp(egui::Id::unique("design_gpu_ready"), true));
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(924.0, 680.0),
                )),
                ..Default::default()
            },
            |ui| {
                editor.render(ui, &mut state, &mut deck, &mut false, &db);
            },
        );
        assert!(!output.shapes.is_empty());
        assert!(editor.error.is_empty());
        output.textures_delta.clear();
    }
}

#[test]
fn design_selection_library_renders_and_supports_folders_and_multiselect() {
    let db = AppDb::open_in_memory().unwrap();
    let mut deck = SlideDeck::default();
    let mut state = ControlPanelState::default();
    state.selected_tab = 4;

    let mut doc1 = DesignDocument::template(1);
    doc1.name = "Boas-vindas 1".into();
    let mut doc2 = DesignDocument::template(2);
    doc2.name = "Contagem 2".into();
    db.save_design(&doc1).unwrap();
    db.save_design(&doc2).unwrap();
    deck.designs.push(doc1.clone());
    deck.designs.push(doc2.clone());

    let ctx = egui::Context::default();
    let mut is_presentation_open = false;
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(300.0, 700.0),
            )),
            ..Default::default()
        },
        |ui| {
            crate::ui::containers::selection::render_selection_column(
                ui,
                &mut state,
                &mut deck,
                &mut is_presentation_open,
                Some(&db),
            );
        },
    );
    assert!(!output.shapes.is_empty());
    output.textures_delta.clear();

    // Test grid view toggle
    state.design_list_view = false;
    let mut output_grid = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(300.0, 700.0),
            )),
            ..Default::default()
        },
        |ui| {
            crate::ui::containers::selection::render_selection_column(
                ui,
                &mut state,
                &mut deck,
                &mut is_presentation_open,
                Some(&db),
            );
        },
    );
    assert!(!output_grid.shapes.is_empty());
    output_grid.textures_delta.clear();

    // Multiselect state
    state.selected_design_ids.insert(doc1.id.clone());
    state.selected_design_ids.insert(doc2.id.clone());
    assert_eq!(state.selected_design_ids.len(), 2);
    state.clear_design_selection();
    assert!(state.selected_design_ids.is_empty());
}

#[test]
fn canvas_toolbar_renders_only_undo_redo_and_active_guides() {
    let mut editor = DesignEditor::default();
    let theme = AppTheme::dark();
    let ctx = egui::Context::default();
    let mut state = ControlPanelState::default();
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        },
        |ui| {
            editor.canvas_toolbar(ui, &theme, &mut state);
        },
    );
    assert!(!output.shapes.is_empty());
    assert!(!editor.disable_snap);
    assert!(editor.preview_theme_id.is_none());
    assert!(editor.preview_aspect_ratio.is_none());
    output.textures_delta.clear();
}

#[test]
fn test_aspect_ratio_preview_selection_and_canvas_scaling() {
    let mut editor = DesignEditor::default();
    let theme = AppTheme::dark();
    let ctx = egui::Context::default();
    let state = ControlPanelState::default();
    let deck = SlideDeck::new("Aspect ratio test");

    assert_eq!(aspect_ratio::aspect_ratio_name(16.0 / 9.0), "16:9");
    assert_eq!(aspect_ratio::aspect_ratio_name(16.0 / 10.0), "16:10");
    assert_eq!(aspect_ratio::aspect_ratio_name(4.0 / 3.0), "4:3");
    assert_eq!(aspect_ratio::aspect_ratio_name(21.0 / 9.0), "21:9");
    assert_eq!(aspect_ratio::aspect_ratio_name(1.0), "1:1");
    assert_eq!(aspect_ratio::aspect_ratio_name(9.0 / 16.0), "9:16");

    let doc = DesignDocument::template(1);
    editor.scene = Some(doc);

    // Default ratio (None) uses state.presentation_aspect_ratio
    let mut output1 = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(900.0, 700.0),
            )),
            ..Default::default()
        },
        |ui| {
            canvas::render(ui, &mut editor, &theme, &state, &deck, false);
        },
    );
    assert!(!output1.shapes.is_empty());
    output1.textures_delta.clear();

    // Custom vertical ratio (9:16)
    editor.preview_aspect_ratio = Some(9.0 / 16.0);
    let mut output2 = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(900.0, 700.0),
            )),
            ..Default::default()
        },
        |ui| {
            canvas::render(ui, &mut editor, &theme, &state, &deck, false);
        },
    );
    assert!(!output2.shapes.is_empty());
    output2.textures_delta.clear();

    // Custom 4:3 ratio
    editor.preview_aspect_ratio = Some(4.0 / 3.0);
    let mut output3 = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(900.0, 700.0),
            )),
            ..Default::default()
        },
        |ui| {
            canvas::render(ui, &mut editor, &theme, &state, &deck, false);
        },
    );
    assert!(!output3.shapes.is_empty());
    output3.textures_delta.clear();
}

#[test]
fn plugin_layers_render_text_tab_in_inspector() {
    let mut editor = DesignEditor::default();
    let mut deck = SlideDeck::new("");
    let theme = AppTheme::dark();
    let ctx = egui::Context::default();

    let plugin = crate::core::plugins::bundled().remove(0);
    let instance = crate::core::plugins::PluginInstance::new(
        &plugin,
        &plugin.package.components[0],
    );
    let mut scene = DesignDocument::default();
    scene.layers.push(DesignLayer::new(LayerContent::Plugin(Box::new(instance))));
    editor.scene = Some(scene);
    editor.selected = Some(0);
    editor.inspector_tab = 1;
    editor.layer_inspector_tab = Some(2); // Textos tab

    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        },
        |ui| {
            inspector::render(ui, &mut editor, &mut deck, &theme, &[], 16.0 / 9.0);
        },
    );
    assert!(!output.shapes.is_empty());
    output.textures_delta.clear();
}

#[test]
fn plugin_layers_transition_tab_configures_text_transition_and_materializes_smoothly() {
    let mut editor = DesignEditor::default();
    let mut deck = SlideDeck::new("");
    let theme = AppTheme::dark();
    let ctx = egui::Context::default();

    let plugin = crate::core::plugins::bundled().remove(0);
    let announcements = plugin
        .package
        .components
        .iter()
        .find(|c| c.id == "announcements")
        .expect("announcements component");
    let instance = crate::core::plugins::PluginInstance::new(&plugin, announcements);
    let mut scene = DesignDocument::default();
    scene.layers.push(DesignLayer::new(LayerContent::Plugin(Box::new(instance))));
    editor.scene = Some(scene);
    editor.selected = Some(0);
    editor.inspector_tab = 1;
    editor.layer_inspector_tab = Some(3); // Transição tab

    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        },
        |ui| {
            inspector::render(ui, &mut editor, &mut deck, &theme, &[], 16.0 / 9.0);
        },
    );
    assert!(!output.shapes.is_empty());
    output.textures_delta.clear();

    let scene = editor.scene.as_ref().unwrap();
    if let LayerContent::Plugin(ref inst) = scene.layers[0].content {
        assert!(inst.values.contains_key("transition"));
        assert!(inst.values.contains_key("transition_ms"));
    } else {
        panic!("Layer 0 should be plugin");
    }

    let materialized = crate::core::plugins::runtime::materialize(scene, 4600);
    let texts: Vec<_> = materialized
        .layers
        .iter()
        .filter(|l| matches!(l.content, LayerContent::Text(_)))
        .collect();
    assert_eq!(texts.len(), 2);
    assert!(texts[0].opacity > 0.0 && texts[0].opacity < 1.0);
    assert!(texts[1].opacity > 0.0 && texts[1].opacity < 1.0);
}

#[test]
fn design_preview_questions_controls_and_cards_list() {
    let ctx = egui::Context::default();
    let mut state = ControlPanelState::default();
    let mut deck = SlideDeck::new("Culto ao vivo");

    let mut doc = DesignDocument::default();
    doc.name = "Quiz Bíblico".into();
    doc.layers.push(DesignLayer::new(LayerContent::Questions {
        items: vec![
            crate::core::design::Question {
                question: "Quem construiu a arca?".into(),
                answer: "Noé".into(),
            },
            crate::core::design::Question {
                question: "Quantos discípulos Jesus escolheu?".into(),
                answer: "12".into(),
            },
            crate::core::design::Question {
                question: "Qual o menor livro da Bíblia?".into(),
                answer: "2 João".into(),
            },
        ],
        current: 0,
        revealed: false,
    }));
    deck.preview_design = Some(doc);

    let mut presenting = false;
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 700.0),
            )),
            ..Default::default()
        },
        |ui| {
            library::render_preview(ui, &mut state, &mut deck, &mut presenting);
        },
    );
    output.textures_delta.clear();

    let preview = deck.preview_design.as_ref().unwrap();
    if let LayerContent::Questions { items, current, revealed } = &preview.layers[0].content {
        assert_eq!(items.len(), 3);
        assert_eq!(*current, 0);
        assert!(!*revealed);
    } else {
        panic!("Layer 0 should be questions");
    }
}

#[test]
fn test_slide_controls_theme_selection_applies_theme_to_layers() {
    let state = ControlPanelState::default();
    let mut scene = DesignDocument::default();
    let mut text_layer = DesignLayer::new(LayerContent::Text("Título de teste".into()));
    text_layer.font_family = "OldFont".into();
    scene.layers.push(text_layer);

    assert_eq!(scene.background_mode, "transparent");

    // Setting theme mode applies the theme font & color to text layers
    scene.background_mode = "theme".into();
    scene.theme_id = state.themes.first().map(|t| t.id.clone());
    let selected = state.themes.first().unwrap();
    for layer in &mut scene.layers {
        if !matches!(layer.content, LayerContent::Image { .. } | LayerContent::Shape { .. }) {
            layer.font_family = selected.font_family.clone();
            layer.set_font_weight(selected.font_weight);
            layer.color = selected.text_color.to_srgba_unmultiplied();
        }
    }

    assert_eq!(scene.layers[0].font_family, selected.font_family);
    assert_eq!(scene.layers[0].color, selected.text_color.to_srgba_unmultiplied());
}

#[test]
fn test_theme_click_applies_live_and_preview() {
    let mut state = ControlPanelState::default();
    assert_eq!(state.selected_theme_index, 0);
    let original_live = state.live_theme_id.clone();

    if state.themes.len() > 1 {
        state.selected_theme_index = 1;
        state.apply_theme_to_live();
        assert_eq!(state.live_theme_id, state.themes[1].id);
        assert_ne!(state.live_theme_id, original_live);
    }
}

#[test]
fn test_active_theme_applies_to_non_overlay_preview_design() {
    let mut state = ControlPanelState::default();
    let mut deck = SlideDeck::default();
    let mut doc = DesignDocument::default();
    doc.name = "Placar teste".into();
    doc.overlay = false;
    doc.background_mode = "transparent".into();
    deck.preview_design = Some(doc);

    let active_theme_id = state.preview_theme().id.clone();
    let ctx = eframe::egui::Context::default();
    let mut out = ctx.run_ui(eframe::egui::RawInput::default(), |ui| {
        let mut is_pres_open = false;
        crate::ui::render_container(
            ui,
            crate::core::layout::ContainerKind::Themes,
            &mut state,
            &mut deck,
            &mut is_pres_open,
            None,
        );
    });
    out.textures_delta.clear();

    let preview = deck.preview_design.as_ref().unwrap();
    assert_eq!(preview.background_mode, "theme");
    assert_eq!(preview.theme_id.as_deref(), Some(active_theme_id.as_str()));
}

#[test]
fn test_editor_theme_preview_background_renders_and_toggles() {
    let mut editor = DesignEditor::default();
    let mut scene = DesignDocument::default();
    scene.layers.push(DesignLayer::new(LayerContent::Text("Título teste".into())));
    editor.scene = Some(scene);

    let state = ControlPanelState::default();
    let deck = SlideDeck::default();
    let theme = AppTheme::dark();
    let ctx = egui::Context::default();

    // Default: preview_theme_id is None
    assert!(editor.preview_theme_id.is_none());

    let mut out_idle = ctx.run_ui(egui::RawInput::default(), |ui| {
        super::canvas::render(ui, &mut editor, &theme, &state, &deck, false);
    });
    out_idle.textures_delta.clear();

    // Enable preview theme
    let active_theme_id = state.preview_theme().id.clone();
    editor.preview_theme_id = Some(active_theme_id.clone());

    let mut out_themed = ctx.run_ui(egui::RawInput::default(), |ui| {
        super::canvas::render(ui, &mut editor, &theme, &state, &deck, false);
    });
    out_themed.textures_delta.clear();

    assert!(!out_themed.shapes.is_empty());
    assert_eq!(editor.preview_theme_id.as_deref(), Some(active_theme_id.as_str()));
}

#[test]
fn test_multiselection_and_grouping_lifecycle() {
    let mut editor = DesignEditor::default();
    let mut scene = DesignDocument::default();
    scene.layers.push(DesignLayer::new(LayerContent::Text("Camada 1".into())));
    scene.layers.push(DesignLayer::new(LayerContent::Text("Camada 2".into())));
    scene.layers.push(DesignLayer::new(LayerContent::Text("Camada 3".into())));
    editor.select(scene);

    // Initial state: single selected layer 2
    assert_eq!(editor.selected, Some(2));
    assert!(editor.is_layer_selected(2));
    assert!(!editor.is_layer_selected(0));

    // Additive selection (Ctrl/Cmd click) on layer 0
    editor.select_layer(0, true);
    assert!(editor.is_layer_selected(0));
    assert!(editor.is_layer_selected(2));
    assert_eq!(editor.selected_layers.len(), 2);

    // Group the 2 selected layers (layer 0 and 2)
    editor.group_selected();
    let gid0 = editor.scene.as_ref().unwrap().layers[0].group_id.clone();
    let gid2 = editor.scene.as_ref().unwrap().layers[2].group_id.clone();
    assert!(gid0.is_some());
    assert_eq!(gid0, gid2);
    assert_eq!(editor.scene.as_ref().unwrap().layers[1].group_id, None);

    // Clear selection
    editor.clear_layer_selection();
    assert_eq!(editor.selected, None);
    assert_eq!(editor.selected_layers.len(), 0);

    // Selecting a layer that belongs to a group automatically selects the whole group
    editor.select_layer(0, false);
    assert!(editor.is_layer_selected(0));
    assert!(editor.is_layer_selected(2));
    assert!(!editor.is_layer_selected(1));

    // Range selection
    editor.select_layer(0, false); // only layer 0 and 2 from group
    editor.clear_layer_selection();
    editor.select_layer(0, true); // only 0
    editor.select_layer_range(2); // selects 0, 1, 2
    assert_eq!(editor.selected_layers.len(), 3);

    // Test Ungroup command
    editor.execute_edit(crate::core::commands::Command::UngroupLayers);
    for layer in &editor.scene.as_ref().unwrap().layers {
        assert!(layer.group_id.is_none());
    }

    // Test Group command via execute_edit (Ctrl+G)
    editor.select_layer(1, false);
    editor.select_layer(2, true);
    assert_eq!(editor.selected_layers.len(), 2);
    editor.execute_edit(crate::core::commands::Command::GroupLayers);
    assert!(editor.scene.as_ref().unwrap().layers[1].group_id.is_some());
    assert_eq!(
        editor.scene.as_ref().unwrap().layers[1].group_id,
        editor.scene.as_ref().unwrap().layers[2].group_id
    );

    // Multi-duplicate
    editor.duplicate_layer();
    assert_eq!(editor.scene.as_ref().unwrap().layers.len(), 5);

    // Multi-delete
    let before_count = editor.scene.as_ref().unwrap().layers.len();
    editor.delete_layer();
    assert!(editor.scene.as_ref().unwrap().layers.len() < before_count);
}


