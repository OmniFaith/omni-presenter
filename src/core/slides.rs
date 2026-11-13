use crate::core::models::Song;
pub use crate::core::breaks::resolve_conditional_breaks;

#[derive(Clone, Debug, PartialEq)]
pub struct MediaOutput {
    pub path: String,
    pub kind: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SlideDeck {
    pub designs: Vec<crate::core::design::DesignDocument>,
    pub preview_design: Option<crate::core::design::DesignDocument>,
    pub live_design: Option<crate::core::design::DesignDocument>,
    pub live_overlay: Option<crate::core::design::DesignDocument>,
    pub preview_media: Option<MediaOutput>,
    pub live_media: Option<MediaOutput>,
    pub slide_media: Vec<Option<MediaOutput>>,
    pub live_slide_media: Vec<Option<MediaOutput>>,
    // Preview state (what is shown in lyrics container & preview monitor)
    title: String,
    artist: String,
    raw_text: String,
    slides: Vec<String>,
    preview_slide_index: usize,
    library: Vec<Song>,
    selected_song_index: usize,
    is_song: bool,
    is_bible: bool,

    // Live state (what is projected on projector & shown in Retorno Ao Vivo)
    live_title: String,
    live_artist: String,
    live_slides: Vec<String>,
    live_index: usize,
    is_content_hidden: bool,
    live_is_song: bool,
    live_is_bible: bool,
}

impl Default for SlideDeck {
    fn default() -> Self {
        let songs = crate::core::seed::songs();

        if let Some(first_song) = songs.first().cloned() {
            let slides = build_song_slides(&first_song.title, &first_song.artist, &first_song.text);
            let media_len = slides.len();
            Self {
                designs: Vec::new(),
                preview_design: None,
                live_design: None,
                live_overlay: None,
                preview_media: None,
                live_media: None,
                slide_media: vec![None; media_len],
                live_slide_media: vec![None; media_len],
                title: first_song.title.clone(),
                artist: first_song.artist.clone(),
                raw_text: first_song.text,
                slides: slides.clone(),
                preview_slide_index: 0,
                library: songs,
                selected_song_index: 0,
                is_song: true,
                is_bible: false,
                live_title: first_song.title,
                live_artist: first_song.artist,
                live_slides: slides,
                live_index: 0,
                is_content_hidden: false,
                live_is_song: true,
                live_is_bible: false,
            }
        } else {
            Self {
                designs: Vec::new(),
                preview_design: None,
                live_design: None,
                live_overlay: None,
                preview_media: None,
                live_media: None,
                slide_media: Vec::new(),
                live_slide_media: Vec::new(),
                title: String::new(),
                artist: String::new(),
                raw_text: String::new(),
                slides: Vec::new(),
                preview_slide_index: 0,
                library: Vec::new(),
                selected_song_index: 0,
                is_song: false,
                is_bible: false,
                live_title: String::new(),
                live_artist: String::new(),
                live_slides: Vec::new(),
                live_index: 0,
                is_content_hidden: false,
                live_is_song: false,
                live_is_bible: false,
            }
        }
    }
}

impl SlideDeck {
    pub fn new(raw_text: &str) -> Self {
        let slides = parse_slides_from_text(raw_text);
        let media_len = slides.len();
        Self {
            designs: Vec::new(),
            preview_design: None,
            live_design: None,
            live_overlay: None,
            preview_media: None,
            live_media: None,
            slide_media: vec![None; media_len],
            live_slide_media: vec![None; media_len],
            title: crate::core::i18n::Language::system().text("services-presentation").to_string(),
            artist: String::new(),
            raw_text: raw_text.to_owned(),
            slides: slides.clone(),
            preview_slide_index: 0,
            library: Vec::new(),
            selected_song_index: 0,
            is_song: false,
            is_bible: false,

            live_title: crate::core::i18n::Language::system().text("services-presentation").to_string(),
            live_artist: String::new(),
            live_slides: slides,
            live_index: 0,
            is_content_hidden: false,
            live_is_song: false,
            live_is_bible: false,
        }
    }

