use super::bible::{
    BibleReferencePosition, BibleReferenceSize, BibleSettings, BibleSplitAction,
    BibleSplitPunctuation, BibleVerseNumberStyle, BibleVersionStyle,
};
use super::bible_split::split_verse_text;

pub fn to_superscript(n: usize) -> String {
    n.to_string()
        .chars()
        .map(|c| match c {
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            '9' => '⁹',
            _ => c,
        })
        .collect()
}

pub fn format_verse_number(number: usize, style: BibleVerseNumberStyle) -> String {
    match style {
        BibleVerseNumberStyle::Bracket => format!("[{number}]"),
        BibleVerseNumberStyle::Superscript => to_superscript(number),
        BibleVerseNumberStyle::Dot => format!("{number}."),
    }
}

pub fn clean_version_code(raw: &str) -> String {
    let mut code = raw
        .rsplit_once('(')
        .map(|(_, c)| c.trim_end_matches(')'))
        .unwrap_or(raw)
        .trim();
    if let Some((lang, rest)) = code.split_once('_').or_else(|| code.split_once('-')) {
        if (2..=4).contains(&lang.len())
            && lang.chars().all(|c| c.is_ascii_alphabetic())
            && !rest.is_empty()
        {
            code = rest.trim();
        }
    }
    if let Some((lang, rest)) = code.split_once(' ') {
        if (2..=4).contains(&lang.len())
            && lang.chars().all(|c| c.is_ascii_alphabetic())
            && !rest.is_empty()
        {
            code = rest.trim();
        }
    }
    code.to_uppercase()
}

pub fn format_bible_reference(
    settings: &BibleSettings,
    book_name: &str,
    book_abbrev: &str,
    version_code: &str,
    chapter: usize,
    start_verse: usize,
    end_verse: usize,
) -> String {
    let raw_book = if settings.use_abbreviation && !book_abbrev.trim().is_empty() {
        book_abbrev.trim()
    } else {
        book_name.trim()
    };
    let book = if settings.uppercase_reference {
        raw_book.to_uppercase()
    } else {
        raw_book.to_string()
    };
    let sep = settings.separator.as_str();
    let range = if start_verse == end_verse {
        format!("{chapter}{sep}{start_verse}")
    } else {
        format!("{chapter}{sep}{start_verse}–{end_verse}")
    };
    let ver = if settings.show_version && !version_code.trim().is_empty() {
        let code = clean_version_code(version_code);
        match settings.version_style {
            BibleVersionStyle::Parentheses => format!(" ({code})"),
            BibleVersionStyle::Hyphen => format!(" - {code}"),
        }
    } else {
        String::new()
    };
    let mut reference = format!("{book} {range}{ver}");
    if settings.uppercase_reference {
        reference = reference.to_uppercase();
    }
    reference
}

pub fn format_bible_slide_with_custom_ref(
    settings: &BibleSettings,
    reference: String,
    start_verse: usize,
    verses: &[String],
) -> (String, String) {
    let processed_verses: Vec<String> = if settings.auto_split_long_verses
        && settings.split_action == BibleSplitAction::BreakLines
    {
        verses
            .iter()
            .map(|v| {
                let chunks = split_verse_text(
                    v,
                    settings.max_chars_per_slide,
                    settings.split_punctuation == BibleSplitPunctuation::NaturalPunctuation,
                    false,
                );
                chunks.join("\n")
            })
            .collect()
    } else {
        verses.to_vec()
    };

    let mut body = if processed_verses.len() <= 1 || !settings.show_verse_numbers {
        processed_verses.join("\n")
    } else {
        processed_verses
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let num = start_verse + i;
                let num_str = format_verse_number(num, settings.verse_number_style);
                format!("{num_str} {v}")
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    if settings.quotes_around_verse && !body.trim().is_empty() {
        body = format!("“{body}”");
    }

    let styled_ref = if reference.trim().is_empty() {
        String::new()
    } else if settings.italic_reference {
        format!("*{reference}*")
    } else {
        reference.clone()
    };

    let content_core = if styled_ref.is_empty() {
        body
    } else {
        match settings.reference_position {
            BibleReferencePosition::Bottom | BibleReferencePosition::BelowVerse => {
                format!("{body}\n{styled_ref}")
            }
            BibleReferencePosition::Top => {
                format!("{styled_ref}\n{body}")
            }
            BibleReferencePosition::EndOfVerse => {
                format!("{body} ({styled_ref})")
            }
            BibleReferencePosition::Hidden => body,
        }
    };

    let mut content = String::new();
    if let Some(valign_dir) = settings.vertical_alignment.as_markup_directive() {
        content.push_str(valign_dir);
    }
    if let Some(dir) = settings.alignment.as_markup_directive() {
        content.push_str(dir);
    }
    if let Some(line_dir) = settings.line_spacing.as_markup_directive() {
        content.push_str(line_dir);
    }
    if !styled_ref.is_empty() && settings.reference_position != BibleReferencePosition::Hidden && settings.reference_position != BibleReferencePosition::EndOfVerse {
        content.push_str(&format!("//ref-pos: {}\n", settings.reference_position.as_str()));
        content.push_str(&format!("//ref-align: {}\n", settings.reference_alignment.as_str()));
        if settings.reference_size != BibleReferenceSize::Normal {
            content.push_str(&format!("//ref-size: {}\n", settings.reference_size.as_str()));
        }
    }
    content.push_str(&content_core);

    (reference, content)
}

pub fn format_bible_slide(
    settings: &BibleSettings,
    book_name: &str,
    book_abbrev: &str,
    version_code: &str,
    chapter: usize,
    start_verse: usize,
    end_verse: usize,
    verses: &[String],
) -> (String, String) {
    let reference = format_bible_reference(
        settings,
        book_name,
        book_abbrev,
        version_code,
        chapter,
        start_verse,
        end_verse,
    );
    format_bible_slide_with_custom_ref(settings, reference, start_verse, verses)
}

