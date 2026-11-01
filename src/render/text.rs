//! All typography is laid out in a 720p design space, then scaled as geometry.
//! Preview size, UI zoom and monitor DPI never change wrapping or auto-fit.
use crate::core::markup;
use crate::core::models::PresentationTheme;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use std::sync::Arc;

fn luminance(color: Color32) -> f32 {
    let channel = |value: u8| {
        let value = value as f32 / 255.0;
        if value <= 0.03928 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgb(lerp(a.r(), b.r()), lerp(a.g(), b.g()), lerp(a.b(), b.b()))
}

fn contrast(a: Color32, b: Color32) -> f32 {
    let (light, dark) = if luminance(a) > luminance(b) {
        (a, b)
    } else {
        (b, a)
    };
    (luminance(light) + 0.05) / (luminance(dark) + 0.05)
}

/// Solid/gradient backgrounds can be light while a theme keeps white text.
/// Pick a readable foreground in that case; image/video backgrounds keep the
/// user's color because the actual pixels are unknown here.
fn readable_text_color(theme: &PresentationTheme) -> Color32 {
    if matches!(theme.bg_type.as_str(), "image" | "video" | "shader") {
        return theme.text_color;
    }
    let background = mix(theme.bg_top, theme.bg_bottom, 0.5);
    if contrast(theme.text_color, background) >= 2.6 {
        return theme.text_color;
    }
    if luminance(background) > 0.45 {
        Color32::from_rgb(24, 28, 36)
    } else {
        Color32::WHITE
    }
}

pub struct TextLayout {
    pub galley: Arc<egui::Galley>,
    pub scale: f32,
    pub font_size: f32,
    pub size: Vec2,
}

pub fn layout(
    painter: &egui::Painter,
    theme: &PresentationTheme,
    text: &str,
    available: Vec2,
) -> TextLayout {
    let ctx = painter.ctx();
    let directives = crate::core::markup::parse_slide_directives(text);
    let active_family_name = directives.font_family.as_deref().unwrap_or(&theme.font_family);
    let line_spacing = directives.line_spacing.unwrap_or(theme.line_spacing);
    let letter_spacing = directives.letter_spacing.unwrap_or(theme.letter_spacing);
    let align_h = directives.align.as_deref().unwrap_or(theme.align_h.as_str());

    let family = crate::core::models::ensure_font_loaded_with_style(
        ctx,
        active_family_name,
        theme.font_weight,
        theme.font_italic,
    );
    let emphasis_family = crate::core::models::ensure_font_loaded_with_style(
        ctx,
        active_family_name,
        theme.font_weight.max(700),
        theme.font_italic,
    );
    let italic_family = crate::core::models::ensure_font_loaded_with_style(
        ctx,
        active_family_name,
        theme.font_weight,
        true,
    );
    let emphasis_italic_family = crate::core::models::ensure_font_loaded_with_style(
        ctx,
        active_family_name,
        theme.font_weight.max(700),
        true,
    );
    let transformed = theme.transform_text(text);
    let styled = markup::runs(&transformed);
    let opacity = theme.options.text_opacity.clamp(0.0, 1.0);
    let base_color = if let Some(dir_col) = directives.color {
        dir_col.linear_multiply(opacity)
    } else {
        readable_text_color(theme).linear_multiply(opacity)
    };
    let emphasis_color = Color32::from_rgb(
        theme.options.emphasis_color[0],
        theme.options.emphasis_color[1],
        theme.options.emphasis_color[2],
    )
    .linear_multiply(opacity);
    let requested = if let Some(dir_size) = directives.font_size {
        dir_size.max(0.1)
    } else if theme.font_size.is_finite() {
        theme.font_size.max(0.1)
    } else {
        44.0
    };
    let make_layout = |size: f32| {
        let scale = size / 96.0;
        let mut job = egui::text::LayoutJob::default();
        for run in &styled {
            let is_emphasis = run.style.emphasis;
            let italic = theme.font_italic || run.style.italic;
            let run_family = if let Some(custom_font) = &run.style.font_family {
                crate::core::models::ensure_font_loaded_with_style(
                    ctx,
                    custom_font,
                    if is_emphasis { theme.font_weight.max(700) } else { theme.font_weight },
                    italic,
                )
            } else if is_emphasis {
                if italic {
                    emphasis_italic_family.clone()
                } else {
                    emphasis_family.clone()
                }
            } else if italic && !theme.font_italic {
                italic_family.clone()
            } else {
                family.clone()
            };
            let color = if let Some(custom_color) = run.style.color {
                custom_color.linear_multiply(opacity)
            } else if is_emphasis {
                emphasis_color
            } else {
                base_color
            };
            let underline_color = if let Some(custom_color) = run.style.color {
                custom_color.linear_multiply(opacity)
            } else if is_emphasis {
                emphasis_color
            } else {
                base_color
            };
            let run_font_size = if let Some(custom_sz) = run.style.font_size {
                96.0 * (custom_sz / requested.max(1.0))
            } else {
                96.0
            };
            let format = egui::TextFormat {
                font_id: egui::FontId::new(run_font_size, run_family),
                color,
                extra_letter_spacing: letter_spacing / scale,
                line_height: Some(run_font_size * line_spacing.max(0.5)),
                underline: if theme.font_underline || run.style.underline {
                    Stroke::new(2.0, underline_color)
                } else {
                    Stroke::NONE
                },
                strikethrough: if theme.font_strikethrough || run.style.strikethrough {
                    Stroke::new(2.0, underline_color)
                } else {
                    Stroke::NONE
                },
                ..Default::default()
            };
            job.append(run.text, 0.0, format);
        }
        job.wrap.max_width = if theme.options.wrap_text {
            available.x.max(1.0) / scale
        } else {
            f32::INFINITY
        };
        job.halign = match align_h {
            "center" => egui::Align::Center,
            "right" => egui::Align::RIGHT,
            _ => egui::Align::LEFT,
        };
        job.justify = align_h == "justify" && theme.options.wrap_text;
        let mut galley = painter.layout_job(job);
        if theme.options.paragraph_spacing != 0.0 {
            let inner = Arc::make_mut(&mut galley);
            let mut extra = 0.0;
            for row in &mut inner.rows {
                row.pos.y += extra;
                if row.ends_with_newline {
                    extra += theme.options.paragraph_spacing / scale;
                }
            }
            inner.rect.max.y += extra;
            inner.mesh_bounds.max.y += extra;
        }
        let size = galley.size() * scale;
        TextLayout {
            galley,
            scale,
            font_size: scale * 96.0,
            size,
        }
    };
    let mut result = make_layout(requested);
    if theme.options.auto_fit
        && (result.size.x > available.x + 0.1 || result.size.y > available.y + 0.1)
    {
        let mut low = 0.1;
        let mut high = requested;
        for _ in 0..18 {
            let middle = (low + high) * 0.5;
            let candidate = make_layout(middle);
            if candidate.size.x <= available.x && candidate.size.y <= available.y {
                low = middle;
            } else {
                high = middle;
            }
        }
        result = make_layout(low);
    }
    result
}

fn text_shape(
    galley: Arc<egui::Galley>,
    position: Pos2,
    scale: f32,
    color: Color32,
    angle: f32,
    override_color: Option<Color32>,
) -> egui::Shape {
    let mut shape = egui::epaint::TextShape::new(Pos2::ZERO, galley, color);
    if let Some(override_color) = override_color {
        shape = shape.with_override_text_color(override_color);
    }
    shape.transform(egui::emath::TSTransform {
        scaling: scale,
        translation: position.to_vec2(),
    });
    shape.angle = angle;
    shape.into()
}

pub fn render(painter: &egui::Painter, rect: Rect, theme: &PresentationTheme, text: &str) {
    let clean_text = crate::core::slides::strip_comments(text);
    if clean_text.trim().is_empty() || !rect.is_positive() {
        return;
    }
    let aspect_ratio = rect.width() / rect.height().max(0.001);
    let resolved = crate::core::breaks::resolve_conditional_breaks(&clean_text, aspect_ratio);
    let text = resolved.as_str();
    let view_scale = rect.height() / 720.0;
    let design_size = rect.size() / view_scale;
    let box_rect = Rect::from_min_size(
        egui::pos2(theme.box_x * design_size.x, theme.box_y * 720.0),
        egui::vec2(theme.box_w * design_size.x, theme.box_h * 720.0),
    );
    let padding = theme
        .box_padding
        .max(0.0)
        .min(box_rect.width().min(box_rect.height()) * 0.45);
    let inner = box_rect.shrink(padding);
    let text_layout = layout(painter, theme, text, inner.size());
    let to_view = |position: Pos2| rect.min + position.to_vec2() * view_scale;
    let view_box = Rect::from_min_size(to_view(box_rect.min), box_rect.size() * view_scale);
    let mut clipped =
        painter.with_clip_rect(painter.clip_rect().intersect(rect).intersect(view_box));
    clipped.multiply_opacity(theme.options.text_opacity.clamp(0.0, 1.0));
    if theme.box_bg_enabled {
        clipped.rect_filled(view_box, theme.box_radius * view_scale, theme.box_bg_color);
    }
    if theme.box_border_enabled {
        clipped.rect_stroke(
            view_box,
            theme.box_radius * view_scale,
            Stroke::new(theme.box_border_width * view_scale, theme.box_border_color),
            egui::StrokeKind::Inside,
        );
    }
    let directives = crate::core::markup::parse_slide_directives(text);
    let align_h = directives.align.as_deref().unwrap_or(theme.align_h.as_str());
    let x = match align_h {
        "center" => inner.center().x,
        "right" => inner.right(),
        _ => inner.left(),
    };
    let y = match theme.align_v.as_str() {
        "top" => inner.top(),
        "bottom" => inner.bottom() - text_layout.size.y,
        _ => inner.center().y - text_layout.size.y * 0.5,
    };
    let position = to_view(egui::pos2(x, y));
    let scale = text_layout.scale * view_scale;
    let angle = theme.options.text_rotation.to_radians();
    let base_color =
        readable_text_color(theme).linear_multiply(theme.options.text_opacity.clamp(0.0, 1.0));
    let shadow_enabled = theme.shadow_enabled || directives.shadow_color.is_some();
    let shadow_color = directives.shadow_color.unwrap_or(theme.shadow_color);
    if shadow_enabled {
        let offset = egui::vec2(theme.shadow_offset_x, theme.shadow_offset_y) * view_scale;
        let blur = theme.shadow_blur * view_scale;
        render_shadow(
            &mut clipped,
            text_layout.galley.clone(),
            position,
            offset,
            blur,
            scale,
            angle,
            shadow_color,
            theme.options.text_opacity,
        );
    }
    if theme.options.text_glow > 0.0 {
        let glow_r = theme.options.text_glow * view_scale;
        let num_rings = (glow_r / 2.0).ceil().clamp(3.0, 8.0) as usize;
        let sigma = glow_r * 0.45;
        let glow_color = theme.text_color;
        let base_alpha =
            (glow_color.a() as f32 / 255.0) * theme.options.text_opacity.clamp(0.0, 1.0);
        for ring in 1..=num_rings {
            let r = glow_r * (ring as f32 / num_rings as f32);
            let w = (-0.5 * (r / sigma).powi(2)).exp();
            let count = (8 + ring * 6).min(48);
            let step = std::f32::consts::TAU / count as f32;
            let ring_alpha =
                (base_alpha * w * 0.18 / (num_rings as f32).sqrt()).clamp(0.005, 0.4);
            let sample_color = glow_color.linear_multiply(ring_alpha);
            let phase = ring as f32 * 0.381966;
            for i in 0..count {
                let a = i as f32 * step + phase;
                clipped.add(text_shape(
                    text_layout.galley.clone(),
                    position + egui::vec2(a.cos(), a.sin()) * r,
                    scale,
                    sample_color,
                    angle,
                    Some(sample_color),
                ));
            }
        }
    }
    if theme.outline_enabled && theme.outline_width > 0.0 {
        let outline_r = theme.outline_width * view_scale;
        let num_rings = (outline_r / 0.75).ceil().max(1.0) as usize;
        let r_step = outline_r / num_rings as f32;
        for ring in 1..=num_rings {
            let r = ring as f32 * r_step;
            let count =
                ((2.0 * std::f32::consts::PI * r / 0.85).ceil().max(8.0) as usize).min(64);
            let angle_step = std::f32::consts::TAU / count as f32;
            let offset_angle = ring as f32 * 0.381966;
            for i in 0..count {
                let a = i as f32 * angle_step + offset_angle;
                clipped.add(text_shape(
                    text_layout.galley.clone(),
                    position + egui::vec2(a.cos(), a.sin()) * r,
                    scale,
                    theme.outline_color,
                    angle,
                    Some(theme.outline_color),
                ));
            }
        }
    }
    clipped.add(text_shape(
        text_layout.galley,
        position,
        scale,
        base_color,
        angle,
        None,
    ));
}

fn render_shadow(
    clipped: &mut egui::Painter,
    galley: Arc<egui::Galley>,
    position: Pos2,
    offset: Vec2,
    blur: f32,
    scale: f32,
    angle: f32,
    color: Color32,
    opacity: f32,
) {
    let opacity = opacity.clamp(0.0, 1.0);
    let base_a = (color.a() as f32 / 255.0) * opacity;
    if base_a <= 0.001 {
        return;
    }

    if blur <= 0.6 {
        let sample_color = color.linear_multiply(opacity);
        clipped.add(text_shape(
            galley,
            position + offset,
            scale,
            sample_color,
            angle,
            Some(sample_color),
        ));
        return;
    }

    // 1. Core shadow pass: Provides rich, solid contrast behind the text
    let core_alpha = base_a * 0.65;
    let core_color = color.linear_multiply(core_alpha);
    clipped.add(text_shape(
        galley.clone(),
        position + offset,
        scale,
        core_color,
        angle,
        Some(core_color),
    ));

    // 4 cardinal sub-pixel taps at the core for anti-aliasing the core boundary
    let core_sub_r = (blur * 0.12).clamp(0.4, 1.2);
    let sub_color = color.linear_multiply(base_a * 0.18);
    for &(dx, dy) in &[
        (-core_sub_r, 0.0),
        (core_sub_r, 0.0),
        (0.0, -core_sub_r),
        (0.0, core_sub_r),
    ] {
        clipped.add(text_shape(
            galley.clone(),
            position + offset + egui::vec2(dx, dy),
            scale,
            sub_color,
            angle,
            Some(sub_color),
        ));
    }

    // 2. Continuous Vogel Spiral with Gaussian falloff (eliminates concentric rings and scallops)
    let num_samples = ((blur * 3.5).ceil().clamp(36.0, 72.0)) as usize;
    let golden_angle = 2.399963229728653; // radians: PI * (3.0 - sqrt(5.0))
    let sigma = blur * 0.52;

    let mut sum_weights = 0.0f32;
    for i in 0..num_samples {
        let t = (i as f32 + 0.5) / (num_samples as f32);
        let r = blur * t.sqrt();
        sum_weights += (-0.5 * (r / sigma).powi(2)).exp();
    }
    let weight_scale = if sum_weights > 0.001 {
        1.0 / sum_weights
    } else {
        1.0 / num_samples as f32
    };

    for i in 0..num_samples {
        let t = (i as f32 + 0.5) / (num_samples as f32);
        let r = blur * t.sqrt();
        let theta = i as f32 * golden_angle;
        let w = (-0.5 * (r / sigma).powi(2)).exp();
        let sample_alpha =
            (base_a * 0.75 * w * weight_scale * (num_samples as f32).sqrt() * 0.42)
                .clamp(0.003, 0.35);
        let sample_color = color.linear_multiply(sample_alpha);
        clipped.add(text_shape(
            galley.clone(),
            position + offset + egui::vec2(theta.cos(), theta.sin()) * r,
            scale,
            sample_color,
            angle,
            Some(sample_color),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_solid_theme_uses_readable_text_color() {
        let mut theme = PresentationTheme::default();
        theme.bg_type = "gradient".into();
        theme.bg_top = Color32::from_rgb(245, 247, 250);
        theme.bg_bottom = Color32::from_rgb(235, 238, 243);
        theme.text_color = Color32::WHITE;
        let color = readable_text_color(&theme);
        assert!(contrast(color, theme.bg_top) >= 4.0);
        assert!(luminance(color) < 0.2);

        theme.text_color = Color32::from_rgb(24, 28, 36);
        assert_eq!(readable_text_color(&theme), theme.text_color);
    }

    #[test]
    fn preview_and_secondary_screen_have_identical_normalized_typography() {
        let ctx = egui::Context::default();
        let mut normalized = Vec::new();
        for size in [egui::vec2(320.0, 180.0), egui::vec2(1920.0, 1080.0)] {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
                    ..Default::default()
                },
                |ui| {
                    let mut theme = PresentationTheme::default();
                    theme.font_family = "Proportional".into();
                    theme.font_size = 260.0;
                    theme.options.auto_fit = false;
                    theme.shadow_enabled = false;
                    render(
                        ui.painter(),
                        Rect::from_min_size(Pos2::ZERO, size),
                        &theme,
                        "Aa",
                    );
                },
            );
            let bounds = output
                .shapes
                .iter()
                .find_map(|clipped| {
                    if let egui::Shape::Text(text) = &clipped.shape {
                        Some(text.visual_bounding_rect())
                    } else {
                        None
                    }
                })
                .unwrap();
            normalized.push([
                bounds.left() / size.x,
                bounds.top() / size.y,
                bounds.width() / size.x,
                bounds.height() / size.y,
            ]);
            output.textures_delta.clear();
        }
        for (preview, live) in normalized[0].iter().zip(normalized[1]) {
            assert!((preview - live).abs() < 0.001, "{normalized:?}");
        }
    }

    #[test]
    fn typography_wraps_and_has_no_120_point_ceiling() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let mut theme = PresentationTheme::default();
            theme.font_family = "Proportional".into();
            theme.font_size = 360.0;
            theme.options.auto_fit = false;
            let result = layout(ui.painter(), &theme, "Aa", egui::vec2(1200.0, 700.0));
            assert!((result.font_size - 360.0).abs() < 0.01);
            theme.font_size = 44.0;
            let text = "Uma passagem longa precisa manter a mesma quebra de linha na prévia e na tela secundária.";
            let result = layout(ui.painter(), &theme, text, egui::vec2(450.0, 400.0));
            assert!(result.galley.rows.len() > 1);
            assert!(result.size.x <= 450.1 && result.size.y <= 400.1);
            for size in [egui::vec2(320.0, 180.0), egui::vec2(1920.0, 1080.0)] {
                render(ui.painter(), Rect::from_min_size(Pos2::ZERO, size), &theme, text);
            }
        });
        output.textures_delta.clear();
    }

    #[test]
    fn text_shadow_and_outline_render_smoothly_without_panicking() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let mut theme = PresentationTheme::default();
            theme.shadow_enabled = true;
            theme.shadow_blur = 24.0;
            theme.shadow_offset_x = 0.0;
            theme.shadow_offset_y = 3.0;
            theme.shadow_color = Color32::from_black_alpha(230);
            theme.outline_enabled = true;
            theme.outline_width = 4.0;
            theme.outline_color = Color32::BLACK;
            render(
                ui.painter(),
                Rect::from_min_size(Pos2::ZERO, egui::vec2(1920.0, 1080.0)),
                &theme,
                "Se não for para te Adorar",
            );
        });
        output.textures_delta.clear();
        assert!(!output.shapes.is_empty());
    }
}
