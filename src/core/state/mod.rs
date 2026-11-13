use crate::core::bible::version_code;
use crate::core::layout::{ContainerKind, LayoutTree};
use crate::core::models::{
    CustomTextItem, MediaItem, PlaylistItem, PresentationTheme, QuickItem, QuickItemKind,
    SavedVerseItem,
};
use crate::ui::theme::AppTheme;

mod editors;
pub mod events;
mod lyrics_search;
mod player;
pub mod setup_wizard;
mod web_media;
#[cfg(test)]
mod theme_preferences_tests;
pub use editors::{
    EditorSnapshot, SongEditorState, StropheEditorState, ThemeDragHandle, ThemeEditorState,
};
pub use events::EventEditorDraft;
pub use lyrics_search::WebLyricsSearchState;
pub use player::{ActivePlayerMedia, PlayerMediaKind};
pub use setup_wizard::SetupWizardState;
pub use web_media::WebMediaModalState;
mod confirm;
pub use confirm::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarPage {
    Home,
    Bible,
    Chat,
    Design,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    Outputs,
    Interface,
    Shortcuts,
    Plugins,
    Bible,
    Connections,
    Profiles,
    About,
    Transfer,
}

/// Target for the floating screen preview window.
#[derive(Clone, Debug, PartialEq)]
pub enum FloatingPreviewTarget {
    Physical { monitor_name: String, index: usize },
    Virtual { id: String, name: String },
}

/// State for the floating screen preview window.
#[derive(Clone, Debug, PartialEq)]
pub struct FloatingPreviewState {
    pub open: bool,
    pub target: FloatingPreviewTarget,
    pub aspect_ratio: f32,
    pub show_info: bool,
}

impl Default for FloatingPreviewState {
    fn default() -> Self {
        Self {
            open: true,
            target: FloatingPreviewTarget::Physical {
                monitor_name: String::new(),
                index: 1,
            },
            aspect_ratio: 16.0 / 9.0,
            show_info: true,
        }
    }
}

/// Ephemeral UI state for the settings modal and browser server status.
#[derive(Clone)]
pub struct SettingsUiState {
    pub plugin_request: Option<crate::core::plugins::PluginRequest>,
    pub shortcut_capture: crate::core::shortcuts::ShortcutCapture,
    pub open: bool,
    pub tab: SettingsTab,
    pub draft: Option<crate::core::preferences::AppSettings>,
    pub save_requested: bool,
    pub error: String,
    pub server_status: String,
    pub server_running: bool,
    pub lan_address: String,
    pub discovery_url: String,
    pub discovery_chat_url: String,
    pub discovery_error: String,
    pub floating_preview: Option<FloatingPreviewState>,
    pub show_physical_config: bool,
}

impl Default for SettingsUiState {
    fn default() -> Self {
        Self {
            plugin_request: None,
            shortcut_capture: Default::default(),
            open: false,
            tab: SettingsTab::Outputs,
            draft: None,
            save_requested: false,
            error: String::new(),
            server_status: "Servidor desativado".into(),
            server_running: false,
            lan_address: crate::services::web::local_address(),
            discovery_url: String::new(),
            discovery_chat_url: String::new(),
            discovery_error: String::new(),
            floating_preview: None,
            show_physical_config: false,
        }
    }
}

/// Team chat state.
#[derive(Clone)]
pub struct ChatNotice {
    pub item: crate::services::chat::NotificationItem,
    pub since: std::time::Instant,
}

