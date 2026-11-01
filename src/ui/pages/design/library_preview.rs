use super::super::icon;
use super::{pin, playlist};
use crate::{
    core::{
        design::{LayerContent, now_ms},
        layout::ContainerKind,
        slides::SlideDeck,
        state::{ControlPanelState, SidebarPage},
    },
    ui::components::{centered_horizontal, render_header_bar},
};
use eframe::egui::{self, RichText};
use egui_phosphor::regular::*;

pub fn render_preview(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    presenting: &mut bool,
) {
    let theme = state.theme.clone();
    let language = state.settings.language;
    let Some(mut scene) = deck.preview_design.clone() else {
        return;
    };
    if !scene.overlay && scene.background_mode != "color" {
        scene.background_mode = "theme".into();
        scene.theme_id = Some(state.preview_theme().id.clone());
    }
    render_header_bar(ui, ContainerKind::Lyrics, "workspace-design-preview", &theme);
    ui.add_space(4.0);
    centered_horizontal(ui, "design_preview_actions", 140.0, |ui| {
        if icon(ui, PLAY, language.text("common-project"), &theme).clicked() {
            deck.project_preview();
            *presenting = true;
            state.show_presentation_dock = state.settings.operator().show_dock;
        }
        if icon(ui, PENCIL_SIMPLE, language.text("common-edit"), &theme).clicked() {
            state.active_sidebar_page = SidebarPage::Design;
        }
        if icon(ui, PUSH_PIN, language.text("common-pin"), &theme).clicked() {
            pin(state, &scene);
        }
        if icon(ui, LIST_PLUS, language.text("common-add-playlist"), &theme).clicked() {
            playlist(state, &scene);
        }
        if deck.live_overlay.is_some()
            && icon(ui, X, language.text("control-remove-overlay"), &theme).clicked()
        {
            deck.live_overlay = None;
        }
    });
    ui.add_space(8.0);
    ui.vertical_centered(|ui| {
        ui.strong(&scene.name);
        ui.label(
            RichText::new(language.text(if scene.overlay {
                "control-overlay-design"
            } else {
                "control-dynamic-slide"
            }))
            .size(11.0)
            .color(theme.text_muted),
        );
    });
    ui.add_space(4.0);
    let mut changed = false;
    egui::ScrollArea::vertical()
        .id_salt("design_preview_controls")
        .show(ui, |ui| {
            for layer in &mut scene.layers {
                match &mut layer.content {
                    LayerContent::Plugin(instance) => {
                        ui.separator();
                        changed |= ui.push_id(&layer.id, |ui| crate::ui::components::plugin_fields::component_operational(ui, instance, language, &theme)).inner;
                    }
                    LayerContent::Countdown(timer) => {
                        // Follow the live timer, including auto-starts triggered by a playlist item.
                        if let Some(live) = deck
                            .live_design
                            .as_ref()
                            .filter(|live| live.id == scene.id)
                            .or(deck
                                .live_overlay
                                .as_ref()
                                .filter(|live| live.id == scene.id))
                            && let Some(crate::core::design::DesignLayer {
                                content: LayerContent::Countdown(running),
                                ..
                            }) = live.layers.iter().find(|item| item.id == layer.id)
                            {
                                timer.started_at = running.started_at;
                                timer.remaining_ms = running.remaining_ms;
                            }
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(6.0);

                        let is_running = timer.started_at.is_some();
                        egui::Frame::new()
                            .fill(theme.bg_card)
                            .stroke(egui::Stroke::new(1.0, theme.border_default))
                            .corner_radius(8.0)
                            .inner_margin(egui::Margin::same(10))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(CLOCK).size(15.0).color(theme.accent));
                                    ui.strong(&layer.name);
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let badge_bg = if is_running { theme.accent } else { theme.bg_pill };
                                        let badge_fg = if is_running { theme.text_on_accent } else { theme.text_secondary };
                                        let badge_text = language.text(if is_running { "design-running" } else { "design-paused" });
                                        let gal = ui.painter().layout_no_wrap(badge_text.to_string(), egui::FontId::proportional(10.5), badge_fg);
                                        let badge_w = gal.size().x + 12.0;
                                        let (rect, _) = ui.allocate_exact_size(egui::vec2(badge_w, 18.0), egui::Sense::hover());
                                        ui.painter().rect_filled(rect, 9.0, badge_bg);
                                        ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, badge_text, egui::FontId::proportional(10.5), badge_fg);
                                    });
                                });

                                ui.add_space(6.0);

                                ui.horizontal(|ui| {
                                    let (icon, label, fill, fg) = if is_running {
                                        (PAUSE, language.text("control-pause"), theme.bg_pill, theme.text_primary)
                                    } else {
                                        (PLAY, language.text("control-start"), theme.accent, theme.text_on_accent)
                                    };

                                    if ui.add(
                                        egui::Button::new(RichText::new(format!("{icon} {label}")).size(12.0).color(fg))
                                            .fill(fill)
                                            .corner_radius(6.0)
                                            .min_size(egui::vec2(90.0, 30.0)),
                                    ).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                        if is_running {
                                            timer.pause(now_ms());
                                        } else {
                                            timer.start(now_ms());
                                        }
                                        changed = true;
                                    }

                                    if ui.add(
                                        egui::Button::new(RichText::new(format!("{ARROWS_CLOCKWISE} {}", language.text("control-restart"))).size(12.0).color(theme.text_secondary))
                                            .fill(theme.bg_pill)
                                            .corner_radius(6.0)
                                            .min_size(egui::vec2(80.0, 30.0)),
                                    ).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                        timer.reset();
                                        changed = true;
                                    }

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(
                                            RichText::new(timer.text(now_ms()))
                                                .size(20.0)
                                                .monospace()
                                                .strong()
                                                .color(if is_running { theme.accent } else { theme.text_primary }),
                                        );
                                    });
                                });
                            });
                    }
                    LayerContent::Questions {
                        items,
                        current,
                        revealed,
                    } => {
                        // Sync with live presentation if active on air
                        if let Some(live) = deck
                            .live_design
                            .as_ref()
                            .filter(|live| live.id == scene.id)
                            .or(deck
                                .live_overlay
                                .as_ref()
                                .filter(|live| live.id == scene.id))
                            && let Some(crate::core::design::DesignLayer {
                                content:
                                    LayerContent::Questions {
                                        current: live_current,
                                        revealed: live_revealed,
                                        ..
                                    },
                                ..
                            }) = live.layers.iter().find(|item| item.id == layer.id)
                            {
                                *current = *live_current;
                                *revealed = *live_revealed;
                            }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(6.0);

                        if items.is_empty() {
                            ui.label(
                                RichText::new(crate::ui::i18n::text(ui, "design-this-layer-has-no-questions-yet"))
                                    .size(12.0)
                                    .color(theme.text_muted),
                            );
                        } else {
                            if *current >= items.len() {
                                *current = items.len().saturating_sub(1);
                            }

                            // 1. Painel de Controle Principal (Barra de Navegação e Revelação)
                            egui::Frame::new()
                                .fill(theme.bg_card)
                                .stroke(egui::Stroke::new(1.0, theme.border_default))
                                .corner_radius(8.0)
                                .inner_margin(egui::Margin::same(10))
                                .show(ui, |ui| {
                                    // Cabeçalho da camada + Pill com contador
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(QUESTION).size(15.0).color(theme.accent));
                                        ui.strong(&layer.name);

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            let cur_str = (*current + 1).to_string();
                                            let tot_str = items.len().to_string();
                                            let counter_text = language.format("design-question-position", &[("current", &cur_str), ("total", &tot_str)]);
                                            ui.label(
                                                RichText::new(counter_text)
                                                    .size(11.0)
                                                    .strong()
                                                    .color(theme.accent),
                                            );
                                        });
                                    });

                                    ui.add_space(8.0);

                                    // Barra de botões de navegação e revelação
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 6.0;

                                        let prev_btn = egui::Button::new(RichText::new(CARET_LEFT).size(14.0))
                                            .fill(theme.bg_pill)
                                            .corner_radius(6.0)
                                            .min_size(egui::vec2(34.0, 32.0));
                                        if ui.add_enabled(*current > 0, prev_btn)
                                            .on_hover_text(crate::ui::i18n::text(ui, "design-previous-question"))
                                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                                            .clicked()
                                        {
                                            *current -= 1;
                                            *revealed = false;
                                            changed = true;
                                        }

                                        let (rev_icon, rev_key, rev_fill, rev_fg, rev_stroke) = if *revealed {
                                            (
                                                EYE_SLASH,
                                                "control-hide-answer",
                                                theme.status_warning_bg,
                                                theme.status_warning_fg,
                                                egui::Stroke::new(1.0, theme.status_warning_fg.linear_multiply(0.4)),
                                            )
                                        } else {
                                            (
                                                EYE,
                                                "control-reveal-answer",
                                                theme.accent,
                                                theme.text_on_accent,
                                                egui::Stroke::NONE,
                                            )
                                        };

                                        let rev_btn = egui::Button::new(
                                            RichText::new(format!("{rev_icon}  {}", language.text(rev_key)))
                                                .size(12.5)
                                                .strong()
                                                .color(rev_fg),
                                        )
                                        .fill(rev_fill)
                                        .stroke(rev_stroke)
                                        .corner_radius(6.0)
                                        .min_size(egui::vec2(ui.available_width() - 40.0, 32.0));

                                        if ui.add(rev_btn)
                                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                                            .on_hover_text(if *revealed {
                                                crate::ui::i18n::text(ui, "design-hide-the-answer-from-the-presentation")
                                            } else {
                                                crate::ui::i18n::text(ui, "design-show-the-answer-in-the-presentation")
                                            })
                                            .clicked()
                                        {
                                            *revealed = !*revealed;
                                            changed = true;
                                        }

                                        let next_btn = egui::Button::new(RichText::new(CARET_RIGHT).size(14.0))
                                            .fill(theme.bg_pill)
                                            .corner_radius(6.0)
                                            .min_size(egui::vec2(34.0, 32.0));
                                        if ui.add_enabled(*current + 1 < items.len(), next_btn)
                                            .on_hover_text(crate::ui::i18n::text(ui, "design-next-question"))
                                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                                            .clicked()
                                        {
                                            *current += 1;
                                            *revealed = false;
                                            changed = true;
                                        }
                                    });

                                    ui.add_space(8.0);

                                    // Pergunta atual em destaque
                                    if let Some(active_q) = items.get(*current) {
                                        egui::Frame::new()
                                            .fill(theme.bg_pill)
                                            .stroke(egui::Stroke::new(1.0, theme.border_subtle))
                                            .corner_radius(6.0)
                                            .inner_margin(egui::Margin::symmetric(10, 8))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        RichText::new(crate::ui::i18n::text(ui, "design-on-screen"))
                                                            .size(9.5)
                                                            .strong()
                                                            .color(theme.accent),
                                                    );
                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        if *revealed {
                                                            let visible_msg = crate::ui::i18n::format(ui, "design-answer-visible-on-screen", &[("icon", CHECK_CIRCLE)]);
                                                            ui.label(
                                                                RichText::new(visible_msg)
                                                                    .size(10.0)
                                                                    .color(theme.status_success_fg),
                                                            );
                                                        } else {
                                                            let hidden_msg = crate::ui::i18n::format(ui, "design-answer-hidden-on-screen", &[("icon", EYE_SLASH)]);
                                                            ui.label(
                                                                RichText::new(hidden_msg)
                                                                    .size(10.0)
                                                                    .color(theme.text_muted),
                                                            );
                                                        }
                                                    });
                                                });

                                                ui.add_space(3.0);

                                                ui.label(
                                                    RichText::new(&active_q.question)
                                                        .size(13.0)
                                                        .strong()
                                                        .color(theme.text_primary),
                                                );

                                                if !active_q.answer.is_empty() {
                                                    ui.add_space(4.0);
                                                    let ans_bg = if *revealed {
                                                        theme.status_success_bg
                                                    } else {
                                                        theme.bg_card
                                                    };
                                                    let ans_border = if *revealed {
                                                        egui::Stroke::new(1.0, theme.status_success_fg.linear_multiply(0.3))
                                                    } else {
                                                        egui::Stroke::new(1.0, theme.border_default)
                                                    };

                                                    egui::Frame::new()
                                                        .fill(ans_bg)
                                                        .stroke(ans_border)
                                                        .corner_radius(4.0)
                                                        .inner_margin(egui::Margin::symmetric(8, 6))
                                                        .show(ui, |ui| {
                                                            ui.horizontal_wrapped(|ui| {
                                                                ui.label(
                                                                    RichText::new(crate::ui::i18n::text(ui, "design-answer-variant-3"))
                                                                        .size(11.0)
                                                                        .strong()
                                                                        .color(if *revealed { theme.status_success_fg } else { theme.text_muted }),
                                                                );
                                                                ui.label(
                                                                    RichText::new(&active_q.answer)
                                                                        .size(11.5)
                                                                        .color(theme.text_primary),
                                                                );
                                                            });
                                                        });
                                                }
                                            });
                                    }
                                });

                            ui.add_space(10.0);

                            // 2. Lista de Cards de cada Pergunta
                            ui.horizontal(|ui| {
                                let count_str = items.len().to_string();
                                ui.label(
                                    RichText::new(crate::ui::i18n::format(ui, "design-questions", &[("name", &count_str)]))
                                        .size(11.5)
                                        .strong()
                                        .color(theme.text_secondary),
                                );
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(
                                        RichText::new(crate::ui::i18n::text(ui, "design-click-a-card-to-select"))
                                            .size(10.5)
                                            .color(theme.text_muted),
                                    );
                                });
                            });

                            ui.add_space(4.0);

                            for (index, q) in items.iter().enumerate() {
                                let is_active = index == *current;

                                ui.push_id(index, |ui| {
                                    let bg = if is_active {
                                        theme.bg_card_active
                                    } else {
                                        theme.bg_card
                                    };
                                    let stroke = if is_active {
                                        egui::Stroke::new(1.5, theme.accent)
                                    } else {
                                        egui::Stroke::new(1.0, theme.border_default)
                                    };

                                    let card_frame = egui::Frame::new()
                                        .fill(bg)
                                        .stroke(stroke)
                                        .corner_radius(7.0)
                                        .inner_margin(egui::Margin::symmetric(10, 8));

                                    let card_resp = card_frame.show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = 8.0;

                                            // Número da pergunta
                                            let badge_size = egui::vec2(22.0, 22.0);
                                            let (badge_rect, _) = ui.allocate_exact_size(badge_size, egui::Sense::hover());
                                            let (num_bg, num_fg) = if is_active {
                                                (theme.accent, theme.text_on_accent)
                                            } else {
                                                (theme.bg_pill, theme.text_secondary)
                                            };
                                            ui.painter().rect_filled(badge_rect, 11.0, num_bg);
                                            if !is_active {
                                                ui.painter().rect_stroke(
                                                    badge_rect,
                                                    11.0,
                                                    egui::Stroke::new(1.0, theme.border_default),
                                                    egui::StrokeKind::Inside,
                                                );
                                            }
                                            ui.painter().text(
                                                badge_rect.center(),
                                                egui::Align2::CENTER_CENTER,
                                                format!("{}", index + 1),
                                                egui::FontId::proportional(10.5),
                                                num_fg,
                                            );

                                            // Pergunta e Resposta
                                            ui.vertical(|ui| {
                                                ui.spacing_mut().item_spacing.y = 2.0;

                                                let title_color = if is_active {
                                                    theme.accent
                                                } else {
                                                    theme.text_primary
                                                };
                                                ui.label(
                                                    RichText::new(&q.question)
                                                        .size(12.0)
                                                        .strong()
                                                        .color(title_color),
                                                );

                                                if !q.answer.is_empty() {
                                                    let ans_snippet = if q.answer.chars().count() > 45 {
                                                        format!("{}...", q.answer.chars().take(43).collect::<String>())
                                                    } else {
                                                        q.answer.clone()
                                                    };
                                                    if is_active && *revealed {
                                                        ui.label(
                                                            RichText::new(format!("{CHECK} {ans_snippet}"))
                                                                .size(10.5)
                                                                .color(theme.status_success_fg),
                                                        );
                                                    } else {
                                                        ui.label(
                                                            RichText::new(crate::ui::i18n::format(ui, "design-answer", &[("ans_snippet", &ans_snippet)]))
                                                                .size(10.5)
                                                                .color(theme.text_muted),
                                                        );
                                                    }
                                                }
                                            });

                                            // Lado direito
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if is_active {
                                                    // Toggle rápido no card ativo
                                                    let btn_icon = if *revealed { EYE_SLASH } else { EYE };
                                                    let btn = egui::Button::new(
                                                        RichText::new(btn_icon)
                                                            .size(12.0)
                                                            .color(if *revealed { theme.status_warning_fg } else { theme.accent }),
                                                    )
                                                    .fill(theme.bg_pill)
                                                    .corner_radius(4.0)
                                                    .min_size(egui::vec2(26.0, 24.0));

                                                    if ui.add(btn)
                                                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                                                        .on_hover_text(crate::ui::i18n::text(ui, if *revealed { "control-hide-answer" } else { "control-reveal-answer" }))
                                                        .clicked()
                                                    {
                                                        *revealed = !*revealed;
                                                        changed = true;
                                                    }

                                                    ui.label(
                                                        RichText::new(crate::ui::i18n::text(ui, "design-active"))
                                                            .size(9.0)
                                                            .strong()
                                                            .color(theme.accent),
                                                    );
                                                } else {
                                                    ui.label(
                                                        RichText::new(CARET_RIGHT)
                                                            .size(12.0)
                                                            .color(theme.icon_muted),
                                                    );
                                                }
                                            });
                                        });
                                    });

                                    let interact = card_resp.response.interact(egui::Sense::click());
                                    if interact.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                        if !is_active {
                                            ui.painter().rect_stroke(
                                                card_resp.response.rect,
                                                7.0,
                                                egui::Stroke::new(1.0, theme.border_active),
                                                egui::StrokeKind::Inside,
                                            );
                                        }
                                    }

                                    if interact.clicked() {
                                        *current = index;
                                        *revealed = false;
                                        changed = true;
                                    }
                                });

                                ui.add_space(4.0);
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    if changed {
        for live in [&mut deck.live_design, &mut deck.live_overlay]
            .into_iter()
            .flatten()
            .filter(|live| live.id == scene.id)
        {
            for layer in &scene.layers {
                if let Some(target) = live.layers.iter_mut().find(|item| item.id == layer.id) {
                    target.content = layer.content.clone();
                }
            }
        }
    }
    deck.preview_design = Some(scene);
}
