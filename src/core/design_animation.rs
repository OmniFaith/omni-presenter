use super::design::{DesignLayer, new_id};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationEffect {
    #[default]
    None,
    Fade,
    SlideLeft,
    SlideRight,
    SlideUp,
    SlideDown,
    Zoom,
    ZoomOut,
    Pop,
    Float,
    Wipe,
}

impl AnimationEffect {
    pub const ALL: [Self; 11] = [
        Self::None,
        Self::Fade,
        Self::SlideLeft,
        Self::SlideRight,
        Self::SlideUp,
        Self::SlideDown,
        Self::Zoom,
        Self::ZoomOut,
        Self::Pop,
        Self::Float,
        Self::Wipe,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "design-none",
            Self::Fade => "design-dissolve-fade",
            Self::SlideLeft => "design-slide-left",
            Self::SlideRight => "design-slide-right",
            Self::SlideUp => "design-slide-up",
            Self::SlideDown => "design-slide-down",
            Self::Zoom => "design-zoom-in",
            Self::ZoomOut => "design-zoom-out",
            Self::Pop => "design-pop",
            Self::Float => "design-float",
            Self::Wipe => "design-wipe",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LayerAnimation {
    pub in_effect: AnimationEffect,
    pub in_duration_ms: u32,
    pub in_delay_ms: u32,
    pub out_effect: AnimationEffect,
    pub out_duration_ms: u32,
    pub out_delay_ms: u32,
    pub easing: String,
    pub start_ms: u32,
    pub end_ms: u32,
}

impl Default for LayerAnimation {
    fn default() -> Self {
        Self {
            in_effect: AnimationEffect::None,
            in_duration_ms: 500,
            in_delay_ms: 0,
            out_effect: AnimationEffect::None,
            out_duration_ms: 500,
            out_delay_ms: 0,
            easing: "smooth".into(),
            start_ms: 0,
            end_ms: 0,
        }
    }
}

impl LayerAnimation {
    pub fn is_animated(&self) -> bool {
        self.in_effect != AnimationEffect::None
            || self.out_effect != AnimationEffect::None
            || self.start_ms > 0
            || self.end_ms > 0
    }

    pub fn ease(progress: f32, easing: &str) -> f32 {
        let p = progress.clamp(0.0, 1.0);
        match easing {
            "linear" => p,
            "ease_in" => p * p,
            "ease_out" => p * (2.0 - p),
            "ease_in_out" => {
                if p < 0.5 {
                    2.0 * p * p
                } else {
                    -1.0 + (4.0 - 2.0 * p) * p
                }
            }
            _ => p * p * (3.0 - 2.0 * p),
        }
    }

    pub fn apply_at(
        &self,
        base_rect: [f32; 4],
        base_opacity: f32,
        elapsed_ms: u64,
        slide_duration_ms: u64,
    ) -> ([f32; 4], f32) {
        if !self.is_animated() {
            return (base_rect, base_opacity);
        }

        let start = (self.start_ms.max(self.in_delay_ms) as u64).min(slide_duration_ms);
        let effective_end = if self.end_ms == 0 || (self.end_ms as u64) > slide_duration_ms {
            slide_duration_ms
        } else {
            (self.end_ms as u64).max(start)
        };

        // If outside active lifetime window, layer is not visible
        if elapsed_ms < start {
            return (base_rect, 0.0);
        }
        if effective_end > 0 && elapsed_ms >= effective_end {
            return (base_rect, 0.0);
        }

        let mut rect = base_rect;
        let mut opacity = base_opacity;

        // Entrance effect
        if self.in_effect != AnimationEffect::None {
            let dur = (self.in_duration_ms as u64).max(1).min(effective_end.saturating_sub(start).max(1));
            if elapsed_ms < start + dur {
                let raw_progress = (elapsed_ms - start) as f32 / dur as f32;
                let p = Self::ease(raw_progress, &self.easing);
                let inv = 1.0 - p;

                match self.in_effect {
                    AnimationEffect::Fade => {
                        opacity = base_opacity * p;
                    }
                    AnimationEffect::SlideLeft => {
                        rect[0] = base_rect[0] + inv * 0.35;
                        opacity = base_opacity * p;
                    }
                    AnimationEffect::SlideRight => {
                        rect[0] = base_rect[0] - inv * 0.35;
                        opacity = base_opacity * p;
                    }
                    AnimationEffect::SlideUp => {
                        rect[1] = base_rect[1] + inv * 0.25;
                        opacity = base_opacity * p;
                    }
                    AnimationEffect::SlideDown => {
                        rect[1] = base_rect[1] - inv * 0.25;
                        opacity = base_opacity * p;
                    }
                    AnimationEffect::Zoom => {
                        let scale = 0.5 + 0.5 * p;
                        let cx = base_rect[0] + base_rect[2] * 0.5;
                        let cy = base_rect[1] + base_rect[3] * 0.5;
                        let nw = base_rect[2] * scale;
                        let nh = base_rect[3] * scale;
                        rect = [cx - nw * 0.5, cy - nh * 0.5, nw, nh];
                        opacity = base_opacity * p;
                    }
                    AnimationEffect::ZoomOut => {
                        let scale = 1.4 - 0.4 * p;
                        let cx = base_rect[0] + base_rect[2] * 0.5;
                        let cy = base_rect[1] + base_rect[3] * 0.5;
                        let nw = base_rect[2] * scale;
                        let nh = base_rect[3] * scale;
                        rect = [cx - nw * 0.5, cy - nh * 0.5, nw, nh];
                        opacity = base_opacity * p;
                    }
                    AnimationEffect::Pop => {
                        let scale = if p < 0.7 {
                            0.3 + 0.9 * (p / 0.7)
                        } else {
                            1.2 - 0.2 * ((p - 0.7) / 0.3)
                        };
                        let cx = base_rect[0] + base_rect[2] * 0.5;
                        let cy = base_rect[1] + base_rect[3] * 0.5;
                        let nw = base_rect[2] * scale;
                        let nh = base_rect[3] * scale;
                        rect = [cx - nw * 0.5, cy - nh * 0.5, nw, nh];
                        opacity = base_opacity * p.min(1.0);
                    }
                    AnimationEffect::Float => {
                        rect[1] = base_rect[1] + inv * 0.12;
                        opacity = base_opacity * p;
                    }
                    AnimationEffect::Wipe => {
                        rect[2] = base_rect[2] * p;
                        opacity = base_opacity * p.clamp(0.2, 1.0);
                    }
                    AnimationEffect::None => {}
                }
            }
        }

        // Exit effect
        if self.out_effect != AnimationEffect::None && effective_end > start {
            let out_dur = (self.out_duration_ms as u64).max(1).min(effective_end.saturating_sub(start).max(1));
            let out_start = effective_end
                .saturating_sub(out_dur)
                .saturating_sub(self.out_delay_ms as u64)
                .max(start);
            if elapsed_ms >= out_start && elapsed_ms < effective_end {
                let raw_progress = (elapsed_ms - out_start) as f32 / out_dur as f32;
                let p = Self::ease(raw_progress, &self.easing);

                match self.out_effect {
                    AnimationEffect::Fade => {
                        opacity *= 1.0 - p;
                    }
                    AnimationEffect::SlideLeft => {
                        rect[0] -= p * 0.35;
                        opacity *= 1.0 - p;
                    }
                    AnimationEffect::SlideRight => {
                        rect[0] += p * 0.35;
                        opacity *= 1.0 - p;
                    }
                    AnimationEffect::SlideUp => {
                        rect[1] -= p * 0.25;
                        opacity *= 1.0 - p;
                    }
                    AnimationEffect::SlideDown => {
                        rect[1] += p * 0.25;
                        opacity *= 1.0 - p;
                    }
                    AnimationEffect::Zoom | AnimationEffect::ZoomOut => {
                        let scale = 1.0 - 0.4 * p;
                        let cx = rect[0] + rect[2] * 0.5;
                        let cy = rect[1] + rect[3] * 0.5;
                        let nw = rect[2] * scale;
                        let nh = rect[3] * scale;
                        rect = [cx - nw * 0.5, cy - nh * 0.5, nw, nh];
                        opacity *= 1.0 - p;
                    }
                    AnimationEffect::Pop => {
                        let scale = 1.0 - 0.6 * p;
                        let cx = rect[0] + rect[2] * 0.5;
                        let cy = rect[1] + rect[3] * 0.5;
                        let nw = rect[2] * scale;
                        let nh = rect[3] * scale;
                        rect = [cx - nw * 0.5, cy - nh * 0.5, nw, nh];
                        opacity *= 1.0 - p;
                    }
                    AnimationEffect::Float => {
                        rect[1] += p * 0.12;
                        opacity *= 1.0 - p;
                    }
                    AnimationEffect::Wipe => {
                        rect[2] = base_rect[2] * (1.0 - p);
                        opacity *= 1.0 - p;
                    }
                    AnimationEffect::None => {}
                }
            }
        }

        (rect, opacity.clamp(0.0, 1.0))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesignSlide {
    pub id: String,
    pub name: String,
    pub layers: Vec<DesignLayer>,
    #[serde(default = "default_slide_duration")]
    pub duration_secs: f32,
    #[serde(default = "default_slide_transition")]
    pub transition: String,
    #[serde(default = "default_slide_transition_ms")]
    pub transition_ms: u32,
}

pub fn default_slide_duration() -> f32 {
    5.0
}

fn default_slide_transition() -> String {
    "fade".into()
}

fn default_slide_transition_ms() -> u32 {
    350
}

impl Default for DesignSlide {
    fn default() -> Self {
        Self {
            id: new_id(),
            name: "Slide 1".into(),
            layers: Vec::new(),
            duration_secs: 5.0,
            transition: "fade".into(),
            transition_ms: 350,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_ease_and_interpolation() {
        let anim = LayerAnimation {
            in_effect: AnimationEffect::Fade,
            in_duration_ms: 1000,
            in_delay_ms: 200,
            out_effect: AnimationEffect::None,
            out_duration_ms: 500,
            out_delay_ms: 0,
            easing: "linear".into(),
            start_ms: 0,
            end_ms: 0,
        };

        let base_rect = [0.1, 0.2, 0.5, 0.3];
        let base_opacity = 1.0;

        // Before in_delay
        let (_, op) = anim.apply_at(base_rect, base_opacity, 100, 5000);
        assert_eq!(op, 0.0);

        // Halfway through entrance
        let (_, op) = anim.apply_at(base_rect, base_opacity, 700, 5000);
        assert!((op - 0.5).abs() < 0.05);

        // After entrance
        let (r, op) = anim.apply_at(base_rect, base_opacity, 1500, 5000);
        assert_eq!(op, 1.0);
        assert_eq!(r, base_rect);
    }

    #[test]
    fn timeline_window_start_and_end_trimming() {
        let anim = LayerAnimation {
            in_effect: AnimationEffect::None,
            in_duration_ms: 0,
            in_delay_ms: 0,
            out_effect: AnimationEffect::None,
            out_duration_ms: 0,
            out_delay_ms: 0,
            easing: "linear".into(),
            start_ms: 1000,
            end_ms: 3000,
        };

        let base_rect = [0.0, 0.0, 1.0, 1.0];
        let base_opacity = 1.0;

        // Before start_ms: hidden
        let (_, op) = anim.apply_at(base_rect, base_opacity, 500, 5000);
        assert_eq!(op, 0.0);

        // Within active window: visible
        let (_, op) = anim.apply_at(base_rect, base_opacity, 2000, 5000);
        assert_eq!(op, 1.0);

        // After end_ms: hidden
        let (_, op) = anim.apply_at(base_rect, base_opacity, 3500, 5000);
        assert_eq!(op, 0.0);
    }
}
