use super::ControlPanelState;

#[derive(Clone, Debug, PartialEq)]
pub struct EventEditorDraft {
    pub id: Option<String>,
    pub name: String,
    pub pattern: String, // "none", "weekly", "monthly", "daily"
    pub days_of_week: Vec<u8>,
    pub time: String,
    pub date: String,
    pub copy_current_playlist: bool,
    pub theme_id: String,
    pub idle_theme_id: String,
}

impl Default for EventEditorDraft {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            pattern: "weekly".to_string(),
            days_of_week: vec![0], // Domingo por padrão
            time: "19:00".to_string(),
            date: String::new(),
            copy_current_playlist: false,
            theme_id: String::new(),
            idle_theme_id: String::new(),
        }
    }
}

impl ControlPanelState {
    pub fn open_new_event_modal(&mut self) {
        self.event_editor_draft = Some(EventEditorDraft::default());
        self.show_event_editor_modal = true;
    }

    pub fn open_edit_event_modal(&mut self, event_id: &str) {
        if let Some(service) = self.settings.services.iter().find(|s| s.id == event_id) {
            self.event_editor_draft = Some(EventEditorDraft {
                id: Some(service.id.clone()),
                name: service.display_name(self.settings.language),
                pattern: if service.recurrence.enabled {
                    service.recurrence.pattern.clone()
                } else {
                    "none".to_string()
                },
                days_of_week: if service.recurrence.days_of_week.is_empty() {
                    vec![0]
                } else {
                    service.recurrence.days_of_week.clone()
                },
                time: if service.recurrence.time.is_empty() {
                    "19:00".to_string()
                } else {
                    service.recurrence.time.clone()
                },
                date: service.event_date.clone().unwrap_or_default(),
                copy_current_playlist: false,
                theme_id: service.theme_id.clone(),
                idle_theme_id: service.idle_theme_id.clone(),
            });
            self.show_event_editor_modal = true;
        }
    }

    pub fn save_event_from_draft(&mut self) {
        let Some(draft) = self.event_editor_draft.take() else {
            self.show_event_editor_modal = false;
            return;
        };

        let trimmed_name = draft.name.trim().to_string();
        if trimmed_name.is_empty() {
            self.show_event_editor_modal = false;
            return;
        }

        self.capture_profiles();

        let recurrence_enabled = draft.pattern != "none";
        let recurrence = crate::core::preferences::EventRecurrence {
            enabled: recurrence_enabled,
            pattern: draft.pattern.clone(),
            days_of_week: draft.days_of_week,
            time: draft.time.trim().to_string(),
        };
        let event_date = if !draft.date.trim().is_empty() {
            Some(draft.date.trim().to_string())
        } else {
            None
        };

        if let Some(id) = draft.id {
            if let Some(service) = self.settings.services.iter_mut().find(|s| s.id == id) {
                service.name = trimmed_name.clone();
                service.recurrence = recurrence;
                service.event_date = event_date;
                service.theme_id = draft.theme_id.clone();
                service.idle_theme_id = draft.idle_theme_id.clone();
            }
            if self.settings.active_service == id && !draft.theme_id.is_empty() {
                if let Some(idx) = self.themes.iter().position(|t| t.id == draft.theme_id) {
                    self.selected_theme_index = idx;
                    self.live_theme_index = idx;
                    self.live_theme_id = draft.theme_id.clone();
                }
            }
            let msg = self.settings.language.format(
                "workspace-event-updated",
                &[("name", &trimmed_name)],
            );
            self.notify(msg);
        } else {
            let new_id = crate::core::preferences::new_id("service");
            let playlist = if draft.copy_current_playlist {
                let mut items = self.playlist.clone();
                for it in &mut items {
                    it.played = false;
                    it.id = crate::core::preferences::new_id("item");
                }
                items
            } else {
                Vec::new()
            };

            let new_profile = crate::core::preferences::ServiceProfile {
                id: new_id.clone(),
                name: trimmed_name.clone(),
                theme_id: draft.theme_id.clone(),
                idle_theme_id: draft.idle_theme_id.clone(),
                playlist,
                event_date,
                recurrence,
                ..Default::default()
            };

            self.settings.services.push(new_profile);
            self.switch_service(&new_id);
            if !draft.theme_id.is_empty() {
                if let Some(idx) = self.themes.iter().position(|t| t.id == draft.theme_id) {
                    self.selected_theme_index = idx;
                    self.live_theme_index = idx;
                    self.live_theme_id = draft.theme_id.clone();
                }
            }
            let msg = self.settings.language.format(
                "workspace-event-created",
                &[("name", &trimmed_name)],
            );
            self.notify(msg);
        }

        self.show_event_editor_modal = false;
    }

