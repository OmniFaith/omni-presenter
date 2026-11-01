use super::PresentationApp;
use crate::{
    core::{commands::Command, state::SidebarPage},
    services::search::{SearchSnapshot, SearchTarget},
};
use eframe::egui;

impl PresentationApp {
    pub(super) fn handle_search_shortcut(&mut self, ctx: &egui::Context) {
        if self
            .control_state
            .settings_ui
            .shortcut_capture
            .command
            .is_some()
        {
            return;
        }
        if self
            .control_state
            .settings
            .shortcuts
            .consume(ctx, Command::Search)
        {
            egui::DragAndDrop::clear_payload(ctx);
            if self.control_state.active_sidebar_page == SidebarPage::Bible {
                self.control_state.bible.show_search = !self.control_state.bible.show_search;
                self.control_state.bible.search_focus = self.control_state.bible.show_search;
            } else {
                self.global_search.toggle();
            }
        }
    }

    pub(super) fn draw_global_search(&mut self, ctx: &egui::Context) {
        if self.global_search.needs_search() {
            self.global_search.search(
                SearchSnapshot::capture(&self.control_state, &self.slide_deck),
                ctx,
            );
        }
        if let Some(target) = self.global_search.show(
            ctx,
            self.control_state.settings.language,
            &self.control_state.theme,
        ) {
            self.activate_search_result(ctx, target);
        }
    }

    fn activate_search_result(&mut self, ctx: &egui::Context, target: SearchTarget) {
        match target {
            SearchTarget::PluginCommand { plugin, command } => {
                self.control_state.settings_ui.plugin_request = Some(crate::core::plugins::PluginRequest::Command { plugin, command });
            }
            SearchTarget::Command(command) => self.execute_command(ctx, command),
            SearchTarget::Settings(tab) => {
                self.control_state.settings_ui.tab = tab;
                self.control_state.settings_ui.open = true;
            }
            SearchTarget::Song(index) => {
                self.control_state.active_sidebar_page = SidebarPage::Home;
                self.control_state.selected_tab = 0;
                self.slide_deck.preview_song(index);
            }
            SearchTarget::Theme(id) => {
                if let Some(index) = self
                    .control_state
                    .themes
                    .iter()
                    .position(|theme| theme.id == id)
                {
                    self.control_state.selected_theme_index = index;
                    self.control_state.active_sidebar_page = SidebarPage::Home;
                }
            }
            SearchTarget::Design(id) => {
                self.design_editor
                    .flush(&self.db, &mut self.slide_deck, true);
                if let Some(design) = self
                    .slide_deck
                    .designs
                    .iter()
                    .find(|design| design.id == id)
                    .cloned()
                {
                    self.design_editor.select(design);
                    let _ = self.slide_deck.preview_design_by_id(&id);
                    self.control_state.active_sidebar_page = SidebarPage::Design;
                }
            }
            SearchTarget::Media(index) => {
                self.control_state.active_sidebar_page = SidebarPage::Home;
                self.control_state.selected_tab = 1;
                if let Some(item) = self.control_state.media_items.get(index).cloned()
                    && let Err(error) = crate::services::media::activate(
                        &mut self.control_state,
                        &mut self.slide_deck,
                        &item.title,
                        &item.file_path,
                        "",
                        false,
                    ) {
                        self.control_state.notify(error);
                    }
            }
            SearchTarget::Text(index) => {
                self.control_state.active_sidebar_page = SidebarPage::Home;
                self.control_state.selected_tab = 2;
                if let Some(item) = self.control_state.custom_texts.get(index) {
                    self.slide_deck.load_custom_content(
                        item.title.clone(),
                        self.control_state
                            .settings
                            .language
                            .text("library-texts")
                            .into(),
                        item.content.clone(),
                    );
                }
            }
            SearchTarget::Verse(index) => {
                self.control_state.active_sidebar_page = SidebarPage::Home;
                self.control_state.selected_tab = 3;
                if let Some(item) = self.control_state.saved_verses.get(index) {
                    self.slide_deck.load_custom_content(
                        item.reference.clone(),
                        item.version.clone(),
                        format!("{}\n\n— {}", item.text, item.reference),
                    );
                }
            }
            SearchTarget::Bible {
                version,
                book,
                chapter,
                verse,
                end,
            } => {
                let state = &mut self.control_state;
                if let Some(bible) = state.bible.library.get(&version) {
                    state.active_sidebar_page = SidebarPage::Bible;
                    state.bible_version = version;
                    state.bible_book = bible.books[book].name.clone();
                    state.bible_chapter = chapter;
                    state.bible_verse = verse;
                    state.bible.verse_end = end;
                    state.bible.selection_anchor = verse;
                    state.bible.expanded_book = Some(book);
                    state.bible.scroll_to_verse = true;
                    crate::ui::pages::bible::sync_preview(state, &mut self.slide_deck, true);
                }
            }
            SearchTarget::Chat {
                channel,
                peer,
                query,
            } => {
                self.control_state.active_sidebar_page = SidebarPage::Chat;
                self.control_state.chat.channel = channel.unwrap_or_default();
                self.control_state.chat.peer = peer;
                self.control_state.chat.search = query;
                self.control_state.chat.search_open = true;
                self.control_state.chat.search_all = false;
                self.control_state.chat.last_sync = None;
            }
        }
    }
}
