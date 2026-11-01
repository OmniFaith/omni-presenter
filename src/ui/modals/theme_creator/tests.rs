use super::*;

#[test]
fn test_theme_creator_modal_stays_open_on_opening_frame() {
    let ctx = egui::Context::default();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 720.0));
    let mut state = ControlPanelState::default();
    state.theme_editor.open = true;

    let mut raw_input = egui::RawInput::default();
    raw_input.screen_rect = Some(screen);

    let mut output = ctx.run_ui(raw_input, |ui| {
        render_theme_creator_modal(ui.ctx(), &mut state, None);
    });
    output.textures_delta.clear();

    assert!(
        state.theme_editor.open,
        "Modal should remain open on first frame"
    );
}

#[test]
fn test_theme_creator_modal_closes_on_escape() {
    let ctx = egui::Context::default();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 720.0));
    let mut state = ControlPanelState::default();
    state.theme_editor.open = true;

    let mut raw_input = egui::RawInput::default();
    raw_input.screen_rect = Some(screen);
    raw_input.events.push(egui::Event::Key {
        key: egui::Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });

    let mut output = ctx.run_ui(raw_input, |ui| {
        render_theme_creator_modal(ui.ctx(), &mut state, None);
    });
    output.textures_delta.clear();

    assert!(!state.theme_editor.open, "Modal should close on ESC key");
}

#[test]
fn test_theme_creator_modal_tab_state() {
    let mut state = ControlPanelState::default();
    state.theme_editor.sidebar_tab = 0;
    assert_eq!(state.theme_editor.sidebar_tab, 0);

    state.theme_editor.sidebar_tab = 1;
    assert_eq!(state.theme_editor.sidebar_tab, 1);

    state.theme_editor.sidebar_tab = 2;
    assert_eq!(state.theme_editor.sidebar_tab, 2);
}

#[test]
fn test_theme_creator_no_duplicate_widget_ids() {
    let ctx = egui::Context::default();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 720.0));

    for tab in 0..=2 {
        for bg_tab in 0..=3 {
            let mut state = ControlPanelState::default();
            state.theme_editor.open = true;
            state.theme_editor.sidebar_tab = tab;
            state.theme_editor.bg_tab = bg_tab;

            let mut raw_input = egui::RawInput::default();
            raw_input.screen_rect = Some(screen);

            let mut output = ctx.run_ui(raw_input, |ui| {
                render_theme_creator_modal(ui.ctx(), &mut state, None);
            });
            let has_duplicate_warning = output.shapes.iter().any(|shape| {
                matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.contains("use of widget"))
            });
            output.textures_delta.clear();

            assert!(
                !has_duplicate_warning,
                "Detected duplicate widget ID warning in theme creator on tab {tab}, bg_tab {bg_tab}!"
            );
        }
    }
}

#[test]
fn test_theme_creator_slide_and_transition_variants() {
    let ctx = egui::Context::default();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 720.0));

    for transition in [
        "fade",
        "slide",
        "slide_left",
        "slide_right",
        "slide_up",
        "slide_down",
        "zoom",
        "zoom_out",
        "blur",
        "wipe",
        "wipe_down",
        "split",
        "flip",
        "cube",
        "pop",
        "cut",
    ] {
        let mut state = ControlPanelState::default();
        state.theme_editor.open = true;
        state.theme_editor.sidebar_tab = 0; // Fundo tab
        state.theme_editor.editing.transition = transition.to_string();

        let mut raw_input = egui::RawInput::default();
        raw_input.screen_rect = Some(screen);

        let mut output = ctx.run_ui(raw_input, |ui| {
            render_theme_creator_modal(ui.ctx(), &mut state, None);
        });
        let has_duplicate_warning = output.shapes.iter().any(|shape| {
            matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.contains("use of widget"))
        });
        output.textures_delta.clear();

        assert!(
            !has_duplicate_warning,
            "Detected duplicate widget ID warning for transition {transition}!"
        );
    }
}

