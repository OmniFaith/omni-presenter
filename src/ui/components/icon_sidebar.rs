use crate::core::state::{ControlPanelState, SidebarPage};
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Vec2};

const ICON_LYRICS: &str = "\u{E001}"; // music-note-2.svg
const ICON_BIBLE: &str = "\u{E002}";  // christian-bible.svg
const ICON_CHAT: &str = "\u{E003}";   // chat-circle-2.svg
pub const ICON_DESIGN: &str = "\u{E004}"; // Layers.svg

fn sidebar_button(
    ui: &mut egui::Ui,
    icon: &str,
    is_active: bool,
    tooltip: &str,
    theme: &AppTheme,
) -> egui::Response {
    theme.icon_button(ui, icon, tooltip, is_active, Vec2::splat(38.0))
}

pub fn render_icon_sidebar(ui: &mut egui::Ui, state: &mut ControlPanelState) {
    let theme = &state.theme;
    let language = state.settings.language;

    ui.vertical_centered(|ui| {
        ui.add_space(8.0);

        // 1. Home / Presentation (Letras)
        let is_home = state.active_sidebar_page == SidebarPage::Home;
        if sidebar_button(
            ui,
            ICON_LYRICS,
            is_home,
            language.text("nav-home"),
            theme,
        )
        .clicked()
        {
            state.active_sidebar_page = SidebarPage::Home;
        }

        ui.add_space(8.0);

        // 2. Bible
        let is_bible = state.active_sidebar_page == SidebarPage::Bible;
        if state.settings.bible_enabled {
            let resp = sidebar_button(
                ui,
                ICON_BIBLE,
                is_bible,
                language.text("nav-bible"),
                theme,
            );
            if resp.clicked() {
                if is_bible {
                    state.bible.show_search = !state.bible.show_search;
                    state.bible.search_focus = state.bible.show_search;
                } else {
                    state.active_sidebar_page = SidebarPage::Bible;
                }
            }
            resp.context_menu(|ui| {
                crate::ui::components::context_menu_header(ui);
                let theme = &state.theme;
                if crate::ui::components::context_menu_item(
                    ui,
                    ICON_BIBLE,
                    language.text("command-bible"),
                    theme,
                )
                .clicked()
                {
                    state.active_sidebar_page = SidebarPage::Bible;
                    ui.close();
                }
                if crate::ui::components::context_menu_item(
                    ui,
                    egui_phosphor::regular::MAGNIFYING_GLASS,
                    language.text("command-bible-search"),
                    theme,
                )
                .clicked()
                {
                    state.bible.show_search = true;
                    state.bible.search_focus = true;
                    ui.close();
                }
            });
        }

        ui.add_space(8.0);

        // 3. Chat
        let is_chat = state.active_sidebar_page == SidebarPage::Chat;
        if state.settings.chat_enabled {
            let response = sidebar_button(
                ui,
                ICON_CHAT,
                is_chat,
                language.text("nav-chat"),
                theme,
            );
            if state.chat.notifications.total > 0 {
                let label = if state.chat.notifications.total > 99 {
                    "99+".into()
                } else {
                    state.chat.notifications.total.to_string()
                };
                let width = if label.len() > 2 { 25.0 } else { 18.0 };
                let rect = egui::Rect::from_center_size(
                    response.rect.right_top() + egui::vec2(-1.0, 2.0),
                    egui::vec2(width, 18.0),
                );
                ui.painter()
                    .rect_filled(rect.expand(2.0), 11.0, theme.bg_panel);
                ui.painter().rect_filled(rect, 9.0, theme.status_danger);
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(9.0),
                    egui::Color32::WHITE,
                );
            }
            if response.clicked() {
                state.active_sidebar_page = SidebarPage::Chat;
            }
        }

        ui.add_space(8.0);
        if sidebar_button(
            ui,
            ICON_DESIGN,
            state.active_sidebar_page == SidebarPage::Design,
            language.text("nav-design"),
            theme,
        )
        .clicked()
        {
            state.active_sidebar_page = SidebarPage::Design;
        }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(8.0);

            let is_settings = state.settings_ui.open;
            if sidebar_button(
                ui,
                egui_phosphor::regular::SLIDERS_HORIZONTAL,
                is_settings,
                language.text("nav-settings"),
                theme,
            )
            .clicked()
            {
                state.settings_ui.open = !state.settings_ui.open;
            }
        });
    });
}
