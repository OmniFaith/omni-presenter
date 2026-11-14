//! Text-only lyric import; never executes macros or imports presentation objects.
use crate::core::models::Song;
use std::{io::Read, path::Path};
use zip::ZipArchive;

const LIMIT: u64 = 16 * 1024 * 1024;
pub const EXAMPLE_PT: &str = include_str!("../../docs/exemplo-importacao-letras.txt");
pub const EXAMPLE_EN: &str = include_str!("../../docs/lyrics-import-example.txt");
pub const EXAMPLE_ES: &str = include_str!("../../docs/ejemplo-importacion-letras.txt");
pub const EXAMPLE: &str = EXAMPLE_PT;

pub fn example(language: crate::core::i18n::Language) -> &'static str {
    match language {
        crate::core::i18n::Language::Portuguese => EXAMPLE_PT,
        crate::core::i18n::Language::English => EXAMPLE_EN,
        crate::core::i18n::Language::Spanish => EXAMPLE_ES,
    }
}

pub fn example_filename(language: crate::core::i18n::Language) -> &'static str {
    match language {
        crate::core::i18n::Language::Portuguese => "exemplo-importacao-letras.txt",
        crate::core::i18n::Language::English => "lyrics-import-example.txt",
        crate::core::i18n::Language::Spanish => "ejemplo-importacion-letras.txt",
    }
}

fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn decode(bytes: &[u8]) -> Result<String, String> {
    let text = if bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff]) {
        if !bytes.len().is_multiple_of(2) {
            return Err("lyrics-invalid-text".into());
        }
        let little = bytes[0] == 0xff;
        let units: Vec<_> = bytes[2..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| {
                if little {
                    u16::from_le_bytes([c[0], c[1]])
                } else {
                    u16::from_be_bytes([c[0], c[1]])
                }
            })
            .collect();
        String::from_utf16(&units).map_err(error)?
    } else {
        std::str::from_utf8(bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes))
            .map_err(|_| "lyrics-invalid-text".to_owned())?
            .to_owned()
    };
    if text
        .chars()
        .any(|c| c == '\0' || (c.is_control() && !matches!(c, '\n' | '\r' | '\t')))
    {
        return Err("lyrics-invalid-text".into());
    }
    Ok(text.replace("\r\n", "\n").replace('\r', "\n"))
}

pub fn read(path: &Path) -> Result<Song, String> {
    if std::fs::metadata(path).map_err(error)?.len() > 128 * 1024 * 1024 {
        return Err("lyrics-too-large".into());
    }
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mut text = if matches!(extension.as_str(), "pptx" | "docx" | "odt" | "odp") {
        document(path, &extension)?
    } else {
        if !matches!(
            extension.as_str(),
            "" | "txt" | "text" | "md" | "markdown" | "lrc" | "srt" | "vtt"
        ) {
            return Err("lyrics-unsupported".into());
        }
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(error)?
            .take(LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(error)?;
        if bytes.len() as u64 > LIMIT {
            return Err("lyrics-too-large".into());
        }
        decode(&bytes)?
    };
    if extension == "lrc" {
        text = lrc(&text);
    }
    if matches!(extension.as_str(), "srt" | "vtt") {
        text = subtitles(&text);
    }
    let mut song = Song {
        id: None,
        title: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Lyrics")
            .to_owned(),
        artist: String::new(),
        text: String::new(),
        tags: String::new(),
        folder: String::new(),
    };
    let mut header = true;
    let mut lines = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if header {
            if let Some((key, value)) = trimmed.strip_prefix("//").and_then(|s| s.split_once(':')) {
                let target = match key.trim().to_lowercase().as_str() {
                    "title" | "titulo" | "título" => Some(&mut song.title),
                    "artist" | "artista" => Some(&mut song.artist),
                    "tags" | "etiquetas" => Some(&mut song.tags),
                    "folder" | "pasta" | "carpeta" => Some(&mut song.folder),
                    _ => None,
                };
                if let Some(target) = target {
                    *target = value.trim().to_owned();
                    continue;
                }
            }
            if !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("#*") {
                header = false;
            }
        }
        lines.push(line);
    }
    song.text = lines.join("\n").trim().to_owned();
    if song.title.trim().is_empty()
        || crate::core::slides::strip_comments(&song.text)
            .trim()
            .is_empty()
    {
        return Err("lyrics-empty".into());
    }
    Ok(song)
}

