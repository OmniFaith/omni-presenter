use crate::core::bible::fold;

#[derive(Clone, Debug)]
pub struct SearchQuery {
    normalized: String,
    tokens: Vec<String>,
}

impl SearchQuery {
    pub fn new(query: &str) -> Self {
        let normalized = fold(query.trim());
        let tokens = normalized.split_whitespace().map(str::to_owned).collect();
        Self { normalized, tokens }
    }
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
    pub fn text(&self) -> &str {
        &self.normalized
    }
    pub fn matches(&self, normalized: &str) -> bool {
        self.tokens.iter().all(|token| normalized.contains(token))
    }
}

#[derive(Clone, Default)]
pub struct SearchCache {
    query: String,
    revision: u64,
    matches: Vec<usize>,
    initialized: bool,
}

impl SearchCache {
    pub fn filter<'a>(
        &mut self,
        query: &str,
        revision: u64,
        fields: impl IntoIterator<Item = &'a str>,
    ) -> &[usize] {
        if !self.initialized || self.query != query || self.revision != revision {
            let parsed = SearchQuery::new(query);
            self.matches = fields
                .into_iter()
                .enumerate()
                .filter_map(|(index, text)| parsed.matches(text).then_some(index))
                .collect();
            self.query = query.into();
            self.revision = revision;
            self.initialized = true;
        }
        &self.matches
    }
}
