use super::PresentationApp;
use crate::core::{commands::Command, state::SidebarPage};
use eframe::egui;

impl PresentationApp {
    pub(super) fn handle_global_input(
        &mut self,
        ctx: &egui::Context,
        is_text_focused: bool,
        modal_was_open: bool,
    ) {
        if egui::DragAndDrop::has_any_payload(ctx) {
            if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
                egui::DragAndDrop::clear_payload(ctx);
            }
            return;
        }
        if self.global_search.open
            || modal_was_open
            || is_text_focused
            || ctx.egui_wants_keyboard_input()
            || !self.control_state.settings.operator().keyboard_shortcuts
        {
            return;
        }
        let command = Command::ALL.into_iter().find(|command| {
            *command != Command::Search
                && command.available_on(self.control_state.active_sidebar_page)
                && self.control_state.settings.shortcuts.pressed(ctx, *command)
        });
        if let Some(command) = command {
            if command == Command::JumpSlide {
                let bindings = self.control_state.settings.shortcuts.bindings(Command::JumpSlide);
                ctx.input_mut(|input| {
                    input.events.retain(|event| {
                        match event {
                            egui::Event::Key { key, modifiers, .. } => {
                                !bindings.iter().any(|b| b.matches(*key, *modifiers))
                            }
                            egui::Event::Text(text) => {
                                !bindings.iter().any(|b| {
                                    !b.ctrl && !b.alt && (
                                        b.key == *text
                                        || (b.key == "Backslash" && text == "\\")
                                        || (b.key == "Slash" && text == "/")
                                    )
                                })
                            }
                            _ => true,
                        }
                    });
                });
            }
            self.execute_command(ctx, command);
        }
    }

    pub(super) fn execute_command(&mut self, ctx: &egui::Context, command: Command) {
        if let Some(page) = command.page() {
            if (page == SidebarPage::Bible && !self.control_state.settings.bible_enabled)
                || (page == SidebarPage::Chat && !self.control_state.settings.chat_enabled)
            {
                return;
            }
            self.control_state.active_sidebar_page = page;
            return;
        }
        match command {
            Command::Search => self.global_search.toggle(),
            Command::BibleSearch => {
                self.control_state.bible.show_search = !self.control_state.bible.show_search;
                self.control_state.bible.search_focus = self.control_state.bible.show_search;
            }
            Command::NextSlide => {
                if !self.slide_deck.next_slide() && self.is_presentation_open {
                    self.is_presentation_open = false;
                    if let Some(media) = &mut self.control_state.active_player_media {
                        if media.is_projecting {
                            media.is_projecting = false;
                            if media.is_playing {
                                media.is_playing = false;
                                media.video_paused_at = Some(media.video_position());
                                crate::services::video::stop(ctx, &media.path);
                            }
                        }
                    }
                    self.control_state.audio_player.pause();
                }
                crate::services::media::sync_live_media_playback(
                    &mut self.control_state,
                    &self.slide_deck,
                    self.is_presentation_open,
                );
            }
            Command::PreviousSlide => {
                self.slide_deck.prev_slide();
                crate::services::media::sync_live_media_playback(
                    &mut self.control_state,
                    &self.slide_deck,
                    self.is_presentation_open,
                );
            }
            Command::FirstSlide => {
                self.slide_deck.first_slide();
                crate::services::media::sync_live_media_playback(
                    &mut self.control_state,
                    &self.slide_deck,
                    self.is_presentation_open,
                );
            }
            Command::LastSlide => {
                self.slide_deck.last_slide();
                crate::services::media::sync_live_media_playback(
                    &mut self.control_state,
                    &self.slide_deck,
                    self.is_presentation_open,
                );
            }
            Command::Settings => self.control_state.settings_ui.open = true,
            Command::JumpSlide => {
                self.control_state.show_presentation_dock = true;
                self.control_state.focus_dock_jump_input = true;
            }
            Command::ToggleHide => self.slide_deck.toggle_hide_content(),
            Command::TogglePresentation => {
                self.is_presentation_open = !self.is_presentation_open;
                if self.is_presentation_open {
                    self.slide_deck.project_preview();
                    self.control_state.apply_theme_to_live();
                    if let Some(media) = &mut self.control_state.active_player_media {
                        if media.kind == crate::core::state::PlayerMediaKind::Video
                            && self.slide_deck.live_media.as_ref().map(|m| &m.path) == Some(&media.path)
                        {
                            media.is_projecting = true;
                            if media.is_playing {
                                let pos = media.video_position();
                                let _ = self.control_state.audio_player.play_video_stream(
                                    media.title.clone(),
                                    media.path.clone(),
                                    pos,
                                );
                            }
                        }
                    }
                } else {
                    if let Some(media) = &mut self.control_state.active_player_media {
                        if media.is_projecting {
                            media.is_projecting = false;
                            if media.is_playing {
                                media.is_playing = false;
                                media.video_paused_at = Some(media.video_position());
                                crate::services::video::stop(ctx, &media.path);
                            }
                        }
                    }
                    self.control_state.audio_player.pause();
                }
            }
            Command::StopPresentation => {
                self.is_presentation_open = false;
                if let Some(media) = &mut self.control_state.active_player_media {
                    if media.is_projecting {
                        media.is_projecting = false;
                        if media.is_playing {
                            media.is_playing = false;
                            media.video_paused_at = Some(media.video_position());
                            crate::services::video::stop(ctx, &media.path);
                        }
                    }
                }
                self.control_state.audio_player.pause();
            }
            Command::NewSong => {
                self.control_state.song_editor = Default::default();
                self.control_state.song_editor.open = true;
                self.control_state.song_editor.title = self
                    .control_state
                    .settings
                    .language
                    .text("library-new-song")
                    .into();
            }
            Command::WebLyricsSearch => {
                self.control_state.web_lyrics_search.open = true;
            }
            Command::ImportMedia => crate::ui::containers::selection::import_media(
                &mut self.control_state,
                Some(&self.db),
                None,
            ),
            Command::NewText => {
                self.control_state.text_editor_title.clear();
                self.control_state.text_editor_content.clear();
                self.control_state.text_editor_folder.clear();
                self.control_state.editing_text_id = None;
                self.control_state.show_create_text_modal = true;
            }
            Command::SaveVerse => {
                self.control_state.verse_editor_ref = format!(
                    "{} {}:{}",
                    self.control_state.bible_book,
                    self.control_state.bible_chapter,
                    self.control_state.bible_verse
                );
                self.control_state.verse_editor_version = self.control_state.bible_version.clone();
                self.control_state.verse_editor_text.clear();
                self.control_state.verse_editor_folder.clear();
                self.control_state.show_save_verse_modal = true;
            }
            Command::NewTheme => self.control_state.open_create_theme(),
            Command::NewDesign => {
                self.design_editor
                    .flush(&self.db, &mut self.slide_deck, true);
                let design = crate::core::design::DesignDocument::template(0);
                match self.db.save_design(&design) {
                    Ok(()) => {
                        self.slide_deck.designs.push(design.clone());
                        self.design_editor.select(design.clone());
                        let _ = self.slide_deck.preview_design_by_id(&design.id);
                        self.control_state.active_sidebar_page = SidebarPage::Design;
                    }
                    Err(error) => self.control_state.notify(error.to_string()),
                }
            }
            Command::ToggleDock => {
                self.control_state.show_presentation_dock =
                    !self.control_state.show_presentation_dock
            }
            Command::ResetLayout => self.control_state.layout = Default::default(),
            command if command.is_design_edit() => self.design_editor.execute_edit(command),
            _ => {}
        }
    }
}
