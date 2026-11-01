//! Projection overlays need absolute placement, real alpha and no WM fullscreen
//! animation. XWayland provides these on GNOME, unlike xdg-toplevel fullscreen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionWindowMode {
    Overlay,
    Fullscreen,
}

pub fn projection_window_mode() -> ProjectionWindowMode {
    #[cfg(target_os = "linux")]
    if std::env::var_os("DISPLAY").is_none_or(|display| display.is_empty()) {
        return ProjectionWindowMode::Fullscreen;
    }
    ProjectionWindowMode::Overlay
}

pub fn configure(options: &mut eframe::NativeOptions) {
    #[cfg(target_os = "linux")]
    if projection_window_mode() == ProjectionWindowMode::Overlay {
        // winit chooses the backend once per process, so this must happen before
        // the root window and the secondary projection windows are constructed.
        options.event_loop_builder = Some(Box::new(|builder| {
            use winit::platform::x11::EventLoopBuilderExtX11;
            builder.with_x11();
        }));
    }
    #[cfg(not(target_os = "linux"))]
    let _ = options;
}
