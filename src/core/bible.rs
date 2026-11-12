use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, OnceLock};

pub use crate::core::bible_books::*;
pub use crate::core::bible_split::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BibleBook {
    pub name: String,
    #[serde(default)]
    pub abbrev: String,
    pub chapters: Vec<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BibleTranslation {
    pub id: String,
    pub name: String,
    pub books: Vec<BibleBook>,
}

#[derive(Deserialize)]
struct FastBook {
    #[serde(default)]
    name: String,
    #[serde(default)]
    abbrev: String,
    #[serde(default)]
    chapters: Vec<Vec<String>>,
}

impl BibleTranslation {
    /// Accept the Omni Presenter, nested-array and Scrollmapper JSON formats. Preserve missing
    /// verses as empty entries so subsequent references are never silently renumbered.
    pub fn from_json(bytes: &[u8], code: &str, name: &str) -> Result<Self, String> {
        let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes);
        let code_clean = code.trim().to_uppercase();
        if !code_clean.is_empty()
            && code_clean.chars().count() <= 32
            && code_clean.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            && !name.trim().is_empty()
            && let Ok(fast_books) = serde_json::from_slice::<Vec<FastBook>>(bytes)
                && fast_books.len() == 66 {
                    let lang = detect_bible_language(&code_clean, name);
                    let canon_names = book_names_for_language(lang);
                    let canon_abbrevs = book_abbrevs_for_language(lang);
                    let mut valid = true;
                    let mut books = Vec::with_capacity(66);
                    for (index, fb) in fast_books.into_iter().enumerate() {
                        if fb.chapters.is_empty() || fb.chapters.len() > 150 {
                            valid = false;
                            break;
                        }
                        for chap in &fb.chapters {
                            if chap.is_empty() || chap.len() > 250 {
                                valid = false;
                                break;
                            }
                        }
                        if !valid {
                            break;
                        }
                        let book_name = if !fb.name.trim().is_empty() {
                            fb.name.trim().to_string()
                        } else {
                            canon_names[index].into()
                        };
                        let book_abbrev = if !fb.abbrev.trim().is_empty() {
                            fb.abbrev.trim().to_lowercase()
                        } else {
                            canon_abbrevs[index].into()
                        };
                        books.push(BibleBook {
                            name: book_name,
                            abbrev: book_abbrev,
                            chapters: fb.chapters,
                        });
                    }
                    if valid {
                        return Ok(Self {
                            id: code_clean,
                            name: name.trim().to_owned(),
                            books,
                        });
                    }
                }

        let data: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|e| format!("JSON inválido: {e}"))?;
        let source = data
            .get("books")
            .unwrap_or(&data)
            .as_array()
            .ok_or("O arquivo deve conter uma lista de livros.")?;
        if source.is_empty() {
            return Err("A tradução não contém livros.".into());
        }
        let id = data
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or(code)
            .trim()
            .to_uppercase();
        if id.is_empty()
            || id.chars().count() > 32
            || !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err("Informe uma sigla válida (até 32 letras, números, traços ou sublinhados).".into());
        }
        let name = data
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(name)
            .trim()
            .to_owned();
        if name.is_empty() {
            return Err("Informe o nome da tradução.".into());
        }

        let lang = detect_bible_language(&id, &name);
        let canon_names = book_names_for_language(lang);
        let canon_abbrevs = book_abbrevs_for_language(lang);

        let mut books: Vec<BibleBook> = (0..66)
            .map(|i| BibleBook {
                name: canon_names[i].into(),
                abbrev: canon_abbrevs[i].into(),
                chapters: Vec::new(),
            })
            .collect();

        for (index, book) in source.iter().enumerate() {
            let canon_idx = if source.len() == 66 {
                index
            } else {
                let abbrev = book.get("abbrev").and_then(|v| v.as_str()).unwrap_or("");
                let book_name = book.get("name").and_then(|v| v.as_str()).unwrap_or("");
                book_canonical_index(abbrev, book_name).unwrap_or(index.min(65))
            };

            let chapters = book
                .get("chapters")
                .and_then(|v| v.as_array())
                .ok_or_else(|| format!("Faltam os capítulos de {}.", BOOK_NAMES[canon_idx]))?;
            if chapters.is_empty() || chapters.len() > 150 {
                return Err(format!(
                    "Quantidade de capítulos inválida em {}.",
                    BOOK_NAMES[canon_idx]
                ));
            }
            let mut parsed_chapters = Vec::new();
            for (chapter_index, chapter) in chapters.iter().enumerate() {
                if let Some(number) = chapter.get("chapter").and_then(|v| v.as_u64())
                    && number as usize != chapter_index + 1 {
                        return Err("Os capítulos precisam estar em ordem numérica.".into());
                    }
                let verses = chapter
                    .get("verses")
                    .unwrap_or(chapter)
                    .as_array()
                    .ok_or("Capítulo sem lista de versículos.")?;
                let mut parsed = Vec::new();
                for verse in verses {
                    let number = verse
                        .get("verse")
                        .and_then(|v| v.as_u64())
                        .map(|n| n as usize)
                        .unwrap_or(parsed.len() + 1);
                    if number <= parsed.len() || number > 250 {
                        return Err("Numeração de versículos inválida ou duplicada.".into());
                    }
                    let text = verse
                        .get("text")
                        .unwrap_or(verse)
                        .as_str()
                        .ok_or("O texto de cada versículo deve ser uma string.")?;
                    parsed.resize(number - 1, String::new());
                    parsed.push(text.split_whitespace().collect::<Vec<_>>().join(" "));
                }
                if parsed.is_empty() {
                    return Err("Há um capítulo sem versículos no arquivo.".into());
                }
                parsed_chapters.push(parsed);
            }
            if let Some(n) = book.get("name").and_then(|v| v.as_str()).filter(|s| !s.trim().is_empty()) {
                books[canon_idx].name = n.trim().to_string();
            }
            if let Some(a) = book.get("abbrev").and_then(|v| v.as_str()).filter(|s| !s.trim().is_empty()) {
                books[canon_idx].abbrev = a.trim().to_lowercase();
            }
            books[canon_idx].chapters = parsed_chapters;
        }
        Ok(Self { id, name, books })
    }

    pub fn chapter(&self, book: usize, chapter: usize) -> Option<&[String]> {
        self.books
            .get(book)?
            .chapters
            .get(chapter.checked_sub(1)?)
            .map(Vec::as_slice)
    }

    pub fn passage(
        &self,
        book: usize,
        chapter: usize,
        start: usize,
        end: usize,
    ) -> Option<BiblePassage> {
        if start == 0 || end < start {
            return None;
        }
        let verses = self.chapter(book, chapter)?.get(start - 1..end)?;
        if verses.iter().any(|v| v.is_empty()) {
            return None;
        }
        let reference = format!(
            "{} {}:{}{}",
            self.books[book].name,
            chapter,
            start,
            if start == end {
                String::new()
            } else {
                format!("–{end}")
            }
        );
        Some(BiblePassage {
            reference: reference.clone(),
            version: self.id.clone(),
            content: format!("{}\n{}", verses.join("\n"), reference),
        })
    }

    pub fn reference(&self, query: &str) -> Option<(usize, usize, usize, usize)> {
        let original = query.to_lowercase().replace(' ', "");
        let query = fold(query).replace(' ', "");
        let mut candidates: Vec<_> = self
            .books
            .iter()
            .enumerate()
            .flat_map(|(index, b)| {
                let mut v = vec![
                    (
                        index,
                        fold(&b.name).replace(' ', ""),
                        original.starts_with(&b.name.to_lowercase().replace(' ', "")),
                    ),
                    (
                        index,
                        fold(&b.abbrev).replace(' ', ""),
                        original.starts_with(&b.abbrev.to_lowercase().replace(' ', "")),
                    ),
                ];
                if index < 66 {
                    for name in [BOOK_NAMES_PT[index], BOOK_NAMES_EN[index], BOOK_NAMES_ES[index]] {
                        v.push((
                            index,
                            fold(name).replace(' ', ""),
                            original.starts_with(&name.to_lowercase().replace(' ', "")),
                        ));
                    }
                    for abbrev in [BOOK_ABBREVS_PT[index], BOOK_ABBREVS_EN[index], BOOK_ABBREVS_ES[index]] {
                        v.push((
                            index,
                            fold(abbrev).replace(' ', ""),
                            original.starts_with(&abbrev.to_lowercase().replace(' ', "")),
                        ));
                    }
                }
                v
            })
            .collect();
        candidates.sort_by_key(|(_, name, exact)| std::cmp::Reverse((name.len(), *exact)));
        for (book, name, _) in candidates {
            let Some(numbers) = query.strip_prefix(&name) else {
                continue;
            };
            let mut parts = numbers.split([':', '.']);
            let Ok(chapter) = parts.next().unwrap_or("").parse::<usize>() else {
                continue;
            };
            let range = parts.next().unwrap_or("1");
            if parts.next().is_some() {
                continue;
            }
            let mut range = range.split(['-', '–']);
            let Ok(start) = range.next().unwrap_or("").parse::<usize>() else {
                continue;
            };
            let Ok(end) = range.next().unwrap_or(&start.to_string()).parse::<usize>() else {
                continue;
            };
            if range.next().is_none() && self.passage(book, chapter, start, end).is_some() {
                return Some((book, chapter, start, end));
            }
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct BiblePassage {
    pub reference: String,
    pub version: String,
    pub content: String,
}

#[derive(Clone)]
pub struct BibleLibrary {
    pub translations: Vec<Arc<BibleTranslation>>,
}

include!(concat!(env!("OUT_DIR"), "/bible_assets.rs"));

pub const DEFAULT_BIBLE: &str = "ARA";

pub fn default_for_language(language: crate::core::i18n::Language) -> &'static str {
    match language {
        crate::core::i18n::Language::Portuguese => "ARA",
        crate::core::i18n::Language::English => "KJV",
        crate::core::i18n::Language::Spanish => "RVR1960",
    }
}

pub fn clean_version_id(stem_or_id: &str) -> String {
    let trimmed = stem_or_id.trim();
    if let Some((lang, rest)) = trimmed.split_once('_') {
        if (2..=4).contains(&lang.len())
            && lang.chars().all(|c| c.is_ascii_alphabetic())
            && !rest.is_empty()
        {
            return rest.to_uppercase();
        }
    }
    trimmed.to_uppercase()
}

fn bundled_name(id: &str) -> &'static str {
    let clean = clean_version_id(id);
    if let Some(meta) = BUNDLED_BIBLES_META.iter().find(|m| {
        m.id.eq_ignore_ascii_case(id)
            || m.id.eq_ignore_ascii_case(&clean)
            || m.stem.eq_ignore_ascii_case(id)
    }) {
        return meta.name;
    }
    match clean.as_str() {
        "AA" => "Almeida Atualizada",
        "ACF" => "Almeida Corrigida Fiel",
        "ARA" => "Almeida Revista e Atualizada",
        "ARC" => "Almeida Revista e Corrigida",
        "KJA" => "King James Atualizada",
        "NAA" => "Nova Almeida Atualizada",
        "NBV" => "Nova Bíblia Viva",
        "NTLH" => "Nova Tradução na Linguagem de Hoje",
        "NVI" => "Nova Versão Internacional",
        "NVI2011" => "Nova Versão Internacional (2011)",
        "NVT" => "Nova Versão Transformadora",
        "TB" => "Tradução Brasileira",
        "VFL" => "Versão Fácil de Ler",
        "KJV" => "King James Version",
        "RVR1960" => "Reina-Valera 1960",
        _ => "Bíblia",
    }
}

