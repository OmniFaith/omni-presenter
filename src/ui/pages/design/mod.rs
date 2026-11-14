mod canvas;
mod canvas_interaction;
mod geometry;
mod feedback;
mod menus;
pub mod controls;
mod inspector;
pub mod library;
mod sidebar;
mod slide_controls;
mod theme_preview;
mod aspect_ratio;
mod toolbar;
mod inspector_text;
mod inspector_appearance;
mod inspector_transition;
pub mod animation_bar;
pub(super) mod animation_timeline;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod interaction_tests;

use crate::core::{
    design::{DesignDocument, DesignLayer, LayerContent, new_id},
    slides::SlideDeck,
    state::ControlPanelState,
};
use crate::services::db::AppDb;
use crate::ui::draw_splitter_handle;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, RichText, Vec2};
use egui_phosphor::regular::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimelineMode {
    #[default]
    Layers,
    Slides,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimelineDragHandle {
    Start,
    End,
    Move,
}

#[derive(Clone, Copy, Debug)]
pub struct TimelineDrag {
    pub layer_index: usize,
    pub handle: TimelineDragHandle,
    pub initial_pointer_x: f32,
    pub initial_start_ms: u32,
    pub initial_end_ms: u32,
}

pub struct DesignEditor {
    language: crate::core::i18n::Language,
    plugins: Vec<crate::core::plugins::InstalledPlugin>,
    pub scene: Option<DesignDocument>,
    pub selected: Option<usize>,
    pub selected_layers: std::collections::BTreeSet<usize>,
    hovered_layer: Option<usize>,
    inspector_tab: usize,
    undo: Vec<DesignDocument>,
    redo: Vec<DesignDocument>,
    pending_since: Option<std::time::Instant>,
    drag: Option<canvas::Drag>,
    pub disable_snap: bool,
    pub dense_cards: bool,
    pub error: String,
    left_width: f32,
    right_width: f32,
    saved: bool,
    confirm_delete: bool,
    rename_in_progress: bool,
    layer_inspector_tab: Option<usize>,
    image_import: Option<(String, Option<String>, crate::services::tasks::BackgroundTask<Option<String>>)>,
    pub preview_theme_id: Option<String>,
    pub theme_search: String,
    pub preview_aspect_ratio: Option<f32>,
    pub animation_playing: bool,
    pub animation_started_at: Option<std::time::Instant>,
    pub animation_time_ms: u64,
    pub animation_loop: bool,
    pub show_filmstrip: bool,
    pub timeline_mode: TimelineMode,
    pub timeline_drag: Option<TimelineDrag>,
    pub timeline_height: f32,
}

impl Default for DesignEditor {
    fn default() -> Self {
        Self {
            language: Default::default(),
            plugins: Vec::new(),
            scene: None,
            selected: None,
            selected_layers: std::collections::BTreeSet::new(),
            hovered_layer: None,
            inspector_tab: 0,
            undo: Vec::new(),
            redo: Vec::new(),
            pending_since: None,
            drag: None,
            disable_snap: false,
            dense_cards: true,
            error: String::new(),
            left_width: 220.0,
            right_width: 285.0,
            saved: true,
            confirm_delete: false,
            rename_in_progress: false,
            layer_inspector_tab: None,
            image_import: None,
            preview_theme_id: None,
            theme_search: String::new(),
            preview_aspect_ratio: None,
            animation_playing: false,
            animation_started_at: None,
            animation_time_ms: 0,
            animation_loop: true,
            show_filmstrip: true,
            timeline_mode: TimelineMode::Layers,
            timeline_drag: None,
            timeline_height: 140.0,
        }
    }
}

impl DesignEditor {
    pub fn execute_edit(&mut self, command: crate::core::commands::Command) {
        use crate::core::commands::Command;
        match command {
            Command::UndoDesign => self.history(false),
            Command::RedoDesign => self.history(true),
            Command::DuplicateLayer | Command::DeleteLayer => {
                let before = self.scene.clone();
                if command == Command::DuplicateLayer {
                    self.duplicate_layer();
                } else {
                    self.delete_layer();
                }
                if let Some(before) = before {
                    self.changed(before, true);
                }
            }
            Command::GroupLayers => self.group_selected(),
            Command::UngroupLayers => self.ungroup_selected(),
            _ => {}
        }
    }

    pub fn select_layer(&mut self, index: usize, additive: bool) {
        let Some(scene) = &self.scene else { return; };
        if index >= scene.layers.len() { return; }
        if additive {
            if self.selected_layers.contains(&index) {
                self.selected_layers.remove(&index);
                self.selected = self.selected_layers.iter().next_back().copied();
            } else {
                self.selected_layers.insert(index);
                self.selected = Some(index);
            }
        } else {
            self.selected_layers.clear();
            self.selected = Some(index);
            if let Some(gid) = scene.layers[index].group_id.as_deref() {
                for (i, layer) in scene.layers.iter().enumerate() {
                    if layer.group_id.as_deref() == Some(gid) {
                        self.selected_layers.insert(i);
                    }
                }
            } else {
                self.selected_layers.insert(index);
            }
        }
    }

    pub fn select_layer_range(&mut self, target: usize) {
        let Some(scene) = &self.scene else { return; };
        if target >= scene.layers.len() { return; }
        if let Some(anchor) = self.selected {
            let (min, max) = if anchor < target { (anchor, target) } else { (target, anchor) };
            for i in min..=max {
                if i < scene.layers.len() {
                    self.selected_layers.insert(i);
                }
            }
            self.selected = Some(target);
        } else {
            self.select_layer(target, false);
        }
    }

    pub fn is_layer_selected(&self, index: usize) -> bool {
        self.selected_layers.contains(&index) || self.selected == Some(index)
    }

    pub fn clear_layer_selection(&mut self) {
        self.selected_layers.clear();
        self.selected = None;
    }

    pub fn group_selected(&mut self) {
        if self.selected_layers.len() < 2 {
            return;
        }
        let before = self.scene.clone();
        let Some(scene) = self.scene.as_mut() else { return; };
        let gid = crate::core::design::new_id();
        let name = self.language.text("design-group").to_string();
        for &idx in &self.selected_layers {
            if let Some(layer) = scene.layers.get_mut(idx) {
                layer.group_id = Some(gid.clone());
                layer.group_name = Some(name.clone());
            }
        }
        if let Some(before) = before {
            self.changed(before, true);
        }
    }

    pub fn ungroup_selected(&mut self) {
        let before = self.scene.clone();
        let Some(scene) = self.scene.as_mut() else { return; };
        let mut modified = false;
        for &idx in &self.selected_layers {
            if let Some(layer) = scene.layers.get_mut(idx)
                && layer.group_id.is_some() {
                    layer.group_id = None;
                    layer.group_name = None;
                    modified = true;
                }
        }
        if modified && let Some(before) = before {
            self.changed(before, true);
        }
    }

    pub fn select(&mut self, scene: DesignDocument) {
        self.hovered_layer = None;
        self.selected = scene.layers.len().checked_sub(1);
        self.selected_layers.clear();
        if let Some(s) = self.selected {
            self.selected_layers.insert(s);
        }
        self.inspector_tab = 0;
        self.scene = Some(scene);
        self.undo.clear();
        self.redo.clear();
        self.pending_since = None;
        self.drag = None;
        self.saved = true;
        self.confirm_delete = false;
        self.animation_playing = false;
        self.animation_started_at = None;
        self.animation_time_ms = 0;
    }

    pub fn play_animation(&mut self) {
        self.animation_playing = true;
        self.animation_started_at = Some(std::time::Instant::now());
        self.animation_time_ms = 0;
    }

    pub fn pause_animation(&mut self) {
        self.animation_playing = false;
        self.animation_started_at = None;
    }

    pub fn toggle_animation(&mut self) {
        if self.animation_playing {
            self.pause_animation();
        } else {
            self.play_animation();
        }
    }

    #[cfg(test)]
    pub(super) fn canvas_toolbar(
        &mut self,
        ui: &mut egui::Ui,
        theme: &AppTheme,
        state: &mut ControlPanelState,
    ) {
        toolbar::canvas_toolbar(self, ui, theme, state);
    }

    pub fn flush(&mut self, db: &AppDb, deck: &mut SlideDeck, force: bool) {
        if !force
            && self
                .pending_since
                .is_none_or(|time| time.elapsed().as_millis() < 500)
        {
            return;
        }
        if self.saved {
            return;
        }
        if let Some(scene) = &self.scene {
            match db.save_design(scene) {
                Ok(()) => {
                    if let Some(existing) = deck.designs.iter_mut().find(|item| item.id == scene.id)
                    {
                        *existing = scene.clone();
                    } else {
                        deck.designs.push(scene.clone());
                    }
                    if deck
                        .preview_design
                        .as_ref()
                        .is_some_and(|preview| preview.id == scene.id)
                    {
                        let _ = deck.preview_design_by_id(&scene.id);
                    }
                    self.saved = true;
                    self.pending_since = None;
                    self.error.clear();
                }
                Err(error) => self.error = self.language.format("workspace-could-not-save", &[("error", &error.to_string())]),
            }
        }
    }

    fn changed(&mut self, before: DesignDocument, record: bool) {
        if self.scene.as_ref() == Some(&before) {
            return;
        }
        if record
            && self
                .scene
                .as_ref()
                .is_some_and(|scene| scene.id == before.id)
        {
            self.undo.push(before);
            if self.undo.len() > 60 {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
        self.saved = false;
        self.pending_since = Some(std::time::Instant::now());
    }

    fn new_scene(&mut self, kind: usize, db: &AppDb, deck: &mut SlideDeck) {
        self.flush(db, deck, true);
        self.select(DesignDocument::template_localized(kind, self.language));
        self.saved = false;
        self.flush(db, deck, true);
        if let Some(scene) = &self.scene {
            let _ = deck.preview_design_by_id(&scene.id);
        }
    }

    fn add_layer(&mut self, content: LayerContent) {
        let before = self.scene.clone();
        let Some(scene) = self.scene.as_mut() else {
            return;
        };
        let mut layer = DesignLayer::new(content);
        layer.name = self.language.text(&layer.name).to_owned();
        if matches!(layer.content, LayerContent::Image { .. }) {
            layer.rect = [0.0, 0.0, 1.0, 1.0];
        }
        if matches!(layer.content, LayerContent::Shape { .. }) {
            layer.color = [59, 130, 246, 255];
        }
        let new_idx = scene.layers.len();
        self.selected = Some(new_idx);
        self.selected_layers.clear();
        self.selected_layers.insert(new_idx);
        self.inspector_tab = 1;
        scene.layers.push(layer);
        if let Some(before) = before {
            self.changed(before, true);
        }
    }

    fn duplicate_layer(&mut self) {
        let Some(scene) = &mut self.scene else { return; };
        if self.selected_layers.len() > 1 {
            let mut indices: Vec<usize> = self.selected_layers.iter().copied().collect();
            indices.sort_unstable();
            let mut new_selection = std::collections::BTreeSet::new();
            let mut gid_map = std::collections::HashMap::new();
            let mut copies = Vec::new();
            for &idx in &indices {
                if let Some(layer) = scene.layers.get(idx) {
                    let mut copy = layer.clone();
                    copy.id = new_id();
                    copy.name.push_str(self.language.text("common-copy-suffix"));
                    copy.rect[0] = (copy.rect[0] + 0.02).min(1.0 - copy.rect[2]);
                    copy.rect[1] = (copy.rect[1] + 0.02).min(1.0 - copy.rect[3]);
                    if let Some(gid) = &layer.group_id {
                        let new_gid = gid_map.entry(gid.clone()).or_insert_with(new_id).clone();
                        copy.group_id = Some(new_gid);
                    }
                    copies.push(copy);
                }
            }
            let insert_start = indices.last().copied().unwrap_or(scene.layers.len()) + 1;
            for (offset, copy) in copies.into_iter().enumerate() {
                let pos = (insert_start + offset).min(scene.layers.len());
                scene.layers.insert(pos, copy);
                new_selection.insert(pos);
            }
            self.selected = new_selection.iter().next_back().copied();
            self.selected_layers = new_selection;
        } else if let Some(index) = self.selected
            && let Some(layer) = scene.layers.get(index)
        {
            let mut copy = layer.clone();
            copy.id = new_id();
            copy.name.push_str(self.language.text("common-copy-suffix"));
            copy.rect[0] = (copy.rect[0] + 0.02).min(1.0 - copy.rect[2]);
            copy.rect[1] = (copy.rect[1] + 0.02).min(1.0 - copy.rect[3]);
            scene.layers.insert(index + 1, copy);
            self.selected = Some(index + 1);
            self.selected_layers.clear();
            self.selected_layers.insert(index + 1);
        }
    }

    fn delete_layer(&mut self) {
        let Some(scene) = &mut self.scene else { return; };
        if self.selected_layers.len() > 1 {
            let mut indices: Vec<usize> = self.selected_layers.iter().copied().collect();
            indices.sort_unstable();
            for &idx in indices.iter().rev() {
                if idx < scene.layers.len() && !scene.layers[idx].locked {
                    scene.layers.remove(idx);
                }
            }
            self.selected_layers.clear();
            self.selected = scene.layers.len().checked_sub(1);
            if let Some(s) = self.selected {
                self.selected_layers.insert(s);
            }
        } else if let Some(index) = self.selected
            && index < scene.layers.len()
            && !scene.layers[index].locked
        {
            scene.layers.remove(index);
            self.selected = index.checked_sub(1);
            self.selected_layers.clear();
            if let Some(s) = self.selected {
                self.selected_layers.insert(s);
            }
        }
    }

    fn history(&mut self, redo: bool) {
        let source = if redo { &mut self.redo } else { &mut self.undo };
        if let Some(previous) = source.pop() {
            if let Some(current) = self.scene.replace(previous) {
                if redo {
                    self.undo.push(current);
                } else {
                    self.redo.push(current);
                }
            }
            self.selected = self
                .scene
                .as_ref()
                .and_then(|scene| scene.layers.len().checked_sub(1));
            self.selected_layers.clear();
            if let Some(s) = self.selected {
                self.selected_layers.insert(s);
            }
            self.saved = false;
            self.pending_since = Some(std::time::Instant::now());
        }
    }

    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut ControlPanelState,
        deck: &mut SlideDeck,
        presenting: &mut bool,
        db: &AppDb,
    ) {
        self.language = state.settings.language;
        self.plugins.clone_from(&state.settings.plugins);
        if let Some((_, _, task)) = &self.image_import {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(33));
            if let Some(result) = task.take() {
                let (scene_id, layer_id, _) = self.image_import.take().unwrap();
                match result {
                    Ok(Some(path)) if self.scene.as_ref().is_some_and(|scene| scene.id == scene_id) => {
                        if let Some(layer_id) = layer_id {
                            let before = self.scene.clone().unwrap();
                            if let Some(layer) = self.scene.as_mut().and_then(|scene| scene.layers.iter_mut().find(|layer| layer.id == layer_id))
                                && let LayerContent::Image { path: old_path, .. } = &mut layer.content { *old_path = path; }
                            self.changed(before, true);
                        } else { self.add_layer(LayerContent::Image { path, cover: true }); }
                    }
                    Err(error) => self.error = error,
                    _ => {}
                }
            }
        }
        let theme = state.theme.clone();
        controls::configure(ui, &theme);
        if self.scene.is_none()
            && let Some(scene) = deck.designs.first()
        {
            self.select(scene.clone());
        }
        // A design selected in the library can be opened directly in the editor.
        if let Some(requested) = deck.preview_design.as_ref()
            && self
                .scene
                .as_ref()
                .is_none_or(|scene| scene.id != requested.id)
            && let Some(scene) = deck
                .designs
                .iter()
                .find(|scene| scene.id == requested.id)
                .cloned()
        {
            self.flush(db, deck, true);
            self.select(scene);
        }
        self.header(ui, state, deck, presenting, db, &theme);
        if self.image_import.is_some() {
            ui.horizontal(|ui| { ui.spinner(); ui.label(crate::ui::i18n::text(ui, "design-importing-image")); });
        }
        ui.add_space(8.0);
        if !self.error.is_empty() {
            ui.colored_label(theme.status_danger, crate::ui::i18n::message(ui, &self.error));
        }
        let area = ui.available_rect_before_wrap();
        ui.allocate_rect(area, egui::Sense::hover());
        let total_w = area.width();
        let gap = 6.0;
        let left_max = (total_w * 0.24).clamp(195.0, 285.0);
        let right_max = (total_w * 0.36).clamp(280.0, 420.0);
        let mut left_w = self.left_width.clamp(190.0, left_max);
        let mut right_w = self.right_width.clamp(260.0, right_max);
        let min_center = 150.0f32.min(total_w * 0.4);
        let usable = (total_w - gap * 2.0 - min_center).max(300.0);
        if left_w + right_w > usable {
            let scale = usable / (left_w + right_w);
            left_w *= scale;
            right_w *= scale;
        }
        let left_rect = egui::Rect::from_min_size(area.min, egui::vec2(left_w, area.height()));
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(area.right() - right_w, area.top()),
            egui::vec2(right_w, area.height()),
        );
        let center_rect = egui::Rect::from_min_max(
            egui::pos2(left_rect.right() + gap, area.top()),
            egui::pos2(right_rect.left() - gap, area.bottom()),
        );
        let mut left_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("design_layers_body")
                .max_rect(left_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        left_ui.set_clip_rect(left_rect);
        left_ui.set_min_size(left_rect.size());
        panel(&theme).show(&mut left_ui, |ui| {
            sidebar::render(ui, self, deck, db, &theme, state)
        });
        let mut right_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("design_inspector_body")
                .max_rect(right_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        right_ui.set_clip_rect(right_rect);
        right_ui.set_min_size(right_rect.size());
        panel(&theme).show(&mut right_ui, |ui| {
            if let Some(tab) =
                controls::segments(ui, &["design-slide", "design-layer"], self.inspector_tab, &theme)
            {
                self.inspector_tab = tab;
            }
            ui.add_space(10.0);
            egui::ScrollArea::vertical()
                .id_salt("design_properties")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let before = self.scene.clone();
                    inspector::render(
                        ui,
                        self,
                        deck,
                        &theme,
                        &state.themes,
                        state.presentation_aspect_ratio,
                    );
                    if let Some(before) = before {
                        self.changed(before, true);
                    }
                });
        });
        let mut center_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("design_center_body")
                .max_rect(center_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        center_ui.set_clip_rect(center_rect);
        center_ui.set_min_size(center_rect.size());
        egui::Frame::new()
            .fill(theme.bg_app)
            .inner_margin(12)
            .show(&mut center_ui, |ui| {
                toolbar::canvas_toolbar(self, ui, &theme, state);
                ui.add_space(12.0);
                canvas::render(ui, self, &theme, state, deck, *presenting);
                if let Some(scene) = &self.scene
                    && let Some(live) = &deck.live_design && live.id == scene.id && *presenting {
                        ui.add_space(10.0);
                        ui.label(RichText::new(crate::ui::i18n::text(ui, "design-live-design-click-update-live-to-publish-visual-changes"))
                            .size(11.0).color(theme.accent));
                    }
            });
        let mut left_drag = 0.0;
        let mut right_drag = 0.0;
        for (index, x) in [left_rect.right() + gap * 0.5, right_rect.left() - gap * 0.5]
            .into_iter()
            .enumerate()
        {
            let hot_rect = egui::Rect::from_center_size(
                egui::pos2(x, area.center().y),
                egui::vec2(10.0, area.height()),
            );
            let response = ui.interact(
                hot_rect,
                ui.scope_id().with(("design_splitter", index)),
                egui::Sense::drag(),
            );
            let hot = response.hovered()
                || response.dragged()
                || ui
                    .ctx()
                    .pointer_hover_pos()
                    .is_some_and(|p| hot_rect.contains(p));
            if hot {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                draw_splitter_handle(ui, &theme, x, area, true, response.dragged());
            }
            if response.dragged() {
                if index == 0 {
                    left_drag = response.drag_delta().x;
                } else {
                    right_drag = -response.drag_delta().x;
                }
            }
        }
        if left_drag != 0.0 {
            self.left_width = (left_w + left_drag).clamp(195.0, left_max);
            ui.ctx().request_repaint();
        }
        if right_drag != 0.0 {
            self.right_width = (right_w + right_drag).clamp(260.0, right_max);
            ui.ctx().request_repaint();
        }
        if !self.saved {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(500));
        }
    }

    fn header(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut ControlPanelState,
        deck: &mut SlideDeck,
        presenting: &mut bool,
        db: &AppDb,
        theme: &AppTheme,
    ) {
        panel(theme).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (badge, _) = ui.allocate_exact_size(Vec2::splat(30.0), egui::Sense::hover());
                ui.painter().rect_filled(badge, 7.0, theme.accent_soft);
                ui.painter().text(
                    badge.center(),
                    egui::Align2::CENTER_CENTER,
                    STACK,
                    egui::FontId::proportional(17.0),
                    theme.accent,
                );
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.label(
                        RichText::new(crate::ui::i18n::text(ui, "design-editor"))
                            .size(13.0)
                            .strong()
                            .color(theme.text_primary),
                    );
                    ui.label(
                        RichText::new(crate::ui::i18n::text(ui, "design-custom-presentations"))
                            .size(10.5)
                            .color(theme.text_muted),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.header_actions(ui, state, deck, presenting, db, theme)
                });
            });
        });
    }

    fn header_actions(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut ControlPanelState,
        deck: &mut SlideDeck,
        presenting: &mut bool,
        db: &AppDb,
        theme: &AppTheme,
    ) {
        ui.spacing_mut().item_spacing.x = 6.0;
        if let Some(scene) = self.scene.clone() {
            ui.label(
                RichText::new(crate::ui::i18n::text(ui, if self.saved { "design-saved" } else { "design-saving" }))
                    .size(11.0)
                    .color(theme.text_muted),
            );
            if icon(ui, FLOPPY_DISK, crate::ui::i18n::text(ui, "design-save-now"), theme).clicked() {
                self.flush(db, deck, true);
            }
            if icon(ui, LIST_PLUS, crate::ui::i18n::text(ui, "workspace-add-to-playlist"), theme).clicked() {
                self.flush(db, deck, true);
                library::playlist(state, &scene);
            }
            if icon(ui, PUSH_PIN, crate::ui::i18n::text(ui, "design-pin-to-quick-bar"), theme).clicked() {
                self.flush(db, deck, true);
                library::pin(state, &scene);
            }
            if deck.live_overlay.is_some()
                && icon(ui, X, crate::ui::i18n::text(ui, "control-remove-overlay"), theme).clicked()
            {
                deck.live_overlay = None;
            }
            if ui
                .add_enabled(*presenting, egui::Button::new(crate::ui::i18n::format(ui, "design-overlay-variant-3", &[("icon", STACK)])))
                .on_hover_text(
                    crate::ui::i18n::text(ui, "design-add-this-design-over-the-live-content-without-replacing-the"),
                )
                .clicked()
            {
                let mut extra = scene.clone();
                extra.background_mode = "transparent".into();
                extra.overlay = true;
                extra.start_timers();
                self.scene = Some(extra.clone());
                self.saved = false;
                self.pending_since = Some(std::time::Instant::now());
                deck.live_overlay = Some(extra);
            }
            let is_live = deck
                .live_design
                .as_ref()
                .is_some_and(|live| live.id == scene.id)
                && *presenting;
            let project_label = if is_live {
                crate::ui::i18n::format(ui, "design-update-live", &[("icon", PLAY)])
            } else {
                crate::ui::i18n::format(ui, "design-present", &[("icon", PLAY)])
            };
            if ui
                .add(
                    egui::Button::new(RichText::new(project_label).color(theme.text_on_accent))
                        .fill(theme.accent),
                )
                .clicked()
            {
                self.flush(db, deck, true);
                if let Err(error) = deck.preview_design_by_id(&scene.id) {
                    self.error = error;
                } else {
                    deck.project_preview();
                    *presenting = true;
                    state.show_presentation_dock = state.settings.operator().show_dock;
                    if let Some(live) = deck
                        .live_design
                        .as_ref()
                        .filter(|live| live.id == scene.id)
                        .or(deck
                            .live_overlay
                            .as_ref()
                            .filter(|live| live.id == scene.id))
                    {
                        self.scene = Some(live.clone());
                        self.saved = false;
                        self.pending_since = Some(std::time::Instant::now());
                    }
                }
            }
        }
        ui.menu_button(crate::ui::i18n::format(ui, "design-new-design", &[("icon", PLUS)]), |ui| {
            self.new_design_menu(ui, state, deck, db, theme);
        });
    }

    fn new_design_menu(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut ControlPanelState,
        deck: &mut SlideDeck,
        db: &AppDb,
        theme: &AppTheme,
    ) {
        ui.set_width(300.0);
        ui.spacing_mut().item_spacing = egui::vec2(4.0, 3.0);
        ui.label(
            RichText::new(crate::ui::i18n::text(ui, "design-templates"))
                .size(10.5)
                .strong()
                .color(theme.text_muted),
        );
        ui.add_space(2.0);
        for (kind, label, description, icon, (r, g, b), tag) in [
            (0, "design-blank", "design-start-from-scratch", FILE_TEXT, (148, 163, 184), "SLIDE"),
            (1, "design-welcome-label", "design-title-and-message", HAND_WAVING, (251, 191, 36), "SLIDE"),
            (2, "design-countdown", "design-service-countdown", TIMER, (56, 189, 248), "TIMER"),
            (3, "design-questions-and-answers", "design-interactive-activity", QUESTION, (52, 211, 153), "QUIZ"),
            (4, "design-clock-overlay-label", "design-clock-overlay", CLOCK, (167, 139, 250), "EXTRA"),
            (5, "design-lower-third-overlay", "design-lower-third-banner", RECTANGLE, (251, 113, 133), "EXTRA"),
        ] {
            if kind == 4 {
                ui.add_space(3.0);
                ui.separator();
                ui.add_space(2.0);
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "control-overlay-design"))
                        .size(10.0)
                        .strong()
                        .color(theme.text_muted),
                );
                ui.add_space(1.0);
            }
            let (rect, response) = ui
                .allocate_exact_size(egui::vec2(ui.available_width(), 46.0), egui::Sense::click());
            if response.hovered() {
                ui.painter().rect_filled(rect, 8.0, theme.bg_card_hover);
                ui.painter().rect_stroke(rect, 8.0, egui::Stroke::new(1.0, theme.border_subtle), egui::StrokeKind::Inside);
            }
            let badge =
                egui::Rect::from_min_size(rect.min + egui::vec2(6.0, 8.0), egui::vec2(30.0, 30.0));
            let badge_bg = egui::Color32::from_rgba_unmultiplied(r, g, b, 35);
            let badge_stroke = egui::Color32::from_rgba_unmultiplied(r, g, b, 70);
            let badge_fg = egui::Color32::from_rgb(r, g, b);
            ui.painter().rect_filled(badge, 8.0, badge_bg);
            ui.painter().rect_stroke(badge, 8.0, egui::Stroke::new(1.0, badge_stroke), egui::StrokeKind::Inside);
            ui.painter().text(
                badge.center(),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(15.0),
                badge_fg,
            );
            let title = ui.painter().layout_no_wrap(
                crate::ui::i18n::text(ui, label).to_owned(),
                egui::FontId::proportional(12.5),
                theme.text_primary,
            );
            ui.painter().galley(
                egui::pos2(badge.right() + 10.0, rect.center().y - title.size().y + 1.0),
                title,
                theme.text_primary,
            );
            let subtitle = ui.painter().layout_no_wrap(
                crate::ui::i18n::text(ui, description).to_owned(),
                egui::FontId::proportional(10.5),
                theme.text_muted,
            );
            ui.painter().galley(
                egui::pos2(badge.right() + 10.0, rect.center().y + 2.0),
                subtitle,
                theme.text_muted,
            );

            let chip_w = 40.0;
            let chip_rect = egui::Rect::from_min_size(
                egui::pos2(rect.right() - chip_w - 6.0, rect.center().y - 8.0),
                egui::vec2(chip_w, 16.0),
            );
            let chip_bg = if tag == "EXTRA" {
                egui::Color32::from_rgba_unmultiplied(139, 92, 246, 35)
            } else {
                egui::Color32::from_rgba_unmultiplied(100, 116, 139, 25)
            };
            let chip_fg = if tag == "EXTRA" {
                egui::Color32::from_rgb(196, 181, 253)
            } else {
                egui::Color32::from_rgb(148, 163, 184)
            };
            ui.painter().rect_filled(chip_rect, 4.0, chip_bg);
            ui.painter().text(
                chip_rect.center(),
                egui::Align2::CENTER_CENTER,
                tag,
                egui::FontId::proportional(9.0),
                chip_fg,
            );

            if response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if response.clicked() {
                self.new_scene(kind, db, deck);
                state.selected_tab = 4;
                ui.close();
            }
        }
    }

    fn reorder(&mut self, forward: bool) {
        let before = self.scene.clone();
        if let Some(scene) = &mut self.scene
            && let Some(index) = self.selected
        {
            let target = if forward {
                (index + 1).min(scene.layers.len().saturating_sub(1))
            } else {
                index.saturating_sub(1)
            };
            if index < scene.layers.len() {
                scene.layers.swap(index, target);
                self.selected = Some(target);
            }
        }
        if let Some(before) = before {
            self.changed(before, true);
        }
    }
}

pub fn panel(theme: &AppTheme) -> egui::Frame {
    egui::Frame::new()
        .fill(theme.bg_panel)
        .stroke(egui::Stroke::new(1.0, theme.border_subtle))
        .corner_radius(12.0)
        .inner_margin(egui::Margin::symmetric(11, 8))
}
pub(super) fn icon(ui: &mut egui::Ui, icon: &str, label: &str, theme: &AppTheme) -> egui::Response {
    theme.icon_button(ui, icon, label, false, Vec2::splat(24.0))
}
