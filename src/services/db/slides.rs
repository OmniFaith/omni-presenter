//! Persistence adapter for the slide catalogue. The deck itself only accepts domain data.
use super::AppDb;
use crate::core::slides::SlideDeck;

impl SlideDeck {
    pub fn save_current_song_text(&self, db: Option<&AppDb>) -> rusqlite::Result<()> {
        if self.is_song()
            && let Some(db) = db
            && let Some(song) = self.current_song()
            && let Some(id) = song.id
        {
            db.update_song(id, &song.title, &song.artist, &song.text)?;
        }
        Ok(())
    }

    pub fn from_db(db: &AppDb) -> Self {
        Self::from_catalog(
            db.get_all_songs().unwrap_or_default(),
            db.load_designs().unwrap_or_default(),
        )
    }

    pub fn reload_from_db(&mut self, db: &AppDb) {
        if let Ok(songs) = db.get_all_songs() {
            self.replace_library(songs);
        }
    }
}
