//! Seed data functions for fresh workspace (clean, no mocked data).

use crate::core::models::{PlaylistItem, QuickItem, Song};

/// Song library for a clean install (empty).
pub fn songs() -> Vec<Song> {
    Vec::new()
}

/// Playlist for a fresh service (empty).
pub fn playlist() -> Vec<PlaylistItem> {
    Vec::new()
}

/// Quick items ("atalhos") for the top bar (empty).
pub fn quick_items() -> Vec<QuickItem> {
    Vec::new()
}
