use super::{
    shader_controls::shader_controls,
    widgets::{
        color_row_solid, property_slider, property_slider_pct, section_title,
        transition_tile,
    },
};
use crate::core::state::ControlPanelState;
use eframe::egui::{self, Color32, RichText, Stroke, StrokeKind, Vec2};
use egui_phosphor::regular::*;

pub(super) fn background_tab(ui: &mut egui::Ui, state: &mut ControlPanelState) -> bool {
    let mut push_snapshot = false;

    if let Some(tab_id) = super::widgets::segments(
        ui,
        &["common-color", "common-image", "chat-video", "settings-animated"],
        state.theme_editor.bg_tab,
        &state.theme,
    ) {
        state.theme_editor.bg_tab = tab_id;
        state.theme_editor.editing.bg_type = match tab_id {
            1 => "image".to_string(),
            2 => "video".to_string(),
            3 => "shader".to_string(),
            _ => {
                if state.theme_editor.editing.is_gradient {
                    "gradient".to_string()
                } else {
                    "color".to_string()
                }
            }
        };
        push_snapshot = true;
    }

    ui.add_space(8.0);

    match state.theme_editor.bg_tab {
        0 => {
            let cur_mode = if state.theme_editor.editing.is_gradient { 1 } else { 0 };
            if let Some(mode) = super::widgets::segments(
                ui,
                &["settings-solid", "settings-gradient"],
                cur_mode,
                &state.theme,
            ) {
                let is_grad = mode == 1;
                state.theme_editor.editing.is_gradient = is_grad;
                state.theme_editor.editing.bg_type = if is_grad {
                    "gradient".to_string()
                } else {
                    "color".to_string()
                };
                push_snapshot = true;
            }

            ui.add_space(6.0);

            if state.theme_editor.editing.is_gradient {
                if color_row_solid(
                    ui,
                    "settings-top-color",
                    &mut state.theme_editor.editing.bg_top,
                    &state.theme,
                ) {
                    push_snapshot = true;
                }
                ui.add_space(3.0);
                if color_row_solid(
                    ui,
                    "settings-bottom-color",
                    &mut state.theme_editor.editing.bg_bottom,
                    &state.theme,
                ) {
                    push_snapshot = true;
                }

                ui.add_space(4.0);

                if property_slider(
                    ui,
                    "settings-angle",
                    &mut state.theme_editor.editing.options.gradient_angle,
                    0.0..=360.0,
                    "°",
                    0,
                    1.0,
                    &state.theme,
                ) {
                    push_snapshot = true;
                }

                ui.add_space(4.0);

                let cur_angle = state.theme_editor.editing.options.gradient_angle.rem_euclid(360.0);
                let cur_segment = if (cur_angle - 0.0).abs() < 1.0 || (cur_angle - 360.0).abs() < 1.0 {
                    0
                } else if (cur_angle - 90.0).abs() < 1.0 {
                    1
                } else if (cur_angle - 45.0).abs() < 1.0 {
                    2
                } else if (cur_angle - 135.0).abs() < 1.0 {
                    3
                } else {
                    usize::MAX
                };

                if let Some(idx) = super::widgets::segments(
                    ui,
                    &["settings-vertical", "settings-horizontal", "settings-diagonal-1", "settings-diagonal-2"],
                    cur_segment,
                    &state.theme,
                ) {
                    state.theme_editor.editing.options.gradient_angle = match idx {
                        1 => 90.0,
                        2 => 45.0,
                        3 => 135.0,
                        _ => 0.0,
                    };
                    push_snapshot = true;
                }

                ui.add_space(6.0);

                let c_top = state.theme_editor.editing.bg_top;
                let c_bot = state.theme_editor.editing.bg_bottom;
                let preview_h = 32.0;
                let (prev_rect, _) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), preview_h),
                    egui::Sense::hover(),
                );

                crate::render::paint_gradient_rect(
                    ui.painter(),
                    prev_rect,
                    c_top,
                    c_bot,
                    state.theme_editor.editing.options.gradient_angle,
                    6.0,
                );
                ui.painter().rect_stroke(
                    prev_rect,
                    6.0,
                    Stroke::new(1.0, state.theme.border_default),
                    StrokeKind::Inside,
                );

                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    let invert_btn = egui::Button::new(
                        RichText::new(crate::ui::i18n::format(
                            ui,
                            "settings-reverse",
                            &[("icon", ARROWS_DOWN_UP)],
                        ))
                            .size(11.0)
                            .color(state.theme.text_secondary),
                    )
                    .fill(state.theme.bg_pill)
                    .stroke(Stroke::new(1.0, state.theme.border_default))
                    .corner_radius(6.0)
                    .min_size(Vec2::new(0.0, 24.0));

                    if ui
                        .add(invert_btn)
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text(crate::ui::i18n::text(ui, "settings-reverse-gradient-colors"))
                        .clicked()
                    {
                        std::mem::swap(&mut state.theme_editor.editing.bg_top, &mut state.theme_editor.editing.bg_bottom);
                        push_snapshot = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let presets = [
                            (
                                "settings-obsidian",
                                Color32::from_rgb(11, 15, 23),
                                Color32::from_rgb(3, 4, 7),
                            ),
                            (
                                "settings-midnight",
                                Color32::from_rgb(11, 27, 54),
                                Color32::from_rgb(2, 7, 18),
                            ),
                            (
                                "settings-ember",
                                Color32::from_rgb(33, 20, 14),
                                Color32::from_rgb(10, 5, 3),
                            ),
                            (
                                "settings-emerald",
                                Color32::from_rgb(10, 35, 24),
                                Color32::from_rgb(3, 12, 7),
                            ),
                            (
                                "settings-amethyst",
                                Color32::from_rgb(29, 16, 48),
                                Color32::from_rgb(8, 4, 15),
                            ),
                            (
                                "settings-charcoal",
                                Color32::from_rgb(31, 41, 55),
                                Color32::from_rgb(17, 24, 39),
                            ),
                        ];
                        for (name, top, bot) in presets.into_iter().rev() {
                            let p_btn = egui::Button::new(
                                RichText::new(crate::ui::i18n::text(ui, name))
                                    .size(10.5)
                                    .color(state.theme.text_secondary),
                            )
                            .fill(state.theme.bg_pill)
                            .stroke(Stroke::new(1.0, state.theme.border_default))
                            .corner_radius(5.0)
                            .min_size(Vec2::new(0.0, 22.0));

                            if ui
                                .add(p_btn)
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                            {
                                state.theme_editor.editing.bg_top = top;
                                state.theme_editor.editing.bg_bottom = bot;
                                push_snapshot = true;
                            }
                        }
                    });
                });
            } else {
                if color_row_solid(
                    ui,
                    "settings-background-color",
                    &mut state.theme_editor.editing.bg_top,
                    &state.theme,
                ) {
                    state.theme_editor.editing.bg_bottom = state.theme_editor.editing.bg_top;
                    push_snapshot = true;
                }

                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(crate::ui::i18n::text(ui, "settings-palette"))
                            .size(11.0)
                            .color(state.theme.text_secondary),
                    );
                    let palette = [
                        ("settings-black", Color32::from_rgb(10, 10, 12)),
                        ("settings-graphite", Color32::from_rgb(26, 28, 34)),
                        ("settings-blue", Color32::from_rgb(15, 23, 42)),
                        ("settings-navy", Color32::from_rgb(30, 58, 138)),
                        ("settings-wine", Color32::from_rgb(69, 10, 24)),
                        ("settings-white", Color32::from_rgb(248, 250, 252)),
                    ];
                    for (tip, c) in palette {
                        let (sq_rect, sq_resp) =
                            ui.allocate_exact_size(Vec2::new(20.0, 20.0), egui::Sense::click());
                        ui.painter().rect_filled(sq_rect, 4.0, c);
                        ui.painter().rect_stroke(
                            sq_rect,
                            4.0,
                            Stroke::new(1.0, state.theme.border_default),
                            StrokeKind::Inside,
                        );
                        if sq_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if sq_resp.on_hover_text(crate::ui::i18n::text(ui, tip)).clicked() {
                            state.theme_editor.editing.bg_top = c;
                            state.theme_editor.editing.bg_bottom = c;
                            push_snapshot = true;
                        }
                    }
                });
            }
        }
        1 => {
            section_title(ui, "settings-background-image", &state.theme);
            let img_picker_id = ui.make_persistent_id("theme_creator_image_picker");
            let img_task = ui
                .ctx()
                .data_mut(|data| data.get_temp::<crate::services::tasks::BackgroundTask<Option<String>>>(img_picker_id));
            if let Some(task) = &img_task {
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(33));
                if let Some(result) = task.take() {
                    ui.ctx()
                        .data_mut(|data| data.remove::<crate::services::tasks::BackgroundTask<Option<String>>>(img_picker_id));
                    if let Ok(Some(file)) = result {
                        state.theme_editor.editing.bg_image_path = file;
                        state.theme_editor.editing.bg_type = "image".to_string();
                        push_snapshot = true;
                    }
                }
            }
            ui.horizontal(|ui| {
                let btn = ui.add_enabled(
                    img_task.is_none(),
                    egui::Button::new(crate::ui::i18n::format(
                        ui,
                        "settings-choose-image-from-computer",
                        &[("icon", FOLDER_OPEN)],
                    )),
                );
                if btn.clicked() {
                    let filter = crate::ui::i18n::text(ui, "design-images").to_owned();
                    let task = crate::services::tasks::BackgroundTask::spawn_with_repaint(ui.ctx(), move || {
                        Ok(rfd::FileDialog::new()
                            .add_filter(filter, &["png", "jpg", "jpeg", "webp"])
                            .pick_file()
                            .map(|file| file.to_string_lossy().into_owned()))
                    });
                    ui.ctx().data_mut(|data| data.insert_temp(img_picker_id, task));
                }
                if img_task.is_some() {
                    ui.spinner();
                }
            });

            if !state.theme_editor.editing.bg_image_path.is_empty() {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(&state.theme_editor.editing.bg_image_path)
                        .size(10.5)
                        .color(state.theme.text_secondary),
                );
            }
        }
        2 => {
            section_title(ui, "settings-background-video", &state.theme);
            ui.add_space(4.0);
            let vid_picker_id = ui.make_persistent_id("theme_creator_video_picker");
            let vid_task = ui
                .ctx()
                .data_mut(|data| data.get_temp::<crate::services::tasks::BackgroundTask<Option<String>>>(vid_picker_id));
            if let Some(task) = &vid_task {
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(33));
                if let Some(result) = task.take() {
                    ui.ctx()
                        .data_mut(|data| data.remove::<crate::services::tasks::BackgroundTask<Option<String>>>(vid_picker_id));
                    if let Ok(Some(file)) = result {
                        state.theme_editor.editing.options.video.path = file;
                        state.theme_editor.editing.bg_type = "video".to_string();
                        push_snapshot = true;
                    }
                }
            }
            ui.horizontal(|ui| {
                let btn = ui.add_enabled(
                    vid_task.is_none(),
                    egui::Button::new(crate::ui::i18n::format(
                        ui,
                        "settings-choose-mp4-video",
                        &[("icon", VIDEO)],
                    )),
                );
                if btn.clicked() {
                    let filter = crate::ui::i18n::text(ui, "settings-videos").to_owned();
                    let task = crate::services::tasks::BackgroundTask::spawn_with_repaint(ui.ctx(), move || {
                        Ok(rfd::FileDialog::new()
                            .add_filter(filter, &["mp4", "webm"])
                            .pick_file()
                            .map(|file| file.to_string_lossy().into_owned()))
                    });
                    ui.ctx().data_mut(|data| data.insert_temp(vid_picker_id, task));
                }
                if vid_task.is_some() {
                    ui.spinner();
                }
            });
            let path = crate::services::video::path(&state.theme_editor.editing);
            if !path.is_empty() {
                ui.label(
                    RichText::new(
                        std::path::Path::new(path)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy(),
                    )
                    .size(11.0)
                    .color(state.theme.text_secondary),
                );
                ui.label(
                    RichText::new(crate::ui::i18n::text(
                        ui,
                        &crate::services::video::status(
                            ui.ctx(),
                            path,
                            &state.theme_editor.editing.options.video,
                        ),
                    ))
                    .size(10.5)
                    .color(state.theme.text_secondary),
                );
            }
            let video = &mut state.theme_editor.editing.options.video;
            if ui
                .checkbox(&mut video.repeat, crate::ui::i18n::text(ui, "settings-loop-continuously"))
                .changed()
            {
                push_snapshot = true;
            }
            if ui
                .checkbox(&mut video.mirror, crate::ui::i18n::text(ui, "settings-flip-horizontally"))
                .changed()
            {
                push_snapshot = true;
            }
            ui.add_space(2.0);
            if property_slider(
                ui,
                "settings-speed",
                &mut video.speed,
                0.25..=3.0,
                "×",
                2,
                0.05,
                &state.theme,
            ) {
                push_snapshot = true;
            }
            ui.add_space(2.0);
            if property_slider(
                ui,
                "settings-saturation",
                &mut video.saturation,
                0.0..=2.0,
                "",
                2,
                0.02,
                &state.theme,
            ) {
                push_snapshot = true;
            }
            ui.add_space(2.0);
            if property_slider(
                ui,
                "settings-contrast",
                &mut video.contrast,
                0.25..=2.5,
                "",
                2,
                0.02,
                &state.theme,
            ) {
                push_snapshot = true;
            }
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_sized(
                    [88.0, 20.0],
                    egui::Label::new(
                        RichText::new(crate::ui::i18n::text(ui, "settings-start-at"))
                            .size(11.5)
                            .color(state.theme.text_secondary),
                    ),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::DragValue::new(&mut video.start)
                                .range(0.0..=f32::MAX)
                                .suffix(" s")
                                .speed(0.5),
                        )
                        .changed()
                    {
                        push_snapshot = true;
                    }
                });
            });
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_sized(
                    [88.0, 20.0],
                    egui::Label::new(
                        RichText::new(crate::ui::i18n::text(ui, "settings-fit"))
                            .size(11.5)
                            .color(state.theme.text_secondary),
                    ),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let combo_w = (ui.available_width() - 4.0).clamp(90.0, 180.0);
                    egui::ComboBox::from_id_salt("video_fit")
                        .selected_text(match video.fit.as_str() {
                            "contain" => crate::ui::i18n::text(ui, "settings-fit-entire-image"),
                            "stretch" => crate::ui::i18n::text(ui, "settings-stretch"),
                            _ => crate::ui::i18n::text(ui, "settings-fill"),
                        })
                        .width(combo_w)
                        .show_ui(ui, |ui| {
                            for (value, key) in [
                                ("cover", "settings-fill"),
                                ("contain", "settings-fit-entire-image"),
                                ("stretch", "settings-stretch"),
                            ] {
                                if ui
                                    .selectable_value(&mut video.fit, value.into(), crate::ui::i18n::text(ui, key))
                                    .changed()
                                {
                                    push_snapshot = true;
                                }
                            }
                        });
                });
            });
            ui.label(
                RichText::new(crate::ui::i18n::text(ui, "settings-muted-background-video-native-playback-with-ffmpeg"))
                    .size(10.5)
                    .color(state.theme.text_secondary),
            );
        }
        _ => {
            if shader_controls(ui, &mut state.theme_editor.editing, &state.theme, &state.settings.plugins) {
                push_snapshot = true;
            }
        }
    }

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(6.0);

    section_title(ui, "settings-lighting-adjustments", &state.theme);

    if property_slider_pct(
        ui,
        "design-darken",
        &mut state.theme_editor.editing.options.background_overlay,
        0.0,
        90.0,
        &state.theme,
    ) {
        push_snapshot = true;
    }

    ui.add_space(2.0);
    if property_slider_pct(
        ui,
        "settings-brightness",
        &mut state.theme_editor.editing.options.background_brightness,
        10.0,
        200.0,
        &state.theme,
    ) {
        push_snapshot = true;
    }

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(crate::ui::i18n::text(ui, "settings-slide-transition"))
                .size(11.5)
                .color(state.theme.text_secondary),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let test_btn = egui::Button::new(
                RichText::new(crate::ui::i18n::format(ui, "settings-test", &[("icon", PLAY)]))
                    .size(11.0)
                    .color(state.theme.text_secondary),
            )
            .fill(state.theme.bg_pill)
            .stroke(Stroke::new(1.0, state.theme.border_default))
            .corner_radius(6.0)
            .min_size(Vec2::new(64.0, 22.0));

            if ui
                .add(test_btn)
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .on_hover_text(crate::ui::i18n::text(ui, "settings-test-the-transition-in-the-preview"))
                .clicked()
            {
                state.theme_editor.sample_index = (state.theme_editor.sample_index + 1) % 3;
            }
        });
    });

    ui.add_space(4.0);

    let transitions = [
        ("fade", CIRCLE_HALF, "settings-dissolve"),
        ("slide", ARROWS_LEFT_RIGHT, "settings-slide"),
        ("zoom", MAGNIFYING_GLASS_PLUS, "settings-zoom-in"),
        ("zoom_out", MAGNIFYING_GLASS_MINUS, "settings-zoom-out"),
        ("blur", DROP, "design-blur"),
        ("wipe", COLUMNS, "settings-horizontal-curtain"),
        ("wipe_down", ROWS, "settings-vertical-curtain"),
        ("split", ARROWS_OUT, "settings-reveal"),
        ("flip", ARROWS_CLOCKWISE, "settings-rotate"),
        ("cube", CUBE, "settings-3d-cube"),
        ("pop", SPARKLE, "settings-bounce"),
        ("cut", SCISSORS, "settings-cut"),
    ];

    let card_gap = 6.0;
    let t_card_w = ((ui.available_width() - card_gap) * 0.5).floor();
    for row in transitions.chunks(2) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = card_gap;
            for (t_id, t_icon, t_label) in row {
                let is_active = if *t_id == "slide" {
                    state.theme_editor.editing.transition.starts_with("slide")
                } else {
                    state.theme_editor.editing.transition == *t_id
                };
                if transition_tile(
                    ui,
                    t_icon,
                    t_label,
                    is_active,
                    Vec2::new(t_card_w, 28.0),
                    &state.theme,
                ) {
                    if *t_id == "slide" {
                        if !state.theme_editor.editing.transition.starts_with("slide") {
                            state.theme_editor.editing.transition = "slide_left".to_string();
                        }
                    } else {
                        state.theme_editor.editing.transition = t_id.to_string();
                    }
                    push_snapshot = true;
                }
            }
        });
        ui.add_space(card_gap);
    }

    if state.theme_editor.editing.transition.starts_with("slide") {
        ui.add_space(2.0);
        ui.label(
            RichText::new(crate::ui::i18n::text(ui, "settings-orientation"))
                .size(11.0)
                .color(state.theme.text_secondary),
        );
        ui.add_space(2.0);
        let cur_dir = match state.theme_editor.editing.transition.as_str() {
            "slide_right" => 1,
            "slide_up" => 2,
            "slide_down" => 3,
            _ => 0,
        };
        if let Some(dir_idx) = super::widgets::segments(
            ui,
            &["design-left", "design-right", "settings-up", "settings-down"],
            cur_dir,
            &state.theme,
        ) {
            state.theme_editor.editing.transition = match dir_idx {
                1 => "slide_right".to_string(),
                2 => "slide_up".to_string(),
                3 => "slide_down".to_string(),
                _ => "slide_left".to_string(),
            };
            push_snapshot = true;
        }
    }

    if state.theme_editor.editing.transition != "cut" {
        ui.add_space(4.0);
        if property_slider(
            ui,
            "design-duration",
            &mut state.theme_editor.editing.options.transition_ms,
            80..=2000,
            " ms",
            0,
            10.0,
            &state.theme,
        ) {
            push_snapshot = true;
        }

        if state.theme_editor.editing.transition == "blur" {
            ui.add_space(3.0);
            if property_slider(
                ui,
                "design-blur",
                &mut state.theme_editor.editing.options.transition_strength,
                1.0..=32.0,
                "",
                1,
                0.2,
                &state.theme,
            ) {
                push_snapshot = true;
            }
        }
        ui.add_space(3.0);
        ui.horizontal(|ui| {
            ui.add_sized(
                [68.0, 20.0],
                egui::Label::new(
                    RichText::new(crate::ui::i18n::text(ui, "settings-easing"))
                        .size(11.5)
                        .color(state.theme.text_secondary),
                ),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let combo_w = (ui.available_width() - 4.0).clamp(80.0, 140.0);
                egui::ComboBox::from_id_salt("transition_curve")
                    .selected_text(
                        match state
                            .theme_editor
                            .editing
                            .options
                            .transition_easing
                            .as_str()
                        {
                            "linear" => crate::ui::i18n::text(ui, "design-linear"),
                            "ease_out" => crate::ui::i18n::text(ui, "design-ease-out"),
                            _ => crate::ui::i18n::text(ui, "design-smooth"),
                        },
                    )
                    .width(combo_w)
                    .show_ui(ui, |ui| {
                        for (value, key) in [
                            ("smooth", "design-smooth"),
                            ("linear", "design-linear"),
                            ("ease_out", "design-ease-out"),
                        ] {
                            if ui
                                .selectable_value(
                                    &mut state.theme_editor.editing.options.transition_easing,
                                    value.into(),
                                    crate::ui::i18n::text(ui, key),
                                )
                                .changed()
                            {
                                push_snapshot = true;
                            }
                        }
                    });
            });
        });
    }

    push_snapshot
}
