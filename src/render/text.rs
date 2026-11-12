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
    let directives = if theme.options.show_song_effects {
        crate::core::markup::parse_slide_directives(text)
    } else {
        crate::core::markup::SlideDirectives::default()
    };
    let active_family_name = directives
        .font_family
        .as_deref()
        .unwrap_or(&theme.font_family);
    let line_spacing = directives.line_spacing.unwrap_or(theme.line_spacing);
    let letter_spacing = directives.letter_spacing.unwrap_or(theme.letter_spacing);
    let align_h = directives
        .align
        .as_deref()
        .unwrap_or(theme.align_h.as_str());

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
    let styled = markup::runs(text);
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
            let is_emphasis = theme.options.show_song_effects && run.style.emphasis;
            let italic = theme.font_italic || (theme.options.show_song_effects && run.style.italic);
            let run_family = if theme.options.show_song_effects
                && let Some(custom_font) = &run.style.font_family
            {
                crate::core::models::ensure_font_loaded_with_style(
                    ctx,
                    custom_font,
                    if is_emphasis {
                        theme.font_weight.max(700)
                    } else {
                        theme.font_weight
                    },
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
            let color = if theme.options.show_song_effects
                && let Some(custom_color) = run.style.color
            {
                custom_color.linear_multiply(opacity)
            } else if is_emphasis {
                emphasis_color
            } else {
                base_color
            };
            let underline_color = if theme.options.show_song_effects
                && let Some(custom_color) = run.style.color
            {
                custom_color.linear_multiply(opacity)
            } else if is_emphasis {
                emphasis_color
            } else {
                base_color
            };
            let run_font_size = if theme.options.show_song_effects
                && let Some(custom_sz) = run.style.font_size
            {
                96.0 * (custom_sz / requested.max(1.0))
            } else {
                96.0
            };
            let format = egui::TextFormat {
                font_id: egui::FontId::new(run_font_size, run_family),
                color,
                extra_letter_spacing: letter_spacing / scale,
                line_height: Some(run_font_size * line_spacing.max(0.5)),
                underline: if theme.font_underline
                    || (theme.options.show_song_effects && run.style.underline)
                {
                    Stroke::new(2.0, underline_color)
                } else {
                    Stroke::NONE
                },
                strikethrough: if theme.font_strikethrough
                    || (theme.options.show_song_effects && run.style.strikethrough)
                {
                    Stroke::new(2.0, underline_color)
                } else {
                    Stroke::NONE
                },
                ..Default::default()
            };
            let run_text = theme.transform_text(run.text);
            job.append(&run_text, 0.0, format);
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
    render_internal(painter, rect, theme, text, None, 0.0);
}

pub fn render_with_override_color(
    painter: &egui::Painter,
    rect: Rect,
    theme: &PresentationTheme,
    text: &str,
    override_color: Option<Color32>,
) {
    render_internal(painter, rect, theme, text, override_color, 0.0);
}

pub fn render_blurred(
    painter: &egui::Painter,
    rect: Rect,
    theme: &PresentationTheme,
    text: &str,
    blur_radius: f32,
) {
    render_internal(painter, rect, theme, text, None, blur_radius);
}

fn render_internal(
    painter: &egui::Painter,
    rect: Rect,
    theme: &PresentationTheme,
    text: &str,
    override_color: Option<Color32>,
    blur_radius: f32,
) {
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
    let directives = crate::core::markup::parse_slide_directives(text);
    let (verse_text, ref_text_opt) = if let Some(ref pos) = directives.ref_pos {
        let mut lines: Vec<&str> = text.lines().collect();
        let visible_lines: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, l)| !crate::core::markup::is_directive_command(l.trim_start()))
            .map(|(i, _)| i)
            .collect();
        if visible_lines.len() > 1 {
            match pos.as_str() {
                "top" => {
                    let r = lines.remove(visible_lines[0]).trim().to_string();
                    let v = lines.join("\n");
                    (v, Some(r))
                }
                "bottom" | "below" => {
                    let r = lines
                        .remove(*visible_lines.last().unwrap())
                        .trim()
                        .to_string();
                    let v = lines.join("\n");
                    (v, Some(r))
                }
                _ => (text.to_string(), None),
            }
        } else {
            (text.to_string(), directives.ref_text.clone())
        }
    } else {
        (text.to_string(), directives.ref_text.clone())
    };

    let to_view = |position: Pos2| rect.min + position.to_vec2() * view_scale;
    let view_box = Rect::from_min_size(to_view(box_rect.min), box_rect.size() * view_scale);

    let ref_layout_opt = if let Some(ref ref_str) = ref_text_opt {
        let ref_align = directives.ref_align.as_deref().unwrap_or("right");
        let ref_size_mult = match directives.ref_size.as_deref() {
            Some("compact") | Some("small") | Some("pequeno") | Some("reduzido") => 0.70,
            _ => 1.0, // Default: normal (same size as verse text)
        };
        let ref_theme = {
            let mut t = theme.clone();
            t.font_size = (theme.font_size * ref_size_mult).clamp(16.0, 72.0);
            t.align_h = ref_align.to_string();
            t
        };
        Some(layout(painter, &ref_theme, ref_str, inner.size()))
    } else {
        None
    };

    let screen_top = 28.0;
    let screen_bottom = design_size.y - 28.0;
    let ref_x = match directives.ref_align.as_deref().unwrap_or("right") {
        "center" => inner.center().x,
        "right" => inner.right(),
        _ => inner.left(),
    };
    let centered_edge_reference = matches!(directives.ref_pos.as_deref(), Some("top" | "bottom"))
        && directives.valign.as_deref().unwrap_or(&theme.align_v) == "center";

    let (verse_inner, ref_pos_and_data) = if let Some(ref_layout) = ref_layout_opt {
        let ref_pos_str = directives.ref_pos.as_deref().unwrap_or("bottom");
        // Top/bottom references belong to the slide, not to the theme's text
        // box. Reserve the same clearance at both edges so switching between
        // them never moves the centered verse, even with an offset theme box.
        let inner = if centered_edge_reference {
            let half_height = (inner.height() * 0.5)
                .min((design_size.y * 0.5 - screen_top - ref_layout.size.y - 16.0).max(1.0));
            Rect::from_center_size(
                egui::pos2(inner.center().x, design_size.y * 0.5),
                egui::vec2(inner.width(), half_height * 2.0),
            )
        } else {
            inner
        };
        match ref_pos_str {
            "top" => {
                let ref_y = screen_top;
                let v_top = (screen_top + ref_layout.size.y + 16.0).max(inner.top());
                let v_inner = Rect::from_min_max(Pos2::new(inner.left(), v_top), inner.max);
                let ref_pos = to_view(egui::pos2(ref_x, ref_y));
                let ref_scale = ref_layout.scale * view_scale;
                (v_inner, Some((ref_layout, ref_pos, ref_scale, false)))
            }
            "bottom" => {
                let ref_y = screen_bottom - ref_layout.size.y;
                let v_bottom = (ref_y - 16.0).min(inner.bottom());
                let v_inner = Rect::from_min_max(inner.min, Pos2::new(inner.right(), v_bottom));
                let ref_pos = to_view(egui::pos2(ref_x, ref_y));
                let ref_scale = ref_layout.scale * view_scale;
                (v_inner, Some((ref_layout, ref_pos, ref_scale, false)))
            }
            _ => {
                let v_bottom = (inner.bottom() - ref_layout.size.y - 12.0).max(inner.top() + 20.0);
                let v_inner = Rect::from_min_max(inner.min, Pos2::new(inner.right(), v_bottom));
                (v_inner, Some((ref_layout, Pos2::ZERO, 0.0, true)))
            }
        }
    } else {
        (inner, None)
    };

    let align_h = directives
        .align
        .as_deref()
        .unwrap_or(theme.align_h.as_str());
    let mut verse_theme = theme.clone();
    verse_theme.align_h = align_h.to_string();
    let text_layout = layout(painter, &verse_theme, &verse_text, verse_inner.size());
    let verse_clip = if centered_edge_reference {
        Rect::from_min_size(to_view(verse_inner.min), verse_inner.size() * view_scale)
    } else {
        view_box
    };
    let mut clipped =
        painter.with_clip_rect(painter.clip_rect().intersect(rect).intersect(verse_clip));
    // Screen-edge references are outside the theme's verse box by design.
    let mut ref_painter = painter.with_clip_rect(painter.clip_rect().intersect(rect));
    let x = match align_h {
        "center" => verse_inner.center().x,
        "right" => verse_inner.right(),
        _ => verse_inner.left(),
    };
    let align_v = directives
        .valign
        .as_deref()
        .unwrap_or(theme.align_v.as_str());
    let y = match align_v {
        "top" => verse_inner.top(),
        "bottom" => (verse_inner.bottom() - text_layout.size.y).max(verse_inner.top()),
        _ => (verse_inner.center().y - text_layout.size.y * 0.5).max(verse_inner.top()),
    };
    let position = to_view(egui::pos2(x, y));
    let scale = text_layout.scale * view_scale;
    let angle = theme.options.text_rotation.to_radians();

    let ref_render_data = match ref_pos_and_data {
        Some((layout, _, _, true)) => {
            let ref_y = (y + text_layout.size.y + 12.0).min(screen_bottom - layout.size.y);
            let ref_pos = to_view(egui::pos2(ref_x, ref_y));
            let ref_scale = layout.scale * view_scale;
            Some((layout, ref_pos, ref_scale))
        }
        Some((layout, pos, sc, false)) => Some((layout, pos, sc)),
        None => None,
    };

    if let Some(color) = override_color {
        clipped.add(text_shape(
            text_layout.galley,
            position,
            scale,
            color,
            angle,
            Some(color),
        ));
        if let Some((ref_layout, ref_pos, ref_scale)) = ref_render_data {
            ref_painter.add(text_shape(
                ref_layout.galley,
                ref_pos,
                ref_scale,
                color,
                angle,
                Some(color),
            ));
        }
        return;
    }

    clipped.multiply_opacity(theme.options.text_opacity.clamp(0.0, 1.0));
    ref_painter.multiply_opacity(theme.options.text_opacity.clamp(0.0, 1.0));
    let mut box_painter = painter.with_clip_rect(painter.clip_rect().intersect(rect));
    box_painter.multiply_opacity(theme.options.text_opacity.clamp(0.0, 1.0));
    if theme.box_bg_enabled {
        box_painter.rect_filled(view_box, theme.box_radius * view_scale, theme.box_bg_color);
    }
    if theme.box_border_enabled {
        box_painter.rect_stroke(
            view_box,
            theme.box_radius * view_scale,
            Stroke::new(theme.box_border_width * view_scale, theme.box_border_color),
            egui::StrokeKind::Inside,
        );
    }
    let base_color =
        readable_text_color(theme).linear_multiply(theme.options.text_opacity.clamp(0.0, 1.0));
    let shadow_enabled = theme.shadow_enabled || directives.shadow_color.is_some();
    let shadow_color = directives.shadow_color.unwrap_or(theme.shadow_color);
    if shadow_enabled {
        let offset = egui::vec2(theme.shadow_offset_x, theme.shadow_offset_y) * view_scale;
        let shadow_blur = (theme.shadow_blur + blur_radius * 0.6) * view_scale;
        render_shadow(
            &mut clipped,
            text_layout.galley.clone(),
            position,
            offset,
            shadow_blur,
            scale,
            angle,
            shadow_color,
            theme.options.text_opacity,
        );
        if let Some((ref ref_layout, ref_pos, ref_scale)) = ref_render_data {
            render_shadow(
                &mut ref_painter,
                ref_layout.galley.clone(),
                ref_pos,
                offset,
                shadow_blur,
                ref_scale,
                angle,
                shadow_color,
                theme.options.text_opacity,
            );
        }
    }

    let blur = blur_radius * view_scale;
    if blur <= 0.4 {
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
    } else {
        // Convolução circular concêntrica contínua com decaimento gaussiano 2D e rotação áurea entre anéis.
        // Elimina totalmente os artefatos de grade/dither da espiral de Vogel e garante difusão óptica pura.
        let max_r = blur.min(16.0);
        let num_rings = ((max_r / 1.4).ceil().clamp(3.0, 6.0)) as usize;
        let r_step = max_r / num_rings as f32;
        let sigma = max_r * 0.50;

        let center_weight = 1.0f32;
        let mut sum_weights = center_weight;
        for ring in 1..=num_rings {
            let r = ring as f32 * r_step;
            let w = (-0.5 * (r / sigma).powi(2)).exp();
            let count = 4 + ring * 4;
            sum_weights += w * count as f32;
        }
        let weight_scale = if sum_weights > 0.001 {
            1.0 / sum_weights
        } else {
            1.0
        };
        let boost = (1.0 + (num_rings as f32) * 0.24).clamp(1.15, 2.2);
        let base_a = base_color.a() as f32 / 255.0;

        // Amostra central com atenuação óptica proporcional ao desfoque (sem espícula nítida)
        let core_alpha = (base_a * center_weight * weight_scale * boost).clamp(0.01, 0.45);
        let core_col = base_color.linear_multiply(core_alpha);
        clipped.add(text_shape(
            text_layout.galley.clone(),
            position,
            scale,
            core_col,
            angle,
            Some(core_col),
        ));

        // Anéis concêntricos estagiados com espaçamento contínuo e fase áurea (evita eixos ou estrias)
        for ring in 1..=num_rings {
            let r = ring as f32 * r_step;
            let w = (-0.5 * (r / sigma).powi(2)).exp();
            let count = 4 + ring * 4;
            let step_angle = std::f32::consts::TAU / count as f32;
            let phase = ring as f32 * 0.6180339887;
            let sample_alpha = (base_a * w * weight_scale * boost).clamp(0.003, 0.35);
            let sample_col = base_color.linear_multiply(sample_alpha);

            for i in 0..count {
                let theta = i as f32 * step_angle + phase;
                clipped.add(text_shape(
                    text_layout.galley.clone(),
                    position + egui::vec2(theta.cos(), theta.sin()) * r,
                    scale,
                    sample_col,
                    angle,
                    Some(sample_col),
                ));
            }
        }
    }

    if let Some((ref ref_layout, ref_pos, ref_scale)) = ref_render_data {
        let ref_color = base_color.linear_multiply(0.88);
        ref_painter.add(text_shape(
            ref_layout.galley.clone(),
            ref_pos,
            ref_scale,
            ref_color,
            angle,
            None,
        ));
    }
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

    // Difusão gaussiana concêntrica suave (sem núcleo rígido residual quando o desfoque é amplo)
    let max_r = blur.min(18.0);
    let num_rings = ((max_r / 1.5).ceil().clamp(3.0, 6.0)) as usize;
    let r_step = max_r / num_rings as f32;
    let sigma = max_r * 0.50;

    let center_weight = 1.0f32;
    let mut sum_weights = center_weight;
    for ring in 1..=num_rings {
        let r = ring as f32 * r_step;
        let w = (-0.5 * (r / sigma).powi(2)).exp();
        let count = 4 + ring * 4;
        sum_weights += w * count as f32;
    }
    let weight_scale = if sum_weights > 0.001 {
        1.0 / sum_weights
    } else {
        1.0
    };
    let boost = (1.0 + (num_rings as f32) * 0.22).clamp(1.1, 2.0);

    // Amostra central de sombra atenuada de acordo com o raio de difusão
    let center_alpha = (base_a * center_weight * weight_scale * boost).clamp(0.005, 0.5);
    let center_color = color.linear_multiply(center_alpha);
    clipped.add(text_shape(
        galley.clone(),
        position + offset,
        scale,
        center_color,
        angle,
        Some(center_color),
    ));

    // Anéis concêntricos com rotação áurea relativa
    for ring in 1..=num_rings {
        let r = ring as f32 * r_step;
        let w = (-0.5 * (r / sigma).powi(2)).exp();
        let count = 4 + ring * 4;
        let step_angle = std::f32::consts::TAU / count as f32;
        let phase = ring as f32 * 0.6180339887;
        let tap_alpha = (base_a * w * weight_scale * boost).clamp(0.003, 0.35);
        let tap_color = color.linear_multiply(tap_alpha);

        for i in 0..count {
            let theta = i as f32 * step_angle + phase;
            clipped.add(text_shape(
                galley.clone(),
                position + offset + egui::vec2(theta.cos(), theta.sin()) * r,
                scale,
                tap_color,
                angle,
                Some(tap_color),
            ));
        }
    }
}

