use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BibleSplitAction {
    #[default]
    MultipleSlides,
    BreakLines,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BibleSplitPunctuation {
    #[default]
    NaturalPunctuation,
    WordBoundary,
}

pub fn default_max_chars() -> usize {
    120
}

pub fn default_true() -> bool {
    true
}

/// Splits a long verse into balanced, readable chunks based on character limit and natural punctuation.
pub fn split_verse_text(
    text: &str,
    max_chars: usize,
    use_punctuation: bool,
    add_ellipsis: bool,
) -> Vec<String> {
    let text = text.trim();
    if text.chars().count() <= max_chars || max_chars < 30 {
        return vec![text.to_string()];
    }

    let mut result = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        let char_count = remaining.chars().count();
        if char_count <= max_chars {
            let chunk = if add_ellipsis && !result.is_empty() {
                format!("... {remaining}")
            } else {
                remaining.to_string()
            };
            result.push(chunk);
            break;
        }

        let mut split_byte_idx = None;

        if use_punctuation {
            let min_chars = max_chars * 40 / 100;
            let mut best_score = 0;
            let mut best_idx = None;

            for (char_idx, (byte_idx, ch)) in remaining.char_indices().enumerate() {
                if char_idx > max_chars {
                    break;
                }
                if char_idx >= min_chars {
                    let score = match ch {
                        '.' | '!' | '?' => 4,
                        ';' | ':' | '—' => 3,
                        ',' => 2,
                        _ => 0,
                    };
                    if score >= best_score && score > 0 {
                        best_score = score;
                        let next_byte = byte_idx + ch.len_utf8();
                        best_idx = Some(next_byte);
                    }
                }
            }

            if let Some(idx) = best_idx {
                split_byte_idx = Some(idx);
            }
        }

        // Fallback to word boundary if no punctuation match
        if split_byte_idx.is_none() {
            let mut last_space_idx = None;
            for (char_idx, (byte_idx, ch)) in remaining.char_indices().enumerate() {
                if char_idx > max_chars {
                    break;
                }
                if ch.is_whitespace() {
                    last_space_idx = Some(byte_idx);
                }
            }
            split_byte_idx = last_space_idx;
        }

        // Hard boundary fallback if a single word exceeds limit
        let split_at = split_byte_idx.unwrap_or_else(|| {
            remaining
                .char_indices()
                .nth(max_chars)
                .map(|(i, _)| i)
                .unwrap_or(remaining.len())
        });

        let (chunk, rest) = remaining.split_at(split_at);
        let chunk = chunk.trim();
        let rest = rest.trim_start();

        if chunk.is_empty() {
            if !rest.is_empty() {
                result.push(rest.to_string());
            }
            break;
        }

        let is_last = rest.is_empty();
        let formatted_chunk = if add_ellipsis {
            if !result.is_empty() && !is_last {
                format!("... {chunk} ...")
            } else if !result.is_empty() {
                format!("... {chunk}")
            } else if !is_last {
                format!("{chunk} ...")
            } else {
                chunk.to_string()
            }
        } else {
            chunk.to_string()
        };

        result.push(formatted_chunk);
        remaining = rest;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_verse_not_split() {
        let text = "No princípio criou Deus os céus e a terra.";
        let chunks = split_verse_text(text, 120, true, false);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_long_verse_splits_at_punctuation() {
        let text = "Porque Deus amou o mundo de tal maneira que deu o seu Filho unigênito, para que todo aquele que nele crê não pereça, mas tenha a vida eterna.";
        let chunks = split_verse_text(text, 80, true, false);
        assert!(chunks.len() >= 2);
        assert!(chunks[0].contains("unigênito,") || chunks[0].contains("maneira"));
    }

    #[test]
    fn test_split_with_ellipsis() {
        let text = "Primeira parte do texto muito longo aqui nesta linha, segunda parte do texto continua aqui.";
        let chunks = split_verse_text(text, 55, true, true);
        assert!(chunks.len() >= 2);
        assert!(chunks[0].ends_with("..."));
        assert!(chunks[1].starts_with("..."));
    }
}

