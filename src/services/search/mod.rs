use crate::core::{
    bible::{BibleLibrary, fold},
    commands::{Command, SETTINGS_PAGES},
    i18n::Language,
    slides::SlideDeck,
    state::{ControlPanelState, SettingsTab},
};
mod bible_index;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchTarget {
    PluginCommand { plugin: String, command: String },
    Command(Command),
    Settings(SettingsTab),
    Song(usize),
    Theme(String),
    Design(String),
    Media(usize),
    Text(usize),
    Verse(usize),
    Bible {
        version: String,
        book: usize,
        chapter: usize,
        verse: usize,
        end: usize,
    },
    Chat {
        channel: Option<String>,
        peer: Option<String>,
        query: String,
    },
}

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub title: String,
    pub detail: String,
    pub category: &'static str,
    pub target: SearchTarget,
    score: u32,
}

struct SearchDocument {
    result: SearchResult,
    body: String,
}

pub struct SearchSnapshot {
    documents: Vec<SearchDocument>,
    bibles: BibleLibrary,
    bible_enabled: bool,
    chat: Option<(crate::services::chat::ChatService, String, String)>,
}

impl SearchSnapshot {
    pub fn capture(state: &ControlPanelState, deck: &SlideDeck) -> Self {
        let mut documents = Vec::new();
        let language = state.settings.language;
        let mut add = |title: String, detail: String, body: String, category, target| {
            documents.push(SearchDocument {
                result: SearchResult {
                    title,
                    detail,
                    category,
                    target,
                    score: 0,
                },
                body,
            });
        };
        for command in Command::ALL {
            if ((command == Command::Bible || command == Command::BibleSearch)
                && !state.settings.bible_enabled)
                || (command == Command::Chat && !state.settings.chat_enabled)
            {
                continue;
            }
            let keywords = Language::ALL
                .into_iter()
                .map(|language| language.text(command.label_key()).to_owned())
                .collect::<Vec<_>>()
                .join(" ");
            add(
                language.text(command.label_key()).into(),
                state
                    .settings
                    .shortcuts
                    .bindings(command)
                    .iter()
                    .map(|key| key.display())
                    .collect::<Vec<_>>()
                    .join("  /  "),
                keywords,
                "search-actions",
                SearchTarget::Command(command),
            );
        }
        for (tab, key) in SETTINGS_PAGES {
            let mut keywords = Language::ALL
                .into_iter()
                .map(|language| language.text(key).to_owned())
                .collect::<Vec<_>>();
            if tab == SettingsTab::Interface {
                for key in ["settings-language", "settings-scale", "settings-appearance"] {
                    keywords.extend(
                        Language::ALL
                            .into_iter()
                            .map(|language| language.text(key).to_owned()),
                    );
                }
            }
            add(
                language.text(key).into(),
                String::new(),
                keywords.join(" "),
                "search-settings",
                SearchTarget::Settings(tab),
            );
        }
        for plugin in state.settings.plugins.iter().filter(|plugin| plugin.enabled) {
            for command in &plugin.package.commands {
                add(command.localized_name(language).into(), plugin.package.localized_name(language).into(), command.localized_description(language).into(), "plugins-title",
                    SearchTarget::PluginCommand { plugin: plugin.package.id.clone(), command: command.id.clone() });
            }
        }
        for (index, song) in deck.library().iter().enumerate() {
            add(
                song.title.clone(),
                song.artist.clone(),
                format!("{} {} {}", song.text, song.tags, song.folder),
                "library-songs",
                SearchTarget::Song(index),
            );
        }
        for theme in &state.themes {
            add(
                theme.name.clone(),
                String::new(),
                theme.bg_type.clone(),
                "search-themes",
                SearchTarget::Theme(theme.id.clone()),
            );
        }
        for design in &deck.designs {
            add(
                design.name.clone(),
                design.folder.clone(),
                serde_json::to_string(&design.layers).unwrap_or_default(),
                "library-designs",
                SearchTarget::Design(design.id.clone()),
            );
        }
        for (index, media) in state.media_items.iter().enumerate() {
            add(
                media.title.clone(),
                media.file_type.clone(),
                format!("{} {}", media.file_path, media.folder),
                "library-media",
                SearchTarget::Media(index),
            );
        }
        for (index, text) in state.custom_texts.iter().enumerate() {
            add(
                text.title.clone(),
                text.folder.clone(),
                text.content.clone(),
                "library-texts",
                SearchTarget::Text(index),
            );
        }
        for (index, verse) in state.saved_verses.iter().enumerate() {
            add(
                verse.reference.clone(),
                verse.version.clone(),
                verse.text.clone(),
                "library-verses",
                SearchTarget::Verse(index),
            );
        }
        Self {
            documents,
            bibles: state.bible.library.clone(),
            bible_enabled: state.settings.bible_enabled,
            chat: if state.settings.chat_enabled && !state.chat.token.is_empty() {
                state.chat.service.clone().map(|service| {
                    (
                        service,
                        state.chat.token.clone(),
                        state.chat.profile_id.clone(),
                    )
                })
            } else {
                None
            },
        }
    }

    pub fn search(self, query: &str) -> Vec<SearchResult> {
        self.search_with_cancellation(query, &AtomicBool::new(false))
    }

