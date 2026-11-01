use serde::{Deserialize, Serialize};
use serde_json;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LrcLibTrack {
    pub id: i64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "trackName", default)]
    pub track_name: String,
    #[serde(rename = "artistName", default)]
    pub artist_name: String,
    #[serde(rename = "albumName", default)]
    pub album_name: Option<String>,
    #[serde(default)]
    pub duration: Option<f64>,
    #[serde(default)]
    pub instrumental: bool,
    #[serde(rename = "plainLyrics", default)]
    pub plain_lyrics: Option<String>,
    #[serde(rename = "syncedLyrics", default)]
    pub synced_lyrics: Option<String>,
}

impl LrcLibTrack {
    pub fn title(&self) -> &str {
        if !self.track_name.is_empty() {
            &self.track_name
        } else if let Some(ref name) = self.name {
            name
        } else {
            ""
        }
    }

    pub fn formatted_duration(&self) -> String {
        if let Some(sec) = self.duration {
            let total_sec = sec.round() as u64;
            let m = total_sec / 60;
            let s = total_sec % 60;
            format!("{m}:{s:02}")
        } else {
            String::new()
        }
    }

    /// Returns properly formatted lyrics divided into presentation strophes separated by `\n\n`.
    pub fn lyrics_text(&self) -> String {
        format_lyrics_into_strophes(self.plain_lyrics.as_deref(), self.synced_lyrics.as_deref())
    }

    /// Returns the individual strophes for slide preview.
    pub fn strophes(&self) -> Vec<String> {
        let text = self.lyrics_text();
        if text.is_empty() {
            Vec::new()
        } else {
            text.split("\n\n")
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect()
        }
    }
}

/// Formats raw lyrics into distinct presentation strophes separated by double newlines (`\n\n`).
/// Prevents songs from becoming one giant unreadable slide by enforcing 2-4 lines per slide.
pub fn format_lyrics_into_strophes(plain: Option<&str>, synced: Option<&str>) -> String {
    // If synced lyrics is available, timing gaps and blank lines offer great stanza boundary hints
    if let Some(synced) = synced {
        let formatted = parse_synced_into_strophes(synced);
        if !formatted.trim().is_empty() {
            return formatted;
        }
    }

    if let Some(plain) = plain {
        let trimmed = plain.trim();
        if !trimmed.is_empty() {
            return format_plain_into_strophes(trimmed);
        }
    }

    String::new()
}

fn parse_synced_into_strophes(synced: &str) -> String {
    let mut strophes: Vec<Vec<String>> = Vec::new();
    let mut current_strophe: Vec<String> = Vec::new();
    let mut prev_time: Option<f64> = None;

    for line in synced.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !current_strophe.is_empty() {
                strophes.push(std::mem::take(&mut current_strophe));
            }
            continue;
        }

        let (time, text) = parse_lrc_line(trimmed);
        let text = text.trim();

        if text.is_empty() {
            // Explicit blank LRC timestamp line marks a break between verses
            if !current_strophe.is_empty() {
                strophes.push(std::mem::take(&mut current_strophe));
            }
            prev_time = time;
            continue;
        }

        // Check if there is a musical gap >= 4.0s or if current strophe reached 4 lines
        let has_timing_break = match (prev_time, time) {
            (Some(p), Some(c)) => (c - p) >= 4.0,
            _ => false,
        };

        if (has_timing_break || current_strophe.len() >= 4) && !current_strophe.is_empty() {
            strophes.push(std::mem::take(&mut current_strophe));
        }

        current_strophe.push(text.to_string());
        prev_time = time;
    }

    if !current_strophe.is_empty() {
        strophes.push(current_strophe);
    }

    strophes
        .into_iter()
        .filter(|s| !s.is_empty())
        .map(|s| s.join("\n"))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn parse_lrc_line(line: &str) -> (Option<f64>, String) {
    let mut time = None;
    let mut text = line;

    while text.starts_with('[') {
        if let Some(close_idx) = text.find(']') {
            let tag = &text[1..close_idx];
            if let Some(seconds) = parse_lrc_timestamp(tag) {
                time = Some(seconds);
            }
            text = text[close_idx + 1..].trim_start();
        } else {
            break;
        }
    }

    (time, text.to_string())
}

