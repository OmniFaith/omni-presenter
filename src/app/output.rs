use crate::ui::AppTheme;
use crate::ui::presentation::{PresentationAction, PresentationContent, PresentationView};
use eframe::egui;

use super::PresentationApp;

impl PresentationApp {
    pub(super) fn restart_browser_server(&mut self) {
        self.web_control
            .set_auth(self.control_state.chat.service.clone());
        self.browser_server = None;
        self.control_state.settings_ui.server_running = false;
        self.control_state.settings_ui.discovery_url.clear();
        self.control_state.settings_ui.discovery_chat_url.clear();
        self.control_state.settings_ui.discovery_error.clear();
        if self.control_state.settings.server_enabled || self.control_state.settings.chat_enabled {
            let chat = if self.control_state.settings.chat_enabled {
                self.control_state.chat.service.clone()
            } else {
                None
            };
            match crate::services::web::BrowserServer::start_with_control(
                &self.control_state.settings,
                chat,
                Some(self.web_control.clone()),
            ) {
                Ok(server) => {
                    self.control_state.chat.https_error = server.chat_https_error.clone();
                    self.control_state.settings_ui.discovery_url =
                        server.discovery_url.clone().unwrap_or_default();
                    self.control_state.settings_ui.discovery_chat_url =
                        server.discovery_chat_url.clone().unwrap_or_default();
                    self.control_state.settings_ui.discovery_error =
                        server.discovery_error.clone().unwrap_or_default();
                    self.browser_server = Some(server);
                    self.control_state.settings_ui.server_running =
                        self.control_state.settings.server_enabled;
                    self.control_state.settings_ui.server_status = self
                        .control_state
                        .settings
                        .language
                        .format(
                            "services-server-running-port",
                            &[("port", &self.control_state.settings.server_port.to_string())],
                        );
                }
                Err(error) => {
                    self.control_state.chat.error = error.clone();
                    self.control_state.settings_ui.server_status = error;
                }
            }
        } else {
            self.control_state.settings_ui.server_status = self
                .control_state
                .settings
                .language
                .text("services-server-disabled")
                .into();
        }
    }

    /// Moves the live presentation viewport to the newly selected monitor.
    pub(super) fn move_presentation_to_monitor(&mut self, ctx: &egui::Context, new_index: usize) {
        self.selected_monitor_index = new_index;
        self.update_presentation_aspect_ratio();

        // Each physical viewport is keyed by monitor identity and owns its
        // geometry. Changing the reference must not move another live output.
        ctx.request_repaint();
    }

    /// Pre-warms slide rendering and theme textures so transitions are instant.
    pub(super) fn prewarm(&mut self, ctx: &egui::Context) {
        if self.slide_deck.live_design.is_some() {
            return;
        }
        let live_theme = crate::services::media::live_theme(&self.control_state, &self.slide_deck);
        let prewarm_key = format!(
            "{}:{}:{}",
            self.slide_deck.live_title(),
            live_theme.id,
            self.is_presentation_open
        );
        if self.last_prewarmed_key == prewarm_key {
            return;
        }
        self.last_prewarmed_key = prewarm_key;
        if live_theme.bg_type == "image" && !live_theme.bg_image_path.is_empty() {
            crate::render::get_or_load_texture(ctx, &live_theme.bg_image_path);
        }
        let target_font_size = live_theme.font_size;
        crate::render::prewarm_slides_in_egui(
            ctx,
            &live_theme,
            self.slide_deck.live_slides(),
            target_font_size,
        );
    }

