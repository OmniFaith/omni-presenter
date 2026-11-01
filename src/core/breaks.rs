//! Conditional and format-specific line break processing.
//!
//! Supported markers:
//! - `|` or `[br]`: Soft / conditional break. Breaks to newline on narrow / 4:3 displays (`aspect_ratio <= 1.45`),
//!   or collapses into a single space on widescreen / 16:9 displays (`aspect_ratio > 1.45`).
//! - `[4:3]`, `[4x3]`, `|4:3|`, `[quebra: 4:3]`: Explicit break for 4:3 displays. Breaks to newline on `aspect_ratio <= 1.45`.
//! - `[16:9]`, `[16x9]`, `|16:9|`, `[quebra: 16:9]`: Explicit break for 16:9 displays. Breaks to newline on `1.45 < aspect_ratio < 2.1`.
//! - `[9:16]`, `[9x16]`, `|9:16|`, `[quebra: 9:16]`: Explicit break for portrait/vertical displays. Breaks on `aspect_ratio < 0.85`.
//! - `[21:9]`, `[21x9]`, `|21:9|`, `[quebra: 21:9]`: Explicit break for ultrawide displays. Breaks on `aspect_ratio >= 2.1`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakToken {
    SoftBreak,
    Ratio43,
    Ratio169,
    Ratio916,
    Ratio219,
}

impl BreakToken {
    pub fn should_break_aspect(self, aspect_ratio: f32) -> bool {
        match self {
            Self::SoftBreak => is_4_3_aspect_ratio(aspect_ratio),
            Self::Ratio43 => is_4_3_aspect_ratio(aspect_ratio),
            Self::Ratio169 => is_16_9_aspect_ratio(aspect_ratio),
            Self::Ratio916 => is_vertical_aspect_ratio(aspect_ratio),
            Self::Ratio219 => is_ultrawide_aspect_ratio(aspect_ratio),
        }
    }
}

pub fn is_4_3_aspect_ratio(aspect_ratio: f32) -> bool {
    aspect_ratio.is_finite() && aspect_ratio > 0.85 && aspect_ratio <= 1.45
}

pub fn is_16_9_aspect_ratio(aspect_ratio: f32) -> bool {
    aspect_ratio.is_finite() && aspect_ratio > 1.45 && aspect_ratio < 2.1
}

pub fn is_vertical_aspect_ratio(aspect_ratio: f32) -> bool {
    aspect_ratio.is_finite() && aspect_ratio > 0.0 && aspect_ratio <= 0.85
}

pub fn is_ultrawide_aspect_ratio(aspect_ratio: f32) -> bool {
    aspect_ratio.is_finite() && aspect_ratio >= 2.1
}

/// Finds the next break token in `text`, returning `(byte_start, byte_end, token)`.
pub fn find_next_break(text: &str) -> Option<(usize, usize, BreakToken)> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut idx = 0;

    while idx < len {
        // 1. Pipe format checks: `|4:3|`, `|16:9|`, `|9:16|`, `|21:9|` or simple `|`
        if bytes[idx] == b'|' {
            let slice = &text[idx..];
            if slice.starts_with("|4:3|") || slice.starts_with("|4x3|") {
                return Some((idx, idx + 5, BreakToken::Ratio43));
            }
            if slice.starts_with("|16:9|") || slice.starts_with("|16x9|") {
                return Some((idx, idx + 6, BreakToken::Ratio169));
            }
            if slice.starts_with("|9:16|") || slice.starts_with("|9x16|") {
                return Some((idx, idx + 6, BreakToken::Ratio916));
            }
            if slice.starts_with("|21:9|") || slice.starts_with("|21x9|") {
                return Some((idx, idx + 6, BreakToken::Ratio219));
            }
            // Standard single pipe soft break
            return Some((idx, idx + 1, BreakToken::SoftBreak));
        }

        // 2. Bracket format checks: `[4:3]`, `[16:9]`, `[9:16]`, `[21:9]`, `[br]`, `[quebra: ...]`
        if bytes[idx] == b'[' {
            let slice = &text[idx..];
            if slice.len() >= 5 {
                let tag5 = &slice[..5];
                if tag5.eq_ignore_ascii_case("[4:3]") || tag5.eq_ignore_ascii_case("[4x3]") {
                    return Some((idx, idx + 5, BreakToken::Ratio43));
                }
            }
            if slice.len() >= 6 {
                let tag6 = &slice[..6];
                if tag6.eq_ignore_ascii_case("[16:9]") || tag6.eq_ignore_ascii_case("[16x9]") {
                    return Some((idx, idx + 6, BreakToken::Ratio169));
                }
                if tag6.eq_ignore_ascii_case("[9:16]") || tag6.eq_ignore_ascii_case("[9x16]") {
                    return Some((idx, idx + 6, BreakToken::Ratio916));
                }
                if tag6.eq_ignore_ascii_case("[21:9]") || tag6.eq_ignore_ascii_case("[21x9]") {
                    return Some((idx, idx + 6, BreakToken::Ratio219));
                }
            }
            if slice.len() >= 4 && slice[..4].eq_ignore_ascii_case("[br]") {
                return Some((idx, idx + 4, BreakToken::SoftBreak));
            }
            // Extended syntax `[quebra: 4:3]`, `[quebra: 16:9]`, `[quebra: 9:16]`
            if let Some(close) = slice.find(']') {
                let inside = &slice[1..close].trim();
                if let Some(arg) = inside.strip_prefix("quebra:") {
                    let clean_arg = arg.trim();
                    if clean_arg == "4:3" || clean_arg == "4x3" {
                        return Some((idx, idx + close + 1, BreakToken::Ratio43));
                    }
                    if clean_arg == "16:9" || clean_arg == "16x9" {
                        return Some((idx, idx + close + 1, BreakToken::Ratio169));
                    }
                    if clean_arg == "9:16" || clean_arg == "9x16" {
                        return Some((idx, idx + close + 1, BreakToken::Ratio916));
                    }
                    if clean_arg == "21:9" || clean_arg == "21x9" {
                        return Some((idx, idx + close + 1, BreakToken::Ratio219));
                    }
                }
            }
        }

        // Advance utf-8 character
        if let Some(c) = text[idx..].chars().next() {
            idx += c.len_utf8();
        } else {
            break;
        }
    }
    None
}

