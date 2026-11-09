//! Shared activation path for playable links in chat, shortcuts and the playlist.
use crate::core::{
    i18n::Language,
    models::MediaItem,
    slides::SlideDeck,
    state::{ControlPanelState, PlayerMediaKind},
};
use std::hash::{Hash, Hasher};

pub fn activate(
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    title: &str,
    path: &str,
    mime: &str,
    project: bool,
) -> Result<bool, String> {
    let is_web = path.starts_with("http://") || path.starts_with("https://");
    if !is_web && !std::path::Path::new(path).is_file() {
        return Err(state.settings.language.text("services-media-file-not-found").to_string());
    }
    let kind = if mime.starts_with("audio/") {
        "audio"
    } else if mime.starts_with("video/") {
        "video"
    } else {
        MediaItem::detect_type(path)
    };
    if kind == "audio" {
        let path = audio_path(path)?;
        state.play_media(title.into(), path, PlayerMediaKind::Audio, false);
        if !state.audio_player.is_playing() {
            return Err(state.settings.language.text("services-could-not-start-audio-on-the-output-device").to_string());
        }
        return Ok(false);
    }
    if !["video", "image"].contains(&kind) {
        return Err(
            state.settings.language.text("services-this-file-type-can-be-downloaded-but-cannot-be-presented").to_string(),
        );
    }
    // Image headers can themselves be expensive or malformed. Validation and
    // decoding belong to the shared background loader, never the click handler.
    if kind == "video" {
        state.play_media(title.into(), path.into(), PlayerMediaKind::Video, project);
    }
    deck.load_custom_content(title.into(), state.settings.language.text("workspace-media").to_string(), String::new());
    deck.preview_media = Some(crate::core::slides::MediaOutput {
        path: path.into(),
        kind: kind.into(),
    });
    if project {
        deck.project_preview();
        state.apply_theme_to_live();
        state.show_presentation_dock = state.settings.operator().show_dock;
    }
    Ok(project)
}

/// Temporary render parameters for an output, never a theme-library entry.
fn output_theme(
    base: &crate::core::models::PresentationTheme,
    media: Option<&crate::core::slides::MediaOutput>,
) -> crate::core::models::PresentationTheme {
    let mut output = base.clone();
    if let Some(media) = media {
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        media.path.hash(&mut hash);
        output.id = format!("media_output_{:x}", hash.finish());
        output.bg_type = media.kind.clone();
        output.bg_image_path = media.path.clone();
        output.is_animated = false;
        output.is_gradient = false;
        output.options.background_brightness = 1.0;
        output.options.background_overlay = 0.0;
        output.options.video = crate::core::models::theme_options::VideoOptions {
            path: if media.kind == "video" {
                media.path.clone()
            } else {
                String::new()
            },
            repeat: false,
            ..Default::default()
        };
    }
    output
}

pub fn preview_theme(
    state: &ControlPanelState,
    deck: &SlideDeck,
) -> crate::core::models::PresentationTheme {
    let mut theme = output_theme(state.preview_theme(), deck.preview_media.as_ref());
    theme.options.video.hwaccel = state.settings.hardware_acceleration;
    if let Some(media) = &state.active_player_media
        && media.kind == PlayerMediaKind::Video
        && deck.preview_media.as_ref().map(|m| &m.path) == Some(&media.path)
    {
        theme.options.video.paused = !media.is_playing;
        theme.options.video.fit = media.video_fit.clone();
    }
    theme
}

pub fn live_theme(
    state: &ControlPanelState,
    deck: &SlideDeck,
) -> crate::core::models::PresentationTheme {
    let mut theme = output_theme(state.live_theme(), deck.live_media.as_ref());
    theme.options.video.hwaccel = state.settings.hardware_acceleration;
    if let Some(media) = &state.active_player_media
        && media.kind == PlayerMediaKind::Video
        && deck.live_media.as_ref().map(|m| &m.path) == Some(&media.path)
    {
        theme.options.video.paused = !media.is_playing;
        theme.options.video.fit = media.video_fit.clone();
    }
    theme
}

pub fn audio_path(path: &str) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if rodio::Decoder::new(std::io::BufReader::new(file)).is_ok() {
        return Ok(path.into());
    }
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hash);
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    meta.len().hash(&mut hash);
    meta.modified().ok().hash(&mut hash);
    let directory = std::env::temp_dir().join("omni-presenter-chat-audio");
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let target = directory.join(format!("{:x}.wav", hash.finish()));
    if !target.exists() {
        let output = std::process::Command::new("ffmpeg")
            .args([
                "-nostdin",
                "-y",
                "-loglevel",
                "error",
                "-i",
                path,
                "-vn",
                "-acodec",
                "pcm_s16le",
            ])
            .arg(&target)
            .output()
            .map_err(|e| Language::system().format("services-could-not-convert-the-audio", &[("e", &e.to_string())]))?;
        if !output.status.success() {
            let _ = std::fs::remove_file(&target);
            return Err(Language::system().format(
                "services-unsupported-audio-format-label",
                &[("name", &String::from_utf8_lossy(&output.stderr))],
            ));
        }
    }
    Ok(target.to_string_lossy().into_owned())
}
