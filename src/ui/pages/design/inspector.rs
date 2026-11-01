use super::{DesignEditor, controls, icon, slide_controls};
use crate::{
    core::{
        design::{LayerContent, Question, ShapeKind, new_id, now_ms},
        slides::SlideDeck,
    },
    ui::theme::AppTheme,
};
use eframe::egui::{self, RichText, Vec2};
use egui_phosphor::regular::*;

#[cfg(test)]
pub(super) use super::inspector_text::render_text_tab;

#[cfg(test)]
#[path = "typography_tests.rs"]
mod typography_tests;

pub(super) fn import_image(language: crate::core::i18n::Language) -> Result<Option<String>, String> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter(language.text("design-images"), &["png", "jpg", "jpeg", "webp", "gif", "bmp"])
        .pick_file()
    else {
        return Ok(None);
    };
    let image = image::open(&path)
        .map_err(|error| language.format("design-could-not-open-the-image", &[("error", &error.to_string())]))?
        .thumbnail(2560, 2560);
    let folder = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join("design-assets");
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let target = folder.join(format!("{}.png", new_id()));
    image.save(&target).map_err(|e| e.to_string())?;
    Ok(Some(target.to_string_lossy().into_owned()))
}

pub(super) fn render(
    ui: &mut egui::Ui,
    editor: &mut DesignEditor,
    deck: &mut SlideDeck,
    theme: &AppTheme,
    themes: &[crate::core::models::PresentationTheme],
    aspect: f32,
) {
    let export_id = ui.make_persistent_id("design_export_file_picker");
    let export_task = ui
        .ctx()
        .data_mut(|data| data.get_temp::<crate::services::tasks::BackgroundTask<Option<String>>>(export_id));
    if let Some(task) = &export_task {
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(33));
        if let Some(result) = task.take() {
            ui.ctx().data_mut(|data| data.remove::<crate::services::tasks::BackgroundTask<Option<String>>>(export_id));
            if let Err(error) = result {
                editor.error = error;
            }
        }
    }
    let Some(scene) = &mut editor.scene else {
        controls::empty(
            ui,
            STACK,
            "design-your-next-slide",
            "design-create-a-design-to-customize-your-content",
            theme,
        );
        return;
    };
    if editor.inspector_tab == 0 {
        match slide_controls::render(ui, scene, themes, theme) {
            slide_controls::Action::Duplicate => {
                let mut copy = scene.clone();
                copy.id = new_id();
                copy.name.push_str(editor.language.text("common-copy-suffix"));
                deck.preview_design = Some(copy.clone());
                editor.scene = Some(copy);
                editor.selected = None;
                editor.undo.clear();
                editor.redo.clear();
            }
            slide_controls::Action::Export => {
                let mut export = scene.clone();
                export.width = 1280;
                export.height = (1280.0 / aspect.clamp(0.5, 3.5)) as u32;
                let themes = themes.to_vec();
                let language = crate::ui::i18n::language(ui.ctx());
                let task = crate::services::tasks::BackgroundTask::spawn_with_repaint(ui.ctx(), move || {
                    let Some(path) = rfd::FileDialog::new()
                        .add_filter("PNG", &["png"])
                        .set_file_name("design.png")
                        .save_file()
                    else {
                        return Ok(None);
                    };
                    let frame = crate::render::design::frame(
                        &export.resolve_theme_fonts(themes.iter().find(|theme| Some(&theme.id) == export.theme_id.as_ref())),
                        1280,
                    );
                    std::fs::write(&path, frame.png()).map_err(|e| language.format("design-export-failed", &[("error", &e.to_string())]))?;
                    Ok(Some(path.to_string_lossy().into_owned()))
                });
                ui.ctx().data_mut(|data| data.insert_temp(export_id, task));
            }
            slide_controls::Action::Delete => editor.confirm_delete = true,
            slide_controls::Action::None => {}
        }
        return;
    }
    let Some(index) = editor.selected else {
        controls::empty(
            ui,
            CURSOR,
            "design-choose-a-layer",
            "design-click-an-element-on-the-canvas-to-edit-its-properties",
            theme,
        );
        return;
    };

    if editor.selected_layers.len() > 1 {
        let count = editor.selected_layers.len();
        let all_grouped = editor.selected_layers.iter().all(|&idx| {
            scene.layers.get(idx).and_then(|l| l.group_id.as_ref()).is_some()
        });
        let mut do_ungroup = false;
        let mut do_group = false;
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(crate::ui::i18n::language(ui.ctx()).count("design-group-selected-count", "", count))
                    .size(11.0)
                    .color(theme.text_muted),
            );
            if all_grouped {
                if ui.small_button(RichText::new(format!("{FOLDER_MINUS} {}", crate::ui::i18n::text(ui, "design-ungroup"))).size(10.0)).clicked() {
                    do_ungroup = true;
                }
            } else if ui.small_button(RichText::new(format!("{FOLDER_PLUS} {}", crate::ui::i18n::text(ui, "design-group"))).size(10.0)).clicked() {
                do_group = true;
            }
        });
        ui.add_space(4.0);
        if do_ungroup {
            for &idx in &editor.selected_layers {
                if let Some(l) = scene.layers.get_mut(idx) {
                    l.group_id = None;
                    l.group_name = None;
                }
            }
            editor.saved = false;
            editor.pending_since = Some(std::time::Instant::now());
        } else if do_group {
            let gid = new_id();
            let gname = crate::ui::i18n::text(ui, "design-group-name").to_string();
            for &idx in &editor.selected_layers {
                if let Some(l) = scene.layers.get_mut(idx) {
                    l.group_id = Some(gid.clone());
                    l.group_name = Some(gname.clone());
                }
            }
            editor.saved = false;
            editor.pending_since = Some(std::time::Instant::now());
        }
    }

    let mut rename_group_action: Option<(String, String)> = None;
    {
        let Some(layer) = scene.layers.get_mut(index) else {
            return;
        };

        // Sync runtime state from the live design so controls stay in step.
        let mut runtime_changed = false;
        if let Some(live) = deck
            .live_overlay
            .as_ref()
            .filter(|live| live.id == scene.id)
            .or(deck.live_design.as_ref().filter(|live| live.id == scene.id))
            && let Some(running) = live.layers.iter().find(|item| item.id == layer.id)
        {
            match (&mut layer.content, &running.content) {
                (LayerContent::Countdown(draft), LayerContent::Countdown(live_timer)) => {
                    draft.started_at = live_timer.started_at;
                    draft.remaining_ms = live_timer.remaining_ms;
                }
                (
                    LayerContent::Questions {
                        current, revealed, ..
                    },
                    LayerContent::Questions {
                        current: live_current,
                        revealed: live_revealed,
                        ..
                    },
                ) => {
                    *current = *live_current;
                    *revealed = *live_revealed;
                }
                _ => {}
            }
        }

        if let Some(gid) = layer.group_id.clone() {
            let default_gname = crate::ui::i18n::text(ui, "design-group-name").to_string();
            let mut gname = layer.group_name.clone().unwrap_or(default_gname);
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{FOLDER} {}", crate::ui::i18n::text(ui, "design-group-label"))).size(11.0).color(theme.text_muted));
                if ui.text_edit_singleline(&mut gname).changed() {
                    layer.group_name = Some(gname.clone());
                    rename_group_action = Some((gid, gname));
                }
            });
            ui.add_space(4.0);
        }

        // Layer header: icon, type, lock button and name.
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "{}  {}",
                    super::sidebar::content_icon(&layer.content),
                    crate::ui::i18n::text(ui, layer.content.label())
                ))
                .size(12.0)
                .strong()
                .color(theme.accent),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let lock_icon = if layer.locked { LOCK_SIMPLE } else { LOCK_SIMPLE_OPEN };
                let lock_btn = ui
                    .add(
                        egui::Button::new(RichText::new(lock_icon).size(13.0))
                            .min_size(Vec2::new(26.0, 22.0))
                            .selected(layer.locked),
                    )
                    .on_hover_text(if layer.locked {
                        crate::ui::i18n::text(ui, "design-locked-layer-click-to-unlock")
                    } else {
                        crate::ui::i18n::text(ui, "design-lock-layer-on-the-canvas")
                    });
                if lock_btn.clicked() {
                    layer.locked = !layer.locked;
                }
            });
        });
        ui.add_space(3.0);
        ui.add(
            egui::TextEdit::singleline(&mut layer.name)
                .desired_width(f32::INFINITY)
                .margin(egui::vec2(9.0, 7.0))
                .hint_text(crate::ui::i18n::text(ui, "design-layer-name")),
        );
        ui.add_space(4.0);

        // Sub-tabs: Posição / Aparência / Textos / Transição
        let has_text_content = !matches!(
            layer.content,
            LayerContent::Image { .. } | LayerContent::Shape { .. } | LayerContent::Vector { .. }
        );

        let tabs: &[&str] = if has_text_content {
            &[
                "design-position",
                "design-appearance",
                "design-text",
                "design-transition",
            ]
        } else {
            &[
                "design-position",
                "design-appearance",
                "design-transition",
            ]
        };

        let mut tab_idx = editor.layer_inspector_tab.unwrap_or(0);
        if tab_idx >= tabs.len() {
            tab_idx = 0;
        }

        if tabs.len() == 4 && ui.available_width() < 340.0 {
            if let Some(s) = controls::segments_height(ui, &tabs[0..2], if tab_idx < 2 { tab_idx } else { usize::MAX }, 28.0, theme) {
                tab_idx = s;
            }
            ui.add_space(3.0);
            if let Some(s) = controls::segments_height(ui, &tabs[2..4], if tab_idx >= 2 { tab_idx - 2 } else { usize::MAX }, 28.0, theme) {
                tab_idx = s + 2;
            }
        } else if let Some(s) = controls::segments_height(ui, tabs, tab_idx, 28.0, theme) {
            tab_idx = s;
        }
        editor.layer_inspector_tab = Some(tab_idx);
        ui.add_space(4.0);

        let scene_id = scene.id.clone();
        let import_in_progress = editor.image_import.is_some();
        let theme_font = editor
            .preview_theme_id
            .as_ref()
            .or(scene.theme_id.as_ref())
            .and_then(|id| themes.iter().find(|t| &t.id == id))
            .or_else(|| themes.first())
            .map(|t| t.font_family.as_str())
            .filter(|f| !f.is_empty() && *f != crate::core::models::fonts::THEME_FONT);
        if has_text_content {
            match tab_idx {
                0 => render_position_tab(ui, layer, &scene_id, import_in_progress, &mut editor.image_import, index, theme, &mut runtime_changed),
                1 => super::inspector_appearance::render_appearance_tab(ui, layer, theme, &mut runtime_changed),
                2 => super::inspector_text::render_text_tab(ui, layer, index, theme_font, theme, &mut runtime_changed),
                3 => super::inspector_transition::render_transition_tab(ui, layer, &mut scene.transition, &mut scene.transition_ms, &mut scene.transition_strength, &mut scene.transition_easing, theme, &mut runtime_changed),
                _ => {}
            }
        } else {
            match tab_idx {
                0 => render_position_tab(ui, layer, &scene_id, import_in_progress, &mut editor.image_import, index, theme, &mut runtime_changed),
                1 => super::inspector_appearance::render_appearance_tab(ui, layer, theme, &mut runtime_changed),
                2 => super::inspector_transition::render_transition_tab(ui, layer, &mut scene.transition, &mut scene.transition_ms, &mut scene.transition_strength, &mut scene.transition_easing, theme, &mut runtime_changed),
                _ => {}
            }
        }

        for target in [&mut deck.live_design, &mut deck.preview_design, &mut deck.live_overlay]
            .into_iter()
            .flatten()
            .filter(|target| target.id == scene.id)
        {
            target.transition = scene.transition.clone();
            target.transition_ms = scene.transition_ms;
            target.transition_strength = scene.transition_strength;
            target.transition_easing = scene.transition_easing.clone();
        }

        if runtime_changed {
            for live in [&mut deck.live_design, &mut deck.live_overlay]
                .into_iter()
                .flatten()
                .filter(|live| live.id == scene.id)
            {
                if let Some(live_layer) = live.layers.iter_mut().find(|item| item.id == layer.id) {
                    live_layer.content = layer.content.clone();
                }
            }
        }

        if matches!(layer.content, LayerContent::Countdown(_) | LayerContent::Questions { .. }) {
            ui.add_space(10.0);
            ui.label(
                RichText::new(crate::ui::i18n::text(ui, "design-starting-pausing-and-revealing-answers-updates-the-live-design"))
                    .size(10.5)
                    .color(theme.text_muted),
            );
        }
    }

    if let Some((gid, gname)) = rename_group_action {
        for l in &mut scene.layers {
            if l.group_id.as_ref() == Some(&gid) {
                l.group_name = Some(gname.clone());
            }
        }
        editor.saved = false;
        editor.pending_since = Some(std::time::Instant::now());
    }
}

