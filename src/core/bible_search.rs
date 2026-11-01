use super::bible::{BibleTranslation, fold};

/// Keep leading book numbers (`1 Samuel`) separate from chapter/verse numbers.
pub fn split_reference(query: &str) -> (String, String) {
    let folded = fold(query).replace(' ', "");
    let mut seen_letter = false;
    for (index, c) in folded.char_indices() {
        if c.is_alphabetic() {
            seen_letter = true;
        } else if c == ':' || c == '.' || (c.is_ascii_digit() && seen_letter) {
            return (folded[..index].to_owned(), folded[index..].to_owned());
        }
    }
    (folded, String::new())
}

/// Complete numbers once another digit would exceed the chapter/verse count.
pub fn number_is_complete(text: &str, max: usize) -> bool {
    let Ok(value) = text.parse::<usize>() else {
        return false;
    };
    let max = max.max(1);
    let digits = max.ilog10() as usize + 1;
    value >= 1 && (text.len() >= digits || value > max / 10)
}

pub fn matching_books(bible: &BibleTranslation, query: &str) -> Vec<usize> {
    let query = fold(query).replace(' ', "");
    if query.is_empty() {
        return Vec::new();
    }
    let mut matches: Vec<usize> = bible
        .books
        .iter()
        .enumerate()
        .filter(|(_, book)| {
            let name = fold(&book.name).replace(' ', "");
            let abbrev = fold(&book.abbrev).replace(' ', "");
            let reference_prefix = |key: &str| {
                !key.is_empty()
                    && query.strip_prefix(key).is_some_and(|rest| {
                        rest.starts_with(|c: char| c.is_ascii_digit() || c == ':' || c == '.')
                    })
            };
            name.starts_with(&query)
                || abbrev.starts_with(&query)
                || reference_prefix(&name)
                || reference_prefix(&abbrev)
        })
        .map(|(index, _)| index)
        .collect();
    matches.sort_by_key(|&index| {
        let abbrev = fold(&bible.books[index].abbrev).replace(' ', "");
        (usize::from(!abbrev.starts_with(&query)), index)
    });
    matches
}

pub struct QueryParse {
    pub book: Option<usize>,
    pub matches: Vec<usize>,
    pub chapter: Option<usize>,
    pub verse: Option<usize>,
    pub has_separator: bool,
    pub book_complete: bool,
}

pub fn parse_query(bible: &BibleTranslation, query: &str) -> QueryParse {
    let (book_part, numbers) = split_reference(query);
    let matches = matching_books(bible, &book_part);
    let mut exact = None;
    for &index in &matches {
        for key in [
            fold(&bible.books[index].name).replace(' ', ""),
            fold(&bible.books[index].abbrev).replace(' ', ""),
        ] {
            if !key.is_empty() && key == book_part {
                exact = Some(index);
            }
        }
    }
    let book_complete = exact.is_some() && matches.len() == 1;
    let book = exact.or_else(|| matches.first().copied());
    let has_separator = numbers.contains([':', '.']);
    let (chapter, verse) = if numbers.is_empty() {
        (None, None)
    } else {
        match numbers.split_once([':', '.']) {
            Some((chapter, verse)) => (chapter.parse().ok(), verse.parse().ok()),
            None => (numbers.parse().ok(), None),
        }
    };
    QueryParse {
        book,
        matches,
        chapter,
        verse,
        has_separator,
        book_complete,
    }
}
