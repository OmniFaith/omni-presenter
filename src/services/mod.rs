//! Service layer: persistence, networking and side-effecting integrations.

pub mod audio;
pub mod chat;
pub mod control;
pub mod credentials;
pub mod db;
pub mod discovery;
pub mod effects;
pub mod images;
pub(crate) mod i18n;
pub mod media;
pub mod media_conversion;
pub mod lrclib;
pub mod player;
pub mod plugins;
#[cfg(test)]
mod media_tests;
pub mod monitor;
pub mod profiles;
pub mod search;
pub mod shaders;
pub mod tasks;
pub mod video;
pub mod web;
mod web_http;
pub mod web_video;
pub mod windowing;
