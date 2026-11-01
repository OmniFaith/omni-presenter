#![recursion_limit = "256"]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod app;
pub mod core;
pub mod render;
pub mod services;
pub mod ui;

use app::PresentationApp;
use eframe::egui;

fn main() -> eframe::Result {
    #[cfg(target_os = "linux")]
    if std::env::args().nth(1).as_deref() == Some("--configure-mdns") {
        if let Err(error) = services::discovery::install::configure() {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        services::audio::silence_audio_backend_noise();
    }

    let native_options = native_options();

    eframe::run_native(
        "Presentation Control Panel",
        native_options,
        Box::new(|cc| {
            if let Some(state) = &cc.wgpu_render_state {
                services::shaders::install(state, &cc.egui_ctx);
                render::design::install(state, &cc.egui_ctx);
            }
            setup_fonts_and_style(&cc.egui_ctx);
            Ok(Box::new(PresentationApp::default()))
        }),
    )
}

fn native_options() -> eframe::NativeOptions {
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            // eframe uses the ROOT viewport flag to enable alpha-capable
            // backbuffers for every wgpu viewport, including the projector.
            .with_transparent(true)
            .with_inner_size([1260.0, 760.0])
            .with_min_inner_size([980.0, 600.0])
            .with_title("Omni Presenter"),
        ..Default::default()
    };
    services::windowing::configure(&mut options);
    options
}

fn setup_fonts_and_style(ctx: &egui::Context) {
    // Setup custom icon font with Phosphor fallback for vector icons
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "custom-icons".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/custom-icons.ttf"
        ))),
    );
    fonts.font_data.insert(
        "phosphor".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(
            egui_phosphor::Variant::Regular.font_bytes(),
        )),
    );
    if let Some(font_keys) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        font_keys.push("custom-icons".to_owned());
        font_keys.push("phosphor".to_owned());
    }
    if let Some(bytes) = core::models::fonts::color_emoji_font_bytes() {
        fonts.font_data.insert(
            "color-emoji".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(bytes)),
        );
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            if let Some(font_keys) = fonts.families.get_mut(&family) {
                font_keys.push("color-emoji".to_owned());
            }
        }
    }
    ctx.set_fonts(fonts);

    // Setup clean modern light visuals matching target design
    let mut visuals = egui::Visuals::light();
    visuals.window_fill = egui::Color32::WHITE;
    visuals.panel_fill = egui::Color32::WHITE;
    visuals.extreme_bg_color = egui::Color32::from_rgb(248, 249, 250);
    visuals.override_text_color = Some(egui::Color32::from_rgb(20, 24, 33));
    visuals.widgets.noninteractive.bg_fill = egui::Color32::WHITE;
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(226, 232, 240));
    visuals.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(20, 24, 33));
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(245, 246, 249);
    visuals.widgets.inactive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(75, 85, 99));
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(238, 241, 246);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(17, 24, 39));
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(229, 233, 240);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(17, 24, 39));
    visuals.selection.bg_fill = egui::Color32::from_rgb(224, 231, 255);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style_of(egui::Theme::Light)).clone();
    style.interaction.selectable_labels = false;
    ctx.set_style_of(egui::Theme::Light, style.clone());
    ctx.set_style_of(egui::Theme::Dark, style);
}
