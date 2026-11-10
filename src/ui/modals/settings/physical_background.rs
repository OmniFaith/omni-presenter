use super::components::{ButtonTone, card, card_header, setting_toggle, styled_button};
use crate::core::{
    models::PresentationTheme,
    preferences::{AppSettings, PhysicalBackground, VirtualScreen},
};
use crate::services::db::AppDb;
use crate::services::monitor::MonitorInfo;
use crate::ui::AppTheme;
use eframe::egui::{self, RichText, Stroke};
use egui_phosphor::regular::{
    CHECK, CHECK_SQUARE, CHECKERBOARD, CIRCLE, FOLDER_OPEN, IMAGE, PALETTE, SQUARE, STACK,
    TELEVISION_SIMPLE, VIDEO,
};

pub(super) fn render(
    ui: &mut egui::Ui,
    draft: &mut AppSettings,
    themes: &[PresentationTheme],
    theme: &AppTheme,
    db: Option<&AppDb>,
    monitors: &[MonitorInfo],
) {
    let multi_id = ui.make_persistent_id("physical_wallpaper_selected_monitors");
    let legacy_target_id = ui.make_persistent_id("physical_wallpaper_target_monitor");

    let mut selected_monitors: Vec<String> = ui
        .ctx()
        .data_mut(|d| d.get_temp(multi_id))
        .unwrap_or_else(|| {
            ui.ctx().data_mut(|d| {
                if let Some(target) = d.get_temp::<String>(legacy_target_id) {
                    if target != "all" {
                        return vec![target];
                    }
                }
                Vec::new()
            })
        });

    selected_monitors.retain(|name| monitors.iter().any(|m| &m.name == name));

    let is_all = selected_monitors.is_empty();
    let designs = db.and_then(|db| db.load_designs().ok()).unwrap_or_default();

    card(ui, theme, |ui| {
        card_header(
            ui,
            IMAGE,
            "settings-wallpaper",
            if draft.use_wallpaper_in_presentation {
                "settings-shown-desktop-and-presentation"
            } else {
                "settings-shown-only-when-no-presentation-is-open"
            },
            theme,
        );

        ui.add_space(2.0);
        let mut use_in_pres = draft.use_wallpaper_in_presentation;
        if ui
            .checkbox(
                &mut use_in_pres,
                RichText::new(crate::ui::i18n::text(
                    ui,
                    "settings-use-wallpaper-during-presentation",
                ))
                .size(11.5)
                .color(theme.text_primary),
            )
            .on_hover_text(crate::ui::i18n::text(
                ui,
                "settings-use-wallpaper-during-presentation-desc",
            ))
            .changed()
        {
            draft.use_wallpaper_in_presentation = use_in_pres;
        }

        let mut current_bg = if is_all {
            draft.physical_background.clone()
        } else {
            draft.wallpaper_for_monitor(&selected_monitors[0]).clone()
        };
        let prev_bg = current_bg.clone();

        if monitors.len() > 1 {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);

                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "settings-link-monitors"))
                        .size(11.5)
                        .strong()
                        .color(theme.text_secondary),
                );

                egui::ScrollArea::horizontal()
                    .id_salt("wallpaper_monitors_scroll")
                    .auto_shrink([false, false])
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);

                            let all_label = format!("{} {}", TELEVISION_SIMPLE, crate::ui::i18n::text(ui, "settings-all-monitors-default"));
                            let all_resp = monitor_pill(
                                ui,
                                "mon_pill_all",
                                &all_label,
                                is_all,
                                false,
                                false,
                                theme,
                            );
                            if all_resp.clicked() {
                                selected_monitors.clear();
                            }

                            for (idx, mon) in monitors.iter().enumerate() {
                                let is_selected = selected_monitors.contains(&mon.name);
                                let has_custom = draft.has_monitor_wallpaper(&mon.name);
                                let label = format!("{}: {}", idx + 1, mon.display_name);
                                let pill_resp = monitor_pill(
                                    ui,
                                    &format!("mon_pill_{}", mon.name),
                                    &label,
                                    is_selected,
                                    true,
                                    has_custom,
                                    theme,
                                );
                                if pill_resp.clicked() {
                                    if is_selected {
                                        selected_monitors.retain(|n| n != &mon.name);
                                    } else {
                                        selected_monitors.push(mon.name.clone());
                                    }
                                }
                            }
                        });
                    });
            });

            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);

                if is_all {
                    let custom_count = monitors.iter().filter(|m| draft.has_monitor_wallpaper(&m.name)).count();
                    let status = if custom_count > 0 {
                        format!("{} ({} {})", crate::ui::i18n::text(ui, "settings-using-default-wallpaper"), custom_count, crate::ui::i18n::text(ui, "settings-custom-status"))
                    } else {
                        crate::ui::i18n::text(ui, "settings-using-default-wallpaper").to_string()
                    };
                    ui.label(RichText::new(status).size(11.0).color(theme.text_secondary));
                } else if selected_monitors.len() == 1 {
                    let mon_name = &selected_monitors[0];
                    let has_custom = draft.has_monitor_wallpaper(mon_name);
                    let status = if has_custom { crate::ui::i18n::text(ui, "settings-custom-for-this-monitor") } else { crate::ui::i18n::text(ui, "settings-using-default-wallpaper") };
                    ui.label(RichText::new(status).size(11.0).color(if has_custom { theme.accent } else { theme.text_secondary }));

                    if has_custom && action_pill_btn(ui, crate::ui::i18n::text(ui, "settings-reset-to-default"), Some(crate::ui::i18n::text(ui, "settings-reset-to-default-tooltip")), theme) {
                        draft.remove_monitor_wallpaper(mon_name);
                    }
                } else {
                    let count = selected_monitors.len();
                    let custom_count = selected_monitors.iter().filter(|m| draft.has_monitor_wallpaper(m)).count();
                    let status = format!("{} {} {} ({})", CHECK, count, crate::ui::i18n::text(ui, "settings-monitors-selected"), crate::ui::i18n::text(ui, "settings-changes-apply-to-all-selected"));
                    ui.label(RichText::new(status).size(11.0).color(theme.accent));

                    if custom_count > 0 && action_pill_btn(ui, crate::ui::i18n::text(ui, "settings-reset-to-default"), Some(crate::ui::i18n::text(ui, "settings-reset-to-default-tooltip")), theme) {
                        for m in &selected_monitors {
                            draft.remove_monitor_wallpaper(m);
                        }
                    }
                }

                if !is_all && action_pill_btn(ui, crate::ui::i18n::text(ui, "settings-apply-to-all-monitors"), Some(crate::ui::i18n::text(ui, "settings-apply-to-all-monitors-desc")), theme) {
                    draft.physical_background = current_bg.clone();
                    draft.monitor_backgrounds.clear();
                    selected_monitors.clear();
                }

                if selected_monitors.len() > 1 && action_pill_btn(ui, crate::ui::i18n::text(ui, "settings-clear-selection"), None, theme) {
                    selected_monitors.clear();
                }
            });

            ui.ctx().data_mut(|d| {
                d.insert_temp(multi_id, selected_monitors.clone());
                let legacy_val = if selected_monitors.is_empty() {
                    "all".to_string()
                } else {
                    selected_monitors[0].clone()
                };
                d.insert_temp(legacy_target_id, legacy_val);
            });
            ui.add_space(4.0);
        }

        ui.add_space(5.0);
        let idle_theme_id = draft.service().idle_theme_id.clone();
        render_wallpaper_body(
            ui,
            &mut current_bg,
            theme,
            &idle_theme_id,
            &designs,
            themes,
            &mut draft.recent_wallpapers,
            db,
            "physical",
        );

        if current_bg != prev_bg {
            if is_all {
                draft.physical_background = current_bg;
            } else {
                for mon_name in &selected_monitors {
                    *draft.wallpaper_for_monitor_mut(mon_name) = current_bg.clone();
                }
            }
        }
    });
}

