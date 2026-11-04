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
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VideoFormat {
    #[default]
    Mp4,
    LosslessMkv,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AudioFormat {
    #[default]
    Flac,
    Mp3,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Formats {
    pub image: ImageFormat,
    pub video: VideoFormat,
    pub audio: AudioFormat,
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
        Self {
            paths,
            folder,
            formats: Formats::default(),
            dont_ask: false,
            from_import,
            ffmpeg_available: None,
            ffmpeg_check: Some(ffmpeg_check),
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
    std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let batch = directory.join(crate::core::preferences::new_id("conversion"));
    std::fs::create_dir(&batch).map_err(|error| error.to_string())?;
    let mut results = Vec::with_capacity(paths.len());
    for path in paths {
        if !path.is_file() {
            let _ = std::fs::remove_dir_all(&batch);
            return Err(format!("{}: file not found", path.display()));
        }
        if !convertible(&path) {
            results.push(path);
            continue;
        }
        let kind = MediaItem::detect_type(&path.to_string_lossy());
        let (extension, options): (&str, &[&str]) = match kind {
            "image" => match formats.image {
                ImageFormat::Png => ("png", &["-map", "0:v:0", "-frames:v", "1", "-c:v", "png"]),
                ImageFormat::Webp => (
                    "webp",
                    &[
                        "-map",
                        "0:v:0",
                        "-frames:v",
                        "1",
                        "-c:v",
                        "libwebp",
                        "-lossless",
                        "1",
                        "-compression_level",
                        "4",
                    ],
                ),
            },
            "video" => match formats.video {
                VideoFormat::Mp4 => (
                    "mp4",
                    &[
                        "-map",
                        "0:v:0",
                        "-map",
                        "0:a?",
                        "-c:v",
                        "libx264",
                        "-preset",
                        "slow",
                        "-crf",
                        "16",
                        "-vf",
                        "pad=ceil(iw/2)*2:ceil(ih/2)*2",
                        "-pix_fmt",
                        "yuv420p",
                        "-c:a",
                        "aac",
                        "-b:a",
                        "320k",
                        "-movflags",
                        "+faststart",
                    ],
                ),
                VideoFormat::LosslessMkv => (
                    "mkv",
                    &[
                        "-map", "0:v:0", "-map", "0:a?", "-c:v", "ffv1", "-level", "3", "-c:a",
                        "flac",
                    ],
                ),
            },
            "audio" => match formats.audio {
                AudioFormat::Flac => ("flac", &["-map", "0:a:0", "-vn", "-c:a", "flac"]),
                AudioFormat::Mp3 => (
                    "mp3",
                    &["-map", "0:a:0", "-vn", "-c:a", "libmp3lame", "-b:a", "320k"],
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
        // -n ensures another process cannot cause an existing file to be overwritten.
        let output = ffmpeg()
            .args(["-nostdin", "-n", "-loglevel", "error", "-i"])
            .arg(&path)
            .args(options)
            .arg(&target)
            .output();
        let error = match output {
            Ok(output) if output.status.success() => None,
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
