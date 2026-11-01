//! Inline lyric markup used for per-word emphasis, custom fonts, colors, sizes, and slide directives.
//!
//! Supported syntax:
//! - Markdown markers: `**texto**` (bold), `*texto*` (italic), `__texto__` (underline), `~~texto~~` (strikethrough), `==texto==` (emphasis)
//! - Selection font tag: `[fonte: Nome]texto[/fonte]` or `[font: Nome]texto[/font]`
//! - Selection color tag: `[cor: #HEX]texto[/cor]` or `[color: #HEX]texto[/color]`
//! - Selection size tag: `[tamanho: 46]texto[/tamanho]` or `[size: 46]texto[/size]`
//! - Selection shadow tag: `[sombra: #HEX]texto[/sombra]` or `[shadow: #HEX]texto[/shadow]`
//! - Line commands:
//!   - `//fonte: Nome` (slide default font family)
//!   - `//cor: #HEX` (slide default text color)
//!   - `//tamanho: N` (slide font size)
//!   - `//sombra: #HEX` (slide shadow/box color)
//!   - `//alinhar: centro|esquerda|direita|justificado` (slide text alignment)
//!   - `//linha: N` (slide line spacing, e.g. 1.38)
//!   - `//letra: N` (slide letter spacing, e.g. 1.8)

use eframe::egui::Color32;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SlideDirectives {
    pub font_family: Option<String>,
    pub color: Option<Color32>,
    pub font_size: Option<f32>,
    pub shadow_color: Option<Color32>,
    pub align: Option<String>,
    pub line_spacing: Option<f32>,
    pub letter_spacing: Option<f32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MarkupStyle {
    pub emphasis: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub font_family: Option<String>,
    pub color: Option<Color32>,
    pub font_size: Option<f32>,
    pub shadow_color: Option<Color32>,
}

impl MarkupStyle {
    pub fn is_plain(&self) -> bool {
        !self.emphasis
            && !self.italic
            && !self.underline
            && !self.strikethrough
            && self.font_family.is_none()
            && self.color.is_none()
            && self.font_size.is_none()
            && self.shadow_color.is_none()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MarkupRun<'a> {
    pub text: &'a str,
    pub style: MarkupStyle,
}

pub fn parse_hex_color(s: &str) -> Option<Color32> {
    let s = s.trim().trim_start_matches('#');
    if s.len() == 3 {
        let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
        let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
        let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
        Some(Color32::from_rgb(r, g, b))
    } else if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Color32::from_rgb(r, g, b))
    } else if s.len() == 8 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        let a = u8::from_str_radix(&s[6..8], 16).ok()?;
        Some(Color32::from_rgba_premultiplied(r, g, b, a))
    } else {
        None
    }
}

pub fn is_directive_command(trimmed: &str) -> bool {
    let s = trimmed.trim_start();
    let after_slash = if let Some(rest) = s.strip_prefix("//") {
        rest.trim_start()
    } else {
        return false;
    };
    for prefix in &[
        "fonte:", "font:", "cor:", "color:", "tamanho:", "size:",
        "sombra:", "shadow:", "fundo:", "alinhar:", "align:",
        "linha:", "line:", "letra:", "letter:",
    ] {
        if after_slash.starts_with(prefix) {
            return true;
        }
    }
    false
}

fn strip_command_prefix<'a>(trimmed: &'a str, cmd: &str) -> Option<&'a str> {
    let prefix1 = format!("//{cmd}:");
    let prefix2 = format!("//{cmd} ");
    let prefix3 = format!("// {cmd}:");
    let prefix4 = format!("// {cmd} ");

    if let Some(rest) = trimmed.strip_prefix(&prefix1) {
        Some(rest.trim())
    } else if let Some(rest) = trimmed.strip_prefix(&prefix2) {
        Some(rest.trim())
    } else if let Some(rest) = trimmed.strip_prefix(&prefix3) {
        Some(rest.trim())
    } else if let Some(rest) = trimmed.strip_prefix(&prefix4) {
        Some(rest.trim())
    } else {
        None
    }
}