fn monitor_pill(
    ui: &mut egui::Ui,
    id_salt: &str,
    label: &str,
    selected: bool,
    show_checkbox: bool,
    has_custom: bool,
    theme: &AppTheme,
) -> egui::Response {
    let font_size = 11.5;
    let bg = if selected {
        theme.accent_soft
    } else {
        theme.bg_pill
    };
    let stroke = if selected {
        Stroke::new(1.5, theme.accent)
    } else if has_custom {
        Stroke::new(1.0, theme.accent.linear_multiply(0.7))
    } else {
        Stroke::new(1.0, theme.border_default)
    };
    let text_color = if selected {
        theme.text_primary
    } else {
        theme.text_secondary
    };

    let check_icon = if !show_checkbox {
        ""
    } else if selected {
        CHECK_SQUARE
    } else {
        SQUARE
    };
    let prefix = if check_icon.is_empty() { "" } else { " " };

    let custom_badge = if has_custom && !selected {
        " •"
    } else {
        ""
    };

    let full_text = format!("{check_icon}{prefix}{label}{custom_badge}");

    ui.push_id(id_salt, |ui| {
        let btn = egui::Button::new(
            RichText::new(full_text)
                .size(font_size)
                .color(text_color),
        )
        .fill(bg)
        .stroke(stroke)
        .corner_radius(5.0)
        .min_size(egui::vec2(0.0, 24.0));

        ui.add(btn)
    })
    .inner
}

