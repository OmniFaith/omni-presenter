use crate::{
    core::{models::MediaItem, state::ControlPanelState},
    services::{
        db::AppDb,
        media_conversion::{self, AudioFormat, ImageFormat, VideoFormat},
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

fn format_options(
    ui: &mut egui::Ui,
    theme: &crate::ui::AppTheme,
    selected: usize,
    options: [(&str, &str); 2],
) -> Option<usize> {
    let width = ui.available_width();
    let mut changed = None;
    if width >= 330.0 {
        let card_width = (width - ui.spacing().item_spacing.x) / 2.0;
        ui.horizontal(|ui| {
            for (index, (title, description)) in options.iter().enumerate() {
                if format_card(ui, card_width, title, description, selected == index, theme) {
                    changed = Some(index);
                }
            }
        });
    } else {
        for (index, (title, description)) in options.iter().enumerate() {
            if format_card(ui, width, title, description, selected == index, theme) {
                changed = Some(index);
            }
        }
    }
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
    egui::Area::new(egui::Id::unique("media_conversion_activity"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-24.0, -24.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(theme.bg_panel)
                .stroke(egui::Stroke::new(1.0, theme.border_default))
                .corner_radius(10.0)
                .inner_margin(14)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            RichText::new(
                                state.settings.language.text("library-conversion-working"),
                            )
                            .size(12.0)
                            .color(theme.text_primary),
                        );
                    })
                });
        });
}

pub fn render_media_conversion(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    db: Option<&AppDb>,
) {
    conversion_activity(ctx, state);
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
                        if let Some(choice) = format_options(
                            ui,
                            &theme,
                            usize::from(prompt.formats.image == ImageFormat::Webp),
                            [
                                ("PNG", language.text("library-conversion-png-description")),
                                ("WebP", language.text("library-conversion-webp-description")),
                            ],
                        ) {
                            prompt.formats.image = if choice == 0 {
                                ImageFormat::Png
                            } else {
                                ImageFormat::Webp
                            };
                        }
                    }
                    if video {
                        section_title(ui, VIDEO, language.text("library-conversion-video"), &theme);
                        if let Some(choice) = format_options(
                            ui,
                            &theme,
                            usize::from(prompt.formats.video == VideoFormat::LosslessMkv),
                            [
                                ("MP4", language.text("library-conversion-mp4-description")),
                                ("MKV", language.text("library-conversion-mkv-description")),
                            ],
                        ) {
                            prompt.formats.video = if choice == 0 {
                                VideoFormat::Mp4
                            } else {
                                VideoFormat::LosslessMkv
                            };
                        }
                    }
                    if audio {
                        section_title(
                            ui,
                            MUSIC_NOTE,
                            language.text("library-conversion-audio"),
                            &theme,
                        );
                        if let Some(choice) = format_options(
                            ui,
                            &theme,
                            usize::from(prompt.formats.audio == AudioFormat::Mp3),
                            [
                                ("FLAC", language.text("library-conversion-flac-description")),
                                ("MP3", language.text("library-conversion-mp3-description")),
                            ],
                        ) {
                            prompt.formats.audio = if choice == 0 {
                                AudioFormat::Flac
                            } else {
                                AudioFormat::Mp3
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
            state.media_conversion_running = true;
            state.notify(language.text("library-conversion-working"));
            state.media_conversion_retry = Some(prompt.clone());
            state.media_import_task = Some((
                prompt.folder,
                BackgroundTask::spawn(move || {
                    let directory = std::env::current_dir()
                        .map_err(|error| error.to_string())?
                        .join("converted-media");
                    media_conversion::convert(prompt.paths, prompt.formats, &directory)
                }),
            ));
        }
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