#[derive(Clone, Default)]
pub struct ChatState {
    pub channel_selection: crate::core::interaction::Selection<String>,
    pub editing_channel: Option<String>,
    pub service: Option<crate::services::chat::ChatService>,
    pub token: String,
    pub profile_id: String,
    pub input: String,
    pub channel: String,
    pub peer: Option<String>,
    pub search: String,
    pub search_all: bool,
    pub new_channel: String,
    pub new_channel_description: String,
    pub new_channel_tool: String,
    pub show_channels: bool,
    pub bible_reference: String,
    pub bible_version: String,
    pub show_bible: bool,
    pub show_songs: bool,
    pub song_search: String,
    pub show_emoji: bool,
    pub show_access: bool,
    pub error: String,
    pub https_error: Option<String>,
    pub recorder: Option<crate::services::chat::recording::Recorder>,
    pub recording_target: Option<crate::services::chat::SendMessage>,
    pub recording_started: Option<std::time::Instant>,
    pub recording_cancelled: bool,
    pub last_sync: Option<std::time::Instant>,
    pub history_before: Option<u64>,
    pub audio_player: Option<crate::services::audio::AudioPlayer>,
    pub audio_file_id: Option<String>,
    pub search_open: bool,
    pub search_focus: bool,
    pub notifications: crate::services::chat::Notifications,
    pub notification_cursor: Option<u64>,
    pub last_notification_poll: Option<std::time::Instant>,
    pub notice: Option<ChatNotice>,
    pub emoji_search: String,
    pub emoji_category: usize,
    pub emoji_anchor: Option<eframe::egui::Rect>,
    pub emoji_opened_frame: u64,
    pub reacting_to: Option<u64>,
    pub cached_snapshot: Option<crate::services::chat::Snapshot>,
}

/// Top bar quick items ("atalhos") state.
#[derive(Clone, Default)]
pub struct QuickBarState {
    pub items: Vec<QuickItem>,
    pub active_id: Option<String>,
}

#[derive(Clone)]
pub struct ControlPanelState {
    pub theme: AppTheme,
    pub active_sidebar_page: SidebarPage,
    pub search_query: String,
    pub tag_filter: String,
    pub settings: crate::core::preferences::AppSettings,

    pub song_editor: SongEditorState,
    pub theme_editor: ThemeEditorState,
    pub strophe_editor: StropheEditorState,
    pub settings_ui: SettingsUiState,
    pub chat: ChatState,
    pub quick_bar: QuickBarState,

    pub toast: Option<(String, std::time::Instant)>,
    pub selected_tab: usize, // 0 = Letras, 1 = Mídias, 2 = Textos, 3 = Versículos, 4 = Designs
    pub containers: [ContainerKind; 7],
    pub layout: LayoutTree,
    pub bible_layout: LayoutTree,
    pub chat_layout: LayoutTree,
    pub bible: crate::core::bible::BibleState,
    pub right_split_ratio: f32,
    pub themes: Vec<PresentationTheme>,
    pub selected_theme_index: usize,
    pub live_theme_id: String,
    pub live_theme_index: usize,

    // Playlist state
    pub playlist: Vec<PlaylistItem>,
    pub selected_playlist_index: Option<usize>,
    pub playlist_selection: crate::core::interaction::Selection<String>,
    pub show_event_editor_modal: bool,
    pub event_editor_draft: Option<EventEditorDraft>,
    pub open_folders: std::collections::HashSet<String>,
    pub show_create_folder_modal: bool,
    pub new_folder_name: String,
    pub web_lyrics_search: WebLyricsSearchState,
    pub web_media_modal: WebMediaModalState,
    pub is_blackout: bool,
    pub show_presentation_dock: bool,
    pub show_presentation_settings_modal: bool,
    pub show_setup_wizard: bool,
    pub show_profile_picker_modal: bool,
    pub show_web_hub_modal: bool,
    pub setup_wizard: SetupWizardState,
    pub presentation_started_at: Option<std::time::Instant>,
    pub presentation_timer_seconds_mode: bool,
    pub focus_dock_jump_input: bool,
    pub focused_container: Option<ContainerKind>,
    pub show_search: bool,
    pub presentation_aspect_ratio: f32,
    /// Render slide-card previews with the presentation theme background.
    pub dock_preview_bg: bool,
    /// Height of the presentation dock panel.
    pub dock_height: f32,
    /// Show slide thumbnails instead of the full text list in the lyrics panel.
    pub lyrics_cards_layout: bool,

    // Multiselect state for songs
    pub selected_song_ids: std::collections::HashSet<i64>,
    pub song_selection_anchor: Option<i64>,

