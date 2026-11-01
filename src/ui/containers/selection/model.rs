use crate::core::{
    models::{PlaylistItemKind, QuickItemKind},
    slides::SlideDeck,
    state::ControlPanelState,
};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LibraryKind {
    Song,
    Media,
    Text,
    Verse,
}

impl LibraryKind {
    pub fn from_tab(tab: usize) -> Self {
        match tab {
            1 => Self::Media,
            2 => Self::Text,
            3 => Self::Verse,
            _ => Self::Song,
        }
    }

    pub fn category(self) -> &'static str {
        match self {
            Self::Song => "song",
            Self::Media => "media",
            Self::Text => "text",
            Self::Verse => "bible",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Song => "library-songs",
            Self::Media => "library-media",
            Self::Text => "library-texts",
            Self::Verse => "library-verses",
        }
    }

    pub fn create_label(self) -> &'static str {
        match self {
            Self::Song => "library-new-song",
            Self::Media => "library-import",
            Self::Text => "library-new-text",
            Self::Verse => "library-save-verse",
        }
    }

    pub fn icon(self) -> &'static str {
        use egui_phosphor::regular::*;
        match self {
            Self::Song => MUSIC_NOTES,
            Self::Media => VIDEO,
            Self::Text => FILE_TEXT,
            Self::Verse => BOOK_OPEN,
        }
    }

    pub fn playlist_kind(self) -> PlaylistItemKind {
        match self {
            Self::Song => PlaylistItemKind::Song,
            Self::Media => PlaylistItemKind::Media,
            Self::Text => PlaylistItemKind::Text,
            Self::Verse => PlaylistItemKind::Bible,
        }
    }

    pub fn quick_kind(self) -> QuickItemKind {
        match self {
            Self::Song => QuickItemKind::Song,
            Self::Media => QuickItemKind::Media,
            Self::Text => QuickItemKind::Text,
            Self::Verse => QuickItemKind::Verse,
        }
    }

    pub fn selected(self, state: &ControlPanelState) -> &HashSet<i64> {
        match self {
            Self::Song => &state.selected_song_ids,
            Self::Media => &state.selected_media_ids,
            Self::Text => &state.selected_text_ids,
            Self::Verse => &state.selected_verse_ids,
        }
    }

    pub fn selected_mut(self, state: &mut ControlPanelState) -> &mut HashSet<i64> {
        match self {
            Self::Song => &mut state.selected_song_ids,
            Self::Media => &mut state.selected_media_ids,
            Self::Text => &mut state.selected_text_ids,
            Self::Verse => &mut state.selected_verse_ids,
        }
    }

    pub fn anchor(self, state: &mut ControlPanelState) -> &mut Option<i64> {
        match self {
            Self::Song => &mut state.song_selection_anchor,
            Self::Media => &mut state.media_selection_anchor,
            Self::Text => &mut state.text_selection_anchor,
            Self::Verse => &mut state.verse_selection_anchor,
        }
    }

    pub fn folders(self, state: &ControlPanelState) -> &HashSet<String> {
        match self {
            Self::Song => &state.open_folders,
            Self::Media => &state.open_media_folders,
            Self::Text => &state.open_text_folders,
            Self::Verse => &state.open_verse_folders,
        }
    }

    pub fn folders_mut(self, state: &mut ControlPanelState) -> &mut HashSet<String> {
        match self {
            Self::Song => &mut state.open_folders,
            Self::Media => &mut state.open_media_folders,
            Self::Text => &mut state.open_text_folders,
            Self::Verse => &mut state.open_verse_folders,
        }
    }

    pub fn clear(self, state: &mut ControlPanelState) {
        self.selected_mut(state).clear();
        *self.anchor(state) = None;
    }
}

#[derive(Clone, Debug)]
pub(super) struct LibraryEntry {
    pub id: i64,
    pub index: usize,
    pub title: String,
    pub subtitle: String,
    pub content: String,
    pub folder: String,
    pub tags: String,
}

pub(super) fn entries(
    kind: LibraryKind,
    state: &ControlPanelState,
    deck: &SlideDeck,
) -> Vec<LibraryEntry> {
    let id = |id: Option<i64>, index: usize| id.unwrap_or(-(index as i64 + 1));
    match kind {
        LibraryKind::Song => deck
            .library()
            .iter()
            .enumerate()
            .map(|(index, song)| LibraryEntry {
                id: id(song.id, index),
                index,
                title: song.title.clone(),
                subtitle: song.artist.clone(),
                content: song.text.clone(),
                folder: song.folder.clone(),
                tags: song.tags.clone(),
            })
            .collect(),
        LibraryKind::Media => state
            .media_items
            .iter()
            .enumerate()
            .map(|(index, item)| LibraryEntry {
                id: id(item.id, index),
                index,
                title: item.title.clone(),
                subtitle: item.file_type.clone(),
                content: item.file_path.clone(),
                folder: item.folder.clone(),
                tags: String::new(),
            })
            .collect(),
        LibraryKind::Text => state
            .custom_texts
            .iter()
            .enumerate()
            .map(|(index, item)| LibraryEntry {
                id: id(item.id, index),
                index,
                title: item.title.clone(),
                subtitle: state.settings.language.text(kind.label()).into(),
                content: item.content.clone(),
                folder: item.folder.clone(),
                tags: String::new(),
            })
            .collect(),
        LibraryKind::Verse => state
            .saved_verses
            .iter()
            .enumerate()
            .map(|(index, item)| LibraryEntry {
                id: id(item.id, index),
                index,
                title: item.reference.clone(),
                subtitle: item.version.clone(),
                content: format!("{}\n\n— {}", item.text, item.reference),
                folder: item.folder.clone(),
                tags: String::new(),
            })
            .collect(),
    }
}

pub(super) fn select_range(
    kind: LibraryKind,
    state: &mut ControlPanelState,
    id: i64,
    visible: &[i64],
    shift: bool,
    additive: bool,
) {
    let mut anchor = *kind.anchor(state);
    crate::core::interaction::select(
        kind.selected_mut(state),
        &mut anchor,
        id,
        visible,
        shift,
        additive,
    );
    *kind.anchor(state) = anchor;
}
