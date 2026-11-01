//! Text manipulation and selection actions for the song lyric editor.

use crate::core::state::ControlPanelState;

/// Safely maps a unicode char index (0-based count of chars) to a byte offset.
pub fn char_to_byte_idx(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(text.len())
}

/// Safely maps a byte offset to a unicode char index.
#[allow(dead_code)]
pub fn byte_to_char_idx(text: &str, byte_idx: usize) -> usize {
    let clamped = byte_idx.min(text.len());
    text[..clamped].chars().count()
}

/// Helper to wrap currently selected text in prefix and suffix (e.g. `**` for bold).
pub fn wrap_selection(state: &mut ControlPanelState, prefix: &str, suffix: &str) -> bool {
    let text = &state.song_editor.text;
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );

    if start_char == end_char {
        return false;
    }

    let start_byte = char_to_byte_idx(text, start_char);
    let end_byte = char_to_byte_idx(text, end_char);

    if start_byte >= end_byte || end_byte > text.len() {
        return false;
    }

    let selected = &text[start_byte..end_byte];

    // Toggle off if already wrapped
    if selected.starts_with(prefix)
        && selected.ends_with(suffix)
        && selected.len() >= prefix.len() + suffix.len()
    {
        let unwrapped = &selected[prefix.len()..selected.len() - suffix.len()];
        let new_text = format!("{}{}{}", &text[..start_byte], unwrapped, &text[end_byte..]);
        let unwrap_char_len = unwrapped.chars().count();
        state.song_editor.text = new_text;
        state.song_editor.selection = (start_char, start_char + unwrap_char_len);
        return true;
    }

    let wrapped = format!("{prefix}{selected}{suffix}");
    let wrapped_char_len = wrapped.chars().count();
    let new_text = format!("{}{}{}", &text[..start_byte], wrapped, &text[end_byte..]);
    state.song_editor.text = new_text;
    state.song_editor.selection = (start_char, start_char + wrapped_char_len);
    true
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CaseMode {
    Upper,
    Lower,
    Title,
}

/// Transforms case of the currently selected text.
pub fn transform_selection_case(state: &mut ControlPanelState, mode: CaseMode) -> bool {
    let text = &state.song_editor.text;
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );

    if start_char == end_char {
        return false;
    }

    let start_byte = char_to_byte_idx(text, start_char);
    let end_byte = char_to_byte_idx(text, end_char);

    if start_byte >= end_byte || end_byte > text.len() {
        return false;
    }

    let selected = &text[start_byte..end_byte];
    let transformed = match mode {
        CaseMode::Upper => selected.to_uppercase(),
        CaseMode::Lower => selected.to_lowercase(),
        CaseMode::Title => {
            let mut result = String::with_capacity(selected.len());
            let mut capitalize_next = true;
            for c in selected.chars() {
                if c.is_alphanumeric() {
                    if capitalize_next {
                        result.extend(c.to_uppercase());
                        capitalize_next = false;
                    } else {
                        result.extend(c.to_lowercase());
                    }
                } else {
                    result.push(c);
                    capitalize_next = true;
                }
            }
            result
        }
    };

    let trans_char_len = transformed.chars().count();
    let new_text = format!("{}{}{}", &text[..start_byte], transformed, &text[end_byte..]);
    state.song_editor.text = new_text;
    state.song_editor.selection = (start_char, start_char + trans_char_len);
    true
}

/// Prefixes lines within the current selection.
pub fn prefix_selected_lines(state: &mut ControlPanelState, prefix: &str) -> bool {
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );
    let text = &state.song_editor.text;
    let start_byte = char_to_byte_idx(text, start_char);
    let end_byte = char_to_byte_idx(text, end_char);

    let mut new_text = String::with_capacity(text.len() + 32);
    let mut current_byte = 0;
    for line in text.split('\n') {
        let line_len = line.len();
        let line_end_byte = current_byte + line_len;
        let intersects = if start_byte == end_byte {
            true
        } else {
            line_end_byte >= start_byte && current_byte <= end_byte
        };

        if intersects && !line.trim().is_empty() {
            new_text.push_str(prefix);
            new_text.push_str(line);
        } else {
            new_text.push_str(line);
        }
        new_text.push('\n');
        current_byte = line_end_byte + 1;
    }
    if !text.ends_with('\n') && new_text.ends_with('\n') {
        new_text.pop();
    }
    state.song_editor.text = new_text;
    true
}

