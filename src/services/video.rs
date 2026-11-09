//! Background-video decoding is kept off the UI thread. All native views share
//! the latest frame and one texture per clip/settings combination.
use crate::core::i18n::Language;
use crate::core::models::{PresentationTheme, theme_options::VideoOptions};
use eframe::egui::{self, Color32, Rect};
use std::{
    collections::HashMap,
    io::Read,
    process::{Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

struct Frame {
    sequence: u64,
    image: Arc<egui::ColorImage>,
}
struct Shared {
    frame: Mutex<Option<Frame>>,
    status: Mutex<String>,
    used: Mutex<Instant>,
    stop: AtomicBool,
    paused: AtomicBool,
    expired: AtomicBool,
    finished: AtomicBool,
}
struct Entry {
    shared: Arc<Shared>,
    texture: Option<egui::TextureHandle>,
    sequence: u64,
}
type Cache = Arc<Mutex<HashMap<String, Entry>>>;
#[cfg(test)]
#[path = "video_4k_tests.rs"]
mod video_4k_tests;

fn cache(ctx: &egui::Context) -> Cache {
    ctx.data_mut(|d| {
        d.get_temp_mut_or_insert_with(egui::Id::unique("video_cache"), || {
            Arc::new(Mutex::new(HashMap::new()))
        })
        .clone()
    })
}

fn key(path: &str, options: &VideoOptions) -> String {
    let mut options = options.clone();
    options.path.clear();
    options.paused = false;
    options.fit.clear();
    format!(
        "{path}:{}",
        serde_json::to_string(&options).unwrap_or_default()
    )
}

pub fn path(theme: &PresentationTheme) -> &str {
    if !theme.options.video.path.is_empty() {
        &theme.options.video.path
    } else if theme.bg_image_path.rsplit('.').next().is_some_and(|ext| {
        ["mp4", "webm", "mov", "mkv", "avi"].contains(&ext.to_lowercase().as_str())
    }) {
        &theme.bg_image_path
    } else {
        ""
    }
}

fn start(path: String, options: VideoOptions, ctx: egui::Context) -> Arc<Shared> {
    let shared = Arc::new(Shared {
        frame: Mutex::new(None),
        status: Mutex::new(Language::system().text("services-loading-video").to_string()),
        used: Mutex::new(Instant::now()),
        stop: AtomicBool::new(false),
        paused: AtomicBool::new(options.paused),
        expired: AtomicBool::new(false),
        finished: AtomicBool::new(false),
    });
    let worker = shared.clone();
    std::thread::spawn(move || {
        let result = (|| -> Result<(), String> {
            let stream_path = crate::services::web_video::resolve_video_stream(&path);
            let metadata = Command::new("ffprobe")
                .args([
                    "-v",
                    "error",
                    "-select_streams",
                    "v:0",
                    "-show_entries",
                    "stream=width,height",
                    "-of",
                    "csv=s=x:p=0",
                ])
                .arg(&stream_path)
                .output()
                .map_err(|_| {
                    Language::system().text("services-install-ffmpeg-ffmpeg-and-ffprobe-to-play-videos").to_string()
                })?;
            if !metadata.status.success() {
                return Err(Language::system().text("services-could-not-open-this-video-file").to_string());
            }
            let dimensions = String::from_utf8_lossy(&metadata.stdout);
            let mut values = dimensions
                .trim()
                .split('x')
                .filter_map(|s| s.trim().parse::<usize>().ok());
            let source_w = values.next().ok_or_else(|| Language::system().text("services-invalid-video-width").to_string())?;
            let source_h = values.next().ok_or_else(|| Language::system().text("services-invalid-video-height").to_string())?;
            if source_w == 0 || source_h == 0 {
                return Err(Language::system().text("services-the-video-has-no-image").to_string());
            }
            let scale = (1280.0 / source_w as f32)
                .min(720.0 / source_h as f32)
                .min(1.0);
            let width = ((source_w as f32 * scale) as usize / 2 * 2).max(2);
            let height = ((source_h as f32 * scale) as usize / 2 * 2).max(2);
            let speed = options.speed.clamp(0.1, 4.0);
            let mut command = Command::new("ffmpeg");
            command.args([
                "-nostdin",
                "-v",
                "error",
                "-threads",
                "0",
                "-filter_threads",
                "0",
            ]);
            if options.hwaccel {
                command.args(["-hwaccel", "auto"]);
            }
            if options.repeat {
                command.args(["-stream_loop", "-1"]);
            }
            command
                .args([
                    "-readrate",
                    &speed.to_string(),
                    "-ss",
                    &options.start.max(0.0).to_string(),
                    "-i",
                ])
                .arg(&stream_path);
            let mut filter_parts = vec![
                format!("setpts=PTS/{speed}"),
                "fps=30".to_string(),
                format!("scale={width}:{height}"),
            ];
            if (options.contrast - 1.0).abs() > 0.01 || (options.saturation - 1.0).abs() > 0.01 {
                filter_parts.push(format!(
                    "eq=contrast={}:saturation={}",
                    options.contrast, options.saturation
                ));
            }
            if options.mirror {
                filter_parts.push("hflip".to_string());
            }
            let filter = filter_parts.join(",");
            let mut child = command
                .args([
                    "-an", "-sn", "-vf", &filter, "-f", "rawvideo", "-pix_fmt", "rgba", "pipe:1",
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| Language::system().format("services-could-not-start-ffmpeg", &[("e", &e.to_string())]))?;
            let mut stdout = child.stdout.take().unwrap();
            let mut sequence = 0;
            let mut bytes = vec![0u8; width * height * 4];
            loop {
                if worker.stop.load(Ordering::Relaxed)
                    || worker.used.lock().unwrap().elapsed() > Duration::from_secs(2)
                {
                    worker.expired.store(true, Ordering::Relaxed);
                    break;
                }
                if worker.paused.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(30));
                    continue;
                }
                if stdout.read_exact(&mut bytes).is_err() {
                    break;
                }
                sequence += 1;
                let image = Arc::new(egui::ColorImage::from_rgba_unmultiplied(
                    [width, height],
                    &bytes,
                ));
                *worker.frame.lock().unwrap() = Some(Frame { sequence, image });
                *worker.status.lock().unwrap() = Language::system().text("services-playing-muted-video").to_string();
                ctx.request_repaint();
            }
            let _ = child.kill();
            let _ = child.wait();
            if sequence == 0 {
                return Err(Language::system().text("services-the-video-produced-no-frames-check-the-file-and-codec").to_string());
            }
            if !worker.expired.load(Ordering::Relaxed) {
                *worker.status.lock().unwrap() = Language::system().text("services-playback-finished").to_string();
            }
            Ok(())
        })();
        if let Err(error) = result {
            *worker.status.lock().unwrap() = error;
            ctx.request_repaint();
        }
        worker.finished.store(true, Ordering::Relaxed);
        ctx.request_repaint_of(egui::ViewportId::ROOT);
    });
    shared
}

pub fn status(ctx: &egui::Context, path: &str, options: &VideoOptions) -> String {
    let cache = cache(ctx);
    let cache = cache.lock().unwrap();
    cache
        .get(&key(path, options))
        .map(|entry| entry.shared.status.lock().unwrap().clone())
        .unwrap_or_else(|| Language::system().text("services-select-a-video-to-preview").to_string())
}

pub fn probe_duration(path: &str) -> Option<f32> {
    let stream_path = crate::services::web_video::resolve_video_stream(path);
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(&stream_path)
        .output()
        .ok()?;
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout);
        text.trim().parse::<f32>().ok()
    } else {
        None
    }
}

