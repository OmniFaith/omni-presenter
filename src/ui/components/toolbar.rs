use crate::ui::theme::AppTheme;
use eframe::egui::{self, Vec2};

/// Renders a standardized minimalist toolbar action icon button
pub fn toolbar_icon_button(
    ui: &mut egui::Ui,
    icon: &str,
    tooltip: &str,
    theme: &AppTheme,
) -> egui::Response {
    theme.icon_button(ui, icon, tooltip, false, Vec2::new(28.0, 28.0))
}

/// Helper to center a group of widgets horizontally within the available width of `ui`
pub fn centered_horizontal<R>(
    ui: &mut egui::Ui,
    id_source: impl std::hash::Hash + std::fmt::Debug,
    default_w: f32,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let id = ui.make_persistent_id(id_source);
    let prev_w = ui
        .ctx()
        .data_mut(|d| d.get_temp::<f32>(id))
        .unwrap_or(default_w);
    let avail_w = ui.available_width();
    let pad = ((avail_w - prev_w) * 0.5).max(0.0);

    let (res, actual_w) = ui
        .horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            if pad > 0.0 {
                ui.add_space(pad);
            }
            let inner_resp = ui.scope(add_contents);
            (inner_resp.inner, inner_resp.response.rect.width())
        })
        .inner;

    if actual_w > 0.0 && (actual_w - prev_w).abs() > 0.5 {
        ui.ctx().data_mut(|d| d.insert_temp(id, actual_w));
    }
    res
}
