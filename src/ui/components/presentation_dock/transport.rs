use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, FontId, Margin, Stroke, Vec2};
use egui_phosphor::regular::{CARET_LEFT, CARET_RIGHT, SKIP_BACK, SKIP_FORWARD};

use super::BTN_H;

/// Editable field that jumps the live presentation to a specific strophe number.
pub(crate) fn jump_input(
    ui: &mut egui::Ui,
    tokens: &AppTheme,
    deck: &mut SlideDeck,
    state: &mut ControlPanelState,
) {
    let is_song = deck.is_live_song();
    let total = deck.total_slides();
    let is_blank = deck.current_slide_text().trim().is_empty() && deck.current_index() + 1 == total;
    let current = if is_song {
        if deck.current_index() == 0 {
            "T".to_string()
        } else if is_blank {
            "—".to_string()
        } else {
            deck.current_index().to_string()
        }
    } else if total > 0 {
        if is_blank {
            "—".to_string()
        } else {
            (deck.current_index() + 1).to_string()
        }
    } else {
        "0".to_string()
    };

    let id = ui.make_persistent_id("dock_jump_input");
    let mut should_select_all = false;

    // Check if the jump shortcut was pressed while this field wasn't focused yet,
    // or if requested globally via state.focus_dock_jump_input.
    let shortcut_pressed = state
        .settings
        .shortcuts
        .pressed(ui.ctx(), crate::core::commands::Command::JumpSlide);

    if shortcut_pressed || state.focus_dock_jump_input {
        state.focus_dock_jump_input = false;
        state
            .settings
            .shortcuts
            .consume(ui.ctx(), crate::core::commands::Command::JumpSlide);
        // Also consume any matching text event (e.g. "/", "'") so it isn't typed into the box
        ui.ctx().input_mut(|input| {
            input.events.retain(|event| {
                !matches!(event, egui::Event::Text(t) if t == "\\" || t == "/" || t == "'" || t == "\"")
            });
        });
        ui.memory_mut(|m| m.request_focus(id));
        should_select_all = true;
        ui.ctx().request_repaint();
    }

    let focused = ui.ctx().memory(|m| m.has_focus(id)) || should_select_all;
    let mut buffer = if focused {
        ui.ctx()
            .data_mut(|d| d.get_temp::<String>(id))
            .unwrap_or_else(|| current.clone())
    } else {
        current
    };

    let frame = egui::Frame::new()
        .fill(if focused {
            tokens.bg_card_active
        } else {
            tokens.bg_panel
        })
        .stroke(Stroke::new(
            1.0,
            if focused {
                tokens.accent
            } else {
                tokens.border_subtle
            },
        ))
        .corner_radius(tokens.radius_sm)
        .inner_margin(Margin::symmetric(4, 2));

    let language = state.settings.language;
    let bindings = state
        .settings
        .shortcuts
        .bindings(crate::core::commands::Command::JumpSlide);
    let shortcut_hint = if bindings.is_empty() {
        String::new()
    } else {
        let keys = bindings
            .iter()
            .map(|b| b.display())
            .collect::<Vec<_>>()
            .join(language.text("control-or"));
        language.format("control-shortcut-hint", &[("keys", &keys)])
    };
    let jump_hint = language.format("control-jump-hint", &[("shortcut", &shortcut_hint)]);

    let response = ui
        .add_sized(
            Vec2::new(34.0, BTN_H),
            egui::TextEdit::singleline(&mut buffer)
                .id(id)
                .align(egui::Align2::CENTER_CENTER)
                .font(FontId::proportional(13.0))
                .text_color(if focused {
                    tokens.accent
                } else {
                    tokens.text_primary
                })
                .frame(frame),
        )
        .on_hover_text(jump_hint);

    // Keep only numeric characters, 't'/'T', and '-'/'—'/'b'
    buffer.retain(|c| c.is_ascii_digit() || c.eq_ignore_ascii_case(&'t') || c == '-' || c == '—' || c.eq_ignore_ascii_case(&'b'));

    if response.clicked() || response.gained_focus() || should_select_all {
        if let Some(mut edit) = egui::TextEdit::load_state(ui.ctx(), id) {
            let char_count = buffer.chars().count();
            edit.cursor.set_char_range(Some(egui::text::CCursorRange::two(
                egui::text::CCursor::new(0),
                egui::text::CCursor::new(char_count),
            )));
            edit.store(ui.ctx(), id);
        }
        ui.ctx().request_repaint();
    }

    let enter_pressed = (focused || response.lost_focus())
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));

    if enter_pressed {
        let trimmed = buffer.trim();
        if is_song {
            if trimmed.eq_ignore_ascii_case("t") || trimmed == "0" {
                deck.select_live_slide(0);
            } else if trimmed == "-" || trimmed == "—" || trimmed.eq_ignore_ascii_case("b") {
                deck.select_live_slide(total.saturating_sub(1));
            } else if let Ok(n) = trimmed.parse::<usize>() {
                if n < total {
                    deck.select_live_slide(n);
                } else if n <= total && total > 0 {
                    deck.select_live_slide(total - 1);
                }
            }
        } else if trimmed == "-" || trimmed == "—" || trimmed.eq_ignore_ascii_case("b") {
            deck.select_live_slide(total.saturating_sub(1));
        } else if let Ok(n) = trimmed.parse::<usize>() {
            if n >= 1 && n <= total {
                deck.select_live_slide(n - 1);
            }
        }
        crate::services::media::sync_live_media_playback(state, deck, false);
        ui.ctx().data_mut(|d| d.remove_temp::<String>(id));
        ui.memory_mut(|m| m.surrender_focus(id));
        ui.ctx().request_repaint();
    } else if response.lost_focus() {
        ui.ctx().data_mut(|d| d.remove_temp::<String>(id));
    } else {
        ui.ctx().data_mut(|d| d.insert_temp(id, buffer));
    }
}