/// ffprobe can inspect a large WebM for seconds; never invoke it from a click
/// or a UI frame. Cache both successful and unavailable duration results.
pub fn duration(path: &str) -> Option<f32> {
    enum State {
        Pending(crate::services::tasks::BackgroundTask<Option<f32>>),
        Ready(Option<f32>),
    }
    static DURATIONS: std::sync::OnceLock<Mutex<HashMap<String, State>>> =
        std::sync::OnceLock::new();
    let mut cache = DURATIONS.get_or_init(Mutex::default).lock().unwrap();
    if !cache.contains_key(path) {
        let file = path.to_owned();
        cache.insert(
            path.into(),
            State::Pending(crate::services::tasks::BackgroundTask::spawn(move || {
                Ok(probe_duration(&file))
            })),
        );
        return None;
    }
    let value = match cache.get(path).unwrap() {
        State::Ready(value) => return *value,
        State::Pending(task) => task.take(),
    };
    if let Some(value) = value {
        let value = value
            .ok()
            .flatten()
            .filter(|value| value.is_finite() && *value >= 0.0);
        cache.insert(path.into(), State::Ready(value));
        value
    } else {
        None
    }
}

pub fn paint(painter: &egui::Painter, rect: Rect, theme: &PresentationTheme, radius: f32) {
    let p = path(theme);
    paint_video(painter, rect, p, &theme.options.video, radius);
}

