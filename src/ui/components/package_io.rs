//! Background package dialogs shared by interface themes and plugins.
use crate::services::tasks::BackgroundTask;
use eframe::egui;
use serde::{Serialize, de::DeserializeOwned};

pub fn cancel_import<T: Clone + Send + Sync + 'static>(ctx: &egui::Context, id: &'static str) {
    ctx.data_mut(|data| data.remove::<BackgroundTask<Option<T>>>(egui::Id::unique(id)));
}

pub fn import<T: Clone + DeserializeOwned + Send + Sync + 'static>(
    ui: &mut egui::Ui,
    id: &'static str,
    label: &str,
    validate: fn(&T) -> Result<(), String>,
) -> Option<Result<T, String>> {
    let id = egui::Id::unique(id);
    let pending = ui
        .ctx()
        .data(|data| data.get_temp::<BackgroundTask<Option<T>>>(id));
    if ui
        .add_enabled(pending.is_none(), egui::Button::new(label))
        .clicked()
    {
        let task = BackgroundTask::spawn_with_repaint(ui.ctx(), move || {
            let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .pick_file()
            else {
                return Ok(None);
            };
            if std::fs::metadata(&path).map_err(|e| e.to_string())?.len() > 2 * 1024 * 1024 {
                return Err("plugins-too-large".into());
            }
            let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
            let package: T = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            validate(&package)?;
            Ok(Some(package))
        });
        ui.ctx().data_mut(|data| data.insert_temp(id, task));
    }
    if let Some(task) = pending {
        if let Some(result) = task.take() {
            ui.ctx()
                .data_mut(|data| data.remove::<BackgroundTask<Option<T>>>(id));
            return match result {
                Ok(Some(package)) => Some(Ok(package)),
                Ok(None) => None,
                Err(error) => Some(Err(error)),
            };
        }
        ui.spinner();
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(33));
    }
    None
}

pub fn export<T: Serialize>(
    ui: &mut egui::Ui,
    id: &'static str,
    label: &str,
    name: &str,
    package: &T,
) {
    let id = egui::Id::unique(id);
    let pending = ui
        .ctx()
        .data(|data| data.get_temp::<BackgroundTask<()>>(id));
    if ui
        .add_enabled(pending.is_none(), egui::Button::new(label))
        .clicked()
    {
        let name = name.to_owned();
        let bytes = serde_json::to_vec_pretty(package).map_err(|e| e.to_string());
        let task = BackgroundTask::spawn_with_repaint(ui.ctx(), move || {
            let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_file_name(name)
                .save_file()
            else {
                return Ok(());
            };
            std::fs::write(path, bytes?).map_err(|e| e.to_string())
        });
        ui.ctx().data_mut(|data| data.insert_temp(id, task));
    }
    if let Some(task) = pending {
        if let Some(result) = task.take() {
            ui.ctx()
                .data_mut(|data| data.remove::<BackgroundTask<()>>(id));
            if let Err(error) = result {
                ui.colored_label(egui::Color32::LIGHT_RED, error);
            }
        } else {
            ui.spinner();
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(33));
        }
    }
}