    pub fn from_catalog(songs: Vec<Song>, designs: Vec<crate::core::design::DesignDocument>) -> Self {
        if songs.is_empty() {
            return Self {
                designs,
                ..Self::default()
            };
        }

        // Try to find "Isaías 53" as default, else index 0
        let default_idx = songs
            .iter()
            .position(|s| s.title.contains("Isaías"))
            .unwrap_or(0);

        let def_title = songs[default_idx].title.clone();
        let def_artist = songs[default_idx].artist.clone();
        let def_text = songs[default_idx].text.clone();
        let slides = build_song_slides(&def_title, &def_artist, &def_text);
        let current_index = 0;
        let media_len = slides.len();

        Self {
            designs,
            preview_design: None,
            live_design: None,
            live_overlay: None,
            preview_media: None,
            live_media: None,
            slide_media: vec![None; media_len],
            live_slide_media: vec![None; media_len],
            title: def_title.clone(),
            artist: def_artist.clone(),
            raw_text: def_text,
            slides: slides.clone(),
            preview_slide_index: current_index,
            library: songs,
            selected_song_index: default_idx,
            is_song: true,
            is_bible: false,

            live_title: def_title,
            live_artist: def_artist,
            live_slides: slides,
            live_index: current_index,
            is_content_hidden: false,
            live_is_song: true,
            live_is_bible: false,
        }
    }

