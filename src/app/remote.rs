use super::PresentationApp;
use crate::{
    core::models::{PlaylistItem, PlaylistItemKind, QuickItemKind},
    services::control::ControlCommand,
};
use serde_json::{Value, json};

impl PresentationApp {
    pub(super) fn sync_web_control(&mut self, ctx: &eframe::egui::Context) {
        let wake = ctx.clone();
        self.web_control.set_waker(move || wake.request_repaint());
        for pending in self.web_control.drain() {
            if pending.issued.elapsed().as_secs() >= 15 {
                continue;
            }
            let result = self.apply_web_command(pending.command);
            let _ = pending.reply.send(result);
        }
        if self.browser_server.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }

    pub(super) fn apply_web_command(&mut self, command: ControlCommand) -> Result<Value, String> {
        use ControlCommand::*;
        let persist = matches!(
            &command,
            PlaylistAdd { .. }
                | PlaylistSelect { .. }
                | PlaylistMove { .. }
                | PlaylistRemove { .. }
                | PlaylistPlayed { .. }
                | PlaylistClear
                | Pin { .. }
                | QuickRemove { .. }
                | Service { .. }
                | Bible { playlist: true, .. }
                | Text { playlist: true, .. }
                | DockVisibility { .. }
                | DockPreviewMode { .. }
        );
        match command {
            DockVisibility { visible } => self.control_state.show_presentation_dock = visible,
            DockPreviewMode { background } => self.control_state.dock_preview_bg = background,
            WorkspaceReset { bible } => {
                if bible {
                    self.control_state.bible_layout =
                        crate::core::layout::LayoutTree::bible_workspace();
                } else {
                    self.control_state.layout = Default::default();
                }
                self.control_state.capture_profiles();
                self.db
                    .save_settings(&self.control_state.settings)
                    .map_err(|error| error.to_string())?;
            }
            WorkspaceLayout { bible, layout } => {
                fn collect(
                    node: &crate::core::layout::LayoutNode,
                    leaves: &mut Vec<crate::core::layout::ContainerKind>,
                ) -> bool {
                    match node {
                        crate::core::layout::LayoutNode::Leaf(kind) => {
                            leaves.push(*kind);
                            true
                        }
                        crate::core::layout::LayoutNode::Split {
                            fraction,
                            first,
                            second,
                            ..
                        } => {
                            fraction.is_finite()
                                && (0.05..=0.95).contains(fraction)
                                && collect(first, leaves)
                                && collect(second, leaves)
                        }
                    }
                }
                let mut leaves = Vec::new();
                if !collect(&layout.root, &mut leaves) || leaves.len() != 7 {
                    return Err("Layout inválido.".into());
                }
                use crate::core::layout::ContainerKind::*;
                if [Selection, Lyrics, Stage, Preview, Themes, Playlist, Player]
                    .iter()
                    .any(|kind| leaves.iter().filter(|leaf| *leaf == kind).count() != 1)
                {
                    return Err("Layout inválido.".into());
                }
                if bible {
                    self.control_state.bible_layout = layout;
                } else {
                    self.control_state.layout = layout;
                }
                self.control_state.capture_profiles();
                self.db
                    .save_settings(&self.control_state.settings)
                    .map_err(|error| error.to_string())?;
            }
            PinPreview => {
                let deck = &self.slide_deck;
                let (kind, content) = if let Some(scene) = &deck.preview_design {
                    (QuickItemKind::Design, scene.id.clone())
                } else if self
                    .control_state
                    .bible
                    .library
                    .catalog()
                    .iter()
                    .any(|(id, _)| id == deck.artist())
                {
                    (QuickItemKind::Verse, deck.raw_text().into())
                } else {
                    (
                        if deck.current_song().is_some_and(|song| {
                            song.title == deck.title() && song.artist == deck.artist()
                        }) {
                            QuickItemKind::Song
                        } else {
                            QuickItemKind::Text
                        },
                        deck.raw_text().into(),
                    )
                };
                self.control_state.add_quick_item(
                    deck.title().into(),
                    deck.artist().into(),
                    kind,
                    content,
                );
                self.control_state.capture_profiles();
                self.db
                    .save_settings(&self.control_state.settings)
                    .map_err(|error| error.to_string())?;
            }
            Select { kind, id, project } => {
                let item = self.web_control.resource(kind, &id)?;
                if kind == crate::services::control::ResourceKind::Song {
                    self.slide_deck.reload_from_db(&self.db);
                    let id = id.parse::<i64>().map_err(|_| "Música não encontrada.")?;
                    let index = self
                        .slide_deck
                        .library()
                        .iter()
                        .position(|song| song.id == Some(id))
                        .ok_or("Música não encontrada.")?;
                    self.control_state.selected_song_ids.clear();
                    self.control_state.selected_song_ids.insert(id);
                    self.control_state.song_selection_anchor = Some(id);
                    self.slide_deck.select_song(index);
                    if project {
                        self.remote_project_preview()?;
                    }
                } else {
                    self.activate_remote_item(&item, project)?;
                }
            }
            ProjectPreview { index } => {
                if let Some(index) = index {
                    if index >= self.slide_deck.slides().len() {
                        return Err("Estrofe não encontrada.".into());
                    }
                    self.slide_deck.select_preview_slide(index);
                }
                self.remote_project_preview()?;
            }
            PreviewSlide { index } => {
                if index >= self.slide_deck.slides().len() {
                    return Err("Estrofe não encontrada.".into());
                }
                self.slide_deck.select_preview_slide(index);
            }
            LiveSlide { index } => {
                if index >= self.slide_deck.live_slides().len() {
                    return Err("Estrofe não encontrada.".into());
                }
                self.slide_deck.select_live_slide(index);
            }
            Navigate { direction } => match direction.as_str() {
                "next" => {
                    if !self.slide_deck.next_slide() && self.is_presentation_open {
                        self.is_presentation_open = false;
                    }
                }
                "previous" => self.slide_deck.prev_slide(),
                "first" => self.slide_deck.first_slide(),
                "last" => self.slide_deck.last_slide(),
                _ => return Err("Direção inválida.".into()),
            },
            Visibility { hidden } => {
                if self.slide_deck.is_content_hidden() != hidden {
                    self.slide_deck.toggle_hide_content();
                }
            }
            Stop => {
                self.is_presentation_open = false;
                self.control_state.stop_player();
            }
            Theme { id, live } => {
                self.control_state.selected_theme_index = self
                    .control_state
                    .themes
                    .iter()
                    .position(|theme| theme.id == id)
                    .ok_or("Tema não encontrado.")?;
                if live {
                    self.control_state.apply_theme_to_live();
                }
            }
            PlaylistAdd { kind, id } => {
                let mut item = self.web_control.resource(kind, &id)?;
                item.id = crate::core::preferences::new_id("remote_playlist");
                self.control_state.playlist.push(item);
            }
            PlaylistSelect { id, project } => {
                let index = self
                    .control_state
                    .playlist
                    .iter()
                    .position(|item| item.id == id)
                    .ok_or("Item não encontrado no repertório.")?;
                let item = self.control_state.playlist[index].clone();
                self.activate_remote_item(&item, project)?;
                self.control_state.selected_playlist_index = Some(index);
                if project {
                    self.control_state.playlist[index].played = true;
                }
            }
            PlaylistMove { id, target, after } => {
                if !self.control_state.playlist.iter().any(|item| item.id == id)
                    || !self
                        .control_state
                        .playlist
                        .iter()
                        .any(|item| item.id == target)
                {
                    return Err("Item não encontrado no repertório.".into());
                }
                let selected = self
                    .control_state
                    .selected_playlist_index
                    .and_then(|index| self.control_state.playlist.get(index))
                    .map(|item| item.id.clone());
                crate::core::interaction::move_items(
                    &mut self.control_state.playlist,
                    &[id],
                    &target,
                    after,
                    |item| &item.id,
                );
                self.control_state.selected_playlist_index = self
                    .control_state
                    .playlist
                    .iter()
                    .position(|item| Some(&item.id) == selected.as_ref());
            }
            PlaylistRemove { id } => {
                let selected = self
                    .control_state
                    .selected_playlist_index
                    .and_then(|index| self.control_state.playlist.get(index))
                    .map(|item| item.id.clone());
                self.control_state.playlist.retain(|item| item.id != id);
                self.control_state.selected_playlist_index = self
                    .control_state
                    .playlist
                    .iter()
                    .position(|item| Some(&item.id) == selected.as_ref());
            }
            PlaylistPlayed { id, played } => {
                self.control_state
                    .playlist
                    .iter_mut()
                    .find(|item| item.id == id)
                    .ok_or("Item não encontrado.")?
                    .played = played;
            }
            PlaylistClear => {
                self.control_state.playlist.clear();
                self.control_state.selected_playlist_index = None;
            }
            Pin { kind, id } => {
                let item = self.web_control.resource(kind, &id)?;
                let kind = match item.kind {
                    PlaylistItemKind::Song => QuickItemKind::Song,
                    PlaylistItemKind::Media => QuickItemKind::Media,
                    PlaylistItemKind::Bible => QuickItemKind::Verse,
                    PlaylistItemKind::Text => QuickItemKind::Text,
                    PlaylistItemKind::Design => QuickItemKind::Design,
                };
                self.control_state
                    .add_quick_item(item.title, item.subtitle, kind, item.content);
            }
            QuickSelect { id, project } => {
                let item = self
                    .control_state
                    .quick_bar
                    .items
                    .iter()
                    .find(|item| item.id == id)
                    .ok_or("Atalho não encontrado.")?
                    .to_playlist();
                self.activate_remote_item(&item, project)?;
                self.control_state.quick_bar.active_id = Some(id);
            }
            QuickRemove { id } => {
                self.control_state
                    .quick_bar
                    .items
                    .retain(|item| item.id != id);
            }
            Bible {
                reference,
                version,
                merge,
                project,
                playlist,
            } => {
                let bible = self
                    .control_state
                    .bible
                    .library
                    .get(&version)
                    .ok_or("Versão bíblica não encontrada.")?;
                let (book, chapter, start, end) = bible
                    .reference(&reference)
                    .ok_or("Referência bíblica inválida.")?;
                let passage = bible
                    .passage(book, chapter, start, end)
                    .ok_or("Passagem indisponível.")?;
                if !playlist || project {
                    self.control_state.bible_book = bible.books[book].name.clone();
                    self.control_state.bible_chapter = chapter;
                    self.control_state.bible_verse = start;
                    self.control_state.bible_version = version.clone();
                    self.control_state.bible.verse_end = end;
                    self.control_state.bible.selection_anchor = start;
                    self.control_state.bible.merge_selection = merge;
                    self.control_state.bible.expanded_book = Some(book);
                    self.control_state.bible.preview_key.clear();
                }
                let content = if merge {
                    passage.content
                } else {
                    (start..=end)
                        .filter_map(|verse| bible.passage(book, chapter, verse, verse))
                        .map(|passage| passage.content)
                        .collect::<Vec<_>>()
                        .join("\n\n")
                };
                if playlist {
                    self.remote_content(
                        passage.reference,
                        version,
                        content,
                        PlaylistItemKind::Bible,
                        false,
                        true,
                    )?;
                }
                if !playlist || project {
                    crate::ui::pages::bible::sync_preview(
                        &mut self.control_state,
                        &mut self.slide_deck,
                        true,
                    );
                    if project {
                        self.remote_project_preview()?;
                    }
                }
            }
            Text {
                title,
                content,
                project,
                playlist,
            } => {
                if content.trim().is_empty() {
                    return Err("Digite o texto do aviso.".into());
                }
                self.remote_content(
                    if title.trim().is_empty() {
                        "Aviso".into()
                    } else {
                        title
                    },
                    "Texto".into(),
                    content,
                    PlaylistItemKind::Text,
                    project,
                    playlist,
                )?;
            }
            SaveSong {
                id,
                title,
                artist,
                content,
            } => {
                if title.trim().is_empty() || content.trim().is_empty() {
                    return Err("Preencha o título e a letra.".into());
                }
                let id = if let Some(id) = id {
                    if !self
                        .slide_deck
                        .library()
                        .iter()
                        .any(|song| song.id == Some(id))
                    {
                        return Err("Música não encontrada.".into());
                    }
                    self.db
                        .update_song(id, title.trim(), artist.trim(), &content)
                        .map_err(|e| e.to_string())?;
                    id
                } else {
                    self.db
                        .add_song(title.trim(), artist.trim(), &content)
                        .map_err(|e| e.to_string())?
                };
                self.slide_deck.reload_from_db(&self.db);
                if let Some(index) = self
                    .slide_deck
                    .library()
                    .iter()
                    .position(|song| song.id == Some(id))
                {
                    self.slide_deck.select_song(index);
                }
            }
            OverlayClear => self.slide_deck.live_overlay = None,
            Timer { operation } => self.remote_design_control(&operation)?,
            Player { operation, value } => self.remote_player_control(&operation, value)?,
            Service { id } => {
                if !self
                    .control_state
                    .settings
                    .services
                    .iter()
                    .any(|service| service.id == id)
                {
                    return Err("Culto não encontrado.".into());
                }
                self.control_state.switch_service(&id);
            }
            ServiceDate { date } => {
                self.control_state.switch_service_date(&date);
            }
            ServiceStepDate { delta } => {
                self.control_state.step_service_date(delta);
            }
        }
        if persist {
            self.control_state.capture_profiles();
            self.db
                .save_settings(&self.control_state.settings)
                .map_err(|e| e.to_string())?;
        }
        self.last_control_publish = None;
        self.publish_web_control();
        Ok(json!({"ok":true,"state":self.web_control.snapshot()}))
    }