pub fn bundled_meta(id: &str) -> Option<&'static BundledBibleMeta> {
    let clean = clean_version_id(id);
    BUNDLED_BIBLES_META.iter().find(|m| {
        m.id.eq_ignore_ascii_case(id)
            || m.id.eq_ignore_ascii_case(&clean)
            || m.stem.eq_ignore_ascii_case(id)
    })
}

pub fn bible_language(id: &str) -> &'static str {
    bundled_meta(id).map(|m| m.language).unwrap_or("Outro")
}

pub fn is_pt_bible(id: &str) -> bool {
    bundled_meta(id).map(|m| m.is_pt).unwrap_or_else(|| {
        id.starts_with("PT_") || id == "ARA" || id == "NVI" || id == "ACF" || id == "ARC"
    })
}

fn bundled_id(stem: &str) -> String {
    clean_version_id(stem)
}

impl Default for BibleLibrary {
    fn default() -> Self {
        Self::prewarm_bundled();
        Self {
            translations: Vec::new(),
        }
    }
}

impl BibleLibrary {
    pub fn prewarm_bundled() {
        static ONCE: OnceLock<()> = OnceLock::new();
        ONCE.get_or_init(|| {
            std::thread::Builder::new()
                .name("bible_prewarm".into())
                .spawn(|| {
                    let lib = BibleLibrary { translations: Vec::new() };
                    let _ = lib.get(DEFAULT_BIBLE);
                })
                .ok();
        });
    }