    /// Refreshes the catalogue without replacing the prepared or live content.
    pub fn replace_library(&mut self, songs: Vec<Song>) {
        let selected = self.current_song().cloned();
        self.selected_song_index = selected
            .and_then(|selected| {
                songs.iter().position(|song| match selected.id {
                    Some(id) => song.id == Some(id),
                    None => song.title == selected.title && song.artist == selected.artist,
                })
            })
            .unwrap_or(0);
        self.library = songs;
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn preview_design_by_id(&mut self, id: &str) -> Result<(), String> {
        let scene = self
            .designs
            .iter()
            .find(|scene| scene.id == id)
            .cloned()
            .ok_or_else(|| crate::core::i18n::Language::system().text("services-this-design-is-no-longer-in-the-library").to_string())?;
        self.load_custom_content(scene.name.clone(), crate::core::i18n::Language::system().text("workspace-design").to_string(), String::new());
        self.preview_design = Some(scene);
        Ok(())
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn artist(&self) -> &str {
        &self.artist
    }

    pub fn library(&self) -> &[Song] {
        &self.library
    }

    pub fn library_mut(&mut self) -> &mut Vec<Song> {
        &mut self.library
    }

    pub fn selected_song_index(&self) -> usize {
        self.selected_song_index
    }

    pub fn current_song(&self) -> Option<&Song> {
        self.library.get(self.selected_song_index)
    }

    /// Seleciona uma música apenas para a pré-visualização (não interfere no ao vivo)
    pub fn select_song(&mut self, index: usize) {
        if index < self.library.len() {
            self.preview_design = None;
            self.preview_media = None;
            self.selected_song_index = index;
            self.title = self.library[index].title.clone();
            self.artist = self.library[index].artist.clone();
            self.raw_text = self.library[index].text.clone();
            self.slides = build_song_slides(&self.title, &self.artist, &self.raw_text);
            self.slide_media = vec![None; self.slides.len()];
            self.preview_slide_index = 0;
            self.is_song = true;
            self.is_bible = false;
        }
    }

    pub fn preview_song(&mut self, index: usize) {
        self.select_song(index);
    }

    /// Projeta a música selecionada diretamente no ao vivo
    pub fn project_song(&mut self, index: usize) {
        self.select_song(index);
        self.project_preview();
    }

    pub fn sync_preview_media(&mut self) {
        self.preview_media = self
            .slide_media
            .get(self.preview_slide_index)
            .cloned()
            .flatten();
    }

    pub fn sync_live_media(&mut self) {
        self.live_media = self
            .live_slide_media
            .get(self.live_index)
            .cloned()
            .flatten();
    }

    /// Transfere o conteúdo atual da pré-visualização para o retorno ao vivo / projetor
    pub fn project_preview(&mut self) {
        if let Some(mut scene) = self.preview_design.clone().filter(|scene| scene.overlay) {
            scene.background_mode = "transparent".into();
            scene.start_timers();
            self.live_overlay = Some(scene);
            self.is_content_hidden = false;
            return;
        }
        self.live_design = self.preview_design.clone();
        self.live_slide_media = self.slide_media.clone();
        if let Some(scene) = &mut self.live_design {
            scene.start_timers();
        }
        self.live_title = self.title.clone();
        self.live_artist = self.artist.clone();
        self.live_slides = self.slides.clone();
        self.live_index = self
            .preview_slide_index
            .min(self.live_slides.len().saturating_sub(1));
        self.is_content_hidden = false;
        self.live_is_song = self.is_song;
        self.live_is_bible = self.is_bible;
        self.sync_live_media();
        if self.live_media.is_none() && self.preview_media.is_some() && self.slides.len() <= 1 {
            self.live_media = self.preview_media.clone();
            self.live_slide_media = vec![self.preview_media.clone()];
        }
    }

    /// Projeta uma estrofe específica da pré-visualização no retorno ao vivo / projetor
    pub fn project_preview_slide(&mut self, slide_index: usize) {
        if self.preview_design.is_some() {
            self.project_preview();
            return;
        }
        self.live_design = self.preview_design.clone();
        self.live_slide_media = self.slide_media.clone();
        self.live_title = self.title.clone();
        self.live_artist = self.artist.clone();
        self.live_slides = self.slides.clone();
        self.preview_slide_index = slide_index.min(self.slides.len().saturating_sub(1));
        self.live_index = slide_index.min(self.live_slides.len().saturating_sub(1));
        self.is_content_hidden = false;
        self.live_is_song = self.is_song;
        self.live_is_bible = self.is_bible;
        self.sync_live_media();
        if self.live_media.is_none() && self.preview_media.is_some() && self.slides.len() <= 1 {
            self.live_media = self.preview_media.clone();
            self.live_slide_media = vec![self.preview_media.clone()];
        }
    }

    pub fn clear_live(&mut self) {
        self.live_design = None;
        self.live_overlay = None;
        self.live_media = None;
        self.live_slide_media.clear();
        self.live_title.clear();
        self.live_artist.clear();
        self.live_slides = vec![String::new()];
        self.live_index = 0;
        self.is_content_hidden = false;
        self.live_is_song = false;
        self.live_is_bible = false;
    }

    pub fn live_title(&self) -> &str {
        &self.live_title
    }

    pub fn live_artist(&self) -> &str {
        &self.live_artist
    }

    pub fn live_slides(&self) -> &[String] {
        &self.live_slides
    }

    pub fn total_live_slides(&self) -> usize {
        self.live_slides.len()
    }

    pub fn live_index(&self) -> usize {
        self.live_index
    }

    pub fn select_live_slide(&mut self, index: usize) {
        if index < self.live_slides.len() {
            self.live_index = index;
            self.is_content_hidden = false;
            self.sync_live_media();
            if self.is_previewing_live_song() {
                self.preview_slide_index = index;
                self.sync_preview_media();
            }
        }
    }

    pub fn is_previewing_live_song(&self) -> bool {
        !self.live_slides.is_empty()
            && self.title == self.live_title
            && self.artist == self.live_artist
    }

    pub fn is_live_song(&self) -> bool {
        if self.live_design.is_some() || self.live_media.is_some() || self.live_title.is_empty() {
            return false;
        }
        self.live_is_song || self.library.iter().any(|s| s.title == self.live_title)
    }

    pub fn is_song(&self) -> bool {
        self.is_song
    }

    pub fn set_is_song(&mut self, is_song: bool) {
        self.is_song = is_song;
    }

    pub fn is_bible(&self) -> bool {
        self.is_bible
    }

    pub fn set_is_bible(&mut self, is_bible: bool) {
        self.is_bible = is_bible;
    }

    pub fn is_live_bible(&self) -> bool {
        self.live_is_bible
    }

    pub fn load_custom_content(&mut self, title: String, artist: String, text: String) {
        self.preview_design = None;
        self.preview_media = None;
        self.title = title;
        self.artist = artist;
        self.raw_text = text.clone();
        let parsed = parse_slides_from_text(&text);
        self.slides = if parsed.is_empty() {
            vec![String::new()]
        } else {
            parsed
        };
        self.slide_media = vec![None; self.slides.len()];
        self.preview_slide_index = 0;
        self.is_song = false;
        self.is_bible = text.contains("//ref-pos:") || text.lines().any(|l| l.trim().starts_with("//ref-"));
    }

    pub fn load_custom_slides(&mut self, title: String, artist: String, slides: Vec<String>) {
        self.preview_design = None;
        self.preview_media = None;
        self.title = title;
        self.artist = artist;
        self.raw_text = slides.join("\n\n");
        self.slides = if slides.is_empty() {
            vec![String::new()]
        } else {
            slides
        };
        self.slide_media = vec![None; self.slides.len()];
        self.preview_slide_index = 0;
        self.is_song = false;
        self.is_bible = self.raw_text.contains("//ref-pos:") || self.raw_text.lines().any(|l| l.trim().starts_with("//ref-"));
    }

    pub fn load_custom_slides_with_media(
        &mut self,
        title: String,
        artist: String,
        slides: Vec<String>,
        media: Vec<Option<MediaOutput>>,
    ) {
        self.preview_design = None;
        self.title = title;
        self.artist = artist;
        self.raw_text = slides.join("\n\n");
        let (final_slides, final_media) = if slides.is_empty() {
            (vec![String::new()], vec![None])
        } else {
            (slides, media)
        };
        self.slides = final_slides;
        self.slide_media = final_media;
        self.preview_slide_index = 0;
        self.is_song = false;
        self.is_bible = self.raw_text.contains("//ref-pos:") || self.raw_text.lines().any(|l| l.trim().starts_with("//ref-"));
        self.sync_preview_media();
    }

    pub fn project_custom_content(&mut self, title: String, artist: String, text: String) {
        self.load_custom_content(title, artist, text);
        self.project_preview();
        self.live_is_song = false;
        self.live_is_bible = self.is_bible;
    }

    pub fn add_or_update_song(&mut self, title: String, artist: String, text: String) {
        let song = Song::new(title, artist, text);
        self.library.push(song);
        self.select_song(self.library.len() - 1);
    }

    pub fn raw_text(&self) -> &str {
        &self.raw_text
    }

    pub fn set_raw_text(&mut self, new_text: String) {
        if self.raw_text != new_text {
            self.raw_text = new_text;
            self.sync_slides();
        }
    }

    pub fn sync_slides(&mut self) {
        if self.is_song && !self.title.trim().is_empty() {
            self.slides = build_song_slides(&self.title, &self.artist, &self.raw_text);
        } else {
            self.slides = parse_slides_from_text(&self.raw_text);
        }
        if self.preview_slide_index >= self.slides.len() && !self.slides.is_empty() {
            self.preview_slide_index = self.slides.len() - 1;
        }
    }

    pub fn move_slide_up(&mut self, index: usize) {
        let min_idx = if self.is_song { 2 } else { 1 };
        let max_idx = if self.is_song && self.slides.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
            self.slides.len().saturating_sub(1)
        } else {
            self.slides.len()
        };
        if index >= min_idx && index < max_idx {
            self.slides.swap(index, index - 1);
            if self.is_song {
                let content_slides: Vec<_> = self.slides[1..].iter().filter(|s| !s.trim().is_empty()).cloned().collect();
                self.raw_text = content_slides.join("\n\n");
            } else {
                self.raw_text = self.slides.join("\n\n");
            }
            self.preview_slide_index = index - 1;
            if self.is_previewing_live_song() {
                self.live_slides = self.slides.clone();
                self.live_index = self.preview_slide_index;
            }
            self.sync_current_song_text();
        }
    }

    pub fn move_slide_down(&mut self, index: usize) {
        let min_idx = if self.is_song { 1 } else { 0 };
        let max_idx = if self.is_song && self.slides.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
            self.slides.len().saturating_sub(2)
        } else {
            self.slides.len().saturating_sub(1)
        };
        if index >= min_idx && index <= max_idx {
            self.slides.swap(index, index + 1);
            if self.is_song {
                let content_slides: Vec<_> = self.slides[1..].iter().filter(|s| !s.trim().is_empty()).cloned().collect();
                self.raw_text = content_slides.join("\n\n");
            } else {
                self.raw_text = self.slides.join("\n\n");
            }
            self.preview_slide_index = index + 1;
            if self.is_previewing_live_song() {
                self.live_slides = self.slides.clone();
                self.live_index = self.preview_slide_index;
            }
            self.sync_current_song_text();
        }
    }

