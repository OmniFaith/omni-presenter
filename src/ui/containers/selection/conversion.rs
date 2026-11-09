use crate::{
    core::{models::MediaItem, state::ControlPanelState},
    services::{
        db::AppDb,
        media_conversion::{self, AudioFormat, ImageFormat, VideoAspectRatio, VideoFormat, VideoResolution},
        tasks::BackgroundTask,
    },
    ui::components::dialog,
};
use eframe::egui::{self, RichText};
use egui_phosphor::regular::{CHECK_CIRCLE, FILE, IMAGE, MUSIC_NOTE, VIDEO};

fn format_card(
    ui: &mut egui::Ui,
    width: f32,
    title: &str,
    description: &str,
    selected: bool,
    theme: &crate::ui::AppTheme,
) -> bool {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 70.0), egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::Role::RadioButton,
            ui.is_enabled(),
            selected,
            format!("{title}: {description}"),
        )
    });
    let hover = ui
        .ctx()
        .animate_bool(response.id.with("hover"), response.hovered());
    let active = ui.ctx().animate_bool(response.id.with("active"), selected);
    let fill = theme
        .bg_pill
        .lerp_to_gamma(theme.accent_soft, active)
        .lerp_to_gamma(theme.bg_card_hover, hover * (1.0 - active) * 0.65);
    let border = theme.border_default.lerp_to_gamma(theme.accent, active);
    ui.painter().rect(
        rect,
        10.0,
        fill,
        egui::Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );
    let painter = ui.painter().with_clip_rect(rect.shrink(3.0));
    painter.text(
        rect.left_top() + egui::vec2(13.0, 12.0),
        egui::Align2::LEFT_TOP,
        title,
        egui::FontId::proportional(14.0),
        if selected {
            theme.accent
        } else {
            theme.text_primary
        },
    );
    painter.text(
        rect.left_top() + egui::vec2(13.0, 40.0),
        egui::Align2::LEFT_TOP,
        description,
        egui::FontId::proportional(10.5),
        theme.text_secondary,
    );
    if selected {
        painter.text(
            rect.right_top() + egui::vec2(-12.0, 12.0),
            egui::Align2::RIGHT_TOP,
            CHECK_CIRCLE,
            egui::FontId::proportional(15.0),
            theme.accent,
        );
    }
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

fn format_cards_grid(
    ui: &mut egui::Ui,
    theme: &crate::ui::AppTheme,
    selected: usize,
    options: &[(&str, &str)],
) -> Option<usize> {
    let width = ui.available_width();
    let mut changed = None;
    let cols = if width >= 380.0 { 2 } else { 1 };
    let card_width = if cols == 2 {
        ((width - ui.spacing().item_spacing.x) / 2.0).max(120.0)
    } else {
        width
    };

    for (row_idx, chunk) in options.chunks(cols).enumerate() {
        ui.horizontal(|ui| {
            for (col_idx, (title, description)) in chunk.iter().enumerate() {
                let index = row_idx * cols + col_idx;
                if format_card(ui, card_width, title, description, selected == index, theme) {
                    changed = Some(index);
                }
            }
        });
        ui.add_space(3.0);
    }
    changed
}