fn parse_lrc_timestamp(tag: &str) -> Option<f64> {
    let mut parts = tag.split(':');
    let mins: f64 = parts.next()?.parse().ok()?;
    let secs: f64 = parts.next()?.parse().ok()?;
    Some(mins * 60.0 + secs)
}

fn format_plain_into_strophes(plain: &str) -> String {
    let raw_strophes: Vec<&str> = plain
        .split("\n\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    // If it already has multiple distinct strophes and no single strophe is overly long (>6 lines)
    if raw_strophes.len() > 1 && raw_strophes.iter().all(|s| s.lines().count() <= 6) {
        return raw_strophes.join("\n\n");
    }

    // Otherwise, break into chunks of 4 lines
    let lines: Vec<&str> = plain
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();

    if lines.is_empty() {
        return String::new();
    }

    lines
        .chunks(4)
        .map(|chunk| chunk.join("\n"))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Strips standard LRC bracketed timestamps e.g. `[00:12.35]` from each line.
pub fn strip_lrc_timestamps(lrc: &str) -> String {
    let mut lines = Vec::new();
    for line in lrc.lines() {
        let mut text = line.trim();
        while text.starts_with('[') {
            if let Some(close_idx) = text.find(']') {
                text = text[close_idx + 1..].trim_start();
            } else {
                break;
            }
        }
        lines.push(text);
    }
    lines.join("\n")
}

pub fn urlencoding(input: &str) -> String {
    let mut encoded = String::new();
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                use std::fmt::Write;
                let _ = write!(encoded, "%{:02X}", byte);
            }
        }
    }
    encoded
}

