//! Syntax highlighting and layouter for song & strophe lyric editing.
//!
//! Provides visual feedback for:
//! - Tags: `#*Refrão`, `#*Tag` (colored with tag color or accent)
//! - Line Comments: `// comentário` (italicized, muted gray)
//! - Inline Comments: `Letra // comentário` (comment part in muted gray)
//! - Markup: `**negrito**`, `*itálico*`, `__sublinhado__`, `~~tachado~~`

use crate::core::preferences::StropheTagDefinition;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, Stroke};

pub fn highlight_lyrics_job(
    text: &str,
    tags: &[StropheTagDefinition],
    theme: &AppTheme,
    font_size: f32,
    wrap_width: f32,
    hide_markers: bool,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = wrap_width;

    let base_color = theme.text_primary;
    let comment_color = theme.text_muted;
    let markup_tag_color = theme.accent;

    let mut line_start = true;
    let mut lines = text.split('\n').peekable();

    while let Some(line) = lines.next() {
        let trimmed_start = line.trim_start();
        let is_comment_line = trimmed_start.starts_with("//");

        if !line_start {
            let (nl_size, nl_color) = if hide_markers && is_comment_line {
                (0.01, Color32::TRANSPARENT)
            } else {
                (font_size, base_color)
            };
            job.append(
                "\n",
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::proportional(nl_size),
                    color: nl_color,
                    ..Default::default()
                },
            );
        }
        line_start = false;

        if trimmed_start.starts_with("#*") {
            // Entire line or prefix is a stanza tag
            let leading_spaces = &line[..line.len() - trimmed_start.len()];
            if !leading_spaces.is_empty() {
                job.append(
                    leading_spaces,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::proportional(font_size),
                        color: base_color,
                        ..Default::default()
                    },
                );
            }

            let tag_name = trimmed_start.trim_start_matches("#*").trim();
            let [r, g, b] = StropheTagDefinition::color_for(tags, tag_name, [100, 116, 139]);
            let tag_color = Color32::from_rgb(r, g, b);

            job.append(
                "#*",
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::proportional(font_size * 0.9),
                    color: tag_color.linear_multiply(0.7),
                    ..Default::default()
                },
            );
            job.append(
                &trimmed_start[2..],
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::new(font_size, egui::FontFamily::Name("inter_bold".into())),
                    color: tag_color,
                    ..Default::default()
                },
            );
        } else if is_comment_line {
            // Line comment or directive command
            if hide_markers {
                job.append(
                    line,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::proportional(0.01),
                        color: Color32::TRANSPARENT,
                        ..Default::default()
                    },
                );
            } else if trimmed_start.starts_with("//fonte:")
                || trimmed_start.starts_with("//font:")
                || trimmed_start.starts_with("//f:")
                || trimmed_start.starts_with("//f ")
            {
                job.append(
                    line,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::new(font_size * 0.95, egui::FontFamily::Name("inter_bold".into())),
                        color: Color32::from_rgb(168, 85, 247), // purple for font directive
                        ..Default::default()
                    },
                );
            } else if trimmed_start.starts_with("//cor:")
                || trimmed_start.starts_with("//color:")
                || trimmed_start.starts_with("//c:")
                || trimmed_start.starts_with("//c ")
            {
                let color_val = if let Some(hex) = trimmed_start.split(':').nth(1) {
                    crate::core::markup::parse_hex_color(hex).unwrap_or(Color32::from_rgb(251, 146, 60))
                } else {
                    Color32::from_rgb(251, 146, 60)
                };
                job.append(
                    line,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::new(font_size * 0.95, egui::FontFamily::Name("inter_bold".into())),
                        color: color_val,
                        ..Default::default()
                    },
                );
            } else if trimmed_start.starts_with("//tamanho:")
                || trimmed_start.starts_with("//size:")
                || trimmed_start.starts_with("//t:")
                || trimmed_start.starts_with("//s:")
                || trimmed_start.starts_with("//t ")
                || trimmed_start.starts_with("//s ")
            {
                job.append(
                    line,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::new(font_size * 0.95, egui::FontFamily::Name("inter_bold".into())),
                        color: Color32::from_rgb(56, 189, 248), // cyan for size
                        ..Default::default()
                    },
                );
            } else if trimmed_start.starts_with("//sombra:")
                || trimmed_start.starts_with("//shadow:")
                || trimmed_start.starts_with("//fundo:")
                || trimmed_start.starts_with("//box:")
                || trimmed_start.starts_with("//b:")
                || trimmed_start.starts_with("//b ")
            {
                let color_val = if let Some(hex) = trimmed_start.split(':').nth(1) {
                    crate::core::markup::parse_hex_color(hex).unwrap_or(Color32::from_rgb(148, 163, 184))
                } else {
                    Color32::from_rgb(148, 163, 184)
                };
                job.append(
                    line,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::new(font_size * 0.95, egui::FontFamily::Name("inter_bold".into())),
                        color: color_val,
                        ..Default::default()
                    },
                );
            } else if trimmed_start.starts_with("//alinhar:")
                || trimmed_start.starts_with("//align:")
                || trimmed_start.starts_with("//a:")
                || trimmed_start.starts_with("//a ")
            {
                job.append(
                    line,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::new(font_size * 0.95, egui::FontFamily::Name("inter_bold".into())),
                        color: Color32::from_rgb(96, 165, 250), // blue for alignment
                        ..Default::default()
                    },
                );
            } else if trimmed_start.starts_with("//linha:")
                || trimmed_start.starts_with("//line:")
                || trimmed_start.starts_with("//l:")
                || trimmed_start.starts_with("//l ")
                || trimmed_start.starts_with("//letra:")
                || trimmed_start.starts_with("//letter:")
                || trimmed_start.starts_with("//ls:")
                || trimmed_start.starts_with("//ls ")
            {
                job.append(
                    line,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::new(font_size * 0.95, egui::FontFamily::Name("inter_bold".into())),
                        color: Color32::from_rgb(52, 211, 153), // emerald for spacing
                        ..Default::default()
                    },
                );
            } else {
                job.append(
                    line,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::proportional(font_size),
                        color: comment_color,
                        italics: true,
                        ..Default::default()
                    },
                );
            }
        } else {
            // Normal line, possibly with inline comment `//` and/or markup `**...**`
            let (main_part, comment_part) = match line.find("//") {
                Some(idx) => {
                    if hide_markers {
                        let clean_main = line[..idx].trim_end();
                        (clean_main, Some(&line[clean_main.len()..]))
                    } else {
                        (&line[..idx], Some(&line[idx..]))
                    }
                }
                None => (line, None),
            };

            highlight_line_markup(&mut job, main_part, base_color, markup_tag_color, font_size, hide_markers);

            if let Some(comment) = comment_part {
                if hide_markers {
                    job.append(
                        comment,
                        0.0,
                        egui::TextFormat {
                            font_id: egui::FontId::proportional(0.01),
                            color: Color32::TRANSPARENT,
                            ..Default::default()
                        },
                    );
                } else {
                    job.append(
                        comment,
                        0.0,
                        egui::TextFormat {
                            font_id: egui::FontId::proportional(font_size),
                            color: comment_color,
                            italics: true,
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }

    job
}

fn highlight_line_markup(
    job: &mut egui::text::LayoutJob,
    text: &str,
    base_color: Color32,
    markup_color: Color32,
    font_size: f32,
    hide_markers: bool,
) {
    let mut remainder = text;
    let base_format = egui::TextFormat {
        font_id: egui::FontId::proportional(font_size),
        color: base_color,
        ..Default::default()
    };

    while !remainder.is_empty() {
        if let Some((start_idx, end_idx, open_len, close_len, marker_type)) = find_next_markup(remainder) {
            // Append plain text preceding the markup
            if start_idx > 0 {
                append_plain_or_breaks(job, &remainder[..start_idx], base_format.clone(), hide_markers);
            }

            // Determine marker color
            let tag_color = match &marker_type {
                MarkupType::Font(_) => Color32::from_rgb(168, 85, 247),
                MarkupType::Color(c) => *c,
                MarkupType::Size(_) => Color32::from_rgb(56, 189, 248),
                MarkupType::Shadow(c) => *c,
                _ => markup_color.linear_multiply(0.75),
            };

            // Append opening marker
            let marker_str = &remainder[start_idx..start_idx + open_len];
            if hide_markers {
                job.append(
                    marker_str,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::proportional(0.01),
                        color: Color32::TRANSPARENT,
                        ..Default::default()
                    },
                );
            } else {
                job.append(
                    marker_str,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::proportional(font_size * 0.9),
                        color: tag_color,
                        ..Default::default()
                    },
                );
            }

            // Append inner styled text
            let inner_text = &remainder[start_idx + open_len..end_idx - close_len];
            let mut format = egui::TextFormat {
                font_id: egui::FontId::proportional(font_size),
                color: base_color,
                ..Default::default()
            };
            match marker_type {
                MarkupType::Emphasis => {
                    format.font_id.family = egui::FontFamily::Name("inter_bold".into());
                    format.color = markup_color;
                }
                MarkupType::Italic => {
                    format.italics = true;
                }
                MarkupType::Underline => {
                    format.underline = Stroke::new(1.0, base_color);
                }
                MarkupType::Strikethrough => {
                    format.strikethrough = Stroke::new(1.0, base_color);
                }
                MarkupType::Font(name) => {
                    format.font_id.family = egui::FontFamily::Name(name.into());
                }
                MarkupType::Color(c) => {
                    format.color = c;
                }
                MarkupType::Size(_) => {
                    // Do not change font size in the text editor input field to keep typing area uniform
                }
                MarkupType::Shadow(c) => {
                    format.color = c;
                }
            }
            append_plain_or_breaks(job, inner_text, format, hide_markers);

            // Append closing marker
            let close_marker_str = &remainder[end_idx - close_len..end_idx];
            if hide_markers {
                job.append(
                    close_marker_str,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::proportional(0.01),
                        color: Color32::TRANSPARENT,
                        ..Default::default()
                    },
                );
            } else {
                job.append(
                    close_marker_str,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::proportional(font_size * 0.9),
                        color: tag_color,
                        ..Default::default()
                    },
                );
            }

            remainder = &remainder[end_idx..];
        } else {
            // No more markup
            append_plain_or_breaks(job, remainder, base_format, hide_markers);
            break;
        }
    }
}

