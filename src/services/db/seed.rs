use rusqlite::{Result, params};

use crate::core::models::{PresentationTheme, color_to_hex};

use super::AppDb;

impl AppDb {
    /// Seeds only base system presentation themes if empty (no folders or mock data).
    pub(super) fn seed_defaults_if_empty(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // System presentation themes (styling presets: fonts, colors, layouts)
        let theme_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM themes;", [], |row| row.get(0))
            .unwrap_or(0);
        if theme_count == 0 {
            for theme in PresentationTheme::default_themes() {
                let _ = conn.execute(
                    "INSERT INTO themes (
                        id, name, bg_type, bg_top, bg_bottom, bg_image_path, transition,
                        font_family, font_size, font_bold, font_italic, font_underline, font_strikethrough,
                        text_color, text_case, line_spacing, letter_spacing, align_h, align_v,
                        box_x, box_y, box_w, box_h, shadow_enabled, shadow_color, shadow_blur,
                        is_gradient, is_animated, is_favorite, sort_order
                    ) VALUES (
                        ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                        ?8, ?9, ?10, ?11, ?12, ?13,
                        ?14, ?15, ?16, ?17, ?18, ?19,
                        ?20, ?21, ?22, ?23, ?24, ?25, ?26,
                        ?27, ?28, ?29, ?30
                    );",
                    params![
                        theme.id,
                        theme.name,
                        theme.bg_type,
                        color_to_hex(theme.bg_top),
                        color_to_hex(theme.bg_bottom),
                        theme.bg_image_path,
                        theme.transition,
                        theme.font_family,
                        theme.font_size,
                        theme.font_bold as i32,
                        theme.font_italic as i32,
                        theme.font_underline as i32,
                        theme.font_strikethrough as i32,
                        color_to_hex(theme.text_color),
                        theme.text_case,
                        theme.line_spacing,
                        theme.letter_spacing,
                        theme.align_h,
                        theme.align_v,
                        theme.box_x,
                        theme.box_y,
                        theme.box_w,
                        theme.box_h,
                        theme.shadow_enabled as i32,
                        color_to_hex(theme.shadow_color),
                        theme.shadow_blur,
                        theme.is_gradient as i32,
                        theme.is_animated as i32,
                        theme.is_favorite as i32,
                        theme.sort_order
                    ],
                );
            }
        }

        Ok(())
    }
}