    /// Renders the physical presentation viewport when a presentation is live.
    pub(super) fn draw_presentation_viewport(&mut self, ctx: &egui::Context) {
        let settings = self.control_state.settings.clone();
        let live_theme = crate::services::media::live_theme(&self.control_state, &self.slide_deck);
        let transparent = matches!(
            settings.physical_background,
            crate::core::preferences::PhysicalBackground::Transparent
        ) && settings
            .monitor_backgrounds
            .values()
            .all(|bg| matches!(bg, crate::core::preferences::PhysicalBackground::Transparent));
        let active = self.is_presentation_open
            && settings.physical_output_enabled
            && (!transparent
                || self.physical_transition.visible()
                || crate::ui::presentation_assets::ready(
                    ctx,
                    &live_theme,
                    self.slide_deck.live_design.as_ref(),
                    self.slide_deck.live_overlay.as_ref(),
                    &self.control_state.themes,
                ));
        let duration = if settings.operator().reduce_motion {
            0
        } else {
            settings.physical_transition_ms
        };
        let opacity =
            self.physical_transition
                .update(active, ctx.input(|input| input.time), duration);
        if self.physical_transition.moving() {
            ctx.request_repaint();
        }
        if !settings.physical_output_enabled
            || !(active || opacity > 0.0 || settings.persistent_physical_output)
        {
            return;
        }
        let targets: Vec<_> = self
            .monitors
            .iter()
            .filter(|m| settings.physical_monitor_enabled(&m.name))
            .cloned()
            .collect();

        let is_hidden = self.slide_deck.is_content_hidden();
        let projected_text = self.slide_deck.projected_text();
        let clean_text = crate::core::slides::strip_comments(projected_text);

        let design = self.slide_deck.live_design.clone();
        let overlay = self.slide_deck.live_overlay.clone();
        let is_song = self.slide_deck.is_live_song();
        let live_title = self.slide_deck.live_title().to_owned();
        let live_artist = self.slide_deck.live_artist().to_owned();
        let live_index = self.slide_deck.live_index();
        for target_monitor in &targets {
            let monitor_bg = settings.wallpaper_for_monitor(&target_monitor.name);
            let wallpaper_design = match monitor_bg {
                crate::core::preferences::PhysicalBackground::Design { id } => self
                    .slide_deck
                    .designs
                    .iter()
                    .find(|d| d.id == *id)
                    .cloned(),
                _ => None,
            };
            let aspect = if settings.service().aspect_ratio > 0.0 {
                settings.service().aspect_ratio
            } else {
                target_monitor.resolution.0 as f32 / target_monitor.resolution.1.max(1) as f32
            };
            let mut nav_action = false;
            PresentationView::show(
                ctx,
                PresentationContent {
                    text: &clean_text,
                    hidden: is_hidden,
                    theme: &live_theme,
                    design: design.as_ref(),
                    overlay: overlay.as_ref(),
                    themes: &self.control_state.themes,
                    aspect,
                    active,
                    opacity,
                    wallpaper_design: wallpaper_design.as_ref(),
                    title: &live_title,
                    artist: &live_artist,
                    slide_index: live_index,
                    is_song,
                },
                Some(target_monitor),
                &self.control_state.settings,
                !self.control_state.settings.operator().reduce_motion,
                &mut self.is_presentation_open,
                |action| match action {
                    PresentationAction::ToggleHide => {
                        self.slide_deck.toggle_hide_content();
                        true
                    }
                    PresentationAction::NextSlide => {
                        nav_action = true;
                        self.slide_deck.next_slide()
                    }
                    PresentationAction::PrevSlide => {
                        nav_action = true;
                        self.slide_deck.prev_slide();
                        true
                    }
                    PresentationAction::FirstSlide => {
                        nav_action = true;
                        self.slide_deck.first_slide();
                        true
                    }
                    PresentationAction::LastSlide => {
                        nav_action = true;
                        self.slide_deck.last_slide();
                        true
                    }
                    PresentationAction::Search => {
                        self.global_search.open = false;
                        self.global_search.toggle();
                        ctx.send_viewport_cmd_to(
                            egui::ViewportId::ROOT,
                            egui::ViewportCommand::Focus,
                        );
                        true
                    }
                },
            );
            if nav_action {
                crate::services::media::sync_live_media_playback(
                    &mut self.control_state,
                    &self.slide_deck,
                    self.is_presentation_open,
                );
            }
        }
    }

    /// Publishes the current state to the web viewer, if the server is running.
    pub(super) fn publish_browser(&mut self, ctx: &egui::Context) {
        if let Some(server) = &mut self.browser_server {
            server.refresh_discovery();
            self.control_state.settings_ui.discovery_url =
                server.discovery_url.clone().unwrap_or_default();
            self.control_state.settings_ui.discovery_chat_url =
                server.discovery_chat_url.clone().unwrap_or_default();
            self.control_state.settings_ui.discovery_error =
                server.discovery_error.clone().unwrap_or_default();
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
        if !self.control_state.settings.server_enabled {
            return;
        }
        let Some(server) = &mut self.browser_server else {
            return;
        };
        let output_theme = if self.is_presentation_open {
            crate::services::media::live_theme(&self.control_state, &self.slide_deck)
        } else {
            self.control_state
                .themes
                .iter()
                .find(|t| t.id == self.control_state.settings.service().idle_theme_id)
                .unwrap_or(self.control_state.live_theme()).clone()
        };
        server.publish(
            &self.control_state.settings,
            &self.slide_deck,
            &output_theme,
            self.is_presentation_open,
            &self.control_state.themes,
        );
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }

    /// Floating bottom-center toast notification.
    pub(super) fn draw_toast(&mut self, ctx: &egui::Context, theme: &AppTheme) {
        let Some((message, since)) = self.control_state.toast.clone() else {
            return;
        };
        if since.elapsed().as_secs_f32() >= 3.0 {
            self.control_state.toast = None;
            return;
        }
        egui::Area::new(egui::Id::unique("app_toast"))
            .order(egui::Order::Tooltip)
            .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -24.0])
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(theme.bg_card)
                    .stroke(egui::Stroke::new(1.0, theme.border_default))
                    .corner_radius(10.0)
                    .inner_margin(14)
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 4],
                        blur: 20,
                        spread: 0,
                        color: egui::Color32::from_black_alpha(24),
                    })
                    .show(ui, |ui| {
                            ui.label(egui::RichText::new(self.control_state.settings.language.message(&message)).color(theme.text_primary));
                    });
            });
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
