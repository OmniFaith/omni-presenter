use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, Response, Stroke};
use egui_phosphor::regular::{
    ARROW_SQUARE_OUT, CHECK, COPY, COPY_SIMPLE, EYE, GEAR_SIX, INFO, POWER, QR_CODE,
    SLIDERS_HORIZONTAL, TRASH,
};

/// Action emitted by a screen card interaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenCardAction {
    None,
    Select,
    ToggleEnabled,
    OpenSettings,
    OpenPreview,
    CopyLink,
    ShowQrCode,
    OpenBrowser,
    Duplicate,
    Delete,
}

/// Properties for rendering a presentation or virtual screen card.
pub struct ScreenCardProps<'a> {
    pub id_salt: &'a str,
    pub label: &'a str,
    pub name: &'a str,
    pub subtitle: Option<&'a str>,
    pub selected: bool,
    pub enabled: bool,
    pub is_virtual: bool,
    pub theme: &'a AppTheme,
}

/// Renders a screen card with on-hold emphasis, hover quick actions, and context menu.
pub fn render_screen_card(ui: &mut egui::Ui, props: ScreenCardProps<'_>) -> (Response, ScreenCardAction) {
    let language = crate::ui::i18n::language(ui.ctx());
    let mut action = ScreenCardAction::None;

    // Allocate total card layout rectangle (142x88)
    let (rect, _) = ui.allocate_exact_size(egui::vec2(142.0, 88.0), egui::Sense::hover());

    // Define child button rects inside the card
    let prev_rect = egui::Rect::from_min_size(
        rect.left_bottom() + egui::vec2(8.0, -32.0),
        egui::vec2(32.0, 24.0),
    );
    let cfg_rect = egui::Rect::from_min_size(
        rect.right_bottom() + egui::vec2(-40.0, -32.0),
        egui::vec2(32.0, 24.0),
    );

    // Coordinate-based containment: 100% deterministic, impervious to widget hit-test oscillation
    let pointer_pos = ui.input(|i| i.pointer.hover_pos());
    let pointer_in_card = pointer_pos.is_some_and(|pos| rect.contains(pos));
    let pointer_in_prev = pointer_pos.is_some_and(|pos| prev_rect.contains(pos));
    let pointer_in_cfg = pointer_pos.is_some_and(|pos| cfg_rect.contains(pos));
    let pointer_in_child = pointer_in_prev || pointer_in_cfg;

    // Card body interaction: Only senses clicks when NOT over the child buttons.
    // This gives the child buttons exclusive click priority without parent stealing.
    let card_id = ui.make_persistent_id(props.id_salt);
    let body_sense = if pointer_in_child {
        egui::Sense::hover()
    } else {
        egui::Sense::click()
    };
    let response = ui.interact(rect, card_id.with("body"), body_sense);

    // Hover animation is driven strictly by pointer containment in the card rect.
    // This completely eliminates any flickering loop.
    let hover = ui.ctx().animate_bool(card_id.with("hover"), pointer_in_card);
    let on_hold = !pointer_in_child && response.is_pointer_button_down_on();
    let hold_anim = ui.ctx().animate_bool(card_id.with("hold"), on_hold);
    let selected_anim = ui.ctx().animate_bool(card_id.with("selected"), props.selected);

    // Card background with smooth on-hold, hover, and selection interpolation
    let mut fill = props
        .theme
        .bg_pill
        .lerp_to_gamma(props.theme.bg_card_hover, hover)
        .lerp_to_gamma(props.theme.accent_soft, selected_anim * 0.85);

    if hold_anim > 0.0 {
        fill = fill.lerp_to_gamma(props.theme.accent_soft, hold_anim * 0.5);
    }

    ui.painter().rect_filled(rect, 8.0, fill);

    // Border with on-hold tactile emphasis
    let border_stroke_width = 1.0 + selected_anim * 1.0 + hold_anim * 0.5 + hover * 0.5;
    let border_color = if props.selected || hold_anim > 0.0 {
        props.theme.accent
    } else {
        props.theme.border_default.lerp_to_gamma(props.theme.border_active, hover)
    };

    ui.painter().rect_stroke(
        rect,
        8.0,
        Stroke::new(border_stroke_width, border_color),
        egui::StrokeKind::Inside,
    );

    // Status indicator dot (green when active, muted when disabled)
    let dot_pos = rect.left_top() + egui::vec2(12.0, 12.0);
    let (dot_color, dot_radius) = if props.enabled {
        (Color32::from_rgb(34, 197, 94), 3.5)
    } else {
        (props.theme.text_muted, 3.0)
    };
    ui.painter().circle_filled(dot_pos, dot_radius, dot_color);

    // Main Card Label (centered at 32.0 px font, preserving exact test locator)
    let label_color = if props.selected {
        props.theme.accent
    } else {
        props
            .theme
            .text_muted
            .lerp_to_gamma(props.theme.text_primary, hover * 0.6)
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        props.label,
        egui::FontId::proportional(32.0),
        label_color,
    );

    // Selected checkmark at top-right
    if props.selected {
        ui.painter().text(
            rect.right_top() + egui::vec2(-12.0, 12.0),
            egui::Align2::CENTER_CENTER,
            CHECK,
            egui::FontId::proportional(13.0),
            props.theme.accent,
        );
    }

    // Subtitle / Status Text (shown when not hovering buttons)
    if hover < 0.15 {
        if !props.enabled {
            ui.painter().text(
                rect.center_bottom() + egui::vec2(0.0, -11.0),
                egui::Align2::CENTER_CENTER,
                language.text("workspace-disabled"),
                egui::FontId::proportional(10.0),
                props.theme.text_secondary,
            );
        } else if let Some(subtitle) = props.subtitle {
            ui.painter().text(
                rect.center_bottom() + egui::vec2(0.0, -11.0),
                egui::Align2::CENTER_CENTER,
                subtitle,
                egui::FontId::proportional(9.5),
                props.theme.text_secondary,
            );
        }
    }

    // Hover Quick-Action Buttons (Eye & Settings)
    // Appear smoothly when pointer is inside the card
    let show_hover_buttons = pointer_in_card || hover > 0.05;
    if show_hover_buttons {
        let alpha = if pointer_in_card { 255 } else { (hover * 255.0).clamp(0.0, 255.0) as u8 };
        let btn_bg = props.theme.bg_card;
        let btn_bg_alpha = Color32::from_rgba_unmultiplied(btn_bg.r(), btn_bg.g(), btn_bg.b(), alpha);
        let btn_stroke = Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(
                props.theme.border_default.r(),
                props.theme.border_default.g(),
                props.theme.border_default.b(),
                alpha,
            ),
        );

        // Preview button (bottom-left)
        let prev_id = card_id.with("btn_preview");
        let prev_resp = ui
            .interact(prev_rect, prev_id, egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(language.text("workspace-view-in-a-new-window"));
        let prev_h = ui.ctx().animate_bool(prev_id.with("h"), prev_resp.hovered());

        let prev_fill = if prev_h > 0.0 {
            props.theme.accent_soft
        } else {
            btn_bg_alpha
        };
        let prev_border = if prev_h > 0.0 {
            Stroke::new(1.0, props.theme.accent)
        } else {
            btn_stroke
        };
        ui.painter().rect_filled(prev_rect, 6.0, prev_fill);
        ui.painter().rect_stroke(prev_rect, 6.0, prev_border, egui::StrokeKind::Inside);
        let prev_icon_color = if prev_h > 0.0 {
            props.theme.accent
        } else {
            props.theme.text_secondary
        };
        ui.painter().text(
            prev_rect.center(),
            egui::Align2::CENTER_CENTER,
            EYE,
            egui::FontId::proportional(13.5),
            Color32::from_rgba_unmultiplied(
                prev_icon_color.r(),
                prev_icon_color.g(),
                prev_icon_color.b(),
                alpha,
            ),
        );
        if prev_resp.clicked() {
            action = ScreenCardAction::OpenPreview;
        }

        // Settings button (bottom-right)
        let cfg_id = card_id.with("btn_settings");
        let cfg_resp = ui
            .interact(cfg_rect, cfg_id, egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(language.text("workspace-settings-for-this-screen"));
        let cfg_h = ui.ctx().animate_bool(cfg_id.with("h"), cfg_resp.hovered());

        let cfg_fill = if cfg_h > 0.0 {
            props.theme.accent_soft
        } else {
            btn_bg_alpha
        };
        let cfg_border = if cfg_h > 0.0 {
            Stroke::new(1.0, props.theme.accent)
        } else {
            btn_stroke
        };
        ui.painter().rect_filled(cfg_rect, 6.0, cfg_fill);
        ui.painter().rect_stroke(cfg_rect, 6.0, cfg_border, egui::StrokeKind::Inside);
        let cfg_icon_color = if cfg_h > 0.0 {
            props.theme.accent
        } else {
            props.theme.text_secondary
        };
        ui.painter().text(
            cfg_rect.center(),
            egui::Align2::CENTER_CENTER,
            if props.is_virtual {
                SLIDERS_HORIZONTAL
            } else {
                GEAR_SIX
            },
            egui::FontId::proportional(13.5),
            Color32::from_rgba_unmultiplied(
                cfg_icon_color.r(),
                cfg_icon_color.g(),
                cfg_icon_color.b(),
                alpha,
            ),
        );
        if cfg_resp.clicked() {
            action = ScreenCardAction::OpenSettings;
        }
    }

    // Context Menu on Secondary Click
    response.clone().context_menu(|ui| {
        ui.set_min_width(210.0);

        if props.is_virtual {
            if ui
                .button(format!("{SLIDERS_HORIZONTAL}  {}", language.text("workspace-configure-virtual-screen")))
                .clicked()
            {
                action = ScreenCardAction::OpenSettings;
                ui.close();
            }
            if ui
                .button(format!(
                    "{POWER}  {}",
                    if props.enabled {
                        language.text("workspace-disable-virtual-screen")
                    } else {
                        language.text("workspace-enable-virtual-screen")
                    }
                ))
                .clicked()
            {
                action = ScreenCardAction::ToggleEnabled;
                ui.close();
            }
            if ui
                .button(format!("{EYE}  {}", language.text("workspace-view-in-a-new-window")))
                .clicked()
            {
                action = ScreenCardAction::OpenPreview;
                ui.close();
            }
            ui.separator();
            if ui
                .button(format!("{COPY}  {}", language.text("workspace-copy-web-link")))
                .clicked()
            {
                action = ScreenCardAction::CopyLink;
                ui.close();
            }
            if ui
                .button(format!("{QR_CODE}  QR Code"))
                .clicked()
            {
                action = ScreenCardAction::ShowQrCode;
                ui.close();
            }
            if ui
                .button(format!("{ARROW_SQUARE_OUT}  {}", language.text("workspace-open-in-browser")))
                .clicked()
            {
                action = ScreenCardAction::OpenBrowser;
                ui.close();
            }
            if ui
                .button(format!("{COPY_SIMPLE}  {}", language.text("workspace-duplicate-screen")))
                .clicked()
            {
                action = ScreenCardAction::Duplicate;
                ui.close();
            }
            ui.separator();
            if ui
                .button(
                    egui::RichText::new(format!("{TRASH}  {}", language.text("workspace-delete-screen")))
                        .color(Color32::from_rgb(239, 68, 68)),
                )
                .clicked()
            {
                action = ScreenCardAction::Delete;
                ui.close();
            }
        } else {
            if ui
                .button(format!("{GEAR_SIX}  {}", language.text("workspace-projection-settings")))
                .clicked()
            {
                action = ScreenCardAction::OpenSettings;
                ui.close();
            }
            if ui
                .button(format!(
                    "{POWER}  {}",
                    if props.enabled {
                        language.text("workspace-disable-projection-on-this-monitor")
                    } else {
                        language.text("workspace-enable-projection-on-this-monitor")
                    }
                ))
                .clicked()
            {
                action = ScreenCardAction::ToggleEnabled;
                ui.close();
            }
            if ui
                .button(format!("{EYE}  {}", language.text("workspace-view-in-a-new-window")))
                .clicked()
            {
                action = ScreenCardAction::OpenPreview;
                ui.close();
            }
            ui.separator();
            ui.label(
                egui::RichText::new(format!("{INFO}  {}", props.name))
                    .size(11.0)
                    .color(props.theme.text_secondary),
            );
            if let Some(sub) = props.subtitle {
                ui.label(
                    egui::RichText::new(sub)
                        .size(10.0)
                        .color(props.theme.text_muted),
                );
            }
        }
    });

    if action == ScreenCardAction::None && !pointer_in_child && response.clicked() {
        action = ScreenCardAction::Select;
    }

    let final_resp = if pointer_in_child {
        response
    } else {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    };

    (final_resp, action)
}