pub fn parse_slide_directives(text: &str) -> SlideDirectives {
    let mut dirs = SlideDirectives::default();
    for line in text.split('\n') {
        let trimmed = line.trim();
        if let Some(font) = strip_command_prefix(trimmed, "fonte").or_else(|| strip_command_prefix(trimmed, "font")) {
            if !font.is_empty() {
                dirs.font_family = Some(font.to_string());
            }
        }
        if let Some(color_str) = strip_command_prefix(trimmed, "cor").or_else(|| strip_command_prefix(trimmed, "color")) {
            if let Some(c) = parse_hex_color(color_str) {
                dirs.color = Some(c);
            }
        }
        if let Some(size_str) = strip_command_prefix(trimmed, "tamanho").or_else(|| strip_command_prefix(trimmed, "size")) {
            if let Ok(val) = size_str.trim().parse::<f32>() {
                if val > 0.0 {
                    dirs.font_size = Some(val);
                }
            }
        }
        if let Some(shadow_str) = strip_command_prefix(trimmed, "sombra")
            .or_else(|| strip_command_prefix(trimmed, "shadow"))
            .or_else(|| strip_command_prefix(trimmed, "fundo"))
        {
            if let Some(c) = parse_hex_color(shadow_str) {
                dirs.shadow_color = Some(c);
            }
        }
        if let Some(align_str) = strip_command_prefix(trimmed, "alinhar").or_else(|| strip_command_prefix(trimmed, "align")) {
            let lower = align_str.to_lowercase();
            let norm = match lower.as_str() {
                "centro" | "center" => "center",
                "direita" | "right" => "right",
                "justificado" | "justify" => "justify",
                "esquerda" | "left" => "left",
                _ => align_str.trim(),
            };
            dirs.align = Some(norm.to_string());
        }
        if let Some(line_str) = strip_command_prefix(trimmed, "linha").or_else(|| strip_command_prefix(trimmed, "line")) {
            if let Ok(val) = line_str.trim().parse::<f32>() {
                if val > 0.0 {
                    dirs.line_spacing = Some(val);
                }
            }
        }
        if let Some(letter_str) = strip_command_prefix(trimmed, "letra").or_else(|| strip_command_prefix(trimmed, "letter")) {
            if let Ok(val) = letter_str.trim().parse::<f32>() {
                dirs.letter_spacing = Some(val);
            }
        }
    }
    dirs
}

/// Splits text into styled runs, honoring inline tags `[fonte: ...]`, `[cor: ...]`, `[tamanho: ...]`,
/// line commands `//fonte: ...`, `//cor: ...`, `//tamanho: ...`, etc., and markdown markers.
pub fn runs(text: &str) -> Vec<MarkupRun<'_>> {
    let mut out = Vec::new();
    let dirs = parse_slide_directives(text);

    let default_font = dirs.font_family.clone();
    let default_color = dirs.color;
    let default_size = dirs.font_size;
    let default_shadow = dirs.shadow_color;

    let mut first_line = true;
    for line in text.split('\n') {
        let trimmed = line.trim();
        if is_directive_command(trimmed) {
            // Skip line commands from visual slide text
            continue;
        }

        if !first_line {
            out.push(MarkupRun {
                text: "\n",
                style: MarkupStyle {
                    font_family: default_font.clone(),
                    color: default_color,
                    font_size: default_size,
                    shadow_color: default_shadow,
                    ..Default::default()
                },
            });
        }
        first_line = false;

        parse_line_runs(
            &mut out,
            line,
            default_font.as_deref(),
            default_color,
            default_size,
            default_shadow,
        );
    }

    out
}