    pub fn delete_event(&mut self, event_id: &str) {
        if self.settings.services.len() <= 1 {
            return;
        }
        self.capture_profiles();
        if let Some(idx) = self.settings.services.iter().position(|s| s.id == event_id) {
            let deleted_name = self.settings.services[idx].display_name(self.settings.language);
            self.settings.services.remove(idx);
            let next_id = self.settings.services[0].id.clone();
            self.switch_service(&next_id);
            let msg = self.settings.language.format(
                "workspace-event-deleted",
                &[("name", &deleted_name)],
            );
            self.notify(msg);
        }
    }

    pub fn duplicate_event(&mut self, event_id: &str) {
        self.capture_profiles();
        if let Some(service) = self.settings.services.iter().find(|s| s.id == event_id).cloned() {
            let new_id = crate::core::preferences::new_id("service");
            let mut new_profile = service.clone();
            new_profile.id = new_id.clone();
            let base_name = service.display_name(self.settings.language);
            new_profile.name = self.settings.language.format(
                "workspace-copy-label",
                &[("name", &base_name)],
            );
            for item in &mut new_profile.playlist {
                item.played = false;
                item.id = crate::core::preferences::new_id("item");
            }
            for sess in &mut new_profile.sessions {
                sess.id = crate::core::preferences::new_id("session");
                for item in &mut sess.playlist {
                    item.played = false;
                    item.id = crate::core::preferences::new_id("item");
                }
            }
            let name = new_profile.name.clone();
            self.settings.services.push(new_profile);
            self.switch_service(&new_id);
            let msg = self.settings.language.format(
                "workspace-event-duplicated",
                &[("name", &name)],
            );
            self.notify(msg);
        }
    }

    pub fn copy_items_from_event(&mut self, source_id: &str) {
        self.capture_profiles();
        if let Some(source) = self.settings.services.iter().find(|s| s.id == source_id) {
            let mut cloned = source.playlist.clone();
            for it in &mut cloned {
                it.played = false;
                it.id = crate::core::preferences::new_id("item");
            }
            let count = cloned.len();
            let source_name = source.name.clone();
            self.playlist.extend(cloned);
            self.capture_profiles();
            let count_str = count.to_string();
            let msg = self.settings.language.format(
                "workspace-items-copied-from",
                &[("count", &count_str), ("source", &source_name)],
            );
            self.notify(msg);
        }
    }

    pub fn current_service_date(&self) -> String {
        self.settings.service().current_session_date()
    }

    pub fn switch_service_date(&mut self, new_date: &str) {
        self.capture_profiles();
        let target_date = new_date.trim().to_string();
        if let Some(service) = self
            .settings
            .services
            .iter_mut()
            .find(|s| s.id == self.settings.active_service)
        {
            service.event_date = Some(target_date.clone());
            if let Some(session) = service.get_session(&target_date) {
                self.playlist = session.playlist.clone();
            } else {
                self.playlist = Vec::new();
            }
        }
        self.selected_playlist_index = if self.playlist.is_empty() {
            None
        } else {
            Some(0)
        };
        let formatted = format_date_display_localized(&target_date, self.settings.language);
        let count_str = self.playlist.len().to_string();
        let msg = self.settings.language.format(
            "workspace-service-selected",
            &[("date", &formatted), ("count", &count_str)],
        );
        self.notify(msg);
    }

    pub fn step_service_date(&mut self, delta: i32) {
        let cur = self.current_service_date();
        let next_date = self.settings.service().step_date(&cur, delta);
        self.switch_service_date(&next_date);
    }

