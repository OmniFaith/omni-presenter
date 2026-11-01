use super::AppDb;
use rusqlite::{OptionalExtension, params};
type StoredChatFile = (String, String, String, Vec<u8>);

impl AppDb {
    pub(crate) fn chat_media_directory(&self) -> Result<std::path::PathBuf, String> {
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
        let parent = if path.is_empty() {
            std::env::temp_dir().join("omni-presenter-chat")
        } else {
            std::path::Path::new(&path)
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf()
        };
        Ok(parent.join("chat-media"))
    }
    pub(crate) fn chat_setup(&self) -> Result<(), String> {
        self.conn.lock().unwrap().execute_batch(
            "CREATE TABLE IF NOT EXISTS chat_store (id INTEGER PRIMARY KEY CHECK(id=1), value TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS chat_uploads (id TEXT PRIMARY KEY, owner TEXT NOT NULL, name TEXT NOT NULL, mime TEXT NOT NULL, data BLOB NOT NULL);
             CREATE TABLE IF NOT EXISTS chat_tls (id INTEGER PRIMARY KEY CHECK(id=1), certificate TEXT NOT NULL, private_key TEXT NOT NULL, host TEXT NOT NULL);"
        ).map_err(|e| e.to_string())
    }

    pub(crate) fn chat_load(&self) -> Result<Option<String>, String> {
        self.conn
            .lock()
            .unwrap()
            .query_row("SELECT value FROM chat_store WHERE id=1", [], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())
    }

    pub(crate) fn chat_save(&self, value: &str) -> Result<(), String> {
        self.conn.lock().unwrap().execute("INSERT INTO chat_store(id,value) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET value=excluded.value", [value]).map(|_| ()).map_err(|e| e.to_string())
    }

    pub(crate) fn chat_upload(
        &self,
        id: &str,
        owner: &str,
        name: &str,
        mime: &str,
        bytes: &[u8],
    ) -> Result<(), String> {
        self.conn
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO chat_uploads(id,owner,name,mime,data) VALUES(?1,?2,?3,?4,?5)",
                params![id, owner, name, mime, bytes],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub(crate) fn chat_file(&self, id: &str) -> Result<Option<StoredChatFile>, String> {
        self.conn
            .lock()
            .unwrap()
            .query_row(
                "SELECT owner,name,mime,data FROM chat_uploads WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    pub(crate) fn chat_file_info(
        &self,
        id: &str,
    ) -> Result<Option<(String, String, String, usize)>, String> {
        self.conn
            .lock()
            .unwrap()
            .query_row(
                "SELECT owner,name,mime,length(data) FROM chat_uploads WHERE id=?1",
                [id],
                |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get::<_, i64>(3)? as usize,
                    ))
                },
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    pub(crate) fn chat_certificate(&self, host: &str) -> Result<(String, String), String> {
        let conn = self.conn.lock().unwrap();
        let saved: Option<(String, String)> = conn
            .query_row(
                "SELECT certificate,private_key FROM chat_tls WHERE id=1 AND host=?1",
                [host],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if let Some(saved) = saved {
            return Ok(saved);
        }
        let generated = rcgen::generate_simple_self_signed(vec![
            "localhost".into(),
            "127.0.0.1".into(),
            host.into(),
        ])
        .map_err(|e| e.to_string())?;
        let certificate = generated.cert.pem();
        let key = generated.signing_key.serialize_pem();
        conn.execute("INSERT INTO chat_tls VALUES(1,?1,?2,?3) ON CONFLICT(id) DO UPDATE SET certificate=excluded.certificate,private_key=excluded.private_key,host=excluded.host", params![certificate,key,host]).map_err(|e| e.to_string())?;
        Ok((certificate, key))
    }
}
