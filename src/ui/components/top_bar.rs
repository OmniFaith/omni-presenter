use crate::core::models::QuickItemKind;
use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use crate::ui::components::{
    context_menu_danger_item, context_menu_header, context_menu_item, context_menu_separator,
};
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, Margin, Stroke};
use egui_phosphor::regular::{
    ARROWS_CLOCKWISE, CARET_DOWN, CHECK, CHURCH, EYE, MONITOR, PENCIL_SIMPLE, PLAY, PLUS, TRASH,
    USER,
};

pub fn render_top_bar(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    slide_deck: &mut SlideDeck,
    is_presentation_open: &mut bool,
) {
    let language = state.settings.language;
    let mut pin_rect = None;
    ui.horizontal(|ui| {
        let mut remove_id = None;
        let mut select_item = None;
        let mut project_item = None;
        let mut add_to_playlist_item = None;
        let mut edit_song_item = None;
        let mut clear_all_shortcuts = false;
        let mut reset_layout = false;
        let mut reorder = None;

        // 1. "Atalhos" Header Label (Left)
        let header_resp = ui.add(
            egui::Label::new(
                egui::RichText::new(language.text("quick-title"))
                    .size(13.0)
                    .strong()
                    .color(state.theme.text_primary),
            )
            .sense(egui::Sense::click()),
        );
        header_resp.context_menu(|ui| {
            context_menu_header(ui);
            let theme = &state.theme;
            if context_menu_item(ui, ARROWS_CLOCKWISE, language.text("quick-reset"), theme)
                .clicked()
            {
                reset_layout = true;
                ui.close();
            }
            if !state.quick_bar.items.is_empty()
                && context_menu_danger_item(ui, TRASH, language.text("quick-clear"), theme).clicked()
            {
                clear_all_shortcuts = true;
                ui.close();
            }
        });
        ui.add_space(8.0);

        // 2. Right-side options cluster (operador, culto, telas web, status)
        let operators: Vec<(String, String)> = state
            .settings
            .operators
            .iter()
            .map(|p| (p.id.clone(), p.name.clone()))
            .collect();
        let services: Vec<(String, String)> = state
            .settings
            .services
            .iter()
            .map(|p| (p.id.clone(), p.display_name(language)))
            .collect();
        let active_operator = state.settings.active_operator.clone();
        let active_service = state.settings.active_service.clone();
        let operator_name = state.settings.operator().name.clone();
        let service_name = state.settings.service().display_name(language);
        let screen_count = state
            .settings
            .screens
            .iter()
            .filter(|s| s.enabled)
            .count();
        let web_label = if state.settings_ui.server_running {
            language.format("quick-web-screens", &[("count", &screen_count.to_string())])
        } else {
            language.text("quick-web-off").to_string()
        };
        let show_status = *is_presentation_open;
        let show_controls = *is_presentation_open && (!state.show_presentation_dock || state.active_sidebar_page==crate::core::state::SidebarPage::Design);
        let right_reserve = options_cluster_width(
            ui,
            &operator_name,
            &service_name,
            &web_label,
            show_status,
            show_controls,
        );

        // 3. Middle area: Horizontal ScrollArea for shortcut items
        let scroll_width = (ui.available_width() - right_reserve).max(0.0);
        pin_rect = Some(egui::Rect::from_min_size(header_resp.rect.min, egui::vec2((header_resp.rect.width() + scroll_width + 8.0).max(100.0), 32.0)));

        ui.allocate_ui_with_layout(
            egui::vec2(scroll_width, ui.available_height()),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                if state.quick_bar.items.is_empty() {
                    ui.label(
                        egui::RichText::new(language.text("quick-empty"))
                            .size(11.5)
                            .italics()
                            .color(state.theme.text_secondary.linear_multiply(0.6)),
                    );
                } else {
                    egui::ScrollArea::horizontal()
                        .id_salt("quick_items_top_scroll")
                        .auto_shrink([true, false])
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                for item in &state.quick_bar.items {
                                    let is_active = state.quick_bar.active_id.as_deref()
                                        == Some(&item.id);
                                    let icon = match item.kind {
                                        QuickItemKind::Song => egui_phosphor::regular::MUSIC_NOTE,
                                        QuickItemKind::Media => egui_phosphor::regular::VIDEO,
                                        QuickItemKind::Verse => egui_phosphor::regular::BOOK_OPEN,
                                        QuickItemKind::Text => egui_phosphor::regular::FILE_TEXT,
                                        QuickItemKind::Design => egui_phosphor::regular::STACK,
                                    };

                                    let bg_col = if is_active {
                                        state.theme.bg_card_active
                                    } else {
                                        state.theme.bg_pill
                                    };

                                    let border_stroke = if is_active {
                                        Stroke::new(1.0, state.theme.border_active)
                                    } else {
                                        Stroke::new(1.0, state.theme.border_default)
                                    };

                                    let item_frame = egui::Frame::new()
                                        .fill(bg_col)
                                        .stroke(border_stroke)
                                        .corner_radius(state.theme.radius_sm)
                                        .inner_margin(Margin::symmetric(8, 3));

                                    item_frame.show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = 4.0;

                                            let label_text = format!("{} {}", icon, item.title);
                                            let item_btn = egui::Button::new(
                                                egui::RichText::new(label_text)
                                                    .size(12.0)
                                                    .strong()
                                                    .color(if is_active {
                                                        state.theme.text_primary
                                                    } else {
                                                        state.theme.icon_default
                                                    }),
                                            )
                                            .frame(false).sense(egui::Sense::click_and_drag());

                                            let btn_resp = ui
                                                .add(item_btn)
                                                .on_hover_text(format!(
                                                    "{} - {}",
                                                    item.subtitle, language.text("workspace-click-to-preview-double-click-to-present-right-click-for")
                                                 ));

                                            super::drag_drop::source(ui, &btn_resp, || {
                                                let mut payload = super::drag_drop::DragPayload::new(super::drag_drop::DragSource::QuickBar(vec![item.id.clone()]), &item.title, &item.subtitle, icon);
                                                payload.items.push(item.clone());
                                                if item.kind == QuickItemKind::Design { payload.design = slide_deck.designs.iter().find(|scene| scene.id == item.content).cloned(); }
                                                payload
                                            });
                                            if let Some((payload, after)) = super::drag_drop::reorder_target(ui, btn_resp.rect, &state.theme, true, |source| matches!(source, super::drag_drop::DragSource::QuickBar(ids) if !ids.contains(&item.id)))
                                                && let super::drag_drop::DragSource::QuickBar(ids) = &payload.source { reorder = Some((ids.clone(), item.id.clone(), after)); }

                                            if btn_resp.clicked() {
                                                select_item = Some(item.clone());
                                            }
                                            if btn_resp.double_clicked() {
                                                project_item = Some(item.clone());
                                            }

                                            btn_resp.context_menu(|ui| {
                                                context_menu_header(ui);
                                                let theme = &state.theme;
                                                if context_menu_item(ui, PLAY, "workspace-present-now", theme).clicked() {
                                                    project_item = Some(item.clone());
                                                    ui.close();
                                                }
                                                if context_menu_item(ui, EYE, "workspace-preview", theme).clicked() {
                                                    select_item = Some(item.clone());
                                                    ui.close();
                                                }
                                                if context_menu_item(ui, PLUS, "workspace-add-to-playlist", theme).clicked() {
                                                    add_to_playlist_item = Some(item.clone());
                                                    ui.close();
                                                }
                                                if item.kind == QuickItemKind::Song
                                                    && context_menu_item(ui, PENCIL_SIMPLE, "workspace-edit-song", theme).clicked() {
                                                        edit_song_item = Some(item.clone());
                                                        ui.close();
                                                    }
                                                context_menu_separator(ui);
                                                if context_menu_danger_item(ui, TRASH, "workspace-unpin-item", theme).clicked() {
                                                    remove_id = Some(item.id.clone());
                                                    ui.close();
                                                }
                                            });

                                            let close_btn = egui::Button::new(
                                                egui::RichText::new(egui_phosphor::regular::X)
                                                    .size(11.0)
                                                    .color(state.theme.icon_muted),
                                            )
                                            .frame(false);

                                            if ui
                                                .add(close_btn)
                                                .on_hover_text(language.text("workspace-remove-from-bar"))
                                                .clicked()
                                            {
                                                remove_id = Some(item.id.clone());
                                            }
                                        });
                                    });

                                    ui.add_space(3.0);
                                }
                            });
                        });
                }
            },
        );

        // 4. Far right: operador, culto, telas web e status ao vivo
        let tokens = state.theme.clone();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            if show_status {
                let (label, fg, bg) = if slide_deck.is_content_hidden() {
                    (language.text("workspace-hidden"), tokens.status_warning_fg, tokens.status_warning_bg)
                } else {
                    (language.text("workspace-live-label"), tokens.status_success_fg, tokens.status_success_bg)
                };
                pill(ui, None, Some(fg), label, fg, bg);
            }

            if show_controls
                && tokens
                    .icon_button(ui, EYE, language.text("workspace-show-controls"), false, egui::vec2(28.0, 26.0))
                    .clicked()
                {
                    state.show_presentation_dock = true;
                    if state.active_sidebar_page==crate::core::state::SidebarPage::Design {state.active_sidebar_page=crate::core::state::SidebarPage::Home;}
                }

            screens_pill(ui, &tokens, &web_label, state.settings_ui.server_running);

            if let Some(id) = filter_pill(
                ui,
                &tokens,
                CHURCH,
                language.text("workspace-choose-service"),
                &active_service,
                &service_name,
                &services,
            ) {
                state.switch_service(&id);
            }

            if let Some(id) = filter_pill(
                ui,
                &tokens,
                USER,
                language.text("workspace-choose-operator"),
                &active_operator,
                &operator_name,
                &operators,
            ) {
                state.switch_operator(&id);
            }
        });

        if let Some((ids, target, after)) = reorder { crate::core::interaction::move_items(&mut state.quick_bar.items, &ids, &target, after, |item| &item.id); }

        // Handle layout reset
        if reset_layout {
            state.reset_containers();
        }

        // Handle clear all
        if clear_all_shortcuts {
            state.quick_bar.items.clear();
            state.quick_bar.active_id = None;
        }

        // Handle remove
        if let Some(id) = remove_id {
            state.quick_bar.items.retain(|it| it.id != id);
            if state.quick_bar.active_id.as_deref() == Some(&id) {
                state.quick_bar.active_id = state.quick_bar.items.first().map(|it| it.id.clone());
            }
        }

        // Handle add to playlist
        if let Some(item) = add_to_playlist_item {
            state.playlist.push(item.to_playlist());
        }

        // Handle edit song
        if let Some(item) = edit_song_item
            && let Some(s) = slide_deck.library().iter().find(|s| s.title == item.title) {
                state.song_editor.title = s.title.clone();
                state.song_editor.artist = s.artist.clone();
                state.song_editor.text = s.text.clone();
                state.song_editor.folder = s.folder.clone();
                state.song_editor.tags = s.tags.clone();
                state.song_editor.open = true;
            }

        // Handle project item
        if let Some(item) = project_item {
            state.quick_bar.active_id = Some(item.id.clone());
            let mut opens_projection=true;
            match item.kind {
                QuickItemKind::Design => {
                    if let Err(error) = slide_deck.preview_design_by_id(&item.content) {
                        state.notify(error); opens_projection = false;
                    } else { slide_deck.project_preview(); }
                }
                QuickItemKind::Song => {
                    if let Some(idx) = slide_deck
                        .library()
                        .iter()
                        .position(|s| s.title == item.title && s.artist == item.subtitle && (item.content.is_empty() || s.text == item.content))
                    {
                        slide_deck.project_song(idx);
                        slide_deck.select_preview_slide(0);
                    } else {
                        slide_deck.project_custom_content(
                            item.title.clone(),
                            item.subtitle.clone(),
                            item.content.clone(),
                        );
                    }
                }
                QuickItemKind::Verse => {
                    slide_deck.project_custom_content(
                        item.title.clone(),
                        item.subtitle.clone(),
                        item.content.clone(),
                    );
                }
                QuickItemKind::Media => {
                    match crate::services::media::activate(state,slide_deck,&item.title,&item.content,&item.subtitle,true){Ok(project)=>opens_projection=project,Err(error)=>{state.notify(error);opens_projection=false;}}
                }
                QuickItemKind::Text => {
                    slide_deck.project_custom_content(
                        item.title.clone(),
                        item.subtitle.clone(),
                        item.content.clone(),
                    );
                }
            }
            if opens_projection {
                if !slide_deck.preview_design.as_ref().is_some_and(|scene|scene.overlay) { state.apply_theme_to_live(); }
                *is_presentation_open = true;
                state.show_presentation_dock = state.settings.operator().show_dock;
            }
        }

        // Handle select (preview)
        if let Some(item) = select_item {
            state.quick_bar.active_id = Some(item.id.clone());
            match item.kind {
                QuickItemKind::Design => {
                    if let Err(error) = slide_deck.preview_design_by_id(&item.content) { state.notify(error); }
                }
                QuickItemKind::Song => {
                    if let Some(idx) = slide_deck
                        .library()
                        .iter()
                        .position(|s| s.title == item.title && s.artist == item.subtitle && (item.content.is_empty() || s.text == item.content))
                    {
                        slide_deck.select_song(idx);
                    } else {
                        slide_deck.load_custom_content(
                            item.title.clone(),
                            item.subtitle.clone(),
                            item.content.clone(),
                        );
                    }
                }
                QuickItemKind::Verse => {
                    slide_deck.load_custom_content(
                        item.title.clone(),
                        item.subtitle.clone(),
                        item.content.clone(),
                    );
                }
                QuickItemKind::Media => {
                    if let Err(error)=crate::services::media::activate(state,slide_deck,&item.title,&item.content,&item.subtitle,false){state.notify(error);}
                }
                QuickItemKind::Text => {
                    slide_deck.load_custom_content(
                        item.title.clone(),
                        item.subtitle.clone(),
                        item.content.clone(),
                    );
                }
            }
        }
    });
    if !egui::DragAndDrop::payload::<super::drag_drop::DragPayload>(ui.ctx())
        .is_some_and(|payload| matches!(payload.source, super::drag_drop::DragSource::QuickBar(_)))
        && let Some(payload) = pin_rect.and_then(|rect| {
            super::drag_drop::content_target(ui, rect, &state.theme, "drag-drop-pin")
        }) {
            if let super::drag_drop::DragSource::ChatMessage(id) = payload.source
                && let Some(service) = &state.chat.service
                    && let Err(error) = service.request_action(&state.chat.token, id, "shortcut") {
                        state.chat.error = error;
                    }
            for item in &payload.items {
                state.add_quick_item(
                    item.title.clone(),
                    item.subtitle.clone(),
                    item.kind.clone(),
                    item.content.clone(),
                );
            }
        }
}

