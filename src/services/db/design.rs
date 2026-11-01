use super::AppDb;
use crate::core::design::DesignDocument;
use rusqlite::{Result, params};

impl AppDb {
    pub fn load_designs(&self) -> Result<Vec<DesignDocument>> {
        let conn = self.conn.lock().unwrap();
        let mut query = conn.prepare("SELECT value FROM design_documents ORDER BY rowid")?;
        query
            .query_map([], |row| {
                let value: String = row.get(0)?;
                serde_json::from_str(&value).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })
            })?
            .collect()
    }
    pub fn save_design(&self, scene: &DesignDocument) -> Result<()> {
        let value = serde_json::to_string(scene)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
        self.conn.lock().unwrap().execute(
            "INSERT INTO design_documents (id, value) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET value=excluded.value",
            params![scene.id, value])?;
        Ok(())
    }
    pub fn delete_design(&self, id: &str) -> Result<()> {
        self.conn
            .lock()
            .unwrap()
            .execute("DELETE FROM design_documents WHERE id=?1", [id])?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scene_roundtrip_update_delete() {
        let db = AppDb::open_in_memory().unwrap();
        let mut scene = DesignDocument::template(2);
        db.save_design(&scene).unwrap();
        scene.name = "Culto da noite".into();
        db.save_design(&scene).unwrap();
        assert_eq!(db.load_designs().unwrap(), vec![scene.clone()]);
        db.delete_design(&scene.id).unwrap();
        assert!(db.load_designs().unwrap().is_empty());
    }
}
