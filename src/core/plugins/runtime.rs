//! A bounded scripting runtime that produces portable drawing primitives and host actions.
use super::*;
use crate::core::design::{DesignDocument, LayerContent, ShapeKind};
use rhai::{AST, Dynamic, Engine, Scope};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

fn engine() -> &'static Engine {
    static ENGINE: OnceLock<Engine> = OnceLock::new();
    ENGINE.get_or_init(|| {
        let mut engine = Engine::new();
        engine
            .set_max_operations(80_000)
            .set_max_call_levels(24)
            .set_max_expr_depths(48, 24)
            .set_max_array_size(512)
            .set_max_map_size(128)
            .set_max_string_size(65_536);
        engine.set_module_resolver(rhai::module_resolvers::DummyModuleResolver::new());
        engine.on_print(|_| {});
        engine.register_fn("color", |value: &str| -> rhai::Array {
            crate::core::interface_theme::parse_color(value)
                .unwrap_or([255; 4])
                .into_iter()
                .map(|channel| Dynamic::from_int(i64::from(channel)))
                .collect()
        });
        engine
    })
}

pub fn compile(source: &str) -> Result<Arc<AST>, String> {
    if source.len() > 65_536 {
        return Err("plugins-too-large".into());
    }
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<AST>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Mutex::default);
    if let Some(ast) = cache.lock().unwrap().get(source).cloned() {
        return Ok(ast);
    }
    let ast = Arc::new(
        engine()
            .compile(source)
            .map_err(|error| error.to_string())?,
    );
    let mut cache = cache.lock().unwrap();
    if cache.len() >= 128 {
        cache.clear();
    }
    cache.insert(source.into(), ast.clone());
    Ok(ast)
}

