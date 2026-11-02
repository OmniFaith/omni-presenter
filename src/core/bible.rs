use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, OnceLock};

pub const BOOK_NAMES: [&str; 66] = [
    "Gênesis",
    "Êxodo",
    "Levítico",
    "Números",
    "Deuteronômio",
    "Josué",
    "Juízes",
    "Rute",
    "1 Samuel",
    "2 Samuel",
    "1 Reis",
    "2 Reis",
    "1 Crônicas",
    "2 Crônicas",
    "Esdras",
    "Neemias",
    "Ester",
    "Jó",
    "Salmos",
    "Provérbios",
    "Eclesiastes",
    "Cânticos",
    "Isaías",
    "Jeremias",
    "Lamentações",
    "Ezequiel",
    "Daniel",
    "Oseias",
    "Joel",
    "Amós",
    "Obadias",
    "Jonas",
    "Miqueias",
    "Naum",
    "Habacuque",
    "Sofonias",
    "Ageu",
    "Zacarias",
    "Malaquias",
    "Mateus",
    "Marcos",
    "Lucas",
    "João",
    "Atos",
    "Romanos",
    "1 Coríntios",
    "2 Coríntios",
    "Gálatas",
    "Efésios",
    "Filipenses",
    "Colossenses",
    "1 Tessalonicenses",
    "2 Tessalonicenses",
    "1 Timóteo",
    "2 Timóteo",
    "Tito",
    "Filemom",
    "Hebreus",
    "Tiago",
    "1 Pedro",
    "2 Pedro",
    "1 João",
    "2 João",
    "3 João",
    "Judas",
    "Apocalipse",
];

pub const BOOK_ABBREVS: [&str; 66] = [
    "gn", "ex", "lv", "nm", "dt", "js", "jz", "rt", "1sm", "2sm", "1rs", "2rs", "1cr", "2cr", "ed",
    "ne", "et", "jó", "sl", "pv", "ec", "ct", "is", "jr", "lm", "ez", "dn", "os", "jl", "am", "ob",
    "jn", "mq", "na", "hc", "sf", "ag", "zc", "ml", "mt", "mc", "lc", "jo", "at", "rm", "1co",
    "2co", "gl", "ef", "fp", "cl", "1ts", "2ts", "1tm", "2tm", "tt", "fm", "hb", "tg", "1pe",
    "2pe", "1jo", "2jo", "3jo", "jd", "ap",
];

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
    chapters: Vec<Vec<String>>,
}