/// Resolves all conditional and aspect-ratio-specific line breaks in `text`
/// according to the target aspect ratio (e.g. 1.777 for 16:9, 1.333 for 4:3, 0.562 for 9:16).
pub fn resolve_conditional_breaks(text: &str, aspect_ratio: f32) -> String {
    let mut out = String::with_capacity(text.len());
    let mut first_line = true;

    for raw_line in text.split('\n') {
        if !first_line {
            out.push('\n');
        }
        first_line = false;

        let mut remainder = raw_line;
        while let Some((start, end, token)) = find_next_break(remainder) {
            let before = &remainder[..start];
            let after = &remainder[end..];

            if token.should_break_aspect(aspect_ratio) {
                out.push_str(before.trim_end());
                out.push('\n');
                remainder = after.trim_start();
            } else {
                let trimmed_before = before.trim_end();
                let trimmed_after = after.trim_start();
                out.push_str(trimmed_before);
                if !out.is_empty()
                    && !out.ends_with('\n')
                    && !out.ends_with(' ')
                    && !trimmed_after.is_empty()
                {
                    out.push(' ');
                }
                remainder = trimmed_after;
            }
        }
        out.push_str(remainder);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soft_break_pipe() {
        let input = "Porque Dele | por Ele são todas as coisas";
        let widescreen = resolve_conditional_breaks(input, 16.0 / 9.0);
        let traditional = resolve_conditional_breaks(input, 4.0 / 3.0);

        assert_eq!(widescreen, "Porque Dele por Ele são todas as coisas");
        assert_eq!(traditional, "Porque Dele\npor Ele são todas as coisas");
    }

    #[test]
    fn test_ratio_4_3_break() {
        let input = "Cristo vive [4:3] em nossos corações";
        let widescreen = resolve_conditional_breaks(input, 16.0 / 9.0);
        let traditional = resolve_conditional_breaks(input, 4.0 / 3.0);

        assert_eq!(widescreen, "Cristo vive em nossos corações");
        assert_eq!(traditional, "Cristo vive\nem nossos corações");
    }

    #[test]
    fn test_clean_pipe_syntax() {
        let input = "Jesus é o caminho |4:3| a verdade e a vida";
        let widescreen = resolve_conditional_breaks(input, 16.0 / 9.0);
        let traditional = resolve_conditional_breaks(input, 4.0 / 3.0);

        assert_eq!(widescreen, "Jesus é o caminho a verdade e a vida");
        assert_eq!(traditional, "Jesus é o caminho\na verdade e a vida");
    }

    #[test]
    fn test_ratio_9_16_vertical_break() {
        let input = "Transmissão ao vivo |9:16| Vertical no Mobile";
        let widescreen = resolve_conditional_breaks(input, 16.0 / 9.0);
        let mobile = resolve_conditional_breaks(input, 9.0 / 16.0);

        assert_eq!(widescreen, "Transmissão ao vivo Vertical no Mobile");
        assert_eq!(mobile, "Transmissão ao vivo\nVertical no Mobile");
    }

    #[test]
    fn test_ratio_16_9_break() {
        let input = "Glória ao Pai [16:9] Glória ao Filho";
        let widescreen = resolve_conditional_breaks(input, 16.0 / 9.0);
        let traditional = resolve_conditional_breaks(input, 4.0 / 3.0);

        assert_eq!(widescreen, "Glória ao Pai\nGlória ao Filho");
        assert_eq!(traditional, "Glória ao Pai Glória ao Filho");
    }

    #[test]
    fn test_extended_named_break() {
        let input = "Santo [quebra: 4:3] Poderoso";
        assert_eq!(resolve_conditional_breaks(input, 4.0 / 3.0), "Santo\nPoderoso");
        assert_eq!(resolve_conditional_breaks(input, 16.0 / 9.0), "Santo Poderoso");
    }
}
