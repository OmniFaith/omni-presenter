use crate::core::i18n::Language;

pub fn preferred_header(source: &str) -> Option<Language> {
    source.split(',').enumerate().filter_map(|(index, entry)| {
        let mut parts = entry.trim().split(';');
        let code = parts.next()?.trim();
        if !matches!(code.split(['-', '_']).next()?.to_ascii_lowercase().as_str(), "en" | "pt" | "es") {
            return None;
        }
        let mut quality = 1.0;
        for part in parts {
            if let Some(value) = part.trim().strip_prefix("q=") {
                quality = value.parse::<f32>().ok()?;
                break;
            }
        }
        (quality > 0.0 && quality <= 1.0).then_some((quality, std::cmp::Reverse(index), Language::from_code(code)))
    }).max_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1))).map(|(_, _, language)| language)
}

pub fn language(request: &tiny_http::Request, fallback: Option<Language>) -> Language {
    if let Some(code) = request.url().split_once('?').and_then(|(_, query)| query.split('&').find_map(|part| part.strip_prefix("lang="))) {
        return Language::from_code(code);
    }
    if let Some(header) = request.headers().iter().find(|header| {
        header.field.equiv("X-Omni-Presenter-Language")
            || header.field.equiv("X-Omni-Language")
    }) {
        return Language::from_code(header.value.as_str());
    }
    fallback.or_else(|| request.headers().iter().find(|header| header.field.equiv("Accept-Language"))
        .and_then(|header| preferred_header(header.value.as_str())))
        .unwrap_or(Language::English)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_browser_languages_respect_quality_and_original_order() {
        assert_eq!(preferred_header("en-US,en;q=0.8,es;q=0.9"), Some(Language::English));
        assert_eq!(preferred_header("en;q=0.4,es-MX;q=0.9,pt-BR;q=0.8"), Some(Language::Spanish));
        assert_eq!(preferred_header("fr,pt-PT;q=0.5,en;q=0.5"), Some(Language::Portuguese));
        assert_eq!(preferred_header("es;q=0,en;q=0"), None);
        assert_eq!(preferred_header("es;q=invalid,en;q=0.5"), Some(Language::English));
        assert_eq!(preferred_header("es;q=NaN,en;q=0.5"), Some(Language::English));
    }
}
