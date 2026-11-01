//! Prepare live assets while the output still shows standby. In particular,
//! an image loading placeholder must never become the first fullscreen frame.
use crate::core::{
    design::{DesignDocument, LayerContent},
    models::PresentationTheme,
};
use eframe::egui;

pub fn ready(
    ctx: &egui::Context,
    theme: &PresentationTheme,
    design: Option<&DesignDocument>,
    overlay: Option<&DesignDocument>,
    themes: &[PresentationTheme],
) -> bool {
    let mut ready = match design {
        Some(design) => design_ready(ctx, design, themes),
        None => theme_ready(ctx, theme, false),
    };
    if let Some(overlay) = overlay {
        ready &= design_ready(ctx, overlay, themes);
    }
    ready
}

fn image_ready(ctx: &egui::Context, path: &str, size: u32) -> bool {
    if path.is_empty() {
        return true;
    }
    match crate::services::images::request(ctx, path, size) {
        crate::services::images::ImageLoad::Pending => false,
        crate::services::images::ImageLoad::Ready(_) => {
            size != 4096 || crate::render::get_or_load_texture(ctx, path).is_some()
        }
        // A missing/invalid file uses the normal error/fallback presentation.
        // Do not leave the output waiting forever after a decoding error.
        crate::services::images::ImageLoad::Failed(_) => true,
    }
}

fn theme_ready(ctx: &egui::Context, theme: &PresentationTheme, design: bool) -> bool {
    match theme.bg_type.as_str() {
        "image" => {
            let mut ready = image_ready(ctx, &theme.bg_image_path, 4096);
            if design {
                ready &= image_ready(ctx, &theme.bg_image_path, 2560);
            }
            ready
        }
        "video" => crate::services::video::prepare(ctx, theme),
        _ => true,
    }
}

fn design_ready(
    ctx: &egui::Context,
    design: &DesignDocument,
    themes: &[PresentationTheme],
) -> bool {
    let mut ready = true;
    if design.background_mode == "theme"
        && let Some(theme) = themes
            .iter()
            .find(|theme| Some(&theme.id) == design.theme_id.as_ref())
        {
            ready &= theme_ready(ctx, theme, true);
        }
    for layer in design
        .layers
        .iter()
        .filter(|layer| layer.visible && layer.opacity > 0.0)
    {
        if let LayerContent::Image { path, .. } = &layer.content {
            ready &= image_ready(ctx, path, 2560);
        }
    }
    ready
}
