//! Downloads and extracts web videos (YouTube, Vimeo, Twitch, direct links, etc.)
//! into local MP4 media files for presentation and live playback.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

static STREAM_CACHE: OnceLock<Mutex<HashMap<String, (String, Instant)>>> = OnceLock::new();

/// Resolves a web video link (YouTube, Vimeo, etc.) to a direct streaming URL
/// that ffmpeg and players can stream without downloading to disk.
pub fn resolve_video_stream(url_or_path: &str) -> String {
    let trimmed = url_or_path.trim();
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return trimmed.to_string();
    }

    // Direct video streams don't need extraction
    if trimmed.ends_with(".mp4")
        || trimmed.ends_with(".m3u8")
        || trimmed.ends_with(".webm")
        || trimmed.contains("googlevideo.com/videoplayback")
    {
        return trimmed.to_string();
    }

    // Check in-memory cache (TTL 2 hours)
    let cache = STREAM_CACHE.get_or_init(Mutex::default);
    if let Ok(guard) = cache.lock() {
        if let Some((stream_url, created_at)) = guard.get(trimmed) {
            if created_at.elapsed() < Duration::from_secs(7200) {
                return stream_url.clone();
            }
        }
    }

    // Resolve via yt-dlp -g
    let Some(ytdlp) = find_ytdlp_binary() else {
        return trimmed.to_string();
    };

    let output = Command::new(&ytdlp)
        .args([
            "--no-playlist",
            "-g",
            "-f",
            "best[ext=mp4]/best",
            trimmed,
        ])
        .output();

    if let Ok(output) = output
        && output.status.success()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(stream_url) = stdout.lines().map(str::trim).find(|l| l.starts_with("http")) {
            let stream_url = stream_url.to_string();
            if let Ok(mut guard) = cache.lock() {
                guard.insert(trimmed.to_string(), (stream_url.clone(), Instant::now()));
            }
            return stream_url;
        }
    }

    trimmed.to_string()
}

/// Fetches the video title from a web URL without downloading the video.
pub fn fetch_web_video_title(url: &str) -> Result<String, String> {
    let url = url.trim();
    if !is_valid_web_url(url) {
        return Err("URL inválida. Forneça um link válido começando com http:// ou https://".into());
    }

    let ytdlp = find_ytdlp_binary().ok_or_else(|| {
        "yt-dlp não foi encontrado no sistema. Por favor, instale o yt-dlp.".to_string()
    })?;

    let output = Command::new(&ytdlp)
        .args([
            "--no-playlist",
            "--print",
            "%(title)s",
            url,
        ])
        .output()
        .map_err(|e| format!("Erro ao executar yt-dlp: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_msg = stderr
            .lines()
            .find(|line| line.contains("ERROR:"))
            .map(|line| line.trim_start_matches("ERROR:").trim())
            .unwrap_or_else(|| stderr.trim());
        return Err(if error_msg.is_empty() {
            format!("yt-dlp falhou com código {:?}", output.status.code())
        } else {
            error_msg.to_string()
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let title = stdout.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("").to_string();
    if title.is_empty() {
        Ok("Vídeo Web".into())
    } else {
        Ok(title)
    }
}

/// Validates whether a string is a potentially valid web video URL.
pub fn is_valid_web_url(url: &str) -> bool {
    let trimmed = url.trim();
    (trimmed.starts_with("http://") || trimmed.starts_with("https://")) && trimmed.len() > 10
}

/// Finds the yt-dlp binary on the system.
pub fn find_ytdlp_binary() -> Option<PathBuf> {
    for candidate in ["yt-dlp", "/usr/bin/yt-dlp", "/usr/local/bin/yt-dlp"] {
        if let Ok(output) = Command::new(candidate).arg("--version").output() {
            if output.status.success() {
                return Some(PathBuf::from(candidate));
            }
        }
    }
    None
}

/// Downloads a web video using yt-dlp to `web_media/` directory.
/// Returns `(video_title, file_path)`.
pub fn download_web_video(
    url: &str,
    title_override: Option<&str>,
) -> Result<(String, PathBuf), String> {
    let url = url.trim();
    if !is_valid_web_url(url) {
        return Err("URL inválida. Forneça um link válido começando com http:// ou https://".into());
    }

    let ytdlp = find_ytdlp_binary().ok_or_else(|| {
        "yt-dlp não foi encontrado no sistema. Por favor, instale o yt-dlp.".to_string()
    })?;

    let output_dir = Path::new("web_media");
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("Não foi possível criar a pasta web_media: {e}"))?;

    let output_template = "web_media/%(title).80s [%(id)s].%(ext)s";

    let output = Command::new(&ytdlp)
        .args([
            "--no-playlist",
            "--no-simulate",
            "-S",
            "res:1080,ext:mp4:m4a",
            "--merge-output-format",
            "mp4",
            "--print",
            "TITLE:%(title)s",
            "--print",
            "after_move:FILE:%(filepath)s",
            "-o",
            output_template,
            url,
        ])
        .output()
        .map_err(|e| format!("Erro ao executar yt-dlp: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_msg = stderr
            .lines()
            .find(|line| line.contains("ERROR:"))
            .map(|line| line.trim_start_matches("ERROR:").trim())
            .unwrap_or_else(|| stderr.trim());
        return Err(if error_msg.is_empty() {
            format!("yt-dlp falhou com código de saída {:?}", output.status.code())
        } else {
            error_msg.to_string()
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut detected_title = None;
    let mut downloaded_path = None;

    for line in stdout.lines() {
        let line = line.trim();
        if let Some(t) = line.strip_prefix("TITLE:") {
            if !t.is_empty() && detected_title.is_none() {
                detected_title = Some(t.to_string());
            }
        } else if let Some(p) = line.strip_prefix("FILE:") {
            if !p.is_empty() {
                downloaded_path = Some(PathBuf::from(p));
            }
        }
    }

    let path = downloaded_path
        .ok_or_else(|| "yt-dlp concluiu sem retornar o caminho do arquivo baixado.".to_string())?;

    if !path.exists() {
        return Err(format!("Arquivo baixado não foi encontrado em {:?}", path));
    }

    let final_title = title_override
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(String::from)
        .or(detected_title)
        .unwrap_or_else(|| {
            path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Vídeo Web".into())
        });

    Ok((final_title, path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_web_url() {
        assert!(is_valid_web_url("https://www.youtube.com/watch?v=123456"));
        assert!(is_valid_web_url("http://vimeo.com/12345678"));
        assert!(is_valid_web_url("https://youtu.be/123456"));
        assert!(!is_valid_web_url("ftp://example.com/video.mp4"));
        assert!(!is_valid_web_url("not a url"));
        assert!(!is_valid_web_url("https://"));
        assert!(!is_valid_web_url(""));
    }

    #[test]
    fn test_find_ytdlp_binary() {
        // On systems where yt-dlp is installed, this returns Some
        if let Some(path) = find_ytdlp_binary() {
            assert!(path.exists() || path == Path::new("yt-dlp"));
        }
    }
}
