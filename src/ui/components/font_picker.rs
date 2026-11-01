use crate::{
    core::{bible::fold, search::SearchCache},
    ui::theme::AppTheme,
};
use eframe::egui::{self, RichText};
use std::sync::OnceLock;

struct FontCatalog {
    names: Vec<String>,
    normalized: Vec<String>,
}

#[derive(Clone, Default)]
struct FontPickerState {
    query: String,
    matches: SearchCache,
    was_open: bool,
}

pub fn font_picker(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    selected: &mut String,
    width: f32,
    theme: &AppTheme,
) -> bool {
    picker(ui, id, selected, width, theme, false)
}

pub fn font_picker_with_theme(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    selected: &mut String,
    width: f32,
    theme: &AppTheme,
) -> bool {
    picker(ui, id, selected, width, theme, true)
}

fn font_label(name: &str, language: crate::core::i18n::Language) -> &str {
    match name {
        crate::core::models::fonts::THEME_FONT => language.text("font-theme-default"),
        "Proportional" => language.text("font-default"),
        _ => name,
    }
}

fn picker(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    selected: &mut String,
    width: f32,
    theme: &AppTheme,
    include_theme: bool,
) -> bool {
    let language = crate::ui::i18n::language(ui.ctx());
    let id = ui.make_persistent_id(("font_picker", id));
    let mut state = ui
        .ctx()
        .data(|data| data.get_temp::<FontPickerState>(id))
        .unwrap_or_default();
    let label = font_label(selected, language);
    let response = ui.add_sized(
        [width, 30.0],
        egui::Button::new(
            RichText::new(format!("{label}  {}", egui_phosphor::regular::CARET_DOWN))
                .size(12.0)
                .color(theme.text_primary),
        )
        .fill(theme.bg_pill)
        .stroke(egui::Stroke::new(1.0, theme.border_default))
        .corner_radius(7.0),
    );
    let mut chosen = None;
    let mut open = false;
    egui::Popup::menu(&response).show(|ui| {
        open = true;
        ui.set_width(width.max(235.0));
        let edit = super::controls::search_field(
            ui,
            "font_query",
            &mut state.query,
            language.text("font-search"),
            ui.available_width(),
            theme,
        );
        if !state.was_open {
            edit.request_focus();
        }
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(2.0);
        let fonts = font_catalog();
        let matches: Vec<usize> = state
            .matches
            .filter(&state.query, 0, fonts.normalized.iter().map(String::as_str))
            .iter()
            .copied()
            .filter(|index| {
                include_theme || fonts.names[*index] != crate::core::models::fonts::THEME_FONT
            })
            .collect();
        if ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Enter))
            && let Some(index) = matches.first() {
                chosen = Some(fonts.names[*index].clone());
                ui.close();
            }
        if matches.is_empty() {
            ui.label(
                RichText::new(language.text("font-empty"))
                    .size(12.0)
                    .color(theme.text_muted),
            );
        }
        egui::ScrollArea::vertical()
            .id_salt(id.with("results"))
            .max_height(260.0)
            .show_rows(ui, 24.0, matches.len(), |ui, rows| {
                for row in rows {
                    let name = &fonts.names[matches[row]];
                    let label = font_label(name, language);
                    if ui
                        .add_sized(
                            [ui.available_width(), 24.0],
                            egui::Button::new(RichText::new(label).size(12.0))
                                .selected(selected == name)
                                .frame(false),
                        )
                        .clicked()
                    {
                        chosen = Some(name.clone());
                        ui.close();
                    }
                }
            });
    });
    let changed = chosen.is_some();
    if let Some(name) = chosen {
        *selected = name;
        if selected != crate::core::models::fonts::THEME_FONT {
            crate::core::models::ensure_font_loaded_in_egui(ui.ctx(), selected);
        }
        state.query.clear();
    }
    state.was_open = open;
    ui.ctx().data_mut(|data| data.insert_temp(id, state));
    changed
}

pub fn font_weight_picker(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    font_name: &str,
    weight: &mut u16,
    theme: &AppTheme,
) -> bool {
    let language = crate::ui::i18n::language(ui.ctx());
    let available_weights = crate::core::models::fonts::available_font_weights(font_name);
    let mut changed = false;

    if !available_weights.is_empty() && !available_weights.contains(weight)
        && let Some(&closest) = available_weights
            .iter()
            .min_by_key(|&&w| (w as i32 - *weight as i32).abs())
        {
            *weight = closest;
            changed = true;
        }

    let label = |weight: u16| {
        let key = format!("font.weight.{weight}");
        let name = language.text(&key);
        if name == key {
            weight.to_string()
        } else {
            format!("{weight} — {name}")
        }
    };
    ui.horizontal(|ui| {
        ui.add_sized(
            [68.0, 24.0],
            egui::Label::new(
                RichText::new(language.text("font-weight"))
                    .size(11.5)
                    .color(theme.text_secondary),
            )
            .halign(egui::Align::Min),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            egui::ComboBox::from_id_salt(id)
                .selected_text(label(*weight))
                .width((ui.available_width() - 4.0).clamp(100.0, 180.0))
                .show_ui(ui, |ui| {
                    for &value in &available_weights {
                        changed |= ui.selectable_value(weight, value, label(value)).changed();
                    }
                });
        });
    });
    changed
}

fn font_catalog() -> &'static FontCatalog {
    static CATALOG: OnceLock<FontCatalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let mut names: Vec<String> = [crate::core::models::fonts::THEME_FONT, "Proportional", "Serif", "Monospace", "Montserrat", "Inter", "Roboto"].into_iter().map(str::to_owned).collect();
        let mut seen: std::collections::HashSet<_> = names.iter().cloned().collect();
        for name in crate::core::models::get_system_font_names() { if seen.insert(name.clone()) { names.push(name.clone()); } }
        let normalized = names.iter().map(|name| {
            if name == crate::core::models::fonts::THEME_FONT {
                fold("Theme default padrão do tema predeterminada del tema")
            } else if name == "Proportional" { fold("Proportional interface default padrão da interface predeterminada de la interfaz") } else { fold(name) }
        }).collect();
        FontCatalog { names, normalized }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_search_matches_case_and_accents_in_all_languages() {
        let fonts = font_catalog();
        let mut cache = SearchCache::default();
        for query in ["PADRÃO", "default", "predeterminada"] {
            assert!(
                cache
                    .filter(query, 0, fonts.normalized.iter().map(String::as_str))
                    .iter()
                    .any(|index| fonts.names[*index] == "Proportional")
            );
        }
        assert!(
            cache
                .filter("MoNtSeRrAt", 0, fonts.normalized.iter().map(String::as_str))
                .iter()
                .any(|index| fonts.names[*index] == "Montserrat")
        );
    }
}
