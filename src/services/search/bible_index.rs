use crate::core::{bible::BibleTranslation, search::SearchQuery};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

struct Verse {
    book: usize,
    chapter: usize,
    number: usize,
    normalized: String,
}
struct CachedIndex {
    translation: Arc<BibleTranslation>,
    verses: Arc<OnceLock<Vec<Verse>>>,
}

pub(super) fn find(
    translation: &Arc<BibleTranslation>,
    query: &SearchQuery,
    cancelled: &AtomicBool,
    limit: usize,
) -> Vec<(usize, usize, usize)> {
    static INDEXES: OnceLock<Mutex<HashMap<String, CachedIndex>>> = OnceLock::new();
    let verses = {
        let mut indexes = INDEXES.get_or_init(Mutex::default).lock().unwrap();
        let entry = indexes
            .entry(translation.id.clone())
            .or_insert_with(|| CachedIndex {
                translation: translation.clone(),
                verses: Arc::default(),
            });
        // An import can replace an existing code while the app is running.
        if !Arc::ptr_eq(&entry.translation, translation) {
            *entry = CachedIndex {
                translation: translation.clone(),
                verses: Arc::default(),
            };
        }
        entry.verses.clone()
    };
    let verses = verses.get_or_init(|| {
        translation
            .books
            .iter()
            .enumerate()
            .flat_map(|(book, value)| {
                value
                    .chapters
                    .iter()
                    .enumerate()
                    .flat_map(move |(chapter, verses)| {
                        verses.iter().enumerate().map(move |(verse, text)| Verse {
                            book,
                            chapter: chapter + 1,
                            number: verse + 1,
                            normalized: crate::core::bible::fold(text),
                        })
                    })
            })
            .collect()
    });
    let mut found = Vec::new();
    for verse in verses {
        if cancelled.load(Ordering::Relaxed) {
            break;
        }
        if query.matches(&verse.normalized) {
            found.push((verse.book, verse.chapter, verse.number));
            if found.len() >= limit {
                break;
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::bible::BibleBook;

    #[test]
    fn replacing_a_translation_rebuilds_its_index_and_cancellation_stops_work() {
        let version = |text: &str| {
            Arc::new(BibleTranslation {
                id: "INDEX-TEST".into(),
                name: "Test".into(),
                books: vec![BibleBook {
                    name: "Genesis".into(),
                    abbrev: "gn".into(),
                    chapters: vec![vec![text.into()]],
                }],
            })
        };
        let cancelled = AtomicBool::new(false);
        assert_eq!(
            find(
                &version("Grace and love"),
                &SearchQuery::new("grace"),
                &cancelled,
                10
            ),
            [(0, 1, 1)]
        );
        assert!(
            find(
                &version("Peace and hope"),
                &SearchQuery::new("grace"),
                &cancelled,
                10
            )
            .is_empty()
        );
        cancelled.store(true, Ordering::Relaxed);
        assert!(
            find(
                &version("Grace"),
                &SearchQuery::new("grace"),
                &cancelled,
                10
            )
            .is_empty()
        );
    }
}