fn parse_line_runs<'a>(
    out: &mut Vec<MarkupRun<'a>>,
    line: &'a str,
    default_font: Option<&str>,
    default_color: Option<Color32>,
    default_size: Option<f32>,
    default_shadow: Option<Color32>,
) {
    let mut font_stack: Vec<String> = default_font.map(|f| vec![f.to_string()]).unwrap_or_default();
    let mut color_stack: Vec<Color32> = default_color.map(|c| vec![c]).unwrap_or_default();
    let mut size_stack: Vec<f32> = default_size.map(|s| vec![s]).unwrap_or_default();
    let mut shadow_stack: Vec<Color32> = default_shadow.map(|s| vec![s]).unwrap_or_default();

    let mut rest = line;

    while !rest.is_empty() {
        // 1. Tag [fonte: Nome] or [font: Nome]
        if let Some((tag_len, font_name)) = match_open_tag(rest, &["fonte:", "font:"]) {
            font_stack.push(font_name.to_string());
            rest = &rest[tag_len..];
            continue;
        }
        // 2. Tag [/fonte] or [/font]
        if let Some(tag_len) = match_close_tag(rest, &["/fonte", "/font"]) {
            if font_stack.len() > usize::from(default_font.is_some()) {
                font_stack.pop();
            }
            rest = &rest[tag_len..];
            continue;
        }
        // 3. Tag [cor: #HEX] or [color: #HEX]
        if let Some((tag_len, color_str)) = match_open_tag(rest, &["cor:", "color:"]) {
            if let Some(c) = parse_hex_color(color_str) {
                color_stack.push(c);
            }
            rest = &rest[tag_len..];
            continue;
        }
        // 4. Tag [/cor] or [/color]
        if let Some(tag_len) = match_close_tag(rest, &["/cor", "/color"]) {
            if color_stack.len() > usize::from(default_color.is_some()) {
                color_stack.pop();
            }
            rest = &rest[tag_len..];
            continue;
        }
        // 5. Tag [tamanho: N] or [size: N]
        if let Some((tag_len, size_str)) = match_open_tag(rest, &["tamanho:", "size:"]) {
            if let Ok(val) = size_str.parse::<f32>() {
                if val > 0.0 {
                    size_stack.push(val);
                }
            }
            rest = &rest[tag_len..];
            continue;
        }
        // 6. Tag [/tamanho] or [/size]
        if let Some(tag_len) = match_close_tag(rest, &["/tamanho", "/size"]) {
            if size_stack.len() > usize::from(default_size.is_some()) {
                size_stack.pop();
            }
            rest = &rest[tag_len..];
            continue;
        }
        // 7. Tag [sombra: #HEX] or [shadow: #HEX]
        if let Some((tag_len, shadow_str)) = match_open_tag(rest, &["sombra:", "shadow:", "fundo:"]) {
            if let Some(c) = parse_hex_color(shadow_str) {
                shadow_stack.push(c);
            }
            rest = &rest[tag_len..];
            continue;
        }
        // 8. Tag [/sombra] or [/shadow]
        if let Some(tag_len) = match_close_tag(rest, &["/sombra", "/shadow", "/fundo"]) {
            if shadow_stack.len() > usize::from(default_shadow.is_some()) {
                shadow_stack.pop();
            }
            rest = &rest[tag_len..];
            continue;
        }

        // 9. Markdown markers: **bold**, ~~strikethrough~~, __underline__, *italic*, ==emphasis==
        if let Some((marker_pat, is_bold, is_strike, is_under, is_italic, is_emp)) = match_markdown_marker(rest) {
            let after_open = &rest[marker_pat.len()..];
            if let Some(close_idx) = after_open.find(marker_pat) {
                let inner = &after_open[..close_idx];
                if !inner.is_empty() {
                    let style = MarkupStyle {
                        emphasis: is_bold || is_emp,
                        strikethrough: is_strike,
                        underline: is_under,
                        italic: is_italic,
                        font_family: font_stack.last().cloned(),
                        color: color_stack.last().copied(),
                        font_size: size_stack.last().copied(),
                        shadow_color: shadow_stack.last().copied(),
                    };
                    out.push(MarkupRun { text: inner, style });
                    rest = &after_open[close_idx + marker_pat.len()..];
                    continue;
                }
            }
        }

        // 10. Find next delimiter (tag or markdown marker)
        let next_delim = find_next_delimiter(rest);
        let (plain, tail) = rest.split_at(next_delim);
        if !plain.is_empty() {
            let style = MarkupStyle {
                emphasis: false,
                italic: false,
                underline: false,
                strikethrough: false,
                font_family: font_stack.last().cloned(),
                color: color_stack.last().copied(),
                font_size: size_stack.last().copied(),
                shadow_color: shadow_stack.last().copied(),
            };
            out.push(MarkupRun { text: plain, style });
        }
        rest = tail;
    }
}