    fn remote_content(
        &mut self,
        title: String,
        subtitle: String,
        content: String,
        kind: PlaylistItemKind,
        project: bool,
        playlist: bool,
    ) -> Result<(), String> {
        let item = PlaylistItem {
            id: crate::core::preferences::new_id("remote"),
            title,
            subtitle,
            content,
            kind,
            played: false,
        };
        if playlist {
            self.control_state.playlist.push(item.clone());
        }
        if !playlist || project {
            self.activate_remote_item(&item, project)?;
        }
        Ok(())
    }

    pub(super) fn activate_remote_item(
        &mut self,
        item: &PlaylistItem,
        project: bool,
    ) -> Result<(), String> {
        if item.kind == PlaylistItemKind::Media {
            if crate::services::media::activate(
                &mut self.control_state,
                &mut self.slide_deck,
                &item.title,
                &item.content,
                &item.subtitle,
                project,
            )? {
                self.is_presentation_open = true;
            }
        } else {
            if item.kind == PlaylistItemKind::Design {
                self.design_editor
                    .flush(&self.db, &mut self.slide_deck, true);
                self.slide_deck.designs = self.db.load_designs().map_err(|e| e.to_string())?;
            }
            item.preview(&mut self.slide_deck)?;
            if project {
                self.remote_project_preview()?;
            }
        }
        Ok(())
    }

    fn remote_project_preview(&mut self) -> Result<(), String> {
        if self.slide_deck.slides().is_empty() && self.slide_deck.preview_design.is_none() {
            return Err("Selecione um conteúdo para apresentar.".into());
        }
        self.slide_deck.project_preview();
        if !self
            .slide_deck
            .preview_design
            .as_ref()
            .is_some_and(|scene| scene.overlay)
        {
            self.control_state.apply_theme_to_live();
        }
        self.is_presentation_open = true;
        self.control_state.show_presentation_dock =
            self.control_state.settings.operator().show_dock;
        Ok(())
    }
}
