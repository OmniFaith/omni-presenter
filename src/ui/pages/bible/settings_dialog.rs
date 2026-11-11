use super::settings_preview::{render_settings_preview, PreviewSample};
use crate::core::bible::{
    BibleReferenceAlignment, BibleReferenceHighlight, BibleReferencePosition, BibleReferenceSize,
    BibleSeparator, BibleSettings, BibleSplitAction, BibleSplitPunctuation, BibleVerseAlignment,
    BibleVerseNumberStyle, BibleVerticalAlignment,
};
use crate::core::state::{ControlPanelState, SettingsTab};
use crate::ui::components::dialog;
use crate::ui::modals::settings::components::{
    ButtonTone, apply_settings_style, card, segmented_picker, setting_toggle, styled_button,
};
use eframe::egui::{self, Frame, Margin, RichText, Stroke, Vec2};
use egui_phosphor::regular::{
    ARROW_COUNTER_CLOCKWISE, BOOKS, CHECK, DESKTOP, ROWS, TEXT_T,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum BibleSettingsTab {
    Screen,
    Split,
    Formatting,
}

pub fn render_bible_settings_modal(ctx: &egui::Context, state: &mut ControlPanelState) {
    if !state.bible.show_settings {
        return;
    }

    let tokens = state.theme.clone();
    let initial_settings = state.settings.bible_settings.clone();

    let active_tab_id = egui::Id::unique("bible_settings_active_tab");
    let mut active_tab = ctx
        .data(|d| d.get_temp::<BibleSettingsTab>(active_tab_id))
        .unwrap_or(BibleSettingsTab::Screen);

    let preview_sample_id = egui::Id::unique("bible_preview_sample_choice");
    let mut selected_sample = ctx
        .data(|d| d.get_temp::<PreviewSample>(preview_sample_id))
        .unwrap_or(PreviewSample::John316);

    let mut open_translations_manager = false;
    let mut reset_defaults = false;

    let close = dialog::show(
        ctx,
        "bible_settings_modal",
        "workspace-bible-settings-title",
        "workspace-bible-settings-subtitle",
        620.0,
        &tokens,
        |ui| {
            apply_settings_style(ui, &tokens);
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 8.0);

            // ========================================================
            // 1. TOP: Clean Modern 3-Tab Segmented Switcher
            // ========================================================
            ui.vertical_centered(|ui| {
                ui.set_width(450.0);
                ui.push_id("bible_settings_tab_bar", |ui| {
                    if segmented_picker(
                        ui,
                        &mut active_tab,
                        &[
                            (BibleSettingsTab::Screen, "workspace-bible-tab-screen"),
                            (BibleSettingsTab::Split, "workspace-bible-tab-split"),
                            (BibleSettingsTab::Formatting, "workspace-bible-tab-formatting"),
                        ],
                        &tokens,
                    ) {
                        ui.ctx().data_mut(|d| d.insert_temp(active_tab_id, active_tab));
                    }
                });
            });

            ui.add_space(2.0);

            // ========================================================
            // 2. SHOWCASE: Live Slide Viewport with Active Theme
            // ========================================================
            render_settings_preview(
                ui,
                state,
                &mut selected_sample,
                preview_sample_id,
                &tokens,
            );

            ui.add_space(4.0);

            // ========================================================
            // 3. TAB CONTENT CARDS
            // ========================================================
            match active_tab {
                BibleSettingsTab::Screen => {
                    // Card: Screen Layout & Alignment
                    card(ui, &tokens, |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{DESKTOP}  {}",
                                crate::ui::i18n::text(ui, "workspace-bible-section-layout")
                            ))
                            .size(12.0)
                            .strong()
                            .color(tokens.text_primary),
                        );
                        ui.add_space(4.0);

                        // Reference Position
                        ui.label(
                            RichText::new(crate::ui::i18n::text(ui, "workspace-bible-reference-position"))
                                .size(11.0)
                                .color(tokens.text_secondary),
                        );
                        ui.add_space(2.0);
                        ui.push_id("bible_ref_position_seg", |ui| {
                            segmented_picker(
                                ui,
                                &mut state.settings.bible_settings.reference_position,
                                &[
                                    (BibleReferencePosition::Top, "workspace-bible-ref-top"),
                                    (BibleReferencePosition::Bottom, "workspace-bible-ref-bottom"),
                                    (BibleReferencePosition::BelowVerse, "workspace-bible-ref-below-verse"),
                                    (BibleReferencePosition::EndOfVerse, "workspace-bible-ref-end-of-verse"),
                                    (BibleReferencePosition::Hidden, "workspace-bible-ref-hidden"),
                                ],
                                &tokens,
                            );
                        });

                        ui.add_space(6.0);

                        // Two columns: Reference Alignment & Verse Alignment
                        let col_w = (ui.available_width() - 12.0) / 2.0;
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.set_width(col_w);
                                ui.label(
                                    RichText::new(crate::ui::i18n::text(ui, "workspace-bible-ref-alignment"))
                                        .size(11.0)
                                        .color(tokens.text_secondary),
                                );
                                ui.add_space(2.0);
                                if state.settings.bible_settings.reference_position == BibleReferencePosition::EndOfVerse {
                                    Frame::new()
                                        .fill(tokens.bg_pill)
                                        .stroke(Stroke::new(1.0, tokens.border_default))
                                        .corner_radius(6.0)
                                        .inner_margin(Margin::symmetric(8, 7))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(crate::ui::i18n::text(ui, "workspace-bible-ref-align-inline-hint"))
                                                    .size(10.0)
                                                    .italics()
                                                    .color(tokens.text_muted),
                                            );
                                        });
                                } else if state.settings.bible_settings.reference_position == BibleReferencePosition::Hidden {
                                    Frame::new()
                                        .fill(tokens.bg_pill)
                                        .stroke(Stroke::new(1.0, tokens.border_default))
                                        .corner_radius(6.0)
                                        .inner_margin(Margin::symmetric(8, 7))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(crate::ui::i18n::text(ui, "workspace-bible-ref-hidden"))
                                                    .size(10.0)
                                                    .italics()
                                                    .color(tokens.text_muted),
                                            );
                                        });
                                } else {
                                    ui.push_id("bible_ref_alignment_seg", |ui| {
                                        segmented_picker(
                                            ui,
                                            &mut state.settings.bible_settings.reference_alignment,
                                            &[
                                                (BibleReferenceAlignment::Left, "workspace-bible-ref-align-left"),
                                                (BibleReferenceAlignment::Center, "workspace-bible-ref-align-center"),
                                                (BibleReferenceAlignment::Right, "workspace-bible-ref-align-right"),
                                            ],
                                            &tokens,
                                        );
                                    });
                                }
                            });

                            ui.add_space(12.0);

                            ui.vertical(|ui| {
                                ui.set_width(col_w);
                                ui.label(
                                    RichText::new(crate::ui::i18n::text(ui, "workspace-bible-alignment"))
                                        .size(11.0)
                                        .color(tokens.text_secondary),
                                );
                                ui.add_space(2.0);
                                ui.push_id("bible_alignment_seg", |ui| {
                                    segmented_picker(
                                        ui,
                                        &mut state.settings.bible_settings.alignment,
                                        &[
                                            (BibleVerseAlignment::Left, "workspace-bible-align-left"),
                                            (BibleVerseAlignment::Center, "workspace-bible-align-center"),
                                            (BibleVerseAlignment::Justified, "workspace-bible-align-justified"),
                                        ],
                                        &tokens,
                                    );
                                });
                            });
                        });

                        ui.add_space(6.0);

                        // Vertical Position on Screen
                        ui.label(
                            RichText::new(crate::ui::i18n::text(ui, "workspace-bible-vertical-align"))
                                .size(11.0)
                                .color(tokens.text_secondary),
                        );
                        ui.add_space(2.0);
                        ui.push_id("bible_valign_seg", |ui| {
                            segmented_picker(
                                ui,
                                &mut state.settings.bible_settings.vertical_alignment,
                                &[
                                    (BibleVerticalAlignment::Top, "workspace-bible-valign-top"),
                                    (BibleVerticalAlignment::Center, "workspace-bible-valign-center"),
                                    (BibleVerticalAlignment::Bottom, "workspace-bible-valign-bottom"),
                                ],
                                &tokens,
                            );
                        });
                    });
                }
                BibleSettingsTab::Split => {
                    // Card: Quebra e Divisão de Versículos Extensos
                    card(ui, &tokens, |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{ROWS}  {}",
                                crate::ui::i18n::text(ui, "workspace-bible-section-split")
                            ))
                            .size(12.5)
                            .strong()
                            .color(tokens.text_primary),
                        );
                        ui.add_space(4.0);

                        setting_toggle(
                            ui,
                            &mut state.settings.bible_settings.auto_split_long_verses,
                            "workspace-bible-auto-split",
                            "workspace-bible-auto-split-desc",
                            &tokens,
                        );

                        if state.settings.bible_settings.auto_split_long_verses {
                            ui.add_space(8.0);
                            ui.separator();
                            ui.add_space(8.0);

                            // Header row with title, hint and live pill badge
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new(crate::ui::i18n::text(ui, "workspace-bible-max-chars"))
                                            .size(11.5)
                                            .strong()
                                            .color(tokens.text_primary),
                                    );
                                    ui.label(
                                        RichText::new(crate::ui::i18n::text(ui, "workspace-bible-max-chars-hint"))
                                            .size(9.5)
                                            .color(tokens.text_muted),
                                    );
                                });
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    Frame::new()
                                        .fill(tokens.accent_soft)
                                        .stroke(Stroke::new(1.0, tokens.accent))
                                        .corner_radius(6.0)
                                        .inner_margin(Margin::symmetric(10, 4))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(format!("{} caracteres", state.settings.bible_settings.max_chars_per_slide))
                                                    .size(11.0)
                                                    .strong()
                                                    .color(tokens.accent),
                                            );
                                        });
                                });
                            });

                            ui.add_space(8.0);

                            // Quick presets row
                            let avail_w = ui.available_width();
                            let btn_w = ((avail_w - 18.0) / 4.0).max(60.0);
                            ui.horizontal(|ui| {
                                for (preset_val, preset_label) in [
                                    (80, "80 Curto"),
                                    (120, "120 Padrão"),
                                    (160, "160 Amplo"),
                                    (220, "220 Máximo"),
                                ] {
                                    let is_sel = state.settings.bible_settings.max_chars_per_slide == preset_val;
                                    let bg = if is_sel { tokens.accent_soft } else { tokens.bg_pill };
                                    let stroke = if is_sel { Stroke::new(1.0, tokens.accent) } else { Stroke::new(1.0, tokens.border_default) };
                                    let text_col = if is_sel { tokens.accent } else { tokens.text_secondary };

                                    let btn = egui::Button::new(
                                        RichText::new(preset_label)
                                            .size(10.5)
                                            .color(text_col)
                                    )
                                    .fill(bg)
                                    .stroke(stroke)
                                    .corner_radius(6.0)
                                    .min_size(Vec2::new(btn_w, 24.0));

                                    if ui.add(btn).clicked() {
                                        state.settings.bible_settings.max_chars_per_slide = preset_val;
                                    }
                                }
                            });

                            ui.add_space(6.0);

                            // Clean native egui slider with fine adjust label
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("Ajuste fino: {} carac.", state.settings.bible_settings.max_chars_per_slide))
                                        .size(10.5)
                                        .color(tokens.text_muted),
                                );
                                ui.add(
                                    egui::Slider::new(&mut state.settings.bible_settings.max_chars_per_slide, 50..=280)
                                        .show_value(false)
                                );
                            });

                            ui.add_space(8.0);

                            // Two columns: Action & Split Criterion
                            let col_w = (ui.available_width() - 12.0) / 2.0;
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.set_width(col_w);
                                    ui.label(
                                        RichText::new(crate::ui::i18n::text(ui, "workspace-bible-split-action"))
                                            .size(10.5)
                                            .color(tokens.text_secondary),
                                    );
                                    ui.add_space(2.0);
                                    ui.push_id("bible_split_action_seg", |ui| {
                                        segmented_picker(
                                            ui,
                                            &mut state.settings.bible_settings.split_action,
                                            &[
                                                (BibleSplitAction::MultipleSlides, "workspace-bible-action-slides"),
                                                (BibleSplitAction::BreakLines, "workspace-bible-action-lines"),
                                            ],
                                            &tokens,
                                        );
                                    });
                                });

                                ui.add_space(12.0);

                                ui.vertical(|ui| {
                                    ui.set_width(col_w);
                                    ui.label(
                                        RichText::new(crate::ui::i18n::text(ui, "workspace-bible-split-mode"))
                                            .size(10.5)
                                            .color(tokens.text_secondary),
                                    );
                                    ui.add_space(2.0);
                                    ui.push_id("bible_split_mode_seg", |ui| {
                                        segmented_picker(
                                            ui,
                                            &mut state.settings.bible_settings.split_punctuation,
                                            &[
                                                (BibleSplitPunctuation::NaturalPunctuation, "workspace-bible-mode-punctuation"),
                                                (BibleSplitPunctuation::WordBoundary, "workspace-bible-mode-words"),
                                            ],
                                            &tokens,
                                        );
                                    });
                                });
                            });

                            if state.settings.bible_settings.split_action == BibleSplitAction::MultipleSlides {
                                ui.add_space(6.0);
                                setting_toggle(
                                    ui,
                                    &mut state.settings.bible_settings.split_ellipsis,
                                    "workspace-bible-split-ellipsis",
                                    "workspace-bible-split-ellipsis-desc",
                                    &tokens,
                                );
                            }
                        }
                    });
                }
                BibleSettingsTab::Formatting => {
                    // Card 1: Identificação & Numeração
                    card(ui, &tokens, |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{TEXT_T}  {}",
                                crate::ui::i18n::text(ui, "workspace-bible-section-numbering")
                            ))
                            .size(12.0)
                            .strong()
                            .color(tokens.text_primary),
                        );
                        ui.add_space(4.0);

                        // Separator & Number Style
                        let col_w = (ui.available_width() - 12.0) / 2.0;
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.set_width(col_w);
                                ui.label(
                                    RichText::new(crate::ui::i18n::text(ui, "workspace-bible-separator"))
                                        .size(11.0)
                                        .color(tokens.text_secondary),
                                );
                                ui.add_space(2.0);
                                ui.push_id("bible_separator_seg", |ui| {
                                    segmented_picker(
                                        ui,
                                        &mut state.settings.bible_settings.separator,
                                        &[
                                            (BibleSeparator::Colon, ": (ex: 3:16)"),
                                            (BibleSeparator::Dot, ". (ex: 3.16)"),
                                        ],
                                        &tokens,
                                    );
                                });
                            });

                            ui.add_space(12.0);

                            ui.vertical(|ui| {
                                ui.set_width(col_w);
                                ui.label(
                                    RichText::new(crate::ui::i18n::text(ui, "workspace-bible-verse-number-style"))
                                        .size(11.0)
                                        .color(tokens.text_secondary),
                                );
                                ui.add_space(2.0);
                                ui.push_id("bible_num_style_seg", |ui| {
                                    segmented_picker(
                                        ui,
                                        &mut state.settings.bible_settings.verse_number_style,
                                        &[
                                            (BibleVerseNumberStyle::Bracket, "workspace-bible-num-bracket"),
                                            (BibleVerseNumberStyle::Superscript, "workspace-bible-num-superscript"),
                                            (BibleVerseNumberStyle::Dot, "workspace-bible-num-dot"),
                                        ],
                                        &tokens,
                                    );
                                });
                            });
                        });

                        ui.add_space(6.0);

                        setting_toggle(
                            ui,
                            &mut state.settings.bible_settings.range_number_only,
                            "workspace-bible-range-number-only",
                            "workspace-bible-range-number-only-desc",
                            &tokens,
                        );

                        ui.add_space(4.0);

                        setting_toggle(
                            ui,
                            &mut state.settings.bible_settings.show_verse_numbers,
                            "workspace-bible-show-verse-numbers",
                            "workspace-bible-show-verse-numbers-desc",
                            &tokens,
                        );
                    });

                    ui.add_space(4.0);

                    // Card 2: Opções Tipográficas
                    card(ui, &tokens, |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{BOOKS}  {}",
                                crate::ui::i18n::text(ui, "workspace-bible-section-typography")
                            ))
                            .size(12.0)
                            .strong()
                            .color(tokens.text_primary),
                        );
                        ui.add_space(4.0);

                        // Destaque da Referência
                        ui.label(
                            RichText::new(crate::ui::i18n::text(ui, "workspace-bible-ref-highlight"))
                                .size(11.0)
                                .color(tokens.text_secondary),
                        );
                        ui.add_space(2.0);
                        ui.push_id("bible_ref_highlight_seg", |ui| {
                            segmented_picker(
                                ui,
                                &mut state.settings.bible_settings.reference_highlight,
                                &[
                                    (BibleReferenceHighlight::Discreet, "workspace-bible-highlight-discreet"),
                                    (BibleReferenceHighlight::Accent, "workspace-bible-highlight-accent"),
                                    (BibleReferenceHighlight::Badge, "workspace-bible-highlight-badge"),
                                ],
                                &tokens,
                            );
                        });

                        ui.add_space(6.0);

                        // Tamanho da Referência
                        ui.label(
                            RichText::new(crate::ui::i18n::text(ui, "workspace-bible-ref-size"))
                                .size(11.0)
                                .color(tokens.text_secondary),
                        );
                        ui.add_space(2.0);
                        ui.push_id("bible_ref_size_seg", |ui| {
                            segmented_picker(
                                ui,
                                &mut state.settings.bible_settings.reference_size,
                                &[
                                    (BibleReferenceSize::Normal, "workspace-bible-ref-size-normal"),
                                    (BibleReferenceSize::Compact, "workspace-bible-ref-size-compact"),
                                ],
                                &tokens,
                            );
                        });

                        ui.add_space(6.0);

                        setting_toggle(
                            ui,
                            &mut state.settings.bible_settings.quotes_around_verse,
                            "workspace-bible-quotes",
                            "workspace-bible-quotes-desc",
                            &tokens,
                        );

                        ui.add_space(4.0);

                        setting_toggle(
                            ui,
                            &mut state.settings.bible_settings.show_version,
                            "workspace-bible-show-version-tag",
                            "workspace-bible-show-version-tag-desc",
                            &tokens,
                        );

                        ui.add_space(4.0);

                        setting_toggle(
                            ui,
                            &mut state.settings.bible_settings.use_abbreviation,
                            "workspace-bible-use-abbreviation",
                            "workspace-bible-use-abbreviation-desc",
                            &tokens,
                        );

                        ui.add_space(4.0);

                        setting_toggle(
                            ui,
                            &mut state.settings.bible_settings.uppercase_reference,
                            "workspace-bible-uppercase-ref",
                            "workspace-bible-uppercase-ref-desc",
                            &tokens,
                        );

                        ui.add_space(4.0);

                        setting_toggle(
                            ui,
                            &mut state.settings.bible_settings.italic_reference,
                            "workspace-bible-italic-ref",
                            "workspace-bible-italic-ref-desc",
                            &tokens,
                        );
                    });
                }
            }

            ui.add_space(2.0);
            ui.separator();
            ui.add_space(2.0);

            // ========================================================
            // 4. BOTTOM ACTION FOOTER
            // ========================================================
            ui.horizontal(|ui| {
                if styled_button(
                    ui,
                    Some(BOOKS),
                    "workspace-manage-bible-translations",
                    ButtonTone::Secondary,
                    &tokens,
                )
                .clicked()
                {
                    open_translations_manager = true;
                }

                if styled_button(
                    ui,
                    Some(ARROW_COUNTER_CLOCKWISE),
                    "workspace-bible-reset-defaults",
                    ButtonTone::Ghost,
                    &tokens,
                )
                .clicked()
                {
                    reset_defaults = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if styled_button(
                        ui,
                        Some(CHECK),
                        "settings-done",
                        ButtonTone::Primary,
                        &tokens,
                    )
                    .clicked()
                    {
                        state.bible.show_settings = false;
                    }
                });
            });
        },
    );

    if open_translations_manager {
        state.bible.show_settings = false;
        state.settings_ui.open = true;
        state.settings_ui.tab = SettingsTab::Bible;
    }

    if reset_defaults {
        state.settings.bible_settings = BibleSettings::default();
    }

    if state.settings.bible_settings != initial_settings || reset_defaults {
        state.settings_ui.save_requested = true;
    }

    if close {
        state.bible.show_settings = false;
    }
}
