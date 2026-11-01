use crate::core::bible::version_code;
use crate::core::layout::ContainerKind;
use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use eframe::egui;

mod cards;
mod helpers;
mod import;
mod navigation;
mod search;
mod selection;
mod text;
mod version_bar;

pub use import::render_import_dialog;
pub use navigation::render_bible_navigation;
pub use search::render_bible_search_overlay;
pub use selection::{selected_passage, selection_passages, sync_preview};
pub use text::render_bible_text;

#[cfg(test)]
mod tests;

pub fn render_bible_page(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    presenting: &mut bool,
    rects: &mut Vec<(ContainerKind, egui::Rect)>,
    db: Option<&AppDb>,
) {
    if state.bible_verse == state.bible.verse_end
        && deck.artist() == version_code(&state.bible_version)
        && deck.title() == format!("{} {}", state.bible_book, state.bible_chapter)
        && let Some(bible) = state.bible.library.get(&state.bible_version)
            && let Some((book, chapter, verse, _)) = deck
                .preview_slide_text()
                .lines()
                .last()
                .and_then(|reference| bible.reference(reference))
                && bible.books[book].name == state.bible_book
                    && chapter == state.bible_chapter
                    && verse != state.bible_verse
                {
                    state.bible_verse = verse;
                    state.bible.verse_end = verse;
                    state.bible.selection_anchor = verse;
                    state.bible.scroll_to_verse = true;
                }
    selection::sync_preview(state, deck, false);
    crate::ui::render_layout(ui, state, deck, presenting, rects, db);
}
