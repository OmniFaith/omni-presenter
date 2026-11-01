use crate::ui::theme::AppTheme;
use eframe::egui::{self, RichText};

/// Modal matching the app's floating panels: no native window chrome or resize handle.
pub fn show(
    ctx: &egui::Context,
    id: &str,
    title: &str,
    subtitle: &str,
    width: f32,
    tokens: &AppTheme,
    contents: impl FnOnce(&mut egui::Ui),
) -> bool {
    let language = crate::ui::i18n::language(ctx);
    let title = language.text(title);
    let subtitle = language.text(subtitle);
    let mut close = false;
    let frame = egui::Frame::new()
        .fill(tokens.bg_panel)
        .stroke(egui::Stroke::new(1.0, tokens.border_default))
        .corner_radius(14.0)
        .inner_margin(24)
        .shadow(egui::epaint::Shadow {
            offset: [0, 12],
            blur: 40,
            spread: 0,
            color: egui::Color32::from_black_alpha(45),
        });
    let response = egui::Modal::new(egui::Id::unique("app_dialog").with(id))
        .frame(frame)
        .backdrop_color(egui::Color32::from_black_alpha(95))
        .show(ctx, |ui| {
            ui.set_width(width.min((ctx.content_rect().width() - 80.0).max(180.0)));
            let visuals = &mut ui.style_mut().visuals;
            visuals.widgets.inactive.weak_bg_fill = tokens.bg_pill;
            visuals.widgets.inactive.bg_fill = tokens.bg_pill;
            visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, tokens.border_default);
            visuals.widgets.hovered.weak_bg_fill = tokens.bg_card_hover;
            visuals.widgets.hovered.bg_fill = tokens.bg_card_hover;
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);
            ui.spacing_mut().button_padding = egui::vec2(12.0, 8.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(title)
                        .size(20.0)
                        .strong()
                        .color(tokens.text_primary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if tokens
                        .icon_button(
                            ui,
                            egui_phosphor::regular::X,
                            language.text("common-close"),
                            false,
                            egui::Vec2::splat(28.0),
                        )
                        .clicked()
                    {
                        close = true;
                    }
                });
            });
            if !subtitle.is_empty() {
                ui.label(
                    RichText::new(subtitle)
                        .size(12.0)
                        .color(tokens.text_secondary),
                );
            }
            ui.add_space(10.0);
            contents(ui);
        });
    close || response.should_close()
}

pub fn action(ui: &mut egui::Ui, label: &str, primary: bool, tokens: &AppTheme) -> egui::Response {
    let language = crate::ui::i18n::language(ui.ctx());
    super::controls::text_button(
        ui,
        language.text(label),
        if primary {
            super::controls::ButtonTone::Primary
        } else {
            super::controls::ButtonTone::Neutral
        },
        tokens,
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextInputResponse {
    Submitted(String),
    Cancelled,
    Open,
}

/// Shows an inline text input dialog with save/cancel buttons.
pub fn text_input_dialog(
    ctx: &egui::Context,
    id: &str,
    title: &str,
    placeholder: &str,
    initial: &str,
    tokens: &AppTheme,
) -> TextInputResponse {
    let language = crate::ui::i18n::language(ctx);
    let title = language.text(title);
    let placeholder = language.text(placeholder);

    let memory_id = egui::Id::unique("app_text_input_state").with(id);
    let mut text = ctx.data_mut(|d| {
        d.get_temp::<String>(memory_id)
            .unwrap_or_else(|| initial.to_owned())
    });

    let mut confirmed = false;
    let mut closed = false;

    let response = egui::Modal::new(egui::Id::unique("app_text_input").with(id))
        .frame(
            egui::Frame::new()
                .fill(tokens.bg_panel)
                .stroke(egui::Stroke::new(1.0, tokens.border_default))
                .corner_radius(14.0)
                .inner_margin(24)
                .shadow(egui::epaint::Shadow {
                    offset: [0, 12],
                    blur: 40,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(45),
                }),
        )
        .backdrop_color(egui::Color32::from_black_alpha(95))
        .show(ctx, |ui| {
            ui.set_width(ui.ctx().content_rect().width().min((ctx.content_rect().width() - 80.0).max(180.0)));
            let visuals = &mut ui.style_mut().visuals;
            visuals.widgets.inactive.weak_bg_fill = tokens.bg_pill;
            visuals.widgets.inactive.bg_fill = tokens.bg_pill;
            visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, tokens.border_default);
            visuals.widgets.hovered.weak_bg_fill = tokens.bg_card_hover;
            visuals.widgets.hovered.bg_fill = tokens.bg_card_hover;
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);
            ui.spacing_mut().button_padding = egui::vec2(12.0, 8.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(title)
                        .size(20.0)
                        .strong()
                        .color(tokens.text_primary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if tokens
                        .icon_button(
                            ui,
                            egui_phosphor::regular::X,
                            language.text("common-close"),
                            false,
                            egui::Vec2::splat(28.0),
                        )
                        .clicked()
                    {
                        closed = true;
                    }
                });
            });
            if !placeholder.is_empty() {
                ui.label(
                    RichText::new(placeholder)
                        .size(12.0)
                        .color(tokens.text_secondary),
                );
            }
            ui.add_space(8.0);
            let text_edit = ui.add(
                egui::TextEdit::singleline(&mut text)
                    .hint_text(placeholder)
                    .desired_width(ui.available_width()),
            );
            if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                confirmed = true;
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(RichText::new(language.text("common-cancel")))
                        .clicked()
                    {
                        closed = true;
                    }
                    ui.add_space(8.0);
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(language.text("common-save"))
                                    .strong()
                                    .color(tokens.text_on_accent),
                            )
                            .fill(tokens.accent)
                            .stroke(egui::Stroke::NONE)
                            .corner_radius(8.0)
                            .min_size(egui::Vec2::new(80.0, 34.0)),
                        )
                        .clicked()
                    {
                        confirmed = true;
                    }
                });
            });
        });

    let closed = closed || response.should_close();

    if confirmed {
        ctx.data_mut(|d| d.remove_temp::<String>(memory_id));
        TextInputResponse::Submitted(text)
    } else if closed {
        ctx.data_mut(|d| d.remove_temp::<String>(memory_id));
        TextInputResponse::Cancelled
    } else {
        ctx.data_mut(|d| d.insert_temp(memory_id, text));
        TextInputResponse::Open
    }
}
