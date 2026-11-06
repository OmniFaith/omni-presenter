use rusqlite::Result;

use crate::core::models::PresentationTheme;

use super::AppDb;

impl AppDb {
    /// Seeds only base system presentation themes if empty (no folders or mock data).
    pub(super) fn seed_defaults_if_empty(&self) -> Result<()> {
        // System presentation themes (styling presets: fonts, colors, layouts)
        let theme_count: i64 = {
            let conn = self.conn.lock().unwrap();
            conn.query_row("SELECT COUNT(*) FROM themes;", [], |row| row.get(0))
                .unwrap_or(0)
        };
        if theme_count == 0 {
            for theme in crate::services::shaders::default_themes() {
                let _ = self.save_theme(&theme);
            }
            for theme in PresentationTheme::default_themes() {
                let _ = self.save_theme(&theme);
            }
        }

        Ok(())
    }
}