const PILL_H: f32 = 26.0;
const PILL_PAD: f32 = 10.0;
const PILL_ICON_W: f32 = 15.0;
const PILL_GAP: f32 = 6.0;
const PILL_CARET_W: f32 = 12.0;
const PILL_DOT_W: f32 = 8.0;
const PILL_GAP_X: f32 = 8.0;

fn label_width(ui: &egui::Ui, text: &str) -> f32 {
    let font = egui::FontId::proportional(12.5);
    ui.painter()
        .layout_no_wrap(text.to_owned(), font, egui::Color32::WHITE)
        .size()
        .x
}

fn filter_width(ui: &egui::Ui, text: &str) -> f32 {
    PILL_PAD * 2.0 + PILL_ICON_W + PILL_GAP + label_width(ui, text) + PILL_GAP + PILL_CARET_W
}

fn simple_pill_width(ui: &egui::Ui, text: &str, dot: bool) -> f32 {
    let lead = if dot { PILL_DOT_W } else { PILL_ICON_W };
    PILL_PAD * 2.0 + lead + PILL_GAP + label_width(ui, text)
}

fn options_cluster_width(
    ui: &egui::Ui,
    operator: &str,
    service: &str,
    web: &str,
    show_status: bool,
    show_controls: bool,
) -> f32 {
    let mut width = filter_width(ui, operator)
        + PILL_GAP_X
        + filter_width(ui, service)
        + PILL_GAP_X
        + simple_pill_width(ui, web, false)
        + PILL_GAP_X;
    if show_controls {
        width += 28.0 + PILL_GAP_X;
    }
    if show_status {
        width += simple_pill_width(ui, crate::ui::i18n::language(ui.ctx()).text("workspace-live-label"), true) + PILL_GAP_X;
    }
    width + 8.0
}

