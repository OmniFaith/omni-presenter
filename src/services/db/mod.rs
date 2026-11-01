use rusqlite::{Connection, Result};
use std::path::Path;
use std::sync::{Arc, Mutex};

mod chat;
mod design;
mod media;
mod schema;
mod seed;
mod settings;
mod slides;
mod songs;
mod themes;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct AppDb {
    conn: Arc<Mutex<Connection>>,
}

impl AppDb {
    /// Opens or creates the SQLite database at the given path
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        db.seed_defaults_if_empty()?;
        Ok(db)
    }

    /// Opens an in-memory database (useful for tests)
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        db.seed_defaults_if_empty()?;
        Ok(db)
    }
}
