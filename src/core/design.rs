//! Resolution-independent scenes shared by the editor, projector and browser.
use serde::{Deserialize, Serialize};
pub use super::design_animation::{
    default_slide_duration, AnimationEffect, DesignSlide, LayerAnimation,
};
pub use super::design_style::{ShapeKind,LayerStroke,LayerShadow,DesignTextStyle,LayerBackground};

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn new_id() -> String {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{:x}-{:x}",
        now_ms(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,Lighten,ColorDodge,ColorBurn,HardLight,SoftLight,Difference,Exclusion,
}
impl BlendMode {
    pub const ALL: [Self; 12] = [Self::Normal, Self::Multiply, Self::Screen, Self::Overlay,Self::Darken,Self::Lighten,Self::ColorDodge,Self::ColorBurn,Self::HardLight,Self::SoftLight,Self::Difference,Self::Exclusion];
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "font-normal",
            Self::Multiply => "design-multiply",
            Self::Screen => "design-screen",
            Self::Overlay => "design-overlay",
            Self::Darken => "design-darken",
            Self::Lighten => "design-lighten",
            Self::ColorDodge => "design-color-dodge",
            Self::ColorBurn => "design-color-burn",
            Self::HardLight => "design-hard-light",
            Self::SoftLight => "design-soft-light",
            Self::Difference => "design-difference",
            Self::Exclusion => "design-exclusion",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Normal => "design-keeps-the-original-colors",
            Self::Multiply => "design-darkens-and-combines-colors",
            Self::Screen => "design-combines-light-and-brightens",
            Self::Overlay => "design-contrast-between-light-and-shadow",
            Self::Darken => "design-preserves-the-darkest-tones",
            Self::Lighten => "design-preserves-the-lightest-tones",
            Self::ColorDodge => "design-intensifies-bright-areas",
            Self::ColorBurn => "design-intensifies-dark-areas",
            Self::HardLight => "design-light-with-strong-contrast",
            Self::SoftLight => "design-soft-lighting-and-contrast",
            Self::Difference => "design-highlights-the-difference-between-colors",
            Self::Exclusion => "design-difference-with-reduced-contrast",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Countdown {
    pub duration_secs: u32,
    pub remaining_ms: u64,
    pub started_at: Option<u64>,
    pub finished_text: String,
    #[serde(default = "auto_start")]
    pub auto_start: bool,
}
fn auto_start() -> bool {
    true
}
impl Default for Countdown {
    fn default() -> Self {
        Self {
            duration_secs: 300,
            remaining_ms: 300_000,
            started_at: None,
            finished_text: "design-let-s-begin".into(),
            auto_start: true,
        }
    }
}
impl Countdown {
    pub fn remaining(&self, now: u64) -> u64 {
        self.remaining_ms
            .saturating_sub(self.started_at.map_or(0, |start| now.saturating_sub(start)))
    }
    pub fn start(&mut self, now: u64) {
        if self.started_at.is_none() && self.remaining_ms > 0 {
            self.started_at = Some(now);
        }
    }
    pub fn pause(&mut self, now: u64) {
        self.remaining_ms = self.remaining(now);
        self.started_at = None;
    }
    pub fn reset(&mut self) {
        self.started_at = None;
        self.remaining_ms = u64::from(self.duration_secs) * 1000;
    }
    pub fn text(&self, now: u64) -> String {
        let remaining = self.remaining(now);
        if remaining == 0 {
            return self.finished_text.clone();
        }
        let seconds = remaining.div_ceil(1000);
        if seconds >= 3600 {
            format!(
                "{:02}:{:02}:{:02}",
                seconds / 3600,
                seconds / 60 % 60,
                seconds % 60
            )
        } else {
            format!("{:02}:{:02}", seconds / 60, seconds % 60)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Question {
    pub question: String,
    pub answer: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LayerContent {
    Vector { svg: String },
    Plugin(Box<crate::core::plugins::PluginInstance>),
    Text(String),
    Image {
        path: String,
        cover: bool,
    },
    Shape {
        radius: f32,
    },
    Clock {
        seconds: bool,
        date: bool,
    },
    Countdown(Countdown),
    Questions {
        items: Vec<Question>,
        current: usize,
        revealed: bool,
    },
}
impl LayerContent {
    pub fn label(&self) -> &str {
        match self {
            Self::Vector {..} => "SVG",
            Self::Plugin(instance) => &instance.definition.name,
            Self::Text(_) => "design-text",
            Self::Image { .. } => "common-image",
            Self::Shape { .. } => "design-shape",
            Self::Clock { .. } => "design-clock",
            Self::Countdown(_) => "design-countdown",
            Self::Questions { .. } => "design-questions-and-answers",
        }
    }
    pub fn text_at(&self, now: u64) -> Option<String> {
        match self {
            Self::Text(text) => Some(text.clone()),
            Self::Clock { seconds, date } => {
                let time =
                    chrono::DateTime::from_timestamp_millis(now.min(i64::MAX as u64) as i64)?
                        .with_timezone(&chrono::Local);
                Some(
                    time.format(if *date && *seconds {
                        "%d/%m/%Y\n%H:%M:%S"
                    } else if *date {
                        "%d/%m/%Y\n%H:%M"
                    } else if *seconds {
                        "%H:%M:%S"
                    } else {
                        "%H:%M"
                    })
                    .to_string(),
                )
            }
            Self::Countdown(timer) => Some(timer.text(now)),
            Self::Questions {
                items,
                current,
                revealed,
            } => items.get(*current).map(|item| {
                if *revealed {
                    format!("{}\n\n{}", item.question, item.answer)
                } else {
                    item.question.clone()
                }
            }),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesignLayer {
    pub background: LayerBackground,
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub rect: [f32; 4],
    pub opacity: f32,
    pub blend: BlendMode,
    pub color: [u8; 4],
    pub font_family: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<u16>,
    pub font_size: f32,
    pub bold: bool,
    pub align: String,
    pub content: LayerContent,
    #[serde(default)] pub rotation:f32,
    #[serde(default)] pub blur:f32,
    #[serde(default)] pub stroke:LayerStroke,
    #[serde(default)] pub shadow:LayerShadow,
    #[serde(default)] pub shape:ShapeKind,
    #[serde(default="default_line_width")] pub line_width:f32,
    #[serde(default)] pub text_style:DesignTextStyle,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
    #[serde(default)]
    pub animation: LayerAnimation,
}
fn default_line_width()->f32{4.0}
impl Default for DesignLayer {
    fn default() -> Self { Self::new(LayerContent::Text(String::new())) }
}
impl DesignLayer {
    pub fn effective_font_weight(&self) -> u16 {
        self.font_weight
            .unwrap_or(if self.bold { 700 } else { 400 })
            .clamp(100, 900)
    }

    pub fn set_font_weight(&mut self, weight: u16) {
        let weight = weight.clamp(100, 900);
        self.font_weight = Some(weight);
        self.bold = weight >= 600;
    }

    pub fn new(content: LayerContent) -> Self {
        Self {
            background: LayerBackground::default(),
            id: new_id(),
            name: content.label().into(),
            visible: true,
            locked: false,
            rect: [0.1, 0.32, 0.8, 0.3],
            opacity: 1.0,
            blend: BlendMode::Normal,
            color: [255, 255, 255, 255],
            font_family: "Proportional".into(),
            font_weight: None,
            font_size: 64.0,
            bold: false,
            align: "center".into(),
            content,
            rotation:0.0,blur:0.0,stroke:LayerStroke::default(),shadow:LayerShadow::default(),shape:ShapeKind::Rectangle,line_width:4.0,text_style:DesignTextStyle::default(),
            group_id: None,
            group_name: None,
            animation: LayerAnimation::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesignDocument {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub folder: String,
    pub width: u32,
    pub height: u32,
    pub background: [u8; 4],
    #[serde(default = "transparent_background")]
    pub background_mode: String,
    #[serde(default)]
    pub theme_id: Option<String>,
    #[serde(default)]
    pub overlay: bool,
    /// Bottom-to-top stacking order.
    pub layers: Vec<DesignLayer>,
    /// Transition effect type (e.g. "fade", "slide_left", "slide_right", "slide_up", "slide_down", "zoom", "zoom_out", "wipe", "cut").
    #[serde(default = "default_design_transition")]
    pub transition: String,
    /// Transition duration in milliseconds.
    #[serde(default = "default_transition_ms")]
    pub transition_ms: u32,
    /// Transition strength/intensity.
    #[serde(default = "default_transition_strength")]
    pub transition_strength: f32,
    /// Transition easing curve.
    #[serde(default = "default_transition_easing")]
    pub transition_easing: String,
    /// Multiple slides support.
    #[serde(default)]
    pub slides: Vec<DesignSlide>,
    /// Index of currently selected slide.
    #[serde(default)]
    pub active_slide_index: usize,
    /// Default slide duration in seconds (for presentations/animations).
    #[serde(default = "default_slide_duration")]
    pub slide_duration_secs: f32,
}
fn default_design_transition() -> String {
    "fade".into()
}
fn default_transition_ms() -> u32 {
    350
}
fn default_transition_strength() -> f32 {
    12.0
}
fn default_transition_easing() -> String {
    "smooth".into()
}
fn transparent_background() -> String {
    "transparent".into()
}
impl Default for DesignDocument {
    fn default() -> Self {
        Self {
            id: new_id(),
            name: "design-new-blank-design".into(),
            folder: String::new(),
            width: 1280,
            height: 720,
            background: [16, 24, 42, 255],
            background_mode: "transparent".into(),
            theme_id: None,
            overlay: false,
            transition: "fade".into(),
            transition_ms: 350,
            transition_strength: 12.0,
            transition_easing: "smooth".into(),
            layers: Vec::new(),
            slides: Vec::new(),
            active_slide_index: 0,
            slide_duration_secs: 5.0,
        }
    }
}
impl DesignDocument {
    pub fn template_localized(kind: usize, language: crate::core::i18n::Language) -> Self {
        let mut scene = Self::template(kind);
        scene.name = language.text(&scene.name).to_owned();
        for layer in &mut scene.layers {
            layer.name = language.text(&layer.name).to_owned();
            match &mut layer.content {
                LayerContent::Text(text) => *text = language.text(text).to_owned(),
                LayerContent::Countdown(timer) => timer.finished_text = language.text(&timer.finished_text).to_owned(),
                LayerContent::Questions { items, .. } => for item in items {
                    item.question = language.text(&item.question).to_owned();
                    item.answer = language.text(&item.answer).to_owned();
                },
                _ => {}
            }
        }
        scene
    }

    /// Resolve inherited families on a render copy so saved layers keep following the theme.
    pub fn resolve_theme_fonts(
        &self,
        theme: Option<&crate::core::models::PresentationTheme>,
    ) -> std::borrow::Cow<'_, Self> {
        let inherited = crate::core::models::fonts::THEME_FONT;
        if !self.layers.iter().any(|layer| layer.font_family == inherited) {
            return std::borrow::Cow::Borrowed(self);
        }
        let family = theme
            .map(|theme| theme.font_family.as_str())
            .filter(|family| !family.is_empty() && *family != inherited)
            .unwrap_or("Proportional");
        let mut scene = self.clone();
        for layer in &mut scene.layers {
            if layer.font_family == inherited {
                layer.font_family = family.to_owned();
            }
        }
        std::borrow::Cow::Owned(scene)
    }

    pub fn ensure_slides(&mut self) {
        if self.slides.is_empty() {
            self.slides.push(DesignSlide {
                id: new_id(),
                name: self.name.clone(),
                layers: self.layers.clone(),
                duration_secs: if self.slide_duration_secs > 0.0 { self.slide_duration_secs } else { 5.0 },
                transition: self.transition.clone(),
                transition_ms: self.transition_ms,
            });
            self.active_slide_index = 0;
        } else {
            if self.active_slide_index >= self.slides.len() {
                self.active_slide_index = self.slides.len().saturating_sub(1);
            }
            if self.layers.is_empty() && !self.slides[self.active_slide_index].layers.is_empty() {
                self.layers = self.slides[self.active_slide_index].layers.clone();
            }
        }
    }

    pub fn sync_active_slide_from_layers(&mut self) {
        self.ensure_slides();
        let idx = self.active_slide_index;
        if let Some(slide) = self.slides.get_mut(idx) {
            slide.layers = self.layers.clone();
        }
    }

    pub fn set_active_slide(&mut self, index: usize) {
        self.ensure_slides();
        if index < self.slides.len() {
            let prev = self.active_slide_index;
            if let Some(slide) = self.slides.get_mut(prev) {
                slide.layers = self.layers.clone();
            }
            self.active_slide_index = index;
            self.layers = self.slides[index].layers.clone();
        }
    }

    pub fn add_slide(&mut self) -> usize {
        self.ensure_slides();
        let prev = self.active_slide_index;
        if let Some(slide) = self.slides.get_mut(prev) {
            slide.layers = self.layers.clone();
        }
        let new_idx = self.slides.len();
        self.slides.push(DesignSlide {
            id: new_id(),
            name: format!("Slide {}", new_idx + 1),
            layers: Vec::new(),
            duration_secs: self.slide_duration_secs.max(1.0),
            transition: self.transition.clone(),
            transition_ms: self.transition_ms,
        });
        self.set_active_slide(new_idx);
        new_idx
    }

    pub fn duplicate_current_slide(&mut self) -> usize {
        self.ensure_slides();
        let curr = self.active_slide_index;
        if let Some(slide) = self.slides.get_mut(curr) {
            slide.layers = self.layers.clone();
        }
        let mut clone = self.slides[curr].clone();
        clone.id = new_id();
        clone.name = format!("{} (2)", clone.name);
        for layer in &mut clone.layers {
            layer.id = new_id();
        }
        let insert_idx = curr + 1;
        self.slides.insert(insert_idx, clone);
        self.set_active_slide(insert_idx);
        insert_idx
    }

    pub fn delete_slide(&mut self, index: usize) -> bool {
        self.ensure_slides();
        if self.slides.len() <= 1 {
            return false;
        }
        if index < self.slides.len() {
            self.slides.remove(index);
            let next_idx = if index >= self.slides.len() {
                self.slides.len() - 1
            } else {
                index
            };
            self.active_slide_index = next_idx;
            self.layers = self.slides[next_idx].layers.clone();
            return true;
        }
        false
    }

    pub fn active_slide_duration_ms(&self) -> u64 {
        let secs = self.slides.get(self.active_slide_index)
            .map(|s| s.duration_secs)
            .unwrap_or(self.slide_duration_secs);
        (secs.max(0.5) * 1000.0) as u64
    }

    pub fn animated_layers_at(&self, elapsed_ms: u64) -> Vec<DesignLayer> {
        let slide_duration_ms = self.active_slide_duration_ms();
        let mut result = self.layers.clone();
        for layer in &mut result {
            if layer.animation.is_animated() {
                let (anim_rect, anim_opacity) = layer.animation.apply_at(
                    layer.rect,
                    layer.opacity,
                    elapsed_ms,
                    slide_duration_ms,
                );
                layer.rect = anim_rect;
                layer.opacity = anim_opacity;
            }
        }
        result
    }

    pub fn has_layer_animations(&self) -> bool {
        self.layers.iter().any(|l| l.animation.is_animated())
    }

    pub fn start_timers(&mut self) {
        for layer in &mut self.layers {
            if let LayerContent::Countdown(timer) = &mut layer.content
                && timer.auto_start
            {
                timer.start(now_ms());
            }
        }
    }
    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height.max(1) as f32
    }
    pub fn is_dynamic(&self) -> bool {
        self.has_layer_animations()
            || self.layers.iter().any(|layer| {
                layer.visible
                    && matches!(
                        layer.content,
                        LayerContent::Plugin(ref instance) if instance.definition.animated
                    ) || layer.visible && matches!(
                        layer.content,
                        LayerContent::Clock { .. }
                            | LayerContent::Countdown(Countdown {
                                started_at: Some(_),
                                ..
                            })
                    )
            })
    }
    pub fn template(kind: usize) -> Self {
        let mut scene = Self::default();
        if kind == 0 {
            return scene;
        }
        let lang = crate::core::i18n::Language::system();
        if kind == 4 || kind == 5 {
            scene.overlay = true;
            if kind == 4 {
                scene.name = lang.text("design-clock-overlay-label").to_string();
                let mut pill = DesignLayer::new(LayerContent::Shape { radius: 18.0 });
                pill.name = lang.text("design-clock-background").to_string();
                pill.rect = [0.78, 0.035, 0.185, 0.075];
                pill.color = [15, 23, 42, 225];
                pill.stroke = LayerStroke { enabled: true, width: 1.2, color: [255, 255, 255, 50] };
                pill.shadow = LayerShadow { enabled: true, blur: 20.0, x: 0.0, y: 5.0, color: [0, 0, 0, 180] };
                scene.layers.push(pill);

                let mut clock = DesignLayer::new(LayerContent::Clock {
                    seconds: false,
                    date: false,
                });
                clock.name = lang.text("design-clock-time").to_string();
                clock.rect = [0.78, 0.035, 0.185, 0.075];
                clock.font_size = 32.0;
                clock.set_font_weight(700);
                clock.color = [255, 255, 255, 255];
                clock.align = "center".into();
                clock.text_style.letter_spacing = 2.0;
                clock.shadow = LayerShadow { enabled: true, blur: 12.0, x: 0.0, y: 2.0, color: [0, 0, 0, 160] };
                scene.layers.push(clock);
            } else {
                scene.name = lang.text("design-lower-third-overlay").to_string();
                let mut plate = DesignLayer::new(LayerContent::Shape { radius: 16.0 });
                plate.name = lang.text("design-background-banner").to_string();
                plate.rect = [0.05, 0.78, 0.90, 0.16];
                plate.color = [15, 23, 42, 235];
                plate.stroke = LayerStroke { enabled: true, width: 1.2, color: [255, 255, 255, 45] };
                plate.shadow = LayerShadow { enabled: true, blur: 28.0, x: 0.0, y: 8.0, color: [0, 0, 0, 200] };
                scene.layers.push(plate);

                let mut accent_bar = DesignLayer::new(LayerContent::Shape { radius: 4.0 });
                accent_bar.name = lang.text("design-accent-bar").to_string();
                accent_bar.rect = [0.05, 0.78, 0.008, 0.16];
                accent_bar.color = [59, 130, 246, 255];
                scene.layers.push(accent_bar);

                let mut badge = DesignLayer::new(LayerContent::Shape { radius: 8.0 });
                badge.name = lang.text("design-badge").to_string();
                badge.rect = [0.075, 0.805, 0.10, 0.034];
                badge.color = [59, 130, 246, 45];
                badge.stroke = LayerStroke { enabled: true, width: 1.0, color: [96, 165, 250, 140] };
                scene.layers.push(badge);

                let mut badge_text = DesignLayer::new(LayerContent::Text(lang.text("design-ministry-tag").to_string()));
                badge_text.name = lang.text("design-category").to_string();
                badge_text.rect = [0.075, 0.805, 0.10, 0.034];
                badge_text.font_size = 12.0;
                badge_text.set_font_weight(700);
                badge_text.color = [147, 197, 253, 255];
                badge_text.align = "center".into();
                badge_text.text_style.letter_spacing = 2.0;
                scene.layers.push(badge_text);

                let mut title = DesignLayer::new(LayerContent::Text(lang.text("design-speaker-title").to_string()));
                title.name = lang.text("design-notice-title").to_string();
                title.rect = [0.075, 0.846, 0.85, 0.044];
                title.font_size = 24.0;
                title.set_font_weight(700);
                title.color = [255, 255, 255, 255];
                title.align = "left".into();
                scene.layers.push(title);

                let mut sub = DesignLayer::new(LayerContent::Text(lang.text("design-speaker-subtitle").to_string()));
                sub.name = lang.text("design-notice-subtitle").to_string();
                sub.rect = [0.075, 0.895, 0.85, 0.034];
                sub.font_size = 16.0;
                sub.set_font_weight(400);
                sub.color = [203, 213, 225, 255];
                sub.align = "left".into();
                scene.layers.push(sub);
            }
            return scene;
        }
        scene.background_mode = "color".into();
        scene.background = [11, 17, 32, 255];

        if kind == 1 {
            scene.name = lang.text("design-welcome-label").to_string();

            let mut card = DesignLayer::new(LayerContent::Shape { radius: 28.0 });
            card.name = lang.text("design-background-card").to_string();
            card.rect = [0.10, 0.12, 0.80, 0.76];
            card.color = [23, 32, 54, 230];
            card.stroke = LayerStroke { enabled: true, width: 1.5, color: [255, 255, 255, 35] };
            card.shadow = LayerShadow { enabled: true, blur: 36.0, x: 0.0, y: 14.0, color: [0, 0, 0, 180] };
            scene.layers.push(card);

            let mut pill = DesignLayer::new(LayerContent::Shape { radius: 12.0 });
            pill.name = lang.text("design-header-pill").to_string();
            pill.rect = [0.38, 0.19, 0.24, 0.055];
            pill.color = [245, 158, 11, 35];
            pill.stroke = LayerStroke { enabled: true, width: 1.0, color: [251, 191, 36, 120] };
            scene.layers.push(pill);

            let mut tag = DesignLayer::new(LayerContent::Text(lang.text("design-welcome").to_string()));
            tag.name = lang.text("design-tag").to_string();
            tag.rect = [0.38, 0.19, 0.24, 0.055];
            tag.font_size = 13.0;
            tag.set_font_weight(700);
            tag.color = [253, 230, 138, 255];
            tag.align = "center".into();
            tag.text_style.letter_spacing = 3.5;
            scene.layers.push(tag);

            let mut heading = DesignLayer::new(LayerContent::Text(lang.text("design-welcome-heading").to_string()));
            heading.name = lang.text("design-main-message").to_string();
            heading.rect = [0.15, 0.31, 0.70, 0.22];
            heading.font_size = 56.0;
            heading.set_font_weight(800);
            heading.color = [255, 255, 255, 255];
            heading.align = "center".into();
            heading.text_style.line_height = 1.20;
            heading.text_style.wrap = true;
            heading.text_style.auto_fit = true;
            heading.shadow = LayerShadow { enabled: true, blur: 18.0, x: 0.0, y: 4.0, color: [0, 0, 0, 150] };
            scene.layers.push(heading);

            let mut sub = DesignLayer::new(LayerContent::Text(lang.text("design-welcome-verse").to_string()));
            sub.name = lang.text("design-subtitle").to_string();
            sub.rect = [0.16, 0.58, 0.68, 0.12];
            sub.font_size = 22.0;
            sub.set_font_weight(400);
            sub.color = [203, 213, 225, 255];
            sub.align = "center".into();
            sub.text_style.italic = true;
            sub.text_style.wrap = true;
            sub.text_style.auto_fit = true;
            scene.layers.push(sub);

            let mut footer = DesignLayer::new(LayerContent::Text(lang.text("design-welcome-footer").to_string()));
            footer.name = lang.text("design-welcome-footer-layer").to_string();
            footer.rect = [0.20, 0.74, 0.60, 0.06];
            footer.font_size = 15.0;
            footer.set_font_weight(500);
            footer.color = [148, 163, 184, 220];
            footer.align = "center".into();
            footer.text_style.letter_spacing = 1.0;
            scene.layers.push(footer);
        } else if kind == 2 {
            scene.name = lang.text("design-countdown").to_string();
            scene.background = [9, 13, 22, 255];

            let mut card = DesignLayer::new(LayerContent::Shape { radius: 30.0 });
            card.name = lang.text("design-central-card").to_string();
            card.rect = [0.16, 0.12, 0.68, 0.76];
            card.color = [20, 28, 46, 235];
            card.stroke = LayerStroke { enabled: true, width: 1.5, color: [56, 189, 248, 65] };
            card.shadow = LayerShadow { enabled: true, blur: 44.0, x: 0.0, y: 16.0, color: [0, 0, 0, 210] };
            scene.layers.push(card);

            let mut pill = DesignLayer::new(LayerContent::Shape { radius: 12.0 });
            pill.name = lang.text("design-header-pill").to_string();
            pill.rect = [0.34, 0.19, 0.32, 0.055];
            pill.color = [14, 165, 233, 35];
            pill.stroke = LayerStroke { enabled: true, width: 1.0, color: [56, 189, 248, 110] };
            scene.layers.push(pill);

            let mut tag = DesignLayer::new(LayerContent::Text(lang.text("design-starting-in").to_string()));
            tag.name = lang.text("design-tag").to_string();
            tag.rect = [0.34, 0.19, 0.32, 0.055];
            tag.font_size = 13.0;
            tag.set_font_weight(700);
            tag.color = [186, 230, 253, 255];
            tag.align = "center".into();
            tag.text_style.letter_spacing = 3.5;
            scene.layers.push(tag);

            let mut countdown = DesignLayer::new(LayerContent::Countdown(Countdown::default()));
            countdown.name = lang.text("design-countdown-timer").to_string();
            countdown.rect = [0.18, 0.30, 0.64, 0.34];
            countdown.font_size = 132.0;
            countdown.set_font_weight(800);
            countdown.color = [255, 255, 255, 255];
            countdown.align = "center".into();
            countdown.shadow = LayerShadow { enabled: true, blur: 28.0, x: 0.0, y: 8.0, color: [0, 0, 0, 200] };
            scene.layers.push(countdown);

            let mut sub = DesignLayer::new(LayerContent::Text(lang.text("design-prepare-heart").to_string()));
            sub.name = lang.text("design-guidance").to_string();
            sub.rect = [0.20, 0.67, 0.60, 0.08];
            sub.font_size = 22.0;
            sub.set_font_weight(600);
            sub.color = [241, 245, 249, 255];
            sub.align = "center".into();
            scene.layers.push(sub);

            let mut tip = DesignLayer::new(LayerContent::Text(lang.text("design-silence-phone").to_string()));
            tip.name = lang.text("design-secondary-guidance").to_string();
            tip.rect = [0.22, 0.76, 0.56, 0.05];
            tip.font_size = 15.0;
            tip.set_font_weight(400);
            tip.color = [148, 163, 184, 230];
            tip.align = "center".into();
            tip.text_style.letter_spacing = 0.5;
            scene.layers.push(tip);
        } else if kind == 3 {
            scene.name = lang.text("design-questions-and-answers").to_string();
            scene.background = [10, 15, 29, 255];

            let mut card = DesignLayer::new(LayerContent::Shape { radius: 28.0 });
            card.name = lang.text("design-questions-card").to_string();
            card.rect = [0.10, 0.12, 0.80, 0.76];
            card.color = [21, 30, 50, 235];
            card.stroke = LayerStroke { enabled: true, width: 1.5, color: [52, 211, 153, 55] };
            card.shadow = LayerShadow { enabled: true, blur: 36.0, x: 0.0, y: 12.0, color: [0, 0, 0, 180] };
            scene.layers.push(card);

            let mut pill = DesignLayer::new(LayerContent::Shape { radius: 12.0 });
            pill.name = lang.text("design-header-pill").to_string();
            pill.rect = [0.32, 0.18, 0.36, 0.055];
            pill.color = [16, 185, 129, 35];
            pill.stroke = LayerStroke { enabled: true, width: 1.0, color: [52, 211, 153, 110] };
            scene.layers.push(pill);

            let mut tag = DesignLayer::new(LayerContent::Text(lang.text("design-bible-quiz").to_string()));
            tag.name = lang.text("design-tag").to_string();
            tag.rect = [0.32, 0.18, 0.36, 0.055];
            tag.font_size = 13.0;
            tag.set_font_weight(700);
            tag.color = [167, 243, 208, 255];
            tag.align = "center".into();
            tag.text_style.letter_spacing = 3.0;
            scene.layers.push(tag);

            let mut tip = DesignLayer::new(LayerContent::Text(lang.text("design-press-space-to-reveal").to_string()));
            tip.name = lang.text("design-interaction-tip").to_string();
            tip.rect = [0.22, 0.77, 0.56, 0.05];
            tip.font_size = 14.0;
            tip.set_font_weight(500);
            tip.color = [148, 163, 184, 220];
            tip.align = "center".into();
            tip.text_style.letter_spacing = 0.5;
            scene.layers.push(tip);

            let mut questions = DesignLayer::new(LayerContent::Questions {
                items: vec![
                    Question {
                        question: lang.text("design-quiz-q1").to_string(),
                        answer: lang.text("design-quiz-a1").to_string(),
                    },
                    Question {
                        question: lang.text("design-quiz-q2").to_string(),
                        answer: lang.text("design-quiz-a2").to_string(),
                    },
                    Question {
                        question: lang.text("design-quiz-q3").to_string(),
                        answer: lang.text("design-quiz-a3").to_string(),
                    },
                ],
                current: 0,
                revealed: false,
            });
            questions.name = lang.text("design-questions-layer").to_string();
            questions.rect = [0.14, 0.28, 0.72, 0.46];
            questions.font_size = 40.0;
            questions.set_font_weight(600);
            questions.color = [255, 255, 255, 255];
            questions.align = "center".into();
            questions.text_style.line_height = 1.35;
            questions.text_style.wrap = true;
            questions.text_style.auto_fit = true;
            scene.layers.push(questions);
        }

        scene
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn theme_font_follows_theme_changes_without_overwriting_saved_layers() {
        let mut scene = DesignDocument::default();
        let mut inherited = DesignLayer::new(LayerContent::Text("Tema".into()));
        inherited.font_family = crate::core::models::fonts::THEME_FONT.into();
        inherited.set_font_weight(300);
        let mut custom = inherited.clone();
        custom.font_family = "Monospace".into();
        scene.layers = vec![inherited, custom];
        let mut theme = crate::core::models::PresentationTheme::default();
        theme.font_family = "Serif".into();
        assert_eq!(scene.resolve_theme_fonts(Some(&theme)).layers[0].font_family, "Serif");
        theme.font_family = "Montserrat".into();
        let updated = scene.resolve_theme_fonts(Some(&theme));
        assert_eq!(updated.layers[0].font_family, "Montserrat");
        assert_eq!(updated.layers[1].font_family, "Monospace");
        assert_eq!(updated.layers[0].effective_font_weight(), 300);
        assert_eq!(scene.layers[0].font_family, crate::core::models::fonts::THEME_FONT);
        assert_eq!(scene.resolve_theme_fonts(None).layers[0].font_family, "Proportional");
        let restored: DesignDocument = serde_json::from_str(&serde_json::to_string(&scene).unwrap()).unwrap();
        assert_eq!(restored, scene);
    }

    #[test]
    fn numeric_font_weight_keeps_legacy_bold_designs_compatible() {
        for (bold, weight) in [(false, 400), (true, 700)] {
            let mut layer: DesignLayer = serde_json::from_value(serde_json::json!({"bold": bold})).unwrap();
            assert_eq!(layer.effective_font_weight(), weight);
            layer.set_font_weight(200);
            assert!(!layer.bold);
            layer.set_font_weight(600);
            assert!(layer.bold);
            assert_eq!(layer.effective_font_weight(), 600);
            let restored: DesignLayer = serde_json::from_str(&serde_json::to_string(&layer).unwrap()).unwrap();
            assert_eq!(restored.effective_font_weight(), 600);
        }
    }

    #[test]
    fn countdown_pause_resume_reset_and_finish() {
        let mut timer = Countdown::default();
        timer.start(1000);
        assert_eq!(timer.text(61_000), "04:00");
        timer.pause(61_000);
        assert_eq!(timer.text(100_000), "04:00");
        timer.start(100_000);
        assert_eq!(timer.text(340_000), "design-let-s-begin");
        timer.reset();
        assert_eq!(timer.text(400_000), "05:00");
    }
    #[test]
    fn question_answer_visibility_and_serialization() {
        let mut scene = DesignDocument::template(3);
        let layer = scene.layers.last_mut().unwrap();
        let before = layer.content.text_at(0).unwrap();
        if let LayerContent::Questions { revealed, .. } = &mut layer.content {
            *revealed = true;
        }
        assert!(layer.content.text_at(0).unwrap().len() > before.len());
        let json = serde_json::to_string(&scene).unwrap();
        assert_eq!(
            serde_json::from_str::<DesignDocument>(&json).unwrap(),
            scene
        );
    }

    #[test]
    fn multi_slide_creation_and_layer_synchronization() {
        let mut doc = DesignDocument::default();
        doc.layers.push(DesignLayer::new(LayerContent::Text("Slide 1 Text".into())));
        doc.ensure_slides();
        assert_eq!(doc.slides.len(), 1);
        assert_eq!(doc.slides[0].layers.len(), 1);

        let new_idx = doc.add_slide();
        assert_eq!(new_idx, 1);
        assert_eq!(doc.active_slide_index, 1);
        assert_eq!(doc.layers.len(), 0);

        doc.layers.push(DesignLayer::new(LayerContent::Text("Slide 2 Text".into())));
        doc.sync_active_slide_from_layers();
        assert_eq!(doc.slides[1].layers.len(), 1);

        let dup_idx = doc.duplicate_current_slide();
        assert_eq!(dup_idx, 2);
        assert_eq!(doc.slides.len(), 3);
        assert_eq!(doc.slides[2].layers.len(), 1);

        assert!(doc.delete_slide(1));
        assert_eq!(doc.slides.len(), 2);
    }

    #[test]
    fn layer_animation_interpolation_in_document() {
        let mut doc = DesignDocument::default();
        let mut layer = DesignLayer::new(LayerContent::Text("Animated".into()));
        layer.animation.in_effect = AnimationEffect::Fade;
        layer.animation.in_duration_ms = 1000;
        layer.animation.in_delay_ms = 500;
        layer.opacity = 1.0;
        doc.layers.push(layer);

        assert!(doc.has_layer_animations());
        assert!(doc.is_dynamic());

        let layers_at_100 = doc.animated_layers_at(100);
        assert_eq!(layers_at_100[0].opacity, 0.0);

        let layers_at_1000 = doc.animated_layers_at(1000);
        assert!(layers_at_1000[0].opacity > 0.0 && layers_at_1000[0].opacity < 1.0);

        let layers_at_2000 = doc.animated_layers_at(2000);
        assert_eq!(layers_at_2000[0].opacity, 1.0);
    }
}