    // Media items & selection
    pub media_items: Vec<MediaItem>,
    pub media_import_task: Option<(String, crate::services::tasks::BackgroundTask<Vec<std::path::PathBuf>>)>,
    pub media_conversion_prompt: Option<crate::services::media_conversion::Prompt>,
    pub media_conversion_retry: Option<crate::services::media_conversion::Prompt>,
    pub media_conversion_progress: Option<std::sync::Arc<std::sync::Mutex<crate::services::media_conversion::ConversionProgress>>>,
    pub media_conversion_running: bool,
    pub media_info_dialog: Option<crate::services::media_info::MediaDetails>,
    pub confirm_dialog: Option<ConfirmDialogState>,
    pub open_media_folders: std::collections::HashSet<String>,
    pub selected_media_ids: std::collections::HashSet<i64>,
    pub media_selection_anchor: Option<i64>,

    // Custom texts & selection
    pub custom_texts: Vec<CustomTextItem>,
    pub open_text_folders: std::collections::HashSet<String>,
    pub selected_text_ids: std::collections::HashSet<i64>,
    pub text_selection_anchor: Option<i64>,

    // Saved verses & selection
    pub saved_verses: Vec<SavedVerseItem>,
    pub open_verse_folders: std::collections::HashSet<String>,
    pub selected_verse_ids: std::collections::HashSet<i64>,
    pub verse_selection_anchor: Option<i64>,

    // Designs & selection
    pub open_design_folders: std::collections::HashSet<String>,
    pub selected_design_ids: std::collections::HashSet<String>,
    pub design_selection_anchor: Option<String>,
    pub design_list_view: bool,

    // Modals for Text creation/editing
    pub show_create_text_modal: bool,
    pub editing_text_id: Option<i64>,
    pub text_editor_title: String,
    pub text_editor_content: String,
    pub text_editor_folder: String,

    // Modals for Verse saving
    pub show_save_verse_modal: bool,
    pub verse_editor_ref: String,
    pub verse_editor_version: String,
    pub verse_editor_text: String,
    pub verse_editor_folder: String,

    // Modal for Tag management
    pub show_tag_manager_modal: bool,
    pub tag_manager_new_name: String,
    pub tag_manager_new_color: [u8; 3],

    // Bible projection state
    pub bible_book: String,
    pub bible_chapter: usize,
    pub bible_verse: usize,
    pub bible_version: String,
    pub bible_shortcuts: Vec<String>,

    // Player (Áudios e Vídeos)
    pub audio_player: crate::services::audio::AudioPlayer,
    pub active_player_media: Option<ActivePlayerMedia>,
}

