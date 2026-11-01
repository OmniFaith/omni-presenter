use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShaderOptions {
    pub custom: Option<crate::core::plugins::PluginShader>,
    pub kind: String,
    pub colors: [[u8; 3]; 3],
    pub speed: f32,
    pub scale: f32,
    pub intensity: f32,
    pub softness: f32,
    pub detail: f32,
    pub direction: f32,
    pub seed: f32,
    pub vignette: f32,
    pub grain: f32,
}

impl Default for ShaderOptions {
    fn default() -> Self {
        Self {
            custom: None,
            kind: "aurora".into(),
            colors: [[12, 22, 52], [29, 197, 174], [131, 82, 240]],
            speed: 0.35,
            scale: 1.0,
            intensity: 1.0,
            softness: 0.55,
            detail: 4.0,
            direction: 0.0,
            seed: 17.0,
            vignette: 0.3,
            grain: 0.015,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VideoOptions {
    pub path: String,
    pub speed: f32,
    pub repeat: bool,
    pub start: f32,
    pub mirror: bool,
    pub fit: String,
    pub saturation: f32,
    pub contrast: f32,
    pub paused: bool,
}

impl Default for VideoOptions {
    fn default() -> Self {
        Self {
            path: String::new(),
            speed: 1.0,
            repeat: true,
            start: 0.0,
            mirror: false,
            fit: "cover".into(),
            saturation: 1.0,
            contrast: 1.0,
            paused: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeOptions {
    pub transition_ms: u32,
    pub transition_strength: f32,
    pub transition_easing: String,
    pub auto_fit: bool,
    pub wrap_text: bool,
    pub text_opacity: f32,
    pub text_glow: f32,
    pub emphasis_color: [u8; 3],
    pub paragraph_spacing: f32,
    pub text_rotation: f32,
    pub background_overlay: f32,
    pub background_brightness: f32,
    pub gradient_angle: f32,
    pub video: VideoOptions,
    pub shader: ShaderOptions,
}

impl Default for ThemeOptions {
    fn default() -> Self {
        Self {
            transition_ms: 400,
            transition_strength: 12.0,
            transition_easing: "smooth".into(),
            auto_fit: true,
            wrap_text: true,
            text_opacity: 1.0,
            text_glow: 0.0,
            emphasis_color: [251, 191, 36],
            paragraph_spacing: 0.0,
            text_rotation: 0.0,
            background_overlay: 0.0,
            background_brightness: 1.0,
            gradient_angle: 0.0,
            video: VideoOptions::default(),
            shader: ShaderOptions::default(),
        }
    }
}