    pub fn copy_items_from_session_date(&mut self, from_date: &str) {
        self.capture_profiles();
        let from_date = from_date.trim().to_string();
        let items_to_copy = {
            let service = self.settings.service();
            service
                .get_session(&from_date)
                .map(|s| s.playlist.clone())
        };

        if let Some(mut items) = items_to_copy {
            for it in &mut items {
                it.played = false;
                it.id = crate::core::preferences::new_id("item");
            }
            let count = items.len();
            self.playlist.extend(items);
            self.capture_profiles();
            let count_str = count.to_string();
            let formatted = format_date_display_localized(&from_date, self.settings.language);
            let msg = self.settings.language.format(
                "workspace-items-copied-from-date",
                &[("count", &count_str), ("date", &formatted)],
            );
            self.notify(msg);
        } else {
            let msg = self.settings.language.format(
                "workspace-no-items-for-date",
                &[("date", &from_date)],
            );
            self.notify(msg);
        }
    }

    pub fn copy_items_from_previous_service(&mut self) {
        let cur = self.current_service_date();
        let prev_date = self.settings.service().step_date(&cur, -1);
        self.copy_items_from_session_date(&prev_date);
    }

    pub fn auto_select_nearest_service(&mut self) -> bool {
        if let Some((best_id, best_date)) =
            find_nearest_service_and_date(&self.settings.services)
        {
            let cur_id = self.settings.active_service.clone();
            let cur_date = self.current_service_date();
            let mut changed = false;
            if cur_id != best_id {
                self.switch_service(&best_id);
                changed = true;
            }
            if cur_date != best_date {
                self.switch_service_date(&best_date);
                changed = true;
            }
            let s_name = self.settings.service().name.clone();
            let date_label = format_date_display_localized(&best_date, self.settings.language);
            let msg = self.settings.language.format(
                "workspace-nearest-service-selected",
                &[("name", &s_name), ("date", &date_label)],
            );
            self.notify(msg);
            changed
        } else {
            false
        }
    }
}

pub fn format_date_display(date_str: &str) -> String {
    format_date_display_localized(date_str, crate::core::i18n::Language::Portuguese)
}

pub fn format_date_display_with_time(
    date_str: &str,
    time_str: &str,
    lang: crate::core::i18n::Language,
) -> String {
    let date_part = format_date_display_localized(date_str, lang);
    let trimmed_time = time_str.trim();
    if trimmed_time.is_empty() {
        date_part
    } else if let Some(idx) = date_part.find(" (") {
        let base = &date_part[..idx];
        let tag = &date_part[idx..];
        let at_time = lang.format("workspace-recurrence-at", &[("time", trimmed_time)]);
        format!("{base} {at_time}{tag}")
    } else {
        let at_time = lang.format("workspace-recurrence-at", &[("time", trimmed_time)]);
        format!("{date_part} {at_time}")
    }
}

pub fn format_date_short_with_time(
    date_str: &str,
    time_str: &str,
    lang: crate::core::i18n::Language,
) -> String {
    let date_part = format_date_short_localized(date_str, lang);
    let trimmed_time = time_str.trim();
    if trimmed_time.is_empty() {
        date_part
    } else if let Some(idx) = date_part.find(" (") {
        let base = &date_part[..idx];
        let tag = &date_part[idx..];
        format!("{base} · {trimmed_time}{tag}")
    } else {
        format!("{date_part} · {trimmed_time}")
    }
}

