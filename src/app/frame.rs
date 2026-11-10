use crate::core::layout::ContainerKind;
use crate::core::state::{PlayerMediaKind, SidebarPage};
use crate::services::monitor::{find_recommended_presentation_monitor, get_unique_monitors};
use crate::ui::AppTheme;
use eframe::egui;

use super::PresentationApp;

impl PresentationApp {
    /// Applies appearance/style, syncs monitors and returns the frame context
    /// together with whether a modal was open at the start of the frame.
    pub(super) fn begin_frame(
        &mut self,
        ui: &mut egui::Ui,
        frame: &mut eframe::Frame,
    ) -> (egui::Context, bool) {
        ui.style_mut().interaction.selectable_labels = false;

        let ctx = ui.ctx().clone();
        crate::ui::i18n::install(&ctx, self.control_state.settings.language);
        self.apply_appearance(&ctx);
        let dark = if self.control_state.settings_ui.open {
            self.control_state
                .settings_ui
                .draft
                .as_ref()
                .map(|d| {
                    let op = d.operator();
                    op.theme_mode.is_dark(ctx.system_theme() == Some(egui::Theme::Dark), op.dark_mode)
                })
                .unwrap_or(self.control_state.settings.operator().dark_mode)
        } else {
            self.control_state.settings.operator().dark_mode
        };
        *ui.style_mut() = (*ctx.style_of(if dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        }))
        .clone();
        self.sync_monitors(frame);

        // When presentation starts, automatically ensure presentation dock is shown and track start time
        if self.is_presentation_open && !self.was_presentation_open {
            self.control_state.show_presentation_dock =
                self.control_state.settings.operator().show_dock;
            self.control_state.presentation_started_at = Some(std::time::Instant::now());
        } else if !self.is_presentation_open && self.was_presentation_open {
            self.control_state.presentation_started_at = None;
        }
        self.was_presentation_open = self.is_presentation_open;
        self.update_presentation_aspect_ratio();

        // If a video was projecting, but presentation was closed or another slide/item was projected,
        // synchronize the player state and stop video playback so it doesn't continue running in the background.
        if let Some(media) = &mut self.control_state.active_player_media {
            if media.kind == PlayerMediaKind::Video && media.is_projecting {
                let is_still_projecting = self.is_presentation_open
                    && self.slide_deck.live_media.as_ref().map(|m| &m.path) == Some(&media.path);
                if !is_still_projecting {
                    media.is_projecting = false;
                    if media.is_playing {
                        media.is_playing = false;
                        media.video_paused_at = Some(media.video_position());
                        crate::services::video::stop(&ctx, &media.path);
                    }
                    self.control_state.audio_player.pause();
                }
            }
        }

