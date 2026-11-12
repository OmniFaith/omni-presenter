use super::PresentationApp;
use eframe::egui;

impl PresentationApp {
    /// Restore exclusively owns database writes while the normal event loop is suspended.
    /// The responsive progress screen prevents local/remote edits and stale autosaves.
    pub(super) fn workspace_transfer(&mut self, ui: &mut egui::Ui) -> bool {
        let ctx = ui.ctx().clone();
        if self.restored_workspace {
            ui.label(self.control_state.settings.language.text("transfer-reopen"));
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return true;
        }
        crate::ui::modals::settings::transfer::poll(
            &ctx,
            &mut self.control_state,
            &mut self.slide_deck,
            &self.db,
        );
        if self.restore_task.is_none()
            && let Some(request) = crate::ui::modals::settings::transfer::take_restore(&ctx)
        {
            self.design_editor
                .flush(&self.db, &mut self.slide_deck, true);
            self.control_state.capture_profiles();
            let settings = self.control_state.settings.clone();
            // Stop listeners before the worker replaces persistent content.
            self.browser_server = None;
            let db = self.db.clone();
            self.restore_task = Some(crate::services::tasks::BackgroundTask::spawn_with_repaint(
                &ctx,
                move || {
                    let directory = db.transfer_directory()?;
                    let recovery = directory.join(format!(
                        "{}.zip",
                        crate::core::preferences::new_id("before_import")
                    ));
                    crate::services::transfer::restore(
                        &db,
                        &settings,
                        &request.source,
                        &recovery,
                        &directory.join("imported-assets"),
                    )
                },
            ));
        }
        if let Some(task) = &self.restore_task {
            if let Some(result) = task.take() {
                self.restore_task = None;
                match result {
                    Ok(_) => {
                        self.restored_workspace = true;
                        // on_exit deliberately skips all writes from the old workspace.
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    Err(error) => {
                        self.control_state.settings_ui.error =
                            self.control_state.settings.language.text(&error).to_owned();
                        self.restart_browser_server();
                    }
                }
            } else {
                if ctx.input(|i| i.viewport().close_requested()) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                }
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    ui.spinner();
                    ui.label(
                        self.control_state
                            .settings
                            .language
                            .text("transfer-working"),
                    );
                    ui.label(self.control_state.settings.language.text("transfer-reopen"));
                });
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
            }
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closing_after_restore_never_saves_the_old_profiles_over_imported_settings() {
        let db = crate::services::db::AppDb::open_in_memory().unwrap();
        let mut app = PresentationApp::with_db(db.clone());
        let mut imported = app.control_state.settings.clone();
        imported.operators[0].name = "Imported operator".into();
        db.save_settings(&imported).unwrap();
        app.restored_workspace = true;
        eframe::App::on_exit(&mut app);
        assert_eq!(db.load_settings().unwrap(), Some(imported));
    }
}
