use super::Language;
use fluent_bundle::FluentArgs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    Text(String),
    Number(f64),
    Message(Box<super::Message>),
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Arguments(pub BTreeMap<String, Argument>);

impl FromIterator<(String, Argument)> for Arguments {
    fn from_iter<T: IntoIterator<Item = (String, Argument)>>(values: T) -> Self {
        Self(values.into_iter().collect())
    }
}

impl Arguments {
    pub(super) fn fluent(&self, language: Language) -> FluentArgs<'_> {
        let mut args = FluentArgs::new();
        for (name, value) in &self.0 {
            match value {
                Argument::Text(text) => args.set(name.as_str(), text.as_str()),
                Argument::Number(number) => args.set(name.as_str(), *number),
                Argument::Message(message) => args.set(name.as_str(), message.render(language)),
            }
        }
        args
    }
}

impl From<&str> for Argument { fn from(value: &str) -> Self { Self::Text(value.to_owned()) } }
impl From<&String> for Argument { fn from(value: &String) -> Self { Self::Text(value.clone()) } }
impl From<String> for Argument { fn from(value: String) -> Self { Self::Text(value) } }
impl From<bool> for Argument { fn from(value: bool) -> Self { Self::Text(value.to_string()) } }
impl From<super::Message> for Argument { fn from(value: super::Message) -> Self { Self::Message(Box::new(value)) } }
macro_rules! numbers { ($($kind:ty),*) => { $(impl From<$kind> for Argument { fn from(value: $kind) -> Self { Self::Number(value as f64) } })* }; }
numbers!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);

#[macro_export]
macro_rules! i18n_args {
    ($($name:ident = $value:expr),* $(,)?) => {
        $crate::core::i18n::Arguments::from_iter([
            $((stringify!($name).to_owned(), $crate::core::i18n::Argument::from($value))),*
        ])
    };
}