/// Searches LRCLIB for lyrics using a unified query or track name and artist.
pub fn search_lyrics(track: &str, artist: Option<&str>) -> Result<Vec<LrcLibTrack>, String> {
    let track = track.trim();
    if track.is_empty() {
        return Ok(Vec::new());
    }

        let perform_request = |url: &str| -> Result<Vec<LrcLibTrack>, (u16, String)> {
            let response = ureq::get(url)
                .header("User-Agent", "OmniPresenter/0.7-alpha")
                .header("Lrclib-Client", "Omni Presenter v0.7 alpha")
                .call();

            match response {
                Ok(res) => {
                    let status = res.status().as_u16();
                    if status == 200 {
                        let body = match res.into_body().read_to_string() {
                            Ok(b) => b,
                            Err(e) => return Err((0, format!("Erro ao ler resposta: {e}"))),
                        };
                        let tracks: Vec<LrcLibTrack> = serde_json::from_str(&body)
                            .map_err(|e| (status, format!("Erro ao processar resposta: {e}")))?;
                        Ok(tracks)
                    } else {
                        Err((status, format!("Status HTTP {status}")))
                    }
                }
                Err(e) => {
                    let status = match &e {
                        ureq::Error::StatusCode(code) => *code,
                        _ => {
                            let msg = e.to_string();
                            if msg.contains("503") {
                                503
                            } else if msg.contains("429") {
                                429
                            } else {
                                0
                            }
                        }
                    };
                    Err((status, e.to_string()))
                }
            }
        };

    let has_artist = artist.is_some_and(|a| !a.trim().is_empty());
    let url = if let Some(art) = artist.filter(|a| !a.trim().is_empty()) {
        format!(
            "https://lrclib.net/api/search?track_name={}&artist_name={}",
            urlencoding(track),
            urlencoding(art.trim())
        )
    } else {
        format!("https://lrclib.net/api/search?q={}", urlencoding(track))
    };

    match perform_request(&url) {
        Ok(tracks) => Ok(tracks),
        Err((503 | 429, _)) => {
            // LRCLIB server may return 503 ServerOverloaded on general query. Retry after delay.
            std::thread::sleep(Duration::from_millis(1000));
            let retry_url = if !has_artist {
                format!(
                    "https://lrclib.net/api/search?track_name={}",
                    urlencoding(track)
                )
            } else {
                url.clone()
            };
            match perform_request(&retry_url) {
                Ok(tracks) => Ok(tracks),
                Err((503 | 429, _)) => {
                    std::thread::sleep(Duration::from_millis(1200));
                    perform_request(&retry_url).map_err(|_| "lrclib-server-busy".to_string())
                }
                Err((_, err)) => {
                    if err.contains("ServerOverloaded") || err.contains("503") {
                        Err("lrclib-server-busy".to_string())
                    } else {
                        Err(err)
                    }
                }
            }
        }
        Err((_, err)) => {
            if err.contains("ServerOverloaded") || err.contains("503") {
                Err("lrclib-server-busy".to_string())
            } else {
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_lrc_timestamps() {
        let input = "[00:12.35] Line one\n[00:15.50][00:18.00] Line two\n[00:20.00]\nLine three";
        let output = strip_lrc_timestamps(input);
        assert_eq!(output, "Line one\nLine two\n\nLine three");
    }

    #[test]
    fn test_format_plain_into_strophes() {
        // 8 continuous lines should be chunked into 2 strophes of 4 lines each
        let plain = "L1\nL2\nL3\nL4\nL5\nL6\nL7\nL8";
        let formatted = format_lyrics_into_strophes(Some(plain), None);
        assert_eq!(formatted, "L1\nL2\nL3\nL4\n\nL5\nL6\nL7\nL8");
    }

    #[test]
    fn test_format_synced_timing_gaps() {
        let synced = "[00:01.00] Line 1\n[00:03.00] Line 2\n[00:09.00] Line 3\n[00:11.00] Line 4";
        let formatted = format_lyrics_into_strophes(None, Some(synced));
        // There is a 6-second gap between Line 2 and Line 3 -> should create a strophe break
        assert_eq!(formatted, "Line 1\nLine 2\n\nLine 3\nLine 4");
    }

    #[test]
    fn test_deserialize_lrclib_track() {
        let json = r#"{
            "id": 12345,
            "trackName": "Amazing Grace",
            "artistName": "John Newton",
            "albumName": "Hymns",
            "duration": 185.4,
            "instrumental": false,
            "plainLyrics": "Amazing grace how sweet the sound",
            "syncedLyrics": "[00:10.00] Amazing grace how sweet the sound"
        }"#;

        let track: LrcLibTrack = serde_json::from_str(json).unwrap();
        assert_eq!(track.id, 12345);
        assert_eq!(track.title(), "Amazing Grace");
        assert_eq!(track.artist_name, "John Newton");
        assert_eq!(track.album_name.as_deref(), Some("Hymns"));
        assert_eq!(track.formatted_duration(), "3:05");
        assert_eq!(track.lyrics_text(), "Amazing grace how sweet the sound");
        assert_eq!(track.strophes().len(), 1);
    }

    #[test]
    fn test_synced_fallback_when_plain_empty() {
        let json = r#"{
            "id": 12346,
            "trackName": "Oceans",
            "artistName": "Hillsong",
            "duration": 120.0,
            "instrumental": false,
            "plainLyrics": null,
            "syncedLyrics": "[00:05.10] Spirit lead me where my trust is without borders"
        }"#;

        let track: LrcLibTrack = serde_json::from_str(json).unwrap();
        assert_eq!(
            track.lyrics_text(),
            "Spirit lead me where my trust is without borders"
        );
    }

    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding("hello world"), "hello+world");
        assert_eq!(urlencoding("Como Zaqueu"), "Como+Zaqueu");
        assert_eq!(urlencoding("Rock & Roll"), "Rock+%26+Roll");
    }
}
