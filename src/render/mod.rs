//! Rendering layer: egui painting and typography.
//!
//! Values and data structures live in [`crate::core::models`]; the code that actually
//! draws them to the screen lives here, keeping the model layer free of UI code.

pub mod text;
pub mod design;
pub mod theme;

pub use text::{TextLayout, layout, render};
pub use theme::{
    get_or_load_texture, get_or_load_thumbnail, paint_gradient_rect, paint_theme_bg,
    prewarm_slides_in_egui, render_theme_lyrics,
};