impl Default for ControlPanelState {
    fn default() -> Self {
        Self {
            theme: AppTheme::light(),
            active_sidebar_page: SidebarPage::Home,
            search_query: String::new(),
            tag_filter: String::new(),
            settings: crate::core::preferences::AppSettings::default(),

            song_editor: SongEditorState::default(),
            theme_editor: ThemeEditorState::default(),
            strophe_editor: StropheEditorState::default(),
            settings_ui: SettingsUiState::default(),
            chat: ChatState::default(),
            quick_bar: QuickBarState {
                items: Vec::new(),
                active_id: None,
            },

            toast: None,
            selected_tab: 0,
            containers: [
                ContainerKind::Selection, // Slot 0: Seleção
                ContainerKind::Lyrics,    // Slot 1: Visualização de Letra
                ContainerKind::Stage,     // Slot 2: Retorno Ao Vivo
                ContainerKind::Preview,   // Slot 3: Visualização
                ContainerKind::Themes,    // Slot 4: Temas
                ContainerKind::Playlist,  // Slot 5: Lista de reprodução
                ContainerKind::Player,    // Slot 6: Player (Vídeos e Áudio)
            ],
            layout: LayoutTree::default(),
            bible_layout: LayoutTree::bible_workspace(),
            chat_layout: LayoutTree::default_chat(),
            bible: crate::core::bible::BibleState::default(),
            right_split_ratio: 0.58,
            themes: PresentationTheme::default_themes(),
            selected_theme_index: 0,
            live_theme_id: "obsidian_pro".to_owned(),
            live_theme_index: 0,

            playlist: Vec::new(),
            selected_playlist_index: None,
            playlist_selection: crate::core::interaction::Selection::default(),
            show_event_editor_modal: false,
            event_editor_draft: None,
            open_folders: std::collections::HashSet::new(),
            show_create_folder_modal: false,
            new_folder_name: String::new(),
            web_lyrics_search: WebLyricsSearchState::default(),
            web_media_modal: WebMediaModalState::default(),
            is_blackout: false,
            show_presentation_dock: true,
            show_presentation_settings_modal: false,
            show_setup_wizard: false,
            show_profile_picker_modal: false,
            show_web_hub_modal: false,
            setup_wizard: SetupWizardState::default(),
            presentation_started_at: None,
            presentation_timer_seconds_mode: false,
            focus_dock_jump_input: false,
            focused_container: Some(ContainerKind::Selection),
            show_search: false,
            presentation_aspect_ratio: 16.0 / 9.0,
            dock_preview_bg: false,
            dock_height: 175.0,
            lyrics_cards_layout: false,
            selected_song_ids: std::collections::HashSet::new(),
            song_selection_anchor: None,

            media_items: Vec::new(),
            media_import_task: None,
            media_conversion_prompt: None,
            media_conversion_retry: None,
            media_conversion_progress: None,
            media_conversion_running: false,
            media_info_dialog: None,
            open_media_folders: std::collections::HashSet::new(),
            selected_media_ids: std::collections::HashSet::new(),
            media_selection_anchor: None,

            custom_texts: Vec::new(),
            open_text_folders: std::collections::HashSet::new(),
            selected_text_ids: std::collections::HashSet::new(),
            text_selection_anchor: None,

            saved_verses: Vec::new(),
            open_verse_folders: std::collections::HashSet::new(),
            selected_verse_ids: std::collections::HashSet::new(),
            verse_selection_anchor: None,

            open_design_folders: std::collections::HashSet::new(),
            selected_design_ids: std::collections::HashSet::new(),
            design_selection_anchor: None,
            design_list_view: true,

            show_create_text_modal: false,
            editing_text_id: None,
            text_editor_title: String::new(),
            text_editor_content: String::new(),
            text_editor_folder: String::new(),

            show_save_verse_modal: false,
            verse_editor_ref: String::new(),
            verse_editor_version: crate::core::bible::DEFAULT_BIBLE.to_owned(),
            verse_editor_text: String::new(),
            verse_editor_folder: String::new(),

            show_tag_manager_modal: false,
            tag_manager_new_name: String::new(),
            tag_manager_new_color: [14, 165, 233],

            bible_book: "Gênesis".to_owned(),
            bible_chapter: 1,
            bible_verse: 1,
            bible_version: crate::core::bible::DEFAULT_BIBLE.to_owned(),
            bible_shortcuts: vec![crate::core::bible::DEFAULT_BIBLE.to_owned()],

            audio_player: crate::services::audio::AudioPlayer::default(),
            active_player_media: None,
            confirm_dialog: None,
        }
    }
}

impl ControlPanelState {
    pub fn clear_song_selection(&mut self) {
        self.selected_song_ids.clear();
        self.song_selection_anchor = None;
    }

    pub fn is_song_selected(&self, id: i64) -> bool {
        self.selected_song_ids.contains(&id)
    }

    pub fn clear_media_selection(&mut self) {
        self.selected_media_ids.clear();
        self.media_selection_anchor = None;
    }

    pub fn is_media_selected(&self, id: i64) -> bool {
        self.selected_media_ids.contains(&id)
    }

    pub fn clear_text_selection(&mut self) {
        self.selected_text_ids.clear();
        self.text_selection_anchor = None;
    }

    pub fn is_text_selected(&self, id: i64) -> bool {
        self.selected_text_ids.contains(&id)
    }

    pub fn clear_verse_selection(&mut self) {
        self.selected_verse_ids.clear();
        self.verse_selection_anchor = None;
    }

    pub fn is_verse_selected(&self, id: i64) -> bool {
        self.selected_verse_ids.contains(&id)
    }

    pub fn clear_design_selection(&mut self) {
        self.selected_design_ids.clear();
        self.design_selection_anchor = None;
    }

    pub fn clear_all_selections(&mut self) {
        self.clear_song_selection();
        self.clear_media_selection();
        self.clear_text_selection();
        self.clear_verse_selection();
        self.clear_design_selection();
    }

    pub fn reload_media_items(&mut self, db: &crate::services::db::AppDb) {
        if let Ok(items) = db.get_all_media_items() {
            self.media_items = items;
        }
    }

