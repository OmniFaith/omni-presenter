//! English-source, resource-driven application localization.
#[path = "i18n/catalog.rs"]
pub mod catalog;
#[path = "i18n/arguments.rs"]
mod arguments;
#[path = "i18n/notice.rs"]
mod notice;
pub use arguments::{Argument, Arguments};
pub use notice::Message;
pub use catalog::{Direction, LocaleMetadata};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    #[default]
    #[serde(rename = "en")]
    English,
    #[serde(rename = "pt-BR")]
    Portuguese,
    #[serde(rename = "es")]
    Spanish,
}

impl Language {
    pub const ALL: [Self; 3] = [Self::English, Self::Portuguese, Self::Spanish];

    pub fn from_code(code: &str) -> Self {
        let negotiated = catalog::registry().negotiate(code).metadata.id.as_str();
        match negotiated {
            "pt-BR" => Self::Portuguese,
            "es" => Self::Spanish,
            _ => Self::English,
        }
    }
    pub fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Portuguese => "pt-BR",
            Self::Spanish => "es",
        }
    }
    pub fn name(self) -> &'static str { &catalog::registry().negotiate(self.code()).metadata.native_name }
    pub fn direction(self) -> Direction { catalog::registry().negotiate(self.code()).metadata.direction }
    pub fn available() -> impl Iterator<Item = Self> + Clone {
        Self::ALL.into_iter()
    }
    pub fn system() -> Self {
        ["LC_ALL", "LC_MESSAGES", "LANG"].into_iter().filter_map(|name| std::env::var(name).ok())
            .find(|value| !value.is_empty()).map(|value| Self::from_code(&value)).unwrap_or_default()
    }
    pub fn lookup(self, id: &str) -> Option<&'static str> {
        let registry = catalog::registry();
        let selected = registry.negotiate(self.code());
        let source = registry.source();
        if let Some(text) = selected.text(id).or_else(|| source.text(id)) {
            return Some(text);
        }
        if id.contains('.') || id.contains('_') {
            let normalized = id.replace(['.', '_'], "-");
            if let Some(text) = selected.text(&normalized).or_else(|| source.text(&normalized)) {
                return Some(text);
            }
        }
        None
    }
    pub fn text(self, id: &str) -> &str { self.lookup(id).unwrap_or(id) }
    /// Text-only parameters. Numeric/select messages use `format_args`.
    pub fn format(self, id: &str, values: &[(&str, &str)]) -> String {
        let args = Arguments::from_iter(values.iter().map(|(name, value)| ((*name).to_owned(), Argument::Text((*value).to_owned()))));
        self.format_args(id, &args)
    }
    pub fn format_args(self, id: &str, arguments: &Arguments) -> String {
        catalog::registry().format(self.code(), id, &arguments.fluent(self))
    }
    pub fn count(self, one: &str, _many: &str, count: usize) -> String {
        let base = one.strip_suffix(".one").or_else(|| one.strip_suffix("-one")).unwrap_or(one);
        self.format_args(base, &crate::i18n_args!(count = count))
    }
    /// Localize explicit message IDs; never infer a translation from a phrase.
    pub fn message(self, source: &str) -> String { self.text(source).to_owned() }
    pub fn export(self, prefixes: &[&str]) -> BTreeMap<String, String> {
        catalog::registry().source().messages.keys().filter(|id| prefixes.iter().any(|prefix| id.starts_with(prefix)))
            .map(|id| (id.clone(), self.text(id).to_owned())).collect()
    }
    pub fn interface_messages(self) -> BTreeMap<String, String> { self.export(&[""]) }
    pub fn resources(self) -> serde_json::Value {
        let registry = catalog::registry();
        let selected = registry.negotiate(self.code());
        serde_json::json!({
            "locale": selected.metadata.id, "direction": selected.metadata.direction,
            "resources": selected.sources.iter().map(|(_, source)| source).collect::<Vec<_>>(),
            "fallbackLocale": catalog::SOURCE_LOCALE,
            "fallbackResources": registry.source().sources.iter().map(|(_, source)| source).collect::<Vec<_>>()
        })
    }
}
pub type Translations = BTreeMap<String, BTreeMap<String, String>>;
pub fn localize<'a>(translations: &'a Translations, language: Language, key: &str, fallback: &'a str) -> &'a str {
    if let Some(text) = translations.get(language.code()).and_then(|entry| entry.get(key)) {
        return text;
    }
    let base = language.code().split('-').next().unwrap_or(catalog::SOURCE_LOCALE);
    if let Some(text) = translations.get(base).and_then(|entry| entry.get(key)) {
        return text;
    }
    // Bundled manifests use Portuguese for their source strings.
    if language != Language::Portuguese
        && let Some(text) = translations.get(catalog::SOURCE_LOCALE).and_then(|entry| entry.get(key))
    {
        return text;
    }
    fallback
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn english_is_the_source_and_default_locale() {
        assert_eq!(Language::default().code(), "en");
        assert_eq!(Language::from_code("unsupported").code(), "en");
        assert_eq!(Language::from_code("pt_BR").code(), "pt-BR");
        assert_eq!(serde_json::to_string(&Language::from_code("es-MX")).unwrap(), "\"es\"");
    }
    #[test]
    fn bundled_translations_match_the_english_message_schema() {
        catalog::registry().validate().unwrap();
        let source = &catalog::registry().source().messages;
        for locale in &catalog::registry().catalogs {
            assert_eq!(locale.messages.keys().collect::<Vec<_>>(), source.keys().collect::<Vec<_>>(), "{}", locale.metadata.id);
        }
    }
}