fn action_pill_btn(ui: &mut egui::Ui, text: &str, tooltip: Option<&str>, theme: &AppTheme) -> bool {
    let btn = egui::Button::new(RichText::new(text).size(11.0).color(theme.text_secondary))
        .fill(theme.bg_pill)
        .stroke(Stroke::new(1.0, theme.border_default))
        .corner_radius(4.0)
        .min_size(egui::vec2(0.0, 22.0));
    let mut resp = ui.add(btn);
    if let Some(tip) = tooltip {
        resp = resp.on_hover_text(tip);
    }
    resp.clicked()
}

pub(crate) fn styled_pill_tab(
    ui: &mut egui::Ui,
    text: &str,
    selected: bool,
    has_custom: bool,
    theme: &AppTheme,
) -> egui::Response {
    let label = if has_custom && !selected { format!("{text} •") } else { text.to_string() };
    let color = if selected { theme.accent } else if has_custom { theme.text_primary } else { theme.text_secondary };
    let bg = if selected { theme.accent_soft } else { theme.bg_pill };
    let stroke = if selected { Stroke::new(1.0, theme.accent) } else { Stroke::new(1.0, theme.border_default) };
    ui.add(egui::Button::new(RichText::new(label).size(11.5).color(color)).fill(bg).stroke(stroke).corner_radius(6.0).min_size(egui::vec2(0.0, 26.0)))
}