    pub fn reload_custom_texts(&mut self, db: &crate::services::db::AppDb) {
        if let Ok(items) = db.get_all_custom_texts() {
            self.custom_texts = items;
        }
    }

    pub fn reload_saved_verses(&mut self, db: &crate::services::db::AppDb) {
        if let Ok(items) = db.get_all_saved_verses() {
            self.saved_verses = items;
        }
    }

    pub fn reload_all_selection_items(&mut self, db: &crate::services::db::AppDb) {
        self.reload_media_items(db);
        self.reload_custom_texts(db);
        self.reload_saved_verses(db);
    }

    pub fn notify(&mut self, message: impl Into<String>) {
        self.toast = Some((message.into(), std::time::Instant::now()));
    }

    pub fn is_item_converting(&self, item_path: &str) -> bool {
        if !self.media_conversion_running {
            return false;
        }
        if let Some(prog) = &self.media_conversion_progress {
            if let Ok(g) = prog.lock() {
                if g.current_path.to_string_lossy() == item_path {
                    return true;
                }
            }
        }
        if let Some(prompt) = &self.media_conversion_retry {
            return prompt.paths.iter().any(|p| p.to_string_lossy() == item_path);
        }
        false
    }

    pub fn media_conversion_percent(&self) -> f32 {
        self.media_conversion_progress
            .as_ref()
            .and_then(|p| p.lock().ok())
            .map(|g| g.percent)
            .unwrap_or(0.0)
    }

    pub fn media_conversion_info_tooltip(&self) -> String {
        let mut info = String::from("Convertendo mídia em segundo plano");
        if let Some(prog) = &self.media_conversion_progress {
            if let Ok(g) = prog.lock() {
                if !g.current_filename.is_empty() {
                    info.push_str(&format!("\n• Arquivo: {} ({}/{})", g.current_filename, g.current_file, g.total_files));
                }
                if let Some(d) = &g.details {
                    if let Some(dim) = &d.dimensions_formatted {
                        let ar = d.aspect_ratio.as_deref().unwrap_or("");
                        info.push_str(&format!("\n• Resolução: {dim} ({ar})"));
                    }
                    if let Some(dur) = &d.duration_formatted {
                        info.push_str(&format!("\n• Duração: {dur}"));
                    }
                    if let Some(codec) = &d.video_codec {
                        info.push_str(&format!("\n• Codec: {codec}"));
                    }
                }
                info.push_str(&format!("\n• Progresso: {:.0}%", (g.percent * 100.0).clamp(0.0, 100.0)));
            }
        }
        info
    }

    pub fn capture_profiles(&mut self) {
        if let Some(p) = self
            .settings
            .operators
            .iter_mut()
            .find(|p| p.id == self.settings.active_operator)
        {
            p.layout = self.layout.clone();
            p.bible_layout = self.bible_layout.clone();
            p.chat_layout = self.chat_layout.clone();
            p.quick_items = self.quick_bar.items.clone();
            p.show_dock = self.show_presentation_dock;
            p.dock_preview_bg = self.dock_preview_bg;
            p.dock_height = self.dock_height;
            p.lyrics_cards_layout = self.lyrics_cards_layout;
            p.bible_cards_layout = self.bible.cards_layout;
        }
        if let Some(p) = self
            .settings
            .services
            .iter_mut()
            .find(|p| p.id == self.settings.active_service)
        {
            p.playlist = self.playlist.clone();
            p.last_selected_theme_id = self.themes.get(self.selected_theme_index).map(|t| t.id.clone());
            p.last_selected_theme_default_id = Some(p.theme_id.clone());
            let session_date = p.current_session_date();
            p.save_session_playlist(&session_date, self.playlist.clone());
            p.bible_version = self.bible_version.clone();
            p.bible_shortcuts = self.bible_shortcuts.clone();
            p.bible_shortcuts_initialized = true;
            p.last_bible_book = self.bible_book.clone();
            p.last_bible_chapter = self.bible_chapter;
            p.last_bible_verse = self.bible_verse;
            p.last_bible_verse_end = self.bible.verse_end;
        }
        self.settings.last_bible_book = self.bible_book.clone();
        self.settings.last_bible_chapter = self.bible_chapter;
        self.settings.last_bible_verse = self.bible_verse;
        self.settings.last_bible_verse_end = self.bible.verse_end;
    }