pub fn format_date_display_localized(
    date_str: &str,
    lang: crate::core::i18n::Language,
) -> String {
    use chrono::Datelike;
    let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") else {
        return date_str.to_string();
    };
    let today = chrono::Local::now().date_naive();
    let diff = (date - today).num_days();

    let day_key = match date.weekday().num_days_from_sunday() {
        0 => "workspace-day-sun",
        1 => "workspace-day-mon",
        2 => "workspace-day-tue",
        3 => "workspace-day-wed",
        4 => "workspace-day-thu",
        5 => "workspace-day-fri",
        6 => "workspace-day-sat",
        _ => "workspace-day-sun",
    };
    let day_name = lang.text(day_key);

    let month_key = match date.month() {
        1 => "workspace-month-1",
        2 => "workspace-month-2",
        3 => "workspace-month-3",
        4 => "workspace-month-4",
        5 => "workspace-month-5",
        6 => "workspace-month-6",
        7 => "workspace-month-7",
        8 => "workspace-month-8",
        9 => "workspace-month-9",
        10 => "workspace-month-10",
        11 => "workspace-month-11",
        12 => "workspace-month-12",
        _ => "workspace-month-1",
    };
    let month_name = lang.text(month_key);

    let base = format!("{day_name}, {:02}/{month_name}", date.day());
    if diff == 0 {
        let tag = lang.text("workspace-date-today");
        format!("{base} ({tag})")
    } else if diff == 1 {
        let tag = lang.text("workspace-date-tomorrow");
        format!("{base} ({tag})")
    } else if diff == -1 {
        let tag = lang.text("workspace-date-yesterday");
        format!("{base} ({tag})")
    } else if diff == 7 {
        let tag = lang.text("workspace-date-next-week");
        format!("{base} ({tag})")
    } else if diff == -7 {
        let tag = lang.text("workspace-date-last-week");
        format!("{base} ({tag})")
    } else if diff > 0 && diff < 7 {
        let count_str = diff.to_string();
        let tag = lang.format("workspace-date-in-days", &[("count", &count_str)]);
        format!("{base} ({tag})")
    } else if diff < 0 && diff > -7 {
        let count_str = (-diff).to_string();
        let tag = lang.format("workspace-date-days-ago", &[("count", &count_str)]);
        format!("{base} ({tag})")
    } else {
        base
    }
}

pub fn format_date_short(date_str: &str) -> String {
    format_date_short_localized(date_str, crate::core::i18n::Language::Portuguese)
}

pub fn format_date_short_localized(
    date_str: &str,
    lang: crate::core::i18n::Language,
) -> String {
    use chrono::Datelike;
    let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") else {
        return date_str.to_string();
    };
    let today = chrono::Local::now().date_naive();
    let diff = (date - today).num_days();

    let day_key = match date.weekday().num_days_from_sunday() {
        0 => "workspace-day-sun",
        1 => "workspace-day-mon",
        2 => "workspace-day-tue",
        3 => "workspace-day-wed",
        4 => "workspace-day-thu",
        5 => "workspace-day-fri",
        6 => "workspace-day-sat",
        _ => "workspace-day-sun",
    };
    let day_name = lang.text(day_key);

    let month_key = match date.month() {
        1 => "workspace-month-1",
        2 => "workspace-month-2",
        3 => "workspace-month-3",
        4 => "workspace-month-4",
        5 => "workspace-month-5",
        6 => "workspace-month-6",
        7 => "workspace-month-7",
        8 => "workspace-month-8",
        9 => "workspace-month-9",
        10 => "workspace-month-10",
        11 => "workspace-month-11",
        12 => "workspace-month-12",
        _ => "workspace-month-1",
    };
    let month_name = lang.text(month_key);

    let base = format!("{day_name}, {:02}/{month_name}", date.day());
    if diff == 0 {
        let tag = lang.text("workspace-date-today");
        format!("{base} ({tag})")
    } else {
        base
    }
}

