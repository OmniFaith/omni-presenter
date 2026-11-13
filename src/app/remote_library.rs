use super::PresentationApp;

impl PresentationApp {
    pub(super) fn remote_bible_version(&mut self, id: &str) -> Result<(), String> {
        let state = &mut self.control_state;
        let book_index = state
            .bible
            .library
            .get(&state.bible_version)
            .and_then(|b| {
                b.books
                    .iter()
                    .position(|book| book.name == state.bible_book)
            })
            .unwrap_or(0);
        let bible = state
            .bible
            .library
            .get(id)
            .ok_or("Versão bíblica não encontrada.")?;
        let book = bible
            .books
            .get(book_index)
            .or(bible.books.first())
            .ok_or("Bíblia vazia.")?;
        state.bible_book = book.name.clone();
        state.bible_chapter = state.bible_chapter.clamp(1, book.chapters.len().max(1));
        let total = book
            .chapters
            .get(state.bible_chapter - 1)
            .map_or(1, Vec::len)
            .max(1);
        state.bible_verse = state.bible_verse.clamp(1, total);
        state.bible.verse_end = state.bible.verse_end.clamp(state.bible_verse, total);
        state.bible_version = bible.id.clone();
        state.bible.preview_key.clear();
        state.bible.search_key.clear();
        state.bible.scroll_to_verse = true;
        crate::ui::pages::bible::sync_preview(state, &mut self.slide_deck, true);
        Ok(())
    }

    pub(super) fn remote_playlist_selection(
        &mut self,
        id: String,
        range: bool,
        additive: bool,
    ) -> Result<(), String> {
        let index = self
            .control_state
            .playlist
            .iter()
            .position(|i| i.id == id)
            .ok_or("Item não encontrado.")?;
        let item = self.control_state.playlist[index].clone();
        self.activate_remote_item(&item, false)?;
        let order: Vec<_> = self
            .control_state
            .playlist
            .iter()
            .map(|i| i.id.clone())
            .collect();
        self.control_state
            .playlist_selection
            .click(id, &order, range, additive);
        self.control_state.selected_playlist_index = Some(index);
        Ok(())
    }

    pub(super) fn remote_playlist_batch(
        &mut self,
        operation: &str,
        ids: &[String],
        target: Option<String>,
        after: bool,
        played: bool,
    ) -> Result<(), String> {
        let state = &mut self.control_state;
        if ids
            .iter()
            .any(|id| !state.playlist.iter().any(|i| &i.id == id))
        {
            return Err("Item não encontrado.".into());
        }
        let selected = state
            .selected_playlist_index
            .and_then(|i| state.playlist.get(i))
            .map(|i| i.id.clone());
        match operation {
            "move" => {
                let target = target.ok_or("Destino não encontrado.")?;
                if !state.playlist.iter().any(|i| i.id == target) {
                    return Err("Destino não encontrado.".into());
                }
                crate::core::interaction::move_items(
                    &mut state.playlist,
                    ids,
                    &target,
                    after,
                    |i| &i.id,
                );
            }
            "remove" => state.playlist.retain(|i| !ids.contains(&i.id)),
            "played" => {
                for item in &mut state.playlist {
                    if ids.contains(&item.id) {
                        item.played = played;
                    }
                }
            }
            "duplicate" => {
                let copies: Vec<_> = state
                    .playlist
                    .iter()
                    .filter(|i| ids.contains(&i.id))
                    .map(|i| {
                        let mut copy = i.clone();
                        copy.id = crate::core::preferences::new_id("playlist");
                        copy.played = false;
                        copy
                    })
                    .collect();
                state.playlist.extend(copies);
            }
            "select" => {
                state.playlist_selection.selected = ids.iter().cloned().collect();
            }
            "sort" => state.playlist.sort_by_key(|i| i.title.to_lowercase()),
            "reverse" => state.playlist.reverse(),
            "save" => {}
            _ => return Err("Operação inválida.".into()),
        }
        state.selected_playlist_index = state
            .playlist
            .iter()
            .position(|i| Some(&i.id) == selected.as_ref());
        let order: Vec<_> = state.playlist.iter().map(|i| i.id.clone()).collect();
        state.playlist_selection.retain(&order);
        Ok(())
    }
}
