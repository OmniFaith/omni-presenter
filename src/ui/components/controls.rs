use crate::ui::AppTheme;
use eframe::egui::{self, Margin, RichText};

pub fn search_field(
    ui: &mut egui::Ui,
    id: impl egui::AsIdSalt,
    query: &mut String,
    hint: &str,
    width: f32,
    theme: &AppTheme,
) -> egui::Response {
    let field_w = width.max(100.0);
    let field_h = 32.0;

    let salt_id = ui.make_persistent_id(id);
    let input_id = salt_id.with("search_input");
    let is_focused = ui.ctx().memory(|m| m.has_focus(input_id));

    let stroke_c = if is_focused {
        theme.accent
    } else {
        theme.border_default
    };

    let frame = egui::Frame::new()
        .fill(theme.bg_pill)
        .stroke(egui::Stroke::new(1.0, stroke_c))
        .corner_radius(6.0)
        .inner_margin(Margin::symmetric(10, 6));

    let icon_c = if is_focused {
        theme.accent
    } else {
        theme.text_muted
    };

    let prefix_text = RichText::new(format!("{}  ", egui_phosphor::regular::MAGNIFYING_GLASS))
        .size(13.0)
        .color(icon_c);

    let edit = egui::TextEdit::singleline(query)
        .id(input_id)
        .frame(frame)
        .prefix(prefix_text)
        .hint_text(RichText::new(crate::ui::i18n::text(ui, hint)).color(theme.text_muted).size(12.0))
        .font(egui::FontId::proportional(12.0))
        .margin(Margin::symmetric(10, 6))
        .align(egui::Align2::LEFT_CENTER);

    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(field_w, field_h),
        egui::Sense::click(),
    );

    let edit_resp = ui.put(rect, edit);

    // Clear 'X' button overlay when query is not empty
    if !query.is_empty() {
        let btn_size = egui::vec2(18.0, 18.0);
        let btn_rect = egui::Rect::from_center_size(
            egui::pos2(rect.right() - 14.0, rect.center().y),
            btn_size,
        );
        let clear_resp = ui.put(
            btn_rect,
            egui::Button::new(
                RichText::new(egui_phosphor::regular::X)
                    .size(11.0)
                    .color(theme.text_muted),
            )
            .frame(false),
        );
        if clear_resp.clicked() {
            query.clear();
            edit_resp.request_focus();
        }
    }

    if resp.clicked() && !edit_resp.has_focus() {
        edit_resp.request_focus();
    }

    edit_resp
}

#[derive(Clone, Copy)]
pub enum ButtonTone {
    Neutral,
    Primary,
    Danger,
}

pub fn text_button(
    ui: &mut egui::Ui,
    text: &str,
    tone: ButtonTone,
    theme: &AppTheme,
) -> egui::Response {
    let text = crate::ui::i18n::text(ui, text);
    let (fill, color, stroke) = match tone {
        ButtonTone::Neutral => (
            theme.bg_pill,
            theme.text_primary,
            egui::Stroke::new(1.0, theme.border_default),
        ),
        ButtonTone::Primary => (theme.accent, theme.text_on_accent, egui::Stroke::NONE),
        ButtonTone::Danger => (
            theme.status_danger.linear_multiply(0.12),
            theme.status_danger,
            egui::Stroke::new(1.0, theme.status_danger.linear_multiply(0.3)),
        ),
    };
    ui.add(
        egui::Button::new(RichText::new(text).size(12.5).color(color))
            .fill(fill)
            .stroke(stroke)
            .corner_radius(7.0)
            .min_size(egui::vec2(0.0, 32.0)),
    )
    .on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn section_heading(ui: &mut egui::Ui, text: &str, theme: &AppTheme) {
    let text = crate::ui::i18n::text(ui, text);
    ui.label(
        RichText::new(text)
            .size(10.5)
            .strong()
            .color(theme.text_muted),
    );
    ui.add_space(4.0);
}