fn match_open_tag<'a>(text: &'a str, prefixes: &[&str]) -> Option<(usize, &'a str)> {
    if !text.starts_with('[') {
        return None;
    }
    let close = text.find(']')?;
    let inside = text[1..close].trim();
    for prefix in prefixes {
        if let Some(val) = inside.strip_prefix(prefix) {
            return Some((close + 1, val.trim()));
        }
    }
    None
}

fn match_close_tag(text: &str, tags: &[&str]) -> Option<usize> {
    if !text.starts_with('[') {
        return None;
    }
    let close = text.find(']')?;
    let inside = text[1..close].trim();
    for tag in tags {
        if inside.eq_ignore_ascii_case(tag) {
            return Some(close + 1);
        }
    }
    None
}

fn match_markdown_marker(text: &str) -> Option<(&'static str, bool, bool, bool, bool, bool)> {
    if text.starts_with("**") {
        Some(("**", true, false, false, false, false))
    } else if text.starts_with("~~") {
        Some(("~~", false, true, false, false, false))
    } else if text.starts_with("__") {
        Some(("__", false, false, true, false, false))
    } else if text.starts_with("==") {
        Some(("==", false, false, false, false, true))
    } else if text.starts_with('*') {
        Some(("*", false, false, false, true, false))
    } else {
        None
    }
}

fn find_next_delimiter(text: &str) -> usize {
    let mut min_idx = text.len();
    for pat in ["[", "**", "~~", "__", "==", "*"] {
        if let Some(idx) = text.find(pat) {
            if idx > 0 && idx < min_idx {
                min_idx = idx;
            } else if idx == 0 {
                // If it starts with an unmatched delimiter, advance past 1 char
                let len = text.chars().next().map(char::len_utf8).unwrap_or(1);
                return len;
            }
        }
    }
    min_idx
}

/// Removes all markup markers and tags, returning clean text for search or labels.
pub fn strip(text: &str) -> String {
    runs(text).into_iter().map(|run| run.text).collect()
}

/// Applies (or removes) custom font around a character selection.
pub fn apply_font_to_selection(
    text: &str,
    start: usize,
    end: usize,
    font_name: &str,
) -> (String, usize, usize) {
    apply_generic_tag_to_selection(text, start, end, "fonte", font_name)
}

/// Applies (or removes) custom color around a character selection.
pub fn apply_color_to_selection(
    text: &str,
    start: usize,
    end: usize,
    hex: &str,
) -> (String, usize, usize) {
    apply_generic_tag_to_selection(text, start, end, "cor", hex)
}

/// Applies (or removes) custom size tag around a character selection.
pub fn apply_size_to_selection(
    text: &str,
    start: usize,
    end: usize,
    size_str: &str,
) -> (String, usize, usize) {
    apply_generic_tag_to_selection(text, start, end, "tamanho", size_str)
}

/// Applies (or removes) custom shadow tag around a character selection.
pub fn apply_shadow_to_selection(
    text: &str,
    start: usize,
    end: usize,
    hex: &str,
) -> (String, usize, usize) {
    apply_generic_tag_to_selection(text, start, end, "sombra", hex)
}

