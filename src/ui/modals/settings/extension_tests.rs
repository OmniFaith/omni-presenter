use super::render_settings_modal;
use crate::core::state::{ControlPanelState, SettingsTab};
use eframe::egui;

#[test]
fn theme_gallery_and_plugin_panels_fit_the_settings_window() {
    for width in [850.0, 1120.0, 1600.0] {
        for tab in [SettingsTab::Interface, SettingsTab::Plugins] {
            let ctx = egui::Context::default();
            let mut state = ControlPanelState::default();
            state.settings_ui.open = true;
            state.settings_ui.tab = tab;
            let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, 900.0));
            let right = screen.center().x + 980.0_f32.min(width - 32.0) * 0.5;
            for _ in 0..3 {
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(screen),
                        ..Default::default()
                    },
                    |ui| {
                        render_settings_modal(ui.ctx(), &mut state, &[], None);
                    },
                );
                for shape in &output.shapes {
                    if let egui::Shape::Text(text) = &shape.shape {
                        if [
                            "Noite azul",
                            "Sálvia",
                            "Lavanda",
                            "Areia",
                            "Alto contraste",
                            "Stage Tools",
                            "Text Tools",
                            "Ativar este plugin",
                        ]
                        .contains(&text.galley.job.text.as_str())
                        {
                            assert!(
                                text.pos.x + text.galley.size().x <= right - 12.0,
                                "text outside settings at width {width}: {}",
                                text.galley.job.text
                            );
                            assert!(
                                shape.clip_rect.right() <= right,
                                "settings content was not clipped"
                            );
                        }
                    }
                }
                output.textures_delta.clear();
            }
        }
    }
}
