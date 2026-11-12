use super::{actions, model::LibraryKind};
use crate::{
    core::{
        slides::SlideDeck,
        state::{ControlPanelState, SidebarPage},
    },
    services::db::AppDb,
    ui::components::*,
};
use eframe::egui;
use egui_phosphor::regular::*;

pub(super) fn toolbar(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &SlideDeck,
    db: Option<&AppDb>,
) {
    let theme = state.theme.clone();
    let language = state.settings.language;
    render_header_bar(
        ui,
        crate::core::layout::ContainerKind::Selection,
        language.text("library-title"),
        &theme,
    );
    centered_horizontal(ui, "selection_toolbar", 160.0, |ui| {
        let kind = LibraryKind::from_tab(state.selected_tab);
        if toolbar_icon_button(ui, PLUS_CIRCLE, language.text(kind.create_label()), &theme)
            .clicked()
        {
            if state.selected_tab == 4 {
                state.active_sidebar_page = SidebarPage::Design;
            } else {
                actions::create(kind, None, state, db);
            }
        }
        if toolbar_icon_button(ui, MAGNIFYING_GLASS, language.text("common-search"), &theme)
            .clicked()
        {
            state.show_search = !state.show_search;
        }
        let is_media = state.selected_tab == 1;
        if state.selected_tab == 0 && toolbar_icon_button(ui, DOWNLOAD_SIMPLE, language.text("transfer-lyrics-choose"), &theme).clicked() {
            state.settings_ui.open = true;
            state.settings_ui.tab = crate::core::state::SettingsTab::Transfer;
            crate::ui::modals::settings::transfer::start_lyrics(ui.ctx());
        }
        let link_label = if is_media {
            language.text("library-add-web-video")
        } else {
            language.text("library-web-search")
        };
        if toolbar_icon_button(ui, GLOBE, link_label, &theme).clicked() {
            if is_media {
                state.web_media_modal.open = true;
            } else {
                state.web_lyrics_search.open = true;
            }
        }
        if toolbar_icon_button(ui, FOLDER_PLUS, language.text("common-new-folder"), &theme)
            .clicked()
        {
            state.show_create_folder_modal = true;
        }
        let filter = toolbar_icon_button(ui, FUNNEL, language.text("library-tags"), &theme);
        egui::Popup::menu(&filter).show(|ui| {
            let mut tags: Vec<_> = deck
                .library()
                .iter()
                .flat_map(|song| song.tags.split(','))
                .map(str::trim)
                .filter(|tag| !tag.is_empty())
                .collect();
            tags.sort_unstable();
            tags.dedup();
            if ui
                .selectable_label(
                    state.tag_filter.is_empty(),
                    language.text("library-all-tags"),
                )
                .clicked()
            {
                state.tag_filter.clear();
                ui.close();
            }
            if tags.is_empty() {
                ui.label(language.text("library-no-tags"));
            }
            for tag in tags {
                if ui.selectable_label(state.tag_filter == tag, tag).clicked() {
                    state.tag_filter = tag.into();
                    ui.close();
                }
            }
        });
    });
    let tabs = [
        (MUSIC_NOTES, language.text("library-songs")),
        (VIDEO, language.text("library-media")),
        (FILE_TEXT, language.text("library-texts")),
        (BOOK_OPEN, language.text("library-verses")),
        (STACK, language.text("library-designs")),
    ];
    if let Some(tab) = render_pill_tabs(ui, state.selected_tab, &tabs, &theme) {
        if tab != state.selected_tab {
            state.clear_all_selections();
        }
        state.selected_tab = tab;
    }
    if state.show_search || !state.search_query.is_empty() {
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut state.search_query)
                    .hint_text(language.text("common-search"))
                    .desired_width((ui.available_width() - 32.0).max(40.0)),
            );
            if ui
                .button(X)
                .on_hover_text(language.text("common-close"))
                .clicked()
            {
                state.search_query.clear();
                state.show_search = false;
            }
        });
    }
}