fn lrc(text: &str) -> String {
    text.lines()
        .filter_map(|line| {
            let mut line = line.trim();
            for (prefix, key) in [("[ti:", "title"), ("[ar:", "artist")] {
                if let Some(s) = line.strip_prefix(prefix).and_then(|s| s.strip_suffix(']')) {
                    return Some(format!("//{key}: {s}"));
                }
            }
            while let Some(s) = line.strip_prefix('[') {
                let Some((tag, rest)) = s.split_once(']') else {
                    break;
                };
                if !tag
                    .chars()
                    .all(|c| c.is_ascii_digit() || matches!(c, ':' | '.'))
                    || !tag.contains(':')
                {
                    return None;
                }
                line = rest;
            }
            Some(line.to_owned())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn subtitles(text: &str) -> String {
    text.split("\n\n")
        .filter_map(|block| {
            let lines: Vec<_> = block.lines().collect();
            let timing = lines.iter().position(|l| l.contains(" --> "))?;
            let text = lines[timing + 1..].join("\n");
            (!text.trim().is_empty()).then_some(text)
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn xml(zip: &mut ZipArchive<std::fs::File>, name: &str) -> Result<String, String> {
    let entry = zip.by_name(name).map_err(error)?;
    if entry.size() > LIMIT {
        return Err("lyrics-too-large".into());
    }
    let mut bytes = Vec::new();
    entry
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(error)?;
    if bytes.len() as u64 > LIMIT {
        return Err("lyrics-too-large".into());
    }
    decode(&bytes)
}

fn paragraphs(text: &str, word: bool) -> Result<String, String> {
    let doc = roxmltree::Document::parse(text).map_err(error)?;
    let mut out = Vec::new();
    for paragraph in doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "p")
    {
        let mut line = String::new();
        for n in paragraph.descendants().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "t" => line.push_str(n.text().unwrap_or("")),
                "br" => line.push('\n'),
                "tab" => line.push('\t'),
                _ => {}
            }
        }
        if !word && line.is_empty() {
            line = paragraph
                .descendants()
                .filter(|n| n.is_text())
                .map(|n| n.text().unwrap_or(""))
                .collect();
        }
        out.push(line);
    }
    Ok(out.join("\n"))
}

fn document(path: &Path, extension: &str) -> Result<String, String> {
    let mut zip = ZipArchive::new(std::fs::File::open(path).map_err(error)?).map_err(error)?;
    if zip.len() > 20_000 {
        return Err("lyrics-too-large".into());
    }
    if extension == "docx" {
        return paragraphs(&xml(&mut zip, "word/document.xml")?, true);
    }
    if matches!(extension, "odt" | "odp") {
        let content = xml(&mut zip, "content.xml")?;
        if extension == "odt" {
            return paragraphs(&content, false);
        }
        let doc = roxmltree::Document::parse(&content).map_err(error)?;
        return Ok(doc
            .descendants()
            .filter(|n| {
                n.has_tag_name(("urn:oasis:names:tc:opendocument:xmlns:drawing:1.0", "page"))
            })
            .map(|n| {
                n.descendants()
                    .filter(|n| n.is_element() && n.tag_name().name() == "p")
                    .map(|p| {
                        p.descendants()
                            .filter(|n| n.is_text())
                            .map(|n| n.text().unwrap_or(""))
                            .collect::<String>()
                    })
                    .filter(|s| !s.trim().is_empty())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n\n"));
    }
    let presentation = xml(&mut zip, "ppt/presentation.xml")?;
    let relationships = xml(&mut zip, "ppt/_rels/presentation.xml.rels")?;
    let doc = roxmltree::Document::parse(&presentation).map_err(error)?;
    let rels = roxmltree::Document::parse(&relationships).map_err(error)?;
    let mut slides = Vec::new();
    let mut total = 0;
    for slide in doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "sldId")
    {
        let id = slide
            .attribute((
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
                "id",
            ))
            .ok_or("lyrics-unsupported")?;
        let rel = rels
            .descendants()
            .find(|r| r.attribute("Id") == Some(id))
            .ok_or("lyrics-unsupported")?;
        if rel.attribute("TargetMode") == Some("External") {
            return Err("lyrics-unsupported".into());
        }
        let target = rel.attribute("Target").ok_or("lyrics-unsupported")?;
        let name = if target.starts_with('/') {
            target.trim_start_matches('/').to_owned()
        } else {
            format!("ppt/{target}")
        };
        if name.contains("..") || !name.starts_with("ppt/slides/") {
            return Err("lyrics-unsupported".into());
        }
        let slide = xml(&mut zip, &name)?;
        total += slide.len();
        if total > 64 * 1024 * 1024 {
            return Err("lyrics-too-large".into());
        }
        let text = paragraphs(&slide, true)?
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        if !text.trim().is_empty() {
            slides.push(text.trim().to_owned());
        }
    }
    Ok(slides.join("\n\n"))
}

#[cfg(test)]
#[path = "lyrics_import_tests.rs"]
mod tests;
