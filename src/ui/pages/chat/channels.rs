use crate::core::layout::ContainerKind;
use crate::core::state::ControlPanelState;
use crate::ui::components::render_header_bar;
use eframe::egui::{self, RichText, Vec2};
use egui_phosphor::regular::PLUS;

pub fn render_chat_channels(ui: &mut egui::Ui, state: &mut ControlPanelState) {
    let tokens = state.theme.clone();
    let language = state.settings.language;
    render_header_bar(
        ui,
        ContainerKind::ChatChannels,
        "chat-channels",
        &tokens,
    );
    ui.set_min_height(ui.available_height().max(0.0));
    ui.add_space(2.0);

    let Some(snapshot) = state.chat.cached_snapshot.clone() else {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new(language.text("chat-loading")).color(tokens.text_muted));
        });
        return;
    };

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(language.text("chat-channels"))
                .size(11.5)
                .strong()
                .color(tokens.text_primary),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if tokens
                .icon_button(
                    ui,
                    PLUS,
                    language.text("chat-create"),
                    false,
                    Vec2::splat(22.0),
                )
                .clicked()
            {
                super::channel_list::create(state);
            }
        });
    });
    super::channel_list::selection_bar(ui, state, &tokens);

    let footer_h = 44.0;
    let scroll_h = (ui.available_height() - footer_h).max(60.0);
    egui::ScrollArea::vertical()
        .id_salt("chat_rooms")
        .max_height(scroll_h)
        .min_scrolled_height(scroll_h)
        .show(ui, |ui| {
            super::channel_list::render(ui, state, &snapshot, &tokens);
            ui.add_space(14.0);
            ui.label(
                RichText::new(language.text("chat-private"))
                    .size(11.5)
                    .strong()
                    .color(tokens.text_primary),
            );
            ui.add_space(4.0);
            for profile in snapshot
                .profiles
                .iter()
                .filter(|p| p.id != state.settings.active_operator)
            {
                if nav_button(
                    ui,
                    &profile.name,
                    state.chat.peer.as_deref() == Some(&profile.id),
                    Some(profile.online),
                    state
                        .settings
                        .operators
                        .iter()
                        .find(|p| p.id == profile.id)
                        .and_then(|p| p.avatar.as_deref()),
                    snapshot
                        .notifications
                        .private
                        .get(&profile.id)
                        .copied()
                        .unwrap_or(0),
                    &tokens,
                )
                .on_hover_text(if profile.online {
                    language.text("services-online")
                } else {
                    language.text("chat-offline")
                })
                .clicked()
                {
                    state.chat.peer = Some(profile.id.clone());
                    state.chat.channel_selection.clear();
                    state.chat.history_before = None;
                    state.chat.search_all = false;
                }
            }
        });

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        avatar(
            ui,
            &state.settings.operator().name,
            &tokens,
            true,
            state.settings.operator().avatar.as_deref(),
        );
        ui.vertical(|ui| {
            ui.add(
                egui::Label::new(
                    RichText::new(&state.settings.operator().name)
                        .size(11.5)
                        .strong()
                        .color(tokens.text_primary),
                )
                .truncate(),
            );
            ui.label(
                RichText::new(language.text("chat-connected"))
                    .size(10.0)
                    .color(tokens.status_success_fg),
            );
        });
    });
}

pub(super) fn avatar(
    ui: &mut egui::Ui,
    name: &str,
    tokens: &crate::ui::theme::AppTheme,
    own: bool,
    photo: Option<&[u8]>,
) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(32.0), egui::Sense::hover());
    let initials: String = name
        .split_whitespace()
        .filter_map(|s| s.chars().next())
        .take(2)
        .collect();
    crate::ui::components::avatar::paint(ui, rect, photo, &initials.to_uppercase(), tokens, own);
}

pub(super) fn nav_button(
    ui: &mut egui::Ui,
    label: &str,
    active: bool,
    online: Option<bool>,
    photo: Option<&[u8]>,
    unread: usize,
    tokens: &crate::ui::theme::AppTheme,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new((ui.available_width() - 2.0).max(50.0), 34.0),
        if online.is_none() {
            egui::Sense::click_and_drag()
        } else {
            egui::Sense::click()
        },
    );
    crate::ui::components::interaction::row_emphasis(ui, &response, active, tokens);
    let mut left = rect.left() + 10.0;
    if let Some(online) = online {
        if photo.is_some() {
            let rect = egui::Rect::from_center_size(
                egui::pos2(left + 9.0, rect.center().y),
                Vec2::splat(24.0),
            );
            crate::ui::components::avatar::paint(ui, rect, photo, "", tokens, false);
            left += 18.0;
        }
        ui.painter().circle_filled(
            egui::pos2(left + 3.0, rect.center().y),
            3.5,
            if online {
                tokens.status_success_fg
            } else {
                tokens.icon_muted
            },
        );
        left += 15.0;
    }
    let color = if active {
        tokens.accent
    } else {
        tokens.text_secondary
    };
    let text = ui
        .painter()
        .layout_no_wrap(label.into(), egui::FontId::proportional(12.0), color);
    ui.painter()
        .with_clip_rect(
            egui::Rect::from_min_max(
                rect.min + egui::vec2(6.0, 0.0),
                rect.max - egui::vec2(if unread > 0 { 36.0 } else { 6.0 }, 0.0),
            )
            .intersect(ui.clip_rect()),
        )
        .galley(
            egui::pos2(left, rect.center().y - text.size().y * 0.5),
            text,
            color,
        );
    if unread > 0 {
        let badge = egui::Rect::from_center_size(
            egui::pos2(rect.right() - 17.0, rect.center().y),
            Vec2::new(24.0, 18.0),
        );
        ui.painter().rect_filled(badge, 9.0, tokens.accent);
        ui.painter().text(
            badge.center(),
            egui::Align2::CENTER_CENTER,
            if unread > 99 {
                "99+".into()
            } else {
                unread.to_string()
            },
            egui::FontId::proportional(9.0),
            tokens.text_on_accent,
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
