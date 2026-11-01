use super::*;
use crate::core::design::{DesignDocument, DesignLayer, LayerContent};

#[test]
fn bundled_packages_validate_and_components_respond_to_controls() {
    let plugins = bundled();
    for plugin in &plugins {
        plugin
            .package
            .validate()
            .unwrap_or_else(|error| panic!("{}: {error}", plugin.package.id));
        for component in &plugin.package.components {
            let mut instance = PluginInstance::new(plugin, component);
            let before = runtime::primitives(&instance, instance.created_at).unwrap();
            assert!(!before.is_empty());
            for button in &component.buttons {
                runtime::update(&mut instance, &button.script).unwrap();
                runtime::primitives(&instance, instance.created_at + 2_000).unwrap();
            }
        }
    }
    let mut plugins = bundled();
    let plugin = plugins.remove(0);
    let mut score = PluginInstance::new(&plugin, &plugin.package.components[0]);
    let action = score.definition.buttons[0].script.clone();
    runtime::update(&mut score, &action).unwrap();
    assert_eq!(score.values["score_a"], 1);
    let primitives = runtime::primitives(&score, score.created_at).unwrap();
    assert!(
        primitives.iter().any(|part| part.text == "1"),
        "score text '1' should be present: {primitives:?}"
    );
    assert!(
        primitives.iter().any(|part| part.text == "0"),
        "score text '0' should be present: {primitives:?}"
    );
}

#[test]
fn scoreboard_supports_multiple_teams_orientations_and_styles() {
    let plugins = bundled();
    let plugin = &plugins[0];
    let component = &plugin.package.components[0];
    assert_eq!(component.id, "scoreboard");

    // 1. Default (2 teams, horizontal, cards)
    let mut instance = PluginInstance::new(plugin, component);
    let prims = runtime::primitives(&instance, instance.created_at).unwrap();
    assert!(prims.iter().any(|p| p.text == "Equipe A"));
    assert!(prims.iter().any(|p| p.text == "Equipe B"));
    assert!(!prims.iter().any(|p| p.text == "Equipe C"));

    // 2. 3 teams, vertical, clean
    instance.values.insert("teams".into(), serde_json::json!(3));
    instance.values.insert("orientation".into(), serde_json::json!("vertical"));
    instance.values.insert("style".into(), serde_json::json!("clean"));
    let prims = runtime::primitives(&instance, instance.created_at).unwrap();
    assert!(prims.iter().any(|p| p.text == "Equipe A"));
    assert!(prims.iter().any(|p| p.text == "Equipe B"));
    assert!(prims.iter().any(|p| p.text == "Equipe C"));
    assert!(!prims.iter().any(|p| p.text == "Equipe D"));

    // 3. 4 teams, horizontal, versus
    instance.values.insert("teams".into(), serde_json::json!(4));
    instance.values.insert("orientation".into(), serde_json::json!("horizontal"));
    instance.values.insert("style".into(), serde_json::json!("versus"));
    instance.values.insert("score_c".into(), serde_json::json!(12));
    instance.values.insert("score_d".into(), serde_json::json!(7));
    let prims = runtime::primitives(&instance, instance.created_at).unwrap();
    assert!(prims.iter().any(|p| p.text == "Equipe D"));
    assert!(prims.iter().any(|p| p.text == "12"));
    assert!(prims.iter().any(|p| p.text == "7"));

    // 4. Test button C +1 and D +1
    let btn_c = component.buttons.iter().find(|b| b.label == "C +1").unwrap();
    runtime::update(&mut instance, &btn_c.script).unwrap();
    assert_eq!(instance.values["score_c"], 13);

    let btn_reset = component.buttons.iter().find(|b| b.label == "Zerar").unwrap();
    runtime::update(&mut instance, &btn_reset.script).unwrap();
    assert_eq!(instance.values["score_a"], 0);
    assert_eq!(instance.values["score_b"], 0);
    assert_eq!(instance.values["score_c"], 0);
    assert_eq!(instance.values["score_d"], 0);
}

