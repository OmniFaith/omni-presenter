use crate::core::state::ControlPanelState;
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use egui_phosphor::regular::*;

pub fn render_welcome_profile_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    db: Option<&crate::services::db::AppDb>,
) {
    if !state.show_profile_picker_modal {
        return;
    }

    if state.settings.operators.len() <= 1 {
        state.show_profile_picker_modal = false;
        return;
    }

    let theme = state.theme.clone();
    let language = state.settings.language;
    let mut should_enter = false;

    let frame = egui::Frame::new()
        .fill(theme.bg_panel)
        .stroke(Stroke::new(1.0, theme.border_default))
        .corner_radius(16.0)
        .inner_margin(24)
        .shadow(egui::epaint::Shadow {
            offset: [0, 16],
            blur: 48,
            spread: 0,
            color: Color32::from_black_alpha(85),
        });

    let modal = egui::Modal::new(egui::Id::unique("welcome_profile_picker_modal"))
        .frame(frame)
        .backdrop_color(Color32::from_black_alpha(150))
        .show(ctx, |ui| {
            ui.set_width(500.0_f32.min((ctx.content_rect().width() - 40.0).max(300.0)));
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 12.0);

            // Cabeçalho limpo e direto
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(language.text("workspace-who-is-operating-today"))
                        .size(20.0)
                        .strong()
                        .color(theme.text_primary),
                );
                ui.add_space(2.0);
                ui.label(
                    RichText::new(language.text("workspace-who-is-operating-today-desc"))
                        .size(12.0)
                        .color(theme.text_secondary),
                );
            });

            ui.add_space(10.0);

            // Lista / Grid de Perfis de Operadores
            let operators = state.settings.operators.clone();
            let active_id = state.settings.active_operator.clone();

            let card_w = 154.0;
            let card_h = 168.0;
            let avatar_size = 72.0;

            egui::ScrollArea::vertical()
                .id_salt("welcome_profiles_scroll")
                .max_height(280.0)
                .show(ui, |ui| {
                    let avail_w = ui.available_width();
                    let cols = ((avail_w + 12.0) / (card_w + 12.0)).floor().max(1.0) as usize;

                    for chunk in operators.chunks(cols) {
                        let row_count = chunk.len();
                        let row_w = row_count as f32 * card_w + (row_count.saturating_sub(1)) as f32 * 12.0;
                        let pad = ((avail_w - row_w) / 2.0).max(0.0);

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(12.0, 12.0);
                            if pad > 0.0 {
                                ui.add_space(pad);
                            }

                            for p in chunk {
                                let is_active = p.id == active_id;
                                let (rect, resp) = ui.allocate_exact_size(
                                    egui::vec2(card_w, card_h),
                                    egui::Sense::click(),
                                );

                                if resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }

                                if resp.clicked() {
                                    state.switch_operator(&p.id);
                                }
                                if resp.double_clicked() {
                                    state.switch_operator(&p.id);
                                    should_enter = true;
                                }

                                let hover = ui.ctx().animate_bool(resp.id.with("hov"), resp.hovered());

                                let bg = if is_active {
                                    theme.bg_card_hover.lerp_to_gamma(theme.accent_soft, 0.45)
                                } else if hover > 0.0 {
                                    theme.bg_card_hover
                                } else {
                                    theme.bg_pill
                                };

                                let stroke_color = if is_active {
                                    theme.accent
                                } else if hover > 0.0 {
                                    theme.border_active
                                } else {
                                    theme.border_default
                                };

                                ui.painter().rect_filled(rect, 14.0, bg);
                                ui.painter().rect_stroke(
                                    rect,
                                    14.0,
                                    Stroke::new(if is_active { 2.0 } else { 1.0 }, stroke_color),
                                    egui::StrokeKind::Inside,
                                );

                                // Avatar / Foto do Perfil com anel exterior
                                let avatar_center = egui::pos2(rect.center().x, rect.top() + 52.0);
                                let avatar_rect = egui::Rect::from_center_size(
                                    avatar_center,
                                    Vec2::splat(avatar_size),
                                );

                                let ring_stroke = if is_active {
                                    Stroke::new(2.5, theme.accent)
                                } else if hover > 0.0 {
                                    Stroke::new(1.5, theme.border_active)
                                } else {
                                    Stroke::new(1.0, theme.border_default)
                                };
                                ui.painter().circle_stroke(avatar_center, avatar_size / 2.0 + 4.0, ring_stroke);

                                crate::ui::components::avatar::paint(
                                    ui,
                                    avatar_rect,
                                    p.avatar.as_deref(),
                                    USER,
                                    &theme,
                                    is_active,
                                );

                                // Nome do operador
                                let name_y = avatar_rect.bottom() + 20.0;
                                ui.painter().text(
                                    egui::pos2(rect.center().x, name_y),
                                    egui::Align2::CENTER_CENTER,
                                    &p.name,
                                    egui::FontId::new(14.0, egui::FontFamily::Name("inter_bold".into())),
                                    if is_active { theme.accent } else { theme.text_primary },
                                );

                                // Indicador de perfil ativo
                                if is_active {
                                    ui.painter().text(
                                        egui::pos2(rect.center().x, name_y + 16.0),
                                        egui::Align2::CENTER_CENTER,
                                        language.text("workspace-profile-active"),
                                        egui::FontId::proportional(11.0),
                                        theme.accent,
                                    );
                                }
                            }
                        });
                        ui.add_space(8.0);
                    }
                });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            // Botão de Entrar corrigido e centralizado (com contraste perfeito)
            ui.vertical_centered(|ui| {
                let active_name = state.settings.operator().name.clone();
                let enter_btn = egui::Button::new(
                    RichText::new(language.format(
                        "workspace-enter-as-operator",
                        &[("name", &active_name)],
                    ))
                    .size(13.0)
                    .strong()
                    .color(theme.text_on_accent),
                )
                .fill(theme.accent)
                .corner_radius(8.0)
                .min_size(Vec2::new(220.0, 36.0));

                if ui
                    .add(enter_btn)
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                    || ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    should_enter = true;
                }

                ui.add_space(2.0);
                ui.label(
                    RichText::new(language.text("workspace-double-click-or-enter-to-open"))
                        .size(11.0)
                        .color(theme.text_muted),
                );
            });
        });

    if should_enter || modal.should_close() {
        state.show_profile_picker_modal = false;
        if let Some(db) = db {
            let _ = db.save_settings(&state.settings);
        }
    }
}
