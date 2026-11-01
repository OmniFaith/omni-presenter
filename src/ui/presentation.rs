use crate::core::models::PresentationTheme;
use crate::render::paint_theme_bg;
use crate::services::monitor::MonitorInfo;
use eframe::egui::{self, Color32};

pub const PRESENTATION_VIEWPORT_ID: &str = "presentation_overlay_viewport";

pub fn viewport_id(monitor_name: &str) -> egui::ViewportId {
    egui::ViewportId::from_hash_of((PRESENTATION_VIEWPORT_ID, monitor_name))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationAction {
    ToggleHide,
    NextSlide,
    PrevSlide,
    FirstSlide,
    LastSlide,
    Search,
}

pub struct PresentationView;

pub struct PresentationContent<'a> {
    pub text: &'a str,
    pub hidden: bool,
    pub theme: &'a PresentationTheme,
    pub design: Option<&'a crate::core::design::DesignDocument>,
    pub overlay: Option<&'a crate::core::design::DesignDocument>,
    pub themes: &'a [PresentationTheme],
    pub aspect: f32,
    pub active: bool,
    pub opacity: f32,
    pub wallpaper_design: Option<&'a crate::core::design::DesignDocument>,
    pub title: &'a str,
    pub artist: &'a str,
    pub slide_index: usize,
    pub is_song: bool,
}

