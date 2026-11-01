use crate::{
    core::{
        bible::fold,
        commands::{Command, CommandGroup},
        preferences::AppSettings,
        shortcuts::{KeyBinding, ShortcutCapture},
    },
    ui::AppTheme,
};
use eframe::egui::{self, RichText};
use egui_phosphor::regular::{ARROW_COUNTER_CLOCKWISE, KEYBOARD, PLUS, X};

pub(super) fn shortcuts(
    ui: &mut egui::Ui,
    draft: &mut AppSettings,
    capture: &mut ShortcutCapture,
    theme: &AppTheme,
) {
    let language = draft.language;
    ui.label(
        RichText::new(language.text("shortcuts-description"))
            .size(12.0)
            .color(theme.text_secondary),
    );
    ui.add_space(12.0);
    if let Some(operator) = draft
        .operators
        .iter_mut()
        .find(|operator| operator.id == draft.active_operator)
    {
        operator.keyboard_shortcuts = true;
    }
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let width = (ui.available_width() - 175.0).max(80.0);
        crate::ui::components::controls::search_field(
            ui,
            "shortcut_query",
            &mut capture.query,
            language.text("shortcuts-filter"),
            width,
            theme,
        );
        if super::components::styled_button(
            ui,
            Some(ARROW_COUNTER_CLOCKWISE),
            language.text("shortcuts-reset"),
            super::components::ButtonTone::Secondary,
            theme,
        )
        .clicked()
        {
            draft.shortcuts.reset();
            capture.command = None;
            capture.error.clear();
        }
    });
    ui.add_space(12.0);
    let query = fold(&capture.query);
    let mut shown = 0;
    for group in CommandGroup::ALL {
        let commands: Vec<_> = Command::ALL
            .into_iter()
            .filter(|command| {
                command.group() == group
                    && (query.is_empty()
                        || fold(language.text(command.label_key())).contains(&query)
                        || draft
                            .shortcuts
                            .bindings(*command)
                            .iter()
                            .any(|binding| fold(&binding.display()).contains(&query)))
            })
            .collect();
        if commands.is_empty() {
            continue;
        }
        shown += commands.len();
        ui.label(
            RichText::new(language.text(group.label_key()))
                .size(11.5)
                .strong()
                .color(theme.text_primary),
        );
        ui.add_space(5.0);
        egui::Frame::new()
            .fill(theme.bg_panel)
            .stroke(egui::Stroke::new(1.0, theme.border_default))
            .corner_radius(10.0)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                for (row, command) in commands.into_iter().enumerate() {
                    if row > 0 {
                        ui.add_space(5.0);
                        ui.separator();
                        ui.add_space(5.0);
                    }
                    ui.push_id(command, |ui| {
                        ui.horizontal_top(|ui| {
                            let label_width = (ui.available_width() * 0.43).clamp(120.0, 230.0);
                            ui.allocate_ui_with_layout(
                                egui::vec2(label_width, 26.0),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.set_min_width(label_width);
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(language.text(command.label_key()))
                                                .size(12.5),
                                        )
                                        .wrap(),
                                    );
                                },
                            );
                            ui.vertical(|ui| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing = egui::vec2(5.0, 5.0);
                                    let bindings = draft.shortcuts.bindings(command);
                                    for (index, binding) in bindings.iter().enumerate() {
                                        egui::Frame::new()
                                            .fill(theme.bg_pill)
                                            .stroke(egui::Stroke::new(1.0, theme.border_default))
                                            .corner_radius(6.0)
                                            .inner_margin(egui::Margin::symmetric(5, 2))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.spacing_mut().item_spacing.x = 2.0;
                                                    if ui
                                                        .add(
                                                            egui::Button::new(
                                                                RichText::new(binding.display())
                                                                    .monospace()
                                                                    .size(11.0),
                                                            )
                                                            .frame(false),
                                                        )
                                                        .on_hover_text(
                                                            language.text("shortcuts-edit"),
                                                        )
                                                        .clicked()
                                                    {
                                                        capture.command = Some(command);
                                                        capture.replace_index = Some(index);
                                                        capture.error.clear();
                                                    }
                                                    if ui
                                                        .add(
                                                            egui::Button::new(
                                                                RichText::new(X)
                                                                    .size(10.0)
                                                                    .color(theme.text_muted),
                                                            )
                                                            .frame(false),
                                                        )
                                                        .on_hover_text(
                                                            language.text("shortcuts-clear"),
                                                        )
                                                        .clicked()
                                                    {
                                                        draft.shortcuts.remove(command, index);
                                                    }
                                                });
                                            });
                                    }
                                    if theme
                                        .icon_button(
                                            ui,
                                            PLUS,
                                            language.text("shortcuts-add"),
                                            false,
                                            egui::Vec2::splat(24.0),
                                        )
                                        .clicked()
                                    {
                                        capture.command = Some(command);
                                        capture.replace_index = None;
                                        capture.error.clear();
                                    }
                                    if bindings.is_empty() {
                                        ui.label(
                                            RichText::new(language.text("shortcuts-unassigned"))
                                                .size(11.0)
                                                .color(theme.text_muted),
                                        );
                                    }
                                    if theme
                                        .icon_button(
                                            ui,
                                            ARROW_COUNTER_CLOCKWISE,
                                            language.text("shortcuts-reset-action"),
                                            false,
                                            egui::Vec2::splat(24.0),
                                        )
                                        .clicked()
                                        && let Err(conflict) = draft.shortcuts.restore(command) {
                                            capture.command = Some(command);
                                            capture.replace_index = None;
                                            capture.error = language.format(
                                                "shortcuts-conflict",
                                                &[("action", language.text(conflict.label_key()))],
                                            );
                                        }
                                });
                            });
                        });
                    });
                }
            });
        ui.add_space(14.0);
    }
    if shown == 0 {
        ui.label(language.text("common-empty"));
    }
    if let Some(command) = capture.command {
        capture_dialog(ui.ctx(), draft, capture, command, theme);
    }
}