/// Paints a non-interactive informational pill with an optional icon or dot.
fn pill(
    ui: &mut egui::Ui,
    icon: Option<&str>,
    dot: Option<Color32>,
    text: &str,
    fg: Color32,
    bg: Color32,
) {
    let lead = if dot.is_some() {
        PILL_DOT_W
    } else {
        PILL_ICON_W
    };
    let width = PILL_PAD * 2.0 + lead + PILL_GAP + label_width(ui, text);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, PILL_H), egui::Sense::hover());
    ui.painter().rect_filled(rect, 8.0, bg);
    ui.painter().rect_stroke(
        rect,
        8.0,
        Stroke::new(1.0, fg.linear_multiply(0.25)),
        egui::StrokeKind::Inside,
    );

    let mut x = rect.min.x + PILL_PAD;
    if let Some(color) = dot {
        ui.painter().circle_filled(
            egui::pos2(x + PILL_DOT_W * 0.5, rect.center().y),
            PILL_DOT_W * 0.42,
            color,
        );
        x += PILL_DOT_W + PILL_GAP;
    } else if let Some(icon) = icon {
        ui.painter().text(
            egui::pos2(x + PILL_ICON_W * 0.5, rect.center().y),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(14.0),
            fg,
        );
        x += PILL_ICON_W + PILL_GAP;
    }
    ui.painter().text(
        egui::pos2(x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(12.5),
        fg,
    );
}