impl PresentationView {
    pub fn show(
        ctx: &egui::Context,
        content: PresentationContent<'_>,
        target_monitor: Option<&MonitorInfo>,
        settings: &crate::core::preferences::AppSettings,
        animate: bool,
        is_open: &mut bool,
        mut on_action: impl FnMut(PresentationAction) -> bool,
    ) {
        let PresentationContent {
            text: content_text,
            hidden: is_hidden,
            theme,
            design,
            overlay,
            themes,
            aspect: aspect_ratio,
            active,
            opacity,
            wallpaper_design,
            title: _,
            artist: _,
            slide_index: _,
            is_song: _,
        } = content;
        let viewport_id = viewport_id(target_monitor.map(|m| m.name.as_str()).unwrap_or("default"));
        let monitor_bg = target_monitor
            .map(|m| settings.wallpaper_for_monitor(&m.name))
            .unwrap_or(&settings.physical_background);
        let transparent_wallpaper = matches!(
            monitor_bg,
            crate::core::preferences::PhysicalBackground::Transparent
        );
        let has_live_frame = active || opacity > 0.0;
        let overlay_window = crate::services::windowing::projection_window_mode()
            == crate::services::windowing::ProjectionWindowMode::Overlay;
        // The native-Wayland fallback cannot blend a fullscreen surface with
        // the desktop. True overlays can fade the entire frame directly.
        let stable_background = !overlay_window && transparent_wallpaper && has_live_frame;

        // Only the fallback changes fullscreen/visibility. Overlay windows keep
        // their native geometry, mapping and identity throughout the lifecycle.
        let transparent_standby = transparent_wallpaper && !has_live_frame;

        let builder = if let Some(target_mon) = target_monitor {
            let rect = target_mon.viewport_rect(ctx.zoom_factor());

            let builder = egui::ViewportBuilder::default()
                .with_title(settings.language.text("output-title"))
                .with_decorations(false)
                .with_transparent(true)
                .with_has_shadow(false)
                .with_fullscreen(!overlay_window && !transparent_standby)
                .with_resizable(false)
                .with_position(rect.min)
                .with_inner_size(rect.size());
            if overlay_window || transparent_standby {
                builder
            } else {
                builder.with_monitor(target_mon.winit_index)
            }
        } else {
            egui::ViewportBuilder::default()
                .with_title(settings.language.text("output-title"))
                .with_decorations(false)
                .with_transparent(true)
                .with_has_shadow(false)
                .with_fullscreen(!overlay_window && !transparent_standby)
        };

        #[allow(unused_mut)]
        let mut builder = builder
            .with_active(false)
            .with_visible(overlay_window || !transparent_standby)
            .with_mouse_passthrough(!active)
            .with_window_level(egui::WindowLevel::AlwaysOnTop);
        #[cfg(target_os = "linux")]
        if overlay_window {
            // An unmanaged overlay has no maximize/map animation and the WM
            // cannot relocate or constrain it to another monitor's work area.
            builder = builder
                .with_override_redirect(true)
                .with_window_type(egui::X11WindowType::Notification);
        }
        ctx.show_viewport_immediate(viewport_id, builder, |ui, _class| {
            let standby_id = egui::Id::unique("physical_standby").with(viewport_id);
            let was_standby = ui.data_mut(|data| {
                let previous = data.get_temp::<bool>(standby_id).unwrap_or(true);
                data.insert_temp(standby_id, transparent_standby);
                previous
            });
            if !overlay_window && was_standby && !transparent_standby
                && let Some(monitor) = target_monitor {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::SetMonitor(monitor.winit_index));
                }
            // Guarantee full transparency base on the presentation viewport
            ui.visuals_mut().panel_fill = Color32::TRANSPARENT;
            ui.visuals_mut().window_fill = Color32::TRANSPARENT;
            ui.visuals_mut().extreme_bg_color = Color32::TRANSPARENT;

            use crate::core::commands::Command;
            let shortcuts = &settings.shortcuts;
            if shortcuts.pressed(ui.ctx(), Command::Search) {
                on_action(PresentationAction::Search);
            }
            if settings.operator().keyboard_shortcuts
                && (shortcuts.pressed(ui.ctx(), Command::StopPresentation)
                    || shortcuts.pressed(ui.ctx(), Command::TogglePresentation))
            {
                *is_open = false;
            }
            if settings.operator().keyboard_shortcuts
                && shortcuts.pressed(ui.ctx(), Command::ToggleHide)
            {
                on_action(PresentationAction::ToggleHide);
            }
            if settings.operator().keyboard_shortcuts
                && shortcuts.pressed(ui.ctx(), Command::NextSlide)
            {
                if !on_action(PresentationAction::NextSlide) {
                    *is_open = false;
                }
            }
            if settings.operator().keyboard_shortcuts
                && shortcuts.pressed(ui.ctx(), Command::PreviousSlide)
            {
                on_action(PresentationAction::PrevSlide);
            }
            if settings.operator().keyboard_shortcuts
                && shortcuts.pressed(ui.ctx(), Command::FirstSlide)
            {
                on_action(PresentationAction::FirstSlide);
            }
            if settings.operator().keyboard_shortcuts
                && shortcuts.pressed(ui.ctx(), Command::LastSlide)
            {
                on_action(PresentationAction::LastSlide);
            }

            // Window close request from OS
            if ui.input(|i| i.viewport().close_requested()) {
                *is_open = false;
                if settings.persistent_physical_output {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::CancelClose);
                }
            }

            // Panel with theme background and styled lyrics
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ui, |ui| {
                    let full_rect = ui.max_rect();
                    if opacity < 1.0 || settings.use_wallpaper_in_presentation {
                        paint_wallpaper(
                            ui.painter(),
                            full_rect,
                            monitor_bg,
                            wallpaper_design,
                            themes,
                        );
                    }
                    let width = full_rect.width().min(full_rect.height() * aspect_ratio);
                    let output_rect = egui::Rect::from_center_size(
                        full_rect.center(),
                        egui::vec2(width, width / aspect_ratio),
                    );

                    // Wallpaper belongs to standby or persistent presentation background if enabled.
                    let mut presentation = ui.painter().clone();
                    presentation.multiply_opacity(opacity);
                    let background = if stable_background {
                        ui.painter()
                    } else {
                        &presentation
                    };
                    if has_live_frame && design.is_none() && !settings.use_wallpaper_in_presentation {
                        paint_theme_bg(background, output_rect, theme, 0.0);
                    } else if stable_background
                        && let Some(design) = design {
                            paint_design_background(background, output_rect, design, themes);
                        }
                    let content_rect = if stable_background && animate {
                        output_rect.translate(egui::vec2(
                            0.0,
                            (1.0 - opacity) * 8.0 * output_rect.height() / 720.0,
                        ))
                    } else {
                        output_rect
                    };
                    if opacity > 0.0 && !is_hidden {
                        if let Some(design) = design {
                            if stable_background {
                                let mut foreground = design.clone();
                                foreground.background_mode = "transparent".into();
                                crate::render::design::paint_with_transition(
                                    &presentation,
                                    content_rect,
                                    &foreground,
                                    egui::Id::NULL.with("physical_design_transition"),
                                    animate,
                                );
                            } else {
                                crate::render::design::paint_themed_with_transition(
                                    &presentation,
                                    content_rect,
                                    design,
                                    themes,
                                    egui::Id::NULL.with("physical_design_transition"),
                                    animate,
                                );
                            }
                        } else {
                            crate::services::effects::render(
                                &presentation,
                                content_rect,
                                theme,
                                content_text,
                                egui::Id::NULL.with("physical_slide_transition"),
                                animate,
                            );
                        }
                        if let Some(overlay) = overlay {
                            crate::render::design::paint_with_transition(
                                &presentation,
                                content_rect,
                                overlay,
                                egui::Id::NULL.with("physical_overlay_transition"),
                                animate,
                            );
                        }
                    }
                });
        });
    }
}