    pub fn apply_profiles(&mut self) {
        self.settings.normalize();
        self.apply_operator_profile();
        self.apply_service_profile();
    }

    fn apply_operator_profile(&mut self) {
        let mut operator = self.settings.operator().clone();
        operator.layout.root.sanitize_fractions();
        operator.bible_layout.root.sanitize_fractions();
        operator.chat_layout.root.sanitize_fractions();
        self.layout = operator.layout;
        self.bible_layout = operator.bible_layout;
        self.chat_layout = operator.chat_layout;
        self.quick_bar.items = operator.quick_items;
        self.quick_bar.active_id = self.quick_bar.items.first().map(|q| q.id.clone());
        self.show_presentation_dock = operator.show_dock;
        self.dock_preview_bg = operator.dock_preview_bg;
        self.dock_height = if operator.dock_height >= 100.0 {
            operator.dock_height
        } else {
            175.0
        };
        self.lyrics_cards_layout = operator.lyrics_cards_layout;
        self.bible.cards_layout = operator.bible_cards_layout;
        self.theme = if operator.dark_mode {
            AppTheme::dark()
        } else {
            AppTheme::light()
        };
    }

    fn apply_service_profile(&mut self) {
        let service = self.settings.service().clone();
        let session_date = service.current_session_date();
        if let Some(session) = service.get_session(&session_date) {
            self.playlist = session.playlist.clone();
        } else {
            self.playlist = service.playlist;
        }
        self.selected_playlist_index = if self.playlist.is_empty() {
            None
        } else {
            Some(0)
        };
        self.bible_version = if self.bible.library.get(&service.bible_version).is_some() {
            service.bible_version
        } else {
            self.bible.library.default_version_for_language(self.settings.language)
        };
        let available: Vec<String> = self
            .bible
            .library
            .catalog()
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        self.bible_shortcuts = service
            .bible_shortcuts
            .into_iter()
            .filter(|id| available.contains(&version_code(id)))
            .collect();
        // Seed a new/legacy profile once, but preserve an explicitly empty bar.
        if !service.bible_shortcuts_initialized && self.bible_shortcuts.is_empty() {
            self.bible_shortcuts.push(self.bible_version.clone());
        }
        let last_book = if !service.last_bible_book.trim().is_empty() {
            service.last_bible_book
        } else if !self.settings.last_bible_book.trim().is_empty() {
            self.settings.last_bible_book.clone()
        } else {
            crate::core::bible::book_names_for_language(self.settings.language)[0].to_string()
        };
        self.bible_book = last_book;
        self.bible_chapter = service.last_bible_chapter.max(1);
        self.bible_verse = service.last_bible_verse.max(1);
        self.bible.verse_end = service.last_bible_verse_end.max(self.bible_verse);
        self.bible.selection_anchor = self.bible_verse;
        let book_index = crate::core::bible::book_canonical_index("", &self.bible_book)
            .unwrap_or(0);
        self.bible.expanded_book = Some(book_index);
        self.selected_theme_index = service.last_selected_theme_id.as_ref()
            .filter(|_| service.last_selected_theme_default_id.as_deref() == Some(service.theme_id.as_str()))
            .and_then(|id| self.themes.iter().position(|theme| &theme.id == id))
            .or_else(|| self.themes.iter().position(|theme| theme.id == service.theme_id))
            .unwrap_or(0);
        if (!self.settings.bible_enabled && self.active_sidebar_page == SidebarPage::Bible)
            || (!self.settings.chat_enabled && self.active_sidebar_page == SidebarPage::Chat)
        {
            self.active_sidebar_page = SidebarPage::Home;
        }
    }

    pub fn switch_operator(&mut self, id: &str) {
        if self.settings.active_operator == id
            || !self.settings.operators.iter().any(|p| p.id == id)
        {
            return;
        }
        self.capture_profiles();
        self.settings.active_operator = id.into();
        self.apply_operator_profile();
        let name = self.settings.operator().name.clone();
        self.notify(self.settings.language.format("services-operator", &[("name", &name)]));
    }

