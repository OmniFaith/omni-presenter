use crate::ui::theme::AppTheme;
use eframe::egui::{self, RichText};

/// Shared library empty state. Title/hint are message IDs; action is localized.
pub fn show(
    ui: &mut egui::Ui,
    icon: &str,
    title: &str,
    hint: &str,
    action: &str,
    theme: &AppTheme,
) -> egui::Response {
    let language = crate::ui::i18n::language(ui.ctx());
    ui.vertical_centered(|ui| {
        ui.add_space(28.0);
        ui.label(
            RichText::new(icon)
                .size(32.0)
                .color(theme.text_secondary.linear_multiply(0.4)),
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new(language.text(title))
                .size(12.5)
                .strong()
                .color(theme.text_primary),
        );
        ui.add_space(2.0);
        ui.label(
            RichText::new(language.text(hint))
                .size(10.5)
                .color(theme.text_secondary),
        );
        ui.add_space(10.0);
        ui.add(
            egui::Button::new(RichText::new(action).size(11.0))
                .fill(theme.bg_card)
                .stroke(egui::Stroke::new(1.0, theme.border_default))
                .corner_radius(6.0),
        )
    })
    .inner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_empty_action_is_centered_in_narrow_and_wide_panels() {
        for size in [egui::vec2(280.0, 400.0), egui::vec2(640.0, 500.0)] {
            for theme in [AppTheme::light(), AppTheme::dark()] {
                let ctx = egui::Context::default();
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                        ..Default::default()
                    },
                    |ui| {
                        let center = ui.available_rect_before_wrap().center().x;
                        let response = show(
                            ui,
                            egui_phosphor::regular::STACK,
                            "common-empty",
                            "design-your-designs-will-appear-here-automatically",
                            "+ Create design",
                            &theme,
                        );
                        assert!((response.rect.center().x - center).abs() < 1.0);
                        assert!(response.rect.right() <= ui.max_rect().right());
                    },
                );
                output.textures_delta.clear();
            }
        }
    }
}
