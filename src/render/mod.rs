//! Rendering layer: egui painting and typography.
//!
//! Values and data structures live in [`crate::core::models`]; the code that actually
//! draws them to the screen lives here, keeping the model layer free of UI code.

pub mod text;
pub(crate) mod fonts;
pub(crate) mod web_text;
pub mod design;
pub mod theme;

pub use text::{TextLayout, layout, render, render_blurred, render_with_override_color};
pub use theme::{
    get_or_load_texture, get_or_load_thumbnail, paint_corner_mask, paint_gradient_rect,
    paint_theme_bg, paint_textured_rounded_rect, prewarm_slides_in_egui, render_theme_lyrics,
    render_theme_lyrics_blurred, render_theme_lyrics_with_color,
};
