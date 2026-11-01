use crate::core::bible::version_code;
use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use eframe::egui::{self, RichText, Stroke};
use egui_phosphor::regular::PLUS;

use super::helpers::{capitalize, grid_metrics};
use super::import::open_bible_settings;
use super::search::prepare_search;
use super::selection::{select_reference, sync_preview};

fn card_frame(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    selected: bool,
    active_fill: egui::Color32,
    theme: &crate::ui::theme::AppTheme,
) -> (egui::Rect, egui::Response, f32) {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let hover = ui
        .ctx()
        .animate_bool(response.id.with("hover"), response.hovered());
    let down = ui.ctx().animate_bool(
        response.id.with("down"),
        response.is_pointer_button_down_on(),
    );
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let fill = if selected {
        active_fill
    } else if down > 0.05 {
        theme.bg_card_hover.lerp_to_gamma(active_fill, down * 0.45)
    } else {
        theme.bg_card.lerp_to_gamma(theme.bg_card_hover, hover)
    };
    let stroke = if selected {
        Stroke::new(1.5, theme.accent)
    } else if down > 0.05 {
        Stroke::new(1.3, theme.accent.linear_multiply(0.85))
    } else if hover > 0.05 {
        Stroke::new(1.2, theme.accent.linear_multiply(0.55))
    } else {
        Stroke::new(1.0, theme.border_default)
    };
    ui.painter()
        .rect(rect, 9.0, fill, stroke, egui::StrokeKind::Inside);
    (rect, response, hover)
}

fn book_card(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    abbrev: &str,
    name: &str,
    selected: bool,
    theme: &crate::ui::theme::AppTheme,
) -> egui::Response {
    let (rect, response, _) = card_frame(ui, size, selected, theme.accent_soft, theme);
    let color = if selected {
        theme.accent
    } else {
        theme.text_primary
    };
    ui.painter().text(
        egui::pos2(rect.center().x, rect.top() + 11.0),
        egui::Align2::CENTER_TOP,
        abbrev,
        egui::FontId::proportional(12.0),
        color,
    );
    let painter = ui.painter().with_clip_rect(rect.shrink(5.0));
    let galley = painter.layout(
        name.to_owned(),
        egui::FontId::proportional(9.0),
        theme.text_muted,
        rect.width() - 10.0,
    );
    painter.galley(
        egui::pos2(rect.center().x - galley.size().x / 2.0, rect.top() + 28.0),
        galley,
        theme.text_muted,
    );
    response
}

fn number_card(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    label: &str,
    active: bool,
    theme: &crate::ui::theme::AppTheme,
) -> egui::Response {
    let (rect, response, hover) = card_frame(ui, size, active, theme.accent_soft, theme);
    let is_down = response.is_pointer_button_down_on();
    let text_color = if active {
        theme.accent
    } else if is_down || hover > 0.5 {
        theme.text_primary
    } else {
        theme.text_secondary
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        if active || is_down {
            egui::FontId::proportional(13.0)
        } else {
            egui::FontId::proportional(12.5)
        },
        text_color,
    );
    response
}