    pub fn get(&self, id: &str) -> Option<Arc<BibleTranslation>> {
        let code = version_code(id);
        if let Some(found) = self.translations.iter().find(|t| {
            t.id.eq_ignore_ascii_case(&code) || t.id.eq_ignore_ascii_case(id)
        }) {
            return Some(found.clone());
        }
        static CACHE: OnceLock<Mutex<std::collections::HashMap<String, Arc<BibleTranslation>>>> =
            OnceLock::new();
        let cache = CACHE.get_or_init(Mutex::default);
        if let Some(found) = cache.lock().unwrap().get(&code).cloned() {
            return Some(found);
        }
        let (stem, bytes) = BUNDLED_BIBLES
            .iter()
            .find(|(stem, _)| {
                bundled_id(stem) == code
                    || bundled_id(stem).eq_ignore_ascii_case(id)
                    || stem.eq_ignore_ascii_case(&code)
                    || stem.eq_ignore_ascii_case(id)
                    || stem.strip_prefix("pt_").unwrap_or(stem).to_uppercase() == code
            })?;
        let name = bundled_name(&code);
        let translation = Arc::new(
            BibleTranslation::from_json(bytes, &code, name)
                .map_err(|error| eprintln!("Bíblia {stem} indisponível: {error}"))
                .ok()?,
        );
        cache
            .lock()
            .unwrap()
            .insert(code, translation.clone());
        Some(translation)
    }