        let modal_was_open = self.control_state.settings_ui.open
            || self.control_state.media_conversion_prompt.is_some()
            || self.control_state.song_editor.open
            || self.control_state.theme_editor.open
            || self.control_state.strophe_editor.open
            || self.control_state.show_create_folder_modal
            || self.control_state.show_presentation_settings_modal
            || self.is_preview_modal_open
            || self.control_state.bible.show_import
            || self.control_state.chat.show_channels
            || self.control_state.chat.show_bible
            || self.control_state.chat.show_songs
            || self.control_state.chat.show_access
            || self.control_state.confirm_dialog.is_some();
        (ctx, modal_was_open)
    }

    pub(super) fn apply_appearance(&mut self, ctx: &egui::Context) {
        ctx.options_mut(|options| options.zoom_with_keyboard = false);
        let saved_operator = self.control_state.settings.operator().clone();

        // Theme and appearance mode are previewed from draft if settings modal is open;
        // scale and reduce_motion remain strictly from saved settings so adjusting the slider doesn't resize the UI live.
        let (resolved, dark) = if self.control_state.settings_ui.open {
            let draft = self
                .control_state
                .settings_ui
                .draft
                .as_ref()
                .unwrap_or(&self.control_state.settings);
            AppTheme::resolve(draft, ctx)
        } else {
            let (theme, dark) = AppTheme::resolve(&self.control_state.settings, ctx);
            self.control_state.settings.operator_mut().dark_mode = dark;
            (theme, dark)
        };

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
            self.control_state
                .settings
                .language
                .text("app-title")
                .into(),
        ));
        ctx.set_zoom_factor(saved_operator.scale as f32 / 100.0);
        let appearance = (dark, saved_operator.scale, saved_operator.reduce_motion);
        let theme_mode = if self.control_state.settings_ui.open {
            self.control_state
                .settings_ui
                .draft
                .as_ref()
                .map(|d| d.operator().theme_mode)
                .unwrap_or(saved_operator.theme_mode)
        } else {
            saved_operator.theme_mode
        };
        ctx.set_theme(if theme_mode == crate::core::interface_theme::ThemeMode::System {
            egui::ThemePreference::System
        } else if dark {
            egui::ThemePreference::Dark
        } else {
            egui::ThemePreference::Light
        });
        if self.applied_appearance == Some(appearance) && self.control_state.theme == resolved {
            return;
        }
        self.applied_appearance = Some(appearance);
        self.control_state.theme = resolved;
        let mut visuals = if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        self.control_state.theme.apply_to_visuals(&mut visuals);
        let selected_theme = if dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        };
        ctx.set_visuals(visuals);
        ctx.set_zoom_factor(saved_operator.scale as f32 / 100.0);
        ctx.style_mut_of(selected_theme, |style| {
            style.animation_time = if saved_operator.reduce_motion { 0.0 } else { 0.14 };
            style.interaction.selectable_labels = false;
            style.spacing.button_padding = egui::vec2(10.0, 6.0);
            style.spacing.item_spacing = egui::vec2(8.0, 7.0);
        });
    }

    pub(super) fn update_presentation_aspect_ratio(&mut self) {
        let configured = self.control_state.settings.service().aspect_ratio;
        if configured > 0.0 {
            self.control_state.presentation_aspect_ratio = configured;
            return;
        }
        let aspect = self
            .monitors
            .iter()
            .find(|m| m.winit_index == self.selected_monitor_index)
            .or_else(|| self.monitors.get(self.selected_monitor_index))
            .or_else(|| self.monitors.first())
            .map(|m| {
                if m.resolution.0 > 0 && m.resolution.1 > 0 {
                    m.resolution.0 as f32 / m.resolution.1 as f32
                } else {
                    16.0 / 9.0
                }
            })
            .unwrap_or(16.0 / 9.0);

        self.control_state.presentation_aspect_ratio = aspect;
    }

    pub(super) fn sync_monitors(&mut self, frame: &mut eframe::Frame) {
        // Querying display monitors via winit/X11 IPC is heavy and blocks the UI thread.
        // Only query on first startup or when the user opens the Settings modal.
        if !self.is_initialized
            || self.last_monitor_sync.elapsed().as_secs() >= 2
        {
            self.last_monitor_sync = std::time::Instant::now();
            if let Some(winit_window) = frame.winit_window() {
                let unique_monitors = get_unique_monitors(winit_window);

                self.monitors = unique_monitors;

                if !self.is_initialized && !self.monitors.is_empty() {
                    self.is_initialized = true;
                    let current_name = winit_window
                        .current_monitor()
                        .and_then(|m| m.name())
                        .or_else(|| {
                            crate::services::monitor::detect_primary_monitor_from_window(
                                winit_window,
                            )
                        })
                        .unwrap_or_default();

                    // Select opposite monitor by default for presentation
                    self.selected_monitor_index =
                        find_recommended_presentation_monitor(&self.monitors, &current_name);
                    if let Some(saved) = self
                        .monitors
                        .iter()
                        .find(|m| m.name == self.control_state.settings.monitor_name)
                    {
                        self.selected_monitor_index = saved.winit_index;
                    } else if let Some(selected) = self
                        .monitors
                        .iter()
                        .find(|m| m.winit_index == self.selected_monitor_index)
                    {
                        self.control_state.settings.monitor_name = selected.name.clone();
                    }
                }
            }
            self.update_presentation_aspect_ratio();
        }
    }

    /// Far-left vertical icon navigation bar.
    pub(super) fn draw_left_sidebar(&mut self, ui: &mut egui::Ui, theme: &AppTheme) {
        let left_panel_response = egui::Panel::left("left_icon_sidebar")
            .exact_size(56.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(theme.bg_panel)
                    .stroke(egui::Stroke::NONE)
                    .inner_margin(egui::Margin::symmetric(4, 8)),
            )
            .show(ui, |ui| {
                crate::ui::render_icon_sidebar(ui, &mut self.control_state);
            });

        let left_rect = left_panel_response.response.rect;
        ui.painter().line_segment(
            [
                egui::pos2(left_rect.max.x, left_rect.min.y),
                egui::pos2(left_rect.max.x, left_rect.max.y),
            ],
            egui::Stroke::new(1.0, theme.border_default),
        );
    }

    /// Top shortcut bar hosting quick items and the workspace options cluster.
    pub(super) fn draw_top_bar(&mut self, ui: &mut egui::Ui, theme: &AppTheme) {
        let top_panel_response = egui::Panel::top("top_bar_panel")
            .resizable(false)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(theme.bg_panel)
                    .stroke(egui::Stroke::NONE)
                    .inner_margin(egui::Margin::symmetric(14, 8)),
            )
            .show(ui, |ui| {
                crate::ui::render_top_bar(
                    ui,
                    &mut self.control_state,
                    &mut self.slide_deck,
                    &mut self.is_presentation_open,
                );
            });

        let top_rect = top_panel_response.response.rect;
        ui.painter().line_segment(
            [
                egui::pos2(top_rect.min.x, top_rect.max.y),
                egui::pos2(top_rect.max.x, top_rect.max.y),
            ],
            egui::Stroke::new(1.0, theme.border_default),
        );
    }

    /// Bottom presentation dock, visible while a presentation is active.
    pub(super) fn draw_presentation_dock(&mut self, ui: &mut egui::Ui, theme: &AppTheme) {
        if self.control_state.active_sidebar_page == SidebarPage::Design {
            return;
        }
        if !(self.is_presentation_open && self.control_state.show_presentation_dock) {
            return;
        }
        let dock_h = self.control_state.dock_height.clamp(130.0, 420.0);
        let dock_response = egui::Panel::bottom("presentation_dock")
            .resizable(true)
            .show_separator_line(false)
            .default_size(dock_h)
            .min_size(130.0)
            .max_size(420.0)
            .frame(
                egui::Frame::new()
                    .fill(theme.bg_panel)
                    .stroke(egui::Stroke::NONE)
                    .inner_margin(egui::Margin::symmetric(10, 8)),
            )
            .show(ui, |ui| {
                crate::ui::render_presentation_dock(
                    ui,
                    &mut self.control_state,
                    &mut self.slide_deck,
                    &mut self.is_presentation_open,
                    &mut self.is_preview_modal_open,
                );
            });

        let current_h = dock_response.response.rect.height();
        if current_h >= 130.0 && (current_h - self.control_state.dock_height).abs() > 0.5 {
            self.control_state.dock_height = current_h;
            self.control_state.capture_profiles();
        }

        let dock_rect = dock_response.response.rect;
        ui.painter().line_segment(
            [
                egui::pos2(dock_rect.min.x, dock_rect.min.y),
                egui::pos2(dock_rect.max.x, dock_rect.min.y),
            ],
            egui::Stroke::new(1.0, theme.border_default),
        );
    }

    /// Central workspace, switching between the chat page and the docking layout.
    pub(super) fn draw_workspace(&mut self, ui: &mut egui::Ui, theme: &AppTheme) {
        match self.control_state.active_sidebar_page {
            SidebarPage::Design => {
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(theme.bg_app).inner_margin(10))
                    .show(ui, |ui| {
                        self.design_editor.render(
                            ui,
                            &mut self.control_state,
                            &mut self.slide_deck,
                            &mut self.is_presentation_open,
                            &self.db,
                        )
                    });
            }
            SidebarPage::Chat => {
                let mut rendered_leaf_rects: Vec<(ContainerKind, egui::Rect)> = Vec::new();

                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(theme.bg_app)
                            .inner_margin(egui::Margin::ZERO),
                    )
                    .show(ui, |ui| {
                        crate::ui::render_chat_page(
                            ui,
                            &mut self.control_state,
                            &mut self.slide_deck,
                            &mut self.is_presentation_open,
                            &mut rendered_leaf_rects,
                            Some(&self.db),
                        );
                    });
                self.handle_docking(ui, &rendered_leaf_rects);
            }
            SidebarPage::Home | SidebarPage::Bible => {
                let mut rendered_leaf_rects: Vec<(ContainerKind, egui::Rect)> = Vec::new();

                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(theme.bg_app)
                            .inner_margin(egui::Margin::ZERO),
                    )
                    .show(ui, |ui| {
                        if self.control_state.active_sidebar_page == SidebarPage::Bible {
                            crate::ui::render_bible_page(
                                ui,
                                &mut self.control_state,
                                &mut self.slide_deck,
                                &mut self.is_presentation_open,
                                &mut rendered_leaf_rects,
                                Some(&self.db),
                            );
                        } else {
                            crate::ui::render_layout(
                                ui,
                                &mut self.control_state,
                                &mut self.slide_deck,
                                &mut self.is_presentation_open,
                                &mut rendered_leaf_rects,
                                Some(&self.db),
                            );
                        }
                    });

                // Drag & drop docking handler across all visible containers
                self.handle_docking(ui, &rendered_leaf_rects);
            }
        }
    }
}
