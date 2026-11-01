use crate::{
    core::layout::ContainerKind,
    ui::{
        components::drag_drop::{self, DragPayload, DragSource},
        theme::AppTheme,
    },
};
use eframe::egui;

pub fn render_header_bar(ui: &mut egui::Ui, kind: ContainerKind, title: &str, theme: &AppTheme) {
    let language = crate::ui::i18n::language(ui.ctx());
    let title = language.text(title);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 26.0),
        egui::Sense::click_and_drag(),
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        title,
        egui::FontId::proportional(12.0),
        theme.text_primary,
    );
    ui.painter().text(
        rect.right_center() - egui::vec2(8.0, 0.0),
        egui::Align2::CENTER_CENTER,
        egui_phosphor::regular::DOTS_SIX_VERTICAL,
        egui::FontId::proportional(13.0),
        theme.icon_muted,
    );
    drag_drop::source(ui, &response, || {
        DragPayload::new(
            DragSource::Panel(kind),
            title,
            language.text("drag-panel"),
            egui_phosphor::regular::LAYOUT,
        )
    });
    response
        .on_hover_cursor(egui::CursorIcon::Grab)
        .on_hover_text(language.text("workspace-drag-to-reposition"));
}
