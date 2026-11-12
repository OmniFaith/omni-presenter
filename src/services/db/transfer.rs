//! Typed snapshots of known workspace tables. No SQL from a package is executed.
use super::AppDb;
use rusqlite::{Connection, params_from_iter, types::Value};
use serde::{Deserialize, Serialize};

pub const TABLES: &[&str] = &[
    "app_settings",
    "songs",
    "song_versions",
    "folders",
    "themes",
    "media_items",
    "custom_texts",
    "saved_verses",
    "category_folders",
    "design_documents",
    "bible_translations",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Cell {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Cell>>,
}

fn columns(conn: &Connection, name: &str) -> Result<Vec<String>, String> {
    conn.prepare(&format!("PRAGMA table_info(\"{name}\")"))
        .and_then(|mut s| s.query_map([], |r| r.get(1))?.collect())
        .map_err(|e| e.to_string())
}

impl AppDb {
    pub fn transfer_directory(&self) -> Result<std::path::PathBuf, String> {
        let path: String = self
            .conn
            .lock()
            .unwrap()
            .query_row(
                "SELECT file FROM pragma_database_list WHERE name='main'",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let parent = std::path::Path::new(&path)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        std::path::absolute(parent).map_err(|e| e.to_string())
    }

    pub fn transfer_snapshot(&self) -> Result<Vec<Table>, String> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let mut tables = Vec::new();
        for &name in TABLES {
            let columns = columns(&tx, name)?;
            let mut stmt = tx
                .prepare(&format!("SELECT * FROM \"{name}\" ORDER BY rowid"))
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |r| {
                    (0..columns.len())
                        .map(|i| {
                            Ok(match r.get::<_, Value>(i)? {
                                Value::Null => Cell::Null,
                                Value::Integer(v) => Cell::Integer(v),
                                Value::Real(v) => Cell::Real(v),
                                Value::Text(v) => Cell::Text(v),
                                Value::Blob(_) => return Err(rusqlite::Error::InvalidQuery),
                            })
                        })
                        .collect::<rusqlite::Result<Vec<_>>>()
                })
                .map_err(|e| e.to_string())?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|e| e.to_string())?;
            tables.push(Table {
                name: name.into(),
                columns,
                rows,
            });
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(tables)
    }

    pub fn validate_transfer(&self, tables: &[Table]) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        if tables.len() != TABLES.len() {
            return Err("transfer-invalid".into());
        }
        for &name in TABLES {
            let matches: Vec<_> = tables.iter().filter(|t| t.name == name).collect();
            if matches.len() != 1 {
                return Err("transfer-invalid".into());
            }
            let table = matches[0];
            if table.columns != columns(&conn, name)?
                || table.rows.iter().any(|r| r.len() != table.columns.len())
            {
                return Err("transfer-invalid".into());
            }
        }
        // Deserialize domain JSON before touching the destination database.
        for table in tables {
            if let Some(i) = table.columns.iter().position(|c| c == "value") {
                for row in &table.rows {
                    let Cell::Text(value) = &row[i] else {
                        return Err("transfer-invalid".into());
                    };
                    match table.name.as_str() {
                        "app_settings" => {
                            let settings: crate::core::preferences::AppSettings =
                                serde_json::from_str(value).map_err(|e| e.to_string())?;
                            for theme in &settings.interface_themes {
                                theme.validate()?;
                            }
                            for plugin in &settings.plugins {
                                plugin.package.validate()?;
                            }
                        }
                        "design_documents" => {
                            let _: crate::core::design::DesignDocument =
                                serde_json::from_str(value).map_err(|e| e.to_string())?;
                        }
                        "bible_translations" => {
                            let _: crate::core::bible::BibleTranslation =
                                serde_json::from_str(value).map_err(|e| e.to_string())?;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    pub fn restore_transfer(&self, tables: &[Table]) -> Result<(), String> {
        self.validate_transfer(tables)?;
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for table in tables {
            // Names and column order have been checked against the local schema.
            tx.execute(&format!("DELETE FROM \"{}\"", table.name), [])
                .map_err(|e| e.to_string())?;
            let placeholders = vec!["?"; table.columns.len()].join(",");
            let mut stmt = tx
                .prepare(&format!(
                    "INSERT INTO \"{}\" VALUES ({placeholders})",
                    table.name
                ))
                .map_err(|e| e.to_string())?;
            for row in &table.rows {
                let values = row.iter().map(|c| match c {
                    Cell::Null => Value::Null,
                    Cell::Integer(v) => Value::Integer(*v),
                    Cell::Real(v) => Value::Real(*v),
                    Cell::Text(v) => Value::Text(v.clone()),
                });
                stmt.execute(params_from_iter(values))
                    .map_err(|e| e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())
    }

    pub fn import_lyrics(&self, songs: &[crate::core::models::Song]) -> Result<(), String> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for song in songs {
            tx.execute(
                "INSERT INTO songs(title,artist,text,tags,folder) VALUES (?1,?2,?3,?4,?5)",
                rusqlite::params![song.title, song.artist, song.text, song.tags, song.folder],
            )
            .map_err(|e| e.to_string())?;
            if !song.folder.is_empty() {
                tx.execute(
                    "INSERT OR IGNORE INTO folders(name) VALUES (?1)",
                    [&song.folder],
                )
                .map_err(|e| e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())
    }
}
