use crate::core::bible::{
    BibleReferenceAlignment, BibleReferenceHighlight, BibleReferencePosition, BibleReferenceSize,
    BibleSettings, BibleSplitAction, BibleSplitPunctuation, BibleVerseAlignment,
    BibleVerticalAlignment, format_bible_slide, format_verse_number, split_verse_text,
};
use crate::core::state::ControlPanelState;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, Frame, Margin, Pos2, Rect, RichText, Stroke, StrokeKind, Vec2};
use egui_phosphor::regular::DESKTOP;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PreviewSample {
    John316,
    Psalm23,
    Romans828,
}

pub fn render_settings_preview(
    ui: &mut egui::Ui,
    state: &ControlPanelState,
    selected_sample: &mut PreviewSample,
    preview_sample_id: egui::Id,
    tokens: &AppTheme,
) {
    let live_theme = state.live_theme().clone();
    let aspect = state.presentation_aspect_ratio.clamp(1.0, 2.5);

    let active_bible = state.bible.library.get(&state.bible_version);
    let (sample_idx, chap, s_verse, e_verse, verses) = match *selected_sample {
        PreviewSample::John316 => (
            42,
            3,
            16,
            16,
            vec![crate::ui::i18n::text(ui, "workspace-bible-sample-verse").to_string()],
        ),
        PreviewSample::Psalm23 => (
            18,
            23,
            1,
            2,
            vec![
                crate::ui::i18n::text(ui, "workspace-bible-sample-psalm-1").to_string(),
                crate::ui::i18n::text(ui, "workspace-bible-sample-psalm-2").to_string(),
            ],
        ),
        PreviewSample::Romans828 => (
            44,
            8,
            28,
            28,
            vec![
                crate::ui::i18n::text(ui, "workspace-bible-sample-romans").to_string(),
            ],
        ),
    };

    let (sample_name, sample_abbrev) = if let Some(ref b) = active_bible.as_ref().and_then(|lib| lib.books.get(sample_idx)) {
        (b.name.clone(), b.abbrev.clone())
    } else {
        let names = crate::core::bible::book_names_for_language(state.settings.language);
        let abbrevs = crate::core::bible::book_abbrevs_for_language(state.settings.language);
        (names[sample_idx].to_string(), abbrevs[sample_idx].to_string())
    };
    let book_name = sample_name.as_str();
    let book_abbrev = sample_abbrev.as_str();

    let current_ver = if state.bible_version.trim().is_empty() {
        "NVI"
    } else {
        &state.bible_version
    };

    let (preview_verses, preview_total_parts, preview_ref_override) = if state.settings.bible_settings.auto_split_long_verses
        && state.settings.bible_settings.split_action == BibleSplitAction::MultipleSlides
        && verses.len() == 1
        && verses[0].chars().count() > state.settings.bible_settings.max_chars_per_slide
    {
        let chunks = split_verse_text(
            &verses[0],
            state.settings.bible_settings.max_chars_per_slide,
            state.settings.bible_settings.split_punctuation == BibleSplitPunctuation::NaturalPunctuation,
            state.settings.bible_settings.split_ellipsis,
        );
        let total = chunks.len();
        (vec![chunks.into_iter().next().unwrap_or_default()], total, None)
    } else if state.settings.bible_settings.range_number_only && s_verse < e_verse {
        let num_str = format_verse_number(s_verse, state.settings.bible_settings.verse_number_style);
        let first_with_num = format!("{num_str} {}", &verses[0]);
        (vec![first_with_num], 2, Some(String::new()))
    } else {
        (verses, 1, None)
    };

    let (mut reference, _) = format_bible_slide(
        &state.settings.bible_settings,
        book_name,
        book_abbrev,
        current_ver,
        chap,
        s_verse,
        e_verse,
        &preview_verses,
    );
    if let Some(custom_ref) = preview_ref_override {
        reference = custom_ref;
    } else if preview_total_parts > 1 {
        reference = format!("{reference} (1/{preview_total_parts})");
    }

    let aspect_label = if (aspect - 16.0 / 9.0).abs() < 0.05 {
        "16:9"
    } else if (aspect - 16.0 / 10.0).abs() < 0.05 {
        "16:10"
    } else if (aspect - 4.0 / 3.0).abs() < 0.05 {
        "4:3"
    } else if (aspect - 21.0 / 9.0).abs() < 0.05 {
        "21:9"
    } else {
        "Tela"
    };

    // Showcase Container Frame
    Frame::new()
        .fill(tokens.bg_panel)
        .stroke(Stroke::new(1.0, tokens.border_default))
        .corner_radius(10.0)
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            // Header Toolbar inside the showcase
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);
                    ui.label(
                        RichText::new(DESKTOP)
                            .size(12.0)
                            .color(tokens.accent),
                    );
                    ui.label(
                        RichText::new(&live_theme.name)
                            .size(11.0)
                            .strong()
                            .color(tokens.text_primary),
                    );

                    Frame::new()
                        .fill(tokens.bg_pill)
                        .stroke(Stroke::new(1.0, tokens.border_default))
                        .corner_radius(4.0)
                        .inner_margin(Margin::symmetric(5, 1))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(aspect_label)
                                    .size(9.0)
                                    .color(tokens.text_muted),
                            );
                        });

                    if state.settings.bible_settings.range_number_only && s_verse < e_verse {
                        Frame::new()
                            .fill(tokens.accent_soft)
                            .stroke(Stroke::new(1.0, tokens.accent))
                            .corner_radius(4.0)
                            .inner_margin(Margin::symmetric(5, 1))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new("Slide 1/2 • Número no Início do Versículo")
                                        .size(9.0)
                                        .strong()
                                        .color(tokens.accent),
                                );
                            });
                    } else if preview_total_parts > 1 {
                        Frame::new()
                            .fill(tokens.accent_soft)
                            .stroke(Stroke::new(1.0, tokens.accent))
                            .corner_radius(4.0)
                            .inner_margin(Margin::symmetric(5, 1))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(format!("Parte 1/{preview_total_parts}"))
                                        .size(9.0)
                                        .strong()
                                        .color(tokens.accent),
                                );
                            });
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    for (sample, label) in [
                        (PreviewSample::Romans828, "Rm 8:28"),
                        (PreviewSample::Psalm23, "Sl 23:1-2"),
                        (PreviewSample::John316, "Jo 3:16"),
                    ] {
                        let is_active = *selected_sample == sample;
                        let btn_fill = if is_active {
                            tokens.bg_card_active
                        } else {
                            tokens.bg_pill
                        };
                        let btn_stroke = if is_active {
                            Stroke::new(1.0, tokens.accent)
                        } else {
                            Stroke::new(1.0, tokens.border_default)
                        };

                        let btn = egui::Button::new(
                            RichText::new(label)
                                .size(9.5)
                                .color(if is_active { tokens.accent } else { tokens.text_secondary })
                        )
                        .fill(btn_fill)
                        .stroke(btn_stroke)
                        .corner_radius(4.0)
                        .min_size(Vec2::new(46.0, 19.0));

                        if ui.add(btn).clicked() {
                            *selected_sample = sample;
                            ui.ctx().data_mut(|d| d.insert_temp(preview_sample_id, *selected_sample));
                        }
                    }
                });
            });

            ui.add_space(5.0);

            // Screen Frame Dimensions with actual aspect ratio
            let avail_w = ui.available_width();
            let max_canvas_h = 145.0;
            let (canvas_w, canvas_h) = if avail_w / aspect <= max_canvas_h {
                (avail_w, avail_w / aspect)
            } else {
                (max_canvas_h * aspect, max_canvas_h)
            };

            let (allocated_rect, _) = ui.allocate_exact_size(
                egui::vec2(avail_w, canvas_h),
                egui::Sense::hover(),
            );
            let canvas_rect = Rect::from_center_size(
                allocated_rect.center(),
                egui::vec2(canvas_w, canvas_h),
            );

            // Depth Shadow
            let screen_shadow = egui::epaint::Shadow {
                offset: [0, 4],
                blur: 16,
                spread: 0,
                color: Color32::from_black_alpha(100),
            };
            ui.painter().add(screen_shadow.as_shape(canvas_rect, 8.0));

            // Paint the live presentation theme background
            crate::render::paint_theme_bg(ui.painter(), canvas_rect, &live_theme, 8.0);

            // Subtle outer border
            ui.painter().rect_stroke(
                canvas_rect,
                8.0,
                Stroke::new(1.0, Color32::from_white_alpha(35)),
                StrokeKind::Inside,
            );

            // Render live slide content with precision placement
            let content_rect = canvas_rect.shrink2(Vec2::new(20.0, 10.0));

            let ref_pos = state.settings.bible_settings.reference_position;
            let ref_align = match state.settings.bible_settings.reference_alignment {
                BibleReferenceAlignment::Left => egui::Align::LEFT,
                BibleReferenceAlignment::Center => egui::Align::Center,
                BibleReferenceAlignment::Right => egui::Align::RIGHT,
            };

            let verse_text_color = if live_theme.text_color.a() > 20 {
                live_theme.text_color
            } else {
                Color32::WHITE
            };

            // 1. Top Reference (pinned to top of slide)
            let (verse_area_rect, _top_ref_h) = if ref_pos == BibleReferencePosition::Top {
                let ref_rect = Rect::from_min_max(
                    content_rect.min,
                    Pos2::new(content_rect.max.x, content_rect.min.y + 15.0),
                );
                let mut top_ui = ui.new_child(egui::UiBuilder::new().max_rect(ref_rect));
                top_ui.with_layout(egui::Layout::top_down(ref_align), |ui| {
                    render_preview_reference(ui, &reference, &state.settings.bible_settings, verse_text_color);
                });
                (
                    Rect::from_min_max(
                        Pos2::new(content_rect.min.x, content_rect.min.y + 16.0),
                        content_rect.max,
                    ),
                    16.0,
                )
            } else {
                (content_rect, 0.0)
            };

            // 2. Bottom Reference (pinned to bottom of slide)
            let verse_area_rect = if ref_pos == BibleReferencePosition::Bottom {
                let ref_rect = Rect::from_min_max(
                    Pos2::new(verse_area_rect.min.x, verse_area_rect.max.y - 15.0),
                    verse_area_rect.max,
                );
                let mut bot_ui = ui.new_child(egui::UiBuilder::new().max_rect(ref_rect));
                bot_ui.with_layout(egui::Layout::top_down(ref_align), |ui| {
                    render_preview_reference(ui, &reference, &state.settings.bible_settings, verse_text_color);
                });
                Rect::from_min_max(
                    verse_area_rect.min,
                    Pos2::new(verse_area_rect.max.x, verse_area_rect.max.y - 16.0),
                )
            } else {
                verse_area_rect
            };

            // 3. Verses Body
            let mut screen_ui = ui.new_child(egui::UiBuilder::new().max_rect(verse_area_rect));

            let halign = match state.settings.bible_settings.alignment {
                BibleVerseAlignment::Center => egui::Align::Center,
                BibleVerseAlignment::Left | BibleVerseAlignment::Justified => egui::Align::LEFT,
            };

            let verse_font_size = 11.5;

            let total_measured_h: f32 = preview_verses
                .iter()
                .enumerate()
                .map(|(idx, v)| {
                    let num_str = format_verse_number(s_verse + idx, state.settings.bible_settings.verse_number_style);
                    let prefix = if preview_verses.len() > 1 && state.settings.bible_settings.show_verse_numbers {
                        format!("{num_str} ")
                    } else {
                        String::new()
                    };
                    let quote_open = if state.settings.bible_settings.quotes_around_verse && idx == 0 { "“" } else { "" };
                    let quote_close = if state.settings.bible_settings.quotes_around_verse && idx == preview_verses.len() - 1 { "”" } else { "" };
                    let is_last = idx == preview_verses.len() - 1;
                    let line_text = if is_last && ref_pos == BibleReferencePosition::EndOfVerse {
                        format!("{quote_open}{prefix}{v}{quote_close}  ({reference})")
                    } else {
                        format!("{quote_open}{prefix}{v}{quote_close}")
                    };
                    let g = ui.painter().layout(
                        line_text,
                        egui::FontId::proportional(verse_font_size),
                        verse_text_color,
                        verse_area_rect.width(),
                    );
                    g.size().y
                })
                .sum::<f32>()
                + if ref_pos == BibleReferencePosition::BelowVerse { 18.0 } else { 0.0 };

            let valig_space = match state.settings.bible_settings.vertical_alignment {
                BibleVerticalAlignment::Top => 2.0,
                BibleVerticalAlignment::Center => {
                    ((verse_area_rect.height() - total_measured_h) / 2.0).max(2.0)
                }
                BibleVerticalAlignment::Bottom => {
                    (verse_area_rect.height() - total_measured_h - 4.0).max(2.0)
                }
            };

            screen_ui.vertical(|ui| {
                ui.add_space(valig_space);
                ui.with_layout(egui::Layout::top_down(halign), |ui| {
                    // Verses
                    for (idx, v) in preview_verses.iter().enumerate() {
                        let num_str = format_verse_number(s_verse + idx, state.settings.bible_settings.verse_number_style);
                        let prefix = if preview_verses.len() > 1 && state.settings.bible_settings.show_verse_numbers {
                            format!("{num_str} ")
                        } else {
                            String::new()
                        };

                        let quote_open = if state.settings.bible_settings.quotes_around_verse && idx == 0 {
                            "“"
                        } else {
                            ""
                        };
                        let quote_close = if state.settings.bible_settings.quotes_around_verse && idx == preview_verses.len() - 1 {
                            "”"
                        } else {
                            ""
                        };

                        let is_last = idx == preview_verses.len() - 1;
                        let line_text = if is_last && ref_pos == BibleReferencePosition::EndOfVerse {
                            let styled_ref = if state.settings.bible_settings.italic_reference {
                                format!("({reference})")
                            } else {
                                format!("({reference})")
                            };
                            format!("{quote_open}{prefix}{v}{quote_close}  {styled_ref}")
                        } else {
                            format!("{quote_open}{prefix}{v}{quote_close}")
                        };

                        let mut verse_rich = RichText::new(line_text)
                            .size(verse_font_size)
                            .color(verse_text_color);
                        if live_theme.font_bold {
                            verse_rich = verse_rich.strong();
                        }
                        if live_theme.font_italic {
                            verse_rich = verse_rich.italics();
                        }
                        ui.label(verse_rich);
                    }

                    // 4. BelowVerse Reference (immediately under the verses)
                    if ref_pos == BibleReferencePosition::BelowVerse {
                        ui.add_space(4.0);
                        ui.with_layout(egui::Layout::top_down(ref_align), |ui| {
                            render_preview_reference(ui, &reference, &state.settings.bible_settings, verse_text_color);
                        });
                    }
                });
            });
        });
}

