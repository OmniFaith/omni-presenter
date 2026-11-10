use crate::{
    core::{search::SearchCache, state::ControlPanelState},
    services::chat::ChatService,
};
use eframe::egui::{self, RichText, Vec2};

pub(super) fn emoji_picker(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    service: &ChatService,
) {
    if !state.chat.show_emoji {
        return;
    }
    let tokens = state.theme.clone();
    let language = state.settings.language;
    let screen = ctx.content_rect();
    let anchor = state
        .chat
        .emoji_anchor
        .unwrap_or(egui::Rect::from_min_size(screen.center(), Vec2::ZERO));
    let y = if anchor.top() - 365.0 >= screen.top() + 10.0 {
        anchor.top() - 365.0
    } else if anchor.bottom() + 365.0 <= screen.bottom() - 10.0 {
        anchor.bottom() + 4.0
    } else {
        (anchor.top() - 365.0).max(screen.top() + 10.0)
    };
    let position = egui::pos2(
        (anchor.right() - 350.0).clamp(
            screen.left() + 10.0,
            (screen.right() - 358.0).max(screen.left() + 10.0),
        ),
        y,
    );
    let id = egui::Id::unique("emoji_search_cache");
    let mut cache = ctx
        .data(|data| data.get_temp::<SearchCache>(id))
        .unwrap_or_default();
    let mut selected = None;
    let response = egui::Area::new(egui::Id::unique("chat_emoji_picker"))
        .order(egui::Order::Foreground)
        .fixed_pos(position)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(tokens.bg_panel)
                .stroke(egui::Stroke::new(1.0, tokens.border_default))
                .corner_radius(12.0)
                .inner_margin(14)
                .shadow(egui::epaint::Shadow {
                    offset: [0, 8],
                    blur: 24,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(30),
                })
                .show(ui, |ui| {
                    ui.set_width(320.0);
                    ui.spacing_mut().button_padding = egui::vec2(3.0, 3.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(if state.chat.reacting_to.is_some() {
                                language.text("emoji-react")
                            } else {
                                language.text("services-emojis")
                            })
                            .size(14.0)
                            .strong(),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if tokens
                                .icon_button(
                                    ui,
                                    egui_phosphor::regular::X,
                                    language.text("emoji-close"),
                                    false,
                                    Vec2::splat(24.0),
                                )
                                .clicked()
                            {
                                state.chat.show_emoji = false;
                            }
                        });
                    });
                    if state.chat.reacting_to.is_some() {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 3.0;
                            for emoji in crate::core::emoji::QUICK_REACTIONS {
                                if emoji_button(ui, emoji, &tokens).clicked() {
                                    selected = Some(emoji.to_owned());
                                }
                            }
                        });
                    }
                    crate::ui::components::controls::search_field(
                        ui,
                        "emoji_query",
                        &mut state.chat.emoji_search,
                        language.text("emoji-search"),
                        320.0,
                        &tokens,
                    );
                    let catalogue = crate::core::emoji::catalogue();
                    ui.horizontal(|ui| {
                        for (index, category) in catalogue.iter().enumerate() {
                            if ui
                                .add_sized(
                                    Vec2::new(44.0, 30.0),
                                    egui::Button::new(RichText::new(category.icon).size(20.0))
                                        .fill(if state.chat.emoji_category == index {
                                            tokens.accent_soft
                                        } else {
                                            tokens.bg_panel
                                        })
                                        .stroke(egui::Stroke::NONE)
                                        .corner_radius(8.0),
                                )
                                .on_hover_text(category.label(language))
                                .clicked()
                            {
                                state.chat.emoji_category = index;
                                state.chat.emoji_search.clear();
                            }
                        }
                    });
                    ui.separator();
                    let all = crate::core::emoji::all();
                    let matches = cache.filter(
                        &state.chat.emoji_search,
                        0,
                        all.iter().map(|emoji| emoji.normalized.as_str()),
                    );
                    let results: Vec<_> = matches
                        .iter()
                        .map(|index| all[*index])
                        .filter(|emoji| {
                            !state.chat.emoji_search.trim().is_empty()
                                || emoji.category
                                    == state.chat.emoji_category.min(catalogue.len() - 1)
                        })
                        .collect();
                    if state.chat.emoji_search.trim().is_empty() {
                        crate::ui::components::controls::section_heading(
                            ui,
                            catalogue[state.chat.emoji_category.min(catalogue.len() - 1)]
                                .label(language),
                            &tokens,
                        );
                    }
                    if results.is_empty() {
                        ui.label(language.text("emoji-empty"));
                    }
                    egui::ScrollArea::vertical()
                        .id_salt("emoji_catalogue")
                        .max_height(if state.chat.reacting_to.is_some() {
                            185.0
                        } else {
                            225.0
                        })
                        .show_rows(ui, 39.0, results.len().div_ceil(8), |ui, rows| {
                            for row in rows {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 3.0;
                                    for emoji in results.iter().skip(row * 8).take(8) {
                                        if emoji_button(ui, emoji.value, &tokens)
                                            .on_hover_text(emoji.label(language))
                                            .clicked()
                                        {
                                            selected = Some(emoji.value.into());
                                        }
                                    }
                                });
                            }
                        });
                });
        });
    ctx.data_mut(|data| data.insert_temp(id, cache));
    if let Some(emoji) = selected {
        if let Some(message) = state.chat.reacting_to.take() {
            if let Err(error) = service.react(&state.chat.token, message, &emoji) {
                state.chat.error = error;
            }
        } else {
            state.chat.input.push_str(&emoji);
        }
        state.chat.show_emoji = false;
    }
    if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        || (ctx.cumulative_frame_nr() > state.chat.emoji_opened_frame
            && ctx.input(|input| {
                input.pointer.any_click()
                    && input
                        .pointer
                        .interact_pos()
                        .is_some_and(|point| !response.response.rect.contains(point))
            }))
    {
        state.chat.show_emoji = false;
        state.chat.reacting_to = None;
    }
}

fn emoji_button(ui: &mut egui::Ui, emoji: &str, theme: &crate::ui::AppTheme) -> egui::Response {
    ui.add_sized(
        Vec2::splat(36.0),
        egui::Button::new(RichText::new(emoji).size(23.0))
            .fill(theme.bg_panel)
            .stroke(egui::Stroke::NONE)
            .corner_radius(8.0),
    )
}
