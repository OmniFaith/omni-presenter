use crate::ui::theme::AppTheme;
use eframe::egui::{self, RichText, Vec2};

pub fn configure(ui: &mut egui::Ui, theme: &AppTheme) {
    let style = ui.style_mut();
    style.spacing.item_spacing = egui::vec2(6.0, 8.0);
    style.spacing.button_padding = egui::vec2(9.0, 6.0);
    style.spacing.interact_size.y = 28.0;
    theme.apply_to_visuals(&mut style.visuals);
    style.visuals.widgets.inactive.weak_bg_fill = theme.bg_pill;
    style.visuals.widgets.hovered.weak_bg_fill = theme.bg_card_hover;
    style.visuals.widgets.active.weak_bg_fill = theme.accent_soft;
    for widget in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
    ] {
        widget.corner_radius = egui::CornerRadius::same(7);
    }
}

pub fn caption(ui: &mut egui::Ui, label: &str, theme: &AppTheme) {
    let label = crate::ui::i18n::text(ui, label);
    ui.label(
        RichText::new(label)
            .size(11.5)
            .strong()
            .color(theme.text_secondary),
    );
}
pub fn note(ui: &mut egui::Ui, text: &str, theme: &AppTheme) {
    let text = crate::ui::i18n::text(ui, text);
    ui.add(egui::Label::new(RichText::new(text).size(11.0).color(theme.text_secondary)).wrap());
}
pub fn segments(
    ui: &mut egui::Ui,
    labels: &[&str],
    selected: usize,
    theme: &AppTheme,
) -> Option<usize> {
    segments_height(ui, labels, selected, 34.0, theme)
}

pub fn segments_height(
    ui: &mut egui::Ui,
    labels: &[&str],
    selected: usize,
    height: f32,
    theme: &AppTheme,
) -> Option<usize> {
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), height), egui::Sense::hover());
    ui.painter().rect_filled(rect, 8.0, theme.bg_app);
    let width = rect.width() / labels.len() as f32;
    let mut changed = None;
    for (index, label) in labels.iter().enumerate() {
        let segment = egui::Rect::from_min_size(
            rect.min + egui::vec2(index as f32 * width, 0.0),
            egui::vec2(width, rect.height()),
        )
        .shrink(3.0);
        let response = ui.interact(
            segment,
            ui.make_persistent_id(("design_segment", labels, index)),
            egui::Sense::click(),
        );
        if selected == index || response.hovered() {
            ui.painter().rect_filled(
                segment,
                6.0,
                if selected == index {
                    theme.bg_card_active
                } else {
                    theme.bg_card_hover
                },
            );
            if selected == index {
                ui.painter().rect_stroke(
                    segment,
                    6.0,
                    egui::Stroke::new(1.0, theme.border_active),
                    egui::StrokeKind::Inside,
                );
            }
        }
        ui.painter().text(
            segment.center(),
            egui::Align2::CENTER_CENTER,
            crate::ui::i18n::text(ui, label),
            egui::FontId::proportional(11.5),
            if selected == index {
                theme.accent
            } else {
                theme.text_secondary
            },
        );
        if response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            changed = Some(index);
        }
    }
    changed
}

pub fn empty(
    ui: &mut egui::Ui,
    icon: &str,
    title: &str,
    description: &str,
    theme: &AppTheme,
) {
    egui::Frame::new()
        .fill(theme.bg_app)
        .corner_radius(10.0)
        .inner_margin(16)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.label(RichText::new(icon).size(27.0).color(theme.icon_muted));
                ui.add_space(4.0);
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, title))
                        .size(12.5)
                        .strong()
                        .color(theme.text_primary),
                );
                note(ui, description, theme);
                ui.add_space(10.0);
            });
        });
}

#[cfg(test)]
mod tests {
    #[test]
    fn inspector_segments_with_shared_labels_have_distinct_widget_ids() {
        let ctx = eframe::egui::Context::default();
        for _ in 0..2 {
            let mut output = ctx.run_ui(eframe::egui::RawInput::default(), |ui| {
                let theme = crate::ui::theme::AppTheme::dark();
                super::segments(ui, &["Slide", "Camada"], 0, &theme);
                super::segments(ui, &["Slide", "Extra"], 0, &theme);
            });
            assert!(!output.shapes.iter().any(|shape|matches!(&shape.shape,eframe::egui::Shape::Text(text) if text.galley.job.text.contains("use of widget"))));
            output.textures_delta.clear();
        }
    }
}
