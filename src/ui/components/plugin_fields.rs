use crate::core::i18n::{Language, localize};
use crate::core::plugins::{FieldKind, PluginField, PluginInstance};
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, Margin, RichText, Stroke, StrokeKind, Vec2};
use egui_phosphor::regular::*;
use serde_json::{Map, Value};

#[cfg(test)]
#[path = "plugin_fields_tests.rs"]
mod tests;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ButtonRole {
    Action,
    Danger,
    Warning,
    Secondary,
}

fn classify_button(label: &str) -> (ButtonRole, Option<&'static str>) {
    let lower = label.to_ascii_lowercase();
    if lower.contains("zerar")
        || lower.contains("reset")
        || lower.contains("limpar")
        || lower.contains("redefinir")
        || lower.contains("clear")
        || lower.contains("reiniciar")
    {
        (ButtonRole::Danger, Some(ARROW_COUNTER_CLOCKWISE))
    } else if lower.contains("pausar") || lower.contains("pause") || lower.contains("parar") {
        (ButtonRole::Warning, Some(PAUSE))
    } else if lower.contains("iniciar")
        || lower.contains("start")
        || lower.contains("retomar")
        || lower.contains("play")
    {
        (ButtonRole::Action, Some(PLAY))
    } else if lower.contains('+') {
        (ButtonRole::Action, Some(PLUS))
    } else if lower.contains('-') {
        (ButtonRole::Secondary, Some(MINUS))
    } else {
        (ButtonRole::Secondary, None)
    }
}

fn component_icon(id: &str, name: &str) -> &'static str {
    let lower_id = id.to_ascii_lowercase();
    let lower_name = name.to_ascii_lowercase();
    if lower_id.contains("score")
        || lower_name.contains("placar")
        || lower_name.contains("score")
        || lower_name.contains("marcador")
    {
        TROPHY
    } else if lower_id.contains("stopwatch")
        || lower_id.contains("timer")
        || lower_name.contains("cronômetro")
        || lower_name.contains("cronometro")
    {
        TIMER
    } else if lower_id.contains("goal")
        || lower_name.contains("meta")
        || lower_name.contains("progresso")
    {
        TARGET
    } else if lower_id.contains("announce")
        || lower_id.contains("notice")
        || lower_name.contains("aviso")
    {
        MEGAPHONE
    } else {
        PUZZLE_PIECE
    }
}

pub fn check_condition(condition: Option<&str>, values: &Map<String, Value>) -> bool {
    let Some(cond) = condition else {
        return true;
    };
    let parts: Vec<&str> = cond.split_whitespace().collect();
    if parts.len() == 3 {
        let field_val = values.get(parts[0]).and_then(Value::as_f64).unwrap_or(0.0);
        let target = parts[2].parse::<f64>().unwrap_or(0.0);
        match parts[1] {
            ">=" => field_val >= target - 1e-6,
            "<=" => field_val <= target + 1e-6,
            ">" => field_val > target + 1e-6,
            "<" => field_val < target - 1e-6,
            "==" => (field_val - target).abs() < 1e-6,
            "!=" => (field_val - target).abs() >= 1e-6,
            _ => true,
        }
    } else if let Some((left, right)) = cond.split_once("==") {
        let left_val = values
            .get(left.trim())
            .and_then(Value::as_str)
            .unwrap_or("");
        left_val == right.trim().trim_matches('"')
    } else if let Some((left, right)) = cond.split_once("!=") {
        let left_val = values
            .get(left.trim())
            .and_then(Value::as_str)
            .unwrap_or("");
        left_val != right.trim().trim_matches('"')
    } else {
        true
    }
}

fn localize_select_opt<'a>(
    translations: &'a crate::core::i18n::Translations,
    language: Language,
    opt: &'a str,
) -> &'a str {
    let key = format!("opt_{opt}");
    let loc = crate::core::i18n::localize(translations, language, &key, "");
    if !loc.is_empty() {
        return loc;
    }
    match opt {
        "horizontal" => match language {
            Language::English => "Horizontal",
            Language::Spanish => "Horizontal",
            Language::Portuguese => "Horizontal",
        },
        "vertical" => match language {
            Language::English => "Vertical",
            Language::Spanish => "Vertical",
            Language::Portuguese => "Vertical",
        },
        "cards" => match language {
            Language::English => "Cards",
            Language::Spanish => "Tarjetas",
            Language::Portuguese => "Cartões",
        },
        "clean" => match language {
            Language::English => "Minimalist",
            Language::Spanish => "Minimalista",
            Language::Portuguese => "Minimalista",
        },
        "versus" => match language {
            Language::English => "Versus (VS)",
            Language::Spanish => "Duelo (VS)",
            Language::Portuguese => "Duelo (VS)",
        },
        _ => opt,
    }
}

