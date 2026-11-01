use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, Margin, Rect, Stroke, Vec2};
use egui_phosphor::regular::CARET_RIGHT;

pub const CONTEXT_MENU_WIDTH: f32 = 208.0;
pub const CONTEXT_MENU_ITEM_HEIGHT: f32 = 27.0;

/// Prepares the context menu UI container with exact compact width and clean spacing
pub fn context_menu_header(ui: &mut egui::Ui) {
    ui.set_width(CONTEXT_MENU_WIDTH);
    ui.set_max_width(CONTEXT_MENU_WIDTH);
    ui.set_min_width(CONTEXT_MENU_WIDTH);
    ui.spacing_mut().item_spacing = Vec2::new(0.0, 1.0);
}

/// Renders a modern context menu item with icon, label, and sleek hover highlight
pub fn context_menu_item(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    theme: &AppTheme,
) -> egui::Response {
    ui.set_width(CONTEXT_MENU_WIDTH);
    ui.set_max_width(CONTEXT_MENU_WIDTH);
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(CONTEXT_MENU_WIDTH, CONTEXT_MENU_ITEM_HEIGHT),
        egui::Sense::click(),
    );

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let bg_color = if resp.hovered() {
        theme.bg_card_hover
    } else {
        Color32::TRANSPARENT
    };

    let icon_color = if resp.hovered() {
        theme.accent
    } else {
        theme.icon_default
    };

    let text_color = if resp.hovered() {
        theme.text_primary
    } else {
        theme.text_primary
    };

    ui.painter().rect_filled(rect, 5.0, bg_color);

    // Left Icon (at x = rect.min.x + 8.0)
    let icon_galley = ui.painter().layout_no_wrap(
        icon.to_string(),
        egui::FontId::proportional(13.5),
        icon_color,
    );
    let icon_pos = egui::pos2(
        rect.min.x + 8.0,
        rect.center().y - icon_galley.size().y / 2.0,
    );
    ui.painter().galley(icon_pos, icon_galley, icon_color);

    // Label Text (at x = rect.min.x + 28.0)
    let text_galley = ui.painter().layout_no_wrap(
        crate::ui::i18n::language(ui.ctx()).text(label).to_string(),
        egui::FontId::proportional(12.0),
        text_color,
    );
    let text_pos = egui::pos2(
        rect.min.x + 28.0,
        rect.center().y - text_galley.size().y / 2.0,
    );
    ui.painter().galley(text_pos, text_galley, text_color);

    resp
}

/// Renders a context menu item with raw text label (without i18n lookup) and optional custom icon color
pub fn context_menu_raw_item(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    icon_color_override: Option<Color32>,
    theme: &AppTheme,
) -> egui::Response {
    ui.set_width(CONTEXT_MENU_WIDTH);
    ui.set_max_width(CONTEXT_MENU_WIDTH);
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(CONTEXT_MENU_WIDTH, CONTEXT_MENU_ITEM_HEIGHT),
        egui::Sense::click(),
    );

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let bg_color = if resp.hovered() {
        theme.bg_card_hover
    } else {
        Color32::TRANSPARENT
    };

    let icon_color = if let Some(custom) = icon_color_override {
        custom
    } else if resp.hovered() {
        theme.accent
    } else {
        theme.icon_default
    };

    let text_color = if resp.hovered() {
        theme.text_primary
    } else {
        theme.text_primary
    };

    ui.painter().rect_filled(rect, 5.0, bg_color);

    // Left Icon (at x = rect.min.x + 8.0)
    let icon_galley = ui.painter().layout_no_wrap(
        icon.to_string(),
        egui::FontId::proportional(13.5),
        icon_color,
    );
    let icon_pos = egui::pos2(
        rect.min.x + 8.0,
        rect.center().y - icon_galley.size().y / 2.0,
    );
    ui.painter().galley(icon_pos, icon_galley, icon_color);

    // Label Text (at x = rect.min.x + 28.0)
    let text_galley = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::proportional(12.0),
        text_color,
    );
    let text_pos = egui::pos2(
        rect.min.x + 28.0,
        rect.center().y - text_galley.size().y / 2.0,
    );
    ui.painter().galley(text_pos, text_galley, text_color);

    resp
}

/// Renders a destructive/danger context menu item (e.g. Excluir música, Deletar estrofe)
pub fn context_menu_danger_item(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    _theme: &AppTheme,
) -> egui::Response {
    ui.set_width(CONTEXT_MENU_WIDTH);
    ui.set_max_width(CONTEXT_MENU_WIDTH);
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(CONTEXT_MENU_WIDTH, CONTEXT_MENU_ITEM_HEIGHT),
        egui::Sense::click(),
    );

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let bg_color = if resp.hovered() {
        Color32::from_rgb(254, 242, 242) // Soft red-50 hover
    } else {
        Color32::TRANSPARENT
    };

    let danger_color = Color32::from_rgb(220, 38, 38); // Crimson-600

    ui.painter().rect_filled(rect, 5.0, bg_color);

    // Left Icon
    let icon_galley = ui.painter().layout_no_wrap(
        icon.to_string(),
        egui::FontId::proportional(13.5),
        danger_color,
    );
    let icon_pos = egui::pos2(
        rect.min.x + 8.0,
        rect.center().y - icon_galley.size().y / 2.0,
    );
    ui.painter().galley(icon_pos, icon_galley, danger_color);

    // Label Text
    let text_galley = ui.painter().layout_no_wrap(
        crate::ui::i18n::language(ui.ctx()).text(label).to_string(),
        egui::FontId::proportional(12.0),
        danger_color,
    );
    let text_pos = egui::pos2(
        rect.min.x + 28.0,
        rect.center().y - text_galley.size().y / 2.0,
    );
    ui.painter().galley(text_pos, text_galley, danger_color);

    resp
}