    pub fn catalog(&self) -> Vec<(String, String)> {
        let mut versions: Vec<(String, String)> = BUNDLED_BIBLES_META
            .iter()
            .map(|meta| (meta.id.to_string(), meta.name.to_string()))
            .collect();
        for translation in &self.translations {
            versions.retain(|(id, _)| id != &translation.id);
            versions.push((translation.id.clone(), translation.name.clone()));
        }
        versions
    }


    pub fn default_version(&self) -> String {
        if self.get(DEFAULT_BIBLE).is_some() {
            DEFAULT_BIBLE.to_owned()
        } else {
            self.catalog()
                .first()
                .map(|(id, _)| id.clone())
                .unwrap_or_default()
        }
    }

    pub fn default_version_for_language(&self, language: crate::core::i18n::Language) -> String {
        let preferred = default_for_language(language);
        if self.get(preferred).is_some() {
            preferred.to_owned()
        } else {
            self.default_version()
        }
    }

    pub fn insert(&mut self, translation: BibleTranslation) {
        self.translations.retain(|t| t.id != translation.id);
        self.translations.push(Arc::new(translation));
    }
}

pub fn version_code(version: &str) -> String {
    let raw = version
        .rsplit_once('(')
        .map(|(_, code)| code.trim_end_matches(')'))
        .unwrap_or(version)
        .trim();
    clean_version_id(raw)
}