pub(crate) fn render_wallpaper_body(
    ui: &mut egui::Ui,
    current_bg: &mut PhysicalBackground,
    theme: &AppTheme,
    default_theme_id: &str,
    designs: &[crate::core::design::DesignDocument],
    themes: &[PresentationTheme],
    recent_wallpapers: &mut Vec<String>,
    db: Option<&AppDb>,
    id_prefix: &str,
) {
    let preview_width = (ui.available_width() * 0.32).clamp(140.0, 210.0);
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 16.0;
        ui.allocate_ui_with_layout(
            egui::vec2(preview_width, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(preview_width, preview_width * 9.0 / 16.0),
                    egui::Sense::hover(),
                );
                ui.painter()
                    .rect_filled(rect.expand(3.0), 9.0, theme.bg_app);
                let painter = ui.painter().with_clip_rect(rect);
                let tile = 12.0;
                for y in 0..(rect.height() / tile).ceil() as i32 {
                    for x in 0..(rect.width() / tile).ceil() as i32 {
                        painter.rect_filled(
                            egui::Rect::from_min_size(
                                rect.min + egui::vec2(x as f32 * tile, y as f32 * tile),
                                egui::Vec2::splat(tile),
                            ),
                            0.0,
                            if (x + y) % 2 == 0 {
                                theme.bg_pill
                            } else {
                                theme.bg_card
                            },
                        );
                    }
                }
                let design = match current_bg {
                    PhysicalBackground::Design { id } => {
                        designs.iter().find(|design| design.id == *id)
                    }
                    _ => None,
                };
                crate::ui::presentation::paint_wallpaper(
                    &painter,
                    rect,
                    current_bg,
                    design,
                    themes,
                );
                ui.painter().rect_stroke(
                    rect.expand(3.0),
                    9.0,
                    Stroke::new(1.0, theme.border_default),
                    egui::StrokeKind::Inside,
                );
            },
        );

        ui.vertical(|ui| {
            ui.set_width(ui.available_width());
            let grid_id = format!("wp_types_{id_prefix}");
            kind_picker(ui, current_bg, theme, default_theme_id, &grid_id);
            ui.add_space(3.0);
            match current_bg {
                PhysicalBackground::Transparent => {
                    detail(
                        ui,
                        crate::ui::i18n::text(ui, "settings-keep-the-desktop-visible-without-covering-it-with-a-background"),
                        theme,
                    );
                }
                PhysicalBackground::Color { rgb } => {
                    ui.horizontal(|ui| {
                        ui.color_edit_button_srgb(rgb);
                        detail(
                            ui,
                            &format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]),
                            theme,
                        );
                    });
                }
                background @ (PhysicalBackground::Image { .. }
                | PhysicalBackground::Video { .. }) => {
                    file_picker(ui, background, theme, recent_wallpapers, db, id_prefix);
                }
                PhysicalBackground::Design { id } => {
                    let default_label = crate::ui::i18n::text(ui, "settings-choose-a-design");
                    let selected = designs
                        .iter()
                        .find(|design| design.id == *id)
                        .map(|design| design.name.as_str())
                        .unwrap_or(default_label);
                    egui::ComboBox::from_id_salt(format!("wp_design_{id_prefix}"))
                        .width(ui.available_width())
                        .selected_text(RichText::new(selected).size(12.0))
                        .show_ui(ui, |ui| {
                            for design in designs {
                                ui.selectable_value(id, design.id.clone(), &design.name);
                            }
                        });
                    if designs.is_empty() {
                        detail(ui, crate::ui::i18n::text(ui, "settings-your-saved-designs-appear-here"), theme);
                    }
                }
                PhysicalBackground::Theme { id } => {
                    let default_label = crate::ui::i18n::text(ui, "settings-choose-a-theme");
                    let lang = crate::ui::i18n::language(ui.ctx());
                    let selected = themes
                        .iter()
                        .find(|item| item.id == *id)
                        .map(|item| item.display_name(lang))
                        .unwrap_or_else(|| default_label.to_string());
                    egui::ComboBox::from_id_salt(format!("wp_theme_{id_prefix}"))
                        .width(ui.available_width())
                        .selected_text(RichText::new(&selected).size(12.0))
                        .show_ui(ui, |ui| {
                            for item in themes {
                                let label = item.display_name(lang);
                                ui.selectable_value(id, item.id.clone(), label);
                            }
                        });
                    detail(ui, crate::ui::i18n::text(ui, "settings-use-only-this-theme-s-background-while-on-standby"), theme);
                }
            }
        });
    });
}

fn detail(ui: &mut egui::Ui, text: &str, theme: &AppTheme) {
    ui.label(RichText::new(text).size(11.0).color(theme.text_secondary));
}

