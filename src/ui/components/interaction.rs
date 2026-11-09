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

/// Renders a single-line title and optional subtitle with graceful truncation / ellipsis,
/// guaranteeing that the text never overflows `avail_w`.
pub fn render_item_title_and_subtitle(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    title: &str,
    subtitle: &str,
    title_font: egui::FontId,
    sub_font: egui::FontId,
    title_color: egui::Color32,
    sub_color: egui::Color32,
    start_pos: egui::Pos2,
    avail_w: f32,
) {
    let avail_w = avail_w.max(10.0);
    let sub_trimmed = subtitle.trim();
    let has_sub = !sub_trimmed.is_empty() && avail_w > 50.0;

    if has_sub {
        let sub_natural = painter.layout_no_wrap(sub_trimmed.to_owned(), sub_font.clone(), sub_color);
        let sub_w = sub_natural.size().x;
        let max_sub_w = (avail_w * 0.38).max(24.0);
        let (sub_galley, sub_render_w) = if sub_w <= max_sub_w {
            (sub_natural, sub_w)
        } else {
            let mut job = egui::text::LayoutJob::simple_singleline(sub_trimmed.to_owned(), sub_font.clone(), sub_color);
            job.wrap.max_width = max_sub_w;
            job.wrap.max_rows = 1;
            job.wrap.break_anywhere = true;
            let g = ui.fonts_mut(|f| f.layout_job(job));
            let w = g.size().x;
            (g, w)
        };

        let gap = 8.0;
        let max_title_w = (avail_w - sub_render_w - gap).max(10.0);
        let mut job = egui::text::LayoutJob::simple_singleline(title.to_owned(), title_font, title_color);
        job.wrap.max_width = max_title_w;
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = true;
        let title_galley = ui.fonts_mut(|f| f.layout_job(job));
        let title_render_w = title_galley.size().x;

        painter.galley(
            egui::pos2(start_pos.x, start_pos.y - title_galley.size().y / 2.0),
            title_galley,
            title_color,
        );

        let sub_x = start_pos.x + title_render_w + gap;
        painter.galley(
            egui::pos2(sub_x, start_pos.y - sub_galley.size().y / 2.0),
            sub_galley,
            sub_color,
        );
    } else {
        let mut job = egui::text::LayoutJob::simple_singleline(title.to_owned(), title_font, title_color);
        job.wrap.max_width = avail_w;
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = true;
        let title_galley = ui.fonts_mut(|f| f.layout_job(job));
        painter.galley(
            egui::pos2(start_pos.x, start_pos.y - title_galley.size().y / 2.0),
            title_galley,
            title_color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_item_title_and_subtitle_bounds_long_names() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let painter = ui.painter().clone();
            let avail_w = 150.0;
            let start = egui::pos2(20.0, 14.0);

            // Test with a massive title that would normally exceed 500px:
            render_item_title_and_subtitle(
                ui,
                &painter,
                "Pinterest video #754282637635119717 [754282637635119717] - very long extra text",
                "vídeo",
                egui::FontId::proportional(12.0),
                egui::FontId::proportional(11.0),
                egui::Color32::WHITE,
                egui::Color32::GRAY,
                start,
                avail_w,
            );

            // Test without subtitle
            render_item_title_and_subtitle(
                ui,
                &painter,
                "Another very long title that exceeds available width by a lot",
                "",
                egui::FontId::proportional(12.0),
                egui::FontId::proportional(11.0),
                egui::Color32::WHITE,
                egui::Color32::GRAY,
                start,
                avail_w,
            );
        });
        output.textures_delta.clear();
    }
}