pub fn find_nearest_service_and_date(
    services: &[crate::core::preferences::ServiceProfile],
) -> Option<(String, String)> {
    if services.is_empty() {
        return None;
    }
    use chrono::Datelike;
    let now = chrono::Local::now();
    let today = now.date_naive();
    let current_weekday = today.weekday().num_days_from_sunday() as u8;

    let mut best: Option<(i64, String, String)> = None;

    for s in services {
        let (target_date, target_time) = if s.recurrence.enabled && s.recurrence.pattern == "weekly"
        {
            let days = if s.recurrence.days_of_week.is_empty() {
                vec![0]
            } else {
                s.recurrence.days_of_week.clone()
            };
            let time_str = if s.recurrence.time.trim().is_empty() {
                "19:00"
            } else {
                s.recurrence.time.trim()
            };

            let mut chosen_date = today;
            if !days.contains(&current_weekday) {
                for offset in 1..=7 {
                    let candidate = today + chrono::Duration::days(offset);
                    if days.contains(&(candidate.weekday().num_days_from_sunday() as u8)) {
                        chosen_date = candidate;
                        break;
                    }
                }
            }
            (chosen_date, time_str.to_string())
        } else if let Some(ref d) = s.event_date {
            if let Ok(parsed) = chrono::NaiveDate::parse_from_str(d.trim(), "%Y-%m-%d") {
                let time_str = if s.recurrence.time.trim().is_empty() {
                    "19:00"
                } else {
                    s.recurrence.time.trim()
                };
                (parsed, time_str.to_string())
            } else {
                (today, "19:00".to_string())
            }
        } else {
            (today, "19:00".to_string())
        };

        let (hour, minute) = {
            let parts: Vec<&str> = target_time.split(':').collect();
            let h = parts
                .first()
                .and_then(|p| p.trim().parse::<u32>().ok())
                .unwrap_or(19);
            let m = parts
                .get(1)
                .and_then(|p| p.trim().parse::<u32>().ok())
                .unwrap_or(0);
            (h.min(23), m.min(59))
        };

        if let Some(target_dt) = target_date.and_hms_opt(hour, minute, 0) {
            let target_local = target_dt.and_local_timezone(chrono::Local).single();
            if let Some(dt) = target_local {
                let diff_min = (dt - now).num_minutes();
                let score = if diff_min >= -180 && diff_min <= 0 {
                    diff_min.abs()
                } else if diff_min > 0 {
                    diff_min
                } else {
                    diff_min.abs() + 10000
                };

                if best
                    .as_ref()
                    .map_or(true, |(best_score, _, _)| score < *best_score)
                {
                    best = Some((
                        score,
                        s.id.clone(),
                        target_date.format("%Y-%m-%d").to_string(),
                    ));
                }
            }
        }
    }

    best.map(|(_, id, date)| (id, date))
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::core::models::playlist::{PlaylistItem, PlaylistItemKind};
    use crate::core::preferences::EventRecurrence;

    #[test]
    fn test_event_schedule_label_formatting() {
        let weekly = crate::core::preferences::ServiceProfile {
            name: "Culto de Domingo".into(),
            recurrence: EventRecurrence {
                enabled: true,
                pattern: "weekly".into(),
                days_of_week: vec![0],
                time: "19:00".into(),
            },
            ..Default::default()
        };
        assert_eq!(weekly.schedule_label(), "Todo Domingo às 19:00");

        let multi = crate::core::preferences::ServiceProfile {
            name: "Ensaios".into(),
            recurrence: EventRecurrence {
                enabled: true,
                pattern: "weekly".into(),
                days_of_week: vec![2, 4],
                time: "20:00".into(),
            },
            ..Default::default()
        };
        assert_eq!(multi.schedule_label(), "Ter, Qui às 20:00");
    }

    #[test]
    fn test_event_creation_and_switching_with_separate_playlists() {
        let mut state = ControlPanelState::default();
        state.playlist.push(PlaylistItem {
            id: "item1".into(),
            title: "Música do Domingo".into(),
            subtitle: "Artista 1".into(),
            kind: PlaylistItemKind::Song,
            played: false,
            content: "Letra 1".into(),
        });
        state.capture_profiles();
        assert_eq!(state.playlist.len(), 1);

        state.event_editor_draft = Some(EventEditorDraft {
            id: None,
            name: "Culto de Quarta".into(),
            pattern: "weekly".into(),
            days_of_week: vec![3],
            time: "20:00".into(),
            date: String::new(),
            copy_current_playlist: false,
            ..Default::default()
        });
        state.save_event_from_draft();

        assert_eq!(state.settings.service().name, "Culto de Quarta");
        assert!(state.playlist.is_empty());

        state.playlist.push(PlaylistItem {
            id: "item2".into(),
            title: "Música de Quarta".into(),
            subtitle: "Artista 2".into(),
            kind: PlaylistItemKind::Song,
            played: false,
            content: "Letra 2".into(),
        });
        state.capture_profiles();
        assert_eq!(state.playlist.len(), 1);

        let first_id = state.settings.services[0].id.clone();
        state.switch_service(&first_id);
        assert_eq!(state.playlist.len(), 1);
        assert_eq!(state.playlist[0].title, "Música do Domingo");

        let quarta_id = state.settings.services[1].id.clone();
        state.copy_items_from_event(&quarta_id);
        assert_eq!(state.playlist.len(), 2);
        assert_eq!(state.playlist[1].title, "Música de Quarta");
    }

    #[test]
    fn test_event_duplication_and_deletion() {
        let mut state = ControlPanelState::default();
        state.playlist.push(PlaylistItem {
            id: "item1".into(),
            title: "Hino".into(),
            subtitle: "".into(),
            kind: PlaylistItemKind::Song,
            played: true,
            content: "".into(),
        });
        state.capture_profiles();

        let initial_id = state.settings.active_service.clone();
        state.duplicate_event(&initial_id);

        assert_eq!(state.settings.services.len(), 2);
        assert_eq!(state.settings.service().name, "New Event · copy");
        assert_eq!(state.playlist.len(), 1);
        assert!(!state.playlist[0].played);

        let copy_id = state.settings.active_service.clone();
        state.delete_event(&copy_id);
        assert_eq!(state.settings.services.len(), 1);
        assert_eq!(state.settings.active_service, initial_id);
    }

    #[test]
    fn test_default_and_legacy_service_profile_name_is_internationalized() {
        use crate::core::i18n::Language;
        let default_service = crate::core::preferences::ServiceProfile::default();
        assert_eq!(default_service.display_name(Language::English), "New Event");
        assert_eq!(default_service.display_name(Language::Portuguese), "Novo Evento");
        assert_eq!(default_service.display_name(Language::Spanish), "Nuevo Evento");

        let legacy_service = crate::core::preferences::ServiceProfile {
            name: "Culto principal".into(),
            ..Default::default()
        };
        assert_eq!(legacy_service.display_name(Language::English), "New Event");
        assert_eq!(legacy_service.display_name(Language::Portuguese), "Novo Evento");
        assert_eq!(legacy_service.display_name(Language::Spanish), "Nuevo Evento");

        let custom_service = crate::core::preferences::ServiceProfile {
            name: "Conferência de Jovens".into(),
            ..Default::default()
        };
        assert_eq!(custom_service.display_name(Language::English), "Conferência de Jovens");
        assert_eq!(custom_service.display_name(Language::Portuguese), "Conferência de Jovens");
    }

    #[test]
    fn test_date_stepping_and_session_playlists() {
        let mut state = ControlPanelState::default();
        // Switch to specific date
        state.switch_service_date("2026-10-25");
        state.playlist.push(PlaylistItem {
            id: "item_p1".into(),
            title: "Música de 25/Out".into(),
            subtitle: "".into(),
            kind: PlaylistItemKind::Song,
            played: true,
            content: "".into(),
        });
        state.capture_profiles();

        // Step +1 week to 2026-11-01
        state.step_service_date(1);
        assert_eq!(state.current_service_date(), "2026-11-01");
        assert!(state.playlist.is_empty()); // Fresh list for new Sunday

        // Add song for 2026-11-01
        state.playlist.push(PlaylistItem {
            id: "item_p2".into(),
            title: "Música de 01/Nov".into(),
            subtitle: "".into(),
            kind: PlaylistItemKind::Song,
            played: false,
            content: "".into(),
        });
        state.capture_profiles();

        // Step back -1 week to 2026-10-25
        state.step_service_date(-1);
        assert_eq!(state.current_service_date(), "2026-10-25");
        assert_eq!(state.playlist.len(), 1);
        assert_eq!(state.playlist[0].title, "Música de 25/Out");

        // Step forward again to 2026-11-01
        state.step_service_date(1);
        assert_eq!(state.current_service_date(), "2026-11-01");
        assert_eq!(state.playlist.len(), 1);
        assert_eq!(state.playlist[0].title, "Música de 01/Nov");

        // Copy from previous service (2026-10-25) into 2026-11-01
        state.copy_items_from_previous_service();
        assert_eq!(state.playlist.len(), 2);
        assert_eq!(state.playlist[1].title, "Música de 25/Out");
        assert!(!state.playlist[1].played); // Reset played status when copied
    }

    #[test]
    fn test_format_date_display() {
        let today_str = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
        let formatted = format_date_display(&today_str);
        assert!(formatted.contains("(Hoje)"));
    }

    #[test]
    fn test_format_date_display_and_short_with_time() {
        let today_str = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
        let pt = crate::core::i18n::Language::Portuguese;
        let with_time = format_date_display_with_time(&today_str, "19:00", pt);
        assert!(with_time.contains("às 19:00"));
        assert!(with_time.contains("(Hoje)"));

        let short_with_time = format_date_short_with_time(&today_str, "19:00", pt);
        assert!(short_with_time.contains("· 19:00"));
        assert!(short_with_time.contains("(Hoje)"));

        let empty_time = format_date_short_with_time(&today_str, "", pt);
        assert!(!empty_time.contains("·"));
    }

    #[test]
    fn test_multiple_events_on_same_day() {
        let mut state = ControlPanelState::default();
        // Evento 1: Manhã (Domingo 09:00)
        state.settings.services[0].name = "Culto Matutino".into();
        state.settings.services[0].recurrence = EventRecurrence {
            enabled: true,
            pattern: "weekly".into(),
            days_of_week: vec![0], // Domingo
            time: "09:00".into(),
        };

        // Evento 2: Noite (Domingo 19:00)
        state.event_editor_draft = Some(EventEditorDraft {
            id: None,
            name: "Culto Noturno".into(),
            pattern: "weekly".into(),
            days_of_week: vec![0], // Domingo também
            time: "19:00".into(),
            date: String::new(),
            copy_current_playlist: false,
            ..Default::default()
        });
        state.save_event_from_draft();

        assert_eq!(state.settings.services.len(), 2);

        // Ambas ocorrem em qualquer domingo (ex: 2026-11-01 é domingo)
        assert!(state.settings.services[0].occurs_on_date("2026-11-01"));
        assert!(state.settings.services[1].occurs_on_date("2026-11-01"));

        // Não ocorrem numa quarta-feira (ex: 2026-11-04)
        assert!(!state.settings.services[0].occurs_on_date("2026-11-04"));
        assert!(!state.settings.services[1].occurs_on_date("2026-11-04"));

        // O Culto Noturno está ativo
        let noturno_id = state.settings.active_service.clone();
        assert_eq!(state.settings.service().name, "Culto Noturno");

        // Adicionar música ao culto noturno
        state.playlist.push(PlaylistItem {
            id: "m_noite".into(),
            title: "Música Noite".into(),
            subtitle: "".into(),
            kind: PlaylistItemKind::Song,
            played: false,
            content: "".into(),
        });
        state.capture_profiles();

        // Mudar para culto matutino
        let matutino_id = state.settings.services[0].id.clone();
        state.switch_service(&matutino_id);
        assert_eq!(state.settings.service().name, "Culto Matutino");
        assert!(state.playlist.is_empty());

        // Adicionar música ao culto matutino
        state.playlist.push(PlaylistItem {
            id: "m_manha".into(),
            title: "Música Manhã".into(),
            subtitle: "".into(),
            kind: PlaylistItemKind::Song,
            played: false,
            content: "".into(),
        });
        state.capture_profiles();

        // Alternar de volta para o culto noturno
        state.switch_service(&noturno_id);
        assert_eq!(state.playlist.len(), 1);
        assert_eq!(state.playlist[0].title, "Música Noite");
    }

    #[test]
    fn test_event_default_theme_and_wallpaper_switching() {
        let mut state = ControlPanelState::default();
        let custom_theme = crate::core::models::PresentationTheme {
            id: "theme_youth".into(),
            name: "Tema Jovens".into(),
            ..Default::default()
        };
        state.themes.push(custom_theme);

        // Criar evento com tema e wallpaper padrão
        state.event_editor_draft = Some(EventEditorDraft {
            id: None,
            name: "Culto de Jovens".into(),
            theme_id: "theme_youth".into(),
            idle_theme_id: "obsidian_pro".into(),
            ..Default::default()
        });
        state.save_event_from_draft();

        assert_eq!(state.settings.service().name, "Culto de Jovens");
        assert_eq!(state.settings.service().theme_id, "theme_youth");
        assert_eq!(state.settings.service().idle_theme_id, "obsidian_pro");
        assert_eq!(state.live_theme_id, "theme_youth");
    }
}
