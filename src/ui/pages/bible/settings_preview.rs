use crate::core::bible::{
    BibleSplitAction, BibleSplitPunctuation, format_bible_reference,
    format_bible_slide_with_numbered_ref, split_verse_text,
};
use crate::core::state::ControlPanelState;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, Frame, Margin, Rect, RichText, Stroke, StrokeKind};
use egui_phosphor::regular::DESKTOP;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PreviewSample {
    John316,
    Psalm23,
    Romans828,
}

impl PreviewSample {
    fn passage(self) -> (usize, usize, usize, usize, &'static [&'static str]) {
        match self {
            Self::John316 => (42, 3, 16, 16, &["workspace-bible-sample-verse"]),
            Self::Psalm23 => (
                18,
                23,
                1,
                2,
                &[
                    "workspace-bible-sample-psalm-1",
                    "workspace-bible-sample-psalm-2",
                ],
            ),
            Self::Romans828 => (44, 8, 28, 28, &["workspace-bible-sample-romans"]),
        }
    }
}

struct SampleSlide {
    content: String,
    parts: usize,
    numbered: bool,
}

fn sample_slide(state: &ControlPanelState, sample: PreviewSample) -> SampleSlide {
    let settings = &state.settings.bible_settings;
    let language = state.settings.language;
    let (book_index, chapter, start, end, keys) = sample.passage();
    let mut verses: Vec<String> = keys
        .iter()
        .map(|key| language.text(key).to_owned())
        .collect();
    let bible = state.bible.library.get(&state.bible_version);
    let book = bible.as_ref().and_then(|bible| bible.books.get(book_index));
    let names = crate::core::bible::book_names_for_language(language);
    let abbreviations = crate::core::bible::book_abbrevs_for_language(language);
    let (name, abbreviation) = book
        .map_or((names[book_index], abbreviations[book_index]), |book| {
            (book.name.as_str(), book.abbrev.as_str())
        });
    let mut reference = format_bible_reference(
        settings,
        name,
        abbreviation,
        &state.bible_version,
        chapter,
        start,
        end,
    );
    let numbered = settings.range_number_only && start < end;
    let mut parts = 1;
    if numbered {
        verses.truncate(1);
        reference.clear();
        parts = end - start + 1;
    }
    if settings.auto_split_long_verses
        && settings.split_action == BibleSplitAction::MultipleSlides
        && verses.len() == 1
        && verses[0].chars().count() > settings.max_chars_per_slide
    {
        let chunks = split_verse_text(
            &verses[0],
            settings.max_chars_per_slide,
            settings.split_punctuation == BibleSplitPunctuation::NaturalPunctuation,
            settings.split_ellipsis,
        );
        if !numbered {
            parts = chunks.len();
            if parts > 1 {
                reference = format!("{reference} (1/{parts})");
            }
        }
        verses = vec![chunks.into_iter().next().unwrap_or_default()];
    }
    let (_, content) =
        format_bible_slide_with_numbered_ref(settings, reference, start, &verses, numbered);
    SampleSlide {
        content,
        parts,
        numbered,
    }
}

pub fn render_settings_preview(
    ui: &mut egui::Ui,
    state: &ControlPanelState,
    selected_sample: &mut PreviewSample,
    preview_sample_id: egui::Id,
    tokens: &AppTheme,
) {
    let theme = state.live_theme();
    let language = state.settings.language;
    let aspect = state.presentation_aspect_ratio.clamp(0.5, 3.5);
    Frame::new()
        .fill(tokens.bg_panel)
        .stroke(Stroke::new(1.0, tokens.border_default))
        .corner_radius(10.0)
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(DESKTOP).color(tokens.accent));
                ui.label(RichText::new(theme.display_name(language)).size(11.0));
                ui.label(
                    RichText::new(format!("{aspect:.2}:1"))
                        .size(9.0)
                        .color(tokens.text_muted),
                );
                for sample in [
                    PreviewSample::John316,
                    PreviewSample::Psalm23,
                    PreviewSample::Romans828,
                ] {
                    let (book, chapter, start, end, _) = sample.passage();
                    let abbreviation =
                        crate::core::bible::book_abbrevs_for_language(language)[book];
                    let label = if start == end {
                        format!("{abbreviation} {chapter}:{start}")
                    } else {
                        format!("{abbreviation} {chapter}:{start}–{end}")
                    };
                    if ui
                        .selectable_value(selected_sample, sample, RichText::new(label).size(9.5))
                        .changed()
                    {
                        ui.ctx()
                            .data_mut(|data| data.insert_temp(preview_sample_id, *selected_sample));
                    }
                }
            });
            let slide = sample_slide(state, *selected_sample);
            if slide.numbered {
                ui.label(
                    RichText::new(language.text("workspace-bible-preview-numbered"))
                        .size(9.0)
                        .color(tokens.accent),
                );
            } else if slide.parts > 1 {
                ui.label(
                    RichText::new(language.format(
                        "workspace-bible-preview-part",
                        &[("parts", &slide.parts.to_string())],
                    ))
                    .size(9.0)
                    .color(tokens.accent),
                );
            }
            ui.add_space(5.0);
            let width = ui.available_width();
            let height = (width / aspect).min(145.0);
            let (slot, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
            let canvas = Rect::from_center_size(slot.center(), egui::vec2(height * aspect, height));
            ui.painter().add(
                egui::epaint::Shadow {
                    offset: [0, 4],
                    blur: 16,
                    spread: 0,
                    color: Color32::from_black_alpha(100),
                }
                .as_shape(canvas, 8.0),
            );
            crate::render::paint_theme_bg(ui.painter(), canvas, theme, 8.0);
            // Exactly the same content formatter and renderer as the real output.
            crate::render::render_theme_lyrics(ui.painter(), canvas, theme, &slide.content);
            ui.painter().rect_stroke(
                canvas,
                8.0,
                Stroke::new(1.0, Color32::from_white_alpha(35)),
                StrokeKind::Inside,
            );
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_sample_uses_the_projection_formatter() {
        let mut state = ControlPanelState::default();
        state.settings.bible_settings.auto_split_long_verses = false;
        let slide = sample_slide(&state, PreviewSample::John316);
        let language = state.settings.language;
        let bible = state.bible.library.get(&state.bible_version).unwrap();
        let book = &bible.books[42];
        let (_, expected) = crate::core::bible::format_bible_slide(
            &state.settings.bible_settings,
            &book.name,
            &book.abbrev,
            &state.bible_version,
            3,
            16,
            16,
            &[language.text("workspace-bible-sample-verse").to_owned()],
        );
        assert_eq!(slide.content, expected);
    }

    #[test]
    fn summarized_sample_keeps_numbers_outside_quotes_and_hides_the_full_reference() {
        let mut state = ControlPanelState::default();
        state.settings.bible_settings.range_number_only = true;
        state.settings.bible_settings.quotes_around_verse = true;
        state.settings.bible_settings.auto_split_long_verses = false;
        let slide = sample_slide(&state, PreviewSample::Psalm23);
        let visible = crate::core::slides::strip_directive_commands(&slide.content);
        assert!(visible.starts_with("[1] “"), "{visible}");
        assert!(!slide.content.contains("//ref-pos:"));
        assert!(slide.numbered);
    }
}
