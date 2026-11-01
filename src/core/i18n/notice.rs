use super::{Argument, Arguments, Language};
use serde::{Deserialize, Serialize};

/// Locale-neutral application notification or error. The UI/API chooses the
/// locale when rendering, so changing languages never freezes an old message.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    #[serde(default)]
    pub args: Arguments,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl Message {
    pub fn new(id: impl Into<String>) -> Self { Self { id: id.into(), ..Self::default() } }

    pub fn arg(mut self, name: impl Into<String>, value: impl Into<Argument>) -> Self {
        self.args.0.insert(name.into(), value.into());
        self
    }

    pub fn diagnostic(text: impl Into<String>) -> Self {
        Self { detail: Some(text.into()), ..Self::default() }
    }

    pub fn render(&self, language: Language) -> String {
        if self.id.is_empty() { return self.detail.clone().unwrap_or_default(); }
        language.format_args(&self.id, &self.args)
    }

    pub fn clear(&mut self) { *self = Self::default(); }
    pub fn is_empty(&self) -> bool { self.id.is_empty() && self.detail.as_deref().is_none_or(str::is_empty) }
}

impl std::fmt::Display for Message {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { formatter.write_str(&self.render(Language::default())) }
}
impl std::error::Error for Message {}
impl From<&str> for Message { fn from(id: &str) -> Self { Self::new(id) } }
impl From<String> for Message { fn from(text: String) -> Self { Self::diagnostic(text) } }