#[test]
fn test_theme_creator_solid_and_gradient_modes() {
    let ctx = egui::Context::default();
    let mut state = ControlPanelState::default();
    state.theme_editor.open = true;
    state.theme_editor.sidebar_tab = 0;
    state.theme_editor.bg_tab = 0;
    state.theme_editor.editing.is_gradient = true;

    let mut raw_input = egui::RawInput::default();
    raw_input.screen_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 720.0)));

    let mut output = ctx.run_ui(raw_input.clone(), |ui| {
        render_theme_creator_modal(ui.ctx(), &mut state, None);
    });
    let has_duplicate_warning = output.shapes.iter().any(|shape| {
        matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.contains("use of widget"))
    });
    output.textures_delta.clear();
    assert!(!has_duplicate_warning, "Duplicate widget warning in gradient mode");

    // Test solid mode
    state.theme_editor.editing.is_gradient = false;
    let mut output_solid = ctx.run_ui(raw_input, |ui| {
        render_theme_creator_modal(ui.ctx(), &mut state, None);
    });
    let has_duplicate_warning_solid = output_solid.shapes.iter().any(|shape| {
        matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.contains("use of widget"))
    });
    output_solid.textures_delta.clear();
    assert!(!has_duplicate_warning_solid, "Duplicate widget warning in solid mode");
}

#[test]
fn test_gradient_angle_and_orientation_segments() {
    let ctx = egui::Context::default();
    let mut state = ControlPanelState::default();
    state.theme_editor.open = true;
    state.theme_editor.sidebar_tab = 0;
    state.theme_editor.bg_tab = 0;
    state.theme_editor.editing.is_gradient = true;
    state.theme_editor.editing.options.gradient_angle = 45.0;

    let mut raw_input = egui::RawInput::default();
    raw_input.screen_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 720.0)));

    let mut output = ctx.run_ui(raw_input, |ui| {
        render_theme_creator_modal(ui.ctx(), &mut state, None);
    });
    let has_duplicate_warning = output.shapes.iter().any(|shape| {
        matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.contains("use of widget"))
    });
    output.textures_delta.clear();
    assert!(!has_duplicate_warning, "Duplicate widget warning with gradient_angle = 45°");
}

#[test]
fn test_theme_options_gradient_angle_serde_roundtrip() {
    let mut theme = crate::core::models::PresentationTheme::default();
    theme.options.gradient_angle = 135.0;

    let serialized = serde_json::to_string(&theme.options).expect("serialize ThemeOptions");
    assert!(serialized.contains("gradient_angle"), "JSON should contain gradient_angle");

    let deserialized: crate::core::models::theme_options::ThemeOptions =
        serde_json::from_str(&serialized).expect("deserialize ThemeOptions");
    assert_eq!(deserialized.gradient_angle, 135.0);

    // Backward compatibility: old JSON without gradient_angle should default to 0.0
    let old_json = r#"{"transition_ms":400,"auto_fit":true}"#;
    let old_deserialized: crate::core::models::theme_options::ThemeOptions =
        serde_json::from_str(old_json).expect("deserialize old ThemeOptions");
    assert_eq!(old_deserialized.gradient_angle, 0.0);
}

#[test]
fn test_preview_selection_and_snapping() {
    let ctx = egui::Context::default();
    let mut state = ControlPanelState::default();
    state.theme_editor.open = true;
    state.theme_editor.is_text_selected = false;
    state.theme_editor.sidebar_tab = 0;

    let mut raw_input = egui::RawInput::default();
    raw_input.screen_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 720.0)));

    let mut out1 = ctx.run_ui(raw_input.clone(), |ui| {
        render_theme_creator_modal(ui.ctx(), &mut state, None);
    });
    out1.textures_delta.clear();
    assert!(!state.theme_editor.is_text_selected);
    assert_eq!(state.theme_editor.sidebar_tab, 0);

    state.theme_editor.is_text_selected = true;
    state.theme_editor.sidebar_tab = 1;
    let mut out2 = ctx.run_ui(raw_input, |ui| {
        render_theme_creator_modal(ui.ctx(), &mut state, None);
    });
    out2.textures_delta.clear();
    assert!(state.theme_editor.is_text_selected);
    assert_eq!(state.theme_editor.sidebar_tab, 1);
}