fn apply_generic_tag_to_selection(
    text: &str,
    start: usize,
    end: usize,
    tag_name: &str,
    val: &str,
) -> (String, usize, usize) {
    if start == end {
        return (text.to_string(), start, end);
    }
    let chars: Vec<char> = text.chars().collect();
    let (start, end) = (start.min(chars.len()), end.min(chars.len()));
    let (start, end) = if start <= end { (start, end) } else { (end, start) };

    let offset = |idx: usize| chars[..idx].iter().map(|c| c.len_utf8()).sum::<usize>();
    let byte_start = offset(start);
    let byte_end = offset(end);
    let selected = &text[byte_start..byte_end];

    let val = val.trim();
    let open_prefix = format!("[{tag_name}:");
    let close_tag = format!("[/{tag_name}]");

    if selected.starts_with(&open_prefix) && selected.ends_with(&close_tag) {
        let inner = if let Some(open_end) = selected.find(']') {
            let close_start = selected.rfind(&close_tag).unwrap_or(selected.len());
            &selected[open_end + 1..close_start]
        } else {
            selected
        };

        if val.is_empty() {
            // Remove tag
            let mut updated = String::with_capacity(text.len());
            updated.push_str(&text[..byte_start]);
            updated.push_str(inner);
            updated.push_str(&text[byte_end..]);
            let removed_len = selected.chars().count() - inner.chars().count();
            return (updated, start, end.saturating_sub(removed_len));
        } else {
            // Replace tag value
            let new_tag = format!("[{tag_name}: {val}]{inner}{close_tag}");
            let mut updated = String::with_capacity(text.len() + new_tag.len());
            updated.push_str(&text[..byte_start]);
            updated.push_str(&new_tag);
            updated.push_str(&text[byte_end..]);
            let new_len = new_tag.chars().count();
            return (updated, start, start + new_len);
        }
    }

    if val.is_empty() {
        return (text.to_string(), start, end);
    }

    let wrapped = format!("[{tag_name}: {val}]{selected}{close_tag}");
    let mut updated = String::with_capacity(text.len() + wrapped.len());
    updated.push_str(&text[..byte_start]);
    updated.push_str(&wrapped);
    updated.push_str(&text[byte_end..]);
    let new_len = wrapped.chars().count();
    (updated, start, start + new_len)
}

/// Clears markdown formatting (`**`, `*`, `_`, `~`, `==`, `[fonte: ...]`, `[cor: ...]`, `[tamanho: ...]`, `[sombra: ...]`) from selection.
pub fn clear_selection_formatting(
    text: &str,
    start: usize,
    end: usize,
) -> (String, usize, usize) {
    if start == end {
        return (text.to_string(), start, end);
    }
    let chars: Vec<char> = text.chars().collect();
    let (start, end) = (start.min(chars.len()), end.min(chars.len()));
    let (start, end) = if start <= end { (start, end) } else { (end, start) };

    let offset = |idx: usize| chars[..idx].iter().map(|c| c.len_utf8()).sum::<usize>();
    let byte_start = offset(start);
    let byte_end = offset(end);
    let selected = &text[byte_start..byte_end];

    // Strip markers from selected slice
    let mut clean = String::with_capacity(selected.len());
    let mut rest = selected;
    while !rest.is_empty() {
        if let Some((len, _)) = match_open_tag(rest, &["fonte:", "font:", "cor:", "color:", "tamanho:", "size:", "sombra:", "shadow:", "alinhar:", "align:"]) {
            rest = &rest[len..];
            continue;
        }
        if let Some(len) = match_close_tag(rest, &["/fonte", "/font", "/cor", "/color", "/tamanho", "/size", "/sombra", "/shadow", "/alinhar", "/align"]) {
            rest = &rest[len..];
            continue;
        }
        if rest.starts_with("**") || rest.starts_with("~~") || rest.starts_with("__") || rest.starts_with("==") {
            rest = &rest[2..];
            continue;
        }
        if rest.starts_with('*') || rest.starts_with('_') || rest.starts_with('~') {
            rest = &rest[1..];
            continue;
        }

        let next_ch = rest.chars().next().unwrap();
        clean.push(next_ch);
        rest = &rest[next_ch.len_utf8()..];
    }

    let mut updated = String::with_capacity(text.len());
    updated.push_str(&text[..byte_start]);
    updated.push_str(&clean);
    updated.push_str(&text[byte_end..]);

    let clean_len = clean.chars().count();
    (updated, start, start + clean_len)
}

