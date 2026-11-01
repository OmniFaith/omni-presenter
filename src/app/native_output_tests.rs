//! Real compositor regression: inspect only this probe's own native windows.
//! It uses small rectangles on each monitor and never reads the user's desktop.
use crate::{
    core::{models::PresentationTheme, preferences::AppSettings},
    services::monitor::{MonitorInfo, get_unique_monitors},
    ui::presentation::{PresentationContent, PresentationView},
};
use eframe::egui;
use std::sync::{Arc, Mutex};
use x11rb::{
    connection::Connection,
    protocol::xproto::{AtomEnum, ConnectionExt, ImageFormat},
};

struct Probe {
    monitors: Vec<MonitorInfo>,
    settings: AppSettings,
    theme: PresentationTheme,
    frame: usize,
    ids: Vec<u32>,
    result: Arc<Mutex<Option<Result<(), String>>>>,
    application: Option<super::PresentationApp>,
    image_path: std::path::PathBuf,
    video_path: std::path::PathBuf,
}

impl Probe {
    fn check_windows(&mut self, alpha: u8) -> Result<(), String> {
        let (connection, screen) = x11rb::connect(None).map_err(|e| e.to_string())?;
        let pid = connection
            .intern_atom(false, b"_NET_WM_PID")
            .unwrap()
            .reply()
            .unwrap()
            .atom;
        let state = connection
            .intern_atom(false, b"_NET_WM_STATE")
            .unwrap()
            .reply()
            .unwrap()
            .atom;
        let fullscreen = connection
            .intern_atom(false, b"_NET_WM_STATE_FULLSCREEN")
            .unwrap()
            .reply()
            .unwrap()
            .atom;
        let opaque_region = connection
            .intern_atom(false, b"_NET_WM_OPAQUE_REGION")
            .unwrap()
            .reply()
            .unwrap()
            .atom;
        let root = connection.setup().roots[screen].root;
        let children = connection
            .query_tree(root)
            .unwrap()
            .reply()
            .unwrap()
            .children;
        let mut windows = Vec::new();
        for window in children {
            let owner = connection
                .get_property(false, window, pid, AtomEnum::CARDINAL, 0, 1)
                .unwrap()
                .reply()
                .unwrap();
            if owner.value32().and_then(|mut values| values.next()) != Some(std::process::id()) {
                continue;
            }
            let geometry = connection.get_geometry(window).unwrap().reply().unwrap();
            if geometry.width != 320 || geometry.height != 180 {
                continue;
            }
            let attributes = connection
                .get_window_attributes(window)
                .unwrap()
                .reply()
                .unwrap();
            if !attributes.override_redirect {
                continue;
            }
            let monitor = self
                .monitors
                .iter()
                .find(|monitor| monitor.position == (geometry.x as i32, geometry.y as i32))
                .ok_or_else(|| {
                    format!(
                        "Projection opened at ({}, {}) instead of its target monitor",
                        geometry.x, geometry.y
                    )
                })?;
            if geometry.width as u32 != monitor.resolution.0
                || geometry.height as u32 != monitor.resolution.1
            {
                return Err("Projection geometry changed during the fade".into());
            }
            let states = connection
                .get_property(false, window, state, AtomEnum::ATOM, 0, 64)
                .unwrap()
                .reply()
                .unwrap();
            if states
                .value32()
                .is_some_and(|mut states| states.any(|state| state == fullscreen))
            {
                return Err("Projection requested native fullscreen during a fade".into());
            }
            if geometry.depth != 32 {
                return Err(format!(
                    "Projection has no alpha visual (depth {})",
                    geometry.depth
                ));
            }
            let opaque = connection
                .get_property(false, window, opaque_region, AtomEnum::CARDINAL, 0, 64)
                .unwrap()
                .reply()
                .unwrap();
            if !opaque.value.is_empty() {
                return Err("The compositor is treating the projection as an opaque window".into());
            }
            let image = connection
                .get_image(ImageFormat::Z_PIXMAP, window, 160, 90, 1, 1, u32::MAX)
                .unwrap()
                .reply()
                .map_err(|e| e.to_string())?;
            let pixel = u32::from_le_bytes(image.data[..4].try_into().unwrap());
            let actual = (pixel >> 24) as u8;
            if actual.abs_diff(alpha) > 2 {
                return Err(format!(
                    "Native output alpha is {actual}, expected {alpha}; pixel {pixel:08x}"
                ));
            }
            windows.push(window);
        }
        windows.sort();
        if windows.len() != self.monitors.len() {
            return Err(format!(
                "Expected {} native outputs, found {}",
                self.monitors.len(),
                windows.len()
            ));
        }
        if self.ids.is_empty() {
            self.ids = windows;
        } else if self.ids != windows {
            return Err("Native projection windows were recreated during the fade".into());
        }
        println!(
            "Native overlays: {} outputs, alpha {alpha}, exact geometry, no fullscreen",
            self.ids.len()
        );
        Ok(())
    }
}