pub fn fold(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ã' | 'â' | 'ä' => 'a',
            'é' | 'ê' | 'è' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ô' | 'õ' | 'ò' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            _ => c,
        })
        .collect()
}

#[derive(Clone, Copy, Debug)]
pub struct SearchHit {
    pub book: usize,
    pub chapter: usize,
    pub verse: usize,
}

#[derive(Clone)]
pub struct BibleState {
    pub library: BibleLibrary,
    pub expanded_book: Option<usize>,
    pub verse_end: usize,
    pub selection_anchor: usize,
    pub merge_selection: bool,
    pub cards_layout: bool,
    pub show_search: bool,
    pub search: String,
    pub testament: u8,
    pub search_key: String,
    pub search_results: Vec<SearchHit>,
    pub preview_key: String,
    pub suggestion_index: usize,
    pub search_focus: bool,
    pub scroll_to_book: bool,
    pub scroll_to_verse: bool,
    pub show_import: bool,
    pub show_settings: bool,
    pub import_code: String,
    pub import_name: String,
    pub import_path: Option<std::path::PathBuf>,
    pub import_error: String,
}

impl Default for BibleState {
    fn default() -> Self {
        Self {
            library: BibleLibrary::default(),
            expanded_book: Some(0),
            verse_end: 1,
            selection_anchor: 1,
            merge_selection: false,
            cards_layout: false,
            show_search: false,
            search: String::new(),
            testament: 0,
            search_key: String::new(),
            search_results: Vec::new(),
            preview_key: String::new(),
            suggestion_index: 0,
            search_focus: false,
            scroll_to_book: true,
            scroll_to_verse: true,
            show_import: false,
            show_settings: false,
            import_code: String::new(),
            import_name: String::new(),
            import_path: None,
            import_error: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BibleReferencePosition {
    Top,
    Bottom,
    BelowVerse,
    EndOfVerse,
    Hidden,
}

impl Default for BibleReferencePosition {
    fn default() -> Self {
        Self::Bottom
    }
}

impl BibleReferencePosition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::BelowVerse => "below",
            Self::EndOfVerse => "end",
            Self::Hidden => "hidden",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BibleReferenceAlignment {
    Left,
    Center,
    Right,
}

impl Default for BibleReferenceAlignment {
    fn default() -> Self {
        Self::Right
    }
}

impl BibleReferenceAlignment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BibleSeparator {
    Colon,
    Dot,
}

impl Default for BibleSeparator {
    fn default() -> Self {
        Self::Colon
    }
}

impl BibleSeparator {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Colon => ":",
            Self::Dot => ".",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BibleVerseAlignment {
    Center,
    Left,
    Justified,
}

impl Default for BibleVerseAlignment {
    fn default() -> Self {
        Self::Center
    }
}

impl BibleVerseAlignment {
    pub fn as_markup_directive(&self) -> Option<&'static str> {
        match self {
            Self::Center => Some("//alinhar: centro\n"),
            Self::Left => Some("//alinhar: esquerda\n"),
            Self::Justified => Some("//alinhar: justificado\n"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BibleVerticalAlignment {
    #[default]
    Center,
    Top,
    Bottom,
}

impl BibleVerticalAlignment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Center => "center",
            Self::Bottom => "bottom",
        }
    }

    pub fn as_markup_directive(&self) -> Option<&'static str> {
        match self {
            Self::Top => Some("//valign: top\n"),
            Self::Center => Some("//valign: center\n"),
            Self::Bottom => Some("//valign: bottom\n"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BibleFontScale {
    Compact, // 85%
    #[default]
    Normal,  // 100%
    Large,   // 115%
    ExtraLarge, // 130%
}

impl BibleFontScale {
    pub fn multiplier(&self) -> f32 {
        match self {
            Self::Compact => 0.85,
            Self::Normal => 1.0,
            Self::Large => 1.15,
            Self::ExtraLarge => 1.30,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BibleReferenceHighlight {
    Discreet,
    #[default]
    Accent,
    Badge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BibleVerseNumberStyle {
    #[default]
    Bracket,     // [16]
    Superscript, // ¹⁶
    Dot,         // 16.
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BibleVersionStyle {
    #[default]
    Parentheses, // (NVI)
    Hyphen,      // - NVI
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BibleLineSpacing {
    Compact,
    #[default]
    Normal,
    Spacious,
}

impl BibleLineSpacing {
    pub fn as_markup_directive(&self) -> Option<&'static str> {
        match self {
            Self::Compact => Some("//linha: 1.15\n"),
            Self::Normal => None,
            Self::Spacious => Some("//linha: 1.6\n"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BibleReferenceSize {
    #[default]
    Normal,
    Compact,
}

impl BibleReferenceSize {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Compact => "compact",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct BibleSettings {
    pub reference_position: BibleReferencePosition,
    #[serde(default)]
    pub reference_alignment: BibleReferenceAlignment,
    #[serde(default)]
    pub reference_size: BibleReferenceSize,
    pub show_version: bool,
    pub version_style: BibleVersionStyle,
    pub use_abbreviation: bool,
    pub uppercase_reference: bool,
    pub italic_reference: bool,
    pub show_verse_numbers: bool,
    pub verse_number_style: BibleVerseNumberStyle,
    pub separator: BibleSeparator,
    pub alignment: BibleVerseAlignment,
    pub vertical_alignment: BibleVerticalAlignment,
    pub font_scale: BibleFontScale,
    pub reference_highlight: BibleReferenceHighlight,
    pub line_spacing: BibleLineSpacing,
    pub quotes_around_verse: bool,
    pub auto_split_long_verses: bool,
    #[serde(default = "crate::core::bible_split::default_max_chars")]
    pub max_chars_per_slide: usize,
    #[serde(default)]
    pub split_action: BibleSplitAction,
    #[serde(default)]
    pub split_punctuation: BibleSplitPunctuation,
    #[serde(default = "crate::core::bible_split::default_true")]
    pub split_ellipsis: bool,
    pub one_verse_per_slide: bool,
    #[serde(default)]
    pub range_number_only: bool,
}

impl Default for BibleSettings {
    fn default() -> Self {
        Self {
            reference_position: BibleReferencePosition::Bottom,
            reference_alignment: BibleReferenceAlignment::Right,
            reference_size: BibleReferenceSize::Normal,
            show_version: false,
            version_style: BibleVersionStyle::Parentheses,
            use_abbreviation: false,
            uppercase_reference: false,
            italic_reference: false,
            show_verse_numbers: true,
            verse_number_style: BibleVerseNumberStyle::Bracket,
            separator: BibleSeparator::Colon,
            alignment: BibleVerseAlignment::Center,
            vertical_alignment: BibleVerticalAlignment::Center,
            font_scale: BibleFontScale::Normal,
            reference_highlight: BibleReferenceHighlight::Accent,
            line_spacing: BibleLineSpacing::Normal,
            quotes_around_verse: false,
            auto_split_long_verses: false,
            max_chars_per_slide: 120,
            split_action: BibleSplitAction::MultipleSlides,
            split_punctuation: BibleSplitPunctuation::NaturalPunctuation,
            split_ellipsis: true,
            one_verse_per_slide: false,
            range_number_only: false,
        }
    }
}

pub use super::bible_format::{
    format_bible_slide_with_numbered_ref,
    format_bible_reference, format_bible_slide, format_bible_slide_with_custom_ref,
    format_verse_number, to_superscript,
};

#[cfg(test)]
#[path = "bible_tests.rs"]
mod tests;