#[cfg(test)]
#[path = "text_bible_tests.rs"]
mod bible_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bible_reference_is_visible_and_verse_is_centered() {
        use crate::core::bible::{BibleSettings, format_bible_slide};
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, egui::vec2(1280.0, 720.0));
        let mut theme = PresentationTheme::default_themes().remove(0);
        theme.font_family = "Proportional".into();
        theme.shadow_enabled = false;
        // Bible alignment must override a theme's lower/right placement.
        theme.align_h = "right".into();
        theme.align_v = "bottom".into();
        let (_, content) = format_bible_slide(
            &BibleSettings {
                reference_position: crate::core::bible::BibleReferencePosition::Bottom,
                ..Default::default()
            }, "Salmos", "Sl", "NTLH", 39, 4, 4,
            &["Ó Senhor Deus, quanto tempo ainda vou viver? Mostra-me como é passageira a minha vida. Quando é que vou morrer?".into()],
        );
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ui| render(ui.painter(), screen, &theme, &content),
        );
        let texts: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|s| {
                if let egui::Shape::Text(t) = &s.shape {
                    Some((
                        s.clip_rect,
                        t.visual_bounding_rect(),
                        t.galley.job.text.clone(),
                    ))
                } else {
                    None
                }
            })
            .collect();
        output.textures_delta.clear();
        assert_eq!(texts.len(), 2, "{texts:?}");
        assert!(
            texts[1].0.contains_rect(texts[1].1),
            "Reference is clipped: {texts:?}"
        );
        assert!(
            (texts[0].1.center().x - screen.center().x).abs() < 16.0,
            "Verse is off center: {texts:?}"
        );
        assert!(
            (texts[0].1.center().y - screen.center().y).abs() < 16.0,
            "Verse is off center: {texts:?}"
        );
    }

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

    #[test]
    fn test_show_song_effects_toggle() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let mut theme = PresentationTheme::default();
            theme.text_color = Color32::WHITE;
            theme.font_size = 40.0;
            theme.options.auto_fit = false;
            let text = "//t: 80\n//c: #FF0000\nHello [c: #00FF00]World[/c] and **holy**";

            // With song effects enabled:
            theme.options.show_song_effects = true;
            let layout_with = layout(ui.painter(), &theme, text, egui::vec2(1200.0, 700.0));

            // With song effects disabled:
            theme.options.show_song_effects = false;
            let layout_without = layout(ui.painter(), &theme, text, egui::vec2(1200.0, 700.0));

            // When song effects are disabled, requested size is theme font_size (40.0), not directive size (80.0)
            assert_eq!(layout_without.font_size, 40.0);
            assert_eq!(layout_with.font_size, 80.0);
            assert!(!layout_without.galley.rows.is_empty());
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_render_blurred_produces_smooth_shapes() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let theme = PresentationTheme::default();
            let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 720.0));
            render_blurred(ui.painter(), rect, &theme, "Holy Holy Holy", 16.0);
        });
        output.textures_delta.clear();
        assert!(
            output.shapes.len() >= 30,
            "Gaussian spiral should generate rich multi-sample shapes"
        );
    }
}
