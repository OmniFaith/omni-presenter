//! Versioned extension packages shared by settings, the editor and renderers.
pub mod bundled;
pub mod runtime;
pub mod shader;
pub mod javascript;
pub use bundled::bundled;
#[cfg(test)]
mod tests;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginPackage {
    #[serde(default)] pub translations: crate::core::i18n::Translations,
    #[serde(default)] pub web_panel: Option<WebPanel>,
    pub api_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub components: Vec<PluginComponent>,
    pub commands: Vec<PluginCommand>,
    pub shaders: Vec<PluginShader>,
    pub interface_themes: Vec<crate::core::interface_theme::InterfaceTheme>,
    pub templates: Vec<crate::core::design::DesignDocument>,
    pub settings: Vec<PluginField>,
}

impl Default for PluginPackage {
    fn default() -> Self {
        Self {
            translations: Default::default(), web_panel: None,
            api_version: 1,
            id: String::new(),
            name: String::new(),
            version: "1.0.0".into(),
            author: String::new(),
            description: String::new(),
            components: vec![],
            commands: vec![],
            shaders: vec![],
            interface_themes: vec![],
            templates: vec![],
            settings: vec![],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InstalledPlugin {
    pub package: PluginPackage,
    #[serde(default = "enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub settings: Map<String, Value>,
}
fn enabled() -> bool {
    true
}

#[derive(Clone)]
pub enum PluginRequest {
    Command { plugin: String, command: String },
    Template { plugin: String, index: usize },
    OpenPanel { plugin: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct WebPanel { pub html: String, pub css: String, pub javascript: String }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct JavaScriptComponent { pub source: String, pub initial_state: Map<String,Value> }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldKind {
    Text,
    Multiline,
    Number,
    Toggle,
    Color,
    Select,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginField {
    #[serde(default)] pub translations: crate::core::i18n::Translations,
    pub id: String,
    pub label: String,
    pub kind: FieldKind,
    pub default: Value,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub step: Option<f64>,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub appearance: Option<bool>,
}

impl PluginField {
    pub fn is_transition(&self) -> bool {
        let id = self.id.to_ascii_lowercase();
        id == "transition" || id == "transition_ms" || id.contains("transi")
    }

    pub fn is_appearance(&self) -> bool {
        if self.is_transition() {
            return false;
        }
        if let Some(app) = self.appearance {
            return app;
        }
        if matches!(self.kind, FieldKind::Color) {
            return true;
        }
        let id = self.id.to_ascii_lowercase();
        let label = self.label.to_ascii_lowercase();
        matches!(
            id.as_str(),
            "orientation"
                | "orientacao"
                | "orientação"
                | "style"
                | "visual_style"
                | "estilo"
                | "estilo_visual"
                | "radius"
                | "corner_radius"
                | "arredondamento"
                | "font"
                | "font_size"
                | "font_family"
                | "align"
                | "alignment"
                | "alinhamento"
                | "border"
                | "border_width"
                | "border_radius"
                | "borda"
                | "stroke"
                | "stroke_width"
                | "shadow"
                | "sombra"
                | "accent"
                | "bg"
                | "fundo"
        ) || id.contains("color")
            || id.contains("cor_")
            || id.contains("_cor")
            || id == "cor"
            || id.contains("bg")
            || id.contains("fundo")
            || id.contains("radius")
            || id.contains("border")
            || id.contains("borda")
            || id.contains("stroke")
            || id.contains("shadow")
            || id.contains("sombra")
            || label.contains("orientação")
            || label.contains("orientacao")
            || label.contains("estilo visual")
            || label.contains("arredondamento")
            || label.contains("fundo")
            || label.contains("cor do")
            || label.contains("cor da")
            || label.contains("cor de")
            || label.starts_with("cor ")
            || label == "cor"
            || label.contains("estilo")
            || label.contains("borda")
            || label.contains("sombra")
    }

    pub fn accepts(&self, value: &Value) -> bool {
        match self.kind {
            FieldKind::Text | FieldKind::Multiline => {
                value.as_str().is_some_and(|text| text.len() <= 16_384)
            }
            FieldKind::Number => value.as_f64().is_some_and(|n| {
                n.is_finite()
                    && self.min.is_none_or(|min| n >= min)
                    && self.max.is_none_or(|max| n <= max)
            }),
            FieldKind::Toggle => value.is_boolean(),
            FieldKind::Color => value
                .as_str()
                .is_some_and(|color| crate::core::interface_theme::parse_color(color).is_some()),
            FieldKind::Select => value
                .as_str()
                .is_some_and(|text| self.options.is_empty() || self.options.iter().any(|opt| opt == text)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginButton {
    #[serde(default)] pub translations: crate::core::i18n::Translations,
    pub label: String,
    #[serde(default)] pub event: Option<String>,
    #[serde(default)]
    pub script: String,
    #[serde(default)]
    pub condition: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginComponent {
    #[serde(default)] pub translations: crate::core::i18n::Translations,
    #[serde(default)] pub javascript: Option<JavaScriptComponent>,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub animated: bool,
    #[serde(default)]
    pub fields: Vec<PluginField>,
    #[serde(default)]
    pub buttons: Vec<PluginButton>,
    #[serde(default)] pub render: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginCommand {
    #[serde(default)] pub translations: crate::core::i18n::Translations,
    #[serde(default)] pub javascript: Option<String>,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)] pub script: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginShader {
    #[serde(default)] pub translations: crate::core::i18n::Translations,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// A WGSL function `fn plugin_color(p: vec2f, t: f32) -> vec3f`.
    pub wgsl: String,
    /// The equivalent WebGL function `vec3 plugin_color(vec2 p, float t)`.
    pub glsl: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginInstance {
    #[serde(default)] pub state: Map<String,Value>,
    pub plugin_id: String,
    pub version: String,
    /// Embed the component so saved designs remain portable across installations.
    pub definition: PluginComponent,
    pub values: Map<String, Value>,
    pub created_at: u64,
}

impl PluginInstance {
    pub fn new_localized(plugin: &InstalledPlugin, component: &PluginComponent, language: crate::core::i18n::Language) -> Self {
        let mut instance = Self::new(plugin, component);
        for field in &component.fields {
            if !plugin.settings.contains_key(&field.id)
                && matches!(field.kind, FieldKind::Text | FieldKind::Multiline)
                && let Some(default) = field.default.as_str()
            {
                let text = crate::core::i18n::localize(&field.translations, language, "default", default);
                instance.values.insert(field.id.clone(), Value::String(text.to_owned()));
            }
        }
        instance
    }

    pub fn new(plugin: &InstalledPlugin, component: &PluginComponent) -> Self {
        let mut values = defaults(&component.fields);
        for field in &component.fields {
            if let Some(value) = plugin
                .settings
                .get(&field.id)
                .filter(|value| field.accepts(value))
            {
                values.insert(field.id.clone(), value.clone());
            }
        }
        Self {
            state: component.javascript.as_ref().map(|js|js.initial_state.clone()).unwrap_or_default(),
            plugin_id: plugin.package.id.clone(),
            version: plugin.package.version.clone(),
            definition: component.clone(),
            values,
            created_at: crate::core::design::now_ms(),
        }
    }
}

pub fn defaults(fields: &[PluginField]) -> Map<String, Value> {
    fields
        .iter()
        .map(|field| (field.id.clone(), field.default.clone()))
        .collect()
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
}

pub fn validate_fields(fields: &[PluginField]) -> Result<(), String> {
    let mut ids = std::collections::HashSet::new();
    if fields.len() > 64 {
        return Err("plugins-too-large".into());
    }
    for field in fields {
        if !valid_id(&field.id)
            || !ids.insert(&field.id)
            || !field.accepts(&field.default)
            || field.min.is_some_and(|min| !min.is_finite())
            || field.max.is_some_and(|max| !max.is_finite())
            || field
                .step
                .is_some_and(|step| !step.is_finite() || step <= 0.0)
        {
            return Err(format!("plugins.invalid_field: {}", field.id));
        }
    }
    Ok(())
}

impl PluginPackage {
    pub fn validate(&self) -> Result<(), String> {
        if !matches!(self.api_version,1|2) {
            return Err("plugins-api-mismatch".into());
        }
        if !valid_id(&self.id)
            || self.name.trim().is_empty()
            || self.name.len() > 200
            || self.version.trim().is_empty()
        {
            return Err("plugins-invalid-manifest".into());
        }
        if self.components.len() + self.commands.len() + self.shaders.len() + self.templates.len()
            > 64
        {
            return Err("plugins-too-large".into());
        }
        validate_fields(&self.settings)?;
        let mut ids = std::collections::HashSet::new();
        for component in &self.components {
            if !valid_id(&component.id)
                || component.name.trim().is_empty()
                || !ids.insert(format!("component:{}", component.id))
            {
                return Err("plugins-duplicate-id".into());
            }
            validate_fields(&component.fields)?;
            if let Some(js)=&component.javascript { javascript::validate(js)?; }
            else { runtime::compile(&component.render)?; }
            for button in &component.buttons {
                if component.javascript.is_none() { runtime::compile(&button.script)?; }
            }
            let plugin = InstalledPlugin {
                package: self.clone(),
                enabled: true,
                settings: Map::new(),
            };
            if component.javascript.is_none() {
                runtime::primitives(&PluginInstance::new(&plugin, component),crate::core::design::now_ms())?;
            }
        }
        for command in &self.commands {
            if !valid_id(&command.id)
                || command.name.trim().is_empty()
                || !ids.insert(format!("command:{}", command.id))
            {
                return Err("plugins-duplicate-id".into());
            }
            if let Some(source)=&command.javascript { javascript::validate_expression(source)?; }
            else { runtime::compile(&command.script)?; }
        }
        for shader in &self.shaders {
            if !valid_id(&shader.id)
                || shader.name.trim().is_empty()
                || !ids.insert(format!("shader:{}", shader.id))
            {
                return Err("plugins-duplicate-id".into());
            }
            shader::validate(shader)?;
        }
        for theme in &self.interface_themes {
            theme.validate()?;
        }
        for template in &self.templates {
            if template.name.trim().is_empty()
                || template.width < 128
                || template.height < 128
                || template.layers.len() > 128
            {
                return Err("plugins-invalid-output".into());
            }
        }
        Ok(())
    }
}

macro_rules! named {
    ($($kind:ty),*)=>{$(impl $kind {
        pub fn localized_name(&self,language:crate::core::i18n::Language)->&str {crate::core::i18n::localize(&self.translations,language,"name",&self.name)}
        pub fn localized_description(&self,language:crate::core::i18n::Language)->&str {crate::core::i18n::localize(&self.translations,language,"description",&self.description)}
    })*};
}
named!(PluginPackage,PluginComponent,PluginCommand,PluginShader);
impl PluginField { pub fn localized_label(&self,language:crate::core::i18n::Language)->&str { crate::core::i18n::localize(&self.translations,language,"label",&self.label) } }
impl PluginButton { pub fn localized_label(&self,language:crate::core::i18n::Language)->&str { crate::core::i18n::localize(&self.translations,language,"label",&self.label) } }