fn append_plain_or_breaks(
    job: &mut egui::text::LayoutJob,
    text: &str,
    base_format: egui::TextFormat,
    hide_markers: bool,
) {
    let mut s = text;
    while let Some((start, end, token)) = crate::core::breaks::find_next_break(s) {
        if hide_markers {
            let (actual_start, actual_end) = if start > 0 && s.as_bytes()[start - 1] == b' ' && s[end..].starts_with(' ') {
                (start - 1, end)
            } else {
                (start, end)
            };

            if actual_start > 0 {
                job.append(&s[..actual_start], 0.0, base_format.clone());
            }

            job.append(
                &s[actual_start..actual_end],
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::proportional(0.01),
                    color: Color32::TRANSPARENT,
                    ..base_format.clone()
                },
            );
            s = &s[actual_end..];
        } else {
            if start > 0 {
                job.append(&s[..start], 0.0, base_format.clone());
            }
            let token_text = &s[start..end];
            let (token_color, token_font) = match token {
                crate::core::breaks::BreakToken::SoftBreak => (
                    Color32::from_rgb(56, 189, 248),
                    egui::FontId::new(base_format.font_id.size, egui::FontFamily::Name("inter_bold".into())),
                ),
                crate::core::breaks::BreakToken::Ratio43 => (
                    Color32::from_rgb(251, 146, 60),
                    egui::FontId::new(base_format.font_id.size * 0.95, egui::FontFamily::Name("inter_bold".into())),
                ),
                crate::core::breaks::BreakToken::Ratio169 => (
                    Color32::from_rgb(52, 211, 153),
                    egui::FontId::new(base_format.font_id.size * 0.95, egui::FontFamily::Name("inter_bold".into())),
                ),
                crate::core::breaks::BreakToken::Ratio916 => (
                    Color32::from_rgb(236, 72, 153), // pink for vertical / mobile
                    egui::FontId::new(base_format.font_id.size * 0.95, egui::FontFamily::Name("inter_bold".into())),
                ),
                crate::core::breaks::BreakToken::Ratio219 => (
                    Color32::from_rgb(168, 85, 247), // purple for ultrawide
                    egui::FontId::new(base_format.font_id.size * 0.95, egui::FontFamily::Name("inter_bold".into())),
                ),
            };
            job.append(
                token_text,
                0.0,
                egui::TextFormat {
                    font_id: token_font,
                    color: token_color,
                    ..base_format.clone()
                },
            );
            s = &s[end..];
        }
    }
    if !s.is_empty() {
        job.append(s, 0.0, base_format);
    }
}

