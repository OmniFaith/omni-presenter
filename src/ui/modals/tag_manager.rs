use crate::core::preferences::StropheTagDefinition;
use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use crate::ui::components::{color_picker, dialog};
use eframe::egui::{self, Color32, RichText, Stroke};
use egui_phosphor::regular::{PLUS, TRASH};

pub fn render_tag_manager_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
) {
    if !state.show_tag_manager_modal {
        return;
    }

    let tokens = state.theme.clone();
    let language = state.settings.language;
    let mut should_close = false;
    let mut delete_tag_idx = None;
    let mut add_new_tag = false;

    let close = dialog::show(
        ctx,
        "tag_manager_modal",
        language.text("workspace-tag-manage"),
        language.text("workspace-tag-manage-desc"),
        420.0,
        &tokens,
        |ui| {
            ui.label(
                RichText::new(language.text("workspace-tag-available"))
                    .size(12.0)
                    .strong()
                    .color(tokens.text_primary),
            );
            ui.add_space(4.0);

            // Lista de tags existentes
            egui::Frame::new()
                .fill(tokens.bg_pill)
                .stroke(Stroke::new(1.0, tokens.border_default))
                .corner_radius(8.0)
                .inner_margin(egui::Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if state.settings.strophe_tags.is_empty() {
                        ui.label(
                            RichText::new(language.text("workspace-tag-empty"))
                                .size(11.5)
                                .color(tokens.text_muted),
                        );
                    } else {
                        let total = state.settings.strophe_tags.len();
                        for i in 0..total {
                            let tag = &mut state.settings.strophe_tags[i];
                            ui.horizontal(|ui| {
                                // Color picker swatch button
                                let id_salt = format!("tag_color_{i}_{}", tag.name);
                                color_picker::color_swatch_u8_3(ui, id_salt, &mut tag.color, &tokens);

                                let display_name = tag.localized_name(language);
                                ui.label(
                                    RichText::new(display_name)
                                        .size(12.5)
                                        .strong()
                                        .color(Color32::from_rgb(tag.color[0], tag.color[1], tag.color[2])),
                                );

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let del_btn = egui::Button::new(
                                        RichText::new(TRASH).size(13.0).color(Color32::from_rgb(239, 68, 68)),
                                    )
                                    .fill(Color32::TRANSPARENT);
                                    if ui.add(del_btn).on_hover_text(language.text("workspace-tag-delete-tooltip")).clicked() {
                                        delete_tag_idx = Some(i);
                                    }
                                });
                            });
                            if i + 1 < total {
                                ui.separator();
                            }
                        }
                    }
                });

            ui.add_space(10.0);
            ui.label(
                RichText::new(language.text("workspace-tag-new"))
                    .size(12.0)
                    .strong()
                    .color(tokens.text_primary),
            );
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                // Color swatch picker for the new tag
                color_picker::color_swatch_u8_3(ui, "new_tag_color", &mut state.tag_manager_new_color, &tokens);

                let edit = ui.add(
                    egui::TextEdit::singleline(&mut state.tag_manager_new_name)
                        .hint_text(language.text("workspace-tag-name-hint"))
                        .desired_width(ui.available_width() - 90.0),
                );

                let can_add = !state.tag_manager_new_name.trim().is_empty();
                let add_btn = egui::Button::new(
                    RichText::new(format!("{} {}", PLUS, language.text("workspace-tag-add")))
                        .size(11.5)
                        .color(if can_add { tokens.text_on_accent } else { tokens.text_muted }),
                )
                .fill(if can_add { tokens.accent } else { tokens.bg_pill })
                .corner_radius(6.0);

                if ui.add_enabled(can_add, add_btn).clicked()
                    || (edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && can_add)
                {
                    add_new_tag = true;
                }
            });

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if dialog::action(ui, crate::ui::i18n::text(ui, "common-close"), true, &tokens).clicked() {
                        should_close = true;
                    }
                });
            });
        },
    );

    if let Some(idx) = delete_tag_idx {
        if idx < state.settings.strophe_tags.len() {
            state.settings.strophe_tags.remove(idx);
            if let Some(db) = db {
                let _ = db.save_settings(&state.settings);
            }
        }
    }

    if add_new_tag {
        let name = state.tag_manager_new_name.trim().to_string();
        if !name.is_empty() {
            let color = state.tag_manager_new_color;
            state.settings.strophe_tags.push(StropheTagDefinition::new(name, color));
            state.tag_manager_new_name.clear();
            if let Some(db) = db {
                let _ = db.save_settings(&state.settings);
            }
        }
    }

    if close || should_close {
        state.show_tag_manager_modal = false;
        if let Some(db) = db {
            let _ = db.save_settings(&state.settings);
        }
    }
}