/// Numbers lines within the current selection.
pub fn number_selected_lines(state: &mut ControlPanelState) -> bool {
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );
    let text = &state.song_editor.text;
    let start_byte = char_to_byte_idx(text, start_char);
    let end_byte = char_to_byte_idx(text, end_char);

    let mut counter = 0usize;
    let mut new_text = String::with_capacity(text.len() + 32);
    let mut current_byte = 0;
    for line in text.split('\n') {
        let line_len = line.len();
        let line_end_byte = current_byte + line_len;
        let intersects = if start_byte == end_byte {
            true
        } else {
            line_end_byte >= start_byte && current_byte <= end_byte
        };

        if intersects && !line.trim().is_empty() {
            counter += 1;
            new_text.push_str(&format!("{counter}. {line}"));
        } else {
            new_text.push_str(line);
        }
        new_text.push('\n');
        current_byte = line_end_byte + 1;
    }
    if !text.ends_with('\n') && new_text.ends_with('\n') {
        new_text.pop();
    }
    state.song_editor.text = new_text;
    true
}

/// Indents or decreases indent of lines within the current selection.
pub fn indent_selected_lines(state: &mut ControlPanelState, indent: bool) -> bool {
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );
    let text = &state.song_editor.text;
    let start_byte = char_to_byte_idx(text, start_char);
    let end_byte = char_to_byte_idx(text, end_char);

    let mut new_text = String::with_capacity(text.len() + 32);
    let mut current_byte = 0;
    for line in text.split('\n') {
        let line_len = line.len();
        let line_end_byte = current_byte + line_len;
        let intersects = if start_byte == end_byte {
            true
        } else {
            line_end_byte >= start_byte && current_byte <= end_byte
        };

        if intersects && !line.trim().is_empty() {
            if indent {
                new_text.push_str("  ");
                new_text.push_str(line);
            } else {
                let trimmed = line
                    .strip_prefix("  ")
                    .or_else(|| line.strip_prefix(' '))
                    .unwrap_or(line);
                new_text.push_str(trimmed);
            }
        } else {
            new_text.push_str(line);
        }
        new_text.push('\n');
        current_byte = line_end_byte + 1;
    }
    if !text.ends_with('\n') && new_text.ends_with('\n') {
        new_text.pop();
    }
    state.song_editor.text = new_text;
    true
}

/// Toggles comments (`//`) on lines within the current selection.
pub fn toggle_comment_selection(state: &mut ControlPanelState) -> bool {
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );
    let text = &state.song_editor.text;
    let start_byte = char_to_byte_idx(text, start_char);
    let end_byte = char_to_byte_idx(text, end_char);

    let mut new_text = String::with_capacity(text.len() + 16);
    let mut current_byte = 0;
    for line in text.split('\n') {
        let line_len = line.len();
        let line_end_byte = current_byte + line_len;
        let intersects = line_end_byte >= start_byte && current_byte <= end_byte;

        if intersects && !line.trim().is_empty() {
            if let Some(stripped) = line.strip_prefix("// ") {
                new_text.push_str(stripped);
            } else if let Some(stripped) = line.strip_prefix("//") {
                new_text.push_str(stripped);
            } else {
                new_text.push_str("// ");
                new_text.push_str(line);
            }
        } else {
            new_text.push_str(line);
        }
        new_text.push('\n');
        current_byte = line_end_byte + 1;
    }
    if !text.ends_with('\n') && new_text.ends_with('\n') {
        new_text.pop();
    }
    state.song_editor.text = new_text;
    true
}

/// Applies custom font tag `[fonte: font_name]` to current selection.
pub fn apply_font_selection(state: &mut ControlPanelState, font_name: &str) -> bool {
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );
    if start_char == end_char {
        return false;
    }
    let (updated, new_start, new_end) = crate::core::markup::apply_font_to_selection(
        &state.song_editor.text,
        start_char,
        end_char,
        font_name,
    );
    state.song_editor.text = updated;
    state.song_editor.selection = (new_start, new_end);
    true
}

/// Applies custom color tag `[cor: hex]` to current selection.
pub fn apply_color_selection(state: &mut ControlPanelState, hex: &str) -> bool {
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );
    if start_char == end_char {
        return false;
    }
    let (updated, new_start, new_end) = crate::core::markup::apply_color_to_selection(
        &state.song_editor.text,
        start_char,
        end_char,
        hex,
    );
    state.song_editor.text = updated;
    state.song_editor.selection = (new_start, new_end);
    true
}

/// Applies custom size tag `[tamanho: size]` to current selection.
pub fn apply_size_selection(state: &mut ControlPanelState, size_str: &str) -> bool {
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );
    if start_char == end_char {
        return false;
    }
    let (updated, new_start, new_end) = crate::core::markup::apply_size_to_selection(
        &state.song_editor.text,
        start_char,
        end_char,
        size_str,
    );
    state.song_editor.text = updated;
    state.song_editor.selection = (new_start, new_end);
    true
}

