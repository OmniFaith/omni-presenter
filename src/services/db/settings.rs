use rusqlite::{Result, params};

use super::AppDb;

impl AppDb {
    pub fn load_settings(&self) -> Result<Option<crate::core::preferences::AppSettings>> {
        use rusqlite::OptionalExtension;
        let conn = self.conn.lock().unwrap();
        let value: Option<String> = conn
            .query_row("SELECT value FROM app_settings WHERE id = 1", [], |row| {
                row.get(0)
            })
            .optional()?;
        value
            .map(|value| {
                serde_json::from_str::<crate::core::preferences::AppSettings>(&value)
                    .map(|mut s| {
                        s.normalize();
                        s
                    })
                    .map_err(|err| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(err),
                        )
                    })
            })
            .transpose()
    }

    pub fn save_settings(&self, settings: &crate::core::preferences::AppSettings) -> Result<()> {
        let value = serde_json::to_string(settings)
            .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;
        self.conn.lock().unwrap().execute("INSERT INTO app_settings (id, value) VALUES (1, ?1) ON CONFLICT(id) DO UPDATE SET value = excluded.value", params![value])?;
        Ok(())
    }

    pub fn get_bible_translations(&self) -> Result<Vec<crate::core::bible::BibleTranslation>> {
        let conn = self.conn.lock().unwrap();
        let mut statement =
            conn.prepare("SELECT id, value FROM bible_translations ORDER BY rowid")?;
        statement
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let value: String = row.get(1)?;
                crate::core::bible::BibleTranslation::from_json(value.as_bytes(), &id, &id).map_err(
                    |error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            1,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::other(error)),
                        )
                    },
                )
            })?
            .collect()
    }

    pub fn save_bible_translation(
        &self,
        translation: &crate::core::bible::BibleTranslation,
    ) -> Result<()> {
        let value = serde_json::to_string(translation)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        self.conn.lock().unwrap().execute("INSERT INTO bible_translations (id, value) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET value = excluded.value", params![translation.id, value])?;
        Ok(())
    }
}