fn resolution_segmented_picker(
    ui: &mut egui::Ui,
    theme: &crate::ui::AppTheme,
    selected: VideoResolution,
    current_dim: Option<&str>,
) -> Option<VideoResolution> {
    let mut changed = None;
    let avail_w = ui.available_width();

    // 1. Linha do Original (Recomendado)
    let original_label = match current_dim {
        Some(dim) => format!("★ Manter original ({dim}) (Recomendado)"),
        None => "★ Manter original (Recomendado)".to_string(),
    };
    let is_orig = selected == VideoResolution::Original;
    let (orig_rect, orig_resp) =
        ui.allocate_exact_size(egui::vec2(avail_w, 26.0), egui::Sense::click());
    if orig_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let orig_fill = if is_orig {
        theme.bg_card_active
    } else if orig_resp.hovered() {
        theme.bg_card_hover
    } else {
        theme.bg_pill
    };
    let orig_stroke = if is_orig {
        egui::Stroke::new(1.0, theme.accent)
    } else {
        egui::Stroke::new(1.0, theme.border_default)
    };
    ui.painter()
        .rect(orig_rect, 6.0, orig_fill, orig_stroke, egui::StrokeKind::Inside);
    ui.painter().text(
        orig_rect.center(),
        egui::Align2::CENTER_CENTER,
        &original_label,
        egui::FontId::proportional(11.5),
        if is_orig {
            theme.accent
        } else if orig_resp.hovered() {
            theme.text_primary
        } else {
            theme.text_secondary
        },
    );
    if orig_resp
        .on_hover_text("Manter a resolução original do arquivo de vídeo")
        .clicked()
    {
        changed = Some(VideoResolution::Original);
    }

    ui.add_space(4.0);

    // 2. Presets de resolução
    let presets = [
        (VideoResolution::Uhd4k, "4K", "3840×2160 pixels (4K UHD)"),
        (VideoResolution::Fhd1080p, "1080p", "1920×1080 (Full HD)"),
        (VideoResolution::Hd720p, "720p", "1280×720 (HD)"),
        (VideoResolution::Sd480p, "480p", "854×480 (SD)"),
    ];
    let count = presets.len() as f32;
    let spacing = 4.0;
    let item_w = ((avail_w - spacing * (count - 1.0)) / count).max(40.0);
    let item_h = 24.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = spacing;
        for (res, label, tooltip) in presets {
            let is_sel = selected == res;
            let (rect, resp) =
                ui.allocate_exact_size(egui::vec2(item_w, item_h), egui::Sense::click());
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            let fill = if is_sel {
                theme.bg_card_active
            } else if resp.hovered() {
                theme.bg_card_hover
            } else {
                theme.bg_pill
            };
            let stroke = if is_sel {
                egui::Stroke::new(1.0, theme.accent)
            } else {
                egui::Stroke::new(1.0, theme.border_default)
            };
            ui.painter()
                .rect(rect, 5.0, fill, stroke, egui::StrokeKind::Inside);
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(11.0),
                if is_sel {
                    theme.accent
                } else if resp.hovered() {
                    theme.text_primary
                } else {
                    theme.text_secondary
                },
            );
            if resp.on_hover_text(tooltip).clicked() {
                changed = Some(res);
            }
        }
    });

    changed
}

fn aspect_ratio_segmented_picker(
    ui: &mut egui::Ui,
    theme: &crate::ui::AppTheme,
    selected: VideoAspectRatio,
) -> Option<VideoAspectRatio> {
    let mut changed = None;
    let avail_w = ui.available_width();

    // 1. Linha do Original (Recomendado)
    let is_orig = selected == VideoAspectRatio::Original;
    let (orig_rect, orig_resp) =
        ui.allocate_exact_size(egui::vec2(avail_w, 26.0), egui::Sense::click());
    if orig_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let orig_fill = if is_orig {
        theme.bg_card_active
    } else if orig_resp.hovered() {
        theme.bg_card_hover
    } else {
        theme.bg_pill
    };
    let orig_stroke = if is_orig {
        egui::Stroke::new(1.0, theme.accent)
    } else {
        egui::Stroke::new(1.0, theme.border_default)
    };
    ui.painter()
        .rect(orig_rect, 6.0, orig_fill, orig_stroke, egui::StrokeKind::Inside);
    ui.painter().text(
        orig_rect.center(),
        egui::Align2::CENTER_CENTER,
        "★ Manter proporção original (Recomendado)",
        egui::FontId::proportional(11.5),
        if is_orig {
            theme.accent
        } else if orig_resp.hovered() {
            theme.text_primary
        } else {
            theme.text_secondary
        },
    );
    if orig_resp
        .on_hover_text("Preserva a proporção original do arquivo sem barras ou cortes")
        .clicked()
    {
        changed = Some(VideoAspectRatio::Original);
    }

    ui.add_space(4.0);

    // 2. Presets de proporção
    let ratios = [
        (VideoAspectRatio::R16x9, "16:9", "Widescreen padrão (16:9)"),
        (VideoAspectRatio::R16x10, "16:10", "Monitores de computador (16:10)"),
        (VideoAspectRatio::R4x3, "4:3", "Proporção tradicional (4:3)"),
        (
            VideoAspectRatio::R9x16,
            "9:16",
            "Formato vertical para celulares e stories (9:16)",
        ),
        (VideoAspectRatio::R21x9, "21:9", "Ultrawide / Cinema (21:9)"),
    ];
    let count = ratios.len() as f32;
    let spacing = 4.0;
    let item_w = ((avail_w - spacing * (count - 1.0)) / count).max(40.0);
    let item_h = 24.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = spacing;
        for (ar, label, tooltip) in ratios {
            let is_sel = selected == ar;
            let (rect, resp) =
                ui.allocate_exact_size(egui::vec2(item_w, item_h), egui::Sense::click());
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            let fill = if is_sel {
                theme.bg_card_active
            } else if resp.hovered() {
                theme.bg_card_hover
            } else {
                theme.bg_pill
            };
            let stroke = if is_sel {
                egui::Stroke::new(1.0, theme.accent)
            } else {
                egui::Stroke::new(1.0, theme.border_default)
            };
            ui.painter()
                .rect(rect, 5.0, fill, stroke, egui::StrokeKind::Inside);
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(10.5),
                if is_sel {
                    theme.accent
                } else if resp.hovered() {
                    theme.text_primary
                } else {
                    theme.text_secondary
                },
            );
            if resp.on_hover_text(tooltip).clicked() {
                changed = Some(ar);
            }
        }
    });

    changed
}

