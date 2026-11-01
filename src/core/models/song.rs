#[derive(Clone, Debug, PartialEq)]
pub struct Song {
    pub id: Option<i64>,
    pub title: String,
    pub artist: String,
    pub text: String,
    pub tags: String,
    pub folder: String,
}

impl Song {
    pub fn new(
        title: impl Into<String>,
        artist: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            id: None,
            title: title.into(),
            artist: artist.into(),
            text: text.into(),
            tags: String::new(),
            folder: String::new(),
        }
    }

    pub fn with_folder(mut self, folder: impl Into<String>) -> Self {
        self.folder = folder.into();
        self
    }
}