fn section_panel(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    title: &str,
    theme: &crate::ui::theme::AppTheme,
    body: impl FnOnce(&mut egui::Ui),
) {
    let margin = 8.0;
    egui::Frame::new()
        .fill(theme.bg_panel)
        .stroke(Stroke::new(1.0, theme.border_default))
        .corner_radius(10.0)
        .inner_margin(egui::Margin::same(margin as i8))
        .show(ui, |ui| {
            ui.set_min_size(egui::vec2(
                (size.x - margin * 2.0).max(40.0),
                (size.y - margin * 2.0).max(40.0),
            ));
            ui.set_max_size(egui::vec2(
                (size.x - margin * 2.0).max(40.0),
                (size.y - margin * 2.0).max(40.0),
            ));
            ui.label(
                RichText::new(crate::ui::i18n::text(ui, title))
                    .size(11.5)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(5.0);
            body(ui);
        });
}

pub(super) fn cards_body(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    theme: &crate::ui::theme::AppTheme,
) {
    let info = state.bible.library.get(&state.bible_version).map(|bible| {
        let book = bible
            .books
            .iter()
            .position(|b| b.name == state.bible_book)
            .unwrap_or(0);
        let chapter = state
            .bible_chapter
            .clamp(1, bible.books[book].chapters.len().max(1));
        (
            bible
                .books
                .iter()
                .map(|b| b.name.clone())
                .collect::<Vec<_>>(),
            bible
                .books
                .iter()
                .map(|b| b.abbrev.clone())
                .collect::<Vec<_>>(),
            book,
            chapter,
            bible.books[book].chapters.len(),
            bible
                .chapter(book, chapter)
                .map(|verses| verses.len())
                .unwrap_or(0),
        )
    });
    let Some((names, abbrevs, book, chapter, chapter_count, verse_count)) = info else {
        ui.add_space(30.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(crate::ui::i18n::format(ui, "workspace-add-version", &[("version", &version_code(&state.bible_version))])).size(16.0).strong());
            ui.add_space(8.0);
            ui.label(RichText::new(crate::ui::i18n::text(ui, "workspace-import-this-translation-s-json-file-to-look-up-and")).size(12.0).color(theme.text_secondary));
            ui.add_space(12.0);
            if ui.button(crate::ui::i18n::format(ui, "workspace-import-in-settings", &[("icon", PLUS)])).clicked() { open_bible_settings(state); }
        });
        return;
    };
    prepare_search(state);

    let spacing = 6.0;
    let gap = 6.0;
    let total = (ui.available_height() - gap - 4.0).max(220.0);
    let books_h = (total * 0.52 - gap * 0.5).max(80.0);
    let side_h = (total * 0.48 - gap * 0.5).max(80.0);
    let full_w = ui.available_width();

    let query_trimmed = state.bible.search.trim();
    let matching_indices = if query_trimmed.is_empty() {
        None
    } else {
        let (book_part, _) = super::search::split_reference(query_trimmed);
        if let Some(bible) = state.bible.library.get(&state.bible_version) {
            let m = super::search::matching_books(&bible, &book_part);
            if m.is_empty() { None } else { Some(m) }
        } else {
            None
        }
    };
    let visible_books: Vec<usize> = if let Some(ref m) = matching_indices {
        m.clone()
    } else {
        (0..names.len()).collect()
    };

    section_panel(ui, egui::vec2(full_w, books_h), "workspace-books", theme, |ui| {
        let (per_row, cell) = grid_metrics(ui, 54.0, spacing);
        let tile = egui::vec2(cell, cell);
        egui::ScrollArea::vertical()
            .id_salt("cards_books")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for row in (0..visible_books.len()).step_by(per_row) {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = spacing;
                        for item_idx in row..(row + per_row).min(visible_books.len()) {
                            let index = visible_books[item_idx];
                            if (state.bible.testament == 1 && index >= 39)
                                || (state.bible.testament == 2 && index < 39)
                            {
                                continue;
                            }
                            let abbrev = capitalize(&abbrevs[index]);
                            let selected = state.bible_book == names[index];
                            if book_card(ui, tile, &abbrev, &names[index], selected, theme)
                                .clicked()
                            {
                                select_reference(state, index, 1, 1, 1);
                            }
                        }
                    });
                }
            });
    });

    ui.add_space(gap);
    let side_w = ((full_w - gap) / 2.0).max(60.0);
    ui.columns(2, |cols| {
        section_panel(
            &mut cols[0],
            egui::vec2(side_w, side_h),
            "workspace-chapter",
            theme,
            |ui| {
                let target_w = if chapter_count >= 100 { 50.0 } else { 44.0 };
                let (per_row, cell) = grid_metrics(ui, target_w, spacing);
                let tile = egui::vec2(
                    cell,
                    if chapter_count >= 100 {
                        38.0
                    } else {
                        cell.min(42.0)
                    },
                );
                egui::ScrollArea::vertical()
                    .id_salt("cards_chapters")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for row in (0..chapter_count).step_by(per_row) {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = spacing;
                                for value in row + 1..=(row + per_row).min(chapter_count) {
                                    let active =
                                        state.bible_book == names[book] && chapter == value;
                                    if number_card(ui, tile, &value.to_string(), active, theme)
                                        .clicked()
                                    {
                                        select_reference(state, book, value, 1, 1);
                                    }
                                }
                            });
                        }
                    });
            },
        );

        section_panel(
            &mut cols[1],
            egui::vec2(side_w, side_h),
            "workspace-verse",
            theme,
            |ui| {
                let target_w = if verse_count >= 100 { 50.0 } else { 44.0 };
                let (per_row, cell) = grid_metrics(ui, target_w, spacing);
                let tile = egui::vec2(
                    cell,
                    if verse_count >= 100 {
                        38.0
                    } else {
                        cell.min(42.0)
                    },
                );
                egui::ScrollArea::vertical()
                    .id_salt("cards_verses")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let start = state.bible_verse.min(state.bible.verse_end);
                        let end = state.bible_verse.max(state.bible.verse_end);
                        for row in (0..verse_count).step_by(per_row) {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = spacing;
                                for value in row + 1..=(row + per_row).min(verse_count) {
                                    let active = value >= start && value <= end;
                                    if number_card(ui, tile, &value.to_string(), active, theme)
                                        .clicked()
                                    {
                                        if ui.input(|i| i.modifiers.shift) {
                                            state.bible_verse =
                                                state.bible.selection_anchor.min(value);
                                            state.bible.verse_end =
                                                state.bible.selection_anchor.max(value);
                                        } else {
                                            state.bible_verse = value;
                                            state.bible.verse_end = value;
                                            state.bible.selection_anchor = value;
                                        }
                                        sync_preview(state, deck, true);
                                    }
                                }
                            });
                        }
                    });
            },
        );
    });
}
