use rusqlite::{Result, params};

use crate::core::models::{PresentationTheme, color_to_hex, hex_to_color};

use super::AppDb;

#[cfg(test)]
#[path = "theme_order_tests.rs"]
mod order_tests;

impl AppDb {
    /// Retorna todos os temas salvos no banco SQLite ordenados por favoritos e ordem de ordenação
    pub fn get_all_themes(&self) -> Result<Vec<PresentationTheme>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, bg_type, bg_top, bg_bottom, bg_image_path, transition,
                    font_family, font_size, font_weight, font_bold, font_italic, font_underline, font_strikethrough,
                    text_color, text_case, line_spacing, letter_spacing, align_h, align_v,
                    box_x, box_y, box_w, box_h, outline_enabled, outline_color, outline_width,
                    shadow_enabled, shadow_color, shadow_blur, shadow_offset_x, shadow_offset_y,
                    box_bg_enabled, box_bg_color, box_border_enabled, box_border_color, box_border_width,
                    box_radius, box_padding, is_gradient, is_animated, is_favorite, sort_order, options
             FROM themes
             ORDER BY is_favorite DESC, sort_order ASC, rowid ASC;",
        )?;

        let rows = stmt.query_map([], |row| {
            let bg_top_hex: String = row.get(3)?;
            let bg_bottom_hex: String = row.get(4)?;
            let text_color_hex: String = row.get(14)?;
            let outline_color_hex: String = row.get(25)?;
            let shadow_color_hex: String = row.get(28)?;
            let box_bg_color_hex: String = row.get(33)?;
            let box_border_color_hex: String = row.get(35)?;

            Ok(PresentationTheme {
                id: row.get(0)?,
                name: row.get(1)?,
                bg_type: row.get(2)?,
                bg_top: hex_to_color(&bg_top_hex),
                bg_bottom: hex_to_color(&bg_bottom_hex),
                bg_image_path: row.get(5)?,
                transition: row.get(6)?,
                font_family: row.get(7)?,
                font_size: row.get(8)?,
                font_weight: row.get::<_, u16>(9).unwrap_or(700),
                font_bold: row.get::<_, i32>(10)? != 0,
                font_italic: row.get::<_, i32>(11)? != 0,
                font_underline: row.get::<_, i32>(12)? != 0,
                font_strikethrough: row.get::<_, i32>(13)? != 0,
                text_color: hex_to_color(&text_color_hex),
                text_case: row.get(15)?,
                line_spacing: row.get(16)?,
                letter_spacing: row.get(17)?,
                align_h: row.get(18)?,
                align_v: row.get(19)?,
                box_x: row.get(20)?,
                box_y: row.get(21)?,
                box_w: row.get(22)?,
                box_h: row.get(23)?,
                outline_enabled: row.get::<_, i32>(24).unwrap_or(0) != 0,
                outline_color: hex_to_color(&outline_color_hex),
                outline_width: row.get::<_, f32>(26).unwrap_or(2.5),
                shadow_enabled: row.get::<_, i32>(27)? != 0,
                shadow_color: hex_to_color(&shadow_color_hex),
                shadow_blur: row.get(29)?,
                shadow_offset_x: row.get::<_, f32>(30).unwrap_or(2.0),
                shadow_offset_y: row.get::<_, f32>(31).unwrap_or(2.5),
                box_bg_enabled: row.get::<_, i32>(32).unwrap_or(0) != 0,
                box_bg_color: hex_to_color(&box_bg_color_hex),
                box_border_enabled: row.get::<_, i32>(34).unwrap_or(0) != 0,
                box_border_color: hex_to_color(&box_border_color_hex),
                box_border_width: row.get::<_, f32>(36).unwrap_or(1.5),
                box_radius: row.get::<_, f32>(37).unwrap_or(8.0),
                box_padding: row.get::<_, f32>(38).unwrap_or(16.0),
                is_gradient: row.get::<_, i32>(39)? != 0,
                is_animated: row.get::<_, i32>(40)? != 0,
                is_favorite: row.get::<_, i32>(41)? != 0,
                sort_order: row.get(42)?,
                options: serde_json::from_str(&row.get::<_, String>(43)?).unwrap_or_default(),
            })
        })?;

        let mut themes = Vec::new();
        for t in rows {
            themes.push(t?);
        }
        Ok(themes)
    }

    /// Salva ou atualiza um tema no SQLite
    pub fn save_theme(&self, theme: &PresentationTheme) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO themes (
                id, name, bg_type, bg_top, bg_bottom, bg_image_path, transition,
                font_family, font_size, font_weight, font_bold, font_italic, font_underline, font_strikethrough,
                text_color, text_case, line_spacing, letter_spacing, align_h, align_v,
                box_x, box_y, box_w, box_h, outline_enabled, outline_color, outline_width,
                shadow_enabled, shadow_color, shadow_blur, shadow_offset_x, shadow_offset_y,
                box_bg_enabled, box_bg_color, box_border_enabled, box_border_color, box_border_width,
                box_radius, box_padding, is_gradient, is_animated, is_favorite, sort_order, options, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                ?15, ?16, ?17, ?18, ?19, ?20,
                ?21, ?22, ?23, ?24, ?25, ?26, ?27,
                ?28, ?29, ?30, ?31, ?32,
                ?33, ?34, ?35, ?36, ?37,
                ?38, ?39, ?40, ?41, ?42, ?43, ?44, CURRENT_TIMESTAMP
            )
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                bg_type = excluded.bg_type,
                bg_top = excluded.bg_top,
                bg_bottom = excluded.bg_bottom,
                bg_image_path = excluded.bg_image_path,
                transition = excluded.transition,
                font_family = excluded.font_family,
                font_size = excluded.font_size,
                font_weight = excluded.font_weight,
                font_bold = excluded.font_bold,
                font_italic = excluded.font_italic,
                font_underline = excluded.font_underline,
                font_strikethrough = excluded.font_strikethrough,
                text_color = excluded.text_color,
                text_case = excluded.text_case,
                line_spacing = excluded.line_spacing,
                letter_spacing = excluded.letter_spacing,
                align_h = excluded.align_h,
                align_v = excluded.align_v,
                box_x = excluded.box_x,
                box_y = excluded.box_y,
                box_w = excluded.box_w,
                box_h = excluded.box_h,
                outline_enabled = excluded.outline_enabled,
                outline_color = excluded.outline_color,
                outline_width = excluded.outline_width,
                shadow_enabled = excluded.shadow_enabled,
                shadow_color = excluded.shadow_color,
                shadow_blur = excluded.shadow_blur,
                shadow_offset_x = excluded.shadow_offset_x,
                shadow_offset_y = excluded.shadow_offset_y,
                box_bg_enabled = excluded.box_bg_enabled,
                box_bg_color = excluded.box_bg_color,
                box_border_enabled = excluded.box_border_enabled,
                box_border_color = excluded.box_border_color,
                box_border_width = excluded.box_border_width,
                box_radius = excluded.box_radius,
                box_padding = excluded.box_padding,
                is_gradient = excluded.is_gradient,
                is_animated = excluded.is_animated,
                is_favorite = excluded.is_favorite,
                sort_order = excluded.sort_order,
                options = excluded.options,
                updated_at = CURRENT_TIMESTAMP;",
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
                theme.font_weight as i32,
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
                theme.outline_enabled as i32,
                color_to_hex(theme.outline_color),
                theme.outline_width,
                theme.shadow_enabled as i32,
                color_to_hex(theme.shadow_color),
                theme.shadow_blur,
                theme.shadow_offset_x,
                theme.shadow_offset_y,
                theme.box_bg_enabled as i32,
                color_to_hex(theme.box_bg_color),
                theme.box_border_enabled as i32,
                color_to_hex(theme.box_border_color),
                theme.box_border_width,
                theme.box_radius,
                theme.box_padding,
                theme.is_gradient as i32,
                theme.is_animated as i32,
                theme.is_favorite as i32,
                theme.sort_order,
                serde_json::to_string(&theme.options).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
            ],
        )?;
        Ok(())
    }

    /// Exclui um tema do SQLite pelo ID
    pub fn delete_theme(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM themes WHERE id = ?1;", params![id])?;
        Ok(())
    }

    /// Atualiza o status de favorito de um tema
    pub fn update_theme_favorite(&self, id: &str, is_fav: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE themes SET is_favorite = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2;",
            params![is_fav as i32, id],
        )?;
        Ok(())
    }

    /// Atualiza a ordenação da lista de temas no SQLite
    pub fn update_themes_order(&self, ids: &[String]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let transaction = conn.transaction()?;
        for (idx, id) in ids.iter().enumerate() {
            let updated = transaction.execute(
                "UPDATE themes SET sort_order = ?1 WHERE id = ?2;",
                params![idx as i32, id],
            )?;
            if updated != 1 {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }
        }
        transaction.commit()
    }

    /// Restaura todos os temas padrão de apresentação no SQLite
    pub fn restore_default_presentation_themes(&self) -> Result<()> {
        {
            let conn = self.conn.lock().unwrap();
            let _ = conn.execute(
                "DELETE FROM themes WHERE id IN ('hands_water', 'starry_sky', 'deep_emerald', 'celestial_violet', 'scripture_serif', 'shader_nebula', 'shader_grid', 'shader_prism');",
                [],
            );
            let _ = conn.execute(
                "UPDATE app_settings SET value = REPLACE(REPLACE(value, '\"hands_water\"', '\"obsidian_pro\"'), '\"starry_sky\"', '\"obsidian_pro\"') WHERE id = 1;",
                [],
            );
        }
        for theme in crate::services::shaders::default_themes() {
            self.save_theme(&theme)?;
        }
        for theme in PresentationTheme::default_themes() {
            self.save_theme(&theme)?;
        }
        Ok(())
    }
}
