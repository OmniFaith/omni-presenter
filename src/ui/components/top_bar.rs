use crate::core::models::QuickItemKind;
use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use crate::ui::components::{
    context_menu_danger_item, context_menu_header, context_menu_item, context_menu_separator,
};
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, Margin, Stroke};
use egui_phosphor::regular::{
    ARROWS_CLOCKWISE, CARET_DOWN, EYE, MONITOR, PENCIL_SIMPLE, PLAY, PLUS,
    PUSH_PIN, TRASH, USER,
};

pub fn render_top_bar(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    slide_deck: &mut SlideDeck,
    is_presentation_open: &mut bool,
) {
    let language = state.settings.language;
    let mut pin_rect = None;
    let is_dragging = egui::DragAndDrop::payload::<super::drag_drop::DragPayload>(ui.ctx())
        .is_some_and(|payload| {
            !matches!(payload.source, super::drag_drop::DragSource::QuickBar(_))
                && (!payload.items.is_empty()
                    || matches!(payload.source, super::drag_drop::DragSource::ChatMessage(_)))
        });

    let bar_height = 30.0;
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), bar_height),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
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
                        .size(12.5)
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
                    && context_menu_danger_item(ui, TRASH, language.text("quick-clear"), theme)
                        .clicked()
                {
                    clear_all_shortcuts = true;
                    ui.close();
                }
            });
            ui.add_space(8.0);

        // 2. Right-side options cluster (operador, telas web, status)
        let operator_name = state.settings.operator().name.clone();
        let operator_avatar = state.settings.operator().avatar.clone();
        let screen_count = state
            .settings
            .screens
            .iter()
            .filter(|s| s.enabled)
            .count();
        let show_status = *is_presentation_open;
        let show_controls = *is_presentation_open && (!state.show_presentation_dock || state.active_sidebar_page==crate::core::state::SidebarPage::Design);
        let right_reserve = options_cluster_width(
            ui,
            &operator_name,
            operator_avatar.is_some(),
            screen_count,
            state.settings_ui.server_running,
            show_status,
            show_controls,
        );

        // 3. Middle area: Horizontal ScrollArea for shortcut items
        let scroll_width = (ui.available_width() - right_reserve).max(0.0);
        let items_start_x = header_resp.rect.max.x + 8.0;
        let pin_zone_rect = egui::Rect::from_min_size(
            egui::pos2(items_start_x, header_resp.rect.center().y - bar_height * 0.5),
            egui::vec2(scroll_width.max(80.0), bar_height),
        );
        pin_rect = Some(pin_zone_rect);

        let is_hovering_drop = is_dragging
            && ui
                .ctx()
                .pointer_interact_pos()
                .is_some_and(|p| pin_zone_rect.contains(p));

        ui.allocate_ui_with_layout(
            egui::vec2(scroll_width, bar_height),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                if state.quick_bar.items.is_empty() {
                    if is_dragging {
                        let drop_frame = egui::Frame::new()
                            .fill(if is_hovering_drop {
                                state.theme.accent_soft
                            } else {
                                state.theme.bg_pill
                            })
                            .stroke(Stroke::new(1.0, state.theme.accent))
                            .corner_radius(state.theme.radius_sm)
                            .inner_margin(Margin::symmetric(10, 4));
                        drop_frame.show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "{PUSH_PIN}  {}",
                                    language.text("chat-pin-to-quick-bar")
                                ))
                                .size(11.5)
                                .strong()
                                .color(state.theme.accent),
                            );
                        });
                    } else {
                        ui.label(
                            egui::RichText::new(language.text("quick-empty"))
                                .size(11.5)
                                .italics()
                                .color(state.theme.text_secondary.linear_multiply(0.6)),
                        );
                    }
                } else {
                    egui::ScrollArea::horizontal()
                        .id_salt("quick_items_top_scroll")
                        .auto_shrink([true, false])
                        .show(ui, |ui| {
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                for item in &state.quick_bar.items {
                                    let is_active = state.quick_bar.active_id.as_deref()
                                        == Some(&item.id);
                                    let icon = match item.kind {
                                        QuickItemKind::Song => egui_phosphor::regular::MUSIC_NOTE,
                                        QuickItemKind::Media => egui_phosphor::regular::VIDEO,
                                        QuickItemKind::Verse => egui_phosphor::regular::BOOK_OPEN,
                                        QuickItemKind::Text => egui_phosphor::regular::FILE_TEXT,
                                        QuickItemKind::Design => super::ICON_DESIGN,
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
                                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                            ui.spacing_mut().item_spacing.x = 4.0;

                                            let label_text = format!("{} {}", icon, item.title);
                                            let item_btn = egui::Button::new(
                                                egui::RichText::new(label_text)
                                                    .size(11.5)
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

                                if is_dragging {
                                    let drop_frame = egui::Frame::new()
                                        .fill(if is_hovering_drop {
                                            state.theme.accent_soft
                                        } else {
                                            state.theme.bg_pill
                                        })
                                        .stroke(Stroke::new(1.0, state.theme.accent))
                                        .corner_radius(state.theme.radius_sm)
                                        .inner_margin(Margin::symmetric(8, 3));
                                    drop_frame.show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{PUSH_PIN}  {}",
                                                language.text("chat-pin-to-quick-bar")
                                            ))
                                            .size(11.0)
                                            .strong()
                                            .color(state.theme.accent),
                                        );
                                    });
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

            if screens_pill(ui, &tokens, screen_count, state.settings_ui.server_running) {
                state.show_web_hub_modal = true;
            }

            if operator_pill(ui, &tokens, operator_avatar.as_deref(), &operator_name) {
                state.show_profile_picker_modal = true;
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
    if is_dragging && let Some(rect) = pin_rect {
        let is_hovered = ui
            .ctx()
            .pointer_interact_pos()
            .is_some_and(|p| rect.contains(p));
        if is_hovered {
            ui.painter().rect_filled(
                rect,
                state.theme.radius_sm,
                state.theme.accent.linear_multiply(0.08),
            );
            ui.painter().rect_stroke(
                rect,
                state.theme.radius_sm,
                egui::Stroke::new(1.0, state.theme.accent.linear_multiply(0.4)),
                egui::StrokeKind::Inside,
            );
            if ui.input(|i| i.pointer.any_released()) {
                if let Some(payload) =
                    egui::DragAndDrop::payload::<super::drag_drop::DragPayload>(ui.ctx())
                {
                    if let super::drag_drop::DragSource::ChatMessage(id) = payload.source
                        && let Some(service) = &state.chat.service
                        && let Err(error) =
                            service.request_action(&state.chat.token, id, "shortcut")
                    {
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
                    egui::DragAndDrop::clear_payload(ui.ctx());
                }
            }
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

fn simple_pill_width(ui: &egui::Ui, text: &str, dot: bool) -> f32 {
    let lead = if dot { PILL_DOT_W } else { PILL_ICON_W };
    PILL_PAD * 2.0 + lead + PILL_GAP + label_width(ui, text)
}

fn options_cluster_width(
    ui: &egui::Ui,
    operator: &str,
    has_avatar: bool,
    screen_count: usize,
    running: bool,
    show_status: bool,
    show_controls: bool,
) -> f32 {
    let lead_w = if has_avatar { 20.0 } else { PILL_ICON_W };
    let operator_w = PILL_PAD * 2.0 + lead_w + PILL_GAP + label_width(ui, operator) + PILL_GAP + PILL_CARET_W;
    let count_text = if running { screen_count.to_string() } else { "0".to_string() };
    let web_w = PILL_PAD * 2.0 + PILL_ICON_W + PILL_GAP + label_width(ui, &count_text);

    let mut width = operator_w + PILL_GAP_X + web_w + PILL_GAP_X;
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

fn screens_pill(
    ui: &mut egui::Ui,
    tokens: &AppTheme,
    screen_count: usize,
    running: bool,
) -> bool {
    let count_text = if running {
        screen_count.to_string()
    } else {
        "0".to_string()
    };
    let width = PILL_PAD * 2.0 + PILL_ICON_W + PILL_GAP + label_width(ui, &count_text);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, PILL_H), egui::Sense::click());
    let hover = ui
        .ctx()
        .animate_bool(response.id.with("hover"), response.hovered());
    let (fg, bg) = if running {
        (
            tokens.status_success_fg,
            tokens.status_success_bg.lerp_to_gamma(tokens.bg_card_hover, hover * 0.4),
        )
    } else {
        (
            tokens.text_muted,
            tokens.bg_pill.lerp_to_gamma(tokens.bg_card_hover, hover),
        )
    };
    let border = if response.hovered() {
        tokens.accent
    } else {
        fg.linear_multiply(0.3)
    };
    ui.painter().rect_filled(rect, 8.0, bg);
    ui.painter().rect_stroke(
        rect,
        8.0,
        Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );

    let x = rect.min.x + PILL_PAD;
    ui.painter().text(
        egui::pos2(x + PILL_ICON_W * 0.5, rect.center().y),
        egui::Align2::CENTER_CENTER,
        MONITOR,
        egui::FontId::proportional(14.0),
        fg,
    );
    ui.painter().text(
        egui::pos2(x + PILL_ICON_W + PILL_GAP, rect.center().y),
        egui::Align2::LEFT_CENTER,
        &count_text,
        egui::FontId::proportional(12.5),
        fg,
    );

    let tooltip = if running {
        format!("{screen_count} telas web · Clique para Conexões Web")
    } else {
        "Web desativado · Clique para Conexões Web".to_string()
    };

    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tooltip)
        .clicked()
}

fn operator_pill(
    ui: &mut egui::Ui,
    tokens: &AppTheme,
    avatar: Option<&[u8]>,
    operator_name: &str,
) -> bool {
    let has_avatar = avatar.is_some();
    let lead_w = if has_avatar { 20.0 } else { PILL_ICON_W };
    let width = PILL_PAD * 2.0 + lead_w + PILL_GAP + label_width(ui, operator_name) + PILL_GAP + PILL_CARET_W;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, PILL_H), egui::Sense::click());
    let hover = ui
        .ctx()
        .animate_bool(response.id.with("hover"), response.hovered());
    let bg = tokens.bg_pill.lerp_to_gamma(tokens.bg_card_hover, hover);
    let border = if response.hovered() {
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

    let mut x = rect.min.x + PILL_PAD;
    if let Some(bytes) = avatar {
        let avatar_size = 18.0;
        let avatar_rect = egui::Rect::from_center_size(
            egui::pos2(x + avatar_size * 0.5, rect.center().y),
            egui::vec2(avatar_size, avatar_size),
        );
        crate::ui::components::avatar::paint(
            ui,
            avatar_rect,
            Some(bytes),
            USER,
            tokens,
            false,
        );
        x += avatar_size + PILL_GAP;
    } else {
        ui.painter().text(
            egui::pos2(x + PILL_ICON_W * 0.5, rect.center().y),
            egui::Align2::CENTER_CENTER,
            USER,
            egui::FontId::proportional(14.0),
            tokens.icon_default,
        );
        x += PILL_ICON_W + PILL_GAP;
    }

    ui.painter().text(
        egui::pos2(x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        operator_name,
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

    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(crate::ui::i18n::language(ui.ctx()).text("workspace-choose-operator"))
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
