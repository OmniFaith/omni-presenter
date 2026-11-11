use crate::core::i18n::Language;

pub const BOOK_NAMES_PT: [&str; 66] = [
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

pub const BOOK_NAMES_EN: [&str; 66] = [
    "Genesis",
    "Exodus",
    "Leviticus",
    "Numbers",
    "Deuteronomy",
    "Joshua",
    "Judges",
    "Ruth",
    "1 Samuel",
    "2 Samuel",
    "1 Kings",
    "2 Kings",
    "1 Chronicles",
    "2 Chronicles",
    "Ezra",
    "Nehemiah",
    "Esther",
    "Job",
    "Psalms",
    "Proverbs",
    "Ecclesiastes",
    "Song of Solomon",
    "Isaiah",
    "Jeremiah",
    "Lamentations",
    "Ezekiel",
    "Daniel",
    "Hosea",
    "Joel",
    "Amos",
    "Obadiah",
    "Jonah",
    "Micah",
    "Nahum",
    "Habakkuk",
    "Zephaniah",
    "Haggai",
    "Zechariah",
    "Malachi",
    "Matthew",
    "Mark",
    "Luke",
    "John",
    "Acts",
    "Romans",
    "1 Corinthians",
    "2 Corinthians",
    "Galatians",
    "Ephesians",
    "Philippians",
    "Colossians",
    "1 Thessalonians",
    "2 Thessalonians",
    "1 Timothy",
    "2 Timothy",
    "Titus",
    "Philemon",
    "Hebrews",
    "James",
    "1 Peter",
    "2 Peter",
    "1 John",
    "2 John",
    "3 John",
    "Jude",
    "Revelation",
];

pub const BOOK_NAMES_ES: [&str; 66] = [
    "Génesis",
    "Éxodo",
    "Levítico",
    "Números",
    "Deuteronomio",
    "Josué",
    "Jueces",
    "Rut",
    "1 Samuel",
    "2 Samuel",
    "1 Reyes",
    "2 Reyes",
    "1 Crónicas",
    "2 Crónicas",
    "Esdras",
    "Nehemías",
    "Ester",
    "Job",
    "Salmos",
    "Proverbios",
    "Eclesiastés",
    "Cantares",
    "Isaías",
    "Jeremías",
    "Lamentaciones",
    "Ezequiel",
    "Daniel",
    "Oseas",
    "Joel",
    "Amós",
    "Abdías",
    "Jonás",
    "Miqueas",
    "Nahúm",
    "Habacuc",
    "Sofonías",
    "Hageo",
    "Zacarías",
    "Malaquías",
    "Mateo",
    "Marcos",
    "Lucas",
    "Juan",
    "Hechos",
    "Romanos",
    "1 Corintios",
    "2 Corintios",
    "Gálatas",
    "Efesios",
    "Filipenses",
    "Colosenses",
    "1 Tesalonicenses",
    "2 Tesalonicenses",
    "1 Timoteo",
    "2 Timoteo",
    "Tito",
    "Filemón",
    "Hebreos",
    "Santiago",
    "1 Pedro",
    "2 Pedro",
    "1 Juan",
    "2 Juan",
    "3 Juan",
    "Judas",
    "Apocalipsis",
];

pub const BOOK_NAMES: [&str; 66] = BOOK_NAMES_PT;

pub const BOOK_ABBREVS_PT: [&str; 66] = [
    "gn", "ex", "lv", "nm", "dt", "js", "jz", "rt", "1sm", "2sm", "1rs", "2rs", "1cr", "2cr", "ed",
    "ne", "et", "jó", "sl", "pv", "ec", "ct", "is", "jr", "lm", "ez", "dn", "os", "jl", "am", "ob",
    "jn", "mq", "na", "hc", "sf", "ag", "zc", "ml", "mt", "mc", "lc", "jo", "at", "rm", "1co",
    "2co", "gl", "ef", "fp", "cl", "1ts", "2ts", "1tm", "2tm", "tt", "fm", "hb", "tg", "1pe",
    "2pe", "1jo", "2jo", "3jo", "jd", "ap",
];

pub const BOOK_ABBREVS_EN: [&str; 66] = [
    "gen", "exo", "lev", "num", "deu", "jos", "jdg", "rut", "1sa", "2sa", "1ki", "2ki", "1ch", "2ch", "ezr",
    "neh", "est", "job", "psa", "pro", "ecc", "sng", "isa", "jer", "lam", "ezk", "dan", "hos", "jol", "amo", "oba",
    "jon", "mic", "nam", "hab", "zep", "hag", "zec", "mal", "mat", "mrk", "luk", "jhn", "act", "rom", "1co",
    "2co", "gal", "eph", "php", "col", "1th", "2th", "1ti", "2ti", "tit", "phm", "heb", "jas", "1pe",
    "2pe", "1jn", "2jn", "3jn", "jud", "rev",
];

pub const BOOK_ABBREVS_ES: [&str; 66] = [
    "gn", "ex", "lv", "nm", "dt", "jos", "jue", "rut", "1s", "2s", "1r", "2r", "1cr", "2cr", "esd",
    "neh", "est", "job", "sal", "pr", "ec", "cnt", "is", "jer", "lam", "ez", "dn", "os", "jl", "am", "ob",
    "jon", "miq", "nah", "hab", "sof", "hag", "zac", "mal", "mt", "mr", "lc", "jn", "hch", "ro", "1co",
    "2co", "ga", "ef", "fil", "col", "1ts", "2ts", "1ti", "2ti", "tit", "flm", "heb", "stg", "1p",
    "2p", "1jn", "2jn", "3jn", "jud", "ap",
];

pub const BOOK_ABBREVS: [&str; 66] = BOOK_ABBREVS_PT;

pub fn book_names_for_language(language: Language) -> &'static [&'static str; 66] {
    match language {
        Language::Portuguese => &BOOK_NAMES_PT,
        Language::English => &BOOK_NAMES_EN,
        Language::Spanish => &BOOK_NAMES_ES,
    }
}

