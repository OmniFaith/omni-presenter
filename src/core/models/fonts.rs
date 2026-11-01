use eframe::egui::{self, FontData, FontFamily, FontInsert, FontPriority, InsertFontFamily};
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

pub const THEME_FONT: &str = "Theme";

pub struct SystemFonts {
    pub db: fontdb::Database,
    pub families: Vec<String>,
}

static SYSTEM_FONTS: OnceLock<SystemFonts> = OnceLock::new();
fn loaded_fonts(ctx: &egui::Context) -> std::sync::Arc<Mutex<HashSet<String>>> {
    ctx.data_mut(|d| {
        d.get_temp_mut_or_insert_with(egui::Id::unique("loaded_system_fonts"), || {
            std::sync::Arc::new(Mutex::new(HashSet::new()))
        })
        .clone()
    })
}

/// Retorna a lista completa de famílias de fontes instaladas no sistema operacional
pub fn get_system_font_names() -> &'static [String] {
    &get_system_fonts().families
}

pub fn get_system_fonts() -> &'static SystemFonts {
    SYSTEM_FONTS.get_or_init(|| {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();

        let mut families_set = HashSet::new();
        for face in db.faces() {
            for (family, _) in &face.families {
                let trimmed = family.trim();
                // Ignora fontes internas ou vazias
                if !trimmed.is_empty() && !trimmed.starts_with('.') {
                    families_set.insert(trimmed.to_string());
                }
            }
        }

        let mut list: Vec<String> = families_set.into_iter().collect();
        list.sort_by_key(|a| a.to_lowercase());

        SystemFonts { db, families: list }
    })
}

/// Retorna os bytes de uma fonte de emoji colorida instalada no sistema, se houver.
pub fn color_emoji_font_bytes() -> Option<Vec<u8>> {
    let system = get_system_fonts();
    for family in [
        "Noto Color Emoji",
        "Segoe UI Emoji",
        "Apple Color Emoji",
        "Twemoji",
    ] {
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(family)],
            weight: fontdb::Weight::NORMAL,
            stretch: fontdb::Stretch::Normal,
            style: fontdb::Style::Normal,
        };
        if let Some(id) = system.db.query(&query)
            && let Some(bytes) = system.db.with_face_data(id, |data, _| data.to_vec())
        {
            return Some(bytes);
        }
    }
    None
}

/// Carrega a fonte no egui sob demanda se ainda não estiver carregada, respeitando peso numérico e itálico
pub fn ensure_font_loaded_with_style(
    ctx: &egui::Context,
    font_name: &str,
    weight: u16,
    italic: bool,
) -> FontFamily {
    if font_name.is_empty() || font_name == "Proportional" {
        return FontFamily::Proportional;
    }
    if font_name == "Monospace" {
        return FontFamily::Monospace;
    }

    let clamped_weight = weight.clamp(100, 900);
    let family_key = format!("{font_name}##w{clamped_weight}i{italic}");

    let loaded_set = loaded_fonts(ctx);
    if let Ok(set) = loaded_set.lock()
        && set.contains(&family_key) {
            return FontFamily::Name(family_key.into());
        }

    let sys_fonts = get_system_fonts();
    let query_weight = fontdb::Weight(clamped_weight);
    let style = if italic {
        fontdb::Style::Italic
    } else {
        fontdb::Style::Normal
    };

    let query = fontdb::Query {
        families: &[if font_name == "Serif" {
            fontdb::Family::Serif
        } else {
            fontdb::Family::Name(font_name)
        }],
        weight: query_weight,
        stretch: fontdb::Stretch::Normal,
        style,
    };

    if let Some(face_id) = sys_fonts.db.query(&query) {
        let mut font_bytes = None;
        sys_fonts.db.with_face_data(face_id, |data, _| {
            font_bytes = Some(data.to_vec());
        });

        if let Some(bytes) = font_bytes {
            let font_insert = FontInsert::new(
                &family_key,
                FontData::from_owned(bytes),
                vec![InsertFontFamily {
                    family: FontFamily::Name(family_key.clone().into()),
                    priority: FontPriority::Highest,
                }],
            );
            ctx.add_font(font_insert);

            if let Ok(mut set) = loaded_set.lock() {
                set.insert(family_key.clone());
            }

            return FontFamily::Name(family_key.into());
        }
    }

    // Se falhar carregar variante, tenta carregar nome base
    FontFamily::Proportional
}

