pub mod components;
pub mod containers;
pub mod i18n;
pub mod modals;
pub mod pages;
pub mod presentation;
pub mod presentation_assets;
pub mod presentation_transition;
pub mod theme;

use crate::core::layout::{ContainerKind, LayoutNode, SplitDirection};
use crate::core::slides::SlideDeck;
use crate::core::state::ControlPanelState;
use crate::services::db::AppDb;
use eframe::egui;

pub use components::*;
pub use containers::*;
pub use modals::*;
pub use pages::*;
pub use theme::AppTheme;

/// Renders the full dynamic docking layout tree
pub fn render_layout(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    slide_deck: &mut SlideDeck,
    is_presentation_open: &mut bool,
    rendered_leaf_rects: &mut Vec<(ContainerKind, egui::Rect)>,
    db: Option<&AppDb>,
) {
    let avail_rect = ui.available_rect_before_wrap().shrink(4.0);
    let bible = state.active_sidebar_page == crate::core::state::SidebarPage::Bible;
    let mut root = if bible {
        state.bible_layout.root.clone()
    } else {
        state.layout.root.clone()
    };
    render_node(
        ui,
        &mut root,
        avail_rect,
        "root",
        state,
        slide_deck,
        is_presentation_open,
        rendered_leaf_rects,
        db,
    );

    if bible {
        state.bible_layout.root = root;
    } else {
        state.layout.root = root;
    }
}

