use crate::ui::AppTheme;
use eframe::egui::{self, Color32, Margin, RichText, Stroke, StrokeKind, Vec2};
use egui_phosphor::regular::*;

pub fn parse_hex_color(input: &str) -> Option<Color32> {
    let s = input.trim().trim_start_matches('#');
    match s.len() {
        3 => {
            let r = u8::from_str_radix(&s[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&s[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&s[2..3].repeat(2), 16).ok()?;
            Some(Color32::from_rgb(r, g, b))
        }
        4 => {
            let r = u8::from_str_radix(&s[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&s[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&s[2..3].repeat(2), 16).ok()?;
            let a = u8::from_str_radix(&s[3..4].repeat(2), 16).ok()?;
            Some(Color32::from_rgba_unmultiplied(r, g, b, a))
        }
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some(Color32::from_rgb(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            let a = u8::from_str_radix(&s[6..8], 16).ok()?;
            Some(Color32::from_rgba_unmultiplied(r, g, b, a))
        }
        _ => None,
    }
}

pub fn color_swatch_button(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    color: &mut Color32,
    show_alpha: bool,
    theme: &AppTheme,
) -> bool {
    color_swatch_button_sized(ui, id_salt, color, show_alpha, Vec2::new(76.0, 22.0), None, theme)
}

pub fn color_swatch_button_tooltip(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    color: &mut Color32,
    show_alpha: bool,
    tooltip: &str,
    theme: &AppTheme,
) -> bool {
    color_swatch_button_sized(
        ui,
        id_salt,
        color,
        show_alpha,
        Vec2::new(76.0, 22.0),
        Some(tooltip),
        theme,
    )
}

pub fn color_swatch_button_sized(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    color: &mut Color32,
    show_alpha: bool,
    btn_size: Vec2,
    tooltip: Option<&str>,
    theme: &AppTheme,
) -> bool {
    let mut changed = false;
    let (rect, mut resp) = ui.allocate_exact_size(btn_size, egui::Sense::click());

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let hex_text = if show_alpha && color.a() < 255 {
        format!("#{:02X}{:02X}{:02X}{:02X}", color.r(), color.g(), color.b(), color.a())
    } else {
        format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
    };

    resp = if let Some(tt) = tooltip {
        resp.on_hover_text(format!("{tt} ({hex_text})"))
    } else {
        resp.on_hover_text(format!("{}: {hex_text}", crate::ui::i18n::text(ui, "common-color")))
    };

    if show_alpha && color.a() < 255 {
        ui.painter().rect_filled(rect, 5.0, Color32::from_gray(28));
        let step = 6.0;
        let mut x = rect.min.x;
        let mut row = 0;
        while x < rect.max.x {
            let mut y = rect.min.y;
            let mut col = 0;
            while y < rect.max.y {
                if (row + col) % 2 == 0 {
                    let tile = egui::Rect::from_min_max(
                        egui::pos2(x, y),
                        egui::pos2((x + step).min(rect.max.x), (y + step).min(rect.max.y)),
                    );
                    ui.painter().rect_filled(tile, 0.0, Color32::from_gray(48));
                }
                y += step;
                col += 1;
            }
            x += step;
            row += 1;
        }
    }

    let solid_c = if show_alpha {
        *color
    } else {
        Color32::from_rgb(color.r(), color.g(), color.b())
    };

    ui.painter().rect_filled(rect, 5.0, solid_c);

    let stroke_c = if resp.hovered() {
        theme.border_active
    } else {
        theme.border_default
    };
    ui.painter().rect_stroke(rect, 5.0, Stroke::new(1.0, stroke_c), StrokeKind::Inside);

    if btn_size.x >= 54.0 {
        let lum = 0.299 * (color.r() as f32) + 0.587 * (color.g() as f32) + 0.114 * (color.b() as f32);
        let text_c = if lum > 140.0 {
            Color32::from_rgb(20, 24, 30)
        } else {
            Color32::WHITE
        };

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &hex_text,
            egui::FontId::monospace(10.0),
            text_c,
        );
    }

    let popup_id = ui.make_persistent_id(id_salt);

    egui::Popup::menu(&resp)
        .id(popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            let popup_w = 230.0;
            ui.set_width(popup_w);
            ui.spacing_mut().slider_width = popup_w - 20.0;

            // 1. Hash Hex input row
            let state_id = popup_id.with("hex_popup_state");
            let hex_edit_id = popup_id.with("hex_edit_text");
            let is_hex_focused = ui.ctx().memory(|m| m.has_focus(hex_edit_id));

            let mut hex_input = ui
                .data_mut(|d| d.get_temp::<String>(state_id))
                .unwrap_or_default();

            // When user is not actively typing, always keep hex string synced with current color
            if !is_hex_focused || hex_input.is_empty() {
                hex_input = if show_alpha && color.a() < 255 {
                    format!("{:02X}{:02X}{:02X}{:02X}", color.r(), color.g(), color.b(), color.a())
                } else {
                    format!("{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
                };
            }

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;

                // Unified Hex Input Box with perfectly centered prefix and text
                let input_w = 142.0;
                let input_h = 28.0;

                let stroke_c = if is_hex_focused {
                    theme.accent
                } else {
                    theme.border_default
                };

                let prefix_c = if is_hex_focused {
                    theme.accent
                } else {
                    theme.text_secondary
                };

                let frame = egui::Frame::new()
                    .fill(theme.bg_pill)
                    .stroke(Stroke::new(1.0, stroke_c))
                    .corner_radius(6.0)
                    .inner_margin(Margin::symmetric(8, 4));

                let prefix_text = RichText::new("# ")
                    .strong()
                    .size(13.0)
                    .color(prefix_c);

                let edit = egui::TextEdit::singleline(&mut hex_input)
                    .id(hex_edit_id)
                    .frame(frame)
                    .prefix(prefix_text)
                    .hint_text("RRGGBB")
                    .font(egui::FontId::monospace(12.5))
                    .margin(Margin::symmetric(8, 4))
                    .align(egui::Align2::LEFT_CENTER);

                let (box_rect, box_resp) = ui.allocate_exact_size(
                    Vec2::new(input_w, input_h),
                    egui::Sense::click(),
                );

                let edit_resp = ui.put(box_rect, edit);

                if edit_resp.changed() {
                    if hex_input.contains('#') {
                        hex_input = hex_input.trim_start_matches('#').to_string();
                    }
                    if let Some(parsed) = parse_hex_color(&hex_input) {
                        *color = if show_alpha {
                            parsed
                        } else {
                            Color32::from_rgb(parsed.r(), parsed.g(), parsed.b())
                        };
                        changed = true;
                    }
                }

                if box_resp.clicked() && !edit_resp.has_focus() {
                    edit_resp.request_focus();
                }

                // Copy button (using Phosphor COPY icon, never emojis)
                let hex_to_copy = if show_alpha && color.a() < 255 {
                    format!("#{:02X}{:02X}{:02X}{:02X}", color.r(), color.g(), color.b(), color.a())
                } else {
                    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
                };
                let copy_btn = ui.add(
                    egui::Button::new(RichText::new(COPY).size(12.5).color(theme.text_primary))
                        .fill(theme.bg_pill)
                        .stroke(Stroke::new(1.0, theme.border_default))
                        .corner_radius(6.0)
                        .min_size(Vec2::new(28.0, 28.0)),
                );
                if copy_btn
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text(crate::ui::i18n::format(ui, "workspace-copy", &[("hex_to_copy", &hex_to_copy)]))
                    .clicked()
                {
                    ui.ctx().copy_text(hex_to_copy);
                }

                // Color swatch preview
                let (preview_rect, _) = ui.allocate_exact_size(Vec2::new(28.0, 28.0), egui::Sense::hover());
                ui.painter().rect_filled(preview_rect, 6.0, *color);
                ui.painter().rect_stroke(
                    preview_rect,
                    6.0,
                    Stroke::new(1.0, theme.border_default),
                    StrokeKind::Inside,
                );
            });

            ui.data_mut(|d| d.insert_temp(state_id, hex_input));

            ui.add_space(6.0);

            // 2. Custom 2D Saturation/Value Canvas (Emoji-Free)
            let mut hsva = egui::ecolor::Hsva::from(*color);
            if custom_color_canvas_2d(ui, &mut hsva, theme) {
                *color = Color32::from(hsva);
                changed = true;
            }

            ui.add_space(8.0);

            // 3. Custom Hue Spectrum Slider
            if custom_hue_slider(ui, &mut hsva.h, theme) {
                *color = Color32::from(hsva);
                changed = true;
            }

            // 4. Custom Alpha Slider (when alpha enabled)
            if show_alpha {
                ui.add_space(6.0);
                if custom_alpha_slider(ui, &mut hsva.a, *color, theme) {
                    *color = Color32::from(hsva);
                    changed = true;
                }
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // 3. Quick Palette Swatches
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(4.0, 4.0);
                let swatches = [
                    Color32::WHITE,
                    Color32::from_rgb(241, 245, 249),
                    Color32::from_rgb(148, 163, 184),
                    Color32::from_rgb(51, 65, 85),
                    Color32::from_rgb(15, 23, 42),
                    Color32::BLACK,
                    Color32::from_rgb(239, 68, 68),
                    Color32::from_rgb(249, 115, 22),
                    Color32::from_rgb(245, 158, 11),
                    Color32::from_rgb(16, 185, 129),
                    Color32::from_rgb(6, 182, 212),
                    Color32::from_rgb(59, 130, 246),
                    Color32::from_rgb(99, 102, 241),
                    Color32::from_rgb(139, 92, 246),
                    Color32::from_rgb(236, 72, 153),
                ];
                for sw in swatches {
                    let (sw_rect, sw_resp) = ui.allocate_exact_size(Vec2::new(16.0, 16.0), egui::Sense::click());
                    ui.painter().rect_filled(sw_rect, 3.0, sw);
                    ui.painter().rect_stroke(
                        sw_rect,
                        3.0,
                        Stroke::new(1.0, theme.border_subtle),
                        StrokeKind::Inside,
                    );
                    if sw_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if sw_resp.clicked() {
                        *color = if show_alpha {
                            sw
                        } else {
                            Color32::from_rgb(sw.r(), sw.g(), sw.b())
                        };
                        changed = true;
                    }
                }
            });
        });

    changed
}

fn custom_color_canvas_2d(
    ui: &mut egui::Ui,
    hsva: &mut egui::ecolor::Hsva,
    theme: &AppTheme,
) -> bool {
    let width = ui.available_width();
    let height = 130.0;
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(width, height),
        egui::Sense::click_and_drag(),
    );

    let mut changed = false;
    if let Some(pos) = response.interact_pointer_pos() {
        let new_s = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        let new_v = (1.0 - (pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
        if (new_s - hsva.s).abs() > 0.001 || (new_v - hsva.v).abs() > 0.001 {
            hsva.s = new_s;
            hsva.v = new_v;
            changed = true;
        }
    }

    // Paint 2D saturation/value gradient mesh
    let steps_x = 12;
    let steps_y = 12;
    let mut mesh = egui::Mesh::default();
    for j in 0..=steps_y {
        let ty = j as f32 / steps_y as f32;
        let y = rect.min.y + ty * rect.height();
        let v = 1.0 - ty;
        for i in 0..=steps_x {
            let tx = i as f32 / steps_x as f32;
            let x = rect.min.x + tx * rect.width();
            let s = tx;
            let color = Color32::from(egui::ecolor::Hsva::new(hsva.h, s, v, 1.0));
            mesh.colored_vertex(egui::pos2(x, y), color);
        }
    }
    for j in 0..steps_y {
        for i in 0..steps_x {
            let tl = (j * (steps_x + 1) + i) as u32;
            let tr = tl + 1;
            let bl = ((j + 1) * (steps_x + 1) + i) as u32;
            let br = bl + 1;
            mesh.add_triangle(tl, tr, bl);
            mesh.add_triangle(tr, br, bl);
        }
    }
    ui.painter().add(egui::Shape::mesh(mesh));

    // Outline
    ui.painter().rect_stroke(
        rect,
        6.0,
        Stroke::new(1.0, theme.border_default),
        StrokeKind::Inside,
    );

    // Indicator handle at current S, V position
    let handle_x = rect.left() + hsva.s * rect.width();
    let handle_y = rect.top() + (1.0 - hsva.v) * rect.height();
    let handle_pos = egui::pos2(handle_x, handle_y);

    let picked_color = Color32::from(egui::ecolor::Hsva::new(hsva.h, hsva.s, hsva.v, 1.0));
    ui.painter().circle_filled(handle_pos, 7.0, Color32::WHITE);
    ui.painter().circle_filled(handle_pos, 5.0, picked_color);
    ui.painter().circle_stroke(
        handle_pos,
        7.0,
        Stroke::new(1.5, Color32::from_black_alpha(180)),
    );

    changed
}

fn custom_hue_slider(
    ui: &mut egui::Ui,
    hue: &mut f32,
    theme: &AppTheme,
) -> bool {
    let width = ui.available_width();
    let height = 15.0;
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(width, height),
        egui::Sense::click_and_drag(),
    );

    let mut changed = false;
    if let Some(pos) = response.interact_pointer_pos() {
        let new_h = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        if (new_h - *hue).abs() > 0.001 {
            *hue = new_h;
            changed = true;
        }
    }

    // Paint Hue rainbow bar with 24 subdivisions
    let steps = 24;
    let mut mesh = egui::Mesh::default();
    for i in 0..=steps {
        let tx = i as f32 / steps as f32;
        let x = rect.min.x + tx * rect.width();
        let color = Color32::from(egui::ecolor::Hsva::new(tx, 1.0, 1.0, 1.0));
        mesh.colored_vertex(egui::pos2(x, rect.top()), color);
        mesh.colored_vertex(egui::pos2(x, rect.bottom()), color);
    }
    for i in 0..steps {
        let top_l = (i * 2) as u32;
        let bot_l = top_l + 1;
        let top_r = top_l + 2;
        let bot_r = top_l + 3;
        mesh.add_triangle(top_l, top_r, bot_l);
        mesh.add_triangle(top_r, bot_r, bot_l);
    }
    ui.painter().add(egui::Shape::mesh(mesh));

    // Outline
    ui.painter().rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0, theme.border_default),
        StrokeKind::Inside,
    );

    // Indicator needle on Hue
    let handle_x = rect.left() + *hue * rect.width();
    let needle_rect = egui::Rect::from_center_size(
        egui::pos2(handle_x, rect.center().y),
        Vec2::new(5.0, height + 4.0),
    );
    ui.painter().rect_filled(needle_rect, 2.5, Color32::WHITE);
    ui.painter().rect_stroke(
        needle_rect,
        2.5,
        Stroke::new(1.0, Color32::from_black_alpha(200)),
        StrokeKind::Inside,
    );

    changed
}