#[allow(clippy::too_many_arguments)]
fn render_position_tab(
    ui: &mut egui::Ui,
    layer: &mut crate::core::design::DesignLayer,
    scene_id: &str,
    import_in_progress: bool,
    image_import: &mut Option<(String, Option<String>, crate::services::tasks::BackgroundTask<Option<String>>)>,
    index: usize,
    theme: &AppTheme,
    runtime_changed: &mut bool,
) {
    ui.add_enabled_ui(!layer.locked, |ui| {
        // Content-specific controls.
        match &mut layer.content {
            LayerContent::Vector { .. } => {
                ui.label(crate::ui::i18n::language(ui.ctx()).text("design-vector"));
            }
            LayerContent::Plugin(instance) => {
                let language = crate::ui::i18n::language(ui.ctx());
                *runtime_changed |=
                    crate::ui::components::plugin_fields::component_operational(ui, instance, language, theme);
            }
            LayerContent::Text(text) => {
                ui.add(
                    egui::TextEdit::multiline(text)
                        .desired_rows(3)
                        .desired_width(f32::INFINITY)
                        .margin(egui::vec2(9.0, 7.0))
                        .hint_text(crate::ui::i18n::text(ui, "design-enter-text-here-label")),
                );
            }
            LayerContent::Image { path, cover } => {
                ui.label(
                    RichText::new(
                        std::path::Path::new(path)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy(),
                    )
                    .size(11.0),
                );
                let replace_label = crate::ui::i18n::format(ui, "design-replace-image", &[("icon", IMAGE)]);
                if ui.button(replace_label).clicked() && !import_in_progress {
                    let language = crate::ui::i18n::language(ui.ctx());
                    *image_import = Some((
                        scene_id.to_owned(),
                        Some(layer.id.clone()),
                        crate::services::tasks::BackgroundTask::spawn_with_repaint(ui.ctx(), move || import_image(language)),
                    ));
                }
                ui.checkbox(cover, crate::ui::i18n::text(ui, "design-fill-and-crop"))
                    .on_hover_text(crate::ui::i18n::text(ui, "design-uncheck-to-display-the-entire-image-without-cropping"));
                if !std::path::Path::new(path).is_file() {
                    ui.colored_label(theme.status_danger, crate::ui::i18n::text(ui, "design-image-unavailable-replace-the-file"));
                }
            }
            LayerContent::Shape { radius } => {
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [68.0, 20.0],
                        egui::Label::new(RichText::new(crate::ui::i18n::text(ui, "design-shape")).size(11.5).color(theme.text_secondary)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        egui::ComboBox::from_id_salt("design_shape_kind")
                            .selected_text(crate::ui::i18n::text(ui, layer.shape.label()))
                            .width((ui.available_width() - 4.0).clamp(80.0, 160.0))
                            .show_ui(ui, |ui| {
                                for shape in ShapeKind::ALL {
                                    ui.selectable_value(&mut layer.shape, shape, crate::ui::i18n::text(ui, shape.label()));
                                }
                            });
                    });
                });
                if layer.shape == ShapeKind::Rectangle {
                    ui.add_space(2.0);
                    property_slider(ui, "design-corners", radius, 0.0..=150.0, " px", 0, theme);
                }
                if layer.shape == ShapeKind::Line {
                    ui.add_space(2.0);
                    property_slider(ui, "design-thickness", &mut layer.line_width, 1.0..=120.0, " px", 0, theme);
                }
            }
            LayerContent::Clock { seconds, date } => {
                ui.horizontal(|ui| {
                    ui.checkbox(seconds, crate::ui::i18n::text(ui, "design-seconds"));
                    ui.checkbox(date, crate::ui::i18n::text(ui, "design-date"));
                });
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "design-synchronized-local-time")).size(10.5).color(theme.text_muted),
                );
            }
            LayerContent::Countdown(timer) => {
                ui.checkbox(&mut timer.auto_start, crate::ui::i18n::text(ui, "design-start-automatically"));
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [68.0, 20.0],
                        egui::Label::new(RichText::new(crate::ui::i18n::text(ui, "design-duration")).size(11.5).color(theme.text_secondary)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(egui::DragValue::new(&mut timer.duration_secs).range(1..=86400).suffix(" s").speed(5))
                            .changed()
                        {
                            timer.reset();
                            *runtime_changed = true;
                        }
                    });
                });
                ui.label(
                    RichText::new(timer.text(now_ms())).size(26.0).strong().color(theme.accent),
                );
                ui.horizontal(|ui| {
                    let half = (ui.available_width() - 6.0) * 0.5;
                    let btn_text = if timer.started_at.is_some() {
                        crate::ui::i18n::format(ui, "design-pause", &[("icon", PAUSE)])
                    } else {
                        crate::ui::i18n::format(ui, "design-start", &[("icon", PLAY)])
                    };
                    if ui
                        .add_sized([half, 24.0], egui::Button::new(btn_text))
                        .clicked()
                    {
                        if timer.started_at.is_some() {
                            timer.pause(now_ms());
                        } else {
                            timer.start(now_ms());
                        }
                        *runtime_changed = true;
                    }
                    if icon(ui, ARROW_COUNTER_CLOCKWISE, "design-restart-countdown", theme).clicked() {
                        timer.reset();
                        *runtime_changed = true;
                    }
                });
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(crate::ui::i18n::text(ui, "design-when-finished")).size(11.0).color(theme.text_secondary));
                    ui.add(
                        egui::TextEdit::singleline(&mut timer.finished_text)
                            .desired_width(f32::INFINITY)
                            .hint_text(crate::ui::i18n::text(ui, "design-e-g-starting")),
                    );
                });
            }
            LayerContent::Questions { items, current, revealed } => {
                if items.is_empty() {
                    items.push(Question { question: crate::ui::i18n::text(ui, "design-new-question").to_string(), answer: String::new() });
                }
                *current = (*current).min(items.len() - 1);
                let current_str = (*current + 1).to_string();
                let total_str = items.len().to_string();
                let current_label = crate::ui::i18n::format(
                    ui,
                    "design-question-position",
                    &[("current", &current_str), ("total", &total_str)],
                );
                egui::ComboBox::from_id_salt("design_current_question")
                    .selected_text(current_label)
                    .width(ui.available_width())
                    .show_ui(ui, |ui| {
                        for idx in 0..items.len() {
                            let num_str = (idx + 1).to_string();
                            let item_label = crate::ui::i18n::format(ui, "design-question-number", &[("number", &num_str)]);
                            if ui.selectable_value(current, idx, item_label).changed() {
                                *revealed = false;
                                *runtime_changed = true;
                            }
                        }
                    });
                ui.add(
                    egui::TextEdit::multiline(&mut items[*current].question)
                        .desired_rows(2)
                        .desired_width(f32::INFINITY)
                        .hint_text(crate::ui::i18n::text(ui, "design-write-the-question")),
                );
                ui.add(
                    egui::TextEdit::multiline(&mut items[*current].answer)
                        .desired_rows(2)
                        .desired_width(f32::INFINITY)
                        .hint_text(crate::ui::i18n::text(ui, "design-write-the-answer")),
                );
                ui.horizontal(|ui| {
                    if ui.add_enabled(*current > 0, egui::Button::new(CARET_LEFT)).clicked() {
                        *current -= 1;
                        *revealed = false;
                        *runtime_changed = true;
                    }
                    if ui
                        .add_enabled(*current + 1 < items.len(), egui::Button::new(CARET_RIGHT))
                        .clicked()
                    {
                        *current += 1;
                        *revealed = false;
                        *runtime_changed = true;
                    }
                });
            }
        }
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Position controls.
        section_header(ui, "design-position-and-size", theme);
        let cell_gap = 6.0;
        let cell_w = ((ui.available_width() - cell_gap) * 0.5).max(60.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = cell_gap;
            geo_field(ui, "X", &mut layer.rect[0], false, cell_w, theme);
            geo_field(ui, "Y", &mut layer.rect[1], false, cell_w, theme);
        });
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = cell_gap;
            geo_field(ui, "common-geo-width", &mut layer.rect[2], true, cell_w, theme);
            geo_field(ui, "common-geo-height", &mut layer.rect[3], true, cell_w, theme);
        });
        layer.rect[0] = layer.rect[0].min(1.0 - layer.rect[2]);
        layer.rect[1] = layer.rect[1].min(1.0 - layer.rect[3]);
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let btn_w = ((ui.available_width() - cell_gap) * 0.5).max(50.0);
            if ui.add_sized([btn_w, 24.0], egui::Button::new(RichText::new(crate::ui::i18n::text(ui, "design-center-horizontally")).size(11.0))).clicked() {
                layer.rect[0] = (1.0 - layer.rect[2]) / 2.0;
            }
            if ui.add_sized([btn_w, 24.0], egui::Button::new(RichText::new(crate::ui::i18n::text(ui, "design-center-vertically")).size(11.0))).clicked() {
                layer.rect[1] = (1.0 - layer.rect[3]) / 2.0;
            }
        });
        if matches!(layer.content, LayerContent::Image { .. }) {
            ui.add_space(3.0);
            if ui
                .add_sized(
                    [ui.available_width(), 24.0],
                    egui::Button::new(RichText::new(crate::ui::i18n::text(ui, "design-fill-the-entire-screen")).size(11.0)),
                )
                .clicked()
            {
                layer.rect = [0.0, 0.0, 1.0, 1.0];
            }
        }
        let _ = index;
    });
}