fn section_title(ui: &mut egui::Ui, icon: &str, text: &str, theme: &crate::ui::AppTheme) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(icon).size(15.0).color(theme.accent));
        ui.label(
            RichText::new(text)
                .size(12.0)
                .strong()
                .color(theme.text_primary),
        );
    });
}

fn conversion_activity(ctx: &egui::Context, state: &ControlPanelState) {
    if !state.media_conversion_running {
        return;
    }
    let theme = &state.theme;
    let pct = state.media_conversion_percent();
    let hover_text = state.media_conversion_info_tooltip();
    egui::Area::new(egui::Id::NULL.with("media_conversion_activity"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-24.0, -24.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let resp = egui::Frame::new()
                .fill(theme.bg_panel)
                .stroke(egui::Stroke::new(1.0, theme.border_default))
                .corner_radius(10.0)
                .inner_margin(12)
                .show(ui, |ui| {
                    ui.set_width(280.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(
                                RichText::new(
                                    state.settings.language.text("library-conversion-working"),
                                )
                                .size(12.0)
                                .color(theme.text_primary),
                            );
                            let pct_text = format!("{:.0}%", (pct * 100.0).clamp(0.0, 100.0));
                            let space = (ui.available_width() - 36.0).max(0.0);
                            ui.add_space(space);
                            ui.label(
                                RichText::new(pct_text)
                                    .size(11.5)
                                    .strong()
                                    .color(theme.accent),
                            );
                        });
                        ui.add_space(6.0);
                        let bar_w = 280.0;
                        let bar_h = 4.0;
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(bar_w, bar_h), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, theme.bg_pill);
                        let fill_w = (bar_w * pct.clamp(0.0, 1.0)).max(2.0);
                        let fill_rect =
                            egui::Rect::from_min_size(rect.min, egui::vec2(fill_w, bar_h));
                        ui.painter().rect_filled(fill_rect, 2.0, theme.accent);
                    });
                })
                .response;
            resp.on_hover_text(hover_text);
        });
}

