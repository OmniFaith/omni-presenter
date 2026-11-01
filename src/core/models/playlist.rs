#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PlaylistItemKind {
    Song,
    Media,
    Bible,
    Text,
    Design,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PlaylistItem {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub kind: PlaylistItemKind,
    pub played: bool,
    #[serde(default)]
    pub content: String,
}

impl PlaylistItem {
    pub fn preview(&self, deck: &mut crate::core::slides::SlideDeck) -> Result<(), String> {
        if self.kind == PlaylistItemKind::Design {
            return deck.preview_design_by_id(&self.content);
        }
        if self.kind == PlaylistItemKind::Song {
            deck.set_is_song(true);
            if let Some(index) = deck.library().iter().position(|song| {
                song.title == self.title
                    && song.artist == self.subtitle
                    && (self.content.is_empty() || song.text == self.content)
            }) {
                deck.preview_song(index);
                return Ok(());
            }
        }
        let content = if self.kind == PlaylistItemKind::Media {
            String::new()
        } else if !self.content.is_empty() {
            self.content.clone()
        } else if self.kind == PlaylistItemKind::Bible {
            return Err(
                crate::core::i18n::Language::system()
                    .text("services-add-this-passage-again-from-the-bible-workspace-to-include")
                    .to_string(),
            );
        } else {
            self.title.clone()
        };
        deck.load_custom_content(self.title.clone(), self.subtitle.clone(), content);
        if self.kind == PlaylistItemKind::Song {
            deck.set_is_song(true);
        }
        Ok(())
    }
}
