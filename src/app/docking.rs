use crate::core::layout::{ContainerKind, calculate_drop_zone};
use crate::core::state::SidebarPage;
use eframe::egui;

use super::PresentationApp;

impl PresentationApp {
    /// Handles cross-container drag & drop docking and paints the drop feedback.
    pub(super) fn handle_docking(
        &mut self,
        ui: &mut egui::Ui,
        rendered_leaf_rects: &[(ContainerKind, egui::Rect)],
    ) {
        let ctx = ui.ctx().clone();
        let theme = &self.control_state.theme;
        let pointer_pos = ctx.input(|i| i.pointer.interact_pos());
        let any_released = ctx.input(|i| i.pointer.any_released());

        let Some(payload) =
            egui::DragAndDrop::payload::<crate::ui::components::drag_drop::DragPayload>(&ctx)
        else {
            return;
        };
        let crate::ui::components::drag_drop::DragSource::Panel(dragged) = payload.source else {
            return;
        };

        if let Some(pos) = pointer_pos {
            for (target_kind, target_rect) in rendered_leaf_rects {
                if *target_kind == dragged {
                    continue;
                }
                let Some((zone, indicator_rect, label)) = calculate_drop_zone(pos, *target_rect)
                else {
                    continue;
                };

                let label = self.control_state.settings.language.text(label);
                ui.painter()
                    .rect_filled(indicator_rect, 6.0, theme.accent.linear_multiply(0.1));
                ui.painter().rect_stroke(
                    indicator_rect,
                    6.0,
                    egui::Stroke::new(2.0, theme.accent),
                    egui::StrokeKind::Inside,
                );

                let translated_label = crate::ui::i18n::text(ui, label);
                let font_id = egui::FontId::proportional(12.0);
                let badge_w = (translated_label.len() as f32 * 7.5 + 24.0).max(120.0);
                let badge_rect = egui::Rect::from_center_size(
                    indicator_rect.center(),
                    egui::vec2(badge_w, 28.0),
                );
                ui.painter().rect_filled(badge_rect, 14.0, theme.bg_panel);
                ui.painter().rect_stroke(
                    badge_rect,
                    14.0,
                    egui::Stroke::new(1.0, theme.accent),
                    egui::StrokeKind::Inside,
                );
                ui.painter().text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    translated_label,
                    font_id,
                    theme.text_primary,
                );

                if any_released {
                    match self.control_state.active_sidebar_page {
                        SidebarPage::Bible => {
                            self.control_state
                                .bible_layout
                                .dock(dragged, *target_kind, zone);
                        }
                        SidebarPage::Chat => {
                            self.control_state
                                .chat_layout
                                .dock(dragged, *target_kind, zone);
                        }
                        SidebarPage::Home | SidebarPage::Design => {
                            self.control_state.layout.dock(dragged, *target_kind, zone);
                        }
                    }
                }
                break;
            }
        }

        if any_released {
            egui::DragAndDrop::clear_payload(&ctx);
        }
    }
}
