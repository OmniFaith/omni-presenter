/// A reversible envelope shared by all physical outputs. Stopping keeps the
/// last live frame available until the exit fade has completed.
#[derive(Default)]
pub struct PresentationTransition {
    target: bool,
    from: f32,
    started: f64,
    value: f32,
}

impl PresentationTransition {
    pub fn update(&mut self, active: bool, now: f64, duration_ms: u32) -> f32 {
        if self.target != active {
            self.from = self.value;
            self.started = now;
            self.target = active;
        }
        let target = if active { 1.0 } else { 0.0 };
        if duration_ms == 0 {
            self.value = target;
        } else {
            let t = ((now - self.started) / (duration_ms as f64 / 1000.0)).clamp(0.0, 1.0) as f32;
            let ease = t * t * (3.0 - 2.0 * t);
            self.value = self.from + (target - self.from) * ease;
            if t >= 1.0 {
                self.value = target;
            }
        }
        self.value
    }

    pub fn moving(&self) -> bool {
        self.value != if self.target { 1.0 } else { 0.0 }
    }

    pub fn visible(&self) -> bool {
        self.value > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entering_exiting_and_reversing_keep_opacity_continuous_and_finish() {
        let mut fade = PresentationTransition::default();
        assert_eq!(fade.update(false, 0.0, 400), 0.0);
        assert_eq!(fade.update(true, 1.0, 400), 0.0);
        assert!((fade.update(true, 1.2, 400) - 0.5).abs() < 0.001);
        assert_eq!(fade.update(true, 1.5, 400), 1.0);
        assert!(!fade.moving());
        assert_eq!(fade.update(false, 2.0, 400), 1.0);
        let middle = fade.update(false, 2.2, 400);
        assert!((middle - 0.5).abs() < 0.001);
        assert_eq!(fade.update(true, 2.2, 400), middle);
        assert_eq!(fade.update(true, 2.7, 400), 1.0);
        assert_eq!(fade.update(false, 3.0, 0), 0.0);
        assert!(!fade.moving());
        assert_eq!(fade.update(true, 3.0, 0), 1.0);
    }
}