pub fn book_abbrevs_for_language(language: Language) -> &'static [&'static str; 66] {
    match language {
        Language::Portuguese => &BOOK_ABBREVS_PT,
        Language::English => &BOOK_ABBREVS_EN,
        Language::Spanish => &BOOK_ABBREVS_ES,
    }
}

pub fn book_canonical_index(abbrev: &str, name: &str) -> Option<usize> {
    let a = abbrev.trim().to_lowercase();
    for (i, canon) in BOOK_ABBREVS_PT.iter().enumerate() {
        if canon.eq_ignore_ascii_case(&a) {
            return Some(i);
        }
    }
    for (i, canon) in BOOK_ABBREVS_EN.iter().enumerate() {
        if canon.eq_ignore_ascii_case(&a) {
            return Some(i);
        }
    }
    for (i, canon) in BOOK_ABBREVS_ES.iter().enumerate() {
        if canon.eq_ignore_ascii_case(&a) {
            return Some(i);
        }
    }
    match a.as_str() {
        "job" => Some(17),
        "ps" | "psa" | "psalm" | "psalms" | "sal" => Some(18),
        "prv" | "pro" => Some(19),
        "so" | "sos" | "song" | "cantares" => Some(21),
        "ho" | "hos" => Some(27),
        "mi" | "mic" => Some(32),
        "hk" | "hab" => Some(34),
        "zp" | "zep" => Some(35),
        "hg" | "hag" => Some(36),
        "mk" | "mrk" => Some(40),
        "lk" | "luk" => Some(41),
        "jhn" | "jn" | "john" | "juan" | "jo" => Some(42),
        "act" | "acts" | "atos" | "hch" | "hechos" => Some(43),
        "eph" => Some(48),
        "ph" | "php" => Some(49),
        "phm" => Some(56),
        "jm" | "jas" => Some(58),
        "re" | "rev" => Some(65),
        "1ch" => Some(12),
        "2ch" => Some(13),
        "1kgs" | "1ki" => Some(10),
        "2kgs" | "2ki" => Some(11),
        "jud" => Some(6),
        "ezr" => Some(14),
        _ => {
            let n = crate::core::bible::fold(name);
            for (i, canon_name) in BOOK_NAMES_PT.iter().enumerate() {
                if crate::core::bible::fold(canon_name) == n {
                    return Some(i);
                }
            }
            for (i, canon_name) in BOOK_NAMES_EN.iter().enumerate() {
                if crate::core::bible::fold(canon_name) == n {
                    return Some(i);
                }
            }
            for (i, canon_name) in BOOK_NAMES_ES.iter().enumerate() {
                if crate::core::bible::fold(canon_name) == n {
                    return Some(i);
                }
            }
            None
        }
    }
}

pub fn detect_bible_language(id: &str, sample_text: &str) -> Language {
    let lower_id = id.to_lowercase();
    if lower_id.starts_with("en_")
        || lower_id.starts_with("en-")
        || lower_id.ends_with("_en")
        || lower_id == "kjv"
        || lower_id == "esv"
        || lower_id == "niv"
        || lower_id == "nkjv"
        || lower_id == "asv"
    {
        return Language::English;
    }
    if lower_id.starts_with("es_")
        || lower_id.starts_with("es-")
        || lower_id.ends_with("_es")
        || lower_id == "rvr"
        || lower_id == "rvr1960"
        || lower_id == "nvi_es"
        || lower_id == "dhh"
    {
        return Language::Spanish;
    }
    if !sample_text.is_empty() {
        let lower = sample_text.to_lowercase();
        if lower.contains(" the ")
            || lower.contains(" and ")
            || lower.contains(" god ")
            || lower.contains(" lord ")
        {
            return Language::English;
        }
        if lower.contains(" y ") || lower.contains(" dios ") || lower.contains(" principio ") {
            return Language::Spanish;
        }
    }
    Language::Portuguese
}