fn book_canonical_index(abbrev: &str, name: &str) -> Option<usize> {
    let a = abbrev.trim().to_lowercase();
    for (i, canon) in BOOK_ABBREVS.iter().enumerate() {
        if canon.eq_ignore_ascii_case(&a) {
            return Some(i);
        }
    }
    match a.as_str() {
        "job" => Some(17),
        "ps" => Some(18),
        "prv" => Some(19),
        "so" => Some(21),
        "ho" => Some(27),
        "mi" => Some(32),
        "hk" => Some(34),
        "zp" => Some(35),
        "hg" => Some(36),
        "mk" => Some(40),
        "lk" => Some(41),
        "jhn" => Some(42),
        "act" | "atos" => Some(43),
        "eph" => Some(48),
        "ph" => Some(49),
        "phm" => Some(56),
        "jm" => Some(58),
        "re" => Some(65),
        "1ch" => Some(12),
        "2ch" => Some(13),
        "1kgs" => Some(10),
        "2kgs" => Some(11),
        "jud" => Some(6),
        "ezr" => Some(14),
        _ => {
            let n = fold(name);
            for (i, canon_name) in BOOK_NAMES.iter().enumerate() {
                if fold(canon_name) == n {
                    return Some(i);
                }
            }
            None
        }
    }
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
                        books.push(BibleBook {
                            name: BOOK_NAMES[index].into(),
                            abbrev: BOOK_ABBREVS[index].into(),
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

        let mut books: Vec<BibleBook> = (0..66)
            .map(|i| BibleBook {
                name: BOOK_NAMES[i].into(),
                abbrev: BOOK_ABBREVS[i].into(),
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
                [
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
                ]
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
        crate::core::i18n::Language::English => "EN_KJV",
        crate::core::i18n::Language::Spanish => "ES_RVR1960",
    }
}

fn bundled_name(id: &str) -> &'static str {
    if let Some(meta) = BUNDLED_BIBLES_META.iter().find(|m| m.id.eq_ignore_ascii_case(id)) {
        return meta.name;
    }
    match id {
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
        _ => "Bíblia",
    }
}

pub fn bundled_meta(id: &str) -> Option<&'static BundledBibleMeta> {
    BUNDLED_BIBLES_META.iter().find(|m| m.id.eq_ignore_ascii_case(id))
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
    if stem.starts_with("pt_") {
        stem.trim_start_matches("pt_").to_uppercase()
    } else {
        stem.to_uppercase()
    }
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
        if let Some(found) = self.translations.iter().find(|t| t.id == code) {
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
                    || stem.to_uppercase() == code
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
    version
        .rsplit_once('(')
        .map(|(_, code)| code.trim_end_matches(')'))
        .unwrap_or(version)
        .trim()
        .to_uppercase()
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
            import_code: String::new(),
            import_name: String::new(),
            import_path: None,
            import_error: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_chapters_references_and_ranges_are_real() {
        let bible = BibleLibrary::default()
            .get(DEFAULT_BIBLE)
            .unwrap();
        assert_eq!(bible.books.len(), 66);
        assert_eq!(bible.chapter(16, 10).unwrap().len(), 3);
        let ester = bible.passage(16, 10, 2, 2).unwrap();
        assert!(ester.content.contains("Mardoqueu") || ester.content.contains("Mordecai"));
        assert!(ester.content.ends_with("Ester 10:2"));
        assert_ne!(ester.content, bible.passage(42, 3, 16, 16).unwrap().content);
        assert_eq!(bible.reference("1 João 2:1-3"), Some((61, 2, 1, 3)));
        assert_eq!(bible.reference("Jo3.16"), Some((42, 3, 16, 16)));
        assert!(bible.reference("Livro inexistente 1:1").is_none());
    }

    #[test]
    fn imports_reject_invalid_content_and_accept_nested_arrays() {
        assert!(BibleTranslation::from_json(br#"{"books":[]}"#, "NAA", "Nova Almeida").is_err());
        let books: Vec<_> = BOOK_NAMES.iter().map(|name| serde_json::json!({"name":name,"chapters":[["Um versículo", "Outro versículo"]]})).collect();
        let translation = BibleTranslation::from_json(
            &serde_json::to_vec(&books).unwrap(),
            "TESTE",
            "Tradução teste",
        )
        .unwrap();
        assert_eq!(
            translation.passage(16, 1, 1, 2).unwrap().content,
            "Um versículo\nOutro versículo\nEster 1:1–2"
        );
        let mut broken = serde_json::to_value(&books).unwrap();
        broken[0]["chapters"][0][0] = serde_json::json!(42);
        assert!(
            BibleTranslation::from_json(&serde_json::to_vec(&broken).unwrap(), "TESTE", "Teste")
                .is_err()
        );

        // Test UTF-8 BOM stripping
        let mut bom_bytes = vec![0xef, 0xbb, 0xbf];
        bom_bytes.extend(serde_json::to_vec(&books).unwrap());
        let bom_res = BibleTranslation::from_json(&bom_bytes, "PT_AA", "Almeida Atualizada");
        assert!(bom_res.is_ok());
        assert_eq!(bom_res.unwrap().id, "PT_AA");
    }

    #[test]
    fn bundled_bibles_load_cleanly_with_metadata() {
        let lib = BibleLibrary::default();
        let catalog = lib.catalog();
        assert!(!catalog.is_empty());
        let ara = lib.get("ARA");
        assert!(ara.is_some(), "Default bible ARA must load successfully");
        let kjv = lib.get("EN_KJV");
        assert!(kjv.is_some(), "English KJV bible must load successfully");
        for (id, name) in &catalog {
            assert!(!id.is_empty());
            assert!(!name.is_empty());
            assert_ne!(name, "Bíblia em português", "Bibles should have real translation names: {}", id);
        }
    }
}