/// Applies custom shadow tag `[sombra: hex]` to current selection.
pub fn apply_shadow_selection(state: &mut ControlPanelState, hex: &str) -> bool {
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );
    if start_char == end_char {
        return false;
    }
    let (updated, new_start, new_end) = crate::core::markup::apply_shadow_to_selection(
        &state.song_editor.text,
        start_char,
        end_char,
        hex,
    );
    state.song_editor.text = updated;
    state.song_editor.selection = (new_start, new_end);
    true
}

/// Applies line alignment directive to current selection / stanza.
pub fn apply_align_directive(state: &mut ControlPanelState, align: &str) -> bool {
    let snippet = format!("\n//alinhar: {align}\n");
    insert_text_at_selection(state, &snippet)
}

/// Clears markdown formatting (`**`, `*`, `_`, `~`, `==`, `[fonte: ...]`, `[cor: ...]`, `[tamanho: ...]`, `[sombra: ...]`).
pub fn strip_formatting_from_selection(state: &mut ControlPanelState) -> bool {
    let text = &state.song_editor.text;
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );

    if start_char == end_char {
        let mut out = String::new();
        let mut blank_run = 0;
        for line in state.song_editor.text.lines() {
            if line.trim().is_empty() {
                blank_run += 1;
                if blank_run > 1 {
                    continue;
                }
                out.push('\n');
            } else {
                blank_run = 0;
                out.push_str(line.trim_end());
                out.push('\n');
            }
        }
        state.song_editor.text = out.trim_end().to_owned();
        return true;
    }

    let (updated, new_start, new_end) = crate::core::markup::clear_selection_formatting(
        text,
        start_char,
        end_char,
    );
    state.song_editor.text = updated;
    state.song_editor.selection = (new_start, new_end);
    true
}

/// Inserts text at current selection / cursor position.
pub fn insert_text_at_selection(state: &mut ControlPanelState, snippet: &str) -> bool {
    let text = &state.song_editor.text;
    let (start_char, end_char) = (
        state.song_editor.selection.0.min(state.song_editor.selection.1),
        state.song_editor.selection.0.max(state.song_editor.selection.1),
    );

    let start_byte = char_to_byte_idx(text, start_char);
    let end_byte = char_to_byte_idx(text, end_char);

    let new_text = format!("{}{}{}", &text[..start_byte], snippet, &text[end_byte..]);
    let snippet_chars = snippet.chars().count();
    let new_pos = start_char + snippet_chars;
    state.song_editor.text = new_text;
    state.song_editor.selection = (new_pos, new_pos);
    true
}

/// Inserts or updates a slide directive line (e.g. `//fonte: Montserrat`) in the stanza where the cursor or selection is located.
pub fn apply_or_update_slide_directive(
    state: &mut ControlPanelState,
    directive: &str,
    value: &str,
) -> bool {
    let text = &state.song_editor.text;
    if text.is_empty() {
        state.song_editor.text = format!("//{directive}: {value}\n\n");
        state.song_editor.selection = (0, 0);
        return true;
    }

    let cursor_char = state.song_editor.selection.0.min(state.song_editor.selection.1);
    let cursor_byte = char_to_byte_idx(text, cursor_char);

    // Encontra os limites da estrofe atual (delimitada por linhas em branco \n\n ou início/fim do texto)
    let stanza_start_byte = text[..cursor_byte]
        .rfind("\n\n")
        .map(|idx| idx + 2)
        .unwrap_or(0);
    let stanza_end_byte = text[cursor_byte..]
        .find("\n\n")
        .map(|idx| cursor_byte + idx)
        .unwrap_or(text.len());

    let prefix_text = &text[..stanza_start_byte];
    let stanza_text = &text[stanza_start_byte..stanza_end_byte];
    let suffix_text = &text[stanza_end_byte..];

    let target_prefix = format!("//{directive}:");
    let target_prefix_space = format!("// {directive}:");
    let new_line = format!("//{directive}: {value}");

    let mut found = false;
    let mut updated_lines = Vec::new();
    for line in stanza_text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(&target_prefix) || trimmed.starts_with(&target_prefix_space) {
            updated_lines.push(new_line.clone());
            found = true;
        } else {
            updated_lines.push(line.to_string());
        }
    }

    let updated_stanza = if found {
        updated_lines.join("\n")
    } else {
        format!("{new_line}\n{stanza_text}")
    };

    let new_text = format!("{prefix_text}{updated_stanza}{suffix_text}");
    state.song_editor.text = new_text;
    true
}

/// Normalizes extra blank lines in the lyrics.
pub fn normalize_whitespace(state: &mut ControlPanelState) -> bool {
    let mut out = String::new();
    let mut blank_count = 0;
    for line in state.song_editor.text.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                out.push('\n');
            }
        } else {
            blank_count = 0;
            out.push_str(line.trim_end());
            out.push('\n');
        }
    }
    state.song_editor.text = out.trim_end().to_owned();
    true
}