/// Carrega a fonte no egui sob demanda se ainda não estiver carregada (variante normal)
pub fn ensure_font_loaded_in_egui(ctx: &egui::Context, font_name: &str) {
    if font_name.is_empty()
        || font_name == "Proportional"
        || font_name == "Monospace"
        || font_name == "Serif"
    {
        return;
    }

    let loaded_set = loaded_fonts(ctx);
    if let Ok(set) = loaded_set.lock()
        && set.contains(font_name) {
            return;
        }

    // Busca dados no fontdb
    let sys_fonts = get_system_fonts();
    let query = fontdb::Query {
        families: &[fontdb::Family::Name(font_name)],
        weight: fontdb::Weight::NORMAL,
        stretch: fontdb::Stretch::Normal,
        style: fontdb::Style::Normal,
    };

    if let Some(face_id) = sys_fonts.db.query(&query) {
        let mut font_bytes = None;
        sys_fonts.db.with_face_data(face_id, |data, _| {
            font_bytes = Some(data.to_vec());
        });

        if let Some(bytes) = font_bytes {
            let font_insert = FontInsert::new(
                font_name,
                FontData::from_owned(bytes),
                vec![InsertFontFamily {
                    family: FontFamily::Name(font_name.into()),
                    priority: FontPriority::Highest,
                }],
            );
            ctx.add_font(font_insert);
        }
    }

    if let Ok(mut set) = loaded_set.lock() {
        set.insert(font_name.to_string());
    }
}

pub fn presentation_font_bytes(font_name: &str, weight: u16, italic: bool) -> Vec<u8> {
    let clean = font_name.trim();
    let target = if clean == THEME_FONT {
        "Montserrat"
    } else if clean == "Proportional" {
        "Inter"
    } else {
        clean
    };
    if !target.is_empty() {
        let system = get_system_fonts();
        let query_family = if target == "Serif" {
            fontdb::Family::Serif
        } else if target == "Monospace" {
            fontdb::Family::Monospace
        } else {
            fontdb::Family::Name(target)
        };
        let families = [query_family, fontdb::Family::SansSerif];
        let query = fontdb::Query {
            families: &families,
            weight: fontdb::Weight(weight.clamp(100, 900)),
            stretch: fontdb::Stretch::Normal,
            style: if italic {
                fontdb::Style::Italic
            } else {
                fontdb::Style::Normal
            },
        };
        if let Some(id) = system.db.query(&query)
            && let Some(bytes) = system.db.with_face_data(id, |bytes, _| bytes.to_vec()) {
                return bytes;
            }
    }
    let fonts = egui::FontDefinitions::default();
    let family = if font_name == "Monospace" {
        FontFamily::Monospace
    } else {
        FontFamily::Proportional
    };
    fonts
        .families
        .get(&family)
        .and_then(|list| list.first())
        .and_then(|name| fonts.font_data.get(name))
        .map(|font| font.bytes().to_vec())
        .unwrap_or_default()
}

/// Retorna os pesos numéricos (100..=900) suportados pela família de fontes no sistema.
pub fn available_font_weights(font_name: &str) -> Vec<u16> {
    let clean = font_name.trim();
    if clean.is_empty() {
        return vec![400, 700];
    }
    let sys_fonts = get_system_fonts();
    let mut weights = std::collections::BTreeSet::new();

    let target = if clean == THEME_FONT {
        "Montserrat"
    } else if clean == "Proportional" {
        "Inter"
    } else {
        clean
    };

    for face in sys_fonts.db.faces() {
        let matches = face.families.iter().any(|(fam, _)| {
            fam.eq_ignore_ascii_case(target)
                || fam.split(',').any(|part| part.trim().eq_ignore_ascii_case(target))
        });
        if matches {
            let rounded = ((face.weight.0 as f32 / 100.0).round() as u16 * 100).clamp(100, 900);
            weights.insert(rounded);
        }
    }

    if weights.is_empty() {
        for face in sys_fonts.db.faces() {
            let matches = face.families.iter().any(|(fam, _)| {
                let f_low = fam.to_ascii_lowercase();
                let t_low = target.to_ascii_lowercase();
                f_low.starts_with(&t_low) || t_low.starts_with(&f_low)
            });
            if matches {
                let rounded = ((face.weight.0 as f32 / 100.0).round() as u16 * 100).clamp(100, 900);
                weights.insert(rounded);
            }
        }
    }

    if weights.is_empty() {
        vec![400, 700]
    } else {
        weights.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_fonts_detection() {
        let fonts = get_system_font_names();
        assert!(
            !fonts.is_empty(),
            "O sistema deve ter ao menos algumas fontes instaladas"
        );
    }

    #[test]
    fn test_available_font_weights_filters_to_supported_faces() {
        let dejavu_mono = available_font_weights("DejaVu Sans Mono");
        assert_eq!(dejavu_mono, vec![400, 700]);

        let montserrat = available_font_weights("Montserrat");
        assert!(montserrat.contains(&400));
        assert!(montserrat.contains(&700));

        let fallback = available_font_weights("NonExistentFontFamilyXYZ");
        assert_eq!(fallback, vec![400, 700]);
    }

    #[test]
    fn test_presentation_font_bytes_resolves_theme_font_and_weights() {
        let theme_bytes = presentation_font_bytes(THEME_FONT, 900, false);
        assert!(!theme_bytes.is_empty(), "THEME_FONT must resolve to valid font bytes");

        let prop_bytes = presentation_font_bytes("Proportional", 400, false);
        assert!(!prop_bytes.is_empty(), "Proportional must resolve to valid font bytes");
    }
}