fn custom_alpha_slider(
    ui: &mut egui::Ui,
    alpha: &mut f32,
    base_color: Color32,
    theme: &AppTheme,
) -> bool {
    let width = ui.available_width();
    let height = 15.0;
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(width, height),
        egui::Sense::click_and_drag(),
    );

    let mut changed = false;
    if let Some(pos) = response.interact_pointer_pos() {
        let new_a = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        if (new_a - *alpha).abs() > 0.001 {
            *alpha = new_a;
            changed = true;
        }
    }

    // Paint checkered background
    let step = 6.0;
    let mut x = rect.min.x;
    let mut col = 0;
    while x < rect.max.x {
        let mut y = rect.min.y;
        let mut row = 0;
        while y < rect.max.y {
            let tile = egui::Rect::from_min_max(
                egui::pos2(x, y),
                egui::pos2((x + step).min(rect.max.x), (y + step).min(rect.max.y)),
            );
            let fill = if (row + col) % 2 == 0 {
                Color32::from_gray(38)
            } else {
                Color32::from_gray(64)
            };
            ui.painter().rect_filled(tile, 0.0, fill);
            y += step;
            row += 1;
        }
        x += step;
        col += 1;
    }

    // Alpha gradient overlay from 0 to 1
    let c_left = Color32::from_rgba_unmultiplied(base_color.r(), base_color.g(), base_color.b(), 0);
    let c_right = Color32::from_rgb(base_color.r(), base_color.g(), base_color.b());
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), c_left);
    mesh.colored_vertex(rect.left_bottom(), c_left);
    mesh.colored_vertex(rect.right_top(), c_right);
    mesh.colored_vertex(rect.right_bottom(), c_right);
    mesh.add_triangle(0, 2, 1);
    mesh.add_triangle(2, 3, 1);
    ui.painter().add(egui::Shape::mesh(mesh));

    // Outline
    ui.painter().rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0, theme.border_default),
        StrokeKind::Inside,
    );

    // Indicator needle on Alpha
    let handle_x = rect.left() + *alpha * rect.width();
    let needle_rect = egui::Rect::from_center_size(
        egui::pos2(handle_x, rect.center().y),
        Vec2::new(5.0, height + 4.0),
    );
    ui.painter().rect_filled(needle_rect, 2.5, Color32::WHITE);
    ui.painter().rect_stroke(
        needle_rect,
        2.5,
        Stroke::new(1.0, Color32::from_black_alpha(200)),
        StrokeKind::Inside,
    );

    changed
}