fn ensure_entry<'a>(
    entries: &'a mut HashMap<String, Entry>,
    ctx: &egui::Context,
    path: &str,
    options: &VideoOptions,
) -> &'a mut Entry {
    let key = key(path, options);
    if entries
        .get(&key)
        .is_some_and(|entry| entry.shared.expired.load(Ordering::Relaxed))
    {
        entries.remove(&key);
    }
    if !entries.contains_key(&key) {
        if entries.len() >= 4
            && let Some(oldest) = entries
                .iter()
                .max_by_key(|(_, entry)| entry.shared.used.lock().unwrap().elapsed())
                .map(|(key, _)| key.clone())
                && let Some(entry) = entries.remove(&oldest) {
                    entry.shared.stop.store(true, Ordering::Relaxed);
                }
        entries.insert(
            key.clone(),
            Entry {
                shared: start(path.to_owned(), options.clone(), ctx.clone()),
                texture: None,
                sequence: 0,
            },
        );
    }
    let entry = entries.get_mut(&key).unwrap();
    *entry.shared.used.lock().unwrap() = Instant::now();
    entry
}

/// Start decoding without exposing a black loading placeholder on the output.
pub fn prepare(ctx: &egui::Context, theme: &PresentationTheme) -> bool {
    let path = path(theme);
    if path.is_empty() {
        return true;
    }
    let cache = cache(ctx);
    let mut cache = cache.lock().unwrap();
    let entry = ensure_entry(&mut cache, ctx, path, &theme.options.video);
    let ready = entry.shared.frame.lock().unwrap().is_some()
        || entry.shared.finished.load(Ordering::Relaxed);
    if !ready {
        ctx.request_repaint_after(Duration::from_millis(33));
    }
    ready
}

pub fn paint_video(
    painter: &egui::Painter,
    rect: Rect,
    path: &str,
    options: &VideoOptions,
    radius: f32,
) {
    painter.rect_filled(rect, radius, Color32::BLACK);
    if path.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            Language::system().text("services-select-a-video-to-preview"),
            egui::FontId::proportional(12.0),
            Color32::GRAY,
        );
        return;
    }
    let cache = cache(painter.ctx());
    let mut cache = cache.lock().unwrap();
    let key = key(path, options);
    if !cache.contains_key(&key) && rect.width() < 60.0 {
        return;
    }
    let entry = ensure_entry(&mut cache, painter.ctx(), path, options);
    entry.shared.paused.store(options.paused, Ordering::Relaxed);
    if let Some(frame) = entry.shared.frame.lock().unwrap().as_ref()
        && frame.sequence != entry.sequence {
            entry.sequence = frame.sequence;
            let image = frame.image.clone();
            if let Some(texture) = &mut entry.texture {
                texture.set(image, egui::TextureOptions::LINEAR);
            } else {
                entry.texture = Some(painter.ctx().load_texture(
                    &key,
                    image,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
    if let Some(texture) = &entry.texture {
        let aspect = texture.size()[0] as f32 / texture.size()[1] as f32;
        let mut target = rect;
        let mut uv = Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0));
        match options.fit.as_str() {
            "contain" => {
                let w = rect.width().min(rect.height() * aspect);
                target = Rect::from_center_size(rect.center(), egui::vec2(w, w / aspect));
            }
            "stretch" => {}
            _ => {
                let target_aspect = rect.width() / rect.height();
                if aspect > target_aspect {
                    let fraction = target_aspect / aspect;
                    uv.min.x = (1.0 - fraction) * 0.5;
                    uv.max.x = 1.0 - uv.min.x;
                } else {
                    let fraction = aspect / target_aspect;
                    uv.min.y = (1.0 - fraction) * 0.5;
                    uv.max.y = 1.0 - uv.min.y;
                }
            }
        }
        painter
            .with_clip_rect(painter.clip_rect().intersect(rect))
            .image(texture.id(), target, uv, Color32::WHITE);
    } else {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            entry.shared.status.lock().unwrap().as_str(),
            egui::FontId::proportional(11.0),
            Color32::GRAY,
        );
    }
    if !options.paused {
        painter
            .ctx()
            .request_repaint_after(Duration::from_millis(33));
    }
}