fn kind_picker(
    ui: &mut egui::Ui,
    current_bg: &mut PhysicalBackground,
    theme: &AppTheme,
    default_theme_id: &str,
    grid_id: &str,
) {
    let options = [
        (CHECKERBOARD, PhysicalBackground::Transparent),
        (CIRCLE, PhysicalBackground::Color { rgb: [24, 26, 31] }),
        (
            IMAGE,
            PhysicalBackground::Image {
                path: String::new(),
            },
        ),
        (
            VIDEO,
            PhysicalBackground::Video {
                path: String::new(),
            },
        ),
        (STACK, PhysicalBackground::Design { id: String::new() }),
        (
            PALETTE,
            PhysicalBackground::Theme {
                id: default_theme_id.to_string(),
            },
        ),
    ];
    let gap = 6.0;
    let width = (ui.available_width() - gap * 2.0) / 3.0;
    egui::Grid::new(grid_id)
        .spacing(egui::vec2(gap, gap))
        .show(ui, |ui| {
            for (index, (icon, value)) in options.into_iter().enumerate() {
                let label_text = crate::ui::i18n::text(ui, value.label());
                let selected = std::mem::discriminant(current_bg)
                    == std::mem::discriminant(&value);
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(width, 36.0), egui::Sense::click());
                let color = if selected {
                    theme.accent
                } else {
                    theme.text_secondary
                };
                ui.painter().rect_filled(
                    rect,
                    7.0,
                    if selected {
                        theme.accent_soft
                    } else if response.hovered() {
                        theme.bg_card_hover
                    } else {
                        theme.bg_pill
                    },
                );
                ui.painter().rect_stroke(
                    rect,
                    7.0,
                    Stroke::new(
                        1.0,
                        if selected {
                            theme.accent
                        } else {
                            theme.border_default
                        },
                    ),
                    egui::StrokeKind::Inside,
                );
                ui.painter().text(
                    rect.left_center() + egui::vec2(10.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    icon,
                    egui::FontId::proportional(14.0),
                    color,
                );
                ui.painter().text(
                    rect.left_center() + egui::vec2(29.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    label_text,
                    egui::FontId::proportional(11.0),
                    color,
                );
                response.widget_info(|| {
                    egui::WidgetInfo::selected(
                        egui::Role::RadioButton,
                        ui.is_enabled(),
                        selected,
                        label_text,
                    )
                });
                if response
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                    && !selected
                {
                    *current_bg = value;
                }
                if index % 3 == 2 {
                    ui.end_row();
                }
            }
        });
}

fn record_recent_wallpaper(recent_wallpapers: &mut Vec<String>, path: &str) {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return;
    }
    recent_wallpapers.retain(|p| p != trimmed);
    recent_wallpapers.insert(0, trimmed.to_string());
    if recent_wallpapers.len() > 10 {
        recent_wallpapers.truncate(10);
    }
}

