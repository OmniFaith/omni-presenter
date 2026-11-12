use super::PresentationApp;
use crate::services::db::AppDb;
use std::path::Path;

type StartupError = Box<dyn std::error::Error + Send + Sync>;

fn open_workspace_database(path: &Path) -> Result<AppDb, StartupError> {
    let path = std::path::absolute(path)?;
    let open = || -> rusqlite::Result<AppDb> {
        let db = AppDb::open(&path)?;
        // Invalid saved preferences must not be replaced by default settings
        // through autosave or on_exit after an apparently successful startup.
        db.load_settings()?;
        Ok(db)
    };
    open().map_err(|error| {
        std::io::Error::other(format!("{}: {error}", path.display())).into()
    })
}

impl PresentationApp {
    /// Opens the persistent workspace or returns the original startup failure.
    /// In-memory databases are reserved for explicit callers such as tests.
    pub fn open_workspace(path: impl AsRef<Path>) -> Result<Self, StartupError> {
        let db = open_workspace_database(path.as_ref())?;
        std::thread::spawn(|| {
            let _ = crate::core::models::get_system_fonts();
        });
        Ok(Self::with_db(db))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_path() -> std::path::PathBuf {
        std::env::temp_dir().join(crate::core::preferences::new_id("omni_startup"))
    }

    #[test]
    fn inaccessible_workspace_is_an_error_not_a_temporary_database() {
        let directory = test_path();
        let path = directory.join("workspace.db");
        let error = open_workspace_database(&path).err().expect("Missing parent must fail");
        assert!(error.to_string().contains("workspace.db"));
        assert!(!directory.exists());
    }

    #[test]
    fn invalid_preferences_are_preserved_and_prevent_startup() {
        let path = test_path();
        let db = AppDb::open(&path).unwrap();
        drop(db);
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection.execute("INSERT INTO app_settings (id, value) VALUES (1, ?1)", ["invalid JSON"]).unwrap();
        assert!(open_workspace_database(&path).is_err());
        let original: String = connection.query_row("SELECT value FROM app_settings WHERE id = 1", [], |row| row.get(0)).unwrap();
        assert_eq!(original, "invalid JSON");
        drop(connection);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn valid_workspace_reopens_its_saved_preferences() {
        let path = test_path();
        let settings = crate::core::preferences::AppSettings::default();
        let db = open_workspace_database(&path).unwrap();
        db.save_settings(&settings).unwrap();
        drop(db);
        let reopened = open_workspace_database(&path).unwrap();
        assert_eq!(reopened.load_settings().unwrap(), Some(settings));
        drop(reopened);
        std::fs::remove_file(path).unwrap();
    }
}