impl eframe::App for Probe {
    fn on_exit(&mut self) {
        let mut result = self.result.lock().unwrap();
        if result.is_none() {
            *result = Some(Err(format!(
                "Native media probe closed before completion at frame {}",
                self.frame
            )));
        }
    }
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if self.frame >= 170 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if self.frame < 32 { ctx.set_zoom_factor(1.25); }
        if self.frame >= 32 {
            use crate::core::state::SidebarPage;
            if self.application.is_none() {
                let db = crate::services::db::AppDb::open_in_memory().unwrap();
                db.save_settings(&AppSettings {
                    chat_enabled: false,
                    server_enabled: false,
                    physical_output_enabled: false,
                    ..Default::default()
                })
                .unwrap();
                db.add_media_item(
                    "Image regression probe",
                    &self.image_path.to_string_lossy(),
                    "image",
                    "",
                )
                .unwrap();
                self.application = Some(super::PresentationApp::with_db(db));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(980.0, 600.0)));
            }
            let app = self.application.as_mut().unwrap();
            if self.frame == 45 {
                println!("Native regression: selecting a newly imported media image");
                app.control_state.selected_tab = 1;
                crate::services::media::activate(
                    &mut app.control_state,
                    &mut app.slide_deck,
                    "Image regression probe",
                    &self.image_path.to_string_lossy(),
                    "image/jpeg",
                    false,
                )
                .unwrap();
            }
            if self.frame == 75 {
                println!("Native regression: opening a design containing the image");
                let mut scene = crate::core::design::DesignDocument::default();
                scene.layers.push(crate::core::design::DesignLayer::new(
                    crate::core::design::LayerContent::Image {
                        path: self.image_path.to_string_lossy().into_owned(),
                        cover: true,
                    },
                ));
                app.db.save_design(&scene).unwrap();
                app.slide_deck.designs.push(scene.clone());
                app.slide_deck.preview_design = Some(scene.clone());
                app.design_editor.select(scene);
                app.control_state.active_sidebar_page = SidebarPage::Design;
            }
            if self.frame == 105 {
                println!("Native regression: selecting the image as standby wallpaper");
                app.control_state.settings_ui.open = true;
                let mut draft = app.control_state.settings.clone();
                draft.physical_background = crate::core::preferences::PhysicalBackground::Image {
                    path: self.image_path.to_string_lossy().into_owned(),
                };
                app.control_state.settings_ui.draft = Some(draft);
            }
            if self.frame == 116 {
                println!("Native regression: starting a 4K WebM without blocking the UI");
                app.control_state.settings_ui.open = false;
                app.control_state.active_sidebar_page = SidebarPage::Home;
                app.control_state.selected_tab = 1;
                crate::services::media::activate(
                    &mut app.control_state,
                    &mut app.slide_deck,
                    "WebM 4K",
                    &self.video_path.to_string_lossy(),
                    "video/webm",
                    false,
                )
                .unwrap();
            }
            if self.frame == 155 {
                crate::services::media::activate(
                    &mut app.control_state,
                    &mut app.slide_deck,
                    "WebM 4K",
                    &self.video_path.to_string_lossy(),
                    "video/webm",
                    true,
                )
                .unwrap();
            }
            eframe::App::ui(app, ui, frame);
            if self.frame >= 155 {
                let theme = crate::services::media::live_theme(&app.control_state, &app.slide_deck);
                for monitor in &self.monitors {
                    PresentationView::show(
                        &ctx,
                        PresentationContent {
                            text: "",
                            hidden: false,
                            theme: &theme,
                            design: None,
                            overlay: None,
                            themes: &app.control_state.themes,
                            aspect: 16.0 / 9.0,
                            active: false,
                            opacity: 1.0,
                            wallpaper_design: None,
                            title: "WebM 4K",
                            artist: "",
                            slide_index: 0,
                            is_song: false,
                        },
                        Some(monitor),
                        &self.settings,
                        false,
                        &mut false,
                        |_| true,
                    );
                }
            }
            self.frame += 1;
            if self.frame == 170 {
                *self.result.lock().unwrap() = Some(Ok(()));
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(20));
            return;
        }
        if self.monitors.is_empty() {
            self.monitors = get_unique_monitors(frame.winit_window().unwrap())
                .into_iter()
                .take(2)
                .map(|mut monitor| {
                    monitor.position.0 += 160;
                    monitor.position.1 += 160;
                    monitor.resolution = (320, 180);
                    monitor
                })
                .collect();
        }
        let phase = self.frame / 8;
        if self.frame % 8 == 7 {
            let alpha = [0, 255, 128, 0][phase];
            if let Err(error) = self.check_windows(alpha) {
                *self.result.lock().unwrap() = Some(Err(error));
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
        }
        let opacity = [0.0, 1.0, 0.5, 0.0][phase];
        for monitor in &self.monitors {
            let mut open = opacity > 0.0;
            PresentationView::show(
                &ctx,
                PresentationContent {
                    text: "",
                    hidden: false,
                    theme: &self.theme,
                    design: None,
                    overlay: None,
                    themes: std::slice::from_ref(&self.theme),
                    aspect: 16.0 / 9.0,
                    active: open,
                    opacity,
                    wallpaper_design: None,
                    title: "",
                    artist: "",
                    slide_index: 0,
                    is_song: false,
                },
                Some(monitor),
                &self.settings,
                false,
                &mut open,
                |_| true,
            );
        }
        self.frame += 1;
        ctx.request_repaint_after(std::time::Duration::from_millis(20));
    }
}