    pub fn duplicate_slide(&mut self, index: usize) {
        let min_idx = if self.is_song { 1 } else { 0 };
        let max_idx = if self.is_song && self.slides.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
            self.slides.len().saturating_sub(1)
        } else {
            self.slides.len()
        };
        if index >= min_idx && index < max_idx {
            let copy = self.slides[index].clone();
            self.slides.insert(index + 1, copy);
            if self.is_song {
                let content_slides: Vec<_> = self.slides[1..].iter().filter(|s| !s.trim().is_empty()).cloned().collect();
                self.raw_text = content_slides.join("\n\n");
            } else {
                self.raw_text = self.slides.join("\n\n");
            }
            self.preview_slide_index = index + 1;
            if self.is_previewing_live_song() {
                self.live_slides = self.slides.clone();
            }
            self.sync_current_song_text();
        }
    }

    pub fn delete_slide(&mut self, index: usize) {
        let (min_idx, min_len) = if self.is_song { (1, 3) } else { (0, 2) };
        let max_idx = if self.is_song && self.slides.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
            self.slides.len().saturating_sub(1)
        } else {
            self.slides.len()
        };
        if self.slides.len() >= min_len && index >= min_idx && index < max_idx {
            self.slides.remove(index);
            if self.is_song {
                let content_slides: Vec<_> = self.slides[1..].iter().filter(|s| !s.trim().is_empty()).cloned().collect();
                self.raw_text = content_slides.join("\n\n");
            } else {
                self.raw_text = self.slides.join("\n\n");
            }
            if self.preview_slide_index >= self.slides.len() {
                self.preview_slide_index = self.slides.len().saturating_sub(1);
            }
            if self.is_previewing_live_song() {
                self.live_slides = self.slides.clone();
                if self.live_index >= self.live_slides.len() {
                    self.live_index = self.live_slides.len().saturating_sub(1);
                }
            }
            self.sync_current_song_text();
        }
    }

    pub fn update_slide_text(&mut self, index: usize, new_text: &str) {
        if index < self.slides.len() {
            self.slides[index] = new_text.trim_end().to_string();
            if self.is_song {
                if self.slides.len() > 1 {
                    self.raw_text = self.slides[1..].join("\n\n");
                }
            } else {
                self.raw_text = self.slides.join("\n\n");
            }
            if self.is_previewing_live_song() {
                self.live_slides = self.slides.clone();
            }
            self.sync_current_song_text();
        }
    }

    fn sync_current_song_text(&mut self) {
        if self.is_song
            && let Some(song) = self.library.get_mut(self.selected_song_index)
        {
            song.text = self.raw_text.clone();
        }
    }

    pub fn slides(&self) -> &[String] {
        &self.slides
    }

    pub fn total_slides(&self) -> usize {
        if self.live_slides.is_empty() {
            self.slides.len()
        } else {
            self.live_slides.len()
        }
    }

    pub fn current_index(&self) -> usize {
        self.live_index
    }

    pub fn is_content_hidden(&self) -> bool {
        self.is_content_hidden
    }

    pub fn toggle_hide_content(&mut self) {
        self.is_content_hidden = !self.is_content_hidden;
    }

    pub fn hide_content(&mut self) {
        self.is_content_hidden = true;
    }

    pub fn current_slide_text(&self) -> &str {
        self.live_slide_text()
    }

    pub fn live_slide_text(&self) -> &str {
        if self.live_slides.is_empty() {
            ""
        } else {
            let idx = self.live_index.min(self.live_slides.len() - 1);
            &self.live_slides[idx]
        }
    }

    pub fn projected_text(&self) -> &str {
        if self.is_content_hidden || self.live_slides.is_empty() {
            ""
        } else {
            self.live_slide_text()
        }
    }

    pub fn next_projected_text(&self) -> &str {
        if self.is_content_hidden || self.live_slides.is_empty() {
            ""
        } else if self.live_index + 1 < self.live_slides.len() {
            &self.live_slides[self.live_index + 1]
        } else {
            ""
        }
    }

    pub fn next_slide(&mut self) -> bool {
        if !self.live_slides.is_empty() && self.live_index + 1 < self.live_slides.len() {
            self.live_index += 1;
            self.is_content_hidden = false;
            self.sync_live_media();
            if self.is_previewing_live_song() {
                self.preview_slide_index = self.live_index;
                self.sync_preview_media();
            }
            true
        } else if self.live_slides.is_empty() && !self.slides.is_empty() && self.preview_slide_index + 1 < self.slides.len() {
            self.preview_slide_index += 1;
            self.sync_preview_media();
            true
        } else {
            false
        }
    }

    pub fn prev_slide(&mut self) {
        if self.live_index > 0 {
            self.live_index -= 1;
            self.is_content_hidden = false;
            self.sync_live_media();
            if self.is_previewing_live_song() {
                self.preview_slide_index = self.live_index;
                self.sync_preview_media();
            }
        } else if self.live_slides.is_empty() && self.preview_slide_index > 0 {
            self.preview_slide_index -= 1;
            self.sync_preview_media();
        }
    }

    pub fn first_slide(&mut self) {
        if !self.live_slides.is_empty() {
            self.live_index = 0;
            self.is_content_hidden = false;
            self.sync_live_media();
            if self.is_previewing_live_song() {
                self.preview_slide_index = 0;
                self.sync_preview_media();
            }
        } else if !self.slides.is_empty() {
            self.preview_slide_index = 0;
            self.sync_preview_media();
        }
    }

    pub fn last_slide(&mut self) {
        if !self.live_slides.is_empty() {
            self.live_index = self.live_slides.len() - 1;
            self.is_content_hidden = false;
            self.sync_live_media();
            if self.is_previewing_live_song() {
                self.preview_slide_index = self.live_index;
                self.sync_preview_media();
            }
        } else if !self.slides.is_empty() {
            self.preview_slide_index = self.slides.len() - 1;
            self.sync_preview_media();
        }
    }

    pub fn select_slide(&mut self, index: usize) {
        self.select_live_slide(index);
    }

    pub fn preview_slide_index(&self) -> usize {
        self.preview_slide_index
    }

    pub fn preview_output(&self) -> Self {
        let mut output = Self::default();
        output.title = self.title.clone();
        output.artist = self.artist.clone();
        output.slides = self.slides.clone();
        output.preview_slide_index = self.preview_slide_index;
        output.preview_design = self.preview_design.clone();
        output.preview_media = self.preview_media.clone();
        if self
            .preview_design
            .as_ref()
            .is_some_and(|scene| scene.overlay)
        {
            output.live_design = self.live_design.clone();
            output.live_media = self.live_media.clone();
            output.live_slides = self.live_slides.clone();
            output.live_index = self.live_index;
            output.live_title = self.live_title.clone();
        }
        output.project_preview();
        if self
            .preview_design
            .as_ref()
            .is_some_and(|scene| scene.overlay)
        {
            output.live_overlay = self.preview_design.clone();
        } else {
            output.live_design = self.preview_design.clone();
        }
        output
    }

    pub fn select_preview_slide(&mut self, index: usize) {
        if index < self.slides.len() {
            self.preview_slide_index = index;
        }
    }

    pub fn preview_slide_text(&self) -> &str {
        if self.slides.is_empty() {
            ""
        } else {
            let idx = self.preview_slide_index.min(self.slides.len() - 1);
            &self.slides[idx]
        }
    }

    pub fn progress(&self) -> f32 {
        if self.live_slides.is_empty() {
            0.0
        } else {
            (self.current_index() + 1) as f32 / self.live_slides.len() as f32
        }
    }
}