    pub fn switch_service(&mut self, id: &str) {
        if self.settings.active_service == id || !self.settings.services.iter().any(|p| p.id == id)
        {
            return;
        }
        self.capture_profiles();
        self.settings.active_service = id.into();
        self.apply_service_profile();
        let name = self.settings.service().name.clone();
        self.notify(self.settings.language.format("services-service", &[("name", &name)]));
    }

    pub fn add_quick_item(
        &mut self,
        title: String,
        subtitle: String,
        kind: QuickItemKind,
        content: String,
    ) {
        let mut candidate = QuickItem {
            id: String::new(),
            title,
            subtitle,
            kind,
            content,
        };
        if let Some(existing) = self
            .quick_bar
            .items
            .iter_mut()
            .find(|item| item.same_resource(&candidate))
        {
            existing.title = candidate.title;
            existing.subtitle = candidate.subtitle;
            if !candidate.content.is_empty() {
                existing.content = candidate.content;
            }
            self.quick_bar.active_id = Some(existing.id.clone());
            return;
        }

        let prefix = match candidate.kind {
            QuickItemKind::Song => "song",
            QuickItemKind::Media => "media",
            QuickItemKind::Verse => "verse",
            QuickItemKind::Text => "text",
            QuickItemKind::Design => "design",
        };
        candidate.id = crate::core::preferences::new_id(prefix);
        self.quick_bar.active_id = Some(candidate.id.clone());
        self.quick_bar.items.push(candidate);
    }

    pub fn preview_theme(&self) -> &PresentationTheme {
        if self.selected_theme_index < self.themes.len() {
            &self.themes[self.selected_theme_index]
        } else if !self.themes.is_empty() {
            &self.themes[0]
        } else {
            Self::fallback_default_theme()
        }
    }

    pub fn live_theme(&self) -> &PresentationTheme {
        if let Some(t) = self.themes.iter().find(|t| t.id == self.live_theme_id) {
            t
        } else if self.live_theme_index < self.themes.len() {
            &self.themes[self.live_theme_index]
        } else {
            self.preview_theme()
        }
    }

    pub fn active_theme(&self) -> &PresentationTheme {
        self.live_theme()
    }

    pub fn apply_theme_to_live(&mut self) {
        if self.selected_theme_index < self.themes.len() {
            self.live_theme_id = self.themes[self.selected_theme_index].id.clone();
            self.live_theme_index = self.selected_theme_index;
        }
    }

    pub fn open_create_theme(&mut self) {
        self.theme_editor.editing_index = None;
        let mut new_theme = PresentationTheme::default();
        new_theme.id = format!("custom_{}", self.themes.len() + 1);
        new_theme.name = self.settings.language.format(
            "services-custom-theme",
            &[("name", &(self.themes.len() + 1).to_string())],
        );
        self.theme_editor.editing = new_theme;
        self.theme_editor.undo_stack.clear();
        self.theme_editor.redo_stack.clear();
        self.theme_editor.zoom = 1.0;
        self.theme_editor.bg_tab = match self.theme_editor.editing.bg_type.as_str() {
            "image" => 1,
            "video" => 2,
            "shader" => 3,
            _ => 0,
        };
        self.theme_editor.active_handle = None;
        self.theme_editor.open = true;
    }

    pub fn open_edit_theme(&mut self, index: usize) {
        if index < self.themes.len() {
            self.theme_editor.editing_index = Some(index);
            let mut editing = self.themes[index].clone();
            editing.name = self.settings.language.text(&editing.name).to_string();
            self.theme_editor.editing = editing;
            self.theme_editor.undo_stack.clear();
            self.theme_editor.redo_stack.clear();
            self.theme_editor.zoom = 1.0;
            self.theme_editor.bg_tab = match self.theme_editor.editing.bg_type.as_str() {
                "image" => 1,
                "video" => 2,
                "shader" => 3,
                _ => 0,
            };
            self.theme_editor.active_handle = None;
            self.theme_editor.open = true;
        }
    }