pub fn render_node(
    ui: &mut egui::Ui,
    node: &mut LayoutNode,
    rect: egui::Rect,
    path: &str,
    state: &mut ControlPanelState,
    slide_deck: &mut SlideDeck,
    is_presentation_open: &mut bool,
    rendered_leaf_rects: &mut Vec<(ContainerKind, egui::Rect)>,
    db: Option<&AppDb>,
) {
    if rect.width() <= 10.0 || rect.height() <= 10.0 {
        return;
    }

    match node {
        LayoutNode::Leaf(kind) => {
            let inner_rect = rect.shrink(2.0);
            rendered_leaf_rects.push((*kind, inner_rect));
            let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(inner_rect));
            child_ui.set_clip_rect(rect);
            state.theme.container_frame().show(&mut child_ui, |ui| {
                ui.set_min_height(ui.available_height());
                ui.set_max_height(ui.available_height());
                render_container(ui, *kind, state, slide_deck, is_presentation_open, db);
            });
        }
        LayoutNode::Split {
            direction,
            fraction,
            first,
            second,
        } => {
            let splitter_size = LayoutNode::SPLITTER_SIZE;

            match direction {
                SplitDirection::Horizontal => {
                    let total_w = rect.width();
                    let min_w1 = first.min_width();
                    let min_w2 = second.min_width();
                    let needed = min_w1 + min_w2 + splitter_size;

                    let (min_frac, max_frac) = if total_w > needed {
                        (
                            min_w1 / total_w,
                            (total_w - min_w2 - splitter_size) / total_w,
                        )
                    } else {
                        let ratio = min_w1 / (min_w1 + min_w2);
                        (ratio, ratio)
                    };

                    *fraction = (*fraction).clamp(min_frac, max_frac);
                    let w1 = (*fraction * total_w)
                        .round()
                        .clamp(total_w * min_frac, total_w * max_frac);
                    let splitter_line_x = (rect.min.x + w1).round();

                    let rect1 =
                        egui::Rect::from_min_max(rect.min, egui::pos2(splitter_line_x, rect.max.y));
                    let rect2 = egui::Rect::from_min_max(
                        egui::pos2(splitter_line_x + splitter_size, rect.min.y),
                        rect.max,
                    );

                    // Recurse children FIRST
                    render_node(
                        ui,
                        first,
                        rect1,
                        &format!("{}_1", path),
                        state,
                        slide_deck,
                        is_presentation_open,
                        rendered_leaf_rects,
                        db,
                    );
                    render_node(
                        ui,
                        second,
                        rect2,
                        &format!("{}_2", path),
                        state,
                        slide_deck,
                        is_presentation_open,
                        rendered_leaf_rects,
                        db,
                    );

                    // Splitter interaction AFTER children (so it's on top of children in egui's hit-testing!)
                    let hit_rect = egui::Rect::from_center_size(
                        egui::pos2(splitter_line_x + splitter_size * 0.5, rect.center().y),
                        egui::vec2(10.0, rect.height()),
                    );
                    let splitter_id = ui.scope_id().with(("splitter", path));
                    let splitter_resp = ui.interact(hit_rect, splitter_id, egui::Sense::drag());

                    if splitter_resp.hovered() || splitter_resp.dragged() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                    }
                    if splitter_resp.dragged() {
                        let delta = splitter_resp.drag_delta().x;
                        if total_w > 0.0 {
                            *fraction = (*fraction + delta / total_w).clamp(min_frac, max_frac);
                            ui.ctx().request_repaint();
                        }
                    }

                    if splitter_resp.hovered() || splitter_resp.dragged() {
                        draw_splitter_handle(
                            ui,
                            &state.theme,
                            splitter_line_x + splitter_size * 0.5,
                            rect,
                            true,
                            splitter_resp.dragged(),
                        );
                    }
                }
                SplitDirection::Vertical => {
                    let total_h = rect.height();
                    let min_h1 = first.min_height();
                    let min_h2 = second.min_height();
                    let needed = min_h1 + min_h2 + splitter_size;

                    let (min_frac, max_frac) = if total_h > needed {
                        (
                            min_h1 / total_h,
                            (total_h - min_h2 - splitter_size) / total_h,
                        )
                    } else {
                        let ratio = min_h1 / (min_h1 + min_h2);
                        (ratio, ratio)
                    };

                    *fraction = (*fraction).clamp(min_frac, max_frac);
                    let h1 = (*fraction * total_h)
                        .round()
                        .clamp(total_h * min_frac, total_h * max_frac);
                    let splitter_line_y = (rect.min.y + h1).round();

                    let rect1 =
                        egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, splitter_line_y));
                    let rect2 = egui::Rect::from_min_max(
                        egui::pos2(rect.min.x, splitter_line_y + splitter_size),
                        rect.max,
                    );

                    render_node(
                        ui,
                        first,
                        rect1,
                        &format!("{}_1", path),
                        state,
                        slide_deck,
                        is_presentation_open,
                        rendered_leaf_rects,
                        db,
                    );
                    render_node(
                        ui,
                        second,
                        rect2,
                        &format!("{}_2", path),
                        state,
                        slide_deck,
                        is_presentation_open,
                        rendered_leaf_rects,
                        db,
                    );

                    let hit_rect = egui::Rect::from_center_size(
                        egui::pos2(rect.center().x, splitter_line_y + splitter_size * 0.5),
                        egui::vec2(rect.width(), 10.0),
                    );
                    let splitter_id = ui.scope_id().with(("splitter", path));
                    let splitter_resp = ui.interact(hit_rect, splitter_id, egui::Sense::drag());

                    if splitter_resp.hovered() || splitter_resp.dragged() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
                    }
                    if splitter_resp.dragged() {
                        let delta = splitter_resp.drag_delta().y;
                        if total_h > 0.0 {
                            *fraction = (*fraction + delta / total_h).clamp(min_frac, max_frac);
                            ui.ctx().request_repaint();
                        }
                    }

                    if splitter_resp.hovered() || splitter_resp.dragged() {
                        draw_splitter_handle(
                            ui,
                            &state.theme,
                            splitter_line_y + splitter_size * 0.5,
                            rect,
                            false,
                            splitter_resp.dragged(),
                        );
                    }
                }
            }
        }
    }
}