pub fn render_media_conversion(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
) {
    conversion_activity(ctx, state);
    render_media_info_dialog(ctx, state);
    let Some(prompt) = state.media_conversion_prompt.as_mut() else {
        return;
    };
    if let Some(task) = &prompt.ffmpeg_check {
        if let Some(result) = task.take() {
            prompt.ffmpeg_available = Some(result.unwrap_or(false));
            prompt.ffmpeg_check = None;
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
    if let Some(task) = &prompt.details_check {
        if let Some(result) = task.take() {
            prompt.probed_details = result.ok();
            prompt.details_check = None;
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
    let language = state.settings.language;
    let theme = state.theme.clone();
    let mut convert = false;
    let mut original = false;
    let has = |kind: &str| {
        prompt.paths.iter().any(|path| {
            media_conversion::convertible(path)
                && MediaItem::detect_type(&path.to_string_lossy()) == kind
        })
    };
    let (image, video, audio) = (has("image"), has("video"), has("audio"));
    let from_import = prompt.from_import;
    let filename = prompt
        .paths
        .first()
        .and_then(|path| path.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let file_count = prompt.paths.len().to_string();
    let mut cancel = false;
    let close = dialog::show(
        ctx,
        "media_conversion",
        "library-convert-media",
        "",
        500.0,
        &theme,
        |ui| {
            egui::Frame::new()
                .fill(theme.bg_pill)
                .corner_radius(10.0)
                .inner_margin(egui::Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        let (icon, _) =
                            ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::hover());
                        ui.painter().rect_filled(icon, 9.0, theme.accent_soft);
                        ui.painter().text(
                            icon.center(),
                            egui::Align2::CENTER_CENTER,
                            FILE,
                            egui::FontId::proportional(18.0),
                            theme.accent,
                        );
                        ui.vertical(|ui| {
                            let label = if prompt.paths.len() == 1 {
                                filename.clone()
                            } else {
                                language.format(
                                    "library-conversion-file-count",
                                    &[("count", &file_count)],
                                )
                            };
                            ui.add(
                                egui::Label::new(
                                    RichText::new(label)
                                        .size(12.5)
                                        .strong()
                                        .color(theme.text_primary),
                                )
                                .truncate(),
                            );
                            ui.label(
                                RichText::new(language.text("library-conversion-originals-safe"))
                                    .size(11.0)
                                    .color(theme.text_secondary),
                            );
                        });
                    });
                });
            ui.add_space(4.0);
            egui::ScrollArea::vertical()
                .id_salt("media_conversion_formats")
                .max_height((ctx.content_rect().height() - 310.0).max(150.0))
                .show(ui, |ui| {
                    if image {
                        section_title(ui, IMAGE, language.text("library-conversion-image"), &theme);
                        let selected_idx = match prompt.formats.image {
                            ImageFormat::Png => 0,
                            ImageFormat::Webp => 1,
                            ImageFormat::Jpg => 2,
                        };
                        if let Some(choice) = format_cards_grid(
                            ui,
                            &theme,
                            selected_idx,
                            &[
                                ("PNG", language.text("library-conversion-png-description")),
                                ("WebP", language.text("library-conversion-webp-description")),
                                ("JPG", language.text("library-conversion-jpg-description")),
                            ],
                        ) {
                            prompt.formats.image = match choice {
                                1 => ImageFormat::Webp,
                                2 => ImageFormat::Jpg,
                                _ => ImageFormat::Png,
                            };
                        }
                    }
                    if video {
                        section_title(ui, VIDEO, language.text("library-conversion-video"), &theme);
                        let selected_idx = match prompt.formats.video {
                            VideoFormat::Mp4 => 0,
                            VideoFormat::Webm => 1,
                            VideoFormat::LosslessMkv => 2,
                            VideoFormat::Mov => 3,
                        };
                        if let Some(choice) = format_cards_grid(
                            ui,
                            &theme,
                            selected_idx,
                            &[
                                ("MP4", language.text("library-conversion-mp4-description")),
                                ("WebM", language.text("library-conversion-webm-description")),
                                ("MKV", language.text("library-conversion-mkv-description")),
                                ("MOV", language.text("library-conversion-mov-description")),
                            ],
                        ) {
                            prompt.formats.video = match choice {
                                1 => VideoFormat::Webm,
                                2 => VideoFormat::LosslessMkv,
                                3 => VideoFormat::Mov,
                                _ => VideoFormat::Mp4,
                            };
                        }

                        ui.add_space(3.0);
                        let current_dim = prompt.probed_details.as_ref().and_then(|d| d.dimensions_formatted.as_deref());
                        let current_ar = prompt.probed_details.as_ref().and_then(|d| d.aspect_ratio.as_deref());
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(language.text("library-conversion-resolution"))
                                    .size(11.0)
                                    .color(theme.text_secondary),
                            );
                            if let Some(dim) = current_dim {
                                let ar_str = current_ar.map(|a| format!(" ({a})")).unwrap_or_default();
                                ui.label(
                                    RichText::new(format!("• Atual: {dim}{ar_str}"))
                                        .size(10.5)
                                        .strong()
                                        .color(theme.accent),
                                );
                            }
                        });
                        if let Some(res) = resolution_segmented_picker(ui, &theme, prompt.formats.video_resolution, current_dim) {
                            prompt.formats.video_resolution = res;
                        }

                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(language.text("library-conversion-aspect-ratio"))
                                .size(11.0)
                                .color(theme.text_secondary),
                        );
                        if let Some(ar) = aspect_ratio_segmented_picker(ui, &theme, prompt.formats.video_aspect_ratio) {
                            prompt.formats.video_aspect_ratio = ar;
                        }
                    }
                    if audio {
                        section_title(
                            ui,
                            MUSIC_NOTE,
                            language.text("library-conversion-audio"),
                            &theme,
                        );
                        let selected_idx = match prompt.formats.audio {
                            AudioFormat::Mp3 => 0,
                            AudioFormat::Flac => 1,
                            AudioFormat::Wav => 2,
                            AudioFormat::Aac => 3,
                            AudioFormat::Ogg => 4,
                        };
                        if let Some(choice) = format_cards_grid(
                            ui,
                            &theme,
                            selected_idx,
                            &[
                                ("MP3", language.text("library-conversion-mp3-description")),
                                ("FLAC", language.text("library-conversion-flac-description")),
                                ("WAV", language.text("library-conversion-wav-description")),
                                ("AAC", language.text("library-conversion-aac-description")),
                                ("OGG", language.text("library-conversion-ogg-description")),
                            ],
                        ) {
                            prompt.formats.audio = match choice {
                                1 => AudioFormat::Flac,
                                2 => AudioFormat::Wav,
                                3 => AudioFormat::Aac,
                                4 => AudioFormat::Ogg,
                                _ => AudioFormat::Mp3,
                            };
                        }
                    }
                    ui.collapsing(language.text("library-conversion-details"), |ui| {
                        ui.label(
                            RichText::new(language.text("library-conversion-note"))
                                .size(11.0)
                                .color(theme.text_secondary),
                        );
                    });
                });
            if prompt.ffmpeg_available == Some(false) {
                ui.colored_label(
                    theme.text_secondary,
                    language.text("library-conversion-ffmpeg-missing"),
                );
                ui.hyperlink_to(
                    language.text("library-conversion-ffmpeg-download"),
                    "https://ffmpeg.org/download.html",
                );
            } else if prompt.ffmpeg_available.is_none() {
                ui.spinner();
            }
            if from_import {
                ui.checkbox(
                    &mut prompt.dont_ask,
                    language.text("library-conversion-dont-ask"),
                )
                .on_hover_text(language.text("library-conversion-setting-hint"));
            }
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                if from_import {
                    if dialog::action(ui, "library-conversion-import-original", false, &theme)
                        .clicked()
                    {
                        original = true;
                    }
                } else if dialog::action(ui, "common-cancel", false, &theme).clicked() {
                    cancel = true;
                }
                ui.add_enabled_ui(prompt.ffmpeg_available == Some(true), |ui| {
                    let action = if from_import {
                        "library-conversion-and-import"
                    } else {
                        "library-conversion-start"
                    };
                    if dialog::action(ui, action, true, &theme).clicked() {
                        convert = true;
                    }
                });
            });
        },
    );
    if close || cancel || original || convert {
        let prompt = state.media_conversion_prompt.take().unwrap();
        if (original || convert) && prompt.from_import && prompt.dont_ask {
            state.settings.ask_media_conversion = false;
        }
        if original {
            super::actions::finish_import(state, db, prompt.paths, &prompt.folder);
        } else if convert {
            let progress = std::sync::Arc::new(std::sync::Mutex::new(
                crate::services::media_conversion::ConversionProgress {
                    current_file: 0,
                    total_files: prompt.paths.len(),
                    current_filename: String::new(),
                    current_path: std::path::PathBuf::new(),
                    percent: 0.0,
                    details: prompt.probed_details.clone(),
                },
            ));
            state.media_conversion_progress = Some(progress.clone());
            state.media_conversion_running = true;
            state.notify(language.text("library-conversion-working"));
            state.media_conversion_retry = Some(prompt.clone());
            let paths = prompt.paths.clone();
            let formats = prompt.formats;
            state.media_import_task = Some((
                prompt.folder,
                BackgroundTask::spawn(move || {
                    let directory = std::env::current_dir()
                        .map_err(|error| error.to_string())?
                        .join("converted-media");
                    media_conversion::convert_with_progress(paths, formats, &directory, Some(progress))
                }),
            ));
        }
    }
}

