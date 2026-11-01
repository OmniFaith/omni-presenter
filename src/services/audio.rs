use crate::core::i18n::Language;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[cfg(target_os = "linux")]
pub fn silence_audio_backend_noise() {
    unsafe {
        // Impede que clientes JACK tentem disparar um daemon inexistente
        std::env::set_var("JACK_NO_START_SERVER", "1");

        // 1. Silencia o handler de erros padrão da ALSA (libasound)
        // Pela documentação da ALSA, passar NULL desativa a escrita de erros em stderr
        type SndLibErrorHandler = extern "C" fn(
            file: *const std::ffi::c_char,
            line: std::ffi::c_int,
            function: *const std::ffi::c_char,
            err: std::ffi::c_int,
            fmt: *const std::ffi::c_char,
            ...
        );
        type SndLibErrorSetHandlerFn =
            unsafe extern "C" fn(handler: Option<SndLibErrorHandler>) -> std::ffi::c_int;

        let asound = libc::dlopen(b"libasound.so.2\0".as_ptr() as *const _, libc::RTLD_LAZY);
        if !asound.is_null() {
            let sym = libc::dlsym(asound, b"snd_lib_error_set_handler\0".as_ptr() as *const _);
            if !sym.is_null() {
                let snd_lib_error_set_handler: SndLibErrorSetHandlerFn = std::mem::transmute(sym);
                let _ = snd_lib_error_set_handler(None);
            }
        }

        // 2. Silencia o handler de logs/erros do client JACK (libjack)
        type JackLogFn = extern "C" fn(msg: *const std::ffi::c_char);
        type JackSetLogFn = unsafe extern "C" fn(handler: Option<JackLogFn>);

        let jack = libc::dlopen(b"libjack.so.0\0".as_ptr() as *const _, libc::RTLD_LAZY);
        if !jack.is_null() {
            extern "C" fn silent_jack_log(_: *const std::ffi::c_char) {}
            let sym_err = libc::dlsym(jack, b"jack_set_error_function\0".as_ptr() as *const _);
            if !sym_err.is_null() {
                let jack_set_error: JackSetLogFn = std::mem::transmute(sym_err);
                jack_set_error(Some(silent_jack_log));
            }
            let sym_info = libc::dlsym(jack, b"jack_set_info_function\0".as_ptr() as *const _);
            if !sym_info.is_null() {
                let jack_set_info: JackSetLogFn = std::mem::transmute(sym_info);
                jack_set_info(Some(silent_jack_log));
            }
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub fn silence_audio_backend_noise() {}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AudioTrackInfo {
    pub title: String,
    pub path: String,
    pub duration_secs: f32,
}

struct AudioPlayerInner {
    sink: Option<MixerDeviceSink>,
    player: Option<Player>,
    current_track: Option<AudioTrackInfo>,
    is_playing: bool,
    is_looping: bool,
    volume: f32,
}

impl AudioPlayerInner {
    fn new() -> Self {
        Self {
            sink: None,
            player: None,
            current_track: None,
            is_playing: false,
            is_looping: false,
            volume: 1.0,
        }
    }
}

#[derive(Clone)]
pub struct AudioPlayer {
    inner: Arc<Mutex<AudioPlayerInner>>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AudioPlayerInner::new())),
        }
    }

    pub fn is_active(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.current_track.is_some()
    }

    pub fn current_track(&self) -> Option<AudioTrackInfo> {
        let inner = self.inner.lock().unwrap();
        inner.current_track.clone()
    }

    pub fn is_playing(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.is_playing
            && inner
                .player
                .as_ref()
                .map(|p| !p.is_paused() && !p.empty())
                .unwrap_or(false)
    }

    pub fn is_looping(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.is_looping
    }

    pub fn volume(&self) -> f32 {
        let inner = self.inner.lock().unwrap();
        inner.volume
    }

    pub fn set_volume(&mut self, vol: f32) {
        let mut inner = self.inner.lock().unwrap();
        inner.volume = vol.clamp(0.0, 1.0);
        let vol_val = inner.volume;
        if let Some(player) = &inner.player {
            player.set_volume(vol_val);
        }
    }

    pub fn toggle_loop(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.is_looping = !inner.is_looping;
    }

    pub fn play_file(&mut self, title: String, file_path: String) -> Result<(), String> {
        let path = Path::new(&file_path);
        if !path.exists() {
            return Err(Language::system().format("services-audio-file-not-found", &[("file_path", &file_path)]));
        }

        let file = File::open(path).map_err(|e| Language::system().format("services-could-not-open-the-audio", &[("e", &e.to_string())]))?;
        let reader = BufReader::new(file);
        let source =
            Decoder::new(reader).map_err(|e| Language::system().format("services-unsupported-audio-format", &[("e", &e.to_string())]))?;
        let total_duration = source
            .total_duration()
            .unwrap_or(Duration::from_secs(0))
            .as_secs_f32();

        let mut inner = self.inner.lock().unwrap();

        if inner.sink.is_none() {
            silence_audio_backend_noise();
            inner.sink = DeviceSinkBuilder::open_default_sink()
                .or_else(|_| {
                    DeviceSinkBuilder::from_default_device().and_then(|b| b.open_sink_or_fallback())
                })
                .ok();
        }

        if let Some(old_player) = inner.player.take() {
            old_player.stop();
        }

        let sink = inner
            .sink
            .as_ref()
            .ok_or_else(|| Language::system().text("services-no-audio-device-detected-on-this-system").to_string())?;
        let player = Player::connect_new(sink.mixer());
        player.set_volume(inner.volume);

        if inner.is_looping {
            player.append(source.repeat_infinite());
        } else {
            player.append(source);
        }

        player.play();

        inner.player = Some(player);
        inner.current_track = Some(AudioTrackInfo {
            title,
            path: file_path,
            duration_secs: total_duration,
        });
        inner.is_playing = true;

        Ok(())
    }

    pub fn toggle_play_pause(&mut self) {
        let track_to_restart = {
            let mut inner = self.inner.lock().unwrap();
            if let Some(player) = &inner.player {
                if player.is_paused() {
                    player.play();
                    inner.is_playing = true;
                } else {
                    player.pause();
                    inner.is_playing = false;
                }
                None
            } else {
                inner.current_track.clone()
            }
        };

        if let Some(track) = track_to_restart {
            let _ = self.play_file(track.title, track.path);
        }
    }

    pub fn pause(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(player) = &inner.player {
            player.pause();
            inner.is_playing = false;
        }
    }

    pub fn resume(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(player) = &inner.player {
            player.play();
            inner.is_playing = true;
        }
    }

    pub fn stop(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(player) = inner.player.take() {
            player.stop();
        }
        inner.is_playing = false;
    }

    pub fn close(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(player) = inner.player.take() {
            player.stop();
        }
        inner.is_playing = false;
        inner.current_track = None;
    }

    pub fn position_secs(&self) -> f32 {
        let inner = self.inner.lock().unwrap();
        if let Some(player) = &inner.player {
            player.get_pos().as_secs_f32()
        } else {
            0.0
        }
    }

    pub fn seek(&mut self, target_secs: f32) {
        let inner = self.inner.lock().unwrap();
        if let Some(player) = &inner.player {
            let dur = Duration::from_secs_f32(target_secs.max(0.0));
            let _ = player.try_seek(dur);
        }
    }

    pub fn update(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(player) = &inner.player
            && player.empty() && inner.is_playing {
                inner.is_playing = false;
            }
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}