fn draw_splitter_handle(
    ui: &egui::Ui,
    theme: &AppTheme,
    line: f32,
    rect: egui::Rect,
    vertical_line: bool,
    dragged: bool,
) {
    let painter = ui.painter();
    let accent = theme.accent;
    let soft = egui::Color32::from_rgba_unmultiplied(
        accent.r(),
        accent.g(),
        accent.b(),
        if dragged { 110 } else { 70 },
    );
    if vertical_line {
        let x = line.round();
        painter.line_segment(
            [
                egui::pos2(x, rect.min.y + 10.0),
                egui::pos2(x, rect.max.y - 10.0),
            ],
            egui::Stroke::new(if dragged { 2.0 } else { 1.0 }, soft),
        );
        let grip = egui::Rect::from_center_size(
            egui::pos2(x, rect.center().y),
            egui::vec2(4.0, if dragged { 46.0 } else { 36.0 }),
        );
        painter.rect_filled(grip, 2, accent);
    } else {
        let y = line.round();
        painter.line_segment(
            [
                egui::pos2(rect.min.x + 10.0, y),
                egui::pos2(rect.max.x - 10.0, y),
            ],
            egui::Stroke::new(if dragged { 2.0 } else { 1.0 }, soft),
        );
        let grip = egui::Rect::from_center_size(
            egui::pos2(rect.center().x, y),
            egui::vec2(if dragged { 46.0 } else { 36.0 }, 4.0),
        );
        painter.rect_filled(grip, 2, accent);
    }
}

/// Renders container for the specified kind
pub fn render_container(
    ui: &mut egui::Ui,
    kind: ContainerKind,
    state: &mut ControlPanelState,
    slide_deck: &mut SlideDeck,
    is_presentation_open: &mut bool,
    db: Option<&AppDb>,
) {
    if state.active_sidebar_page == crate::core::state::SidebarPage::Bible {
        match kind {
            ContainerKind::Selection => {
                pages::bible::render_bible_navigation(ui, state, slide_deck);
                return;
            }
            ContainerKind::Lyrics => {
                pages::bible::render_bible_text(ui, state, slide_deck, is_presentation_open);
                return;
            }
            _ => {}
        }
    }
    match kind {
        ContainerKind::Selection => {
            render_selection_column(ui, state, slide_deck, is_presentation_open, db);
        }
        ContainerKind::Lyrics => {
            render_lyrics_column(ui, state, slide_deck, is_presentation_open, db);
        }
        ContainerKind::Stage => {
            render_stage_column(ui, state, slide_deck, is_presentation_open);
        }
        ContainerKind::Themes => {
            let previous_live = state.live_theme_id.clone();
            let applied = render_themes_column(ui, state, db);
            let active_theme_id = state.preview_theme().id.clone();
            if let Some(scene) = &mut slide_deck.preview_design
                && !scene.overlay
                && scene.background_mode != "color"
            {
                scene.background_mode = "theme".into();
                scene.theme_id = Some(active_theme_id);
            }
            if let Some(scene) = &mut slide_deck.live_design
                && !scene.overlay
                && scene.background_mode != "color"
            {
                scene.background_mode = "theme".into();
                if applied.1 || previous_live != state.live_theme_id || scene.theme_id.is_none() {
                    scene.theme_id = Some(state.live_theme_id.clone());
                }
            }
        }
        ContainerKind::Playlist => {
            render_playlist_column(ui, state, slide_deck, is_presentation_open, db);
        }
        ContainerKind::Preview => {
            render_preview_column(ui, state, slide_deck, is_presentation_open);
        }
        ContainerKind::Player => {
            render_player_column(ui, state, slide_deck, is_presentation_open);
        }
        ContainerKind::ChatChannels => {
            pages::chat::render_chat_channels(ui, state);
        }
        ContainerKind::ChatMessages => {
            pages::chat::render_chat_messages(ui, state);
        }
    }
}