fn capture_dialog(
    ctx: &egui::Context,
    draft: &mut AppSettings,
    capture: &mut ShortcutCapture,
    command: Command,
    theme: &AppTheme,
) {
    let language = draft.language;
    if let Some(binding) = ctx.input(|input| input.events.iter().find_map(KeyBinding::from_event)) {
        if binding.key == "Escape" && !binding.ctrl && !binding.alt && !binding.shift {
            capture.command = None;
            capture.error.clear();
            return;
        }
        match draft.shortcuts.add(command, binding, capture.replace_index) {
            Ok(()) => {
                capture.command = None;
                capture.error.clear();
                return;
            }
            Err(conflict) => {
                capture.error = language.format(
                    "shortcuts-conflict",
                    &[("action", language.text(conflict.label_key()))],
                )
            }
        }
    }
    let modal = egui::Modal::new(egui::Id::unique("capture_shortcut")).show(ctx, |ui| {
        ui.set_width(380.0);
        ui.vertical_centered(|ui| {
            ui.add_space(12.0);
            ui.label(RichText::new(KEYBOARD).size(32.0).color(theme.accent));
            ui.add_space(8.0);
            ui.heading(language.text("shortcuts-capture"));
            ui.label(RichText::new(language.text(command.label_key())).color(theme.text_secondary));
            ui.add_space(16.0);
            ui.label(language.text("shortcuts-hint"));
            if !capture.error.is_empty() {
                ui.colored_label(theme.status_danger, &capture.error);
            }
            ui.add_space(16.0);
            if super::components::styled_button(
                ui,
                None,
                language.text("common-cancel"),
                super::components::ButtonTone::Secondary,
                theme,
            )
            .clicked()
            {
                capture.command = None;
            }
            ui.add_space(8.0);
        });
    });
    if modal.should_close() {
        capture.command = None;
        capture.error.clear();
    }
}