fn render_plugin_button(
    ui: &mut egui::Ui,
    label: &str,
    width: f32,
    role: ButtonRole,
    icon: Option<&'static str>,
    theme: &AppTheme,
) -> egui::Response {
    let height = 30.0;
    let (fill, stroke, text_color) = match role {
        ButtonRole::Action => (
            theme.accent_soft,
            Stroke::new(1.0, theme.accent.linear_multiply(0.35)),
            theme.accent,
        ),
        ButtonRole::Danger => (
            theme.status_danger.linear_multiply(0.12),
            Stroke::new(1.0, theme.status_danger.linear_multiply(0.35)),
            theme.status_danger,
        ),
        ButtonRole::Warning => (
            theme.status_warning_bg,
            Stroke::new(1.0, theme.status_warning_fg.linear_multiply(0.35)),
            theme.status_warning_fg,
        ),
        ButtonRole::Secondary => (
            theme.bg_pill,
            Stroke::new(1.0, theme.border_default),
            theme.text_primary,
        ),
    };

    let text_with_icon = if let Some(ic) = icon {
        if (label.contains('+') && ic == PLUS) || (label.contains('-') && ic == MINUS) {
            label.to_string()
        } else {
            format!("{ic}  {label}")
        }
    } else {
        label.to_string()
    };

    let btn = egui::Button::new(
        RichText::new(text_with_icon)
            .size(12.0)
            .strong()
            .color(text_color),
    )
    .fill(fill)
    .stroke(stroke)
    .corner_radius(7.0)
    .wrap()
    .min_size(Vec2::new(width, height));

    ui.add_sized([width, height], btn)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(label)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PluginFieldFilter {
    All,
    OperationalOnly,
    AppearanceOnly,
}

pub fn component(
    ui: &mut egui::Ui,
    instance: &mut PluginInstance,
    language: Language,
    theme: &AppTheme,
) -> bool {
    component_filtered(ui, instance, language, theme, PluginFieldFilter::All)
}

pub fn component_operational(
    ui: &mut egui::Ui,
    instance: &mut PluginInstance,
    language: Language,
    theme: &AppTheme,
) -> bool {
    component_filtered(ui, instance, language, theme, PluginFieldFilter::OperationalOnly)
}

pub fn component_appearance(
    ui: &mut egui::Ui,
    instance: &mut PluginInstance,
    language: Language,
    theme: &AppTheme,
) -> bool {
    component_filtered(ui, instance, language, theme, PluginFieldFilter::AppearanceOnly)
}

pub fn component_filtered(
    ui: &mut egui::Ui,
    instance: &mut PluginInstance,
    language: Language,
    theme: &AppTheme,
    filter: PluginFieldFilter,
) -> bool {
    let mut changed = false;
    let error_id = ui.make_persistent_id((
        "plugin_control_error",
        &instance.plugin_id,
        &instance.definition.id,
        instance.created_at,
    ));
    let mut action_error = ui.data(|data| data.get_temp::<String>(error_id));

    let card_frame = egui::Frame::new()
        .fill(theme.bg_card)
        .stroke(Stroke::new(1.0, theme.border_default))
        .corner_radius(theme.radius_md)
        .inner_margin(Margin::same(10));

    card_frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        let comp_name = localize(
            &instance.definition.translations,
            language,
            "name",
            &instance.definition.name,
        );
        let comp_desc = localize(
            &instance.definition.translations,
            language,
            "description",
            &instance.definition.description,
        );
        let icon_char = component_icon(&instance.definition.id, comp_name);

        // Header
        ui.horizontal(|ui| {
            let (icon_rect, _) =
                ui.allocate_exact_size(Vec2::new(28.0, 28.0), egui::Sense::hover());
            ui.painter().rect_filled(icon_rect, 6.0, theme.accent_soft);
            ui.painter().text(
                icon_rect.center(),
                egui::Align2::CENTER_CENTER,
                icon_char,
                egui::FontId::proportional(15.0),
                theme.accent,
            );

            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 1.0;
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(comp_name)
                            .size(13.0)
                            .strong()
                            .color(theme.text_primary),
                    );
                    if instance.definition.animated {
                        let badge = language.text("plugins-animated");
                        let badge_width = ui
                            .painter()
                            .layout_no_wrap(
                                badge.to_owned(),
                                egui::FontId::proportional(9.0),
                                theme.accent,
                            )
                            .size()
                            .x
                            + 12.0;
                        let (badge_rect, _) = ui.allocate_exact_size(
                            Vec2::new(badge_width, 16.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(badge_rect, 4.0, theme.accent_soft);
                        ui.painter().text(
                            badge_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            badge,
                            egui::FontId::proportional(9.0),
                            theme.accent,
                        );
                    }
                });
                if !comp_desc.trim().is_empty() {
                    ui.label(RichText::new(comp_desc).size(10.5).color(theme.text_muted));
                }
            });
        });

        for field in &instance.definition.fields {
            instance
                .values
                .entry(field.id.clone())
                .or_insert_with(|| field.default.clone());
        }

        let filtered_fields: Vec<PluginField> = match filter {
            PluginFieldFilter::All => instance.definition.fields.clone(),
            PluginFieldFilter::OperationalOnly => instance
                .definition
                .fields
                .iter()
                .filter(|f| !f.is_appearance())
                .cloned()
                .collect(),
            PluginFieldFilter::AppearanceOnly => instance
                .definition
                .fields
                .iter()
                .filter(|f| f.is_appearance())
                .cloned()
                .collect(),
        };

        if !filtered_fields.is_empty() {
            ui.add_space(6.0);
            let (sep_rect, _) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
            ui.painter().hline(
                sep_rect.x_range(),
                sep_rect.center().y,
                Stroke::new(1.0, theme.border_subtle),
            );
            ui.add_space(6.0);

            changed |= fields(
                ui,
                &filtered_fields,
                &mut instance.values,
                language,
                theme,
            );
        }

        // Action Buttons (suppressed in appearance-only mode)
        let visible_buttons: Vec<&crate::core::plugins::PluginButton> =
            if filter == PluginFieldFilter::AppearanceOnly {
                Vec::new()
            } else {
                instance
                    .definition
                    .buttons
                    .iter()
                    .filter(|btn| check_condition(btn.condition.as_deref(), &instance.values))
                    .collect()
            };

        if !visible_buttons.is_empty() {
            ui.add_space(6.0);
            let (sep_rect, _) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
            ui.painter().hline(
                sep_rect.x_range(),
                sep_rect.center().y,
                Stroke::new(1.0, theme.border_subtle),
            );
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{LIGHTNING}  {}",
                        language.text("plugins-quick-actions")
                    ))
                    .size(10.5)
                    .strong()
                    .color(theme.text_muted),
                );
            });
            ui.add_space(3.0);

            let mut action = None;
            let count = visible_buttons.len();
            let separate_reset = count == 5
                && visible_buttons
                    .last()
                    .is_some_and(|button| classify_button(&button.label).0 == ButtonRole::Danger);
            let regular_count = count - usize::from(separate_reset);
            let total_w = ui.available_width();
            let gap = 6.0;
            let preferred_width = visible_buttons
                .iter()
                .map(|button| {
                    let label = button.localized_label(language);
                    ui.painter()
                        .layout_no_wrap(
                            label.to_owned(),
                            egui::FontId::proportional(12.0),
                            theme.text_primary,
                        )
                        .size()
                        .x
                        + 36.0
                })
                .fold(72.0, f32::max);
            let columns = (((total_w + gap) / (preferred_width + gap)) as usize)
                .clamp(1, regular_count.min(4));
            let btn_w = ((total_w - (columns - 1) as f32 * gap) / columns as f32).min(180.0);

            for row in visible_buttons[..regular_count].chunks(columns) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for button in row {
                        let localized_label =
                            localize(&button.translations, language, "label", &button.label);
                        let (role, icon) = classify_button(&button.label);
                        if render_plugin_button(ui, localized_label, btn_w, role, icon, theme)
                            .clicked()
                        {
                            action = Some((*button).clone());
                        }
                    }
                });
            }
            if separate_reset {
                let button = visible_buttons[count - 1];
                let label = button.localized_label(language);
                let (role, icon) = classify_button(&button.label);
                if render_plugin_button(ui, label, total_w.min(360.0), role, icon, theme).clicked()
                {
                    action = Some(button.clone());
                }
            }

            if let Some(button) = action {
                let result = if instance.definition.javascript.is_some() {
                    crate::core::plugins::javascript::dispatch(
                        instance,
                        button.event.as_deref().unwrap_or(&button.label),
                        &Value::Null,
                    )
                } else {
                    crate::core::plugins::runtime::update(instance, &button.script)
                };
                match result {
                    Ok(()) => changed = true,
                    Err(error) => action_error = Some(error),
                }
            }
        }

        if changed {
            action_error = None;
        }
        let now = crate::core::design::now_ms();
        let render_error = if instance.definition.javascript.is_some() {
            crate::core::plugins::javascript::render(instance, now, 640.0, 360.0).err()
        } else {
            crate::core::plugins::runtime::primitives(instance, now).err()
        };
        if let Some(error) = action_error.as_ref().or(render_error.as_ref()) {
            ui.add_space(6.0);
            egui::Frame::new()
                .fill(theme.status_danger.linear_multiply(0.12))
                .stroke(Stroke::new(1.0, theme.status_danger.linear_multiply(0.35)))
                .corner_radius(6.0)
                .inner_margin(Margin::symmetric(8, 6))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.colored_label(theme.status_danger, WARNING_CIRCLE);
                        ui.label(
                            RichText::new(language.format(
                                "plugins-runtime-error",
                                &[("error", language.text(error))],
                            ))
                            .size(11.0)
                            .color(theme.status_danger),
                        );
                    });
                });
        }
    });
    ui.data_mut(|data| {
        if let Some(error) = action_error {
            data.insert_temp(error_id, error);
        } else {
            data.remove::<String>(error_id);
        }
    });
    if instance.definition.animated {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(100));
    }
    changed
}