fn screens_pill(ui: &mut egui::Ui, tokens: &AppTheme, label: &str, running: bool) {
    let (fg, bg) = if running {
        (tokens.status_success_fg, tokens.status_success_bg)
    } else {
        (tokens.text_muted, tokens.bg_pill)
    };
    pill(ui, Some(MONITOR), None, label, fg, bg);
}

/// A pill-shaped dropdown used for the operator and service selectors.
fn filter_pill(
    ui: &mut egui::Ui,
    tokens: &AppTheme,
    icon: &str,
    tooltip: &str,
    active_id: &str,
    active_name: &str,
    options: &[(String, String)],
) -> Option<String> {
    let width = filter_width(ui, active_name);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, PILL_H), egui::Sense::click());
    let popup_id = response.id.with("popup");
    let open = egui::Popup::is_id_open(ui.ctx(), popup_id);
    let hover = ui
        .ctx()
        .animate_bool(response.id.with("hover"), response.hovered());
    let bg = tokens.bg_pill.lerp_to_gamma(tokens.bg_card_hover, hover);
    let border = if open {
        tokens.accent
    } else {
        tokens.border_default
    };
    ui.painter().rect_filled(rect, 8.0, bg);
    ui.painter().rect_stroke(
        rect,
        8.0,
        Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        egui::pos2(rect.min.x + PILL_PAD + PILL_ICON_W * 0.5, rect.center().y),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(14.0),
        tokens.icon_default,
    );
    ui.painter().text(
        egui::pos2(
            rect.min.x + PILL_PAD + PILL_ICON_W + PILL_GAP,
            rect.center().y,
        ),
        egui::Align2::LEFT_CENTER,
        active_name,
        egui::FontId::proportional(12.5),
        tokens.text_primary,
    );
    ui.painter().text(
        egui::pos2(rect.max.x - PILL_PAD - PILL_CARET_W * 0.5, rect.center().y),
        egui::Align2::CENTER_CENTER,
        CARET_DOWN,
        egui::FontId::proportional(11.0),
        tokens.icon_muted,
    );
    let response = response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tooltip);

    let row_width = options
        .iter()
        .map(|(_, name)| label_width(ui, name))
        .fold(label_width(ui, active_name), f32::max)
        + 52.0;

    let mut chosen = None;
    egui::Popup::menu(&response)
        .id(popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.set_min_width(row_width);
            ui.spacing_mut().item_spacing.y = 2.0;
            for (id, name) in options {
                if menu_option(ui, tokens, name, id == active_id, row_width) {
                    chosen = Some(id.clone());
                }
            }
        });
    chosen
}

