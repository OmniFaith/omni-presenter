use crate::ui::AppTheme;
use eframe::egui;

pub struct RowVisual {
    pub hover: f32,
    pub press: f32,
}

pub fn row_emphasis(
    ui: &egui::Ui,
    response: &egui::Response,
    selected: bool,
    theme: &AppTheme,
) -> RowVisual {
    let hover = ui
        .ctx()
        .animate_bool(response.id.with("hover"), response.hovered());
    let press = ui.ctx().animate_bool(
        response.id.with("press"),
        response.is_pointer_button_down_on(),
    );
    let fill = if selected {
        theme.accent_soft
    } else {
        egui::Color32::TRANSPARENT.lerp_to_gamma(theme.bg_card_hover, hover)
    }
    .lerp_to_gamma(theme.accent_soft, press * 0.55);
    ui.painter()
        .rect_filled(response.rect.shrink(press * 0.4), 7.0, fill);
    if selected {
        ui.painter().rect_stroke(
            response.rect,
            7.0,
            egui::Stroke::new(1.0, theme.accent.linear_multiply(0.4)),
            egui::StrokeKind::Inside,
        );
    }
    RowVisual { hover, press }
}

pub fn selection_summary(ui: &mut egui::Ui, count: usize, theme: &AppTheme) -> egui::Response {
    let language = crate::ui::i18n::language(ui.ctx());
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(
                language.format("common-selected", &[("count", &count.to_string())]),
            )
            .size(11.0)
            .strong()
            .color(theme.accent),
        );
        theme.icon_button(
            ui,
            egui_phosphor::regular::X,
            language.text("selection-clear"),
            false,
            egui::Vec2::splat(24.0),
        )
    })
    .inner
}

pub fn view_toggle(ui: &mut egui::Ui, list: &mut bool, theme: &AppTheme) -> bool {
    let language = crate::ui::i18n::language(ui.ctx());
    let before = *list;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        if theme
            .icon_button(
                ui,
                egui_phosphor::regular::LIST_DASHES,
                language.text("view-list"),
                *list,
                egui::vec2(26.0, 26.0),
            )
            .clicked()
        {
            *list = true;
        }
        if theme
            .icon_button(
                ui,
                egui_phosphor::regular::SQUARES_FOUR,
                language.text("view-grid"),
                !*list,
                egui::vec2(26.0, 26.0),
            )
            .clicked()
        {
            *list = false;
        }
    });
    before != *list
}