pub fn fields(
    ui: &mut egui::Ui,
    schema: &[PluginField],
    values: &mut Map<String, Value>,
    language: Language,
    theme: &AppTheme,
) -> bool {
    let mut changed = false;
    let avail_w = ui.available_width();
    let stacked = avail_w < 360.0;
    let label_w = schema
        .iter()
        .filter(|field| !field.hidden)
        .map(|field| {
            ui.painter()
                .layout_no_wrap(
                    field.localized_label(language).to_owned(),
                    egui::FontId::proportional(11.5),
                    theme.text_secondary,
                )
                .size()
                .x
        })
        .fold(0.0, f32::max)
        .min(180.0)
        .min(avail_w * 0.42);
    let ctrl_w = (avail_w - if stacked { 0.0 } else { label_w + 8.0 })
        .max(0.0)
        .min(480.0);

    // Conditions can refer to fields declared later in the manifest.
    for field in schema {
        values
            .entry(field.id.clone())
            .or_insert_with(|| field.default.clone());
    }
    for field in schema {
        if field.hidden {
            continue;
        }
        if !check_condition(field.condition.as_deref(), values) {
            continue;
        }

        let localized_label = localize(&field.translations, language, "label", &field.label);

        if field.kind == FieldKind::Multiline {
            ui.add_space(2.0);
            ui.label(
                RichText::new(localized_label)
                    .size(11.5)
                    .color(theme.text_secondary),
            );
            ui.add_space(2.0);
            let mut text = values
                .get(&field.id)
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let edit = egui::TextEdit::multiline(&mut text)
                .id(ui.make_persistent_id((&field.id, "multiline")))
                .desired_rows(3)
                .desired_width(f32::INFINITY)
                .margin(Margin::symmetric(8, 6))
                .font(egui::FontId::proportional(11.5));
            if ui.add(edit).changed() && text.len() <= 16_384 {
                values.insert(field.id.clone(), Value::String(text));
                changed = true;
            }
            ui.add_space(2.0);
            continue;
        }

        if stacked {
            ui.label(
                RichText::new(localized_label)
                    .size(11.5)
                    .color(theme.text_secondary),
            );
        }
        ui.push_id(&field.id, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;

                if !stacked {
                    ui.add_sized(
                        [label_w, 28.0],
                        egui::Label::new(
                            RichText::new(localized_label)
                                .size(11.5)
                                .color(theme.text_secondary),
                        )
                        .halign(egui::Align::Min)
                        .truncate(),
                    )
                    .on_hover_text(localized_label);
                }

                match field.kind {
                    FieldKind::Text => {
                        let mut text = values
                            .get(&field.id)
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string();
                        let edit = egui::TextEdit::singleline(&mut text)
                            .desired_width(ctrl_w)
                            .margin(Margin::symmetric(8, 6))
                            .font(egui::FontId::proportional(12.0));
                        let resp = ui.add_sized([ctrl_w, 28.0], edit);
                        if resp.changed() && text.len() <= 16_384 {
                            values.insert(field.id.clone(), Value::String(text));
                            changed = true;
                        }
                    }
                    FieldKind::Number => {
                        let mut num = values.get(&field.id).and_then(Value::as_f64).unwrap_or(0.0);
                        let is_integer = (field.default.is_i64() || field.default.is_u64())
                            && field.step.is_none_or(|step| step.fract() == 0.0);
                        let step = field.step.unwrap_or(if is_integer { 1.0 } else { 0.1 });
                        let min_val = field.min.unwrap_or(-1e9);
                        let max_val = field.max.unwrap_or(1e9);
                        let ctrl_w = ctrl_w.min(220.0);

                        let is_slider_candidate = field.min.is_some()
                            && field.max.is_some()
                            && (max_val - min_val) <= 100.0
                            && (field.id.contains("radius")
                                || field.id.contains("arredondamento")
                                || field.id.contains("opacity")
                                || field.id.contains("blur")
                                || field.id.contains("percent"));

                        let mut field_changed = false;

                        if is_slider_candidate {
                            let val_box_w = 44.0;
                            let slider_w = (ctrl_w - val_box_w - 6.0).max(40.0);
                            ui.spacing_mut().slider_width = slider_w;
                            let mut slider =
                                egui::Slider::new(&mut num, min_val..=max_val).show_value(false);
                            if is_integer || field.step.is_some() {
                                slider = slider.step_by(step);
                            }
                            if ui.add(slider).changed() {
                                field_changed = true;
                            }
                            let mut dv = egui::DragValue::new(&mut num)
                                .range(min_val..=max_val)
                                .speed(step);
                            if is_integer {
                                dv = dv.fixed_decimals(0);
                            }
                            if ui.add_sized([val_box_w, 28.0], dv).changed() {
                                field_changed = true;
                            }
                        } else {
                            let btn_size = Vec2::new(28.0, 28.0);
                            let val_w = (ctrl_w - 2.0 * btn_size.x - 12.0).max(32.0);

                            let can_dec = num > min_val;
                            ui.add_enabled_ui(can_dec, |ui| {
                                let minus_btn = egui::Button::new(
                                    RichText::new(MINUS).size(11.0).color(theme.text_secondary),
                                )
                                .fill(theme.bg_pill)
                                .stroke(Stroke::new(1.0, theme.border_default))
                                .corner_radius(5.0)
                                .min_size(btn_size);
                                if ui
                                    .add(minus_btn)
                                    .on_hover_text(
                                        language.format(
                                            "plugins-decrease",
                                            &[("field", localized_label)],
                                        ),
                                    )
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    num = (num - step).max(min_val);
                                    if is_integer {
                                        num = num.round();
                                    }
                                    field_changed = true;
                                }
                            });

                            let mut drag = egui::DragValue::new(&mut num)
                                .range(min_val..=max_val)
                                .speed(step);
                            if is_integer {
                                drag = drag.fixed_decimals(0);
                            }
                            if ui.add_sized([val_w, btn_size.y], drag).changed() {
                                field_changed = true;
                            }

                            let can_inc = num < max_val;
                            ui.add_enabled_ui(can_inc, |ui| {
                                let plus_btn = egui::Button::new(
                                    RichText::new(PLUS).size(11.0).color(theme.text_secondary),
                                )
                                .fill(theme.bg_pill)
                                .stroke(Stroke::new(1.0, theme.border_default))
                                .corner_radius(5.0)
                                .min_size(btn_size);
                                if ui
                                    .add(plus_btn)
                                    .on_hover_text(
                                        language.format(
                                            "plugins-increase",
                                            &[("field", localized_label)],
                                        ),
                                    )
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    num = (num + step).min(max_val);
                                    if is_integer {
                                        num = num.round();
                                    }
                                    field_changed = true;
                                }
                            });
                        }

                        if field_changed
                            && let Some(val) = serde_json::Number::from_f64(num) {
                                values.insert(
                                    field.id.clone(),
                                    if is_integer {
                                        Value::from(num.round() as i64)
                                    } else {
                                        Value::Number(val)
                                    },
                                );
                                changed = true;
                            }
                    }
                    FieldKind::Toggle => {
                        let mut active = values
                            .get(&field.id)
                            .and_then(Value::as_bool)
                            .unwrap_or(false);

                        let switch_size = Vec2::new(36.0, 20.0);
                        let (rect, mut resp) =
                            ui.allocate_exact_size(switch_size, egui::Sense::click());
                        if resp.clicked() {
                            active = !active;
                            resp.mark_changed();
                            values.insert(field.id.clone(), Value::Bool(active));
                            changed = true;
                        }
                        resp.widget_info(|| {
                            egui::WidgetInfo::selected(
                                egui::Role::CheckBox,
                                ui.is_enabled(),
                                active,
                                localized_label,
                            )
                        });

                        let hover = ui.ctx().animate_bool(resp.id.with("hover"), resp.hovered());
                        let on_anim = ui.ctx().animate_bool(resp.id.with("on"), active);

                        let bg_track = theme.bg_pill.lerp_to_gamma(theme.accent, on_anim);
                        ui.painter().rect_filled(rect, 10.0, bg_track);
                        let border_col = if resp.has_focus() {
                            theme.accent
                        } else if hover > 0.0 {
                            theme.border_active
                        } else {
                            theme.border_default
                        };
                        ui.painter().rect_stroke(
                            rect,
                            10.0,
                            Stroke::new(1.0, border_col),
                            StrokeKind::Inside,
                        );

                        let circle_x =
                            egui::lerp((rect.left() + 10.0)..=(rect.right() - 10.0), on_anim);
                        let circle_center = egui::pos2(circle_x, rect.center().y);
                        ui.painter()
                            .circle_filled(circle_center, 7.0, Color32::WHITE);
                        resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                    }
                    FieldKind::Color => {
                        let text = values
                            .get(&field.id)
                            .and_then(Value::as_str)
                            .unwrap_or("#ffffff")
                            .to_string();

                        let parsed =
                            crate::core::interface_theme::parse_color(&text).unwrap_or([255; 4]);
                        let mut color = Color32::from_rgba_unmultiplied(
                            parsed[0], parsed[1], parsed[2], parsed[3],
                        );

                        if crate::ui::components::color_picker::color_swatch_button_sized(
                            ui,
                            ui.make_persistent_id(&field.id),
                            &mut color,
                            true,
                            Vec2::new(ctrl_w.min(220.0), 28.0),
                            Some(localized_label),
                            theme,
                        ) {
                            let [r, g, b, a] = color.to_srgba_unmultiplied();
                            values.insert(
                                field.id.clone(),
                                Value::String(format!("#{r:02X}{g:02X}{b:02X}{a:02X}")),
                            );
                            changed = true;
                        }
                    }
                    FieldKind::Select => {
                        let mut selected = values
                            .get(&field.id)
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string();

                        let display_text =
                            localize_select_opt(&field.translations, language, &selected);
                        let mut combo_changed = false;
                        egui::ComboBox::from_id_salt(&field.id)
                            .selected_text(
                                RichText::new(display_text)
                                    .size(12.0)
                                    .color(theme.text_primary),
                            )
                            .width(ctrl_w)
                            .show_ui(ui, |ui| {
                                for opt in &field.options {
                                    let opt_label =
                                        localize_select_opt(&field.translations, language, opt);
                                    if ui
                                        .selectable_value(&mut selected, opt.clone(), opt_label)
                                        .changed()
                                    {
                                        combo_changed = true;
                                    }
                                }
                            });
                        if combo_changed {
                            values.insert(field.id.clone(), Value::String(selected));
                            changed = true;
                        }
                    }
                    FieldKind::Multiline => unreachable!(),
                }
            })
        });
        ui.add_space(if stacked { 6.0 } else { 2.0 });
    }
    changed
}