#[test]
fn scoreboard_supports_independent_team_and_score_text_options() {
    let plugins = bundled();
    let plugin = &plugins[0];
    let component = &plugin.package.components[0];
    let mut instance = PluginInstance::new(plugin, component);

    instance.values.insert("team_scale".into(), serde_json::json!(1.5));
    instance.values.insert("score_scale".into(), serde_json::json!(0.8));
    instance.values.insert("team_weight".into(), serde_json::json!("bold"));
    instance.values.insert("score_weight".into(), serde_json::json!("black"));
    instance.values.insert("team_color".into(), serde_json::json!("#FF0000FF"));
    instance.values.insert("score_color".into(), serde_json::json!("#00FF00FF"));
    instance.values.insert("score_suffix".into(), serde_json::json!(" pts"));

    let prims = runtime::primitives(&instance, instance.created_at).unwrap();
    let team_prim = prims.iter().find(|p| p.text == "Equipe A").unwrap();
    assert_eq!(team_prim.font_weight, Some(700));
    assert_eq!(team_prim.color, Some([255, 0, 0, 255]));
    assert!((team_prim.font_scale - 0.48 * 1.5).abs() < 0.001);

    let score_prim = prims.iter().find(|p| p.text == "0 pts").unwrap();
    assert_eq!(score_prim.font_weight, Some(900));
    assert_eq!(score_prim.color, Some([0, 255, 0, 255]));
    assert!((score_prim.font_scale - 1.25 * 0.8).abs() < 0.001);

    let mut doc = DesignDocument::default();
    doc.layers.push(DesignLayer::new(LayerContent::Plugin(Box::new(instance))));
    let materialized = runtime::materialize(&doc, 0);
    let team_layer = materialized.layers.iter().find(|l| match &l.content {
        LayerContent::Text(t) => t == "Equipe A",
        _ => false,
    }).unwrap();
    assert_eq!(team_layer.effective_font_weight(), 700);
    assert_eq!(team_layer.color, [255, 0, 0, 255]);

    let score_layer = materialized.layers.iter().find(|l| match &l.content {
        LayerContent::Text(t) => t == "0 pts",
        _ => false,
    }).unwrap();
    assert_eq!(score_layer.effective_font_weight(), 900);
    assert_eq!(score_layer.color, [0, 255, 0, 255]);
}

#[test]
fn plugin_instances_remain_renderable_after_serialization_and_uninstallation() {
    let mut plugins = bundled();
    let plugin = plugins.remove(0);
    let mut layer = DesignLayer::new(LayerContent::Plugin(Box::new(PluginInstance::new(
        &plugin,
        &plugin.package.components[0],
    ))));
    layer.rect = [0.2, 0.3, 0.6, 0.4];
    let scene = DesignDocument {
        layers: vec![layer],
        ..Default::default()
    };
    let saved = serde_json::to_vec(&scene).unwrap();
    drop(plugin);
    let restored: DesignDocument = serde_json::from_slice(&saved).unwrap();
    let rendered = runtime::materialize(&restored, 0);
    assert!(!rendered.layers.is_empty());
    assert!(
        rendered
            .layers
            .iter()
            .all(|layer| !matches!(layer.content, LayerContent::Plugin(_)))
    );
    assert_eq!(rendered.layers[0].rect, [0.2, 0.3, 0.6, 0.4]);
    let frame = crate::render::design::frame(&restored, 320);
    assert!(frame.image.pixels().any(|pixel| pixel[3] > 0));
}

#[test]
fn scripts_stop_at_the_operation_budget_and_reject_invalid_results() {
    assert!(runtime::run("loop {}", &Map::new(), &Value::Null, 0).is_err());
    let mut plugins = bundled();
    let plugin = plugins.remove(0);
    let mut instance = PluginInstance::new(&plugin, &plugin.package.components[0]);
    let before = instance.values.clone();
    assert!(runtime::update(&mut instance, "props.score_a = -1; props").is_err());
    assert_eq!(instance.values, before);
    instance.definition.render = "[#{kind: \"unknown\"}]".into();
    assert!(runtime::primitives(&instance, 0).is_err());
    let mut package = plugin.package.clone();
    package.api_version = 999;
    assert!(package.validate().is_err());
}

#[test]
fn command_scripts_use_the_prepared_content_and_plugin_settings() {
    let mut plugins = bundled();
    let plugin = plugins.remove(0);
    assert!(
        !plugin.package.commands.is_empty(),
        "stage-tools should have commands for this test"
    );
    let props = defaults(&plugin.package.settings);
    let context = serde_json::json!({"title":"Aviso", "text":"Olá equipe", "operator":"Ana"});
    for command in &plugin.package.commands {
        let actions = runtime::actions(command, &props, &context).unwrap();
        assert!(!actions.is_empty());
    }
    let actions = runtime::actions(&plugin.package.commands[0], &props, &context).unwrap();
    assert!(!actions.is_empty());
}

