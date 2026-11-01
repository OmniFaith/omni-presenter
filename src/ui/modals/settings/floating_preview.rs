use crate::core::state::{ControlPanelState, FloatingPreviewTarget};
use crate::render::paint_theme_bg;
use crate::services::monitor::MonitorInfo;
use eframe::egui::{self, Color32, RichText, Stroke};
use egui_phosphor::regular::{ARROW_SQUARE_OUT, CLOCK, COPY, DESKTOP, QR_CODE, TELEVISION_SIMPLE};

/// Renders the floating screen preview window ("programinha flutuante de visualização").
pub fn render_floating_screen_preview(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    monitors: &[MonitorInfo],
    slide_text: Option<&str>,
    slide_title: Option<&str>,
    next_slide_text: Option<&str>,
    output_theme: Option<&crate::core::models::PresentationTheme>,
) {
    let (target, mut aspect_ratio) = match &state.settings_ui.floating_preview {
        Some(ps) if ps.open => (ps.target.clone(), ps.aspect_ratio),
        _ => return,
    };

    let language = state.settings.language;
    let theme = state.theme.clone();
    let mut is_open = true;

    // Resolve target details
    let (window_title, is_virtual, is_enabled, custom_theme_id, custom_bg, lyrics_only, show_clock, show_title, show_next_preview, _show_comments, font_scale, monitor_info_str, web_url) =
        match &target {
            FloatingPreviewTarget::Physical { monitor_name, index } => {
                let mon = monitors.iter().find(|m| &m.name == monitor_name);
                let res_str = if let Some(m) = mon {
                    format!("{} · {} × {}", monitor_name, m.resolution.0, m.resolution.1)
                } else {
                    format!("{} · 1920 × 1080", monitor_name)
                };
                let bg = if state.settings.use_wallpaper_in_presentation {
                    Some(state.settings.wallpaper_for_monitor(monitor_name).clone())
                } else {
                    None
                };
                (
                    language.format(
                        "settings-preview-monitor",
                        &[("number", &index.to_string()), ("name", monitor_name)],
                    ),
                    false,
                    state.settings.physical_output_enabled,
                    None,
                    bg,
                    false,
                    false,
                    false,
                    false,
                    false,
                    100,
                    res_str,
                    None,
                )
            }
            FloatingPreviewTarget::Virtual { id, name } => {
                let screen = state
                    .settings
                    .screens
                    .iter()
                    .find(|s| &s.id == id)
                    .cloned()
                    .unwrap_or_default();
                let host = if state.settings.allow_lan {
                    &state.settings_ui.lan_address
                } else {
                    "127.0.0.1"
                };
                let url_slug = crate::core::preferences::screen_url_slug(&screen, &state.settings.screens);
                let url = state.settings_ui.discovery_url
                    .strip_suffix("/control")
                    .filter(|_| state.settings.allow_lan)
                    .map(|base| format!("{base}/screen/{url_slug}"))
                    .unwrap_or_else(|| crate::services::web::http_url(
                        host, state.settings.server_port, &format!("/screen/{url_slug}"),
                    ));
                (
                    language.format(
                        "settings-preview-virtual",
                        &[("name", if screen.name.is_empty() { name } else { &screen.name })],
                    ),
                    true,
                    screen.enabled,
                    screen.custom_theme_id,
                    screen.custom_background,
                    screen.lyrics_only,
                    screen.show_clock,
                    screen.show_title,
                    screen.show_next_preview,
                    screen.show_comments,
                    screen.font_scale,
                    language.format("settings-virtual-obs-web", &[("name", &screen.font_scale.to_string())]),
                    Some(url),
                )
            }
        };

    let active_theme = if let Some(ref tid) = custom_theme_id {
        state
            .themes
            .iter()
            .find(|t| &t.id == tid)
            .cloned()
            .unwrap_or_else(|| state.live_theme().clone())
    } else {
        output_theme.unwrap_or_else(|| state.live_theme()).clone()
    };

    let aspect = aspect_ratio.clamp(0.5, 3.5);

    let viewport_id = egui::ViewportId::from_hash_of("screen_preview_native_window");
    let builder = egui::ViewportBuilder::default()
        .with_title(&window_title)
        .with_inner_size([720.0, 480.0])
        .with_min_inner_size([380.0, 260.0])
        .with_decorations(true)
        .with_resizable(true)
        .with_active(true);

    ctx.show_viewport_immediate(viewport_id, builder, |ctx, _class| {
        if ctx.input(|i| i.viewport().close_requested()) {
            is_open = false;
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme.bg_card).inner_margin(12))
            .show(ctx, |ui| {

        // Top Toolbar
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            // Status indicator pill
            let (status_bg, status_color, status_text) = if is_enabled {
                (
                    Color32::from_rgba_unmultiplied(34, 197, 94, 35),
                    Color32::from_rgb(34, 197, 94),
                    language.text("workspace-live"),
                )
            } else {
                (theme.bg_pill, theme.text_muted, language.text("workspace-inactive"))
            };

            egui::Frame::new()
                .fill(status_bg)
                .corner_radius(10.0)
                .inner_margin(egui::vec2(8.0, 3.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 5.0;
                        let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(6.0, 6.0), egui::Sense::hover());
                        ui.painter().circle_filled(dot_rect.center(), 3.0, status_color);
                        ui.label(RichText::new(status_text).size(11.0).strong().color(status_color));
                    });
                });

            // Target type icon & label
            let type_icon = if is_virtual {
                TELEVISION_SIMPLE
            } else {
                DESKTOP
            };
            ui.label(
                RichText::new(format!("{type_icon} {}", if is_virtual { language.text("workspace-virtual") } else { language.text("workspace-physical") }))
                    .size(11.5)
                    .color(theme.text_secondary),
            );

            // Right-aligned controls
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Aspect ratio presets
                for (ratio_val, ratio_label) in [(16.0 / 9.0, "16:9"), (4.0 / 3.0, "4:3")] {
                    let is_sel = (aspect_ratio - ratio_val).abs() < 0.05;
                    if ui
                        .selectable_label(is_sel, RichText::new(ratio_label).size(10.5))
                        .clicked()
                    {
                        aspect_ratio = ratio_val;
                    }
                }
            });
        });

        ui.add_space(4.0);

        // Screen Canvas Display Area
        let available = ui.available_size() - egui::vec2(0.0, 32.0);
        let avail_w = available.x.max(120.0);
        let avail_h = available.y.max(80.0);

        // Calculate aspect-fit rectangle
        let target_w = avail_w.min(avail_h * aspect);
        let target_h = target_w / aspect;
        let (canvas_rect, _) = ui.allocate_exact_size(
            egui::vec2(avail_w, target_h),
            egui::Sense::hover(),
        );

        let screen_rect = egui::Rect::from_center_size(
            canvas_rect.center(),
            egui::vec2(target_w, target_h),
        );

        // Outer bezel / background
        ui.painter().rect_filled(canvas_rect, 6.0, Color32::from_rgb(12, 12, 14));

        // Screen background
        if is_virtual && lyrics_only {
            // Transparency checkerboard grid (for OBS / stream overlays)
            paint_transparency_checkerboard(ui.painter(), screen_rect);
        } else if let Some(ref bg) = custom_bg {
            crate::ui::presentation::paint_wallpaper(
                ui.painter(),
                screen_rect,
                bg,
                None,
                &state.themes,
            );
        } else {
            // Theme background
            paint_theme_bg(ui.painter(), screen_rect, &active_theme, 0.0);
        }

        // Inner border stroke
        ui.painter().rect_stroke(
            screen_rect,
            0.0,
            Stroke::new(1.0, Color32::from_black_alpha(100)),
            egui::StrokeKind::Inside,
        );

        // Text & Slide Projection
        let text_to_show = slide_text
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| crate::ui::i18n::text(ui, "settings-for-god-so-loved-the-world-that-he-gave-his-label"));

        let clean_text = crate::core::slides::strip_comments(text_to_show);

        // Render slide effects / typography with font scale
        let _scale_factor = (font_scale as f32 / 100.0).clamp(0.5, 2.0);
        let anim_id = egui::Id::unique("floating_preview_anim");
        crate::services::effects::render(
            ui.painter(),
            screen_rect,
            &active_theme,
            &clean_text,
            anim_id,
            false,
        );

        // Virtual Screen Overlays (Stage clock, title, next slide preview)
        if is_virtual {
            if show_title {
                let title_text = slide_title.unwrap_or_else(|| crate::ui::i18n::text(ui, "settings-john-3-16"));
                let banner_rect = egui::Rect::from_min_size(
                    screen_rect.left_top() + egui::vec2(8.0, 8.0),
                    egui::vec2(160.0, 20.0),
                );
                ui.painter().rect_filled(banner_rect, 4.0, Color32::from_black_alpha(160));
                ui.painter().text(
                    banner_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    title_text,
                    egui::FontId::proportional(11.0),
                    Color32::WHITE,
                );
            }

            if show_clock {
                let now = chrono::Local::now();
                let time_str = now.format("%H:%M:%S").to_string();
                let clock_rect = egui::Rect::from_min_size(
                    screen_rect.right_top() + egui::vec2(-84.0, 8.0),
                    egui::vec2(76.0, 20.0),
                );
                ui.painter().rect_filled(clock_rect, 4.0, Color32::from_black_alpha(160));
                ui.painter().text(
                    clock_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{CLOCK} {time_str}"),
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(250, 204, 21),
                );
            }

            if show_next_preview {
                let next_text = next_slide_text.unwrap_or_else(|| crate::ui::i18n::text(ui, "settings-next-verse-17"));
                let footer_rect = egui::Rect::from_min_size(
                    screen_rect.left_bottom() + egui::vec2(8.0, -24.0),
                    egui::vec2(screen_rect.width() - 16.0, 18.0),
                );
                ui.painter().rect_filled(footer_rect, 3.0, Color32::from_black_alpha(170));
                ui.painter().text(
                    footer_rect.left_center() + egui::vec2(8.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    format!("» {next_text}"),
                    egui::FontId::proportional(10.5),
                    Color32::from_rgb(203, 213, 225),
                );
            }
        }

        ui.add_space(4.0);

        // Bottom Info & Quick Links Bar
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            ui.label(
                RichText::new(&monitor_info_str)
                    .size(11.0)
                    .color(theme.text_secondary),
            );

            if let Some(ref url) = web_url {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button(format!("{ARROW_SQUARE_OUT} {}", language.text("workspace-open")))
                        .on_hover_text(language.text("workspace-open-in-browser"))
                        .clicked()
                    {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(url.clone()));
                    }

                    if ui
                        .small_button(format!("{COPY} {}", language.text("workspace-copy-link")))
                        .on_hover_text(language.text("workspace-copy-url-to-clipboard-label"))
                        .clicked()
                    {
                        ui.ctx().copy_text(url.clone());
                    }

                    if ui
                        .small_button(format!("{QR_CODE} QR"))
                        .on_hover_text("Mostrar QR Code para escanear com o celular")
                        .clicked()
                    {
                        crate::ui::components::open_qr_dialog(
                            ui.ctx(),
                            "Prévia Web",
                            url.clone(),
                        );
                    }
                });
            }
        });
    });
    });

    if !is_open {
        state.settings_ui.floating_preview = None;
    } else if let Some(ref mut ps) = state.settings_ui.floating_preview {
        ps.aspect_ratio = aspect_ratio;
    }
}

/// Paints a subtle dark checkerboard grid showing transparency (OBS style).
fn paint_transparency_checkerboard(painter: &egui::Painter, rect: egui::Rect) {
    painter.rect_filled(rect, 0.0, Color32::from_rgb(24, 24, 27));
    let cell_size = 14.0_f32;
    let color_alt = Color32::from_rgb(39, 39, 42);

    let start_x = rect.min.x;
    let start_y = rect.min.y;
    let cols = (rect.width() / cell_size).ceil() as usize;
    let rows = (rect.height() / cell_size).ceil() as usize;

    let clipped = painter.with_clip_rect(rect);
    for row in 0..rows {
        for col in 0..cols {
            if (row + col) % 2 == 1 {
                let cell_rect = egui::Rect::from_min_size(
                    egui::pos2(start_x + col as f32 * cell_size, start_y + row as f32 * cell_size),
                    egui::vec2(cell_size, cell_size),
                );
                clipped.rect_filled(cell_rect, 0.0, color_alt);
            }
        }
    }
}