fn file_picker(
    ui: &mut egui::Ui,
    background: &mut PhysicalBackground,
    theme: &AppTheme,
    recent_wallpapers: &mut Vec<String>,
    db: Option<&AppDb>,
    id_prefix: &str,
) {
    use crate::services::tasks::BackgroundTask;
    let video = matches!(background, PhysicalBackground::Video { .. });
    let path = match background {
        PhysicalBackground::Image { path } | PhysicalBackground::Video { path } => path,
        _ => return,
    };
    let id = ui.make_persistent_id(("wallpaper_file_picker", video, id_prefix));
    let task = ui
        .ctx()
        .data_mut(|data| data.get_temp::<BackgroundTask<Option<String>>>(id));
    if let Some(task) = &task {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(33));
        if let Some(result) = task.take() {
            ui.ctx()
                .data_mut(|data| data.remove::<BackgroundTask<Option<String>>>(id));
            if let Ok(Some(file)) = result {
                *path = file.clone();
                if !video {
                    record_recent_wallpaper(recent_wallpapers, &file);
                }
            }
        }
    }
    ui.horizontal(|ui| {
        let response = ui
            .add_enabled_ui(task.is_none(), |ui| {
                styled_button(
                    ui,
                    Some(FOLDER_OPEN),
                    "settings-choose-file",
                    ButtonTone::Secondary,
                    theme,
                )
            })
            .inner;
        if response.clicked() {
            let extensions: &[&str] = if video {
                &["mp4", "webm", "mov", "mkv", "avi"]
            } else {
                &["png", "jpg", "jpeg", "webp", "gif", "bmp"]
            };
            let filter = crate::ui::i18n::text(ui, if video { "chat-video" } else { "common-image" }).to_owned();
            let task = BackgroundTask::spawn_with_repaint(ui.ctx(), move || {
                Ok(rfd::FileDialog::new()
                    .add_filter(filter, extensions)
                    .pick_file()
                    .map(|file| file.to_string_lossy().into_owned()))
            });
            ui.ctx().data_mut(|data| data.insert_temp(id, task));
        }
        if task.is_some() {
            ui.spinner();
        }
    });
    if !path.is_empty() {
        let name = std::path::Path::new(&*path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        ui.add(
            egui::Label::new(
                RichText::new(name.as_ref())
                    .size(11.0)
                    .color(theme.text_secondary),
            )
            .truncate(),
        )
        .on_hover_text(path.as_str());
        if !std::path::Path::new(&*path).is_file() {
            ui.colored_label(theme.status_warning_fg, crate::ui::i18n::text(ui, "settings-file-not-found"));
        } else if !video {
            record_recent_wallpaper(recent_wallpapers, path);
        }
    }
    if video {
        detail(ui, crate::ui::i18n::text(ui, "settings-looping-muted-ffmpeg"), theme);
    } else {
        let mut recents: Vec<String> = recent_wallpapers.clone();
        if let Some(db) = db {
            if let Ok(media_items) = db.get_all_media_items() {
                for item in media_items {
                    if item.file_type == "image" && !recents.contains(&item.file_path) {
                        recents.push(item.file_path);
                    }
                }
            }
        }
        recents.retain(|p| !p.trim().is_empty() && std::path::Path::new(p).is_file());

        if !recents.is_empty() {
            ui.add_space(8.0);
            ui.label(
                RichText::new(crate::ui::i18n::text(ui, "settings-recent-images"))
                    .size(11.5)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(4.0);
            egui::ScrollArea::horizontal()
                .id_salt(format!("wallpaper_recent_images_scroll_{id_prefix}"))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        for recent_path in recents.iter().take(8) {
                            let is_selected = path.as_str() == recent_path.as_str();
                            let thumb_size = egui::vec2(68.0, 38.0);
                            let (rect, resp) = ui.allocate_exact_size(thumb_size, egui::Sense::click());

                            ui.painter().rect_filled(rect, 5.0, theme.bg_card);
                            if let Some(texture) = crate::render::get_or_load_thumbnail(ui.ctx(), recent_path) {
                                ui.painter().image(
                                    texture.id(),
                                    rect,
                                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                    egui::Color32::WHITE,
                                );
                            } else {
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    IMAGE,
                                    egui::FontId::proportional(15.0),
                                    theme.text_secondary,
                                );
                            }

                            let stroke_color = if is_selected {
                                theme.accent
                            } else if resp.hovered() {
                                theme.accent_soft
                            } else {
                                theme.border_default
                            };
                            let stroke_width = if is_selected { 2.0 } else { 1.0 };
                            ui.painter().rect_stroke(rect, 5.0, Stroke::new(stroke_width, stroke_color), egui::StrokeKind::Inside);

                            let file_name = std::path::Path::new(recent_path)
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy();
                            let tooltip = format!("{}\n{}", file_name, recent_path);
                            let resp = resp.on_hover_text(tooltip).on_hover_cursor(egui::CursorIcon::PointingHand);
                            if resp.clicked() {
                                *path = recent_path.clone();
                                record_recent_wallpaper(recent_wallpapers, recent_path);
                            }
                        }
                    });
                });
        }
    }
}