pub fn color_row_solid(
    ui: &mut egui::Ui,
    label: &str,
    color: &mut Color32,
    theme: &AppTheme,
) -> bool {
    let mut changed = false;
    let label_w = 76.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        ui.add_sized(
            [label_w, 20.0],
            egui::Label::new(
                RichText::new(label)
                    .size(11.5)
                    .color(theme.text_primary),
            )
            .truncate(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if color_swatch_button(ui, label, color, false, theme) {
                changed = true;
            }
        });
    });
    changed
}

pub fn color_row_rgb(
    ui: &mut egui::Ui,
    label: &str,
    color: &mut [f32; 3],
    theme: &AppTheme,
) -> bool {
    let mut changed = false;
    let label_w = 76.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        ui.add_sized(
            [label_w, 20.0],
            egui::Label::new(
                RichText::new(label)
                    .size(11.5)
                    .color(theme.text_primary),
            )
            .truncate(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let mut c32 = Color32::from_rgb(
                (color[0] * 255.0).round().clamp(0.0, 255.0) as u8,
                (color[1] * 255.0).round().clamp(0.0, 255.0) as u8,
                (color[2] * 255.0).round().clamp(0.0, 255.0) as u8,
            );
            if color_swatch_button(ui, label, &mut c32, false, theme) {
                color[0] = c32.r() as f32 / 255.0;
                color[1] = c32.g() as f32 / 255.0;
                color[2] = c32.b() as f32 / 255.0;
                changed = true;
            }
        });
    });
    changed
}

