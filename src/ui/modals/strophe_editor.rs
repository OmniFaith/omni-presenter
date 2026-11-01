use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use eframe::egui::{self, Color32, RichText, Stroke, StrokeKind, Vec2};
use egui_phosphor::regular::{FLOPPY_DISK, PENCIL_SIMPLE, X};

pub fn render_strophe_editor_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    slide_deck: &mut SlideDeck,
    db: Option<&AppDb>,
) {
    if !state.strophe_editor.open {
        return;
    }

    let screen_rect = ctx.input(|i| i.viewport_rect());
    let mut should_save = false;
    let mut should_cancel = false;
    let mut open_full_editor = false;

    let modal_id = egui::Id::unique("strophe_editor_modal_overlay");

    egui::Area::new(modal_id)
        .order(egui::Order::Foreground)
        .fixed_pos(screen_rect.min)
        .show(ctx, |ui| {
            // Tecla ESC cancela
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                should_cancel = true;
            }

            // 1. Backdrop escurecido semi-transparente
            let _ = ui.allocate_rect(screen_rect, egui::Sense::click_and_drag());
            ui.painter().rect_filled(screen_rect, 0.0, Color32::from_black_alpha(120));

            // 2. Diálogo centralizado
            let modal_w = 540.0f32.min(screen_rect.width() - 40.0);
            let modal_h = 360.0f32.min(screen_rect.height() - 40.0);
            let modal_rect = egui::Rect::from_center_size(screen_rect.center(), Vec2::new(modal_w, modal_h));

            // Sombra suave
            let ambient_shadow = egui::epaint::Shadow {
                offset: [0, 10],
                blur: 24,
                spread: 0,
                color: Color32::from_black_alpha(60),
            };
            ui.painter().add(ambient_shadow.as_shape(modal_rect, 10.0));

            // Cartão modal branco
            ui.painter().rect_filled(modal_rect, 10.0, state.theme.bg_card);
            ui.painter().rect_stroke(
                modal_rect,
                10.0,
                Stroke::new(1.0, state.theme.border_default),
                StrokeKind::Inside,
            );

            let inner_rect = modal_rect.shrink(16.0);
            let mut content_ui = ui.new_child(egui::UiBuilder::new().max_rect(inner_rect));

            content_ui.vertical(|ui| {
                // Header com título e botão fechar
                ui.horizontal(|ui| {
                    let title = crate::ui::i18n::format(
                        ui,
                        "workspace-edit-strophe-title",
                        &[
                            ("number", &(state.strophe_editor.index + 1).to_string()),
                            ("title", slide_deck.title()),
                        ],
                    );
                    ui.label(RichText::new(title).size(15.0).strong().color(state.theme.text_primary));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let close_btn = egui::Button::new(RichText::new(X).size(14.0))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE);
                        if ui.add(close_btn).clicked() {
                            should_cancel = true;
                        }
                    });
                });

                ui.add_space(4.0);
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "workspace-strophe-editor-hint"))
                        .size(11.5)
                        .color(Color32::from_rgb(100, 116, 139)),
                );

                ui.add_space(8.0);

                // Campo multiline para editar o texto da estrofe
                let tags_ref = &state.settings.strophe_tags;
                let tokens_ref = &state.theme;
                let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
                    let job = crate::ui::components::lyrics_highlighter::highlight_lyrics_job(
                        buf.as_str(),
                        tags_ref,
                        tokens_ref,
                        13.5,
                        wrap_width,
                        false,
                    );
                    ui.fonts_mut(|f| f.layout_job(job))
                };
                let edit_res = ui.add(
                    egui::TextEdit::multiline(&mut state.strophe_editor.text)
                        .desired_rows(8)
                        .desired_width(f32::INFINITY)
                        .font(egui::FontId::proportional(13.5))
                        .margin(egui::Margin::symmetric(10, 8))
                        .layouter(&mut layouter),
                );

                // Auto-focar no campo se acabou de abrir
                if ui.memory(|m| m.focused().is_none()) {
                    edit_res.request_focus();
                }

                // Atalho Ctrl+Enter para salvar
                if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter)) {
                    should_save = true;
                }

                ui.add_space(12.0);

                // Barra de ações na base
                ui.horizontal(|ui| {
                    let save_btn = egui::Button::new(
                        RichText::new(format!("{} {}", FLOPPY_DISK, crate::ui::i18n::text(ui, "workspace-save-strophe")))
                            .size(13.0)
                            .strong()
                            .color(state.theme.text_on_accent),
                    )
                    .fill(state.theme.accent)
                    .corner_radius(6.0)
                    .min_size(Vec2::new(130.0, 32.0));

                    if ui.add(save_btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        should_save = true;
                    }

                    let cancel_btn = egui::Button::new(
                        RichText::new(crate::ui::i18n::text(ui, "common-cancel"))
                            .size(13.0)
                            .color(state.theme.text_secondary),
                    )
                    .fill(state.theme.bg_pill)
                    .stroke(Stroke::new(1.0, state.theme.border_default))
                    .corner_radius(6.0)
                    .min_size(Vec2::new(80.0, 32.0));

                    if ui.add(cancel_btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        should_cancel = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let full_edit_btn = egui::Button::new(
                            RichText::new(format!("{} {}", PENCIL_SIMPLE, crate::ui::i18n::text(ui, "workspace-full-editor")))
                                .size(12.0)
                                .color(Color32::from_rgb(75, 85, 99)),
                        )
                        .fill(Color32::TRANSPARENT);

                        if ui.add(full_edit_btn).on_hover_text(crate::ui::i18n::text(ui, "workspace-open-full-editor-tooltip")).clicked() {
                            open_full_editor = true;
                        }
                    });
                });
            });
        });

    if should_save {
        slide_deck.update_slide_text(state.strophe_editor.index, &state.strophe_editor.text);
        if let Err(error) = slide_deck.save_current_song_text(db) {
            state.notify(state.settings.language.format("workspace-could-not-save", &[("error", &error.to_string())]));
            return;
        }
        state.strophe_editor.open = false;
    } else if should_cancel {
        state.strophe_editor.open = false;
    } else if open_full_editor {
        state.song_editor.title = slide_deck.title().to_owned();
        state.song_editor.artist = slide_deck.artist().to_owned();
        state.song_editor.text = slide_deck.raw_text().to_owned();
        state.song_editor.folder = slide_deck
            .current_song()
            .map(|s| s.folder.clone())
            .unwrap_or_default();
        state.song_editor.tags = slide_deck
            .current_song()
            .map(|s| s.tags.clone())
            .unwrap_or_default();
        state.song_editor.open = true;
        state.strophe_editor.open = false;
    }
}
