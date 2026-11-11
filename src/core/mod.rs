//! Core domain layer: data structures, state and pure logic.

pub mod bible;
pub mod bible_books;
pub mod bible_format;
pub mod bible_search;
pub mod bible_split;
pub mod breaks;
pub mod commands;
pub mod design;
pub mod design_style;
pub mod emoji;
pub mod i18n;
pub mod interaction;
pub mod interface_theme;
pub mod plugins;
pub mod layout;
pub mod markup;
pub mod models;
pub mod preferences;
pub mod search;
pub mod seed;
pub mod shortcuts;
pub mod slides;
#[cfg(test)]
mod slides_tests;
pub mod state;
