use super::*;
use crate::core::{models::PresentationTheme, slides::SlideDeck, state::ControlPanelState};
use eframe::egui;
use std::{
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[test]
fn webm_4k_starts_without_probing_on_ui_and_shares_bounded_frames() {
    if Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_err()
    {
        return;
    }
    let path =
        std::env::temp_dir().join(format!("omni-presenter-4k-{}.webm", crate::core::design::new_id()));
    let generated = Command::new("ffmpeg")
        .args([
            "-y",
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:size=3840x2160:rate=2",
            "-t",
            "1",
            "-c:v",
            "libvpx-vp9",
            "-deadline",
            "realtime",
            "-cpu-used",
            "8",
            "-threads",
            "2",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let mut state = ControlPanelState::default();
    let mut deck = SlideDeck::new("Letra ao vivo");
    let themes = state.themes.clone();
    crate::services::media::activate(
        &mut state,
        &mut deck,
        "Vídeo 4K",
        &path.to_string_lossy(),
        "video/webm",
        false,
    )
    .unwrap();
    assert_eq!(state.themes, themes);
    assert_eq!(
        state.active_player_media.as_ref().unwrap().duration_secs,
        0.0,
        "Metadata must not be probed inside the click handler"
    );
    let output: PresentationTheme = crate::services::media::preview_theme(&state, &deck);
    let ctx = egui::Context::default();
    let deadline = Instant::now() + Duration::from_secs(12);
    while !prepare(&ctx, &output) {
        let mut frame = ctx.run_ui(Default::default(), |ui| {
            ui.label("Interface respondendo durante o carregamento");
        });
        frame.textures_delta.clear();
        state.refresh_media_duration();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(20));
    }
    let mut frame = ctx.run_ui(Default::default(), |ui| {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 112.5));
        paint_video(
            ui.painter(),
            rect,
            &path.to_string_lossy(),
            &output.options.video,
            0.0,
        );
        paint_video(
            ui.painter(),
            rect,
            &path.to_string_lossy(),
            &crate::core::models::theme_options::VideoOptions {
                repeat: false,
                ..Default::default()
            },
            0.0,
        );
    });
    frame.textures_delta.clear();
    let cache = cache(&ctx);
    let cache = cache.lock().unwrap();
    assert_eq!(
        cache.len(),
        1,
        "Player, preview and projection must share the decoder"
    );
    let entry = cache.values().next().unwrap();
    let decoded = entry.shared.frame.lock().unwrap();
    let size = decoded.as_ref().unwrap().image.size;
    assert!(
        size[0] <= 1280 && size[1] <= 720,
        "4K decoding must not upload 4K RGBA frames on every UI repaint"
    );
    entry
        .shared
        .stop
        .store(true, std::sync::atomic::Ordering::Relaxed);
    drop(decoded);
    drop(cache);
    std::fs::remove_file(path).unwrap();
}
