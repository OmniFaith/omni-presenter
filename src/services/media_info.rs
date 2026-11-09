use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug, Default)]
pub struct MediaDetails {
    pub title: String,
    pub path: String,
    pub file_size_bytes: u64,
    pub file_size_formatted: String,
    pub file_type: String,
    pub duration_secs: Option<f32>,
    pub duration_formatted: Option<String>,
    pub dimensions: Option<(usize, usize)>,
    pub dimensions_formatted: Option<String>,
    pub aspect_ratio: Option<String>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub format_name: Option<String>,
}

pub fn format_duration(secs: f32) -> String {
    if secs.is_nan() || secs <= 0.0 {
        return "00:00".to_string();
    }
    let total_secs = secs.round() as u64;
    let s = total_secs % 60;
    let m = (total_secs / 60) % 60;
    let h = total_secs / 3600;
    if h > 0 {
        format!("{h:02}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub fn calculate_aspect_ratio(w: usize, h: usize) -> String {
    if h == 0 {
        return String::new();
    }
    let ratio = w as f32 / h as f32;
    if (ratio - 16.0 / 9.0).abs() < 0.06 {
        "16:9".to_string()
    } else if (ratio - 16.0 / 10.0).abs() < 0.06 {
        "16:10".to_string()
    } else if (ratio - 4.0 / 3.0).abs() < 0.06 {
        "4:3".to_string()
    } else if (ratio - 9.0 / 16.0).abs() < 0.06 {
        "9:16 (Vertical)".to_string()
    } else if (ratio - 21.0 / 9.0).abs() < 0.10 {
        "21:9 (Ultrawide)".to_string()
    } else {
        fn gcd(mut a: usize, mut b: usize) -> usize {
            while b != 0 {
                let t = b;
                b = a % b;
                a = t;
            }
            a
        }
        let d = gcd(w, h);
        if d > 1 && (w / d) < 100 && (h / d) < 100 {
            format!("{}:{}", w / d, h / d)
        } else {
            format!("{:.2}:1", ratio)
        }
    }
}

pub fn probe_media_details(path_str: &str) -> MediaDetails {
    let path = Path::new(path_str);
    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(path_str)
        .to_string();
    let meta = std::fs::metadata(path).ok();
    let file_size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    let file_size_formatted = format_bytes(file_size_bytes);
    let kind = crate::core::models::MediaItem::detect_type(path_str);
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut details = MediaDetails {
        title,
        path: path_str.to_string(),
        file_size_bytes,
        file_size_formatted,
        file_type: match kind {
            "video" => format!("Vídeo ({ext})"),
            "audio" => format!("Áudio ({ext})"),
            "image" => format!("Imagem ({ext})"),
            other => other.to_string(),
        },
        duration_secs: None,
        duration_formatted: None,
        dimensions: None,
        dimensions_formatted: None,
        aspect_ratio: None,
        video_codec: None,
        audio_codec: None,
        format_name: None,
    };

    if kind == "image" {
        if let Ok((w, h)) = image::image_dimensions(path) {
            let (w, h) = (w as usize, h as usize);
            details.dimensions = Some((w, h));
            details.dimensions_formatted = Some(format!("{w} × {h}"));
            details.aspect_ratio = Some(calculate_aspect_ratio(w, h));
        }
        return details;
    }

    // Use ffprobe for video/audio
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration,format_name:stream=codec_name,codec_type,width,height",
            "-of",
            "default=noprint_wrappers=1:nokey=0",
        ])
        .arg(path_str)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut width = None;
            let mut height = None;
            let mut current_type = String::new();

            for line in text.lines() {
                let line = line.trim();
                if let Some(val) = line.strip_prefix("codec_type=") {
                    current_type = val.to_string();
                } else if let Some(val) = line.strip_prefix("codec_name=") {
                    if current_type == "video" && details.video_codec.is_none() {
                        details.video_codec = Some(val.to_uppercase());
                    } else if current_type == "audio" && details.audio_codec.is_none() {
                        details.audio_codec = Some(val.to_uppercase());
                    }
                } else if let Some(val) = line.strip_prefix("width=") {
                    if let Ok(w) = val.parse::<usize>() {
                        width = Some(w);
                    }
                } else if let Some(val) = line.strip_prefix("height=") {
                    if let Ok(h) = val.parse::<usize>() {
                        height = Some(h);
                    }
                } else if let Some(val) = line.strip_prefix("duration=") {
                    if details.duration_secs.is_none() {
                        if let Ok(d) = val.parse::<f32>() {
                            details.duration_secs = Some(d);
                            details.duration_formatted = Some(format_duration(d));
                        }
                    }
                } else if let Some(val) = line.strip_prefix("format_name=") {
                    if details.format_name.is_none() {
                        details.format_name = Some(val.to_string());
                    }
                }
            }

            if let (Some(w), Some(h)) = (width, height) {
                details.dimensions = Some((w, h));
                details.dimensions_formatted = Some(format!("{w} × {h}"));
                details.aspect_ratio = Some(calculate_aspect_ratio(w, h));
            }
        }
    }

    details
}

