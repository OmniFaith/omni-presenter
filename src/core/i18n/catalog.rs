//! Fluent resources and BCP 47 negotiation, with per-message English fallback.
use fluent_bundle::{FluentArgs, FluentResource, concurrent::FluentBundle};
use fluent_syntax::ast;
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, BTreeSet}, sync::OnceLock};
include!(concat!(env!("OUT_DIR"), "/locale_assets.rs"));
pub const SOURCE_LOCALE: &str = "en";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LocaleMetadata {
    pub id: String,
    pub native_name: String,
    pub english_name: String,
    pub direction: Direction,
}
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Direction { #[default] Ltr, Rtl }

pub struct Catalog {
    pub metadata: LocaleMetadata,
    negotiation_id: fluent_langneg::LanguageIdentifier,
    pub sources: Vec<(String, String)>,
    pub messages: BTreeMap<String, BTreeSet<String>>,
    simple: BTreeMap<String, String>,
    bundle: FluentBundle<FluentResource>,
}
impl Catalog {
    pub fn load(metadata: LocaleMetadata, sources: &[(&str, &str)]) -> Result<Self, String> {
        let identifier = metadata.id.parse::<unic_langid::LanguageIdentifier>().map_err(|error| error.to_string())?;
        let negotiation_id = metadata.id.parse::<fluent_langneg::LanguageIdentifier>().map_err(|error| error.to_string())?;
        let mut bundle = FluentBundle::new_concurrent(vec![identifier]);
        bundle.set_use_isolating(false);
        let mut messages = BTreeMap::new();
        for (name, source) in sources {
            let resource = FluentResource::try_new(source.to_string()).map_err(|(_, errors)| format!("{}/{name}: {errors:?}", metadata.id))?;
            for entry in resource.entries() {
                if let ast::Entry::Message(message) = entry {
                    let mut arguments = BTreeSet::new();
                    if let Some(pattern) = &message.value { collect_pattern(pattern, &mut arguments); }
                    if messages.insert(message.id.name.to_owned(), arguments).is_some() {
                        return Err(format!("{}/{name}: duplicate message {}", metadata.id, message.id.name));
                    }
                }
            }
            bundle.add_resource(resource).map_err(|errors| format!("{}/{name}: {errors:?}", metadata.id))?;
        }
        let mut simple = BTreeMap::new();
        for (id, arguments) in &messages {
            if arguments.is_empty()
                && let Some(pattern) = bundle.get_message(id).and_then(|message| message.value()) {
                    let mut errors = Vec::new();
                    let value = bundle.format_pattern(pattern, None, &mut errors).into_owned();
                    if !errors.is_empty() { return Err(format!("{}/{id}: {errors:?}", metadata.id)); }
                    simple.insert(id.clone(), value);
                }
        }
        Ok(Self { metadata, negotiation_id, messages, simple, bundle,
            sources: sources.iter().map(|(name, source)| (name.to_string(), source.to_string())).collect() })
    }
    pub fn text(&self, id: &str) -> Option<&str> { self.simple.get(id).map(String::as_str) }
    pub fn format(&self, id: &str, args: &FluentArgs<'_>) -> Option<Result<String, String>> {
        let message = self.bundle.get_message(id)?;
        let pattern = message.value()?;
        let mut errors = Vec::new();
        let value = self.bundle.format_pattern(pattern, Some(args), &mut errors).into_owned();
        Some(if errors.is_empty() { Ok(value) } else { Err(format!("{}/{id}: {errors:?}", self.metadata.id)) })
    }
}
fn collect_pattern(pattern: &ast::Pattern<&str>, arguments: &mut BTreeSet<String>) {
    for element in &pattern.elements {
        if let ast::PatternElement::Placeable { expression } = element { collect_expression(expression, arguments); }
    }
}
fn collect_expression(expression: &ast::Expression<&str>, arguments: &mut BTreeSet<String>) {
    match expression {
        ast::Expression::Inline(expression) => collect_inline(expression, arguments),
        ast::Expression::Select { selector, variants } => {
            collect_inline(selector, arguments);
            for variant in variants { collect_pattern(&variant.value, arguments); }
        }
    }
}
fn collect_inline(expression: &ast::InlineExpression<&str>, arguments: &mut BTreeSet<String>) {
    match expression {
        ast::InlineExpression::VariableReference { id } => { arguments.insert(id.name.to_owned()); }
        ast::InlineExpression::Placeable { expression } => collect_expression(expression, arguments),
        ast::InlineExpression::FunctionReference { arguments: call, .. } => {
            for argument in &call.positional { collect_inline(argument, arguments); }
            for argument in &call.named { collect_inline(&argument.value, arguments); }
        }
        _ => {}
    }
}
pub struct Registry { pub catalogs: Vec<Catalog> }
impl Registry {
    pub fn source(&self) -> &Catalog {
        self.catalogs.iter().find(|catalog| catalog.metadata.id == SOURCE_LOCALE).expect("English source catalog is required")
    }
    pub fn lookup(&self, requested: &str) -> Option<&Catalog> {
        let requested = requested.replace('_', "-");
        let requested = requested.split('.').next()?.parse::<fluent_langneg::LanguageIdentifier>().ok()?;
        let available: Vec<_> = self.catalogs.iter().map(|catalog| &catalog.negotiation_id).collect();
        let matches = fluent_langneg::negotiate_languages(&[requested], &available, None, fluent_langneg::NegotiationStrategy::Lookup);
        matches.first().and_then(|identifier| self.catalogs.iter().find(|catalog| catalog.negotiation_id == ***identifier))
    }
    pub fn negotiate(&self, requested: &str) -> &Catalog { self.lookup(requested).unwrap_or_else(|| self.source()) }
    pub fn format(&self, locale: &str, id: &str, args: &FluentArgs<'_>) -> String {
        if let Some(Ok(value)) = self.negotiate(locale).format(id, args) { return value; }
        if let Some(Ok(value)) = self.source().format(id, args) { return value; }
        if id.contains('.') || id.contains('_') {
            let normalized = id.replace(['.', '_'], "-");
            if let Some(Ok(value)) = self.negotiate(locale).format(&normalized, args) { return value; }
            if let Some(Ok(value)) = self.source().format(&normalized, args) { return value; }
        }
        id.to_owned()
    }
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let source = self.source();
        let mut errors = Vec::new();
        for catalog in &self.catalogs {
            for (id, arguments) in &catalog.messages {
                match source.messages.get(id) {
                    None => errors.push(format!("{}/{id}: not declared in English", catalog.metadata.id)),
                    Some(expected) if expected != arguments => errors.push(format!("{}/{id}: arguments differ from English", catalog.metadata.id)),
                    _ => {}
                }
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
pub fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let catalogs = BUNDLED_LOCALES.iter().map(|(code, metadata, sources)| {
            let metadata: LocaleMetadata = serde_json::from_str(metadata).expect("invalid locale metadata");
            assert_eq!(metadata.id, *code, "locale directory and metadata disagree");
            Catalog::load(metadata, sources).unwrap_or_else(|error| panic!("{error}"))
        }).collect();
        let registry = Registry { catalogs };
        registry.source();
        registry
    })
}