pub fn run(
    source: &str,
    props: &Map<String, Value>,
    context: &Value,
    now: u64,
) -> Result<Value, String> {
    let ast = compile(source)?;
    let mut scope = Scope::new();
    scope.push(
        "props",
        rhai::serde::to_dynamic(props).map_err(|e| e.to_string())?,
    );
    scope.push_constant(
        "context",
        rhai::serde::to_dynamic(context).map_err(|e| e.to_string())?,
    );
    scope.push_constant("now", now.min(i64::MAX as u64) as i64);
    let output: Dynamic = engine()
        .eval_ast_with_scope(&mut scope, &ast)
        .map_err(|error| error.to_string())?;
    rhai::serde::from_dynamic(&output).map_err(|error| error.to_string())
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Primitive {
    pub kind: String,
    pub text: String,
    pub rect: [f32; 4],
    pub color: Option<[u8; 4]>,
    pub opacity: Option<f32>,
    pub alpha: Option<f32>,
    pub font_scale: f32,
    pub font_family: Option<String>,
    pub font_weight: Option<u16>,
    pub radius: f32,
    pub rotation: f32,
    pub align: String,
}
impl Default for Primitive {
    fn default() -> Self {
        Self {
            kind: "text".into(),
            text: String::new(),
            rect: [0.0, 0.0, 1.0, 1.0],
            color: None,
            opacity: None,
            alpha: None,
            font_scale: 1.0,
            font_family: None,
            font_weight: None,
            radius: 0.0,
            rotation: 0.0,
            align: "center".into(),
        }
    }
}

pub fn primitives(instance: &PluginInstance, now: u64) -> Result<Vec<Primitive>, String> {
    let context = serde_json::json!({"created_at": instance.created_at, "elapsed_ms": now.saturating_sub(instance.created_at)});
    let value = run(&instance.definition.render, &instance.values, &context, now)?;
    let output: Vec<Primitive> =
        serde_json::from_value(value).map_err(|error| error.to_string())?;
    if output.len() > 128
        || output.iter().any(|item| {
            !matches!(item.kind.as_str(), "text" | "rect" | "ellipse" | "line")
                || item.text.len() > 16_384
                || item.rect.iter().any(|n| !n.is_finite() || n.abs() > 4.0)
                || !item.font_scale.is_finite()
                || !(0.01..=16.0).contains(&item.font_scale)
                || !item.radius.is_finite()
                || !item.rotation.is_finite()
        })
    {
        return Err("plugins-invalid-output".into());
    }
    Ok(output)
}

pub fn update(instance: &mut PluginInstance, script: &str) -> Result<(), String> {
    let value = run(
        script,
        &instance.values,
        &serde_json::json!({"created_at": instance.created_at}),
        crate::core::design::now_ms(),
    )?;
    let values = value.as_object().ok_or("plugins-invalid-output")?;
    for field in &instance.definition.fields {
        if let Some(value) = values.get(&field.id)
            && !field.accepts(value)
        {
            return Err(format!("plugins.invalid_field: {}", field.id));
        }
    }
    for field in &instance.definition.fields {
        if let Some(value) = values.get(&field.id) {
            instance.values.insert(field.id.clone(), value.clone());
        }
    }
    Ok(())
}

pub fn materialize(scene: &DesignDocument, now: u64) -> std::borrow::Cow<'_, DesignDocument> {
    if !scene
        .layers
        .iter()
        .any(|layer| matches!(layer.content, LayerContent::Plugin(_)))
    {
        return std::borrow::Cow::Borrowed(scene);
    }
    let mut result = scene.clone();
    result.layers.clear();
    for layer in &scene.layers {
        let LayerContent::Plugin(instance) = &layer.content else {
            result.layers.push(layer.clone());
            continue;
        };
        if !layer.visible {
            continue;
        }
        let parts = primitives(instance, now).unwrap_or_else(|_| {
            vec![Primitive {
                text: instance.definition.name.clone(),
                color: Some([255, 160, 70, 255]),
                ..Default::default()
            }]
        });
        for (index, part) in parts.into_iter().enumerate() {
            let mut child = layer.clone();
            child.id = format!("{}:plugin:{index}", layer.id);
            child.content = if part.kind == "text" {
                LayerContent::Text(part.text)
            } else {
                LayerContent::Shape {
                    radius: part.radius,
                }
            };
            child.shape = match part.kind.as_str() {
                "ellipse" => ShapeKind::Ellipse,
                "line" => ShapeKind::Line,
                _ => ShapeKind::Rectangle,
            };
            child.rect = [
                layer.rect[0] + part.rect[0] * layer.rect[2],
                layer.rect[1] + part.rect[1] * layer.rect[3],
                part.rect[2].max(0.001) * layer.rect[2],
                part.rect[3].max(0.001) * layer.rect[3],
            ];
            if layer.rotation != 0.0 {
                let aspect = scene.aspect();
                let cx = layer.rect[0] + layer.rect[2] * 0.5;
                let cy = layer.rect[1] + layer.rect[3] * 0.5;
                let dx = (child.rect[0] + child.rect[2] * 0.5 - cx) * aspect;
                let dy = child.rect[1] + child.rect[3] * 0.5 - cy;
                let (s, c) = layer.rotation.to_radians().sin_cos();
                child.rect[0] = cx + (c * dx - s * dy) / aspect - child.rect[2] * 0.5;
                child.rect[1] = cy + s * dx + c * dy - child.rect[3] * 0.5;
            }
            child.rotation += part.rotation;
            let mut col = part.color.unwrap_or(layer.color);
            let op = part.opacity.or(part.alpha);
            if let Some(op) = op {
                let factor = if op > 1.0 { (op / 255.0).clamp(0.0, 1.0) } else { op.clamp(0.0, 1.0) };
                col[3] = (col[3] as f32 * factor).round() as u8;
                child.opacity = (layer.opacity * factor).clamp(0.0, 1.0);
            }
            child.color = col;
            child.font_size = (layer.font_size * part.font_scale).clamp(1.0, 1000.0);
            if let Some(family) = part.font_family {
                child.font_family = family;
            }
            if let Some(weight) = part.font_weight {
                child.set_font_weight(weight);
            }
            child.align = part.align;
            result.layers.push(child);
        }
    }
    std::borrow::Cow::Owned(result)
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HostAction {
    Notify { text: String },
    PreviewText { title: String, text: String },
    SaveText { title: String, text: String },
    QueueText { title: String, text: String },
}

pub fn actions(
    command: &PluginCommand,
    props: &Map<String, Value>,
    context: &Value,
) -> Result<Vec<HostAction>, String> {
    if let Some(source)=&command.javascript {return super::javascript::command(source,props,context);}
    let output = run(
        &command.script,
        props,
        context,
        crate::core::design::now_ms(),
    )?;
    let actions: Vec<HostAction> = serde_json::from_value(output).map_err(|e| e.to_string())?;
    if actions.len() > 32 {
        return Err("plugins-invalid-output".into());
    }
    Ok(actions)
}