pub fn song_title_slide(title: &str, artist: &str) -> String {
    let t = title.trim();
    let a = artist.trim();
    if !a.is_empty() {
        format!("{t}\n{a}")
    } else {
        t.to_string()
    }
}

pub fn build_song_slides(title: &str, artist: &str, raw_text: &str) -> Vec<String> {
    let mut slides = Vec::new();
    let title_slide = song_title_slide(title, artist);
    if !title_slide.is_empty() {
        slides.push(title_slide);
    }
    slides.extend(parse_slides_from_text(raw_text));
    if slides.is_empty() {
        slides.push(String::new());
    } else if slides.last().map(|s| !s.trim().is_empty()).unwrap_or(false) {
        slides.push(String::new());
    }
    slides
}

pub fn parse_slides_from_text(raw_text: &str) -> Vec<String> {
    let mut slides = Vec::new();
    let mut current_block = Vec::new();

    for line in raw_text.lines() {
        if line.trim().is_empty() {
            if !current_block.is_empty() {
                slides.push(current_block.join("\n"));
                current_block.clear();
            }
        } else {
            current_block.push(line.trim_end().to_string());
        }
    }

    if !current_block.is_empty() {
        slides.push(current_block.join("\n"));
    }

    slides
}

pub fn strip_comments(text: &str) -> String {
    text.lines()
        .filter_map(|line| {
            let trimmed_start = line.trim_start();
            if crate::core::markup::is_directive_command(trimmed_start) {
                Some(line.to_string())
            } else if trimmed_start.starts_with("//") || trimmed_start.starts_with("#*") {
                None
            } else if let Some(idx) = line.find("//") {
                let before = line[..idx].trim_end();
                if before.is_empty() {
                    None
                } else {
                    Some(before.to_string())
                }
            } else {
                Some(line.to_string())
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn strip_directive_commands(text: &str) -> String {
    text.lines()
        .filter(|line| !crate::core::markup::is_directive_command(line.trim_start()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Extrai a etiqueta de cabeçalho da estrofe (#*refrão, #*ponte, etc.) se houver.
pub fn extract_strophe_tag(strophe: &str) -> Option<String> {
    for line in strophe.lines() {
        let trimmed = line.trim();
        if let Some(tag) = trimmed.strip_prefix("#*") {
            let tag = tag.trim();
            if !tag.is_empty() {
                return Some(tag.to_string());
            }
        }
    }
    None
}

/// Extrai o texto da estrofe sem a etiqueta de cabeçalho (#*refrão, etc.).
pub fn strip_strophe_tag(strophe: &str) -> String {
    strophe
        .lines()
        .filter(|line| !line.trim_start().starts_with("#*"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Adiciona ou substitui a etiqueta de tipo de estrofe (#*refrão, etc.).
pub fn set_strophe_tag(strophe: &str, tag: Option<&str>) -> String {
    let clean = strip_strophe_tag(strophe);
    let clean = clean.trim();
    if let Some(tag) = tag {
        let tag = tag.trim().trim_start_matches("#*");
        if clean.is_empty() {
            format!("#*{tag}")
        } else {
            format!("#*{tag}\n{clean}")
        }
    } else {
        clean.to_string()
    }
}



