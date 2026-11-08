use eframe::egui;

use super::PresentationApp;

impl PresentationApp {
    /// Renders every modal dialog and returns the monitor index to move to, if changed.
    pub(super) fn draw_modals(
        &mut self,
        ctx: &egui::Context,
        is_text_focused: &mut bool,
    ) -> Option<usize> {
        // Initial Setup / Welcome Wizard Modal Dialog (First Run)
        let wizard_monitor_target = crate::ui::render_setup_wizard_modal(
            ctx,
            &mut self.control_state,
            &self.monitors,
            Some(&self.db),
        );

        // Settings Modal Dialog
        crate::ui::render_settings_modal(ctx, &mut self.control_state, &self.monitors, Some(&self.db));
        let monitor_changed_to = self.save_settings_draft().or(wizard_monitor_target);

        // Floating Screen Preview Program/Window
        if self.control_state.settings_ui.floating_preview.is_some() {
            let projected_text = self.slide_deck.projected_text();
            let live_title = self.slide_deck.live_title();
            let next_text = self.slide_deck.next_projected_text();
            let output_theme = crate::services::media::live_theme(&self.control_state, &self.slide_deck);
            crate::ui::render_floating_screen_preview(
                ctx,
                &mut self.control_state,
                &self.monitors,
                Some(projected_text),
                Some(live_title),
                if next_text.is_empty() { None } else { Some(next_text) },
                Some(&output_theme),
            );
        }

        // Floating Bible Staged Search Overlay
        if self.control_state.bible.show_search {
            *is_text_focused = true;
        }
        crate::ui::pages::bible::render_bible_search_overlay(
            ctx,
            &mut self.control_state,
            &mut self.slide_deck,
        );

        // Lyrics Editor Modal Dialog (with SQLite DB persistence!)
        crate::ui::render_editor_modal(
            ctx,
            &mut self.control_state,
            &mut self.slide_deck,
            is_text_focused,
            Some(&self.db),
        );

        // Custom Theme Creator Modal Dialog
        if self.control_state.theme_editor.open {
            let cur = self.slide_deck.current_slide_text();
            let prev = self.slide_deck.preview_slide_text();
            self.control_state.theme_editor.preview_text_override = if !cur.trim().is_empty() {
                Some(cur.to_string())
            } else if !prev.trim().is_empty() {
                Some(prev.to_string())
            } else {
                None
            };
        }
        crate::ui::render_theme_creator_modal(ctx, &mut self.control_state, Some(&self.db));

        // Preview Carousel Modal Dialog (if opened via button)
        if self.is_preview_modal_open {
            crate::ui::render_preview_modal(
                ctx,
                &mut self.control_state,
                &mut self.slide_deck,
                &mut self.is_presentation_open,
                &mut self.is_preview_modal_open,
            );
        }

        // Presentation & Dock Settings Modal Dialog
        crate::ui::render_presentation_settings_modal(
            ctx,
            &mut self.control_state,
            &mut self.is_preview_modal_open,
        );

        // Folder Creator Modal Dialog
        crate::ui::render_folder_creator_modal(ctx, &mut self.control_state, Some(&self.db));

        // Event / Service Editor Modal Dialog
        crate::ui::render_event_editor_modal(ctx, &mut self.control_state, Some(&self.db));

        // Strophe / Verse Quick Editor Modal Dialog
        crate::ui::render_strophe_editor_modal(
            ctx,
            &mut self.control_state,
            &mut self.slide_deck,
            Some(&self.db),
        );

        // Strophe Tag Manager Modal Dialog
        crate::ui::render_tag_manager_modal(ctx, &mut self.control_state, Some(&self.db));

        // Custom Text Editor Modal Dialog
        crate::ui::render_text_editor_modal(ctx, &mut self.control_state, Some(&self.db));

        // Verse Saver Modal Dialog
        crate::ui::render_verse_saver_modal(ctx, &mut self.control_state, Some(&self.db));

        // Web Lyrics Search Modal Dialog (LRCLIB)
        crate::ui::render_web_lyrics_modal(
            ctx,
            &mut self.control_state,
            &mut self.slide_deck,
            Some(&self.db),
        );

        // Web Media Modal Dialog (YouTube, Vimeo, etc.)
        crate::ui::render_web_media_modal(ctx, &mut self.control_state, Some(&self.db));

        crate::ui::containers::selection::render_media_conversion(ctx, &mut self.control_state, Some(&self.db));

        // Global QR Code modal dialog for sharing links
        crate::ui::components::render_active_qr_dialog(ctx, &self.control_state.theme);

        monitor_changed_to
    }
}