/// Helper for toggling emphasis in the lyric editor text.
pub fn toggle_emphasis(text: &str, start: usize, end: usize) -> Option<(String, usize, usize)> {
    if start == end {
        return None;
    }
    let chars: Vec<char> = text.chars().collect();
    let start = start.min(chars.len());
    let end = end.min(chars.len());
    let (start, end) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };
    if start == end {
        return None;
    }

    let offset = |idx: usize| chars[..idx].iter().map(|c| c.len_utf8()).sum::<usize>();
    let byte_start = offset(start);
    let byte_end = offset(end);
    let selected: String = text[byte_start..byte_end].to_owned();

    if selected.starts_with("**") && selected.ends_with("**") && selected.len() > 4 {
        let inner = &selected[2..selected.len() - 2];
        let mut updated = String::with_capacity(text.len() - 4);
        updated.push_str(&text[..byte_start]);
        updated.push_str(inner);
        updated.push_str(&text[byte_end..]);
        return Some((updated, start, end - 4));
    }

    let before = &text[..byte_start];
    let after = &text[byte_end..];
    if before.ends_with("**") && after.starts_with("**") {
        let new_before = &before[..before.len() - 2];
        let new_after = &after[2..];
        let mut updated = String::with_capacity(text.len() - 4);
        updated.push_str(new_before);
        updated.push_str(&selected);
        updated.push_str(new_after);
        return Some((updated, start - 2, end - 2));
    }

    let mut updated = String::with_capacity(text.len() + 4);
    updated.push_str(before);
    updated.push_str("**");
    updated.push_str(&selected);
    updated.push_str("**");
    updated.push_str(after);
    Some((updated, start + 2, end + 2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_emphasis_and_plain_text() {
        let parsed = runs("Ele é **Cristo** vivo");
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].text, "Ele é ");
        assert!(parsed[0].style.is_plain());
        assert_eq!(parsed[1].text, "Cristo");
        assert!(parsed[1].style.emphasis);
        assert_eq!(parsed[2].text, " vivo");
    }

    #[test]
    fn parses_font_and_color_tags() {
        let parsed = runs("Diante do trono, [cor: #FFD700]Senhor[/cor], [fonte: Montserrat]oferta[/fonte]");
        let lord = parsed.iter().find(|r| r.text == "Senhor").unwrap();
        assert_eq!(lord.style.color, Some(Color32::from_rgb(255, 215, 0)));

        let offer = parsed.iter().find(|r| r.text == "oferta").unwrap();
        assert_eq!(offer.style.font_family.as_deref(), Some("Montserrat"));
    }

    #[test]
    fn parses_line_commands() {
        let text = "//fonte: Montserrat\n//cor: #00FF00\n//tamanho: 52\n//alinhar: centro\nDiante do trono\nSenhor";
        let dirs = parse_slide_directives(text);
        assert_eq!(dirs.font_family.as_deref(), Some("Montserrat"));
        assert_eq!(dirs.color, Some(Color32::from_rgb(0, 255, 0)));
        assert_eq!(dirs.font_size, Some(52.0));
        assert_eq!(dirs.align.as_deref(), Some("center"));

        let parsed = runs(text);
        assert_eq!(parsed.len(), 3); // "Diante do trono", "\n", "Senhor"
        assert_eq!(parsed[0].text, "Diante do trono");
        assert_eq!(parsed[0].style.font_family.as_deref(), Some("Montserrat"));
        assert_eq!(parsed[0].style.color, Some(Color32::from_rgb(0, 255, 0)));
        assert_eq!(parsed[0].style.font_size, Some(52.0));
        assert_eq!(parsed[2].text, "Senhor");
    }

    #[test]
    fn applies_font_and_color_to_selection() {
        let text = "Diante do trono Senhor";
        let (updated, _, _) = apply_font_to_selection(text, 16, 22, "Inter");
        assert_eq!(updated, "Diante do trono [fonte: Inter]Senhor[/fonte]");

        let (updated_color, _, _) = apply_color_to_selection(&updated, 30, 36, "#FF5500");
        assert_eq!(updated_color, "Diante do trono [fonte: Inter][cor: #FF5500]Senhor[/cor][/fonte]");
    }

    #[test]
    fn strips_all_supported_markers() {
        assert_eq!(
            strip("**forte** *itálico* __linha__ ~~risco~~ ==marca== [cor: #FFF]cor[/cor] [fonte: Arial]fonte[/fonte]"),
            "forte itálico linha risco marca cor fonte"
        );
    }
}
