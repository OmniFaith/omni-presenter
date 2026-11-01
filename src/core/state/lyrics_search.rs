use crate::services::lrclib::LrcLibTrack;
use crate::services::tasks::BackgroundTask;
use std::time::Instant;

#[derive(Clone, Default)]
pub struct WebLyricsSearchState {
    pub open: bool,
    pub query: String,
    pub artist_query: String,
    pub last_searched_query: String,
    pub last_keystroke: Option<Instant>,
    pub is_loading: bool,
    pub results: Vec<LrcLibTrack>,
    pub selected_index: Option<usize>,
    pub error_message: Option<String>,
    pub target_folder: String,
    pub task: Option<BackgroundTask<Vec<LrcLibTrack>>>,
}

impl WebLyricsSearchState {
    pub fn reset_search(&mut self) {
        self.results.clear();
        self.selected_index = None;
        self.error_message = None;
        self.is_loading = false;
        self.task = None;
        self.last_searched_query.clear();
        self.last_keystroke = None;
    }
}
