use rusqlite::{Result, params};

use crate::core::models::{CustomTextItem, MediaItem, SavedVerseItem};

use super::AppDb;

impl AppDb {
    // ---------------------------------------------------------
    // Category Folders (song, media, text, bible)
    // ---------------------------------------------------------

    pub fn get_folders_by_category(&self, category: &str) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT name FROM category_folders WHERE category = ?1 OR (category = 'bible' AND ?1 = 'verse') OR (category = 'verse' AND ?1 = 'bible') ORDER BY name ASC;",
        )?;
        let rows = stmt.query_map(params![category], |row| row.get::<_, String>(0))?;
        let mut folders = Vec::new();
        for f in rows {
            folders.push(f?);
        }
        Ok(folders)
    }

    pub fn add_folder_to_category(&self, category: &str, name: &str) -> Result<()> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Ok(());
        }
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO category_folders (category, name) VALUES (?1, ?2);",
            params![category, trimmed],
        )?;
        Ok(())
    }

    pub fn delete_folder_from_category(&self, category: &str, name: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM category_folders WHERE (category = ?1 OR (category = 'bible' AND ?1 = 'verse') OR (category = 'verse' AND ?1 = 'bible')) AND name = ?2;",
            params![category, name],
        )?;

        match category {
            "media" => {
                conn.execute(
                    "UPDATE media_items SET folder = '' WHERE folder = ?1;",
                    params![name],
                )?;
            }
            "text" => {
                conn.execute(
                    "UPDATE custom_texts SET folder = '' WHERE folder = ?1;",
                    params![name],
                )?;
            }
            "bible" | "verse" => {
                conn.execute(
                    "UPDATE saved_verses SET folder = '' WHERE folder = ?1;",
                    params![name],
                )?;
            }
            _ => {}
        }
        Ok(())
    }

    // ---------------------------------------------------------
    // Media Items
    // ---------------------------------------------------------

    pub fn get_all_media_items(&self) -> Result<Vec<MediaItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, file_path, file_type, folder FROM media_items ORDER BY title ASC;",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(MediaItem {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                file_path: row.get(2)?,
                file_type: row.get(3)?,
                folder: row.get(4)?,
            })
        })?;

        let mut items = Vec::new();
        for item in rows {
            items.push(item?);
        }
        Ok(items)
    }

    pub fn add_media_item(
        &self,
        title: &str,
        file_path: &str,
        file_type: &str,
        folder: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let folder = folder.trim();
        if !folder.is_empty() {
            let _ = conn.execute(
                "INSERT OR IGNORE INTO category_folders (category, name) VALUES ('media', ?1);",
                params![folder],
            );
        }
        conn.execute(
            "INSERT INTO media_items (title, file_path, file_type, folder) VALUES (?1, ?2, ?3, ?4);",
            params![title.trim(), file_path.trim(), file_type.trim(), folder],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn move_media_to_folder(&self, id: i64, folder: &str) -> Result<()> {
        let folder = folder.trim();
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "INSERT OR IGNORE INTO category_folders (category, name) VALUES ('media', ?1);",
            params![folder],
        );
        conn.execute(
            "UPDATE media_items SET folder = ?1 WHERE id = ?2;",
            params![folder, id],
        )?;
        Ok(())
    }

    pub fn delete_media_item(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM media_items WHERE id = ?1;", params![id])?;
        Ok(())
    }

    // ---------------------------------------------------------
    // Custom Texts
    // ---------------------------------------------------------

    pub fn get_all_custom_texts(&self) -> Result<Vec<CustomTextItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, title, content, folder FROM custom_texts ORDER BY title ASC;")?;
        let rows = stmt.query_map([], |row| {
            Ok(CustomTextItem {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                content: row.get(2)?,
                folder: row.get(3)?,
            })
        })?;

        let mut items = Vec::new();
        for item in rows {
            items.push(item?);
        }
        Ok(items)
    }

    pub fn add_custom_text(&self, title: &str, content: &str, folder: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let folder = folder.trim();
        if !folder.is_empty() {
            let _ = conn.execute(
                "INSERT OR IGNORE INTO category_folders (category, name) VALUES ('text', ?1);",
                params![folder],
            );
        }
        conn.execute(
            "INSERT INTO custom_texts (title, content, folder) VALUES (?1, ?2, ?3);",
            params![title.trim(), content.trim(), folder],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_custom_text(
        &self,
        id: i64,
        title: &str,
        content: &str,
        folder: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let folder = folder.trim();
        if !folder.is_empty() {
            let _ = conn.execute(
                "INSERT OR IGNORE INTO category_folders (category, name) VALUES ('text', ?1);",
                params![folder],
            );
        }
        conn.execute(
            "UPDATE custom_texts SET title = ?1, content = ?2, folder = ?3, updated_at = CURRENT_TIMESTAMP WHERE id = ?4;",
            params![title.trim(), content.trim(), folder, id],
        )?;
        Ok(())
    }

    pub fn move_text_to_folder(&self, id: i64, folder: &str) -> Result<()> {
        let folder = folder.trim();
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "INSERT OR IGNORE INTO category_folders (category, name) VALUES ('text', ?1);",
            params![folder],
        );
        conn.execute(
            "UPDATE custom_texts SET folder = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2;",
            params![folder, id],
        )?;
        Ok(())
    }

    pub fn delete_custom_text(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM custom_texts WHERE id = ?1;", params![id])?;
        Ok(())
    }

    // ---------------------------------------------------------
    // Saved Bible Verses
    // ---------------------------------------------------------

    pub fn get_all_saved_verses(&self) -> Result<Vec<SavedVerseItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, reference, version, text, folder FROM saved_verses ORDER BY id DESC;",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SavedVerseItem {
                id: Some(row.get(0)?),
                reference: row.get(1)?,
                version: row.get(2)?,
                text: row.get(3)?,
                folder: row.get(4)?,
            })
        })?;

        let mut items = Vec::new();
        for item in rows {
            items.push(item?);
        }
        Ok(items)
    }

    pub fn add_saved_verse(
        &self,
        reference: &str,
        version: &str,
        text: &str,
        folder: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let folder = folder.trim();
        if !folder.is_empty() {
            let _ = conn.execute(
                "INSERT OR IGNORE INTO category_folders (category, name) VALUES ('bible', ?1);",
                params![folder],
            );
        }
        conn.execute(
            "INSERT INTO saved_verses (reference, version, text, folder) VALUES (?1, ?2, ?3, ?4);",
            params![reference.trim(), version.trim(), text.trim(), folder],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn move_verse_to_folder(&self, id: i64, folder: &str) -> Result<()> {
        let folder = folder.trim();
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "INSERT OR IGNORE INTO category_folders (category, name) VALUES ('bible', ?1);",
            params![folder],
        );
        conn.execute(
            "UPDATE saved_verses SET folder = ?1 WHERE id = ?2;",
            params![folder, id],
        )?;
        Ok(())
    }

    pub fn delete_saved_verse(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM saved_verses WHERE id = ?1;", params![id])?;
        Ok(())
    }
}
