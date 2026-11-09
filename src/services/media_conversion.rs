//! Non-destructive, background media conversion using the user's FFmpeg installation.
use crate::core::models::MediaItem;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(test)]
mod tests;

fn ffmpeg() -> Command {
    let command = Command::new("ffmpeg");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let mut command = command;
        command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        command
    }
    #[cfg(not(target_os = "windows"))]
    {
        command
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageFormat {
    #[default]
    Png,
    Webp,
    Jpg,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VideoFormat {
    #[default]
    Mp4,
    Webm,
    LosslessMkv,
    Mov,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VideoResolution {
    #[default]
    Original,
    Uhd4k,
    Fhd1080p,
    Hd720p,
    Sd480p,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VideoAspectRatio {
    #[default]
    Original,
    R16x9,
    R16x10,
    R4x3,
    R9x16,
    R21x9,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AudioFormat {
    #[default]
    Flac,
    Mp3,
    Wav,
    Aac,
    Ogg,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Formats {
    pub image: ImageFormat,
    pub video: VideoFormat,
    pub video_resolution: VideoResolution,
    pub video_aspect_ratio: VideoAspectRatio,
    pub audio: AudioFormat,
}

#[derive(Clone, Debug, Default)]
pub struct ConversionProgress {
    pub current_file: usize,
    pub total_files: usize,
    pub current_filename: String,
    pub current_path: PathBuf,
    pub percent: f32,
    pub details: Option<crate::services::media_info::MediaDetails>,
}

#[derive(Clone)]
pub struct Prompt {
    pub paths: Vec<PathBuf>,
    pub folder: String,
    pub formats: Formats,
    pub dont_ask: bool,
    pub from_import: bool,
    pub ffmpeg_available: Option<bool>,
    pub ffmpeg_check: Option<crate::services::tasks::BackgroundTask<bool>>,
    pub probed_details: Option<crate::services::media_info::MediaDetails>,
    pub details_check: Option<crate::services::tasks::BackgroundTask<crate::services::media_info::MediaDetails>>,
}

impl Prompt {
    pub fn new(paths: Vec<PathBuf>, folder: String, from_import: bool) -> Self {
        let ffmpeg_check = crate::services::tasks::BackgroundTask::spawn(|| {
            Ok(ffmpeg()
                .arg("-version")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .is_ok_and(|status| status.success()))
        });
        let first_path = paths.first().cloned();
        let details_check = first_path.map(|p| {
            crate::services::tasks::BackgroundTask::spawn(move || {
                Ok(crate::services::media_info::probe_media_details(&p.to_string_lossy()))
            })
        });
        Self {
            paths,
            folder,
            formats: Formats::default(),
            dont_ask: false,
            from_import,
            ffmpeg_available: None,
            ffmpeg_check: Some(ffmpeg_check),
            probed_details: None,
            details_check,
        }
    }
}

pub fn convertible(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    // GIFs may be animated and converting them to a still image would lose frames.
    if extension.eq_ignore_ascii_case("gif") || extension.eq_ignore_ascii_case("svg") {
        return false;
    }
    matches!(
        MediaItem::detect_type(&path.to_string_lossy()),
        "image" | "video" | "audio"
    )
}

pub fn convert(
    paths: Vec<PathBuf>,
    formats: Formats,
    directory: &Path,
) -> Result<Vec<PathBuf>, String> {
    convert_with_progress(paths, formats, directory, None)
}

pub fn convert_with_progress(
    paths: Vec<PathBuf>,
    formats: Formats,
    directory: &Path,
    progress: Option<std::sync::Arc<std::sync::Mutex<ConversionProgress>>>,
) -> Result<Vec<PathBuf>, String> {
    std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let batch = directory.join(crate::core::preferences::new_id("conversion"));
    std::fs::create_dir(&batch).map_err(|error| error.to_string())?;
    let mut results = Vec::with_capacity(paths.len());
    let total = paths.len();

    let scale_filter = match formats.video_resolution {
        VideoResolution::Original => "pad=ceil(iw/2)*2:ceil(ih/2)*2".to_string(),
        VideoResolution::Uhd4k => "scale='min(iw,-2)':'min(2160,ih)':force_original_aspect_ratio=decrease,pad=ceil(iw/2)*2:ceil(ih/2)*2".to_string(),
        VideoResolution::Fhd1080p => "scale='min(iw,-2)':'min(1080,ih)':force_original_aspect_ratio=decrease,pad=ceil(iw/2)*2:ceil(ih/2)*2".to_string(),
        VideoResolution::Hd720p => "scale='min(iw,-2)':'min(720,ih)':force_original_aspect_ratio=decrease,pad=ceil(iw/2)*2:ceil(ih/2)*2".to_string(),
        VideoResolution::Sd480p => "scale='min(iw,-2)':'min(480,ih)':force_original_aspect_ratio=decrease,pad=ceil(iw/2)*2:ceil(ih/2)*2".to_string(),
    };
    let aspect_filter = match formats.video_aspect_ratio {
        VideoAspectRatio::Original => String::new(),
        VideoAspectRatio::R16x9 => ",pad=max(iw\\,ih*(16/9)):max(ih\\,iw/(16/9)):(ow-iw)/2:(oh-ih)/2,setdar=16/9".to_string(),
        VideoAspectRatio::R16x10 => ",pad=max(iw\\,ih*(16/10)):max(ih\\,iw/(16/10)):(ow-iw)/2:(oh-ih)/2,setdar=16/10".to_string(),
        VideoAspectRatio::R4x3 => ",pad=max(iw\\,ih*(4/3)):max(ih\\,iw/(4/3)):(ow-iw)/2:(oh-ih)/2,setdar=4/3".to_string(),
        VideoAspectRatio::R9x16 => ",pad=max(iw\\,ih*(9/16)):max(ih\\,iw/(9/16)):(ow-iw)/2:(oh-ih)/2,setdar=9/16".to_string(),
        VideoAspectRatio::R21x9 => ",pad=max(iw\\,ih*(21/9)):max(ih\\,iw/(21/9)):(ow-iw)/2:(oh-ih)/2,setdar=21/9".to_string(),
    };
    let video_vf = format!("{scale_filter}{aspect_filter}");

    for (idx, path) in paths.into_iter().enumerate() {
        if !path.is_file() {
            let _ = std::fs::remove_dir_all(&batch);
            return Err(format!("{}: file not found", path.display()));
        }
        if !convertible(&path) {
            results.push(path);
            continue;
        }

        let details = crate::services::media_info::probe_media_details(&path.to_string_lossy());
        if let Some(prog) = &progress {
            if let Ok(mut g) = prog.lock() {
                g.current_file = idx + 1;
                g.total_files = total;
                g.current_path = path.clone();
                g.current_filename = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                g.percent = (idx as f32) / (total as f32);
                g.details = Some(details);
            }
        }

        let kind = MediaItem::detect_type(&path.to_string_lossy());
        let (extension, options): (&str, Vec<String>) = match kind {
            "image" => match formats.image {
                ImageFormat::Png => (
                    "png",
                    vec!["-map".into(), "0:v:0".into(), "-frames:v".into(), "1".into(), "-c:v".into(), "png".into()],
                ),
                ImageFormat::Webp => (
                    "webp",
                    vec![
                        "-map".into(),
                        "0:v:0".into(),
                        "-frames:v".into(),
                        "1".into(),
                        "-c:v".into(),
                        "libwebp".into(),
                        "-lossless".into(),
                        "1".into(),
                        "-compression_level".into(),
                        "4".into(),
                    ],
                ),
                ImageFormat::Jpg => (
                    "jpg",
                    vec!["-map".into(), "0:v:0".into(), "-frames:v".into(), "1".into(), "-c:v".into(), "mjpeg".into(), "-q:v".into(), "2".into()],
                ),
            },
            "video" => match formats.video {
                VideoFormat::Mp4 => (
                    "mp4",
                    vec![
                        "-map".into(),
                        "0:v:0".into(),
                        "-map".into(),
                        "0:a?".into(),
                        "-c:v".into(),
                        "libx264".into(),
                        "-preset".into(),
                        "slow".into(),
                        "-crf".into(),
                        "16".into(),
                        "-vf".into(),
                        video_vf.clone(),
                        "-pix_fmt".into(),
                        "yuv420p".into(),
                        "-c:a".into(),
                        "aac".into(),
                        "-b:a".into(),
                        "320k".into(),
                        "-movflags".into(),
                        "+faststart".into(),
                    ],
                ),
                VideoFormat::Webm => (
                    "webm",
                    vec![
                        "-map".into(),
                        "0:v:0".into(),
                        "-map".into(),
                        "0:a?".into(),
                        "-c:v".into(),
                        "libvpx-vp9".into(),
                        "-crf".into(),
                        "30".into(),
                        "-b:v".into(),
                        "0".into(),
                        "-vf".into(),
                        video_vf.clone(),
                        "-pix_fmt".into(),
                        "yuv420p".into(),
                        "-c:a".into(),
                        "libopus".into(),
                        "-b:a".into(),
                        "192k".into(),
                    ],
                ),
                VideoFormat::LosslessMkv => (
                    "mkv",
                    vec![
                        "-map".into(), "0:v:0".into(), "-map".into(), "0:a?".into(), "-c:v".into(), "ffv1".into(), "-level".into(), "3".into(), "-c:a".into(),
                        "flac".into(),
                    ],
                ),
                VideoFormat::Mov => (
                    "mov",
                    vec![
                        "-map".into(),
                        "0:v:0".into(),
                        "-map".into(),
                        "0:a?".into(),
                        "-c:v".into(),
                        "libx264".into(),
                        "-preset".into(),
                        "slow".into(),
                        "-crf".into(),
                        "16".into(),
                        "-vf".into(),
                        video_vf.clone(),
                        "-pix_fmt".into(),
                        "yuv420p".into(),
                        "-c:a".into(),
                        "aac".into(),
                        "-b:a".into(),
                        "320k".into(),
                    ],
                ),
            },
            "audio" => match formats.audio {
                AudioFormat::Flac => ("flac", vec!["-map".into(), "0:a:0".into(), "-vn".into(), "-c:a".into(), "flac".into()]),
                AudioFormat::Mp3 => (
                    "mp3",
                    vec!["-map".into(), "0:a:0".into(), "-vn".into(), "-c:a".into(), "libmp3lame".into(), "-b:a".into(), "320k".into()],
                ),
                AudioFormat::Wav => (
                    "wav",
                    vec!["-map".into(), "0:a:0".into(), "-vn".into(), "-c:a".into(), "pcm_s16le".into()],
                ),
                AudioFormat::Aac => (
                    "m4a",
                    vec!["-map".into(), "0:a:0".into(), "-vn".into(), "-c:a".into(), "aac".into(), "-b:a".into(), "320k".into()],
                ),
                AudioFormat::Ogg => (
                    "ogg",
                    vec!["-map".into(), "0:a:0".into(), "-vn".into(), "-c:a".into(), "libvorbis".into(), "-q:a".into(), "6".into()],
                ),
            },
            _ => unreachable!(),
        };
        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("media");
        let target = (1..)
            .map(|n| batch.join(format!("{stem}-{n}.{extension}")))
            .find(|candidate| !candidate.exists())
            .ok_or("No output filename available")?;

        let dur = crate::services::video::probe_duration(&path.to_string_lossy()).unwrap_or(0.0);
        let mut child = ffmpeg()
            .args(["-nostdin", "-n", "-loglevel", "error", "-progress", "pipe:1", "-i"])
            .arg(&path)
            .args(&options)
            .arg(&target)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("{}: {e}", path.display()))?;

        if let Some(stdout) = child.stdout.take() {
            use std::io::BufRead;
            let reader = std::io::BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                if let Some(us_str) = line.strip_prefix("out_time_us=") {
                    if let Ok(us) = us_str.trim().parse::<f64>() {
                        let sec = (us / 1_000_000.0) as f32;
                        if dur > 0.0 {
                            let file_ratio = (sec / dur).clamp(0.0, 1.0);
                            if let Some(p) = &progress {
                                if let Ok(mut g) = p.lock() {
                                    g.percent = ((idx as f32) + file_ratio) / (total as f32);
                                }
                            }
                        }
                    }
                }
            }
        }

        let output = child.wait_with_output();
        let error = match output {
            Ok(output) if output.status.success() => {
                if let Some(p) = &progress {
                    if let Ok(mut g) = p.lock() {
                        g.percent = ((idx + 1) as f32) / (total as f32);
                    }
                }
                None
            }
            Ok(output) => Some(String::from_utf8_lossy(&output.stderr).trim().to_owned()),
            Err(error) => Some(error.to_string()),
        };
        if let Some(error) = error {
            let _ = std::fs::remove_dir_all(&batch);
            return Err(format!("{}: {error}", path.display()));
        }
        results.push(target);
    }
    Ok(results)
}
