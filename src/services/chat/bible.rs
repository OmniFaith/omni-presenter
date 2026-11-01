use super::ChatService;
use crate::core::{
    bible::{BibleLibrary, DEFAULT_BIBLE, fold},
    bible_search::{number_is_complete, parse_query, split_reference},
};
use serde_json::{Value, json};

impl ChatService {
    pub(super) fn bible_library(&self) -> Result<BibleLibrary, String> {
        let mut library = BibleLibrary::default();
        for translation in self
            .db
            .get_bible_translations()
            .map_err(|error| error.to_string())?
        {
            library.insert(translation);
        }
        Ok(library)
    }

    pub fn bible_search(&self, token: &str, version: &str, query: &str) -> Result<Value, String> {
        self.heartbeat(token)?;
        let library = self.bible_library()?;
        let bible = library
            .get(if version.is_empty() {
                DEFAULT_BIBLE
            } else {
                version
            })
            .ok_or("services-bible-translation-not-found")?;
        Ok(suggestions(&bible, query))
    }
}

pub(crate) fn suggestions(bible: &crate::core::bible::BibleTranslation, query: &str) -> Value {
    let parsed = parse_query(bible, query);
    let (book_part, numbers) = split_reference(query);
    let full_reference = bible.reference(query).or_else(|| {
        if parsed.matches.len() != 1 {
            return None;
        }
        bible.reference(&format!("{} {numbers}", bible.books[parsed.book?].name))
    });
    let selection = full_reference.filter(|_| numbers.split_once([':', '.']).is_some_and(|(_, verse)| !verse.is_empty()))
            .and_then(|(book, chapter, start, end)| bible.passage(book, chapter, start, end))
            .map(|passage| json!({"reference":passage.reference,"version":passage.version,"text":passage.content}));
    let book = full_reference.map(|reference| reference.0).or(parsed.book);
    let mut choices = Vec::new();
    let mut completion = None;
    let stage;
    if query.trim().is_empty()
        || (!parsed.book_complete && selection.is_none() && !parsed.matches.is_empty())
    {
        stage = "book";
        let matches: Vec<_> = if query.trim().is_empty() {
            (0..bible.books.len()).collect()
        } else {
            parsed.matches.clone()
        };
        for index in matches {
            let book = &bible.books[index];
            let next = if book.chapters.len() == 1 {
                format!("{} 1:", book.name)
            } else {
                format!("{} ", book.name)
            };
            choices.push(json!({"label":book.name,"query":next}));
        }
    } else if let Some(index) =
        book.filter(|_| !parsed.matches.is_empty() || full_reference.is_some())
    {
        let book = &bible.books[index];
        let chapter = parsed.chapter.unwrap_or(1).clamp(1, book.chapters.len());
        if !parsed.has_separator {
            stage = "chapter";
            for chapter in 1..=book.chapters.len() {
                choices.push(
                    json!({"label":chapter.to_string(),"query":format!("{} {chapter}:",book.name)}),
                );
            }
            if number_is_complete(&numbers, book.chapters.len()) {
                completion = Some(format!("{} {chapter}:", book.name));
            }
        } else {
            stage = "verse";
            for verse in 1..=book.chapters[chapter - 1].len() {
                choices.push(json!({"label":verse.to_string(),"query":format!("{} {chapter}:{verse}",book.name)}));
            }
        }
    } else {
        stage = "text";
        let needle = fold(query.trim());
        if needle.chars().count() >= 2 {
            'books: for (book, data) in bible.books.iter().enumerate() {
                for (chapter, verses) in data.chapters.iter().enumerate() {
                    for (verse, text) in verses.iter().enumerate() {
                        if !text.is_empty() && fold(text).contains(&needle) {
                            let passage = bible
                                .passage(book, chapter + 1, verse + 1, verse + 1)
                                .expect("verse exists");
                            choices.push(json!({"label":passage.reference,"query":passage.reference,"text":text}));
                            if choices.len() >= 60 {
                                break 'books;
                            }
                        }
                    }
                }
            }
        }
    }
    if let [index] = parsed.matches[..] {
        let book = &bible.books[index];
        if fold(&book.name).replace(' ', "") != book_part {
            completion = Some(if !numbers.is_empty() {
                format!("{} {numbers}", book.name)
            } else if book.chapters.len() == 1 {
                format!("{} 1:", book.name)
            } else {
                format!("{} ", book.name)
            });
        }
    }
    let chapter = book.and_then(|index| {
        let number = parsed.chapter?;
        let verses = bible.chapter(index, number)?;
        Some(json!({"book":bible.books[index].name,"number":number,"total":bible.books[index].chapters.len(),"verses":verses}))
    });
    let auto_select = selection.is_some() && numbers.split_once([':', '.']).is_some_and(|(_, verse)| {
        !verse.contains(['-', '–']) && chapter.as_ref()
            .and_then(|value| value["verses"].as_array())
            .is_some_and(|verses| number_is_complete(verse, verses.len()))
    });
    json!({"stage":stage,"choices":choices,"selection":selection,"completion":completion,"chapter":chapter,"auto_select":auto_select})
}

#[cfg(test)]
mod tests {
    use crate::services::chat::tests::fixture;

    #[test]
    fn bible_search_uses_app_stages_abbreviations_zeroes_ranges_and_text() {
        let (_, chat, tokens) = fixture();
        assert!(chat.bible_search("invalid", "ARA", "João 3:16").is_err());
        let search = |query| chat.bible_search(&tokens[0], "ARA", query).unwrap();
        assert_eq!(search("")["choices"].as_array().unwrap().len(), 66);
        assert_eq!(search("1 sa")["completion"], "1 Samuel ");
        assert_eq!(search("Atos 02")["completion"], "Atos 2:");
        assert_eq!(search("Salmos 119:")["stage"], "verse");
        assert_eq!(
            search("Salmos 119:")["choices"].as_array().unwrap().len(),
            176
        );
        assert_eq!(
            search("jo3.16-18")["selection"]["reference"],
            "João 3:16–18"
        );
        assert_eq!(
            search("Sl119:01-07")["selection"]["reference"],
            "Salmos 119:1–7"
        );
        assert!(search("João 999:999")["selection"].is_null());
        let result = search("o senhor e meu pastor");
        assert_eq!(result["stage"], "text");
        let result = search("meu pastor");
        assert!(
            result["choices"]
                .as_array()
                .unwrap()
                .iter()
                .any(|hit| hit["query"] == "Salmos 23:1")
        );
    }

    #[test]
    fn song_search_accepts_accents_and_lyric_tokens_before_sharing() {
        let (db, chat, tokens) = fixture();
        let id = db
            .add_song(
                "Canção de Esperança",
                "Equipe",
                "**Graça** e paz para todos",
            )
            .unwrap();
        let songs = chat.songs(&tokens[0], "cancao esperanca").unwrap();
        assert_eq!(songs[0]["id"], id);
        assert_eq!(chat.songs(&tokens[0], "graca paz").unwrap()[0]["id"], id);
        let message = chat
            .send(
                &tokens[0],
                super::super::SendMessage {
                    channel_id: Some("general".into()),
                    kind: "song".into(),
                    song_id: Some(id),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(message.text, "Graça e paz para todos");
    }
}
