#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PlayerMediaKind {
    Audio,
    Video,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActivePlayerMedia {
    pub title: String,
    pub path: String,
    pub kind: PlayerMediaKind,
    pub duration_secs: f32,
    pub is_playing: bool,
    pub is_projecting: bool,
    pub video_start_time: std::time::Instant,
    pub video_paused_at: Option<f32>,
}

impl ActivePlayerMedia {
    pub fn video_position(&self) -> f32 {
        let position = self.video_paused_at.unwrap_or_else(|| {
            if self.is_playing {
                self.video_start_time.elapsed().as_secs_f32()
            } else {
                0.0
            }
        });
        if self.duration_secs.is_finite() && self.duration_secs > 0.0 {
            position.clamp(0.0, self.duration_secs)
        } else {
            position.max(0.0)
        }
    }

    pub fn seek_video(&mut self, position: f32) {
        if !position.is_finite() {
            return;
        }
        let position = if self.duration_secs.is_finite() && self.duration_secs > 0.0 {
            position.clamp(0.0, self.duration_secs)
        } else {
            position.max(0.0)
        };
        if self.is_playing {
            if let Ok(duration) = std::time::Duration::try_from_secs_f32(position) {
                self.video_start_time = std::time::Instant::now()
                    .checked_sub(duration)
                    .unwrap_or_else(std::time::Instant::now);
            }
        } else {
            self.video_paused_at = Some(position);
        }
    }
}
