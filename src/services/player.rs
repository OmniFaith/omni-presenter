//! Shared player operations for native widgets, keyboard commands and remote control.
use crate::core::state::{ActivePlayerMedia, ControlPanelState, PlayerMediaKind};

impl ControlPanelState {
    pub fn play_media(&mut self, title: String, path: String, kind: PlayerMediaKind, project: bool) {
        let language = self.settings.language;
        let (duration, playing, message) = match kind {
            PlayerMediaKind::Audio => {
                let result = self.audio_player.play_file(title.clone(), path.clone());
                let playing = result.is_ok();
                let message = match result {
                    Ok(()) => language.format("player-playing-audio", &[("title", &title)]),
                    Err(error) => language.format("player-audio-error", &[("error", &error)]),
                };
                let duration = self.audio_player.current_track().map_or(0.0, |track| track.duration_secs);
                (duration, playing, message)
            }
            PlayerMediaKind::Video => {
                self.audio_player.stop();
                (
                    crate::services::video::duration(&path).unwrap_or(0.0),
                    true,
                    language.format("player-playing-video", &[("title", &title)]),
                )
            }
        };
        self.active_player_media = Some(ActivePlayerMedia {
            title,
            path,
            kind,
            duration_secs: duration,
            is_playing: playing,
            is_projecting: project,
            video_start_time: std::time::Instant::now(),
            video_paused_at: None,
            video_fit: "contain".into(),
        });
        self.notify(message);
    }

    pub fn play_audio(&mut self, title: String, path: String) {
        self.play_media(title, path, PlayerMediaKind::Audio, false);
    }

    pub fn refresh_media_duration(&mut self) {
        if let Some(media) = &mut self.active_player_media
            && media.kind == PlayerMediaKind::Video
            && media.duration_secs <= 0.0
            && let Some(duration) = crate::services::video::duration(&media.path)
        {
            media.duration_secs = duration;
        }
    }

    pub fn close_player(&mut self) {
        self.audio_player.close();
        self.active_player_media = None;
    }

    pub fn toggle_player_playback(&mut self) {
        if let Some(media) = &mut self.active_player_media {
            match media.kind {
                PlayerMediaKind::Audio => {
                    self.audio_player.toggle_play_pause();
                    media.is_playing = self.audio_player.is_playing();
                }
                PlayerMediaKind::Video => {
                    if media.is_playing {
                        media.video_paused_at = Some(media.video_position());
                        media.is_playing = false;
                    } else {
                        let position = media.video_paused_at.take().unwrap_or(0.0);
                        media.is_playing = true;
                        media.seek_video(position);
                    }
                }
            }
        }
    }

    pub fn seek_player(&mut self, position: f32) {
        if !position.is_finite() {
            return;
        }
        if let Some(media) = &mut self.active_player_media {
            let position = if media.duration_secs > 0.0 {
                position.clamp(0.0, media.duration_secs)
            } else {
                position.max(0.0)
            };
            match media.kind {
                PlayerMediaKind::Audio => self.audio_player.seek(position),
                PlayerMediaKind::Video => media.seek_video(position),
            }
        }
    }

    pub fn player_position(&self) -> f32 {
        self.active_player_media.as_ref().map_or(0.0, |media| match media.kind {
            PlayerMediaKind::Audio => self.audio_player.position_secs(),
            PlayerMediaKind::Video => media.video_position(),
        })
    }

    pub fn stop_player(&mut self) {
        if let Some(media) = &mut self.active_player_media {
            match media.kind {
                PlayerMediaKind::Audio => self.audio_player.stop(),
                PlayerMediaKind::Video => {
                    media.video_paused_at = Some(0.0);
                    media.seek_video(0.0);
                }
            }
            media.is_playing = false;
            media.is_projecting = false;
        }
    }
}
