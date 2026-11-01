//! Portable interface theme packages. Presentation backgrounds are a separate catalogue.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    System,
    Light,
    Dark,
    #[default]
    Legacy,
}

impl ThemeMode {
    pub fn is_dark(self, system_dark: bool, legacy_dark: bool) -> bool {
        match self {
            Self::System => system_dark,
            Self::Light => false,
            Self::Dark => true,
            Self::Legacy => legacy_dark,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InterfaceTheme {
    pub translations: crate::core::i18n::Translations,
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: u32,
    pub light: BTreeMap<String, String>,
    pub dark: BTreeMap<String, String>,
    pub rounding: f32,
}

impl Default for InterfaceTheme {
    fn default() -> Self {
        Self {
            translations: Default::default(),
            id: "classic".into(),
            name: "Omni Presenter".into(),
            author: "Omni Presenter".into(),
            version: 1,
            light: BTreeMap::new(),
            dark: BTreeMap::new(),
            rounding: 8.0,
        }
    }
}

pub const COLOR_KEYS: &[&str] = &[
    "bg_app",
    "bg_panel",
    "bg_card",
    "bg_card_active",
    "bg_card_hover",
    "bg_pill",
    "bg_pill_active",
    "border_subtle",
    "border_default",
    "border_active",
    "splitter_bg",
    "splitter_grip",
    "splitter_grip_hover",
    "text_primary",
    "text_secondary",
    "text_muted",
    "text_on_accent",
    "icon_default",
    "icon_active",
    "icon_muted",
    "accent",
    "accent_soft",
    "status_success_bg",
    "status_success_fg",
    "status_warning_bg",
    "status_warning_fg",
    "status_danger",
];

pub fn parse_color(value: &str) -> Option<[u8; 4]> {
    let value = value.strip_prefix('#')?;
    if !value.is_ascii() || !matches!(value.len(), 6 | 8) {
        return None;
    }
    let mut color = [0, 0, 0, 255];
    for (index, channel) in color.iter_mut().enumerate().take(value.len() / 2) {
        *channel = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(color)
}

impl InterfaceTheme {
    pub fn localized_name(&self, language: crate::core::i18n::Language) -> &str {
        let key = match self.id.as_str() {
            "classic" => Some("appearance-theme-classic"),
            "darker_pro" | "midnight" => Some("appearance-theme-darker-pro"),
            "forest" => Some("appearance-theme-forest"),
            "violet" => Some("appearance-theme-violet"),
            "sand" => Some("appearance-theme-sand"),
            "contrast" => Some("appearance-theme-contrast"),
            _ => None,
        };
        if let Some(key) = key {
            let translated = language.text(key);
            if translated != key {
                return translated;
            }
        }
        crate::core::i18n::localize(&self.translations, language, "name", &self.name)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty()
            || self.id.len() > 128
            || self.name.trim().is_empty()
            || self.name.len() > 200
        {
            return Err("appearance-invalid-metadata".into());
        }
        if self.version != 1 || !self.rounding.is_finite() || !(0.0..=24.0).contains(&self.rounding)
        {
            return Err("appearance-invalid-version".into());
        }
        for palette in [&self.light, &self.dark] {
            for (key, color) in palette {
                if !COLOR_KEYS.contains(&key.as_str()) || parse_color(color).is_none() {
                    return Err(format!("{key}: {color}"));
                }
            }
        }
        Ok(())
    }
}

pub fn resolve_builtin(id: &str) -> Option<InterfaceTheme> {
    let target = if id == "midnight" { "darker_pro" } else { id };
    builtins().into_iter().find(|theme| theme.id == target)
}

pub fn builtins() -> Vec<InterfaceTheme> {
    let mut themes = vec![InterfaceTheme::default()];
    themes.extend(
        [
            (
                "darker_pro",
                "Darker Pro",
                [
                    "#F4F4F5", "#FFFFFF", "#FFFFFF", "#EAEAEA", "#E1E1E3", "#D1D1D4", "#18181B",
                    "#52525B", "#71717A", "#18181B", "#FFFFFF",
                ],
                [
                    "#0E0E0F", "#121213", "#161617", "#1E1E20", "#262629", "#28282B", "#F3F3F4",
                    "#BDBDBD", "#969698", "#E4E4E7", "#121213",
                ],
            ),
            (
                "forest",
                "Sálvia",
                [
                    "#F3F6F4", "#FFFFFF", "#FFFFFF", "#EBF0ED", "#E2EAE5", "#CDD9D1", "#24312A",
                    "#4E6356", "#617769", "#356B50", "#FFFFFF",
                ],
                [
                    "#171B19", "#1D2320", "#252D28", "#2D3831", "#35423A", "#48594D", "#ECF1ED",
                    "#B7C6BC", "#A2B5A8", "#A0C7AE", "#17281D",
                ],
            ),
            (
                "violet",
                "Lavanda",
                [
                    "#F6F4F8", "#FFFFFF", "#FFFFFF", "#EFEBF4", "#E7E1EF", "#D9D0E4", "#302A3C",
                    "#63566F", "#776682", "#735390", "#FFFFFF",
                ],
                [
                    "#1B1820", "#231F2A", "#2B2534", "#342D3F", "#3F354D", "#55465F", "#F2EEF6",
                    "#C7BCCE", "#B4A6BE", "#C9B1DE", "#2A1B37",
                ],
            ),
            (
                "sand",
                "Areia",
                [
                    "#F6F3EE", "#FFFEFB", "#FFFEFB", "#F0EAE1", "#E9E1D6", "#D8CDBD", "#352D24",
                    "#6B5D4C", "#7B6B56", "#805B31", "#FFFFFF",
                ],
                [
                    "#1D1B18", "#25221D", "#2E2922", "#383127", "#443B2F", "#5C4E3C", "#F4EFE6",
                    "#CBC0AF", "#B6AA95", "#D7BD96", "#302315",
                ],
            ),
            (
                "contrast",
                "Alto contraste",
                [
                    "#FFFFFF", "#FFFFFF", "#FFFFFF", "#F0F0F0", "#E3E3E3", "#686868", "#000000",
                    "#202020", "#404040", "#003F9E", "#FFFFFF",
                ],
                [
                    "#000000", "#0D0D0D", "#141414", "#242424", "#303030", "#8C8C8C", "#FFFFFF",
                    "#EFEFEF", "#D0D0D0", "#F0CF7A", "#201A0C",
                ],
            ),
        ]
        .into_iter()
        .map(|(id, name, light, dark)| InterfaceTheme {
            id: id.into(),
            name: name.into(),
            light: palette(false, light),
            dark: palette(true, dark),
            rounding: if id == "contrast" { 6.0 } else { 8.0 },
            ..Default::default()
        }),
    );
    themes
}

fn mix(first: &str, second: &str, amount: f32) -> String {
    let a = parse_color(first).unwrap();
    let b = parse_color(second).unwrap();
    let rgb: [u8; 3] = std::array::from_fn(|i| {
        (a[i] as f32 * (1.0 - amount) + b[i] as f32 * amount).round() as u8
    });
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

fn palette(dark: bool, colors: [&str; 11]) -> BTreeMap<String, String> {
    let [
        bg,
        panel,
        card,
        pill,
        hover,
        border,
        text,
        secondary,
        muted,
        accent,
        on_accent,
    ] = colors;
    let soft = mix(card, accent, if dark { 0.13 } else { 0.08 });
    let success = if dark { "#97C8AC" } else { "#23683E" };
    let warning = if dark { "#D8BD8D" } else { "#815A22" };
    [
        ("bg_app", bg.into()),
        ("bg_panel", panel.into()),
        ("bg_card", card.into()),
        ("bg_card_active", soft.clone()),
        ("bg_card_hover", hover.into()),
        ("bg_pill", pill.into()),
        ("bg_pill_active", hover.into()),
        ("border_subtle", mix(card, border, 0.5)),
        ("border_default", border.into()),
        ("border_active", mix(border, secondary, 0.4)),
        ("splitter_bg", bg.into()),
        ("splitter_grip", border.into()),
        ("splitter_grip_hover", secondary.into()),
        ("text_primary", text.into()),
        ("text_secondary", secondary.into()),
        ("text_muted", muted.into()),
        ("text_on_accent", on_accent.into()),
        ("icon_default", secondary.into()),
        ("icon_active", accent.into()),
        ("icon_muted", muted.into()),
        ("accent", accent.into()),
        ("accent_soft", soft),
        ("status_success_bg", mix(card, success, 0.12)),
        ("status_success_fg", success.into()),
        ("status_warning_bg", mix(card, warning, 0.12)),
        ("status_warning_fg", warning.into()),
        (
            "status_danger",
            if dark {
                "#E5A19B".into()
            } else {
                "#AC3939".into()
            },
        ),
    ]
    .into_iter()
    .map(|(key, value)| (key.into(), value))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_palettes_are_unchanged() {
        let definition = builtins()
            .into_iter()
            .find(|theme| theme.id == "classic")
            .unwrap();
        assert_eq!(
            crate::ui::theme::AppTheme::from_definition(&definition, false),
            crate::ui::theme::AppTheme::light()
        );
        assert_eq!(
            crate::ui::theme::AppTheme::from_definition(&definition, true),
            crate::ui::theme::AppTheme::dark()
        );
    }
    #[test]
    fn additional_themes_have_complete_palettes_and_readable_text() {
        fn luminance(color: [u8; 4]) -> f64 {
            let c: Vec<_> = color[..3]
                .iter()
                .map(|value| {
                    let value = *value as f64 / 255.0;
                    if value <= 0.04045 {
                        value / 12.92
                    } else {
                        ((value + 0.055) / 1.055).powf(2.4)
                    }
                })
                .collect();
            c[0] * 0.2126 + c[1] * 0.7152 + c[2] * 0.0722
        }
        fn contrast(a: &str, b: &str) -> f64 {
            let a = luminance(parse_color(a).unwrap());
            let b = luminance(parse_color(b).unwrap());
            (a.max(b) + 0.05) / (a.min(b) + 0.05)
        }
        for definition in builtins().into_iter().filter(|theme| theme.id != "classic") {
            for palette in [&definition.light, &definition.dark] {
                assert_eq!(palette.len(), COLOR_KEYS.len());
                for background in ["bg_panel", "bg_card"] {
                    for foreground in ["text_primary", "text_secondary", "text_muted"] {
                        assert!(
                            contrast(&palette[foreground], &palette[background]) >= 4.5,
                            "{} {foreground}/{background}",
                            definition.id
                        );
                    }
                }
                assert!(
                    contrast(&palette["accent"], &palette["text_on_accent"]) >= 4.5,
                    "{} accent",
                    definition.id
                );
            }
        }
    }
    #[test]
    fn palettes_validate_and_legacy_preferences_remain_distinct_from_system_mode() {
        for theme in builtins() {
            theme.validate().unwrap();
        }
        assert!(ThemeMode::System.is_dark(true, false));
        assert!(!ThemeMode::System.is_dark(false, true));
        assert!(ThemeMode::Legacy.is_dark(false, true));
        let mut invalid = InterfaceTheme::default();
        invalid.dark.insert("accent".into(), "#中文中文".into());
        assert!(invalid.validate().is_err());
    }
}
