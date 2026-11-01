use crate::services::tasks::BackgroundTask;

#[derive(Clone, Default)]
pub struct WebMediaModalState {
    pub open: bool,
    pub url: String,
    pub title: String,
    pub target_folder: String,
    pub download_offline: bool,
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub task: Option<BackgroundTask<(String, String)>>,
}

impl WebMediaModalState {
    pub fn reset(&mut self) {
        self.url.clear();
        self.title.clear();
        self.download_offline = false;
        self.is_loading = false;
        self.error_message = None;
        self.task = None;
    }
}