pub(super) fn render_virtual_screen_appearance(
    ui: &mut egui::Ui,
    screen: &mut VirtualScreen,
    themes: &[PresentationTheme],
    live_theme: &PresentationTheme,
    theme: &AppTheme,
    db: Option<&AppDb>,
    recent_wallpapers: &mut Vec<String>,
    idle_theme_id: &str,
) {
    let designs = db.and_then(|db| db.load_designs().ok()).unwrap_or_default();
    ui.label(
        RichText::new(crate::ui::i18n::text(ui, "settings-screen-appearance-and-background"))
            .size(12.0)
            .strong()
            .color(theme.accent),
    );

    let is_custom_bg = screen.custom_background.is_some();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(6.0, 4.0);
        let theme_btn = styled_pill_tab(
            ui,
            crate::ui::i18n::text(ui, "settings-background-theme-default"),
            !is_custom_bg,
            false,
            theme,
        );
        if theme_btn.clicked() && is_custom_bg {
            screen.custom_background = None;
        }

        let wp_btn = styled_pill_tab(
            ui,
            crate::ui::i18n::text(ui, "settings-background-custom-wallpaper"),
            is_custom_bg,
            false,
            theme,
        );
        if wp_btn.clicked() && !is_custom_bg {
            screen.custom_background = Some(PhysicalBackground::Image {
                path: String::new(),
            });
        }
    });

    ui.add_space(6.0);

    if let Some(ref mut current_bg) = screen.custom_background {
        render_wallpaper_body(
            ui,
            current_bg,
            theme,
            idle_theme_id,
            &designs,
            themes,
            recent_wallpapers,
            db,
            &screen.id,
        );
    } else {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            let active_theme = if let Some(ref tid) = screen.custom_theme_id {
                themes
                    .iter()
                    .find(|t| &t.id == tid)
                    .unwrap_or(live_theme)
            } else {
                live_theme
            };
            let (rect, _) = ui.allocate_exact_size(egui::vec2(58.0, 32.0), egui::Sense::hover());
            crate::render::paint_theme_bg(ui.painter(), rect, active_theme, 4.0);
            ui.painter().rect_stroke(
                rect,
                4.0,
                Stroke::new(1.0, theme.border_default),
                egui::StrokeKind::Inside,
            );

            let current_name = if let Some(ref tid) = screen.custom_theme_id {
                themes
                    .iter()
                    .find(|t| &t.id == tid)
                    .map(|t| t.name.as_str())
                    .unwrap_or_else(|| crate::ui::i18n::text(ui, "workspace-custom"))
            } else {
                crate::ui::i18n::text(ui, "workspace-follow-the-service-theme-global")
            };

            ui.vertical(|ui| {
                ui.label(
                    RichText::new(crate::ui::i18n::text(ui, "workspace-visual-theme"))
                        .size(12.0)
                        .color(theme.text_secondary),
                );
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt(format!("scr_theme_{}", screen.id))
                        .selected_text(
                            RichText::new(current_name)
                                .size(12.5)
                                .color(theme.text_primary),
                        )
                        .width(220.0)
                        .show_ui(ui, |ui| {
                            let is_global = screen.custom_theme_id.is_none();
                            if ui
                                .selectable_label(
                                    is_global,
                                    crate::ui::i18n::text(ui, "workspace-follow-the-service-theme-global"),
                                )
                                .clicked()
                            {
                                screen.custom_theme_id = None;
                            }
                            ui.separator();
                            let lang = crate::ui::i18n::language(ui.ctx());
                            for t in themes {
                                let is_sel = screen.custom_theme_id.as_deref() == Some(&t.id);
                                ui.horizontal(|ui| {
                                    let (p_rect, _) = ui.allocate_exact_size(
                                        egui::vec2(32.0, 18.0),
                                        egui::Sense::hover(),
                                    );
                                    crate::render::paint_theme_bg(ui.painter(), p_rect, t, 3.0);
                                    let label = t.display_name(lang);
                                    if ui.selectable_label(is_sel, label).clicked() {
                                        screen.custom_theme_id = Some(t.id.clone());
                                    }
                                });
                            }
                        });
                    if screen.custom_theme_id.is_some()
                        && styled_button(
                            ui,
                            None,
                            "workspace-restore-global-theme",
                            ButtonTone::Secondary,
                            theme,
                        )
                        .on_hover_text(
                            crate::ui::i18n::text(ui, "workspace-follow-the-default-service-theme-again"),
                        )
                        .clicked()
                    {
                        screen.custom_theme_id = None;
                    }
                });
            });
        });
    }

    ui.add_space(4.0);
    setting_toggle(
        ui,
        &mut screen.lyrics_only,
        "workspace-transparent-background",
        "workspace-remove-the-background-for-overlays-in-obs-studio-vmix-or",
        theme,
    );
}
