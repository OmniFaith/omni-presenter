use super::{ResourceKind, WebControl};
use crate::core::{
    bible::fold,
    models::{PlaylistItem, PlaylistItemKind},
    search::SearchQuery,
};
use serde_json::{Value, json};

impl WebControl {
    pub fn resources(&self, kind: ResourceKind) -> Result<Vec<(PlaylistItem, String)>, String> {
        let rows = match kind {
            ResourceKind::Song => self
                .db
                .get_all_songs()
                .map_err(|e| e.to_string())?
                .into_iter()
                .filter_map(|song| {
                    Some((
                        PlaylistItem {
                            id: song.id?.to_string(),
                            title: song.title,
                            subtitle: song.artist,
                            content: song.text,
                            kind: PlaylistItemKind::Song,
                            played: false,
                        },
                        song.folder,
                    ))
                })
                .collect(),
            ResourceKind::Text => self
                .db
                .get_all_custom_texts()
                .map_err(|e| e.to_string())?
                .into_iter()
                .filter_map(|text| {
                    Some((
                        PlaylistItem {
                            id: text.id?.to_string(),
                            title: text.title,
                            subtitle: "Texto".into(),
                            content: text.content,
                            kind: PlaylistItemKind::Text,
                            played: false,
                        },
                        text.folder,
                    ))
                })
                .collect(),
            ResourceKind::Verse => self
                .db
                .get_all_saved_verses()
                .map_err(|e| e.to_string())?
                .into_iter()
                .filter_map(|verse| {
                    Some((
                        PlaylistItem {
                            id: verse.id?.to_string(),
                            title: verse.reference,
                            subtitle: verse.version,
                            content: verse.text,
                            kind: PlaylistItemKind::Bible,
                            played: false,
                        },
                        verse.folder,
                    ))
                })
                .collect(),
            ResourceKind::Media => self
                .db
                .get_all_media_items()
                .map_err(|e| e.to_string())?
                .into_iter()
                .filter_map(|media| {
                    Some((
                        PlaylistItem {
                            id: media.id?.to_string(),
                            title: media.title,
                            subtitle: media.file_type,
                            content: media.file_path,
                            kind: PlaylistItemKind::Media,
                            played: false,
                        },
                        media.folder,
                    ))
                })
                .collect(),
            ResourceKind::Design => self
                .db
                .load_designs()
                .map_err(|e| e.to_string())?
                .into_iter()
                .map(|design| {
                    (
                        PlaylistItem {
                            id: design.id.clone(),
                            title: design.name,
                            subtitle: if design.overlay { "Extra" } else { "Design" }.into(),
                            content: design.id,
                            kind: PlaylistItemKind::Design,
                            played: false,
                        },
                        design.folder,
                    )
                })
                .collect(),
        };
        Ok(rows)
    }

    pub fn resource(&self, kind: ResourceKind, id: &str) -> Result<PlaylistItem, String> {
        self.resources(kind)?
            .into_iter()
            .map(|(item, _)| item)
            .find(|item| item.id == id)
            .ok_or_else(|| crate::core::i18n::Language::system().text("services-this-content-is-no-longer-in-the-library").to_string())
    }

    pub fn catalog(
        &self,
        kind: ResourceKind,
        query: &str,
        folder: Option<&str>,
    ) -> Result<Value, String> {
        let query = SearchQuery::new(query);
        let resources = self.resources(kind)?;
        let mut folders: Vec<_> = resources
            .iter()
            .map(|(_, folder)| folder.clone())
            .filter(|folder| !folder.is_empty())
            .collect();
        folders.sort();
        folders.dedup();
        let items: Vec<_> = resources.into_iter().filter(|(item, path)| {
            folder.is_none_or(|folder| folder == path) && query.matches(&fold(&format!("{} {} {}", item.title, item.subtitle, if kind == ResourceKind::Media { "" } else { &item.content })))
        }).map(|(item, folder)| json!({"id":item.id,"kind":kind,"title":item.title,"subtitle":item.subtitle,"folder":folder})).collect();
        Ok(json!({"items":items,"folders":folders}))
    }
}
