use super::*;

impl ChatService {
    pub fn upload(
        &self,
        token: &str,
        name: &str,
        mime: &str,
        bytes: &[u8],
    ) -> Result<Attachment, String> {
        let profile = Self::authenticate(&mut self.runtime.lock().unwrap(), token)?;
        if bytes.is_empty() || bytes.len() > MAX_UPLOAD {
            return Err("O arquivo precisa ter entre 1 byte e 256 MiB".into());
        }
        let name: String = name
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or("arquivo")
            .chars()
            .filter(|c| !c.is_control())
            .take(180)
            .collect();
        let mime = if mime.contains(['\r', '\n']) || mime.len() > 120 {
            "application/octet-stream"
        } else {
            mime
        };
        let id = secret()?;
        self.db.chat_upload(&id, &profile, &name, mime, bytes)?;
        Ok(Attachment {
            id,
            name,
            mime: mime.into(),
            size: bytes.len(),
        })
    }

    pub fn download(&self, token: &str, id: &str) -> Result<(String, String, Vec<u8>), String> {
        let mut rt = self.runtime.lock().unwrap();
        let profile = Self::authenticate(&mut rt, token)?;
        let (owner, _, _, _) = self
            .db
            .chat_file_info(id)?
            .ok_or("services-file-not-found")?;
        if owner != profile
            && !rt
                .store
                .messages
                .iter()
                .any(|m| m.file.as_ref().is_some_and(|f| f.id == id) && Self::visible(m, &profile))
        {
            return Err("services-no-access-to-this-file".into());
        }
        let (_, name, mime, data) = self.db.chat_file(id)?.ok_or("services-file-not-found")?;
        Ok((name, mime, data))
    }

    pub(crate) fn materialize(&self, file: &Attachment) -> Result<String, String> {
        let (_, name, mime, data) = self
            .db
            .chat_file(&file.id)?
            .ok_or("services-file-not-found")?;
        let ext = match mime.as_str() {
            "audio/webm" => "webm",
            "audio/wav" => "wav",
            "audio/ogg" => "ogg",
            "audio/mp4" => "m4a",
            "audio/mpeg" => "mp3",
            "image/png" => "png",
            "image/jpeg" => "jpg",
            "video/mp4" => "mp4",
            "video/webm" => "webm",
            _ => name.rsplit('.').next().unwrap_or("bin"),
        };
        let ext: String = ext
            .chars()
            .filter(char::is_ascii_alphanumeric)
            .take(10)
            .collect();
        let directory = self.db.chat_media_directory()?;
        std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
        let path = directory.join(format!("{}.{}", file.id, ext));
        if !path.exists() {
            std::fs::write(&path, data).map_err(|e| e.to_string())?;
        }
        Ok(path.to_string_lossy().into_owned())
    }
}
