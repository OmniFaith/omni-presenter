use crate::ui::modals::settings::components::ButtonTone;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, Rect, RichText, Vec2};
use egui_phosphor::regular::{ARROW_SQUARE_OUT, COPY};
use qrcode::QrCode;

const ACTIVE_QR_DIALOG_ID: &str = "active_qr_modal_state";

/// Renders a QR code with sharp modules and a clean white quiet zone card.
/// Black modules on white background ensures 100% reliable detection by smartphone cameras.
pub fn render_qr_code(ui: &mut egui::Ui, data: &str, size: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
    let painter = ui.painter();

    let Ok(code) = QrCode::new(data.as_bytes()) else {
        painter.rect_filled(rect, 8.0, Color32::from_gray(30));
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "QR Error",
            egui::FontId::proportional(11.0),
            Color32::WHITE,
        );
        return response;
    };

    // Card background with rounded corners
    painter.rect_filled(rect, 8.0, Color32::WHITE);

    let code_width = code.width();
    let quiet_zone = 2; // modules on each side
    let total_modules = (code_width + quiet_zone * 2) as f32;
    let module_size = size / total_modules;

    let start_x = rect.min.x + (quiet_zone as f32) * module_size;
    let start_y = rect.min.y + (quiet_zone as f32) * module_size;

    for y in 0..code_width {
        for x in 0..code_width {
            if code[(x, y)] == qrcode::Color::Dark {
                let mod_rect = Rect::from_min_size(
                    egui::pos2(
                        start_x + (x as f32) * module_size,
                        start_y + (y as f32) * module_size,
                    ),
                    Vec2::splat(module_size.ceil()),
                );
                painter.rect_filled(mod_rect, 0.0, Color32::BLACK);
            }
        }
    }

    response
}

/// Renders a QR code inside a styled card with optional helper text.
pub fn qr_card(
    ui: &mut egui::Ui,
    data: &str,
    size: f32,
    label: Option<&str>,
    theme: &AppTheme,
) {
    ui.vertical_centered(|ui| {
        egui::Frame::new()
            .fill(Color32::WHITE)
            .corner_radius(10.0)
            .inner_margin(6)
            .stroke(egui::Stroke::new(1.0, theme.border_default))
            .show(ui, |ui| {
                render_qr_code(ui, data, size);
            });

        if let Some(lbl) = label {
            ui.add_space(4.0);
            ui.label(
                RichText::new(lbl)
                    .size(10.5)
                    .color(theme.text_muted),
            );
        }
    });
}

/// Triggers opening the global QR code dialog with the given title and URL.
pub fn open_qr_dialog(ctx: &egui::Context, title: impl Into<String>, url: impl Into<String>) {
    ctx.data_mut(|d| {
        d.insert_temp(
            egui::Id::unique(ACTIVE_QR_DIALOG_ID),
            (title.into(), url.into()),
        );
    });
}

/// Renders the active QR modal if one has been triggered via `open_qr_dialog`.
pub fn render_active_qr_dialog(ctx: &egui::Context, theme: &AppTheme) {
    let current: Option<(String, String)> = ctx.data_mut(|d| {
        d.get_temp::<(String, String)>(egui::Id::unique(ACTIVE_QR_DIALOG_ID))
    });

    if let Some((title, url)) = current {
        let mut open = true;
        show_qr_dialog(
            ctx,
            "global_qr_viewer",
            &title,
            "qr-scan-with-phone-camera",
            &url,
            &mut open,
            theme,
        );
        if !open {
            ctx.data_mut(|d| {
                d.remove_temp::<(String, String)>(egui::Id::unique(ACTIVE_QR_DIALOG_ID));
            });
        }
    }
}

/// Displays an app dialog with a prominent QR code for the given URL.
pub fn show_qr_dialog(
    ctx: &egui::Context,
    id: &str,
    title: &str,
    subtitle: &str,
    url: &str,
    is_open: &mut bool,
    theme: &AppTheme,
) {
    if !*is_open {
        return;
    }

    let language = crate::ui::i18n::language(ctx);
    let close = crate::ui::components::dialog::show(
        ctx,
        id,
        title,
        subtitle,
        360.0,
        theme,
        |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(4.0);
                egui::Frame::new()
                    .fill(Color32::WHITE)
                    .corner_radius(12.0)
                    .inner_margin(8)
                    .stroke(egui::Stroke::new(1.0, theme.border_default))
                    .show(ui, |ui| {
                        render_qr_code(ui, url, 200.0);
                    });

                ui.add_space(10.0);

                // URL Display pill
                egui::Frame::new()
                    .fill(theme.bg_pill)
                    .corner_radius(6.0)
                    .inner_margin(egui::vec2(10.0, 7.0))
                    .stroke(egui::Stroke::new(1.0, theme.border_default))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(url)
                                .monospace()
                                .size(11.5)
                                .strong()
                                .color(theme.accent),
                        );
                    });

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if crate::ui::modals::settings::components::styled_button(
                        ui,
                        Some(COPY),
                        language.text("workspace-copy-link"),
                        ButtonTone::Secondary,
                        theme,
                    )
                    .clicked()
                    {
                        ui.ctx().copy_text(url.to_owned());
                    }

                    if crate::ui::modals::settings::components::styled_button(
                        ui,
                        Some(ARROW_SQUARE_OUT),
                        language.text("control-open"),
                        ButtonTone::Primary,
                        theme,
                    )
                    .clicked()
                    {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(url.to_owned()));
                    }
                });
            });
        },
    );

    if close {
        *is_open = false;
    }
}