pub fn stop(ctx: &egui::Context, path: &str) {
    let cache = cache(ctx);
    let mut cache = cache.lock().unwrap();
    cache.retain(|k, entry| {
        if path.is_empty() || k.starts_with(path) {
            entry.shared.stop.store(true, Ordering::Relaxed);
            false
        } else {
            true
        }
    });
}

pub fn stop_all(ctx: &egui::Context) {
    let cache = cache(ctx);
    let mut cache = cache.lock().unwrap();
    for (_, entry) in cache.drain() {
        entry.shared.stop.store(true, Ordering::Relaxed);
    }
}

pub fn pause(ctx: &egui::Context, path: &str) {
    let cache = cache(ctx);
    let cache = cache.lock().unwrap();
    for (k, entry) in cache.iter() {
        if path.is_empty() || k.starts_with(path) {
            entry.shared.paused.store(true, Ordering::Relaxed);
        }
    }
}

pub fn resume(ctx: &egui::Context, path: &str) {
    let cache = cache(ctx);
    let cache = cache.lock().unwrap();
    for (k, entry) in cache.iter() {
        if path.is_empty() || k.starts_with(path) {
            entry.shared.paused.store(false, Ordering::Relaxed);
        }
    }
}

pub fn seek(ctx: &egui::Context, path: &str, _position: f32) {
    stop(ctx, path);
}

pub fn is_finished(ctx: &egui::Context, path: &str) -> bool {
    let cache = cache(ctx);
    let cache = cache.lock().unwrap();
    cache
        .iter()
        .find(|(k, _)| path.is_empty() || k.starts_with(path))
        .map(|(_, entry)| {
            entry.shared.finished.load(Ordering::Relaxed)
                && !entry.shared.expired.load(Ordering::Relaxed)
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ffmpeg_decodes_moving_frames_off_the_ui_thread() {
        if Command::new("ffmpeg")
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_err()
        {
            return;
        }
        let path = std::env::temp_dir().join(format!(
            "{}.mp4",
            crate::core::preferences::new_id("omni-presenter-video-test")
        ));
        let generated = Command::new("ffmpeg")
            .args([
                "-y",
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc2=size=160x90:rate=15",
                "-t",
                "1.2",
                "-c:v",
                "mpeg4",
            ])
            .arg(&path)
            .output()
            .unwrap();
        assert!(generated.status.success());
        let ctx = egui::Context::default();
        let theme = PresentationTheme {
            bg_type: "video".into(),
            bg_image_path: path.to_string_lossy().into_owned(),
            ..Default::default()
        };
        assert!(!prepare(&ctx, &theme));
        let shared = cache(&ctx)
            .lock()
            .unwrap()
            .get(&key(&theme.bg_image_path, &theme.options.video))
            .unwrap()
            .shared
            .clone();
        let deadline = Instant::now() + Duration::from_secs(6);
        let mut first = None;
        let mut changed = false;
        while Instant::now() < deadline {
            prepare(&ctx, &theme);
            if let Some(frame) = shared.frame.lock().unwrap().as_ref() {
                assert_eq!(
                    frame.image.pixels.len(),
                    frame.image.size[0] * frame.image.size[1]
                );
                if let Some((sequence, bytes)) = &first {
                    if frame.sequence > sequence + 4 && frame.image.pixels != *bytes {
                        changed = true;
                        break;
                    }
                } else {
                    first = Some((frame.sequence, frame.image.pixels.clone()));
                }
            }
            std::thread::sleep(Duration::from_millis(40));
        }
        assert!(
            prepare(&ctx, &theme),
            "The first frame must be ready before fullscreen starts"
        );
        shared.stop.store(true, Ordering::Relaxed);
        let _ = std::fs::remove_file(path);
        assert!(changed, "{}", shared.status.lock().unwrap());
    }
}