fn paint_design_background(
    painter: &egui::Painter,
    rect: egui::Rect,
    design: &crate::core::design::DesignDocument,
    themes: &[PresentationTheme],
) {
    match design.background_mode.as_str() {
        "color" => {
            let [r, g, b, a] = design.background;
            painter.rect_filled(rect, 0.0, Color32::from_rgba_unmultiplied(r, g, b, a));
        }
        "theme" => {
            if let Some(theme) = themes
                .iter()
                .find(|theme| Some(&theme.id) == design.theme_id.as_ref())
                .or_else(|| themes.first())
            {
                paint_theme_bg(painter, rect, theme, 0.0);
            }
        }
        _ => {
            if !design.overlay
                && let Some(theme) = themes
                    .iter()
                    .find(|theme| Some(&theme.id) == design.theme_id.as_ref())
                    .or_else(|| themes.first())
                {
                    paint_theme_bg(painter, rect, theme, 0.0);
                }
        }
    }
}

pub fn paint_wallpaper(
    painter: &egui::Painter,
    rect: egui::Rect,
    background: &crate::core::preferences::PhysicalBackground,
    design: Option<&crate::core::design::DesignDocument>,
    themes: &[PresentationTheme],
) {
    use crate::core::preferences::PhysicalBackground;
    match background {
        PhysicalBackground::Transparent => {}
        PhysicalBackground::Color { rgb } => {
            painter.rect_filled(rect, 0.0, Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
        }
        PhysicalBackground::Image { path } | PhysicalBackground::Video { path } => {
            let theme = PresentationTheme {
                id: "physical_wallpaper".into(),
                bg_type: if matches!(background, PhysicalBackground::Video { .. }) {
                    "video"
                } else {
                    "image"
                }
                .into(),
                bg_image_path: path.clone(),
                bg_top: Color32::BLACK,
                bg_bottom: Color32::BLACK,
                is_gradient: false,
                ..Default::default()
            };
            paint_theme_bg(painter, rect, &theme, 0.0);
        }
        PhysicalBackground::Design { .. } => {
            if let Some(design) = design {
                crate::render::design::paint_themed(painter, rect, design, themes);
            }
        }
        PhysicalBackground::Theme { id } => {
            if let Some(theme) = themes.iter().find(|theme| theme.id == *id) {
                paint_theme_bg(painter, rect, theme, 0.0);
            }
        }
    }
}