/// Renders a disabled/inactive context menu item
pub fn context_menu_disabled_item(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    _theme: &AppTheme,
) -> egui::Response {
    ui.set_width(CONTEXT_MENU_WIDTH);
    ui.set_max_width(CONTEXT_MENU_WIDTH);
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(CONTEXT_MENU_WIDTH, CONTEXT_MENU_ITEM_HEIGHT),
        egui::Sense::hover(),
    );

    let muted_color = Color32::from_rgb(180, 185, 195);

    // Left Icon
    let icon_galley = ui.painter().layout_no_wrap(
        icon.to_string(),
        egui::FontId::proportional(13.5),
        muted_color,
    );
    let icon_pos = egui::pos2(
        rect.min.x + 8.0,
        rect.center().y - icon_galley.size().y / 2.0,
    );
    ui.painter().galley(icon_pos, icon_galley, muted_color);

    // Label Text
    let text_galley = ui.painter().layout_no_wrap(
        crate::ui::i18n::language(ui.ctx()).text(label).to_string(),
        egui::FontId::proportional(12.0),
        muted_color,
    );
    let text_pos = egui::pos2(
        rect.min.x + 28.0,
        rect.center().y - text_galley.size().y / 2.0,
    );
    ui.painter().galley(text_pos, text_galley, muted_color);

    resp
}

/// Renders a submenu button with left icon, label, right chevron, and an adjacent popup
pub fn context_menu_submenu<R>(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    theme: &AppTheme,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::Response {
    ui.set_width(CONTEXT_MENU_WIDTH);
    ui.set_max_width(CONTEXT_MENU_WIDTH);
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(CONTEXT_MENU_WIDTH, CONTEXT_MENU_ITEM_HEIGHT),
        egui::Sense::click(),
    );

    let popup_id = ui.make_persistent_id(label);
    let is_open_key = popup_id.with("is_open");
    let mut is_open = ui.data(|d| d.get_temp::<bool>(is_open_key).unwrap_or(false));

    // Open immediately on hover or click
    if resp.hovered() && !is_open {
        is_open = true;
        ui.data_mut(|d| d.insert_temp(is_open_key, true));
    } else if resp.clicked() {
        is_open = !is_open;
        ui.data_mut(|d| d.insert_temp(is_open_key, is_open));
    }
    let is_hovered = resp.hovered() || is_open;

    if is_hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let bg_color = if is_hovered {
        theme.bg_card_hover
    } else {
        Color32::TRANSPARENT
    };

    let icon_color = if is_hovered {
        Color32::from_rgb(0, 174, 239)
    } else {
        Color32::from_rgb(100, 116, 139)
    };

    let text_color = if is_hovered {
        theme.text_primary
    } else {
        theme.text_primary
    };

    ui.painter().rect_filled(rect, 5.0, bg_color);

    // Left Icon
    let icon_galley = ui.painter().layout_no_wrap(
        icon.to_string(),
        egui::FontId::proportional(13.5),
        icon_color,
    );
    let icon_pos = egui::pos2(
        rect.min.x + 8.0,
        rect.center().y - icon_galley.size().y / 2.0,
    );
    ui.painter().galley(icon_pos, icon_galley, icon_color);

    // Label Text
    let text_galley = ui.painter().layout_no_wrap(
        crate::ui::i18n::language(ui.ctx()).text(label).to_string(),
        egui::FontId::proportional(12.0),
        text_color,
    );
    let text_pos = egui::pos2(
        rect.min.x + 28.0,
        rect.center().y - text_galley.size().y / 2.0,
    );
    ui.painter().galley(text_pos, text_galley, text_color);

    // Right Chevron (neatly positioned at rect.max.x - 14.0)
    let chevron_color = Color32::from_rgb(148, 163, 184);
    let chevron_galley = ui.painter().layout_no_wrap(
        CARET_RIGHT.to_string(),
        egui::FontId::proportional(11.0),
        chevron_color,
    );
    let chevron_pos = egui::pos2(
        rect.max.x - 14.0,
        rect.center().y - chevron_galley.size().y / 2.0,
    );
    ui.painter()
        .galley(chevron_pos, chevron_galley, chevron_color);

    if is_open {
        let area = egui::Area::new(popup_id)
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(rect.max.x + 2.0, rect.min.y));
        let frame = egui::Frame::new()
            .fill(theme.bg_card)
            .stroke(Stroke::new(1.0, theme.border_default))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(Margin::symmetric(4, 4))
            .shadow(egui::epaint::Shadow {
                offset: [0, 4],
                blur: 16,
                spread: 0,
                color: Color32::from_black_alpha(20),
            });
        area.show(ui.ctx(), |ui| {
            frame.show(ui, |ui| {
                ui.set_width(CONTEXT_MENU_WIDTH);
                ui.set_max_width(CONTEXT_MENU_WIDTH);
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 1.0);
                add_contents(ui);
            });
        });
    }

    resp
}

/// Renders a clean 1px hairline separator inside a context menu
pub fn context_menu_separator(ui: &mut egui::Ui) {
    ui.add_space(3.0);
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(CONTEXT_MENU_WIDTH, 1.0), egui::Sense::hover());
    let sep_rect = Rect::from_min_max(
        egui::pos2(rect.min.x + 4.0, rect.min.y),
        egui::pos2(rect.max.x - 4.0, rect.max.y),
    );
    let border_color = if ui.visuals().window_stroke.color != Color32::TRANSPARENT {
        ui.visuals().window_stroke.color
    } else {
        ui.visuals().widgets.noninteractive.bg_stroke.color
    };
    ui.painter().rect_filled(sep_rect, 0.5, border_color);
    ui.add_space(3.0);
}