fn render_preview_reference(
    ui: &mut egui::Ui,
    reference: &str,
    settings: &BibleSettings,
    verse_text_color: Color32,
) {
    if reference.trim().is_empty() {
        return;
    }
    let font_size = match settings.reference_size {
        BibleReferenceSize::Normal => 11.5,
        BibleReferenceSize::Compact => 8.5,
    };
    let mut text = RichText::new(reference).size(font_size);
    if settings.italic_reference {
        text = text.italics();
    }
    match settings.reference_highlight {
        BibleReferenceHighlight::Discreet => {
            let ref_color = Color32::from_rgba_premultiplied(
                verse_text_color.r().saturating_sub(60),
                verse_text_color.g().saturating_sub(60),
                verse_text_color.b().saturating_sub(60),
                180,
            );
            ui.label(text.color(ref_color));
        }
        BibleReferenceHighlight::Accent => {
            let ref_color = Color32::from_rgba_premultiplied(
                verse_text_color.r().saturating_sub(15),
                verse_text_color.g().saturating_sub(15),
                verse_text_color.b().saturating_sub(15),
                245,
            );
            ui.label(text.color(ref_color).strong());
        }
        BibleReferenceHighlight::Badge => {
            Frame::new()
                .fill(Color32::from_black_alpha(110))
                .stroke(Stroke::new(1.0, Color32::from_white_alpha(45)))
                .corner_radius(4.0)
                .inner_margin(Margin::symmetric(7, 2))
                .show(ui, |ui| {
                    ui.label(text.color(Color32::WHITE).strong());
                });
        }
    }
}

