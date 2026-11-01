use rusqlite::{Result, params};

use crate::core::models::Song;

use super::AppDb;

impl AppDb {
    /// Returns list of all folder names
    pub fn get_folders(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT name FROM folders ORDER BY name ASC;")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut folders = Vec::new();
        for f in rows {
            folders.push(f?);
        }
        Ok(folders)
    }

    /// Adds a new folder to SQLite
    pub fn add_folder(&self, name: &str) -> Result<()> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Ok(());
        }
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO folders (name) VALUES (?1);",
            params![trimmed],
        )?;
        Ok(())
    }

    /// Deletes a folder and moves its songs to root (no folder)
    pub fn delete_folder(&self, name: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM folders WHERE name = ?1;", params![name])?;
        conn.execute(
            "UPDATE songs SET folder = '' WHERE folder = ?1;",
            params![name],
        )?;
        Ok(())
    }

    /// Moves a song to a given folder
    pub fn move_song_to_folder(&self, song_id: i64, folder: &str) -> Result<()> {
        let trimmed = folder.trim();
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "INSERT OR IGNORE INTO folders (name) VALUES (?1);",
            params![trimmed],
        );
        conn.execute(
            "UPDATE songs SET folder = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2;",
            params![trimmed, song_id],
        )?;
        Ok(())
    }

    /// Retrieves all songs ordered by title
    pub fn get_all_songs(&self) -> Result<Vec<Song>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, artist, text, tags, folder FROM songs ORDER BY title ASC;",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Song {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                artist: row.get(2)?,
                text: row.get(3)?,
                tags: row.get(4)?,
                folder: row.get(5)?,
            })
        })?;

        let mut songs = Vec::new();
        for s in rows {
            songs.push(s?);
        }
        Ok(songs)
    }

    /// Search songs by title, artist, or lyric content
    pub fn search_songs(&self, query: &str) -> Result<Vec<Song>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT id, title, artist, text, tags, folder FROM songs
             WHERE title LIKE ?1 OR artist LIKE ?1 OR text LIKE ?1
             ORDER BY title ASC;",
        )?;
        let rows = stmt.query_map(params![pattern], |row| {
            Ok(Song {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                artist: row.get(2)?,
                text: row.get(3)?,
                tags: row.get(4)?,
                folder: row.get(5)?,
            })
        })?;

        let mut songs = Vec::new();
        for s in rows {
            songs.push(s?);
        }
        Ok(songs)
    }

    /// Inserts a new song into SQLite without folder
    pub fn add_song(&self, title: &str, artist: &str, text: &str) -> Result<i64> {
        self.add_song_with_folder(title, artist, text, "")
    }

    /// Inserts a new song into SQLite specifying folder
    pub fn add_song_with_folder(
        &self,
        title: &str,
        artist: &str,
        text: &str,
        folder: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let folder = folder.trim();
        if !folder.is_empty() {
            let _ = conn.execute(
                "INSERT OR IGNORE INTO folders (name) VALUES (?1);",
                params![folder],
            );
        }
        conn.execute(
            "INSERT INTO songs (title, artist, text, folder) VALUES (?1, ?2, ?3, ?4);",
            params![title, artist, text, folder],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Inserts a new song including folder and tags
    pub fn add_song_full(
        &self,
        title: &str,
        artist: &str,
        text: &str,
        folder: &str,
        tags: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let folder = folder.trim();
        if !folder.is_empty() {
            let _ = conn.execute(
                "INSERT OR IGNORE INTO folders (name) VALUES (?1);",
                params![folder],
            );
        }
        conn.execute(
            "INSERT INTO songs (title, artist, text, folder, tags) VALUES (?1, ?2, ?3, ?4, ?5);",
            params![title, artist, text, folder, tags],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Updates an existing song in SQLite
    pub fn update_song(&self, id: i64, title: &str, artist: &str, text: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE songs SET title = ?1, artist = ?2, text = ?3, updated_at = CURRENT_TIMESTAMP WHERE id = ?4;",
            params![title, artist, text, id],
        )?;
        Ok(())
    }

    /// Updates an existing song including folder and tags
    pub fn update_song_full(
        &self,
        id: i64,
        title: &str,
        artist: &str,
        text: &str,
        folder: &str,
        tags: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let folder = folder.trim();
        if !folder.is_empty() {
            let _ = conn.execute(
                "INSERT OR IGNORE INTO folders (name) VALUES (?1);",
                params![folder],
            );
        }
        conn.execute(
            "UPDATE songs SET title = ?1, artist = ?2, text = ?3, folder = ?4, tags = ?5, updated_at = CURRENT_TIMESTAMP WHERE id = ?6;",
            params![title, artist, text, folder, tags, id],
        )?;
        Ok(())
    }

    /// Deletes a song from SQLite by ID
    pub fn delete_song(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM song_versions WHERE song_id = ?1;", params![id])?;
        conn.execute("DELETE FROM songs WHERE id = ?1;", params![id])?;
        Ok(())
    }
}

impl AppDb {
    /// Returns the version group id shared by linked song versions.
    pub fn song_version_group(&self, song_id: i64) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT group_id FROM song_versions WHERE song_id = ?1;")?;
        let mut rows = stmt.query(params![song_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    /// Returns all songs linked as versions of the given song (including it).
    pub fn song_versions(&self, song_id: i64) -> Result<Vec<Song>> {
        let conn = self.conn.lock().unwrap();
        let Some(group): Option<String> = conn
            .query_row(
                "SELECT group_id FROM song_versions WHERE song_id = ?1;",
                params![song_id],
                |row| row.get(0),
            )
            .ok()
        else {
            return Ok(Vec::new());
        };
        let mut stmt = conn.prepare(
            "SELECT s.id, s.title, s.artist, s.text, s.tags, s.folder
             FROM songs s
             JOIN song_versions v ON v.song_id = s.id
             WHERE v.group_id = ?1
             ORDER BY s.title ASC;",
        )?;
        let rows = stmt.query_map(params![group], |row| {
            Ok(Song {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                artist: row.get(2)?,
                text: row.get(3)?,
                tags: row.get(4)?,
                folder: row.get(5)?,
            })
        })?;
        let mut songs = Vec::new();
        for song in rows {
            songs.push(song?);
        }
        Ok(songs)
    }

    /// Returns just the ids linked as versions of the given song.
    pub fn song_version_ids(&self, song_id: i64) -> Result<Vec<i64>> {
        Ok(self
            .song_versions(song_id)?
            .into_iter()
            .filter_map(|song| song.id)
            .filter(|id| *id != song_id)
            .collect())
    }

    /// Convenience for the editor, which identifies songs by title.
    pub fn song_version_ids_of(&self, title: &str) -> Result<Vec<i64>> {
        let conn = self.conn.lock().unwrap();
        let id: Option<i64> = conn
            .query_row(
                "SELECT id FROM songs WHERE title = ?1 ORDER BY id ASC LIMIT 1;",
                params![title],
                |row| row.get(0),
            )
            .ok();
        drop(conn);
        if let Some(id) = id {
            self.song_version_ids(id)
        } else {
            Ok(Vec::new())
        }
    }

    /// Replaces the version links for a song. Passing only the song itself
    /// removes it from any group.
    pub fn set_song_versions(&self, song_id: i64, ids: &[i64]) -> Result<()> {
        let mut members: Vec<i64> = ids.iter().copied().filter(|id| *id != song_id).collect();
        members.sort_unstable();
        members.dedup();

        let conn = self.conn.lock().unwrap();
        if members.is_empty() {
            conn.execute(
                "DELETE FROM song_versions WHERE song_id = ?1;",
                params![song_id],
            )?;
            return Ok(());
        }

        members.insert(0, song_id);
        let existing: Option<String> = members.iter().find_map(|id| {
            conn.query_row(
                "SELECT group_id FROM song_versions WHERE song_id = ?1;",
                params![id],
                |row| row.get(0),
            )
            .ok()
        });
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or(0);
        let group = existing.unwrap_or_else(|| format!("v{}_{}", members[0], stamp));

        for id in &members {
            conn.execute("DELETE FROM song_versions WHERE song_id = ?1;", params![id])?;
        }
        for id in &members {
            conn.execute(
                "INSERT INTO song_versions (song_id, group_id) VALUES (?1, ?2);",
                params![id, group],
            )?;
        }
        Ok(())
    }
}