pub fn render_media_info_dialog(ctx: &egui::Context, state: &mut ControlPanelState) {
    let Some(details) = state.media_info_dialog.clone() else {
        return;
    };
    let theme = state.theme.clone();
    let mut close = false;

    let dismissed = dialog::show(
        ctx,
        "media_info_dialog",
        "library-media-info",
        "",
        480.0,
        &theme,
        |ui| {
            egui::Frame::new()
                .fill(theme.bg_pill)
                .corner_radius(10.0)
                .inner_margin(12)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        let (icon_rect, _) =
                            ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::hover());
                        ui.painter().rect_filled(icon_rect, 9.0, theme.accent_soft);
                        ui.painter().text(
                            icon_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            FILE,
                            egui::FontId::proportional(18.0),
                            theme.accent,
                        );
                        ui.vertical(|ui| {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&details.title)
                                        .size(13.0)
                                        .strong()
                                        .color(theme.text_primary),
                                )
                                .truncate(),
                            );
                            ui.label(
                                RichText::new(&details.file_type)
                                    .size(11.0)
                                    .color(theme.text_secondary),
                            );
                        });
                    });
                });

            ui.add_space(8.0);

            let rows = [
                ("Caminho", details.path.clone()),
                ("Tamanho", details.file_size_formatted.clone()),
                (
                    "Resolução",
                    details
                        .dimensions_formatted
                        .clone()
                        .unwrap_or_else(|| "—".into()),
                ),
                (
                    "Proporção",
                    details.aspect_ratio.clone().unwrap_or_else(|| "—".into()),
                ),
                (
                    "Duração",
                    details
                        .duration_formatted
                        .clone()
                        .unwrap_or_else(|| "—".into()),
                ),
                (
                    "Codec de vídeo",
                    details.video_codec.clone().unwrap_or_else(|| "—".into()),
                ),
                (
                    "Codec de áudio",
                    details.audio_codec.clone().unwrap_or_else(|| "—".into()),
                ),
                (
                    "Container",
                    details.format_name.clone().unwrap_or_else(|| "—".into()),
                ),
            ];

            egui::Grid::new("media_info_grid")
                .num_columns(2)
                .spacing([16.0, 6.0])
                .show(ui, |ui| {
                    for (label, val) in rows {
                        if val != "—" || label == "Caminho" || label == "Tamanho" {
                            ui.label(
                                RichText::new(label)
                                    .size(11.5)
                                    .color(theme.text_secondary),
                            );
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&val)
                                        .size(11.5)
                                        .color(theme.text_primary),
                                )
                                .truncate(),
                            );
                            ui.end_row();
                        }
                    }
                });

            ui.add_space(10.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if dialog::action(ui, "common-close", true, &theme).clicked() {
                    close = true;
                }
            });
        },
    );

    if dismissed || close {
        state.media_info_dialog = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_prompt_hides_other_formats_and_explains_missing_ffmpeg() {
        let path = std::env::temp_dir().join(format!(
            "{}.png",
            crate::core::preferences::new_id("conversion-prompt")
        ));
        image::RgbaImage::new(2, 2).save(&path).unwrap();
        let mut state = ControlPanelState::default();
        state.settings.language = crate::core::i18n::Language::English;
        let mut prompt = media_conversion::Prompt::new(vec![path.clone()], String::new(), true);
        prompt.ffmpeg_check = None;
        prompt.ffmpeg_available = Some(false);
        state.media_conversion_prompt = Some(prompt);
        let ctx = egui::Context::default();
        crate::ui::i18n::install(&ctx, state.settings.language);
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 800.0),
            )),
            ..Default::default()
        };
        let mut first = ctx.run_ui(input.clone(), |_ui| {
            render_media_conversion(&ctx, &mut state, None)
        });
        first.textures_delta.clear();
        let mut output = ctx.run_ui(input, |_ui| render_media_conversion(&ctx, &mut state, None));
        let text: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text) => Some(text.galley.job.text.clone()),
                _ => None,
            })
            .collect();
        output.textures_delta.clear();
        let language = state.settings.language;
        assert!(
            text.iter()
                .any(|value| value == language.text("library-conversion-image")),
            "{text:?}"
        );
        assert!(
            !text
                .iter()
                .any(|value| value == language.text("library-conversion-video"))
        );
        assert!(
            !text
                .iter()
                .any(|value| value == language.text("library-conversion-audio"))
        );
        assert!(
            text.iter()
                .any(|value| value == language.text("library-conversion-ffmpeg-missing"))
        );
        assert!(
            !text
                .iter()
                .any(|value| value == language.text("library-conversion-note"))
        );
        std::fs::remove_file(path).unwrap();
    }
}