#[test]
#[ignore = "Opens small native X11/XWayland probe windows; requires a desktop compositor"]
fn native_overlays_keep_monitor_geometry_and_real_alpha_through_entry_and_exit() {
    let mut options = crate::native_options();
    options.viewport = options
        .viewport
        .with_inner_size([240.0, 140.0])
        .with_min_inner_size([100.0, 100.0])
        .with_active(false)
        .with_decorations(false)
        .with_mouse_passthrough(true)
        .with_override_redirect(true)
        .with_window_type(egui::X11WindowType::Notification)
        .with_position([30.0, 30.0])
        .with_title("omni presenter native projection probe");
    options.event_loop_builder = Some(Box::new(|builder| {
        use winit::platform::x11::EventLoopBuilderExtX11;
        builder.with_x11().with_any_thread(true);
    }));
    let result = Arc::new(Mutex::new(None));
    let output = result.clone();
    let image_path = std::env::temp_dir().join(format!(
        "omni-presenter-native-image-{}.jpg",
        crate::core::design::new_id()
    ));
    image::RgbImage::from_pixel(1280, 720, image::Rgb([80, 140, 180]))
        .save_with_format(&image_path, image::ImageFormat::Jpeg)
        .unwrap();
    let cleanup = image_path.clone();
    let video_path = image_path.with_extension("webm");
    let generated = std::process::Command::new("ffmpeg")
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
        .arg(&video_path)
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let cleanup_video = video_path.clone();
    let finished = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let watchdog = finished.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(30));
        if !watchdog.load(std::sync::atomic::Ordering::Relaxed) {
            eprintln!("Native media regression did not finish within 30 seconds");
            unsafe {
                libc::raise(libc::SIGABRT);
            }
        }
    });
    eframe::run_native(
        "omni presenter native projection probe",
        options,
        Box::new(move |cc| {
            crate::setup_fonts_and_style(&cc.egui_ctx);
            if let Some(state) = &cc.wgpu_render_state {
                crate::services::shaders::install(state, &cc.egui_ctx);
                crate::render::design::install(state, &cc.egui_ctx);
            }
            Ok(Box::new(Probe {
                monitors: Vec::new(),
                settings: Default::default(),
                theme: PresentationTheme {
                    bg_type: "color".into(),
                    is_gradient: false,
                    bg_top: egui::Color32::from_rgb(180, 40, 20),
                    bg_bottom: egui::Color32::from_rgb(180, 40, 20),
                    ..Default::default()
                },
                frame: 0,
                ids: Vec::new(),
                result: output,
                application: None,
                image_path,
                video_path,
            }))
        }),
    )
    .unwrap();
    finished.store(true, std::sync::atomic::Ordering::Relaxed);
    let _ = std::fs::remove_file(cleanup);
    let _ = std::fs::remove_file(cleanup_video);
    let result = result
        .lock()
        .unwrap()
        .take()
        .expect("Probe must finish its lifecycle");
    result.unwrap();
}
