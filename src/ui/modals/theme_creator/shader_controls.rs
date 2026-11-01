use crate::render::paint_theme_bg;
use eframe::egui::{self, Color32, RichText, Stroke, StrokeKind};
use egui_phosphor::regular::*;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ShaderPickerState {
    pub(super) query: String,
    pub(super) filter_source: String,
    pub(super) compact_mode: bool,
    pub(super) expanded: bool,
}

impl Default for ShaderPickerState {
    fn default() -> Self {
        Self {
            query: String::new(),
            filter_source: "all".to_string(),
            compact_mode: false,
            expanded: false,
        }
    }
}

#[derive(Clone)]
struct ShaderItem {
    id: String,
    name: String,
    description: String,
    source_name: String,
    source_id: String,
    custom: Option<crate::core::plugins::PluginShader>,
}

pub(super) fn shader_controls(
    ui: &mut egui::Ui,
    theme: &mut crate::core::models::PresentationTheme,
    tokens: &crate::ui::theme::AppTheme,
    plugins: &[crate::core::plugins::InstalledPlugin],
) -> bool {
    let mut changed = false;
    let language = crate::ui::i18n::language(ui.ctx());

    // Load or initialize persistent state
    let state_id = egui::Id::unique("theme_shader_picker_state");
    let mut picker_state = ui
        .ctx()
        .data_mut(|d| d.get_temp::<ShaderPickerState>(state_id).unwrap_or_default());

    // Aggregate all shaders (built-in presets + installed plugins)
    let mut all_shaders: Vec<ShaderItem> =
        Vec::with_capacity(crate::services::shaders::PRESETS.len() + 8);

    for (id, name, desc) in crate::services::shaders::PRESETS {
        all_shaders.push(ShaderItem {
            id: id.to_string(),
            name: language.text(name).to_string(),
            description: language.text(desc).to_string(),
            source_name: language.text("shader-native").to_string(),
            source_id: "native".to_string(),
            custom: None,
        });
    }

    for plugin in plugins
        .iter()
        .filter(|p| p.enabled && !p.package.shaders.is_empty())
    {
        for shader in &plugin.package.shaders {
            all_shaders.push(ShaderItem {
                id: format!("{}:{}", plugin.package.id, shader.id),
                name: shader.localized_name(language).to_owned(),
                description: shader.localized_description(language).to_owned(),
                source_name: plugin.package.localized_name(language).to_owned(),
                source_id: plugin.package.id.clone(),
                custom: Some(shader.clone()),
            });
        }
    }

    // Detect currently active shader
    let active_shader_item = all_shaders
        .iter()
        .find(|item| {
            if item.custom.is_some() {
                theme
                    .options
                    .shader
                    .custom
                    .as_ref()
                    .map(|c| {
                        item.custom
                            .as_ref()
                            .map(|ic| ic.id == c.id)
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
                    || theme.options.shader.kind == item.id
            } else {
                theme.options.shader.custom.is_none() && theme.options.shader.kind == item.id
            }
        })
        .cloned();

    super::widgets::section_title(ui, "settings-animated-backgrounds", tokens);
    ui.add_space(2.0);

    // Active Shader Spotlight Banner
    if let Some(active) = &active_shader_item {
        let (rect, resp) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 44.0),
            egui::Sense::hover(),
        );
        ui.painter().rect_filled(rect, 7.0, tokens.bg_card);
        ui.painter().rect_stroke(
            rect,
            7.0,
            Stroke::new(1.0, tokens.border_default),
            StrokeKind::Inside,
        );

        // Mini preview box on the left
        let thumb_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 6.0, rect.top() + 6.0),
            egui::vec2(52.0, 32.0),
        );
        let mut sample = theme.clone();
        sample.id = "active_shader_spotlight_sample".into();
        sample.bg_type = "shader".into();
        paint_theme_bg(ui.painter(), thumb_rect, &sample, 5.0);
        ui.painter().rect_stroke(
            thumb_rect,
            5.0,
            Stroke::new(1.0, tokens.border_default),
            StrokeKind::Inside,
        );

        // Title + badge
        let text_left = thumb_rect.right() + 8.0;
        let title_pos = egui::pos2(text_left, rect.top() + 13.0);
        let title_gal = ui.painter().layout_no_wrap(
            active.name.clone(),
            egui::FontId::proportional(12.0),
            tokens.text_primary,
        );
        let title_w = title_gal.size().x;
        ui.painter().galley(
            title_pos - egui::vec2(0.0, title_gal.size().y * 0.5),
            title_gal,
            tokens.text_primary,
        );

        // Badge pill next to title
        let badge_text = if active.custom.is_some() {
            &active.source_name
        } else {
            language.text("shader-native")
        };
        let badge_font = egui::FontId::proportional(9.0);
        let badge_gal = ui.painter().layout_no_wrap(
            badge_text.to_string(),
            badge_font.clone(),
            tokens.accent,
        );
        let badge_w = badge_gal.size().x + 8.0;
        let badge_rect = egui::Rect::from_min_size(
            egui::pos2(text_left + title_w + 6.0, rect.top() + 6.0),
            egui::vec2(badge_w, 14.0),
        );
        ui.painter().rect_filled(badge_rect, 4.0, tokens.bg_pill);
        ui.painter().rect_stroke(
            badge_rect,
            4.0,
            Stroke::new(1.0, tokens.accent.linear_multiply(0.4)),
            StrokeKind::Inside,
        );
        ui.painter().text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            badge_text,
            badge_font,
            tokens.accent,
        );

        // Description snippet
        let desc_pos = egui::pos2(text_left, rect.top() + 29.0);
        let desc_snippet = if active.description.chars().count() > 36 {
            format!(
                "{}...",
                active.description.chars().take(34).collect::<String>()
            )
        } else {
            active.description.clone()
        };
        ui.painter().text(
            desc_pos,
            egui::Align2::LEFT_CENTER,
            desc_snippet,
            egui::FontId::proportional(10.0),
            tokens.text_muted,
        );

        // Right check icon
        let icon_pos = egui::pos2(rect.right() - 14.0, rect.center().y);
        ui.painter().text(
            icon_pos,
            egui::Align2::CENTER_CENTER,
            CHECK_CIRCLE,
            egui::FontId::proportional(14.0),
            tokens.accent,
        );

        resp.on_hover_text(crate::ui::i18n::format(
            ui,
            "shader-item-description",
            &[
                ("name", &active.name),
                ("description", &active.description),
                ("source", &active.source_name),
            ],
        ));

        ui.add_space(6.0);
    }

    // Toolbar: Search + View Toggles + Expand
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;

        let btn_space = 28.0 * 3.0 + 8.0;
        let search_w = (ui.available_width() - btn_space).max(90.0);
        crate::ui::components::controls::search_field(
            ui,
            "shader_picker_search",
            &mut picker_state.query,
            crate::ui::i18n::text(ui, "settings-search-shader"),
            search_w,
            tokens,
        );

        // Grid mode toggle
        if super::widgets::toggle_icon_button(
            ui,
            SQUARES_FOUR,
            !picker_state.compact_mode,
            "settings-grid-view-2-columns",
            egui::vec2(28.0, 32.0),
            tokens,
        ) {
            picker_state.compact_mode = false;
        }

        // List mode toggle
        if super::widgets::toggle_icon_button(
            ui,
            LIST_DASHES,
            picker_state.compact_mode,
            "settings-compact-list-view",
            egui::vec2(28.0, 32.0),
            tokens,
        ) {
            picker_state.compact_mode = true;
        }

        // Expand / Collapse gallery height toggle
        let expand_icon = if picker_state.expanded {
            ARROWS_IN
        } else {
            ARROWS_OUT
        };
        let expand_tip = if picker_state.expanded {
            "settings-collapse-shader-gallery"
        } else {
            "settings-expand-shader-gallery"
        };
        if super::widgets::toggle_icon_button(
            ui,
            expand_icon,
            picker_state.expanded,
            expand_tip,
            egui::vec2(28.0, 32.0),
            tokens,
        ) {
            picker_state.expanded = !picker_state.expanded;
        }
    });

    ui.add_space(4.0);

    // Filter Chips (Categories / Sources)
    let mut sources: Vec<(String, String, usize)> = Vec::new();
    sources.push(("all".to_string(), crate::ui::i18n::text(ui, "common-all").to_string(), all_shaders.len()));

    let native_count = all_shaders.iter().filter(|s| s.custom.is_none()).count();
    sources.push(("native".to_string(), crate::ui::i18n::text(ui, "settings-native").to_string(), native_count));

    for plugin in plugins
        .iter()
        .filter(|p| p.enabled && !p.package.shaders.is_empty())
    {
        let count = all_shaders
            .iter()
            .filter(|s| s.source_id == plugin.package.id)
            .count();
        if count > 0 {
            sources.push((plugin.package.id.clone(), plugin.package.name.clone(), count));
        }
    }

    if sources.len() > 2 {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
            for (src_id, label, count) in &sources {
                let is_sel = picker_state.filter_source == *src_id;
                let pill_text = format!("{label} ({count})");
                let font = egui::FontId::proportional(10.5);
                let text_size = ui
                    .painter()
                    .layout_no_wrap(pill_text.clone(), font.clone(), Color32::WHITE)
                    .size();
                let pill_w = (text_size.x + 14.0).max(40.0);
                let (rect, resp) =
                    ui.allocate_exact_size(egui::vec2(pill_w, 22.0), egui::Sense::click());

                let bg = if is_sel {
                    tokens.accent
                } else if resp.hovered() {
                    tokens.bg_card_hover
                } else {
                    tokens.bg_pill
                };
                let stroke = if is_sel {
                    Stroke::NONE
                } else if resp.hovered() {
                    Stroke::new(1.0, tokens.border_active)
                } else {
                    Stroke::new(1.0, tokens.border_default)
                };
                let fg = if is_sel {
                    tokens.text_on_accent
                } else if resp.hovered() {
                    tokens.text_primary
                } else {
                    tokens.text_secondary
                };

                ui.painter().rect_filled(rect, 11.0, bg);
                if !stroke.is_empty() {
                    ui.painter()
                        .rect_stroke(rect, 11.0, stroke, StrokeKind::Inside);
                }
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &pill_text,
                    font,
                    fg,
                );

                if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    picker_state.filter_source = src_id.clone();
                }
            }
        });
        ui.add_space(4.0);
    }

    // Filter shaders based on search query and source
    let search_query = crate::core::bible::fold(picker_state.query.trim());

    let filtered_shaders: Vec<&ShaderItem> = all_shaders
        .iter()
        .filter(|item| {
            if picker_state.filter_source != "all"
                && item.source_id != picker_state.filter_source
            {
                return false;
            }
            if !search_query.is_empty() {
                let searchable = crate::core::bible::fold(&format!(
                    "{} {} {} {}",
                    item.name, item.description, item.id, item.source_name
                ));
                if !searchable.contains(&search_query) {
                    return false;
                }
            }
            true
        })
        .collect();

    // Bounded Scrollable Gallery Container
    let gallery_max_h = if picker_state.expanded {
        400.0
    } else {
        220.0
    };

    egui::ScrollArea::vertical()
        .id_salt("theme_shader_gallery_scroll")
        .max_height(gallery_max_h)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            if filtered_shaders.is_empty() {
                ui.add_space(14.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(MAGNIFYING_GLASS)
                            .size(22.0)
                            .color(tokens.icon_muted),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(crate::ui::i18n::text(ui, "settings-no-shaders-found"))
                            .size(11.5)
                            .color(tokens.text_secondary),
                    );
                    ui.add_space(4.0);
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(crate::ui::i18n::text(ui, "settings-clear-filters"))
                                    .size(11.0)
                                    .color(tokens.accent),
                            )
                            .fill(tokens.bg_pill)
                            .corner_radius(6.0),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        picker_state.query.clear();
                        picker_state.filter_source = "all".to_string();
                    }
                });
                ui.add_space(14.0);
            } else if !picker_state.compact_mode {
                // Grid Mode (2 columns)
                let width = ((ui.available_width() - 6.0) * 0.5).floor();
                for pair in filtered_shaders.chunks(2) {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        for item in pair {
                            ui.push_id(&item.id, |ui| {
                                let (rect, response) = ui.allocate_exact_size(
                                    egui::vec2(width, 64.0),
                                    egui::Sense::click(),
                                );
                                let mut sample = theme.clone();
                                sample.id = format!("shader_picker_{}", item.id);
                                sample.bg_type = "shader".to_string();
                                sample.options.shader.kind = item.id.clone();
                                sample.options.shader.custom = item.custom.clone();
                                paint_theme_bg(ui.painter(), rect, &sample, 6.0);

                                let band = egui::Rect::from_min_max(
                                    egui::pos2(rect.left(), rect.bottom() - 20.0),
                                    rect.max,
                                );
                                ui.painter().rect_filled(
                                    band,
                                    egui::CornerRadius {
                                        nw: 0,
                                        ne: 0,
                                        sw: 6,
                                        se: 6,
                                    },
                                    Color32::from_black_alpha(140),
                                );
                                ui.painter().text(
                                    band.center(),
                                    egui::Align2::CENTER_CENTER,
                                    &item.name,
                                    egui::FontId::proportional(11.0),
                                    Color32::WHITE,
                                );

                                let is_active = if item.custom.is_some() {
                                    theme
                                        .options
                                        .shader
                                        .custom
                                        .as_ref()
                                        .map(|c| {
                                            item.custom
                                                .as_ref()
                                                .map(|ic| ic.id == c.id)
                                                .unwrap_or(false)
                                        })
                                        .unwrap_or(false)
                                        || theme.options.shader.kind == item.id
                                } else {
                                    theme.options.shader.custom.is_none()
                                        && theme.options.shader.kind == item.id
                                };

                                if is_active {
                                    ui.painter().rect_stroke(
                                        rect,
                                        6.0,
                                        Stroke::new(2.0, tokens.accent),
                                        StrokeKind::Inside,
                                    );
                                    let check_center =
                                        egui::pos2(rect.right() - 10.0, rect.top() + 10.0);
                                    ui.painter().circle_filled(
                                        check_center,
                                        7.0,
                                        tokens.accent,
                                    );
                                    ui.painter().text(
                                        check_center,
                                        egui::Align2::CENTER_CENTER,
                                        CHECK,
                                        egui::FontId::proportional(9.0),
                                        tokens.text_on_accent,
                                    );
                                } else if response.hovered() {
                                    ui.painter().rect_stroke(
                                        rect,
                                        6.0,
                                        Stroke::new(1.0, tokens.border_active),
                                        StrokeKind::Inside,
                                    );
                                }

                                if response
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .on_hover_text(crate::ui::i18n::format(
                                        ui,
                                        "shader-item-description",
                                        &[
                                            ("name", &item.name),
                                            ("description", &item.description),
                                            ("source", &item.source_name),
                                        ],
                                    ))
                                    .clicked()
                                {
                                    theme.options.shader.kind = item.id.clone();
                                    theme.options.shader.custom = item.custom.clone();
                                    changed = true;
                                }
                            });
                        }
                    });
                    ui.add_space(4.0);
                }
            } else {
                // Compact List Mode (1 column)
                let row_w = ui.available_width();
                for item in &filtered_shaders {
                    ui.push_id(&item.id, |ui| {
                        let is_active = if item.custom.is_some() {
                            theme
                                .options
                                .shader
                                .custom
                                .as_ref()
                                .map(|c| {
                                    item.custom
                                        .as_ref()
                                        .map(|ic| ic.id == c.id)
                                        .unwrap_or(false)
                                        || theme.options.shader.kind == item.id
                                })
                                .unwrap_or(false)
                        } else {
                            theme.options.shader.custom.is_none()
                                && theme.options.shader.kind == item.id
                        };

                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(row_w, 36.0),
                            egui::Sense::click(),
                        );
                        let hovered = response.hovered();

                        let bg = if is_active {
                            tokens.bg_card_active
                        } else if hovered {
                            tokens.bg_card_hover
                        } else {
                            tokens.bg_pill
                        };

                        let border = if is_active {
                            Stroke::new(1.5, tokens.accent)
                        } else if hovered {
                            Stroke::new(1.0, tokens.border_active)
                        } else {
                            Stroke::new(1.0, tokens.border_default)
                        };

                        ui.painter().rect_filled(rect, 6.0, bg);
                        ui.painter()
                            .rect_stroke(rect, 6.0, border, StrokeKind::Inside);

                        // Thumbnail on left
                        let thumb_rect = egui::Rect::from_min_size(
                            egui::pos2(rect.left() + 5.0, rect.top() + 5.0),
                            egui::vec2(44.0, 26.0),
                        );
                        let mut sample = theme.clone();
                        sample.id = format!("shader_picker_list_{}", item.id);
                        sample.bg_type = "shader".to_string();
                        sample.options.shader.kind = item.id.clone();
                        sample.options.shader.custom = item.custom.clone();
                        paint_theme_bg(ui.painter(), thumb_rect, &sample, 4.0);
                        ui.painter().rect_stroke(
                            thumb_rect,
                            4.0,
                            Stroke::new(1.0, tokens.border_default),
                            StrokeKind::Inside,
                        );

                        // Title and description snippet
                        let text_left = thumb_rect.right() + 8.0;
                        let title_color = if is_active {
                            tokens.accent
                        } else {
                            tokens.text_primary
                        };
                        ui.painter().text(
                            egui::pos2(text_left, rect.center().y - 6.0),
                            egui::Align2::LEFT_CENTER,
                            &item.name,
                            egui::FontId::proportional(11.5),
                            title_color,
                        );

                        let sub_text = if !item.description.is_empty() {
                            if item.description.chars().count() > 30 {
                                format!(
                                    "{}...",
                                    item.description
                                        .chars()
                                        .take(28)
                                        .collect::<String>()
                                )
                            } else {
                                item.description.clone()
                            }
                        } else {
                            item.source_name.clone()
                        };
                        ui.painter().text(
                            egui::pos2(text_left, rect.center().y + 7.0),
                            egui::Align2::LEFT_CENTER,
                            sub_text,
                            egui::FontId::proportional(9.5),
                            tokens.text_muted,
                        );

                        // Right indicator
                        if is_active {
                            ui.painter().text(
                                egui::pos2(rect.right() - 12.0, rect.center().y),
                                egui::Align2::CENTER_CENTER,
                                CHECK,
                                egui::FontId::proportional(13.0),
                                tokens.accent,
                            );
                        } else if item.custom.is_some() {
                            let badge_font = egui::FontId::proportional(9.0);
                            let gal = ui.painter().layout_no_wrap(
                                item.source_name.clone(),
                                badge_font.clone(),
                                tokens.text_muted,
                            );
                            let badge_w = gal.size().x + 8.0;
                            let badge_rect = egui::Rect::from_center_size(
                                egui::pos2(
                                    rect.right() - 8.0 - badge_w * 0.5,
                                    rect.center().y,
                                ),
                                egui::vec2(badge_w, 16.0),
                            );
                            ui.painter().rect_filled(badge_rect, 3.0, tokens.border_subtle);
                            ui.painter().text(
                                badge_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                &item.source_name,
                                badge_font,
                                tokens.text_muted,
                            );
                        }

                        if response
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .on_hover_text(crate::ui::i18n::format(
                                ui,
                                "shader-item-description",
                                &[
                                    ("name", &item.name),
                                    ("description", &item.description),
                                    ("source", &item.source_name),
                                ],
                            ))
                            .clicked()
                        {
                            theme.options.shader.kind = item.id.clone();
                            theme.options.shader.custom = item.custom.clone();
                            changed = true;
                        }
                    });
                    ui.add_space(2.0);
                }
            }
        });

    ui.add_space(6.0);

    // Save persistent picker state
    ui.ctx().data_mut(|d| d.insert_temp(state_id, picker_state));

    super::widgets::section_title(ui, "settings-color-palettes", tokens);
    let card_w = ((ui.available_width() - 6.0) * 0.5).floor();
    let palettes: [(&str, [[u8; 3]; 3]); 8] = [
        ("settings-aurora", [[12, 22, 52], [29, 197, 174], [131, 82, 240]]),
        ("settings-ocean", [[8, 31, 55], [32, 115, 179], [63, 211, 224]]),
        ("settings-gold", [[41, 20, 16], [174, 110, 69], [246, 214, 155]]),
        ("settings-cosmic", [[9, 12, 35], [86, 47, 156], [218, 95, 180]]),
        ("settings-embers", [[31, 11, 24], [202, 75, 91], [252, 185, 111]]),
        ("settings-forest", [[10, 35, 20], [34, 197, 94], [134, 239, 172]]),
        ("settings-twilight", [[24, 24, 48], [124, 58, 237], [251, 146, 60]]),
        ("settings-charcoal", [[15, 23, 42], [71, 85, 105], [203, 213, 225]]),
    ];

    for pair in palettes.chunks(2) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            for (label, colors) in pair {
                let (rect, resp) =
                    ui.allocate_exact_size(egui::vec2(card_w, 28.0), egui::Sense::click());
                let is_selected = theme.options.shader.colors == *colors;

                let bg = if is_selected {
                    tokens.bg_card_active
                } else if resp.hovered() {
                    tokens.bg_card_hover
                } else {
                    tokens.bg_pill
                };
                let stroke = if is_selected {
                    Stroke::new(1.5, tokens.accent)
                } else if resp.hovered() {
                    Stroke::new(1.0, tokens.border_active)
                } else {
                    Stroke::new(1.0, tokens.border_default)
                };
                ui.painter().rect_filled(rect, 6.0, bg);
                ui.painter().rect_stroke(rect, 6.0, stroke, StrokeKind::Inside);

                let swatch_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.min.x + 6.0, rect.center().y - 6.0),
                    egui::vec2(27.0, 12.0),
                );
                let stripe_w = 9.0;
                for (i, c) in colors.iter().enumerate() {
                    let stripe = egui::Rect::from_min_size(
                        swatch_rect.min + egui::vec2(i as f32 * stripe_w, 0.0),
                        egui::vec2(stripe_w, 12.0),
                    );
                    let corner = match i {
                        0 => egui::CornerRadius { nw: 3, ne: 0, sw: 3, se: 0 },
                        2 => egui::CornerRadius { nw: 0, ne: 3, sw: 0, se: 3 },
                        _ => egui::CornerRadius::ZERO,
                    };
                    ui.painter()
                        .rect_filled(stripe, corner, Color32::from_rgb(c[0], c[1], c[2]));
                }

                let text_color = if is_selected {
                    tokens.accent
                } else {
                    tokens.text_secondary
                };
                let text_pos = egui::pos2(swatch_rect.max.x + 6.0, rect.center().y);
                ui.painter().text(
                    text_pos,
                    egui::Align2::LEFT_CENTER,
                    crate::ui::i18n::text(ui, label),
                    egui::FontId::proportional(11.0),
                    text_color,
                );

                if resp
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    theme.options.shader.colors = *colors;
                    changed = true;
                }
            }
        });
        ui.add_space(2.0);
    }

    ui.add_space(4.0);

    // Barra de mesclagem das 3 cores ativas
    let preview_h = 24.0;
    let (prev_rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), preview_h),
        egui::Sense::hover(),
    );
    let c0 = Color32::from_rgb(
        theme.options.shader.colors[0][0],
        theme.options.shader.colors[0][1],
        theme.options.shader.colors[0][2],
    );
    let c1 = Color32::from_rgb(
        theme.options.shader.colors[1][0],
        theme.options.shader.colors[1][1],
        theme.options.shader.colors[1][2],
    );
    let c2 = Color32::from_rgb(
        theme.options.shader.colors[2][0],
        theme.options.shader.colors[2][1],
        theme.options.shader.colors[2][2],
    );

    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(prev_rect.left_top(), c0);
    mesh.colored_vertex(egui::pos2(prev_rect.center().x, prev_rect.top()), c1);
    mesh.colored_vertex(prev_rect.right_top(), c2);
    mesh.colored_vertex(prev_rect.right_bottom(), c2);
    mesh.colored_vertex(egui::pos2(prev_rect.center().x, prev_rect.bottom()), c1);
    mesh.colored_vertex(prev_rect.left_bottom(), c0);

    mesh.add_triangle(0, 1, 4);
    mesh.add_triangle(0, 4, 5);
    mesh.add_triangle(1, 2, 3);
    mesh.add_triangle(1, 3, 4);
    ui.painter().add(mesh);

    ui.painter().rect_stroke(
        prev_rect,
        6.0,
        Stroke::new(1.0, tokens.border_default),
        StrokeKind::Inside,
    );

    ui.add_space(6.0);

    // Ajuste individual de cada tom da paleta
    if super::widgets::color_row_u8_3(ui, "settings-background-color", &mut theme.options.shader.colors[0], tokens) {
        changed = true;
    }
    ui.add_space(2.0);
    if super::widgets::color_row_u8_3(ui, "settings-main-light", &mut theme.options.shader.colors[1], tokens) {
        changed = true;
    }
    ui.add_space(2.0);
    if super::widgets::color_row_u8_3(ui, "settings-accent", &mut theme.options.shader.colors[2], tokens) {
        changed = true;
    }
    ui.add_space(2.0);
    let show_properties = super::widgets::collapsible_section_title(
        ui,
        "shader_properties_toggle",
        "settings-properties",
        false,
        tokens,
    );

    if show_properties {
        ui.add_space(2.0);
        let shader = &mut theme.options.shader;
        changed |= super::widgets::property_slider(ui, "settings-speed", &mut shader.speed, 0.0..=3.0, "", 2, 0.05, tokens);
        ui.add_space(2.0);
        changed |= super::widgets::property_slider(ui, "settings-scale-label", &mut shader.scale, 0.2..=6.0, "", 2, 0.05, tokens);
        ui.add_space(2.0);
        changed |= super::widgets::property_slider(ui, "design-intensity", &mut shader.intensity, 0.1..=2.0, "", 2, 0.02, tokens);
        ui.add_space(2.0);
        changed |= super::widgets::property_slider(ui, "settings-smoothness", &mut shader.softness, 0.03..=1.0, "", 2, 0.01, tokens);
        ui.add_space(2.0);

        let detail_label = match shader.kind.as_str() {
            "particles" | "bokeh" => "settings-amount",
            "grid" | "prism" => "settings-density",
            "rings" | "waves" => "settings-frequency",
            _ => "settings-detail",
        };
        changed |= super::widgets::property_slider(ui, detail_label, &mut shader.detail, 1.0..=10.0, "", 1, 0.2, tokens);
        ui.add_space(2.0);
        changed |= super::widgets::property_slider(ui, "settings-direction", &mut shader.direction, 0.0..=360.0, "°", 0, 1.0, tokens);
        ui.add_space(2.0);
        changed |= super::widgets::property_slider(ui, "settings-vignette", &mut shader.vignette, 0.0..=1.0, "", 2, 0.02, tokens);
        ui.add_space(2.0);
        changed |= super::widgets::property_slider(ui, "settings-grain", &mut shader.grain, 0.0..=0.15, "", 3, 0.005, tokens);
        ui.add_space(2.0);

        ui.horizontal(|ui| {
            ui.add_sized(
                [68.0, 20.0],
                egui::Label::new(
                    RichText::new(crate::ui::i18n::text(ui, "settings-variation"))
                        .size(11.5)
                        .color(tokens.text_secondary),
                ),
            );
            let avail_w = ui.available_width();
            let btn_w = 90.0;
            let spacing = ui.spacing().item_spacing.x;
            let dv_w = (avail_w - btn_w - spacing - 2.0).max(40.0);

            if ui
                .add_sized(
                    [dv_w, 20.0],
                    egui::DragValue::new(&mut shader.seed)
                        .range(0.0..=100000.0)
                        .speed(1.0),
                )
                .changed()
            {
                changed = true;
            }
            if ui
                .add_sized(
                    [btn_w, 20.0],
                    egui::Button::new(RichText::new(crate::ui::i18n::text(ui, "settings-new-variation")).size(11.0)),
                )
                .clicked()
            {
                shader.seed = (shader.seed + 31.73) % 100000.0;
                changed = true;
            }
        });
    }
    changed
}
