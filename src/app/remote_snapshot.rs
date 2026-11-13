use super::PresentationApp;
use serde_json::{Value, json};
use std::hash::{Hash, Hasher};

fn color(value: eframe::egui::Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", value.r(), value.g(), value.b())
}
fn theme(value: &crate::core::models::PresentationTheme, language: crate::core::i18n::Language) -> Value {
    let name = value.display_name(language);
    let (top, bottom, colors) = if value.bg_type == "shader" && !value.options.shader.colors.is_empty() {
        let hex = |c: [u8; 3]| format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2]);
        let hex_colors: Vec<String> = value.options.shader.colors.iter().copied().map(hex).collect();
        let top_color = hex_colors.first().cloned().unwrap_or_else(|| color(value.bg_top));
        let bottom_color = hex_colors.last().cloned().unwrap_or_else(|| color(value.bg_bottom));
        (top_color, bottom_color, hex_colors)
    } else {
        (color(value.bg_top), color(value.bg_bottom), Vec::new())
    };
    let shader_kind = value
        .id
        .strip_prefix("shader_")
        .or_else(|| {
            if value.bg_type == "shader" && !value.options.shader.kind.is_empty() {
                Some(value.options.shader.kind.as_str())
            } else {
                None
            }
        })
        .or_else(|| match value.name.as_str() {
            "Aurora · Dinâmico" | "Aurora · Animated" | "Aurora · Animado" => Some("aurora"),
            "Nebulosa · Dinâmico" | "Nebula · Animated" | "Nebulosa · Animado" => Some("nebula"),
            "Ondas · Dinâmico" | "Waves · Animated" | "Olas · Animado" => Some("waves"),
            "Seda · Dinâmico" | "Silk · Animated" | "Seda · Animado" => Some("silk"),
            "Partículas · Dinâmico" | "Particles · Animated" | "Partículas · Animado" => Some("particles"),
            "Anéis · Dinâmico" | "Rings · Animated" | "Anillos · Animado" => Some("rings"),
            "Fluido · Dinâmico" | "Fluid · Animated" | "Fluido · Animado" => Some("fluid"),
            "Grade de Luz · Dinâmico" | "Grade de luz · Dinâmico" | "Light grid · Animated" | "Light Grid · Animated" | "Cuadrícula de luz · Animado" => Some("grid"),
            "Bokeh · Dinâmico" | "Bokeh · Animated" | "Bokeh · Animado" => Some("bokeh"),
            "Prisma · Dinâmico" | "Prism · Animated" | "Prisma · Animado" => Some("prism"),
            _ => None,
        })
        .unwrap_or("");
    let shader_index = crate::services::shaders::kind_index(shader_kind);
    let shader_options = if value.bg_type == "shader" {
        json!({
            "kind": shader_kind,
            "index": shader_index,
            "colors": value.options.shader.colors,
            "speed": value.options.shader.speed,
            "scale": value.options.shader.scale,
            "intensity": value.options.shader.intensity,
            "softness": value.options.shader.softness,
            "detail": value.options.shader.detail,
            "direction": value.options.shader.direction,
            "seed": value.options.shader.seed,
            "vignette": value.options.shader.vignette,
            "grain": value.options.shader.grain,
        })
    } else {
        json!(null)
    };
    json!({
        "id": value.id,
        "name": name,
        "top": top,
        "bottom": bottom,
        "colors": colors,
        "color": color(value.text_color),
        "font": value.font_family,
        "weight": value.font_weight,
        "align": value.align_h,
        "case": value.text_case,
        "image": value.bg_type == "image",
        "shader": value.bg_type == "shader",
        "shader_kind": shader_kind,
        "shader_index": shader_index,
        "shader_options": shader_options,
        "bg_type": value.bg_type
    })
}