#[derive(Clone, PartialEq)]
enum MarkupType {
    Emphasis,     // ** or ==
    Italic,       // *
    Underline,    // __
    Strikethrough,// ~~
    Font(String), // [fonte: Nome]...[/fonte]
    Color(Color32), // [cor: #Hex]...[/cor]
    Size(f32),    // [tamanho: N]...[/tamanho]
    Shadow(Color32), // [sombra: #Hex]...[/sombra]
}

fn find_next_markup(text: &str) -> Option<(usize, usize, usize, usize, MarkupType)> {
    let mut best: Option<(usize, usize, usize, usize, MarkupType)> = None;

    // 1. Check custom font tag: [f: Nome]...[/f] or [fonte: Nome]...[/fonte] or [font: Nome]...[/font]
    for prefix in &["[fonte:", "[font:", "[f:"] {
        if let Some(start) = text.find(prefix) {
            let after = &text[start..];
            if let Some(open_close) = after.find(']') {
                let open_tag_len = open_close + 1;
                let font_name = after[prefix.len()..open_close].trim().to_string();
                let rest = &after[open_tag_len..];
                for close_tag in &["[/fonte]", "[/font]", "[/f]"] {
                    if let Some(close_idx) = rest.find(close_tag) {
                        let end = start + open_tag_len + close_idx + close_tag.len();
                        if best.is_none() || start < best.as_ref().unwrap().0 {
                            best = Some((start, end, open_tag_len, close_tag.len(), MarkupType::Font(font_name.clone())));
                        }
                    }
                }
            }
        }
    }

    // 2. Check custom color tag: [c: #Hex]...[/c] or [cor: #Hex]...[/cor] or [color: #Hex]...[/color]
    for prefix in &["[cor:", "[color:", "[c:"] {
        if let Some(start) = text.find(prefix) {
            let after = &text[start..];
            if let Some(open_close) = after.find(']') {
                let open_tag_len = open_close + 1;
                let color_str = after[prefix.len()..open_close].trim();
                let color = crate::core::markup::parse_hex_color(color_str)
                    .unwrap_or(Color32::from_rgb(251, 146, 60));
                let rest = &after[open_tag_len..];
                for close_tag in &["[/cor]", "[/color]", "[/c]"] {
                    if let Some(close_idx) = rest.find(close_tag) {
                        let end = start + open_tag_len + close_idx + close_tag.len();
                        if best.is_none() || start < best.as_ref().unwrap().0 {
                            best = Some((start, end, open_tag_len, close_tag.len(), MarkupType::Color(color)));
                        }
                    }
                }
            }
        }
    }

    // 3. Check custom size tag: [t: N]...[/t] or [tamanho: N]...[/tamanho] or [size: N]...[/size] or [s: N]...[/s]
    for prefix in &["[tamanho:", "[size:", "[t:", "[s:"] {
        if let Some(start) = text.find(prefix) {
            let after = &text[start..];
            if let Some(open_close) = after.find(']') {
                let open_tag_len = open_close + 1;
                let sz = after[prefix.len()..open_close].trim().parse::<f32>().unwrap_or(46.0);
                let rest = &after[open_tag_len..];
                for close_tag in &["[/tamanho]", "[/size]", "[/t]", "[/s]"] {
                    if let Some(close_idx) = rest.find(close_tag) {
                        let end = start + open_tag_len + close_idx + close_tag.len();
                        if best.is_none() || start < best.as_ref().unwrap().0 {
                            best = Some((start, end, open_tag_len, close_tag.len(), MarkupType::Size(sz)));
                        }
                    }
                }
            }
        }
    }

    // 4. Check custom shadow tag: [b: #Hex]...[/b] or [sombra: #Hex]...[/sombra] or [shadow: #Hex]...[/shadow]
    for prefix in &["[sombra:", "[shadow:", "[fundo:", "[box:", "[b:"] {
        if let Some(start) = text.find(prefix) {
            let after = &text[start..];
            if let Some(open_close) = after.find(']') {
                let open_tag_len = open_close + 1;
                let color_str = after[prefix.len()..open_close].trim();
                let color = crate::core::markup::parse_hex_color(color_str)
                    .unwrap_or(Color32::from_rgb(148, 163, 184));
                let rest = &after[open_tag_len..];
                for close_tag in &["[/sombra]", "[/shadow]", "[/fundo]", "[/box]", "[/b]"] {
                    if let Some(close_idx) = rest.find(close_tag) {
                        let end = start + open_tag_len + close_idx + close_tag.len();
                        if best.is_none() || start < best.as_ref().unwrap().0 {
                            best = Some((start, end, open_tag_len, close_tag.len(), MarkupType::Shadow(color)));
                        }
                    }
                }
            }
        }
    }

    // 5. Check markdown syntax
    let patterns = [
        ("**", MarkupType::Emphasis),
        ("==", MarkupType::Emphasis),
        ("__", MarkupType::Underline),
        ("~~", MarkupType::Strikethrough),
        ("*", MarkupType::Italic),
    ];

    for (pat, m_type) in patterns {
        if let Some(start) = text.find(pat) {
            // Avoid matching single * inside **
            if pat == "*" && (text[start..].starts_with("**") || (start > 0 && text[..start].ends_with('*'))) {
                continue;
            }

            let after_open = start + pat.len();
            if let Some(rel_close) = text[after_open..].find(pat) {
                if rel_close > 0 {
                    let end = after_open + rel_close + pat.len();
                    if best.is_none() || start < best.as_ref().unwrap().0 {
                        best = Some((start, end, pat.len(), pat.len(), m_type));
                    }
                }
            }
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_size_zero_width() {
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(Default::default(), |_| {});
        out.textures_delta.clear();
        let theme = AppTheme::default();
        let text = "Diante do trono, [4:3] Senhor,\nquero levar minha [4:3] oferta de amor // nota\n// Linha comentada\nMais texto";

        let job_hidden = highlight_lyrics_job(text, &[], &theme, 14.0, 500.0, true);
        let galley_hidden = ctx.fonts_mut(|f| f.layout_job(job_hidden));

        let job_plain = highlight_lyrics_job("Diante do trono, Senhor,\nquero levar minha oferta de amor\n\nMais texto", &[], &theme, 14.0, 500.0, false);
        let galley_plain = ctx.fonts_mut(|f| f.layout_job(job_plain));

        println!("Galley hidden rect: {:?}", galley_hidden.rect);
        println!("Galley plain rect: {:?}", galley_plain.rect);
        assert!((galley_hidden.rect.width() - galley_plain.rect.width()).abs() < 10.0);
    }

    #[test]
    fn test_size_markup_keeps_editor_font_size() {
        let theme = AppTheme::default();
        let text = "Bless the [t: 60]Lord[/t], oh my soul";
        let job = highlight_lyrics_job(text, &[], &theme, 14.0, 500.0, false);
        let mut found = false;
        for section in &job.sections {
            let start = usize::from(section.byte_range.start);
            let end = usize::from(section.byte_range.end);
            let slice = &text[start..end];
            if slice == "Lord" {
                assert_eq!(section.format.font_id.size, 14.0);
                found = true;
            }
        }
        assert!(found, "Should find section for 'Lord'");
    }
}
