use crate::core::layout::{ContainerKind, DropZone, LayoutTree};
use crate::core::models::{PlaylistItem, QuickItem};
use serde::{Deserialize, Serialize};

pub fn new_id(prefix: &str) -> String {
    static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!(
        "{prefix}_{time}_{}",
        SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OperatorProfile {
    pub id: String,
    pub name: String,
    pub dark_mode: bool,
    #[serde(default)]
    pub theme_mode: crate::core::interface_theme::ThemeMode,
    pub interface_theme_id: String,
    pub scale: u32,
    pub reduce_motion: bool,
    pub keyboard_shortcuts: bool,
    pub show_dock: bool,
    #[serde(default)]
    pub dock_preview_bg: bool,
    #[serde(default = "default_dock_height")]
    pub dock_height: f32,
    #[serde(default)]
    pub lyrics_cards_layout: bool,
    #[serde(default)]
    pub bible_cards_layout: bool,
    pub layout: LayoutTree,
    pub bible_layout: LayoutTree,
    #[serde(default = "LayoutTree::default_chat")]
    pub chat_layout: LayoutTree,
    pub quick_items: Vec<QuickItem>,
    pub avatar: Option<Vec<u8>>,
    pub chat_notifications: bool,
    pub web_password_hash: String,
}

fn default_dock_height() -> f32 {
    175.0
}

impl Default for OperatorProfile {
    fn default() -> Self {
        Self {
            id: "operator_default".into(),
            name: "Operador principal".into(),
            dark_mode: false,
            theme_mode: crate::core::interface_theme::ThemeMode::System,
            interface_theme_id: "classic".into(),
            scale: 100,
            reduce_motion: false,
            keyboard_shortcuts: true,
            show_dock: true,
            dock_preview_bg: false,
            dock_height: 175.0,
            lyrics_cards_layout: false,
            bible_cards_layout: false,
            layout: LayoutTree::default(),
            bible_layout: LayoutTree::bible_workspace(),
            chat_layout: LayoutTree::default_chat(),
            quick_items: Vec::new(),
            avatar: None,
            chat_notifications: true,
            web_password_hash: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EventRecurrence {
    pub enabled: bool,
    /// Pattern: "none", "weekly", "monthly", "daily"
    pub pattern: String,
    /// Days of week: 0 = Sunday, 1 = Monday, ..., 6 = Saturday
    pub days_of_week: Vec<u8>,
    /// Time of day in "HH:MM" format, e.g. "19:00"
    pub time: String,
    #[serde(default)]
    pub all_day: bool,
    #[serde(default)]
    pub week_of_month: u8,
    #[serde(default)]
    pub months: Vec<u8>,
    #[serde(default)]
    pub specific_dates: Vec<String>,
}

impl Default for EventRecurrence {
    fn default() -> Self {
        Self {
            enabled: false,
            pattern: "none".to_string(),
            days_of_week: vec![0],
            time: "19:00".to_string(),
            all_day: false,
            week_of_month: 0,
            months: Vec::new(),
            specific_dates: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ServiceProfile {
    pub id: String,
    pub name: String,
    pub theme_id: String,
    /// Last preview selection, separate from the event's configured default.
    pub last_selected_theme_id: Option<String>,
    /// The configured default when that selection was saved; detects edits.
    pub last_selected_theme_default_id: Option<String>,
    pub idle_theme_id: String,
    #[serde(default)]
    pub wallpaper: Option<PhysicalBackground>,
    /// 0 = monitor, otherwise the chosen width / height.
    pub aspect_ratio: f32,
    pub bible_version: String,
    #[serde(default)]
    pub bible_shortcuts: Vec<String>,
    #[serde(default)]
    pub bible_shortcuts_initialized: bool,
    pub playlist: Vec<PlaylistItem>,
    pub last_bible_book: String,
    pub last_bible_chapter: usize,
    pub last_bible_verse: usize,
    pub last_bible_verse_end: usize,
    #[serde(default)]
    pub event_date: Option<String>,
    #[serde(default)]
    pub recurrence: EventRecurrence,
    #[serde(default)]
    pub sessions: Vec<ServiceSession>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ServiceSession {
    pub id: String,
    pub date: String, // "YYYY-MM-DD"
    #[serde(default)]
    pub playlist: Vec<PlaylistItem>,
}

impl ServiceProfile {
    pub fn get_session(&self, date: &str) -> Option<&ServiceSession> {
        self.sessions.iter().find(|s| s.date == date)
    }

    pub fn get_session_mut(&mut self, date: &str) -> Option<&mut ServiceSession> {
        self.sessions.iter_mut().find(|s| s.date == date)
    }

    pub fn save_session_playlist(&mut self, date: &str, playlist: Vec<PlaylistItem>) {
        if let Some(session) = self.sessions.iter_mut().find(|s| s.date == date) {
            session.playlist = playlist;
        } else {
            self.sessions.push(ServiceSession {
                id: new_id("session"),
                date: date.to_string(),
                playlist,
            });
            self.sessions.sort_by(|a, b| a.date.cmp(&b.date));
        }
    }

    pub fn occurs_on_date(&self, date_str: &str) -> bool {
        if let Some(ref d) = self.event_date {
            if !d.trim().is_empty() && d.trim() == date_str {
                return true;
            }
        }
        if self.recurrence.enabled {
            if self.recurrence.pattern == "daily" {
                return true;
            }
            if self.recurrence.pattern == "weekly" {
                if let Ok(d) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    use chrono::Datelike;
                    let weekday = d.weekday().num_days_from_sunday() as u8;
                    let days = if self.recurrence.days_of_week.is_empty() {
                        vec![0]
                    } else {
                        self.recurrence.days_of_week.clone()
                    };
                    return days.contains(&weekday);
                }
            }
        }
        false
    }

    pub fn current_session_date(&self) -> String {
        if let Some(ref d) = self.event_date {
            if !d.trim().is_empty() {
                return d.trim().to_string();
            }
        }
        let today = chrono::Local::now().date_naive();
        if self.recurrence.enabled && self.recurrence.pattern == "weekly" {
            let days = if self.recurrence.days_of_week.is_empty() {
                vec![0]
            } else {
                self.recurrence.days_of_week.clone()
            };
            use chrono::Datelike;
            let current_weekday = today.weekday().num_days_from_sunday() as u8;
            if days.contains(&current_weekday) {
                return today.format("%Y-%m-%d").to_string();
            }
            for offset in 1..=7 {
                let candidate = today + chrono::Duration::days(offset);
                if days.contains(&(candidate.weekday().num_days_from_sunday() as u8)) {
                    return candidate.format("%Y-%m-%d").to_string();
                }
            }
        }
        today.format("%Y-%m-%d").to_string()
    }

    pub fn step_date(&self, current_date_str: &str, delta: i32) -> String {
        let step_days = if self.recurrence.enabled && self.recurrence.pattern == "daily" {
            delta as i64
        } else {
            (delta * 7) as i64
        };
        if let Ok(d) = chrono::NaiveDate::parse_from_str(current_date_str, "%Y-%m-%d") {
            let next_date = d + chrono::Duration::days(step_days);
            next_date.format("%Y-%m-%d").to_string()
        } else {
            let today = chrono::Local::now().date_naive();
            let next_date = today + chrono::Duration::days(step_days);
            next_date.format("%Y-%m-%d").to_string()
        }
    }

    pub fn schedule_label(&self) -> String {
        self.schedule_label_localized(crate::core::i18n::Language::Portuguese)
    }

    pub fn schedule_label_localized(&self, lang: crate::core::i18n::Language) -> String {
        if self.recurrence.enabled {
            let time_suffix = if self.recurrence.all_day {
                format!(" ({})", lang.text("workspace-all-day"))
            } else if self.recurrence.time.trim().is_empty() {
                String::new()
            } else {
                format!(
                    " {}",
                    lang.format(
                        "workspace-recurrence-at",
                        &[("time", self.recurrence.time.trim())]
                    )
                )
            };

            match self.recurrence.pattern.as_str() {
                "daily" => {
                    let daily_txt = lang.text("workspace-recurrence-daily");
                    format!("{daily_txt}{time_suffix}")
                }
                "weekly" => {
                    if self.recurrence.days_of_week.is_empty() {
                        let weekly_txt = lang.text("workspace-recurrence-weekly");
                        format!("{weekly_txt}{time_suffix}")
                    } else {
                        let day_keys = [
                            "workspace-day-sun",
                            "workspace-day-mon",
                            "workspace-day-tue",
                            "workspace-day-wed",
                            "workspace-day-thu",
                            "workspace-day-fri",
                            "workspace-day-sat",
                        ];
                        let formatted: Vec<&str> = self
                            .recurrence
                            .days_of_week
                            .iter()
                            .filter_map(|&d| day_keys.get(d as usize).map(|k| lang.text(k)))
                            .collect();
                        if formatted.len() == 1 {
                            let full_day_keys = [
                                "workspace-full-day-sun",
                                "workspace-full-day-mon",
                                "workspace-full-day-tue",
                                "workspace-full-day-wed",
                                "workspace-full-day-thu",
                                "workspace-full-day-fri",
                                "workspace-full-day-sat",
                            ];
                            let day_key = full_day_keys
                                .get(self.recurrence.days_of_week[0] as usize)
                                .copied()
                                .unwrap_or("workspace-full-day-default");
                            let full_day = lang.text(day_key);
                            if !self.recurrence.time.trim().is_empty() {
                                lang.format(
                                    "workspace-recurrence-every-day-at",
                                    &[("day", full_day), ("time", self.recurrence.time.trim())],
                                )
                            } else {
                                lang.format(
                                    "workspace-recurrence-every-day",
                                    &[("day", full_day)],
                                )
                            }
                        } else {
                            format!("{}{time_suffix}", formatted.join(", "))
                        }
                    }
                }
                "monthly" => {
                    let mut desc = lang.text("workspace-recurrence-monthly").to_string();
                    if self.recurrence.week_of_month > 0 && !self.recurrence.days_of_week.is_empty() {
                        let week_label = match self.recurrence.week_of_month {
                            1 => lang.text("workspace-week-1"),
                            2 => lang.text("workspace-week-2"),
                            3 => lang.text("workspace-week-3"),
                            4 => lang.text("workspace-week-4"),
                            5 => lang.text("workspace-week-last"),
                            _ => lang.text("workspace-week-any"),
                        };
                        let day_keys = [
                            "workspace-day-sun", "workspace-day-mon", "workspace-day-tue",
                            "workspace-day-wed", "workspace-day-thu", "workspace-day-fri", "workspace-day-sat",
                        ];
                        let days_str: Vec<&str> = self.recurrence.days_of_week.iter()
                            .filter_map(|&d| day_keys.get(d as usize).map(|k| lang.text(k)))
                            .collect();
                        desc = format!("{week_label} ({})", days_str.join(", "));
                    }
                    format!("{desc}{time_suffix}")
                }
                _ => {
                    if !self.recurrence.specific_dates.is_empty() {
                        let first = &self.recurrence.specific_dates[0];
                        if self.recurrence.specific_dates.len() > 1 {
                            return format!("{} (+{} datas){time_suffix}", first, self.recurrence.specific_dates.len() - 1);
                        } else {
                            return format!("{first}{time_suffix}");
                        }
                    }
                    if let Some(date) = &self.event_date {
                        if !date.is_empty() {
                            return format!("{date}{time_suffix}");
                        }
                    }
                    let def_txt = lang.text("workspace-recurrence-none");
                    format!("{def_txt}{time_suffix}")
                }
            }
        } else if !self.recurrence.specific_dates.is_empty() {
            let first = &self.recurrence.specific_dates[0];
            let time_suffix = if self.recurrence.all_day {
                format!(" ({})", lang.text("workspace-all-day"))
            } else if self.recurrence.time.trim().is_empty() {
                String::new()
            } else {
                format!(
                    " {}",
                    lang.format(
                        "workspace-recurrence-at",
                        &[("time", self.recurrence.time.trim())]
                    )
                )
            };
            if self.recurrence.specific_dates.len() > 1 {
                return format!("{} (+{} datas){time_suffix}", first, self.recurrence.specific_dates.len() - 1);
            } else {
                return format!("{first}{time_suffix}");
            }
        } else if let Some(date) = &self.event_date {
            if !date.trim().is_empty() {
                let time_suffix = if self.recurrence.all_day {
                    format!(" ({})", lang.text("workspace-all-day"))
                } else if self.recurrence.time.trim().is_empty() {
                    String::new()
                } else {
                    format!(
                        " {}",
                        lang.format(
                            "workspace-recurrence-at",
                            &[("time", self.recurrence.time.trim())]
                        )
                    )
                };
                return format!("{date}{time_suffix}");
            }
            String::new()
        } else {
            String::new()
        }
    }

    pub fn display_name(&self, language: crate::core::i18n::Language) -> String {
        if self.name.is_empty()
            || self.name == "workspace-new-event"
            || self.name == "Culto principal"
            || self.name == "services-main-service"
        {
            language.text("workspace-new-event").to_string()
        } else {
            language.text(&self.name).to_string()
        }
    }
}

impl Default for ServiceProfile {
    fn default() -> Self {
        Self {
            id: "service_default".into(),
            name: "workspace-new-event".into(),
            theme_id: "obsidian_pro".into(),
            last_selected_theme_id: None,
            last_selected_theme_default_id: None,
            idle_theme_id: "obsidian_pro".into(),
            wallpaper: None,
            aspect_ratio: 0.0,
            bible_version: String::new(),
            bible_shortcuts: Vec::new(),
            bible_shortcuts_initialized: false,
            playlist: Vec::new(),
            last_bible_book: "Gênesis".into(),
            last_bible_chapter: 1,
            last_bible_verse: 1,
            last_bible_verse_end: 1,
            event_date: None,
            recurrence: EventRecurrence::default(),
            sessions: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VirtualScreen {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub lyrics_only: bool,
    pub show_title: bool,
    pub font_scale: u32,
    pub show_comments: bool,
    pub custom_theme_id: Option<String>,
    pub show_clock: bool,
    pub show_next_preview: bool,
    pub custom_background: Option<PhysicalBackground>,
}

impl Default for VirtualScreen {
    fn default() -> Self {
        Self {
            id: "auditorio".into(),
            name: "Auditório".into(),
            enabled: true,
            lyrics_only: false,
            show_title: false,
            font_scale: 100,
            show_comments: false,
            custom_theme_id: None,
            show_clock: false,
            show_next_preview: false,
            custom_background: None,
        }
    }
}

impl VirtualScreen {
    pub fn slug(&self) -> String {
        let s = slugify(&self.name);
        if s.is_empty() {
            self.id.clone()
        } else {
            s
        }
    }
}

/// Converts arbitrary text into a clean, URL-safe slug.
/// E.g. "Auditório" -> "auditorio", "Tela 1" -> "tela-1", "OBS - Live" -> "obs-live".
pub fn slugify(text: &str) -> String {
    let mut result = String::new();
    let mut prev_dash = false;
    for c in text.chars() {
        let normalized = match c {
            'a'..='z' | '0'..='9' => Some(c),
            'A'..='Z' => Some(c.to_ascii_lowercase()),
            'á' | 'à' | 'ã' | 'â' | 'ä' | 'Á' | 'À' | 'Ã' | 'Â' | 'Ä' => Some('a'),
            'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => Some('e'),
            'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => Some('i'),
            'ó' | 'ò' | 'õ' | 'ô' | 'ö' | 'Ó' | 'Ò' | 'Õ' | 'Ô' | 'Ö' => Some('o'),
            'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => Some('u'),
            'ç' | 'Ç' => Some('c'),
            'ñ' | 'Ñ' => Some('n'),
            ' ' | '-' | '_' | '/' | '\\' | '.' => {
                if !result.is_empty() && !prev_dash {
                    result.push('-');
                    prev_dash = true;
                }
                None
            }
            _ => None,
        };
        if let Some(nc) = normalized {
            result.push(nc);
            prev_dash = false;
        }
    }
    result.trim_matches('-').to_string()
}

/// Resolves a unique URL slug given a screen name, fallback id, and the slugs of screens before it.
pub fn unique_screen_slug(name: &str, id: &str, other_slugs_before: &[String]) -> String {
    let s = slugify(name);
    let base = if s.is_empty() { id.to_string() } else { s };
    let count = other_slugs_before.iter().filter(|&s| s == &base).count();
    if count == 0 {
        base
    } else {
        format!("{}-{}", base, count + 1)
    }
}

/// Resolves a unique URL slug for the virtual presentation screen.
/// If multiple screens share the same base name, an index is appended (`tela-1`, `tela-1-2`, etc.).
pub fn screen_url_slug(screen: &VirtualScreen, all_screens: &[VirtualScreen]) -> String {
    let other_slugs_before: Vec<String> = all_screens
        .iter()
        .take_while(|s| s.id != screen.id)
        .map(|s| s.slug())
        .collect();
    unique_screen_slug(&screen.name, &screen.id, &other_slugs_before)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StropheTagDefinition {
    pub name: String,
    pub color: [u8; 3],
}

impl StropheTagDefinition {
    pub fn new(name: impl Into<String>, color: [u8; 3]) -> Self {
        Self {
            name: name.into(),
            color,
        }
    }

    pub fn default_tags() -> Vec<Self> {
        vec![
            Self::new("Refrão", [14, 165, 233]),
            Self::new("Ponte", [168, 85, 247]),
            Self::new("Verso", [100, 116, 139]),
            Self::new("Intro", [16, 185, 129]),
            Self::new("Final", [249, 115, 22]),
        ]
    }

    pub fn color_for(tags: &[Self], name: &str, fallback: [u8; 3]) -> [u8; 3] {
        let name_lower = name.trim().to_lowercase();
        for tag in tags {
            if tag.name.trim().to_lowercase() == name_lower {
                return tag.color;
            }
        }
        fallback
    }

    pub fn localized_name<'a>(&'a self, lang: crate::core::i18n::Language) -> &'a str {
        Self::localize_tag_name(&self.name, lang)
    }

    pub fn localize_tag_name<'a>(name: &'a str, lang: crate::core::i18n::Language) -> &'a str {
        match name.trim().to_lowercase().as_str() {
            t if t == "refrão" || t == "refrao" || t == "chorus" || t == "estribillo" => {
                lang.text("workspace-tag-chorus")
            }
            t if t == "ponte" || t == "bridge" || t == "puente" => {
                lang.text("workspace-tag-bridge")
            }
            t if t == "verso" || t == "verse" => {
                lang.text("workspace-tag-verse")
            }
            t if t == "intro" => {
                lang.text("workspace-tag-intro")
            }
            t if t == "final" || t == "ending" => {
                lang.text("workspace-tag-ending")
            }
            _ => name,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub interface_themes: Vec<crate::core::interface_theme::InterfaceTheme>,
    pub plugins: Vec<crate::core::plugins::InstalledPlugin>,
    pub language: crate::core::i18n::Language,
    pub shortcuts: crate::core::shortcuts::ShortcutSettings,
    pub operators: Vec<OperatorProfile>,
    pub services: Vec<ServiceProfile>,
    pub active_operator: String,
    pub active_service: String,
    pub screens: Vec<VirtualScreen>,
    pub strophe_tags: Vec<StropheTagDefinition>,
    pub server_enabled: bool,
    pub server_port: u16,
    pub allow_lan: bool,
    pub monitor_name: String,
    pub physical_output_enabled: bool,
    pub persistent_physical_output: bool,
    pub physical_transition_ms: u32,
    pub disabled_physical_monitors: Vec<String>,
    pub physical_background: PhysicalBackground,
    pub monitor_backgrounds: std::collections::HashMap<String, PhysicalBackground>,
    pub recent_wallpapers: Vec<String>,
    pub use_wallpaper_in_presentation: bool,
    pub show_song_info: bool,
    pub bible_enabled: bool,
    pub ask_media_conversion: bool,
    #[serde(default = "default_true")]
    pub hardware_acceleration: bool,
    pub chat_enabled: bool,
    pub last_bible_book: String,
    pub last_bible_chapter: usize,
    pub last_bible_verse: usize,
    pub last_bible_verse_end: usize,
    pub bible_settings: crate::core::bible::BibleSettings,
    pub first_run_completed: bool,
    #[serde(default = "default_true")]
    pub show_profile_picker_on_startup: bool,
}

fn default_true() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            interface_themes: Vec::new(),
            plugins: crate::core::plugins::bundled(),
            operators: vec![OperatorProfile::default()],
            language: Default::default(),
            shortcuts: Default::default(),
            services: vec![ServiceProfile::default()],
            active_operator: "operator_default".into(),
            active_service: "service_default".into(),
            screens: vec![VirtualScreen::default()],
            strophe_tags: StropheTagDefinition::default_tags(),
            server_enabled: false,
            server_port: 8080,
            allow_lan: true,
            monitor_name: String::new(),
            physical_output_enabled: true,
            persistent_physical_output: true,
            physical_transition_ms: 350,
            disabled_physical_monitors: Vec::new(),
            physical_background: PhysicalBackground::default(),
            monitor_backgrounds: std::collections::HashMap::new(),
            recent_wallpapers: Vec::new(),
            use_wallpaper_in_presentation: false,
            show_song_info: true,
            bible_enabled: true,
            ask_media_conversion: true,
            hardware_acceleration: true,
            chat_enabled: true,
            last_bible_book: "Gênesis".into(),
            last_bible_chapter: 1,
            last_bible_verse: 1,
            last_bible_verse_end: 1,
            bible_settings: Default::default(),
            first_run_completed: false,
            show_profile_picker_on_startup: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PhysicalBackground {
    #[default]
    Transparent,
    Color {
        rgb: [u8; 3],
    },
    Image {
        path: String,
    },
    Video {
        path: String,
    },
    Design {
        id: String,
    },
    Theme {
        id: String,
    },
}

impl PhysicalBackground {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Transparent => "settings-wallpaper-transparent",
            Self::Color { .. } => "settings-wallpaper-color",
            Self::Image { .. } => "settings-wallpaper-image",
            Self::Video { .. } => "settings-wallpaper-video",
            Self::Design { .. } => "settings-wallpaper-design",
            Self::Theme { .. } => "settings-wallpaper-theme",
        }
    }
}

impl AppSettings {
    pub fn physical_monitor_enabled(&self, name: &str) -> bool {
        !self
            .disabled_physical_monitors
            .iter()
            .any(|disabled| disabled == name)
    }

    pub fn toggle_physical_monitor(&mut self, name: &str) {
        if self.physical_monitor_enabled(name) {
            self.disabled_physical_monitors.push(name.into());
        } else {
            self.disabled_physical_monitors
                .retain(|disabled| disabled != name);
        }
    }

    pub fn normalize(&mut self) {
        crate::core::plugins::bundled::migrate(&mut self.plugins);
        self.bible_enabled = true;
        self.chat_enabled = true;
        if self.operators.is_empty() {
            self.operators.push(OperatorProfile::default());
        }
        if self.services.is_empty() {
            self.services.push(ServiceProfile::default());
        }
        if !self.operators.iter().any(|p| p.id == self.active_operator) {
            self.active_operator = self.operators[0].id.clone();
        }
        if !self.services.iter().any(|p| p.id == self.active_service) {
            self.active_service = self.services[0].id.clone();
        }
        if self.last_bible_book.trim().is_empty() {
            self.last_bible_book = "Gênesis".into();
        }
        if self.last_bible_chapter == 0 {
            self.last_bible_chapter = 1;
        }
        if self.last_bible_verse == 0 {
            self.last_bible_verse = 1;
        }
        if self.last_bible_verse_end == 0 {
            self.last_bible_verse_end = self.last_bible_verse;
        }
        for s in &mut self.services {
            if s.last_bible_book.trim().is_empty() {
                s.last_bible_book = self.last_bible_book.clone();
            }
            if s.last_bible_chapter == 0 {
                s.last_bible_chapter = self.last_bible_chapter;
            }
            if s.last_bible_verse == 0 {
                s.last_bible_verse = self.last_bible_verse;
            }
            if s.last_bible_verse_end == 0 {
                s.last_bible_verse_end = s.last_bible_verse;
            }
            if s.theme_id == "hands_water" || s.theme_id == "starry_sky" || s.theme_id.is_empty() {
                s.theme_id = "obsidian_pro".into();
            }
            if s.idle_theme_id == "hands_water" || s.idle_theme_id == "starry_sky" || s.idle_theme_id.is_empty() {
                s.idle_theme_id = "obsidian_pro".into();
            }
        }
        self.server_port = self.server_port.max(1);
        self.physical_transition_ms = self.physical_transition_ms.min(2000);
        for p in &mut self.operators {
            p.scale = p.scale.clamp(80, 150);
            p.show_dock = true;
            p.keyboard_shortcuts = true;
            p.chat_notifications = true;
            if p.interface_theme_id == "midnight" {
                p.interface_theme_id = "darker_pro".into();
            }
            if !p.layout.root.contains(ContainerKind::Preview) {
                if p.layout.root.contains(ContainerKind::Stage) {
                    p.layout.dock(
                        ContainerKind::Preview,
                        ContainerKind::Stage,
                        DropZone::Bottom,
                    );
                } else {
                    p.layout = LayoutTree::default_4_columns();
                }
            }
            if !p.layout.root.contains(ContainerKind::Player) {
                if p.layout.root.contains(ContainerKind::Lyrics) {
                    p.layout
                        .dock(ContainerKind::Player, ContainerKind::Lyrics, DropZone::Top);
                } else if p.layout.root.contains(ContainerKind::Themes) {
                    p.layout
                        .dock(ContainerKind::Player, ContainerKind::Themes, DropZone::Top);
                } else if p.layout.root.contains(ContainerKind::Stage) {
                    p.layout.dock(
                        ContainerKind::Player,
                        ContainerKind::Stage,
                        DropZone::Bottom,
                    );
                }
            }
            if !p.bible_layout.root.contains(ContainerKind::Preview) {
                if p.bible_layout.root.contains(ContainerKind::Stage) {
                    p.bible_layout.dock(
                        ContainerKind::Preview,
                        ContainerKind::Stage,
                        DropZone::Bottom,
                    );
                } else {
                    p.bible_layout = LayoutTree::bible_workspace();
                }
            }
            if !p.bible_layout.root.contains(ContainerKind::Player) {
                if p.bible_layout.root.contains(ContainerKind::Lyrics) {
                    p.bible_layout.dock(
                        ContainerKind::Player,
                        ContainerKind::Lyrics,
                        DropZone::Top,
                    );
                } else if p.bible_layout.root.contains(ContainerKind::Themes) {
                    p.bible_layout.dock(
                        ContainerKind::Player,
                        ContainerKind::Themes,
                        DropZone::Top,
                    );
                } else if p.bible_layout.root.contains(ContainerKind::Stage) {
                    p.bible_layout.dock(
                        ContainerKind::Player,
                        ContainerKind::Stage,
                        DropZone::Bottom,
                    );
                }
            }
            if !p.chat_layout.root.contains(ContainerKind::ChatChannels)
                || !p.chat_layout.root.contains(ContainerKind::ChatMessages)
            {
                p.chat_layout = LayoutTree::default_chat();
            }
            p.layout.root.sanitize_fractions();
            p.bible_layout.root.sanitize_fractions();
            p.chat_layout.root.sanitize_fractions();
        }
        for s in &mut self.screens {
            s.font_scale = s.font_scale.clamp(50, 200);
        }
        self.recent_wallpapers.retain(|p| !p.trim().is_empty());
        if self.recent_wallpapers.len() > 10 {
            self.recent_wallpapers.truncate(10);
        }
    }

    pub fn operator(&self) -> &OperatorProfile {
        self.operators
            .iter()
            .find(|p| p.id == self.active_operator)
            .unwrap_or(&self.operators[0])
    }

    pub fn operator_mut(&mut self) -> &mut OperatorProfile {
        let active = self.active_operator.clone();
        if let Some(pos) = self.operators.iter().position(|p| p.id == active) {
            &mut self.operators[pos]
        } else {
            &mut self.operators[0]
        }
    }

    pub fn service(&self) -> &ServiceProfile {
        self.services
            .iter()
            .find(|p| p.id == self.active_service)
            .unwrap_or(&self.services[0])
    }

    pub fn service_mut(&mut self) -> &mut ServiceProfile {
        let active = self.active_service.clone();
        if let Some(pos) = self.services.iter().position(|p| p.id == active) {
            &mut self.services[pos]
        } else {
            &mut self.services[0]
        }
    }

    pub fn wallpaper_for_monitor(&self, monitor_name: &str) -> &PhysicalBackground {
        if !monitor_name.is_empty() {
            if let Some(bg) = self.monitor_backgrounds.get(monitor_name) {
                return bg;
            }
        }
        &self.physical_background
    }

    pub fn effective_wallpaper(&self, monitor_name: &str) -> &PhysicalBackground {
        if let Some(bg) = &self.service().wallpaper {
            return bg;
        }
        self.wallpaper_for_monitor(monitor_name)
    }

    pub fn wallpaper_for_monitor_mut(&mut self, monitor_name: &str) -> &mut PhysicalBackground {
        if !monitor_name.is_empty() {
            return self
                .monitor_backgrounds
                .entry(monitor_name.to_string())
                .or_insert_with(|| self.physical_background.clone());
        }
        &mut self.physical_background
    }

    pub fn remove_monitor_wallpaper(&mut self, monitor_name: &str) {
        self.monitor_backgrounds.remove(monitor_name);
    }

    pub fn has_monitor_wallpaper(&self, monitor_name: &str) -> bool {
        self.monitor_backgrounds.contains_key(monitor_name)
    }

    pub fn record_recent_wallpaper(&mut self, path: &str) {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return;
        }
        self.recent_wallpapers.retain(|p| p != trimmed);
        self.recent_wallpapers.insert(0, trimmed.to_string());
        if self.recent_wallpapers.len() > 10 {
            self.recent_wallpapers.truncate(10);
        }
    }
}
