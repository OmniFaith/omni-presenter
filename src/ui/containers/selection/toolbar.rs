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
        let is_filter_active = !state.tag_filter.is_empty();
        let filter_tooltip = if is_filter_active {
            if state.tag_filter == ":untagged:" {
                format!("{}: {}", language.text("library-tags"), language.text("library-untagged"))
            } else {
                format!("{}: {}", language.text("library-tags"), state.tag_filter)
            }
        } else {
            language.text("library-tags").to_string()
        };
        let filter = theme.icon_button(ui, FUNNEL, &filter_tooltip, is_filter_active, eframe::egui::Vec2::new(28.0, 28.0));
        egui::Popup::menu(&filter).show(|ui| {
            ui.set_width(220.0);
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

            ui.horizontal(|ui| {
                ui.label(eframe::egui::RichText::new(FUNNEL).size(13.0).color(theme.accent));
                ui.label(
                    eframe::egui::RichText::new(language.text("library-filter-title"))
                        .size(12.0)
                        .strong()
                        .color(theme.text_primary),
                );
                if is_filter_active {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button(eframe::egui::RichText::new(format!("{X} {}", language.text("library-clear-filter"))).size(9.5))
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            state.tag_filter.clear();
                            ui.close();
                        }
                    });
                }
            });
            ui.separator();

            if state.selected_tab == 1 {
                let total_media = state.media_items.len();
                let videos_count = state.media_items.iter().filter(|m| m.file_type == "video").count();
                let images_count = state.media_items.iter().filter(|m| m.file_type == "image").count();
                let audios_count = state.media_items.iter().filter(|m| m.file_type == "audio").count();

                for (type_id, label_key, icon, count) in [
                    ("", "library-all-media", VIDEO, total_media),
                    ("video", "library-media-videos", VIDEO, videos_count),
                    ("image", "library-media-images", IMAGE, images_count),
                    ("audio", "library-media-audios", MUSIC_NOTE, audios_count),
                ] {
                    let is_selected = if type_id.is_empty() {
                        state.tag_filter.is_empty()
                    } else {
                        state.tag_filter.eq_ignore_ascii_case(type_id)
                    };
                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 26.0), egui::Sense::click());
                    if is_selected {
                        ui.painter().rect_filled(rect, 4.0, theme.accent_soft);
                    } else if resp.hovered() {
                        ui.painter().rect_filled(rect, 4.0, theme.bg_card_hover);
                    }
                    let icon_color = if is_selected { theme.accent } else { theme.icon_muted };
                    let text_color = if is_selected { theme.text_primary } else { theme.text_secondary };
                    ui.painter().text(rect.left_center() + egui::vec2(8.0, 0.0), egui::Align2::LEFT_CENTER, icon, egui::FontId::proportional(12.0), icon_color);
                    ui.painter().text(rect.left_center() + egui::vec2(26.0, 0.0), egui::Align2::LEFT_CENTER, language.text(label_key), egui::FontId::proportional(11.0), text_color);
                    ui.painter().text(rect.right_center() - egui::vec2(8.0, 0.0), egui::Align2::RIGHT_CENTER, format!("({count})"), egui::FontId::proportional(10.0), theme.text_muted);
                    if resp.clicked() {
                        state.tag_filter = type_id.into();
                        ui.close();
                    }
                }
            } else {
                let total_songs = deck.library().len();
                let untagged_count = deck.library().iter().filter(|s| s.tags.trim().is_empty()).count();

                let mut tag_counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
                for song in deck.library() {
                    for tag in song.tags.split(',') {
                        let trimmed = tag.trim();
                        if !trimmed.is_empty() {
                            *tag_counts.entry(trimmed.to_string()).or_insert(0) += 1;
                        }
                    }
                }

                let all_selected = state.tag_filter.is_empty();
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 26.0), egui::Sense::click());
                if all_selected {
                    ui.painter().rect_filled(rect, 4.0, theme.accent_soft);
                } else if resp.hovered() {
                    ui.painter().rect_filled(rect, 4.0, theme.bg_card_hover);
                }
                ui.painter().text(rect.left_center() + egui::vec2(8.0, 0.0), egui::Align2::LEFT_CENTER, MUSIC_NOTES, egui::FontId::proportional(12.0), if all_selected { theme.accent } else { theme.icon_muted });
                ui.painter().text(rect.left_center() + egui::vec2(26.0, 0.0), egui::Align2::LEFT_CENTER, language.text("library-all-tags"), egui::FontId::proportional(11.0), if all_selected { theme.text_primary } else { theme.text_secondary });
                ui.painter().text(rect.right_center() - egui::vec2(8.0, 0.0), egui::Align2::RIGHT_CENTER, format!("({total_songs})"), egui::FontId::proportional(10.0), theme.text_muted);
                if resp.clicked() {
                    state.tag_filter.clear();
                    ui.close();
                }

                if untagged_count > 0 {
                    let untagged_selected = state.tag_filter == ":untagged:";
                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 26.0), egui::Sense::click());
                    if untagged_selected {
                        ui.painter().rect_filled(rect, 4.0, theme.accent_soft);
                    } else if resp.hovered() {
                        ui.painter().rect_filled(rect, 4.0, theme.bg_card_hover);
                    }
                    ui.painter().text(rect.left_center() + egui::vec2(8.0, 0.0), egui::Align2::LEFT_CENTER, TAG, egui::FontId::proportional(12.0), if untagged_selected { theme.accent } else { theme.icon_muted });
                    ui.painter().text(rect.left_center() + egui::vec2(26.0, 0.0), egui::Align2::LEFT_CENTER, language.text("library-untagged"), egui::FontId::proportional(11.0), if untagged_selected { theme.text_primary } else { theme.text_secondary });
                    ui.painter().text(rect.right_center() - egui::vec2(8.0, 0.0), egui::Align2::RIGHT_CENTER, format!("({untagged_count})"), egui::FontId::proportional(10.0), theme.text_muted);
                    if resp.clicked() {
                        state.tag_filter = ":untagged:".into();
                        ui.close();
                    }
                }

                if tag_counts.is_empty() {
                    ui.add_space(4.0);
                    egui::Frame::new()
                        .fill(theme.bg_card_hover)
                        .corner_radius(6.0)
                        .inner_margin(egui::Margin::symmetric(10, 8))
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(eframe::egui::RichText::new(TAG).size(20.0).color(theme.icon_muted));
                                ui.add_space(2.0);
                                ui.label(
                                    eframe::egui::RichText::new(language.text("library-no-tags"))
                                        .size(11.0)
                                        .strong()
                                        .color(theme.text_primary),
                                );
                                ui.add_space(2.0);
                                ui.label(
                                    eframe::egui::RichText::new(language.text("library-no-tags-tip"))
                                        .size(9.0)
                                        .color(theme.text_muted),
                                );
                            });
                        });
                } else {
                    ui.separator();
                    let search_id = ui.make_persistent_id("library_tag_filter_search");
                    let mut tag_search = ui.ctx().data_mut(|d| d.get_temp::<String>(search_id)).unwrap_or_default();
                    if tag_counts.len() >= 6 {
                        ui.horizontal(|ui| {
                            ui.label(eframe::egui::RichText::new(MAGNIFYING_GLASS).size(11.0).color(theme.icon_muted));
                            ui.add(
                                egui::TextEdit::singleline(&mut tag_search)
                                    .hint_text(language.text("library-search-tags"))
                                    .desired_width(ui.available_width() - 20.0)
                                    .margin(egui::vec2(4.0, 3.0)),
                            );
                            if !tag_search.is_empty() && ui.small_button(X).clicked() {
                                tag_search.clear();
                            }
                        });
                    }
                    ui.ctx().data_mut(|d| d.insert_temp(search_id, tag_search.clone()));

                    egui::ScrollArea::vertical()
                        .max_height(180.0)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            let q = tag_search.trim().to_lowercase();
                            for (tag, count) in &tag_counts {
                                if !q.is_empty() && !tag.to_lowercase().contains(&q) {
                                    continue;
                                }
                                let is_selected = state.tag_filter == *tag;
                                let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 26.0), egui::Sense::click());
                                if is_selected {
                                    ui.painter().rect_filled(rect, 4.0, theme.accent_soft);
                                } else if resp.hovered() {
                                    ui.painter().rect_filled(rect, 4.0, theme.bg_card_hover);
                                }
                                ui.painter().text(rect.left_center() + egui::vec2(8.0, 0.0), egui::Align2::LEFT_CENTER, TAG, egui::FontId::proportional(11.0), if is_selected { theme.accent } else { theme.icon_muted });
                                ui.painter().text(rect.left_center() + egui::vec2(24.0, 0.0), egui::Align2::LEFT_CENTER, tag, egui::FontId::proportional(11.0), if is_selected { theme.text_primary } else { theme.text_secondary });
                                ui.painter().text(rect.right_center() - egui::vec2(8.0, 0.0), egui::Align2::RIGHT_CENTER, format!("({count})"), egui::FontId::proportional(10.0), theme.text_muted);
                                if resp.clicked() {
                                    state.tag_filter = tag.clone();
                                    ui.close();
                                }
                            }
                        });
                }
            }
        });
    });
    let tabs = [
        (MUSIC_NOTES, language.text("library-songs")),
        (VIDEO, language.text("library-media")),
        (FILE_TEXT, language.text("library-texts")),
        (BOOK_OPEN, language.text("library-verses")),
        (ICON_DESIGN, language.text("library-designs")),
    ];
    if let Some(tab) = render_pill_tabs(ui, state.selected_tab, &tabs, &theme) {
        if tab != state.selected_tab {
            state.clear_all_selections();
            state.tag_filter.clear();
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
