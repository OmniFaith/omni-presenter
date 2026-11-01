#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum QuickItemKind {
    Song,
    Media,
    Verse,
    Text,
    Design,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct QuickItem {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub kind: QuickItemKind,
    pub content: String,
}

impl QuickItem {
    pub fn same_resource(&self, other: &Self) -> bool {
        if self.kind != other.kind {
            return false;
        }
        match self.kind {
            QuickItemKind::Media | QuickItemKind::Design => {
                !self.content.is_empty() && self.content == other.content
            }
            QuickItemKind::Song | QuickItemKind::Verse => {
                self.title == other.title && self.subtitle == other.subtitle
            }
            QuickItemKind::Text => self.title == other.title && self.content == other.content,
        }
    }
    pub fn from_playlist(item: &super::PlaylistItem) -> Self {
        use super::PlaylistItemKind;
        Self {
            id: item.id.clone(),
            title: item.title.clone(),
            subtitle: item.subtitle.clone(),
            content: item.content.clone(),
            kind: match item.kind {
                PlaylistItemKind::Song => QuickItemKind::Song,
                PlaylistItemKind::Media => QuickItemKind::Media,
                PlaylistItemKind::Bible => QuickItemKind::Verse,
                PlaylistItemKind::Text => QuickItemKind::Text,
                PlaylistItemKind::Design => QuickItemKind::Design,
            },
        }
    }

    pub fn to_playlist(&self) -> super::PlaylistItem {
        use super::PlaylistItemKind;
        super::PlaylistItem {
            id: crate::core::preferences::new_id("playlist"),
            title: self.title.clone(),
            subtitle: self.subtitle.clone(),
            content: self.content.clone(),
            played: false,
            kind: match self.kind {
                QuickItemKind::Song => PlaylistItemKind::Song,
                QuickItemKind::Media => PlaylistItemKind::Media,
                QuickItemKind::Verse => PlaylistItemKind::Bible,
                QuickItemKind::Text => PlaylistItemKind::Text,
                QuickItemKind::Design => PlaylistItemKind::Design,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_identity_keeps_different_artists_and_files_and_survives_design_renames() {
        let mut first = QuickItem {
            id: "first".into(),
            title: "Grace".into(),
            subtitle: "Artist A".into(),
            kind: QuickItemKind::Song,
            content: "Lyrics".into(),
        };
        let mut other = first.clone();
        other.subtitle = "Artist B".into();
        assert!(!first.same_resource(&other));
        first.kind = QuickItemKind::Media;
        other = first.clone();
        other.content = "another-file.png".into();
        assert!(!first.same_resource(&other));
        first.kind = QuickItemKind::Design;
        other = first.clone();
        other.title = "Renamed design".into();
        assert!(first.same_resource(&other));
        assert_ne!(first.to_playlist().id, first.to_playlist().id);
    }
}
