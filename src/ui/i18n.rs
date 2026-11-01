use crate::core::i18n::Language;
use eframe::egui;

pub fn install(ctx: &egui::Context, language: Language) {
    ctx.data_mut(|data| data.insert_temp(egui::Id::unique("interface_language"), language));
}

pub fn language(ctx: &egui::Context) -> Language {
    ctx.data(|data| data.get_temp(egui::Id::unique("interface_language")))
        .unwrap_or_default()
}

pub fn text<'a>(ui: &egui::Ui, key: &'a str) -> &'a str {
    language(ui.ctx()).text(key)
}

pub fn message(ui: &egui::Ui, source: &str) -> String {
    language(ui.ctx()).message(source)
}

pub fn format(ui: &egui::Ui, id: &str, values: &[(&str, &str)]) -> String {
    language(ui.ctx()).format(id, values)
}

pub fn easing<'a>(ui: &egui::Ui, value: &'a str) -> &'a str {
    text(ui, match value {
        "smooth" => "design-smooth",
        "linear" => "design-linear",
        "ease_in" => "design-ease-in",
        "ease_out" => "design-ease-out",
        "ease_in_out" => "design-ease-in-and-out",
        _ => value,
    })
}