/// First / previous / jump / next / last navigation shared by the dock and the
/// floating preview window.
pub(crate) fn transport_controls(
    ui: &mut egui::Ui,
    tokens: &AppTheme,
    deck: &mut SlideDeck,
    state: &mut ControlPanelState,
    is_presentation_open: &mut bool,
) {
    let language = state.settings.language;
    let is_song = deck.is_live_song();
    let has_prev = deck.current_index() > 0;
    let is_last = deck.total_slides() > 0 && deck.current_index() + 1 >= deck.total_slides();
    let has_next = (deck.total_slides() > 0 && deck.current_index() + 1 < deck.total_slides())
        || (*is_presentation_open && is_last);
    let (first, previous, next, last) = if is_song {
        (
            "control-first-strophe",
            "control-previous-strophe",
            "control-next-strophe",
            "control-last-strophe",
        )
    } else {
        (
            "control-first-slide",
            "control-previous-slide",
            "control-next-slide",
            "control-last-slide",
        )
    };

    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.spacing_mut().item_spacing = egui::vec2(3.0, 0.0);

        if ui
            .add_enabled_ui(has_prev, |ui| {
                tokens.icon_button(ui, SKIP_BACK, language.text(first), false, Vec2::splat(BTN_H))
            })
            .inner
            .clicked()
        {
            deck.first_slide();
            crate::services::media::sync_live_media_playback(state, deck, *is_presentation_open);
        }
        if ui
            .add_enabled_ui(has_prev, |ui| {
                tokens.icon_button(
                    ui,
                    CARET_LEFT,
                    language.text(previous),
                    false,
                    Vec2::splat(BTN_H),
                )
            })
            .inner
            .clicked()
        {
            deck.prev_slide();
            crate::services::media::sync_live_media_playback(state, deck, *is_presentation_open);
        }

        jump_input(ui, tokens, deck, state);

        let next_tooltip = if is_last && *is_presentation_open {
            language.text("control-close-presentation")
        } else {
            language.text(next)
        };

        if ui
            .add_enabled_ui(has_next, |ui| {
                tokens.icon_button(ui, CARET_RIGHT, next_tooltip, false, Vec2::splat(BTN_H))
            })
            .inner
            .clicked()
        {
            if !deck.next_slide() && *is_presentation_open {
                *is_presentation_open = false;
            }
            crate::services::media::sync_live_media_playback(state, deck, *is_presentation_open);
        }
        if ui
            .add_enabled_ui(deck.total_slides() > 0 && deck.current_index() + 1 < deck.total_slides(), |ui| {
                tokens.icon_button(ui, SKIP_FORWARD, language.text(last), false, Vec2::splat(BTN_H))
            })
            .inner
            .clicked()
        {
            deck.last_slide();
            crate::services::media::sync_live_media_playback(state, deck, *is_presentation_open);
        }
    });
}
