use rusqlite::Result;

use super::AppDb;

impl AppDb {
    /// Creates database tables if they do not exist
    pub(super) fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("CREATE TABLE IF NOT EXISTS design_documents (id TEXT PRIMARY KEY, value TEXT NOT NULL);", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS app_settings (id INTEGER PRIMARY KEY CHECK(id = 1), value TEXT NOT NULL);", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS bible_translations (id TEXT PRIMARY KEY, value TEXT NOT NULL);", [])?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_migrations (name TEXT PRIMARY KEY);",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
            );",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS songs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                artist TEXT NOT NULL DEFAULT '',
                text TEXT NOT NULL,
                tags TEXT NOT NULL DEFAULT '',
                folder TEXT NOT NULL DEFAULT '',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS song_versions (
                song_id INTEGER PRIMARY KEY,
                group_id TEXT NOT NULL
            );",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_song_versions_group ON song_versions(group_id);",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS themes (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                bg_type TEXT NOT NULL DEFAULT 'gradient',
                bg_top TEXT NOT NULL DEFAULT '#3B82F6',
                bg_bottom TEXT NOT NULL DEFAULT '#8B5CF6',
                bg_image_path TEXT NOT NULL DEFAULT '',
                transition TEXT NOT NULL DEFAULT 'fade',
                font_family TEXT NOT NULL DEFAULT 'Montserrat',
                font_size REAL NOT NULL DEFAULT 44.0,
                font_weight INTEGER NOT NULL DEFAULT 700,
                font_bold INTEGER NOT NULL DEFAULT 1,
                font_italic INTEGER NOT NULL DEFAULT 0,
                font_underline INTEGER NOT NULL DEFAULT 0,
                font_strikethrough INTEGER NOT NULL DEFAULT 0,
                text_color TEXT NOT NULL DEFAULT '#FFFFFF',
                text_case TEXT NOT NULL DEFAULT 'original',
                line_spacing REAL NOT NULL DEFAULT 1.30,
                letter_spacing REAL NOT NULL DEFAULT 0.0,
                align_h TEXT NOT NULL DEFAULT 'center',
                align_v TEXT NOT NULL DEFAULT 'center',
                box_x REAL NOT NULL DEFAULT 0.12,
                box_y REAL NOT NULL DEFAULT 0.20,
                box_w REAL NOT NULL DEFAULT 0.76,
                box_h REAL NOT NULL DEFAULT 0.60,
                outline_enabled INTEGER NOT NULL DEFAULT 0,
                outline_color TEXT NOT NULL DEFAULT '#000000',
                outline_width REAL NOT NULL DEFAULT 2.5,
                shadow_enabled INTEGER NOT NULL DEFAULT 1,
                shadow_color TEXT NOT NULL DEFAULT '#000000',
                shadow_blur REAL NOT NULL DEFAULT 4.0,
                shadow_offset_x REAL NOT NULL DEFAULT 2.0,
                shadow_offset_y REAL NOT NULL DEFAULT 2.5,
                box_bg_enabled INTEGER NOT NULL DEFAULT 0,
                box_bg_color TEXT NOT NULL DEFAULT '#00000080',
                box_border_enabled INTEGER NOT NULL DEFAULT 0,
                box_border_color TEXT NOT NULL DEFAULT '#FFFFFF40',
                box_border_width REAL NOT NULL DEFAULT 1.5,
                box_radius REAL NOT NULL DEFAULT 8.0,
                box_padding REAL NOT NULL DEFAULT 16.0,
                is_gradient INTEGER NOT NULL DEFAULT 1,
                is_animated INTEGER NOT NULL DEFAULT 0,
                is_favorite INTEGER NOT NULL DEFAULT 0,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS category_folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category TEXT NOT NULL,
                name TEXT NOT NULL,
                UNIQUE(category, name)
            );",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS media_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_type TEXT NOT NULL DEFAULT 'image',
                folder TEXT NOT NULL DEFAULT '',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS custom_texts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                folder TEXT NOT NULL DEFAULT '',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS saved_verses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                reference TEXT NOT NULL,
                version TEXT NOT NULL DEFAULT 'NVI',
                text TEXT NOT NULL,
                folder TEXT NOT NULL DEFAULT '',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
            [],
        )?;

        // Migrações seguras de colunas em bancos existentes
        let migrations = [
            "ALTER TABLE themes ADD COLUMN options TEXT NOT NULL DEFAULT '{}';",
            "ALTER TABLE themes ADD COLUMN font_weight INTEGER NOT NULL DEFAULT 700;",
            "ALTER TABLE themes ADD COLUMN outline_enabled INTEGER NOT NULL DEFAULT 0;",
            "ALTER TABLE themes ADD COLUMN outline_color TEXT NOT NULL DEFAULT '#000000';",
            "ALTER TABLE themes ADD COLUMN outline_width REAL NOT NULL DEFAULT 2.5;",
            "ALTER TABLE themes ADD COLUMN shadow_offset_x REAL NOT NULL DEFAULT 2.0;",
            "ALTER TABLE themes ADD COLUMN shadow_offset_y REAL NOT NULL DEFAULT 2.5;",
            "ALTER TABLE themes ADD COLUMN box_bg_enabled INTEGER NOT NULL DEFAULT 0;",
            "ALTER TABLE themes ADD COLUMN box_bg_color TEXT NOT NULL DEFAULT '#00000080';",
            "ALTER TABLE themes ADD COLUMN box_border_enabled INTEGER NOT NULL DEFAULT 0;",
            "ALTER TABLE themes ADD COLUMN box_border_color TEXT NOT NULL DEFAULT '#FFFFFF40';",
            "ALTER TABLE themes ADD COLUMN box_border_width REAL NOT NULL DEFAULT 1.5;",
            "ALTER TABLE themes ADD COLUMN box_radius REAL NOT NULL DEFAULT 8.0;",
            "ALTER TABLE themes ADD COLUMN box_padding REAL NOT NULL DEFAULT 16.0;",
        ];
        for mig in migrations {
            let _ = conn.execute(mig, []);
        }

        // Limpeza de pastas mockadas herdadas em bancos existentes
        let _ = conn.execute("DELETE FROM folders WHERE name = 'Cantor Cristão';", []);
        let _ = conn.execute(
            "DELETE FROM category_folders WHERE name IN ('Geral', 'Favoritos');",
            [],
        );
        let _ = conn.execute(
            "UPDATE songs SET folder = '' WHERE folder = 'Cantor Cristão';",
            [],
        );
        let _ = conn.execute(
            "UPDATE media_items SET folder = '' WHERE folder = 'Geral';",
            [],
        );
        let _ = conn.execute(
            "UPDATE custom_texts SET folder = '' WHERE folder = 'Geral';",
            [],
        );
        let _ = conn.execute(
            "UPDATE saved_verses SET folder = '' WHERE folder = 'Favoritos';",
            [],
        );
        let _ = conn.execute("DELETE FROM themes WHERE id LIKE 'plugin_shader:%';", []);
        let _ = conn.execute(
            "DELETE FROM themes WHERE id IN ('hands_water', 'starry_sky');",
            [],
        );
        let _ = conn.execute(
            "UPDATE app_settings SET value = REPLACE(REPLACE(value, '\"hands_water\"', '\"obsidian_pro\"'), '\"starry_sky\"', '\"obsidian_pro\"') WHERE id = 1;",
            [],
        );

        Ok(())
    }
    pub fn install_dynamic_themes(&self) -> Result<()> {
        let installed: bool = self.conn.lock().unwrap().query_row(
            "SELECT EXISTS(SELECT 1 FROM app_migrations WHERE name = 'dynamic_backgrounds_v1')",
            [],
            |row| row.get(0),
        )?;
        if installed {
            return Ok(());
        }
        let existing = self.get_all_themes()?;
        for theme in crate::services::shaders::default_themes() {
            if !existing.iter().any(|saved| saved.id == theme.id) {
                self.save_theme(&theme)?;
            }
        }
        self.conn.lock().unwrap().execute(
            "INSERT OR IGNORE INTO app_migrations (name) VALUES ('dynamic_backgrounds_v1')",
            [],
        )?;
        Ok(())
    }
    pub fn upgrade_default_presentation_themes(&self) -> Result<()> {
        let installed: bool = self.conn.lock().unwrap().query_row(
            "SELECT EXISTS(SELECT 1 FROM app_migrations WHERE name = 'presentation_themes_v3_purge_legacy_images')",
            [],
            |row| row.get(0),
        )?;
        if installed {
            return Ok(());
        }

        let legacy_ids = [
            "hands_water",
            "starry_sky",
            "amber_gold",
            "warm_orange",
            "violet_blue",
            "royal_blue",
            "classic_black",
            "clean_white",
            "teal_cyan",
        ];

        {
            let conn = self.conn.lock().unwrap();
            for id in legacy_ids {
                let _ = conn.execute("DELETE FROM themes WHERE id = ?1;", [id]);
            }
            let _ = conn.execute(
                "UPDATE app_settings SET value = REPLACE(REPLACE(value, '\"hands_water\"', '\"obsidian_pro\"'), '\"starry_sky\"', '\"obsidian_pro\"') WHERE id = 1;",
                [],
            );
        }

        for theme in crate::core::models::PresentationTheme::default_themes() {
            self.save_theme(&theme)?;
        }

        for theme in crate::services::shaders::default_themes() {
            self.save_theme(&theme)?;
        }

        let conn = self.conn.lock().unwrap();
        for id in legacy_ids {
            let _ = conn.execute("DELETE FROM themes WHERE id = ?1;", [id]);
        }

        let _ = conn.execute(
            "INSERT OR IGNORE INTO app_migrations (name) VALUES ('presentation_themes_v3_purge_legacy_images')",
            [],
        );
        Ok(())
    }

    pub fn prioritize_dynamic_themes(&self) -> Result<()> {
        let installed: bool = self.conn.lock().unwrap().query_row(
            "SELECT EXISTS(SELECT 1 FROM app_migrations WHERE name = 'dynamic_themes_first_v1')",
            [],
            |row| row.get(0),
        )?;
        if installed {
            return Ok(());
        }

        for theme in crate::services::shaders::default_themes() {
            let _ = self.conn.lock().unwrap().execute(
                "UPDATE themes SET sort_order = ?1 WHERE id = ?2;",
                rusqlite::params![theme.sort_order, theme.id],
            );
        }
        for theme in crate::core::models::PresentationTheme::default_themes() {
            let _ = self.conn.lock().unwrap().execute(
                "UPDATE themes SET sort_order = ?1, is_favorite = 0 WHERE id = ?2;",
                rusqlite::params![theme.sort_order, theme.id],
            );
        }

        self.conn.lock().unwrap().execute(
            "INSERT OR IGNORE INTO app_migrations (name) VALUES ('dynamic_themes_first_v1')",
            [],
        )?;
        Ok(())
    }
}
