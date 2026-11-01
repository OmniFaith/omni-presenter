use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: Option<i64>,
    pub title: String,
    pub file_path: String,
    pub file_type: String, // "image", "video", "audio", "pdf", "other"
    pub folder: String,
}

impl MediaItem {
    pub fn new(
        title: impl Into<String>,
        file_path: impl Into<String>,
        folder: impl Into<String>,
    ) -> Self {
        let file_path = file_path.into();
        let file_type = Self::detect_type(&file_path).to_string();
        Self {
            id: None,
            title: title.into(),
            file_path,
            file_type,
            folder: folder.into(),
        }
    }

    pub fn detect_type(path: &str) -> &'static str {
        if path.starts_with("http://") || path.starts_with("https://") {
            return "video";
        }
        let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "mp4" | "mov" | "webm" | "mkv" | "avi" | "m4v" => "video",
            "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "svg" => "image",
            "pdf" => "pdf",
            "mp3" | "wav" | "ogg" | "m4a" | "flac" | "aac" | "wma" => "audio",
            _ => "other",
        }
    }

    pub fn type_display_name(&self, language: crate::core::i18n::Language) -> &str {
        language.text(match self.file_type.as_str() {
            "video" => "chat-video",
            "image" => "common-image",
            "pdf" => "services-pdf-document",
            "audio" => "chat-audio",
            _ => "workspace-media",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CustomTextItem {
    pub id: Option<i64>,
    pub title: String,
    pub content: String,
    pub folder: String,
}

impl CustomTextItem {
    pub fn new(
        title: impl Into<String>,
        content: impl Into<String>,
        folder: impl Into<String>,
    ) -> Self {
        Self {
            id: None,
            title: title.into(),
            content: content.into(),
            folder: folder.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedVerseItem {
    pub id: Option<i64>,
    pub reference: String,
    pub version: String,
    pub text: String,
    pub folder: String,
}

impl SavedVerseItem {
    pub fn new(
        reference: impl Into<String>,
        version: impl Into<String>,
        text: impl Into<String>,
        folder: impl Into<String>,
    ) -> Self {
        Self {
            id: None,
            reference: reference.into(),
            version: version.into(),
            text: text.into(),
            folder: folder.into(),
        }
    }
}