    pub fn local_results(&self, query: &str) -> Vec<SearchResult> {
        let query = fold(query.trim());
        if query.is_empty() {
            return Vec::new();
        }
        let mut results = Vec::new();
        for document in &self.documents {
            if let Some(score) = rank(
                &document.result.title,
                &document.result.detail,
                &document.body,
                &query,
            ) {
                let mut result = document.result.clone();
                result.score = score;
                if result.detail.is_empty() {
                    result.detail = excerpt(&document.body, &query);
                }
                results.push(result);
            }
        }
        results.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.title.cmp(&b.title)));
        results.truncate(100);
        results
    }

    pub fn search_with_cancellation(
        self,
        query: &str,
        cancelled: &AtomicBool,
    ) -> Vec<SearchResult> {
        let query = fold(query.trim());
        if query.is_empty() {
            return Vec::new();
        }
        let mut results = self.local_results(&query);
        if cancelled.load(Ordering::Relaxed) {
            return Vec::new();
        }
        // Full Bible scans and chat lookups run in the search worker, never in the UI frame.
        if query.chars().count() >= 2 && self.bible_enabled {
            for (version, _) in self.bibles.catalog() {
                if cancelled.load(Ordering::Relaxed) {
                    return Vec::new();
                }
                let Some(bible) = self.bibles.get(&version) else {
                    continue;
                };
                if let Some((book, chapter, verse, end)) = bible.reference(&query)
                    && let Some(passage) = bible.passage(book, chapter, verse, end) {
                        results.push(SearchResult {
                            title: format!("{} · {}", passage.reference, version),
                            detail: excerpt(&passage.content, &query),
                            category: "search-bible",
                            score: 950,
                            target: SearchTarget::Bible {
                                version: version.clone(),
                                book,
                                chapter,
                                verse,
                                end,
                            },
                        });
                    }
                let parsed = crate::core::search::SearchQuery::new(&query);
                for (book, chapter, verse) in bible_index::find(&bible, &parsed, cancelled, 10) {
                    let text = &bible.books[book].chapters[chapter - 1][verse - 1];
                    results.push(SearchResult {
                        title: format!("{} {chapter}:{verse} · {version}", bible.books[book].name),
                        detail: excerpt(text, &query),
                        category: "search-bible",
                        score: 100,
                        target: SearchTarget::Bible {
                            version: version.clone(),
                            book,
                            chapter,
                            verse,
                            end: verse,
                        },
                    });
                }
            }
        }
        if !query.is_empty() && !cancelled.load(Ordering::Relaxed)
            && let Some((service, token, profile)) = self.chat
                && let Ok(snapshot) = service.snapshot(&token, None, None, &query, None) {
                    for message in snapshot
                        .messages
                        .into_iter()
                        .filter(|message| !message.deleted)
                    {
                        let peer = if message.channel_id.is_none() {
                            if message.author_id == profile {
                                message.recipient_id.clone()
                            } else {
                                Some(message.author_id.clone())
                            }
                        } else {
                            None
                        };
                        results.push(SearchResult {
                            title: if message.title.is_empty() {
                                message.author
                            } else {
                                message.title
                            },
                            detail: excerpt(&message.text, &query),
                            category: "search-chat",
                            score: 100,
                            target: SearchTarget::Chat {
                                channel: message.channel_id,
                                peer,
                                query: query.clone(),
                            },
                        });
                    }
                }
        results.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.title.cmp(&b.title)));
        results.truncate(100);
        results
    }
}

fn rank(title: &str, detail: &str, body: &str, query: &str) -> Option<u32> {
    if query.is_empty() {
        return Some(0);
    }
    let title = fold(title);
    let detail = fold(detail);
    let body = fold(body);
    if !query
        .split_whitespace()
        .all(|token| title.contains(token) || detail.contains(token) || body.contains(token))
    {
        return None;
    }
    Some(if title == query {
        1000
    } else if title.starts_with(query) {
        800
    } else if title.contains(query) {
        600
    } else if detail.contains(query) {
        400
    } else {
        100
    })
}

fn excerpt(text: &str, query: &str) -> String {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let chars: Vec<_> = normalized.chars().collect();
    let folded = fold(&normalized);
    let start = query
        .split_whitespace()
        .next()
        .and_then(|token| folded.find(token))
        .map(|offset| folded[..offset].chars().count().saturating_sub(40))
        .unwrap_or(0)
        .min(chars.len());
    let end = (start + 180).min(chars.len());
    format!(
        "{}{}{}",
        if start > 0 { "…" } else { "" },
        chars[start..end].iter().collect::<String>(),
        if end < chars.len() { "…" } else { "" }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_accents_and_body_tokens_and_ranks_titles_first() {
        assert!(
            rank("Adoração", "", "", "adoracao").unwrap()
                > rank("Song", "", "adoração", "adoracao").unwrap()
        );
        assert!(rank("Grace", "Singer", "amazing love", "grace love").is_some());
        assert!(rank("Grace", "", "love", "grace missing").is_none());
        assert!(excerpt(&"á🙏 text ".repeat(80), "text").chars().count() < 190);
    }

    #[test]
    fn searches_content_and_translated_commands() {
        let mut state = ControlPanelState::default();
        state.settings.bible_enabled = false;
        state.settings.chat_enabled = false;
        let results = SearchSnapshot::capture(&state, &SlideDeck::default()).search("keyboard");
        assert!(
            results
                .iter()
                .any(|result| result.target == SearchTarget::Settings(SettingsTab::Shortcuts))
        );
        let results =
            SearchSnapshot::capture(&state, &SlideDeck::default()).search("configuracion");
        assert!(
            results
                .iter()
                .any(|result| result.target == SearchTarget::Command(Command::Settings))
        );
    }
}