#[test]
fn extensions_and_appearance_roundtrip_and_legacy_modules_stay_available() {
    use crate::core::{
        interface_theme::{InterfaceTheme, ThemeMode},
        preferences::AppSettings,
    };
    let mut legacy: AppSettings = serde_json::from_value(serde_json::json!({
        "bible_enabled":false,"chat_enabled":false,"operators":[{"dark_mode":true}]
    }))
    .unwrap();
    assert_eq!(legacy.operator().theme_mode, ThemeMode::Legacy);
    legacy.normalize();
    assert!(legacy.bible_enabled && legacy.chat_enabled);
    let mut custom = InterfaceTheme {
        id: "custom.test".into(),
        name: "Minha paleta".into(),
        ..Default::default()
    };
    custom.dark.insert("accent".into(), "#AABBCC".into());
    legacy.interface_themes.push(custom);
    legacy.operator_mut().interface_theme_id = "custom.test".into();
    legacy.operator_mut().theme_mode = ThemeMode::System;
    if !legacy.plugins.is_empty() {
        legacy.plugins[0].enabled = false;
    }
    let db = crate::services::db::AppDb::open_in_memory().unwrap();
    db.save_settings(&legacy).unwrap();
    assert_eq!(db.load_settings().unwrap().unwrap(), legacy);
}

