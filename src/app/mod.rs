use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use eframe::egui;

mod chat;
mod docking;
mod frame;
mod input;
mod modals;
mod persistence;
mod plugins;
mod search;
mod remote;
mod remote_snapshot;
mod remote_tools;
mod output;
#[cfg(test)]
mod output_tests;
#[cfg(all(test, target_os = "linux"))]
mod native_output_tests;

#[cfg(test)]
mod remote_tests;
#[cfg(test)]
mod tests;

pub struct PresentationApp {
    pub(super) web_control: crate::services::control::WebControl,
    pub(super) last_control_publish: Option<std::time::Instant>,
    pub(super) global_search: crate::ui::components::global_search::GlobalSearch,
    pub(super) design_editor: crate::ui::pages::design::DesignEditor,
    pub(super) db: AppDb,
    pub(super) slide_deck: SlideDeck,
    pub(super) control_state: ControlPanelState,
    pub(super) is_presentation_open: bool,
    pub(super) is_preview_modal_open: bool,
    pub(super) selected_monitor_index: usize,
    pub(super) monitors: Vec<crate::services::monitor::MonitorInfo>,
    pub(super) is_initialized: bool,
    pub(super) was_presentation_open: bool,
    pub(super) last_prewarmed_key: String,
    pub(super) browser_server: Option<crate::services::web::BrowserServer>,
    pub(super) last_saved_settings: String,
    pub(super) settings_changed_at: Option<std::time::Instant>,
    pub(super) pending_settings_json: String,
    pub(super) last_monitor_sync: std::time::Instant,
    pub(super) applied_appearance: Option<(bool, u32, bool)>,
    pub(super) physical_transition: crate::ui::presentation_transition::PresentationTransition,
}

impl Default for PresentationApp {
    fn default() -> Self {
        // Pré-carrega o catálogo de fontes do sistema em segundo plano para não travar a UI
        std::thread::spawn(|| {
            let _ = crate::core::models::get_system_fonts();
        });

        let db_path = "omni-presenter.db";
        let db = AppDb::open(db_path).unwrap_or_else(|_| AppDb::open_in_memory().unwrap());
        Self::with_db(db)
    }
}

impl PresentationApp {
    pub(super) fn with_db(db: AppDb) -> Self {
        if let Err(error) = db.install_dynamic_themes() {
            eprintln!("Não foi possível adicionar os fundos dinâmicos: {error}");
        }
        if let Err(error) = db.upgrade_default_presentation_themes() {
            eprintln!("Não foi possível atualizar os temas de apresentação: {error}");
        }
        let slide_deck = SlideDeck::from_db(&db);
        let mut control_state = ControlPanelState::default();
        match db.get_bible_translations() {
            Ok(translations) => {
                for translation in translations {
                    control_state.bible.library.insert(translation);
                }
            }
            Err(error) => control_state.notify(control_state.settings.language.format(
                "services-could-not-load-bible-translations",
                &[("error", &error.to_string())],
            )),
        }

        control_state.reload_all_selection_items(&db);

        if let Ok(db_themes) = db.get_all_themes()
            && !db_themes.is_empty() {
                control_state.themes = db_themes;
                control_state.selected_theme_index = 0;
                control_state.live_theme_id = control_state.themes[0].id.clone();
                control_state.live_theme_index = 0;
            }

        match db.load_settings() {
            Ok(Some(mut settings)) => {
                settings.normalize();
                control_state.settings = settings;
                crate::services::plugins::sync_shader_themes(&mut control_state);
                control_state.apply_profiles();
                control_state.apply_theme_to_live();
            }
            Ok(None) => {
                control_state.apply_profiles();
                control_state.capture_profiles();
            }
            Err(error) => {
                control_state.apply_profiles();
                control_state.capture_profiles();
                control_state.notify(control_state.settings.language.format(
                    "services-could-not-load-preferences",
                    &[("error", &error.to_string())],
                ));
            }
        }
        crate::services::plugins::sync_shader_themes(&mut control_state);
        let last_saved_settings =
            serde_json::to_string(&control_state.settings).unwrap_or_default();
        match crate::services::chat::ChatService::new(db.clone(), &control_state.settings.operators)
        {
            Ok(chat) => control_state.chat.service = Some(chat),
            Err(error) => control_state.chat.error = error,
        }
        let mut app = Self {
            web_control: crate::services::control::WebControl::new(db.clone()),
            last_control_publish: None,
            global_search: Default::default(),
            design_editor: Default::default(),
            db,
            slide_deck,
            control_state,
            is_presentation_open: false,
            is_preview_modal_open: false,
            selected_monitor_index: 0,
            monitors: Vec::new(),
            is_initialized: false,
            was_presentation_open: false,
            last_prewarmed_key: String::new(),
            browser_server: None,
            last_saved_settings,
            pending_settings_json: String::new(),
            settings_changed_at: None,
            last_monitor_sync: std::time::Instant::now(),
            applied_appearance: None,
            physical_transition: Default::default(),
        };
        app.restart_browser_server();
        app
    }
}

impl eframe::App for PresentationApp {
    fn on_exit(&mut self) {
        self.design_editor
            .flush(&self.db, &mut self.slide_deck, true);
        self.control_state.capture_profiles();
        if let Err(error) = self.db.save_settings(&self.control_state.settings) {
            eprintln!("Falha ao salvar preferências: {error}");
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Secondary physical outputs need a genuinely transparent framebuffer.
        [0.0; 4]
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let (ctx, modal_was_open) = self.begin_frame(ui, frame);
        self.handle_search_shortcut(&ctx);
        self.sync_chat(&ctx);
        self.sync_web_control(&ctx);
        self.run_plugin_request();
        let theme = self.control_state.theme.clone();

        // Chrome (left navigation, top bar, presentation dock)
        self.draw_left_sidebar(ui, &theme);
        self.draw_top_bar(ui, &theme);
        self.draw_presentation_dock(ui, &theme);

        // Central workspace (page-dependent)
        ui.add_enabled_ui(!self.global_search.open, |ui| {
            self.draw_workspace(ui, &theme)
        });
        self.design_editor.flush(
            &self.db,
            &mut self.slide_deck,
            self.control_state.active_sidebar_page != crate::core::state::SidebarPage::Design,
        );

        // Modal dialogs
        let mut is_text_focused = false;
        let monitor_changed_to = if self.global_search.open {
            None
        } else {
            self.draw_modals(&ctx, &mut is_text_focused)
        };

        if let Some(new_index) = monitor_changed_to {
            self.move_presentation_to_monitor(&ctx, new_index);
        }

        // Global keyboard handling
        self.handle_global_input(&ctx, is_text_focused, modal_was_open);
        crate::ui::containers::selection::poll_media_import(&ctx, &mut self.control_state, Some(&self.db));
        self.control_state.refresh_media_duration();
        self.draw_global_search(&ctx);

        // Outputs: prewarm caches, physical projection viewport and web viewer
        self.prewarm(&ctx);
        self.draw_presentation_viewport(&ctx);
        self.publish_browser(&ctx);
        self.publish_web_control();

        self.autosave_profiles(&ctx);
        self.draw_toast(&ctx, &theme);
        crate::ui::pages::chat::render_chat_notice(&ctx, &mut self.control_state);
        crate::ui::components::drag_drop::paint_preview(&ctx, &theme);
    }
}
