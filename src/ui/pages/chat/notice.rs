use crate::core::state::{ControlPanelState, SidebarPage};
use eframe::egui::{self, RichText, Vec2};
use egui_phosphor::regular::X;

use super::channels::avatar;
use super::panel;

pub fn render_chat_notice(ctx: &egui::Context, state: &mut ControlPanelState) {
    let Some(notice) = state.chat.notice.clone() else {
        return;
    };
    if notice.since.elapsed().as_secs() >= 8 {
        state.chat.notice = None;
        return;
    }
    let tokens = state.theme.clone();
    let language = state.settings.language;
    let mut dismiss = false;
    let response = egui::Area::new(egui::Id::unique("chat_notification"))
        .order(egui::Order::Tooltip)
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-18.0, 65.0))
        .show(ctx, |ui| {
            panel(&tokens)
                .shadow(egui::epaint::Shadow {
                    offset: [0, 6],
                    blur: 20,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(30),
                })
                .show(ui, |ui| {
                    ui.set_width(290.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(language.text("chat-new-message"))
                                .size(10.5)
                                .color(tokens.text_muted),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if tokens
                                .icon_button(
                                    ui,
                                    X,
                                    language.text("chat-dismiss-notification"),
                                    false,
                                    Vec2::splat(22.0),
                                )
                                .clicked()
                            {
                                dismiss = true;
                            }
                        });
                    });
                    ui.horizontal(|ui| {
                        let photo = state
                            .settings
                            .operators
                            .iter()
                            .find(|p| p.id == notice.item.author_id)
                            .and_then(|p| p.avatar.as_deref());
                        avatar(ui, &notice.item.author, &tokens, false, photo);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&notice.item.author).size(12.5).strong());
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&notice.item.preview)
                                        .size(12.0)
                                        .color(tokens.text_secondary),
                                )
                                .truncate(),
                            );
                        });
                    });
                })
                .response
                .interact(egui::Sense::click())
        });
    if response.inner.clicked() {
        state.active_sidebar_page = SidebarPage::Chat;
        state.chat.peer = notice.item.peer_id;
        state.chat.channel = notice.item.channel_id.unwrap_or_else(|| "general".into());
        state.chat.search.clear();
        state.chat.search_open = false;
        state.chat.history_before = None;
        dismiss = true;
    }
    if dismiss {
        state.chat.notice = None;
    }
    ctx.request_repaint_after(std::time::Duration::from_millis(100));
}