pub fn color_row_srgba(
    ui: &mut egui::Ui,
    label: &str,
    color: &mut Color32,
    theme: &AppTheme,
) -> bool {
    let mut changed = false;
    let label_w = 76.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        ui.add_sized(
            [label_w, 20.0],
            egui::Label::new(
                RichText::new(label)
                    .size(11.5)
                    .color(theme.text_primary),
            )
            .truncate(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if color_swatch_button(ui, label, color, true, theme) {
                changed = true;
            }
        });
    });
    changed
}

pub fn color_swatch_u8_3(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    color: &mut [u8; 3],
    theme: &AppTheme,
) -> bool {
    color_swatch_u8_3_sized(ui, id_salt, color, Vec2::new(76.0, 22.0), theme)
}

pub fn color_swatch_u8_3_sized(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    color: &mut [u8; 3],
    btn_size: Vec2,
    theme: &AppTheme,
) -> bool {
    let mut c32 = Color32::from_rgb(color[0], color[1], color[2]);
    if color_swatch_button_sized(ui, id_salt, &mut c32, false, btn_size, None, theme) {
        color[0] = c32.r();
        color[1] = c32.g();
        color[2] = c32.b();
        true
    } else {
        false
    }
}

pub fn color_row_u8_3(
    ui: &mut egui::Ui,
    label: &str,
    color: &mut [u8; 3],
    theme: &AppTheme,
) -> bool {
    let mut changed = false;
    let label_w = 84.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        ui.add_sized(
            [label_w, 20.0],
            egui::Label::new(
                RichText::new(crate::ui::i18n::text(ui, label))
                    .size(11.5)
                    .color(theme.text_secondary),
            )
            .truncate(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if color_swatch_u8_3(ui, label, color, theme) {
                changed = true;
            }
        });
    });
    changed
}