pub(super) fn section_header(ui: &mut egui::Ui, title: &str, theme: &AppTheme) {
    ui.add_space(8.0);
    controls::caption(ui, title, theme);
    ui.add_space(2.0);
}

pub(super) fn property_slider(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    suffix: &str,
    decimals: usize,
    theme: &AppTheme,
) -> bool {
    let mut changed = false;
    let label_w = 68.0;
    let val_w = 50.0;
    let spacing = ui.spacing().item_spacing.x;
    ui.horizontal(|ui| {
        ui.add_sized(
            [label_w, 20.0],
            egui::Label::new(RichText::new(crate::ui::i18n::text(ui, label)).size(11.5).color(theme.text_secondary)).truncate(),
        );
        let avail_for_slider = (ui.available_width() - val_w - spacing).max(30.0);
        ui.spacing_mut().slider_width = avail_for_slider;
        let slider = egui::Slider::new(value, range.clone()).show_value(false).fixed_decimals(decimals);
        if ui.add(slider).changed() {
            changed = true;
        }
        let dv = egui::DragValue::new(value)
            .range(range)
            .fixed_decimals(decimals)
            .suffix(suffix)
            .speed(0.5);
        if ui.add_sized([val_w, 20.0], dv).changed() {
            changed = true;
        }
    });
    changed
}

fn geo_field(
    ui: &mut egui::Ui,
    label: &str,
    val: &mut f32,
    is_size: bool,
    width: f32,
    theme: &AppTheme,
) {
    let mut percent = *val * 100.0;
    ui.allocate_ui_with_layout(
        Vec2::new(width, 22.0),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let lbl_w = 12.0;
            ui.add_sized(
                [lbl_w, 20.0],
                egui::Label::new(
                    RichText::new(crate::ui::i18n::text(ui, label))
                        .size(11.0)
                        .strong()
                        .color(theme.text_muted),
                ),
            );
            let drag_w = (width - lbl_w - 4.0).max(30.0);
            let drag = egui::DragValue::new(&mut percent)
                .range(if is_size { 1.0..=100.0 } else { 0.0..=100.0 })
                .fixed_decimals(1)
                .suffix("%")
                .speed(0.25);
            if ui.add_sized([drag_w, 20.0], drag).changed() {
                *val = (percent / 100.0).clamp(0.0, 1.0);
            }
        },
    );
}