fn menu_option(
    ui: &mut egui::Ui,
    tokens: &AppTheme,
    label: &str,
    selected: bool,
    row_width: f32,
) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(row_width - 8.0, 28.0), egui::Sense::click());
    let hover = ui
        .ctx()
        .animate_bool(response.id.with("hover"), response.hovered());
    let highlight = if selected {
        tokens.accent_soft
    } else {
        tokens.bg_pill_active.gamma_multiply(hover)
    };
    if selected || hover > 0.0 {
        ui.painter().rect_filled(rect, 6.0, highlight);
    }
    let text_color = if selected {
        tokens.accent
    } else {
        tokens.text_primary
    };
    ui.painter().text(
        egui::pos2(rect.min.x + 10.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.5),
        text_color,
    );
    if selected {
        ui.painter().text(
            egui::pos2(rect.max.x - 12.0, rect.center().y),
            egui::Align2::CENTER_CENTER,
            CHECK,
            egui::FontId::proportional(12.0),
            tokens.accent,
        );
    }
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_bar_rendering_with_shortcuts() {
        let mut state = ControlPanelState::default();
        state.add_quick_item(
            "Initial Song".to_string(),
            "Artist".to_string(),
            QuickItemKind::Song,
            "Song lyrics".to_string(),
        );
        let mut slide_deck = SlideDeck::default();
        let mut is_presentation_open = false;
        let ctx = egui::Context::default();

        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(1280.0, 40.0),
            )),
            ..Default::default()
        };

        let mut output = ctx.run_ui(raw_input.clone(), |ui| {
            render_top_bar(ui, &mut state, &mut slide_deck, &mut is_presentation_open);
        });
        output.textures_delta.clear();

        assert!(!state.quick_bar.items.is_empty());

        // Test adding a shortcut
        state.add_quick_item(
            "New Test Song".to_string(),
            "Test Artist".to_string(),
            QuickItemKind::Song,
            "Song lyrics".to_string(),
        );

        let count = state.quick_bar.items.len();

        // Adding identical shortcut should deduplicate
        state.add_quick_item(
            "New Test Song".to_string(),
            "Test Artist".to_string(),
            QuickItemKind::Song,
            "Song lyrics".to_string(),
        );
        assert_eq!(state.quick_bar.items.len(), count);

        let mut output2 = ctx.run_ui(raw_input, |ui| {
            render_top_bar(ui, &mut state, &mut slide_deck, &mut is_presentation_open);
        });
        output2.textures_delta.clear();
    }
}