    fn fallback_default_theme() -> &'static PresentationTheme {
        static DEFAULT_THEME: std::sync::LazyLock<PresentationTheme> =
            std::sync::LazyLock::new(PresentationTheme::default);
        &DEFAULT_THEME
    }

    pub fn reset_containers(&mut self) {
        if self.active_sidebar_page == SidebarPage::Bible {
            self.bible_layout = LayoutTree::bible_workspace();
            return;
        }
        self.layout = LayoutTree::default_4_columns();
        self.containers = [
            ContainerKind::Selection,
            ContainerKind::Lyrics,
            ContainerKind::Stage,
            ContainerKind::Preview,
            ContainerKind::Themes,
            ContainerKind::Playlist,
            ContainerKind::Player,
        ];
        self.right_split_ratio = 0.58;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_and_service_profiles_restore_independent_workspaces() {
        let mut state = ControlPanelState::default();
        state.add_quick_item(
            "Song".into(),
            "Artist".into(),
            QuickItemKind::Song,
            "Text".into(),
        );
        state.capture_profiles();
        let original_layout = state.layout.clone();
        let original_playlist = state.playlist.clone();
        state
            .settings
            .operators
            .push(crate::core::preferences::OperatorProfile {
                id: "ana".into(),
                name: "Ana".into(),
                dark_mode: true,
                scale: 125,
                ..Default::default()
            });
        state
            .settings
            .services
            .push(crate::core::preferences::ServiceProfile {
                id: "jovens".into(),
                name: "Jovens".into(),
                theme_id: "midnight_sapphire".into(),
                ..Default::default()
            });
        state.selected_theme_index = 1;
        state.switch_operator("ana");
        assert_eq!(state.theme, AppTheme::dark());
        assert_eq!(state.settings.operator().scale, 125);
        assert!(state.quick_bar.items.is_empty());
        assert_eq!(state.playlist, original_playlist);
        assert_eq!(
            state.selected_theme_index, 1,
            "An operator change must not reset the service's preview theme"
        );
        state
            .layout
            .root
            .swap_leaves(ContainerKind::Lyrics, ContainerKind::Selection);
        let ana_layout = state.layout.clone();
        state.switch_service("jovens");
        assert!(state.playlist.is_empty());
        assert_eq!(state.layout, ana_layout);
        assert_eq!(
            state.live_theme_id, "obsidian_pro",
            "Profile switching must not replace the live theme"
        );
        state.switch_service("service_default");
        assert_eq!(state.playlist, original_playlist);
        state.switch_operator("operator_default");
        assert_eq!(state.layout, original_layout);
        assert!(!state.quick_bar.items.is_empty());
        state.switch_operator("ana");
        assert_eq!(state.layout, ana_layout);
    }

    #[test]
    fn test_container_swapping() {
        let mut state = ControlPanelState::default();
        assert_eq!(
            state.containers,
            [
                ContainerKind::Selection,
                ContainerKind::Lyrics,
                ContainerKind::Stage,
                ContainerKind::Preview,
                ContainerKind::Themes,
                ContainerKind::Playlist,
                ContainerKind::Player,
            ]
        );

        state.containers.swap(5, 2);
        assert_eq!(
            state.containers,
            [
                ContainerKind::Selection,
                ContainerKind::Lyrics,
                ContainerKind::Playlist,
                ContainerKind::Preview,
                ContainerKind::Themes,
                ContainerKind::Stage,
                ContainerKind::Player,
            ]
        );
    }

    #[test]
    fn test_theme_preview_and_live_separation() {
        let mut state = ControlPanelState::default();
        // Initial live theme is obsidian_pro
        assert_eq!(state.live_theme().id, "obsidian_pro");
        assert_eq!(state.preview_theme().id, "obsidian_pro");

        // Operator clicks theme 1 ("midnight_sapphire") in the themes container
        state.selected_theme_index = 1;

        // Preview theme changes to midnight_sapphire
        assert_eq!(state.preview_theme().id, "midnight_sapphire");

        // Live theme remains obsidian_pro completely unchanged!
        assert_eq!(state.live_theme().id, "obsidian_pro");
        assert_eq!(state.active_theme().id, "obsidian_pro");

        // When projected or applied to live:
        state.apply_theme_to_live();

        // Now live theme updates to midnight_sapphire
        assert_eq!(state.live_theme().id, "midnight_sapphire");
        assert_eq!(state.active_theme().id, "midnight_sapphire");
    }
}