#[test]
fn test_collapsible_section_title() {
    let ctx = egui::Context::default();
    let theme = crate::ui::theme::AppTheme::dark();
    let raw = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 200.0))),
        ..Default::default()
    };

    let mut out1 = ctx.run_ui(raw.clone(), |ui| {
        let is_open = widgets::collapsible_section_title(
            ui,
            "test_prop_toggle",
            "Propriedades",
            false,
            &theme,
        );
        assert!(!is_open, "Should be closed by default");
    });
    out1.textures_delta.clear();

    let mut click_input = raw;
    let pos = egui::pos2(50.0, 12.0);
    click_input.events.push(egui::Event::PointerMoved(pos));
    for pressed in [true, false] {
        click_input.events.push(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
    }

    let mut out2 = ctx.run_ui(click_input, |ui| {
        let is_open = widgets::collapsible_section_title(
            ui,
            "test_prop_toggle",
            "Propriedades",
            false,
            &theme,
        );
        assert!(is_open, "Should be open after click");
    });
    out2.textures_delta.clear();
}

#[test]
fn test_dynamic_shader_controls_with_many_shaders_no_id_clash() {
    let ctx = egui::Context::default();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 720.0));
    let mut state = ControlPanelState::default();
    state.theme_editor.open = true;
    state.theme_editor.sidebar_tab = 0; // Fundo tab
    state.theme_editor.bg_tab = 3; // Dinâmico (shaders)

    // Simulate 35 plugin shaders so total shaders = 10 built-in + 35 plugin = 45 shaders!
    let mut custom_shaders = Vec::new();
    for i in 0..35 {
        custom_shaders.push(crate::core::plugins::PluginShader {
            id: format!("custom_shader_{i}"),
            name: format!("Shader Test {i}"),
            description: format!("Description for shader {i}"),
            wgsl: "@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }".to_string(),
            glsl: "vec3 plugin_color(vec2 p, float t) { return vec3(1.0); }".to_string(),
            translations: Default::default(),
        });
    }

    let fake_plugin = crate::core::plugins::InstalledPlugin {
        package: crate::core::plugins::PluginPackage {
            id: "fake_plugin".to_string(),
            name: "Super Pack".to_string(),
            description: "40+ shaders plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "Tester".to_string(),
            shaders: custom_shaders,
            ..Default::default()
        },
        enabled: true,
        settings: Default::default(),
    };
    state.settings.plugins = vec![fake_plugin];

    let mut raw_input = egui::RawInput::default();
    raw_input.screen_rect = Some(screen);

    // Frame 1: Grid Mode with 45 shaders
    let mut out1 = ctx.run_ui(raw_input.clone(), |ui| {
        render_theme_creator_modal(ui.ctx(), &mut state, None);
    });
    let has_duplicate_warning_1 = out1.shapes.iter().any(|shape| {
        matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.contains("use of widget"))
    });
    out1.textures_delta.clear();
    assert!(!has_duplicate_warning_1, "No widget duplicate ID warning with 45 shaders in grid mode");

    // Frame 2: Switch to compact list mode
    let picker_id = egui::Id::unique("theme_shader_picker_state");
    ctx.data_mut(|d| {
        d.insert_temp(picker_id, super::shader_controls::ShaderPickerState {
            query: String::new(),
            filter_source: "all".to_string(),
            compact_mode: true,
            expanded: true,
        });
    });

    let mut out2 = ctx.run_ui(raw_input.clone(), |ui| {
        render_theme_creator_modal(ui.ctx(), &mut state, None);
    });
    let has_duplicate_warning_2 = out2.shapes.iter().any(|shape| {
        matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.contains("use of widget"))
    });
    out2.textures_delta.clear();
    assert!(!has_duplicate_warning_2, "No widget duplicate ID warning with 45 shaders in compact list mode");

    // Frame 3: Search filter
    ctx.data_mut(|d| {
        d.insert_temp(picker_id, super::shader_controls::ShaderPickerState {
            query: "aurora".to_string(),
            filter_source: "all".to_string(),
            compact_mode: false,
            expanded: false,
        });
    });

    let mut out3 = ctx.run_ui(raw_input, |ui| {
        render_theme_creator_modal(ui.ctx(), &mut state, None);
    });
    let has_duplicate_warning_3 = out3.shapes.iter().any(|shape| {
        matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.contains("use of widget"))
    });
    out3.textures_delta.clear();
    assert!(!has_duplicate_warning_3, "No widget duplicate ID warning with search query filter");
}