pub fn color_swatch_u8_4(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    color: &mut [u8; 4],
    show_alpha: bool,
    theme: &AppTheme,
) -> bool {
    color_swatch_u8_4_sized(ui, id_salt, color, show_alpha, Vec2::new(76.0, 22.0), theme)
}

pub fn color_swatch_u8_4_sized(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    color: &mut [u8; 4],
    show_alpha: bool,
    btn_size: Vec2,
    theme: &AppTheme,
) -> bool {
    let mut c32 = Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]);
    if color_swatch_button_sized(ui, id_salt, &mut c32, show_alpha, btn_size, None, theme) {
        color[0] = c32.r();
        color[1] = c32.g();
        color[2] = c32.b();
        color[3] = c32.a();
        true
    } else {
        false
    }
}

pub fn color_row_u8_4(
    ui: &mut egui::Ui,
    label: &str,
    color: &mut [u8; 4],
    show_alpha: bool,
    theme: &AppTheme,
) -> bool {
    let mut changed = false;
    let label_w = 76.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        ui.add_sized(
            [label_w, 20.0],
            egui::Label::new(
                RichText::new(crate::ui::i18n::text(ui, label))
                    .size(11.5)
                    .color(theme.text_secondary),
            )
            .truncate(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if color_swatch_u8_4(ui, label, color, show_alpha, theme) {
                changed = true;
            }
        });
    });
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_colors_with_and_without_hash() {
        // 6 digits
        assert_eq!(parse_hex_color("#FB923C"), Some(Color32::from_rgb(251, 146, 60)));
        assert_eq!(parse_hex_color("FB923C"), Some(Color32::from_rgb(251, 146, 60)));
        assert_eq!(parse_hex_color("fb923c"), Some(Color32::from_rgb(251, 146, 60)));
        assert_eq!(parse_hex_color("##fb923c"), Some(Color32::from_rgb(251, 146, 60)));

        // 3 digits
        assert_eq!(parse_hex_color("#fff"), Some(Color32::WHITE));
        assert_eq!(parse_hex_color("000"), Some(Color32::BLACK));

        // 8 digits (with alpha)
        assert_eq!(
            parse_hex_color("#11223344"),
            Some(Color32::from_rgba_unmultiplied(0x11, 0x22, 0x33, 0x44))
        );

        // Invalid
        assert_eq!(parse_hex_color(""), None);
        assert_eq!(parse_hex_color("#12"), None);
        assert_eq!(parse_hex_color("notahex"), None);
    }
}
