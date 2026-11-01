use crate::core::models::PresentationTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeDragHandle {
    MoveBox,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    TopCenter,
    BottomCenter,
    LeftCenter,
    RightCenter,
}

#[derive(Clone)]
pub struct EditorSnapshot {
    pub title: String,
    pub artist: String,
    pub text: String,
    pub theme: PresentationTheme,
}

#[derive(Clone)]
pub struct SongEditorState {
    pub open: bool,
    pub title: String,
    pub artist: String,
    pub text: String,
    pub folder: String,
    pub tags: String,
    pub zoom: f32,
    pub selected_slide: Option<usize>,
    pub theme: Option<PresentationTheme>,
    pub theme_dirty: bool,
    pub history: Vec<EditorSnapshot>,
    pub future: Vec<EditorSnapshot>,
    pub show_versions: bool,
    pub versions_query: String,
    pub linked_versions: Vec<i64>,
    pub selection: (usize, usize),
    pub preview_ratio_4_3: bool,
    pub hide_syntax_markers: bool,
    pub show_syntax_guide: bool,
}

impl Default for SongEditorState {
    fn default() -> Self {
        Self {
            open: false,
            title: String::new(),
            artist: String::new(),
            text: String::new(),
            folder: String::new(),
            tags: String::new(),
            zoom: 1.0,
            selected_slide: None,
            theme: None,
            theme_dirty: false,
            history: Vec::new(),
            future: Vec::new(),
            show_versions: false,
            versions_query: String::new(),
            linked_versions: Vec::new(),
            selection: (0, 0),
            preview_ratio_4_3: false,
            hide_syntax_markers: false,
            show_syntax_guide: false,
        }
    }
}

#[derive(Clone)]
pub struct ThemeEditorState {
    pub open: bool,
    pub editing_index: Option<usize>,
    pub editing: PresentationTheme,
    pub undo_stack: Vec<PresentationTheme>,
    pub redo_stack: Vec<PresentationTheme>,
    pub zoom: f32,
    pub sample_index: usize,
    pub bg_tab: usize,
    pub active_handle: Option<ThemeDragHandle>,
    pub image_custom_path: String,
    pub show_color_picker: bool,
    pub show_shadow_picker: bool,
    pub sidebar_tab: usize,
    pub is_text_selected: bool,
}

impl Default for ThemeEditorState {
    fn default() -> Self {
        Self {
            open: false,
            editing_index: None,
            editing: PresentationTheme::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            zoom: 1.0,
            sample_index: 0,
            bg_tab: 0,
            active_handle: None,
            image_custom_path: String::new(),
            show_color_picker: false,
            show_shadow_picker: false,
            sidebar_tab: 0,
            is_text_selected: false,
        }
    }
}

#[derive(Clone, Default)]
pub struct StropheEditorState {
    pub open: bool,
    pub index: usize,
    pub text: String,
}