impl PresentationApp {
    pub(super) fn publish_web_control(&mut self) {
        if self
            .last_control_publish
            .is_some_and(|last| last.elapsed().as_millis() < 100)
        {
            return;
        }
        self.last_control_publish = Some(std::time::Instant::now());
        let state = &self.control_state;
        let deck = &self.slide_deck;
        let preview_theme = crate::services::media::preview_theme(state, deck);
        let live_theme = crate::services::media::live_theme(state, deck);
        let idle = state
            .themes
            .iter()
            .find(|theme| theme.id == state.settings.service().idle_theme_id)
            .unwrap_or(state.live_theme());
        self.web_control.publish_outputs(
            deck,
            if deck
                .preview_design
                .as_ref()
                .is_some_and(|scene| scene.overlay)
            {
                &live_theme
            } else {
                &preview_theme
            },
            if self.is_presentation_open {
                &live_theme
            } else {
                idle
            },
            &state.themes,
            state.presentation_aspect_ratio,
            self.is_presentation_open,
        );
        self.web_control.set_bible(state.bible.library.clone());
        let mut catalog_hash = std::collections::hash_map::DefaultHasher::new();
        for song in deck.library() {
            (song.id, &song.title, &song.artist, &song.text, &song.folder, &song.tags).hash(&mut catalog_hash);
        }
        for item in &state.custom_texts {
            (item.id, &item.title, &item.content, &item.folder).hash(&mut catalog_hash);
        }
        for item in &state.media_items {
            (item.id, &item.title, &item.file_type, &item.folder).hash(&mut catalog_hash);
        }
        for item in &state.saved_verses {
            (item.id, &item.reference, &item.text, &item.folder).hash(&mut catalog_hash);
        }
        for scene in &deck.designs {
            (&scene.id, &scene.name, &scene.folder).hash(&mut catalog_hash);
        }
        self.web_control.set_backgrounds(
            state
                .themes
                .iter()
                .filter(|theme| theme.bg_type == "image")
                .map(|theme| (theme.id.clone(), theme.bg_image_path.clone()))
                .chain([&preview_theme, &live_theme].into_iter().filter(|theme| theme.bg_type == "image")
                    .map(|theme| (theme.id.clone(), theme.bg_image_path.clone()))),
        );
        let slides = |items: &[String]| {
            items
                .iter()
                .map(|text| crate::core::markup::strip(&crate::core::slides::strip_comments(text)))
                .collect::<Vec<_>>()
        };
        let design = deck
            .live_overlay
            .as_ref()
            .or(deck.live_design.as_ref())
            .or(deck.preview_design.as_ref());
        let timers: Vec<_> = design.into_iter().flat_map(|scene| &scene.layers).filter_map(|layer| match &layer.content {
            crate::core::design::LayerContent::Countdown(timer) => Some(json!({"name":layer.name,"text":timer.text(crate::core::design::now_ms()),"running":timer.started_at.is_some()})), _ => None,
        }).collect();
        let player = state.active_player_media.as_ref().map(|media| {
            let (playing, position) = if media.kind == crate::core::state::PlayerMediaKind::Audio { (state.audio_player.is_playing(), state.audio_player.position_secs()) }
            else { (media.is_playing, state.player_position()) };
            json!({"title":media.title,"playing":playing,"position":position.min(media.duration_secs.max(0.0)),"duration":media.duration_secs,"volume":state.audio_player.volume()})
        });
        let active_service_date = state.current_service_date();
        let active_svc = state.settings.service();
        let time_str = active_svc.recurrence.time.trim();
        let mut service_dates = Vec::new();
        for s in &active_svc.sessions {
            if s.date != active_service_date {
                service_dates.push(json!({
                    "date": s.date,
                    "label": crate::core::state::events::format_date_short_with_time(&s.date, time_str, state.settings.language),
                    "items": s.playlist.len(),
                    "active": false,
                }));
            }
        }
        let active_count = active_svc.get_session(&active_service_date).map(|s| s.playlist.len()).unwrap_or(state.playlist.len());
        service_dates.push(json!({
            "date": active_service_date,
            "label": crate::core::state::events::format_date_short_with_time(&active_service_date, time_str, state.settings.language),
            "items": active_count,
            "active": true,
        }));
        let mut cur_d = active_service_date.clone();
        for _ in 0..4 {
            cur_d = active_svc.step_date(&cur_d, 1);
            let cnt = active_svc.get_session(&cur_d).map(|s| s.playlist.len()).unwrap_or(0);
            service_dates.push(json!({
                "date": cur_d,
                "label": crate::core::state::events::format_date_short_with_time(&cur_d, time_str, state.settings.language),
                "items": cnt,
                "active": false,
            }));
        }
        service_dates.sort_by(|a, b| a["date"].as_str().cmp(&b["date"].as_str()));
        service_dates.dedup_by(|a, b| a["date"] == b["date"]);

        self.web_control.publish(json!({
            "ready":true,"presenting":self.is_presentation_open,"hidden":deck.is_content_hidden(),"dark":state.settings.operator().dark_mode,"language":state.settings.language,
            "catalog_revision":catalog_hash.finish().to_string(),"bible_version":state.bible_version,
            "bible_versions":state.bible.library.catalog().into_iter().map(|(id,name)| json!({"language":crate::core::bible::bible_language(&id),"id":id,"name":name})).collect::<Vec<_>>(),
            "bible_shortcuts":state.bible_shortcuts,"bible_merge":state.bible.merge_selection,"bible_cards":state.bible.cards_layout,
            "layout":state.layout,"bible_layout":state.bible_layout,"scale":state.settings.operator().scale,"lyrics_cards":state.lyrics_cards_layout,"show_dock":state.show_presentation_dock,"dock_background":state.dock_preview_bg,"progress":deck.progress(),"aspect":state.presentation_aspect_ratio,
            "idle_theme":theme(state.themes.iter().find(|theme| theme.id==state.settings.service().idle_theme_id).unwrap_or(state.live_theme()), state.settings.language),
            "bible_context":{"book":state.bible_book,"chapter":state.bible_chapter,"start":state.bible_verse,"end":state.bible.verse_end},
            "operator":state.settings.operator().name,"chat_enabled":state.settings.chat_enabled,"service":state.settings.active_service,"service_date":active_service_date,"service_dates":service_dates,"services":state.settings.services.iter().map(|service| json!({"id":service.id,"name":service.display_name(state.settings.language)})).collect::<Vec<_>>(),
            "preview":{"title":deck.title(),"subtitle":deck.artist(),"slides":slides(deck.slides()),"index":deck.preview_slide_index(),"raw":deck.raw_text(),"song_id":deck.current_song().filter(|song| song.title == deck.title() && song.artist == deck.artist()).and_then(|song| song.id),"design":deck.preview_design.as_ref().map(|scene| &scene.name),"theme":theme(&preview_theme, state.settings.language)},
            "live":{"title":deck.live_title(),"subtitle":deck.live_artist(),"slides":slides(deck.live_slides()),"index":deck.live_index(),"design":deck.live_design.as_ref().map(|scene| &scene.name),"theme":theme(&live_theme, state.settings.language)},
            "playlist":state.playlist.iter().enumerate().map(|(index,item)| json!({"id":item.id,"title":item.title,"subtitle":item.subtitle,"kind":item.kind,"played":item.played,"selected":state.playlist_selection.selected.contains(&item.id),"focused":Some(index)==state.selected_playlist_index})).collect::<Vec<_>>(),
            "quick":state.quick_bar.items.iter().map(|item|json!({"id":item.id,"title":item.title,"kind":item.kind})).collect::<Vec<_>>(),
            "themes":state.themes.iter().map(|t| theme(t, state.settings.language)).collect::<Vec<_>>(),"player":player,"timers":timers,"questions":design.is_some_and(|scene| scene.layers.iter().any(|layer| matches!(layer.content, crate::core::design::LayerContent::Questions{..}))),
            "overlay":deck.live_overlay.as_ref().map(|scene| &scene.name),"screens":state.settings.screens.iter().filter(|screen| screen.enabled).map(|screen|json!({"id":screen.id,"name":screen.name,"slug":crate::core::preferences::screen_url_slug(screen, &state.settings.screens)})).collect::<Vec<_>>()
        }));
    }
}
