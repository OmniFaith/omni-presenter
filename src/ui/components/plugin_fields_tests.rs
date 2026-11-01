use super::*;

fn input(width: f32) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(width, 1600.0),
        )),
        ..Default::default()
    }
}

fn click(mut input: egui::RawInput, position: egui::Pos2) -> egui::RawInput {
    input.events.push(egui::Event::PointerMoved(position));
    for pressed in [true, false] {
        input.events.push(egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
    }
    input
}

fn position(output: &egui::FullOutput, label: &str) -> egui::Pos2 {
    output
        .shapes
        .iter()
        .find_map(|shape| {
            if let egui::Shape::Text(text) = &shape.shape
                && text.galley.job.text.contains(label)
            {
                Some(text.pos + text.galley.rect.center().to_vec2())
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("control not found: {label}"))
}

fn field_button(output: &egui::FullOutput, field: &str, icon: &str) -> egui::Pos2 {
    let start = output.shapes.iter().position(|shape| {
        matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == field)
    }).unwrap_or_else(|| panic!("field not found: {field}"));
    output.shapes[start..]
        .iter()
        .find_map(|shape| {
            if let egui::Shape::Text(text) = &shape.shape
                && text.galley.job.text == icon
            {
                Some(text.pos + text.galley.rect.center().to_vec2())
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("button not found for {field}"))
}

fn render(
    ctx: &egui::Context,
    input: egui::RawInput,
    instance: &mut PluginInstance,
) -> (egui::FullOutput, bool) {
    let mut changed = false;
    let mut output = ctx.run_ui(input, |ui| {
        changed |= component(ui, instance, Language::Portuguese, &AppTheme::dark());
    });
    output.textures_delta.clear();
    (output, changed)
}

#[test]
fn bundled_controls_fit_narrow_and_wide_panels_in_each_language() {
    for width in [240.0, 360.0, 960.0] {
        for theme in [AppTheme::light(), AppTheme::dark()] {
            for language in Language::ALL {
                let ctx = egui::Context::default();
                let mut visuals = egui::Visuals::default();
                theme.apply_to_visuals(&mut visuals);
                ctx.set_visuals(visuals);
                for plugin in crate::core::plugins::bundled() {
                    for definition in &plugin.package.components {
                        let mut instance = PluginInstance::new(&plugin, definition);
                        let values = instance.values.clone();
                        let mut output = ctx.run_ui(input(width), |ui| {
                            let right = ui.max_rect().right();
                            assert!(!component(ui, &mut instance, language, &theme));
                            assert!(
                                ui.min_rect().right() <= right + 1.0,
                                "{} overflows a {width}px panel in {}: {} > {right}",
                                definition.id,
                                language.code(),
                                ui.min_rect().right(),
                            );
                        });
                        assert_eq!(instance.values, values);
                        assert!(output.shapes.iter().any(|shape| {
                            matches!(&shape.shape, egui::Shape::Text(text)
                                if text.galley.job.text == definition.localized_name(language))
                        }));
                        output.textures_delta.clear();
                    }
                }
            }
        }
    }
}

#[test]
fn score_stepper_and_quick_actions_update_the_rendered_score() {
    let plugin = crate::core::plugins::bundled().remove(0);
    let definition = plugin
        .package
        .components
        .iter()
        .find(|item| item.id == "scoreboard")
        .unwrap();
    let mut instance = PluginInstance::new(&plugin, definition);
    let ctx = egui::Context::default();
    let raw = input(280.0);
    let (output, _) = render(&ctx, raw.clone(), &mut instance);
    let (output, changed) = render(
        &ctx,
        click(raw.clone(), field_button(&output, "Pontos A", PLUS)),
        &mut instance,
    );
    assert!(changed);
    assert_eq!(instance.values["score_a"], 1);
    let (output, changed) = render(
        &ctx,
        click(raw.clone(), position(&output, "A +1")),
        &mut instance,
    );
    assert!(changed);
    assert_eq!(instance.values["score_a"], 2);
    assert!(
        crate::core::plugins::runtime::primitives(&instance, instance.created_at)
            .unwrap()
            .iter()
            .any(|part| part.text == "2")
    );
    let (output, changed) = render(
        &ctx,
        click(raw.clone(), position(&output, "Zerar")),
        &mut instance,
    );
    assert!(changed);
    assert_eq!(instance.values["score_a"], 0);
    let (_, changed) = render(
        &ctx,
        click(raw, field_button(&output, "Pontos A", MINUS)),
        &mut instance,
    );
    assert!(!changed);
    assert_eq!(instance.values["score_a"], 0);
}

#[test]
fn fractional_steps_reach_the_limit_without_rounding_to_integers() {
    let field: PluginField = serde_json::from_value(serde_json::json!({
        "id":"value", "label":"Valor", "kind":"number", "default":0,
        "min":0, "max":1, "step":0.3,
    }))
    .unwrap();
    let ctx = egui::Context::default();
    let raw = input(280.0);
    let mut values = crate::core::plugins::defaults(std::slice::from_ref(&field));
    let draw = |input, values: &mut Map<String, Value>| {
        let mut changed = false;
        let mut output = ctx.run_ui(input, |ui| {
            changed |= fields(
                ui,
                std::slice::from_ref(&field),
                values,
                Language::Portuguese,
                &AppTheme::dark(),
            );
        });
        output.textures_delta.clear();
        (output, changed)
    };
    let (mut output, _) = draw(raw.clone(), &mut values);
    for expected in [0.3, 0.6, 0.9, 1.0] {
        let (next, changed) = draw(click(raw.clone(), position(&output, PLUS)), &mut values);
        output = next;
        assert!(changed);
        assert!((values["value"].as_f64().unwrap() - expected).abs() < 1e-9);
        assert!(field.accepts(&values["value"]));
    }
    let (_, changed) = draw(click(raw, position(&output, PLUS)), &mut values);
    assert!(!changed);
}

#[test]
fn javascript_buttons_dispatch_the_declared_event() {
    let plugin = crate::core::plugins::bundled().remove(0);
    let mut instance = PluginInstance::new(&plugin, &plugin.package.components[0]);
    instance.definition.fields.clear();
    instance.definition.render.clear();
    instance.definition.javascript = Some(crate::core::plugins::JavaScriptComponent {
        source: "({ render() { return '<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>'; }, onEvent(api, event) { return { count: api.state.count + (event.type === 'increment' ? 1 : 0) }; } })".into(),
        initial_state: Map::new(),
    });
    instance.state.insert("count".into(), Value::from(0));
    instance.definition.buttons = vec![
        serde_json::from_value(serde_json::json!({
            "label":"Somar", "event":"increment",
        }))
        .unwrap(),
    ];
    let ctx = egui::Context::default();
    let raw = input(280.0);
    let (output, _) = render(&ctx, raw.clone(), &mut instance);
    let (output, changed) = render(&ctx, click(raw, position(&output, "Somar")), &mut instance);
    assert!(changed);
    assert_eq!(instance.state["count"], 1);
    assert!(!output.shapes.iter().any(|shape| {
        matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.starts_with("Plugin:"))
    }));
}

#[test]
fn component_operational_omits_appearance_fields_and_keeps_operational_controls() {
    let plugin = crate::core::plugins::bundled().remove(0);
    let definition = plugin
        .package
        .components
        .iter()
        .find(|item| item.id == "scoreboard")
        .unwrap();
    let mut instance = PluginInstance::new(&plugin, definition);
    let ctx = egui::Context::default();
    let raw = input(320.0);

    let mut out_all = ctx.run_ui(raw.clone(), |ui| {
        component(ui, &mut instance, Language::Portuguese, &AppTheme::dark());
    });
    out_all.textures_delta.clear();

    let all_texts: Vec<String> = out_all
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
    assert!(all_texts.iter().any(|t| t.contains("Orientação")));
    assert!(all_texts.iter().any(|t| t.contains("Estilo visual")));
    assert!(all_texts.iter().any(|t| t.contains("Cor do destaque")));
    assert!(all_texts.iter().any(|t| t.contains("Fundo da caixa")));
    assert!(all_texts.iter().any(|t| t.contains("Arredondamento")));

    let mut out_op = ctx.run_ui(raw, |ui| {
        component_operational(ui, &mut instance, Language::Portuguese, &AppTheme::dark());
    });
    out_op.textures_delta.clear();

    let op_texts: Vec<String> = out_op
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

    assert!(op_texts.iter().any(|t| t.contains("Equipes")));
    assert!(op_texts.iter().any(|t| t.contains("Equipe A")));
    assert!(op_texts.iter().any(|t| t.contains("Pontos A")));
    assert!(op_texts.iter().any(|t| t.contains("Equipe B")));
    assert!(op_texts.iter().any(|t| t.contains("Pontos B")));
    assert!(op_texts.iter().any(|t| t.contains("A +1")));
    assert!(op_texts.iter().any(|t| t.contains("B +1")));
    assert!(op_texts.iter().any(|t| t.contains("Zerar")));

    assert!(!op_texts.iter().any(|t| t.contains("Orientação")));
    assert!(!op_texts.iter().any(|t| t.contains("Estilo visual")));
    assert!(!op_texts.iter().any(|t| t.contains("Cor do destaque")));
    assert!(!op_texts.iter().any(|t| t.contains("Fundo da caixa")));
    assert!(!op_texts.iter().any(|t| t.contains("Arredondamento")));
}