#[test]
#[ignore = "requires a Chromium binary in OMNI_CHROMIUM"]
fn plugin_shaders_compile_and_draw_in_webgl() {
    let executable = std::env::var("OMNI_CHROMIUM")
        .expect("set OMNI_CHROMIUM to a Chromium executable");
    let fragments: Vec<_> = bundled()
        .into_iter()
        .flat_map(|plugin| plugin.package.shaders)
        .map(|shader| crate::core::plugins::shader::glsl(&shader))
        .collect();
    let html = format!(
        r#"<!doctype html><body><canvas id="canvas" width="64" height="64"></canvas><pre id="result">pending</pre><script>
    try {{
      const gl=document.getElementById('canvas').getContext('webgl2',{{antialias:false}}); if(!gl)throw Error('WebGL unavailable');
      const compile=(type,src)=>{{const s=gl.createShader(type);gl.shaderSource(s,src);gl.compileShader(s);if(!gl.getShaderParameter(s,gl.COMPILE_STATUS))throw Error(gl.getShaderInfoLog(s));return s;}};
      const vertex='#version 300 es\nprecision highp float;out vec2 uv;void main(){{vec2 p=vec2(float((gl_VertexID<<1)&2),float(gl_VertexID&2));gl_Position=vec4(p*vec2(2,-2)+vec2(-1,1),0,1);uv=p;}}';
      for(const fragment of {}){{
        const p=gl.createProgram();gl.attachShader(p,compile(gl.VERTEX_SHADER,vertex));gl.attachShader(p,compile(gl.FRAGMENT_SHADER,fragment));gl.linkProgram(p);if(!gl.getProgramParameter(p,gl.LINK_STATUS))throw Error(gl.getProgramInfoLog(p));gl.useProgram(p);
        const set=(name,...v)=>gl.uniform4f(gl.getUniformLocation(p,name),...v);
        set('screen',64,64,1,0);set('motion',.35,1,1,.55);set('detail',4,0,17,.3);set('color0',.05,.1,.2,1);set('color1',.2,.8,.7,1);set('color2',.6,.3,.9,1);set('finish',.015,0,1,0);
        gl.viewport(0,0,64,64);gl.bindVertexArray(gl.createVertexArray());gl.drawArrays(gl.TRIANGLES,0,3);gl.finish();const pixel=new Uint8Array(4);gl.readPixels(32,32,1,1,gl.RGBA,gl.UNSIGNED_BYTE,pixel);if(gl.getError()!==gl.NO_ERROR||pixel[3]!==255||pixel[0]+pixel[1]+pixel[2]===0)throw Error('No shader output');gl.deleteProgram(p);
      }}document.body.dataset.result='ok';document.getElementById('result').textContent='PASS';
    }}catch(e){{document.getElementById('result').textContent=e.message;}}
    </script>"#,
        serde_json::to_string(&fragments).unwrap()
    );
    let url = format!(
        "data:text/html,{}",
        html.bytes()
            .map(|b| format!("%{b:02X}"))
            .collect::<String>()
    );
    let profile = format!("/tmp/opencode/omni-plugin-webgl-{}", std::process::id());
    let result = std::process::Command::new(executable)
        .args([
            "--headless",
            "--no-sandbox",
            "--disable-dev-shm-usage",
            "--use-gl=angle",
            "--use-angle=swiftshader",
            "--enable-unsafe-swiftshader",
            "--dump-dom",
        ])
        .arg(format!("--user-data-dir={profile}"))
        .arg(url)
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(profile);
    let output = String::from_utf8_lossy(&result.stdout);
    assert!(
        result.status.success() && output.contains("data-result=\"ok\""),
        "{output}\n{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn announcements_transitions_render_interpolated_states() {
    let plugins = bundled();
    let plugin = &plugins[0];
    let component = plugin
        .package
        .components
        .iter()
        .find(|c| c.id == "announcements")
        .unwrap();

    let mut instance = PluginInstance::new(plugin, component);
    instance
        .values
        .insert("messages".into(), serde_json::json!("Aviso 1|Aviso 2"));
    instance.values.insert("seconds".into(), serde_json::json!(5));
    instance
        .values
        .insert("transition_ms".into(), serde_json::json!(800));

    // 1. Cut transition: always exactly 1 text primitive
    instance
        .values
        .insert("transition".into(), serde_json::json!("cut"));
    let prims = runtime::primitives(&instance, 0).unwrap();
    let texts: Vec<_> = prims.iter().filter(|p| !p.text.is_empty()).collect();
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].text, "Aviso 1");

    let prims = runtime::primitives(&instance, 4600).unwrap();
    let texts: Vec<_> = prims.iter().filter(|p| !p.text.is_empty()).collect();
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].text, "Aviso 1");

    let prims = runtime::primitives(&instance, 5200).unwrap();
    let texts: Vec<_> = prims.iter().filter(|p| !p.text.is_empty()).collect();
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].text, "Aviso 2");

    // 2. Fade transition: during hold time (e.g. 1000ms), 1 text; during transition (4600ms), 2 texts
    instance
        .values
        .insert("transition".into(), serde_json::json!("fade"));
    let prims_hold = runtime::primitives(&instance, 1000).unwrap();
    let texts_hold: Vec<_> = prims_hold.iter().filter(|p| !p.text.is_empty()).collect();
    assert_eq!(texts_hold.len(), 1);
    assert_eq!(texts_hold[0].text, "Aviso 1");

    let prims_trans = runtime::primitives(&instance, 4600).unwrap();
    let texts_trans: Vec<_> = prims_trans.iter().filter(|p| !p.text.is_empty()).collect();
    assert_eq!(texts_trans.len(), 2);
    assert_eq!(texts_trans[0].text, "Aviso 1");
    assert_eq!(texts_trans[1].text, "Aviso 2");
    // Ensure both have non-zero opacity
    assert!(texts_trans[0].opacity.unwrap() > 0.0);
    assert!(texts_trans[1].opacity.unwrap() > 0.0);

    // Verify layer color inheritance in materialize
    let mut doc = DesignDocument::default();
    let mut layer = DesignLayer::new(LayerContent::Plugin(Box::new(instance.clone())));
    layer.color = [255, 0, 0, 255]; // Red color chosen by user
    doc.layers.push(layer);
    let materialized = runtime::materialize(&doc, 4600);
    let text_layers: Vec<_> = materialized
        .layers
        .iter()
        .filter(|l| matches!(l.content, LayerContent::Text(_)))
        .collect();
    assert_eq!(text_layers.len(), 2);
    for tl in text_layers {
        assert_eq!(tl.color[0], 255);
        assert_eq!(tl.color[1], 0);
        assert_eq!(tl.color[2], 0);
        assert!(tl.color[3] > 0);
    }

    // 3. Slide horizontal transition
    instance
        .values
        .insert("transition".into(), serde_json::json!("slide_h"));
    let prims_slide = runtime::primitives(&instance, 4600).unwrap();
    let texts_slide: Vec<_> = prims_slide.iter().filter(|p| !p.text.is_empty()).collect();
    assert_eq!(texts_slide.len(), 2);

    // 4. Slide vertical transition
    instance
        .values
        .insert("transition".into(), serde_json::json!("slide_v"));
    let prims_v = runtime::primitives(&instance, 4600).unwrap();
    let texts_v: Vec<_> = prims_v.iter().filter(|p| !p.text.is_empty()).collect();
    assert_eq!(texts_v.len(), 2);

    // 5. Zoom transition
    instance
        .values
        .insert("transition".into(), serde_json::json!("zoom"));
    let prims_zoom = runtime::primitives(&instance, 4600).unwrap();
    let texts_zoom: Vec<_> = prims_zoom.iter().filter(|p| !p.text.is_empty()).collect();
    assert_eq!(texts_zoom.len(), 2);
}

