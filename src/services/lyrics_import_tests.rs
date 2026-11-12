use super::*;
use std::io::Write;

struct Fixture(std::path::PathBuf);
impl Fixture {
    fn new(extension: &str, bytes: &[u8]) -> Self {
        let mut path =
            std::env::temp_dir().join(crate::core::preferences::new_id("omni_lyrics_test"));
        if !extension.is_empty() {
            path.set_extension(extension);
        }
        std::fs::write(&path, bytes).unwrap();
        Self(path)
    }
    fn zip(extension: &str, entries: &[(&str, &str)]) -> Self {
        let file = Self::new(extension, b"");
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&file.0).unwrap());
        for (name, content) in entries {
            zip.start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
        file
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[test]
fn extensionless_and_bom_text_preserve_markup_and_extract_metadata() {
    let file = Fixture::new("", "\u{feff}//título: Canção\r\n//artista: Autor\r\n//pasta: Culto\r\n//tags: fé\r\n#*refrão\r\n//c: #FFF\r\n**Vivo** | aqui\r\n\r\nOutro slide".as_bytes());
    let song = read(&file.0).unwrap();
    assert_eq!(song.title, "Canção");
    assert_eq!(song.artist, "Autor");
    assert_eq!(song.folder, "Culto");
    assert_eq!(song.tags, "fé");
    assert_eq!(
        crate::core::slides::parse_slides_from_text(&song.text).len(),
        2
    );
    assert!(song.text.contains("//c: #FFF\n**Vivo** | aqui"));
    assert!(!song.text.contains("//título"));
}

#[test]
fn utf16_and_invalid_binary_are_handled_explicitly() {
    for little in [true, false] {
        let mut bytes = if little {
            vec![0xff, 0xfe]
        } else {
            vec![0xfe, 0xff]
        };
        for unit in "Olá\nLuz".encode_utf16() {
            bytes.extend(if little {
                unit.to_le_bytes()
            } else {
                unit.to_be_bytes()
            });
        }
        assert_eq!(
            read(&Fixture::new("txt", &bytes).0).unwrap().text,
            "Olá\nLuz"
        );
    }
    assert!(read(&Fixture::new("txt", b"bad\x00file").0).is_err());
    assert!(read(&Fixture::new("txt", &[0xff, 0xfe, 0]).0).is_err());
    assert!(read(&Fixture::new("ppt", b"binary").0).is_err());
    assert!(read(&Fixture::new("txt", b"// comment only").0).is_err());
}

#[test]
fn lrc_and_subtitles_strip_timing_not_lyrics() {
    let song = read(
        &Fixture::new(
            "lrc",
            b"[ti:Song]\n[ar:Artist]\n[00:01.20][00:05.00]Hello\n[00:02.00]123",
        )
        .0,
    )
    .unwrap();
    assert_eq!(song.title, "Song");
    assert_eq!(song.artist, "Artist");
    assert_eq!(song.text, "Hello\n123");
    for ext in ["srt", "vtt"] {
        let song = read(&Fixture::new(ext, b"WEBVTT\n\n1\n00:00:00.000 --> 00:00:02.000\n123\n\n2\n00:00:02.000 --> 00:00:04.000\nWorld").0).unwrap();
        assert_eq!(song.text, "123\n\nWorld");
    }
}

#[test]
fn pptx_uses_presentation_order_and_joins_runs_within_paragraphs() {
    let file = Fixture::zip(
        "pptx",
        &[
            (
                "ppt/presentation.xml",
                r#"<p:presentation xmlns:p="p" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:sldIdLst><p:sldId id="1" r:id="rB"/><p:sldId id="2" r:id="rA"/></p:sldIdLst></p:presentation>"#,
            ),
            (
                "ppt/_rels/presentation.xml.rels",
                r#"<Relationships><Relationship Id="rA" Target="slides/slide1.xml"/><Relationship Id="rB" Target="slides/slide10.xml"/></Relationships>"#,
            ),
            (
                "ppt/slides/slide1.xml",
                r#"<s xmlns:a="a"><a:p><a:r><a:t>Second</a:t></a:r></a:p></s>"#,
            ),
            (
                "ppt/slides/slide10.xml",
                r#"<s xmlns:a="a"><a:p><a:r><a:t>First &amp; </a:t></a:r><a:r><a:t>light</a:t></a:r></a:p><a:p><a:r><a:t>Next line</a:t></a:r></a:p></s>"#,
            ),
        ],
    );
    assert_eq!(
        read(&file.0).unwrap().text,
        "First & light\nNext line\n\nSecond"
    );
}

#[test]
fn docx_odt_and_odp_extract_text_and_boundaries() {
    let file = Fixture::zip(
        "docx",
        &[(
            "word/document.xml",
            r#"<d xmlns:w="w"><w:p><w:r><w:t>Word</w:t></w:r></w:p><w:p/><w:p><w:r><w:t>Next</w:t></w:r></w:p></d>"#,
        )],
    );
    assert_eq!(read(&file.0).unwrap().text, "Word\n\nNext");
    let xml = r#"<d xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0" xmlns:text="t"><draw:page><text:p>One <text:span>light</text:span></text:p></draw:page><draw:page><text:p>Two</text:p></draw:page></d>"#;
    assert_eq!(
        read(&Fixture::zip("odp", &[("content.xml", xml)]).0)
            .unwrap()
            .text,
        "One light\n\nTwo"
    );
    assert_eq!(
        read(&Fixture::zip("odt", &[("content.xml", xml)]).0)
            .unwrap()
            .text,
        "One light\nTwo"
    );
}

#[test]
fn example_is_importable_and_documented_directives_parse() {
    let song = read(&Fixture::new("txt", EXAMPLE.as_bytes()).0).unwrap();
    assert_eq!(song.title, "Exemplo completo de letras — Omni Presenter");
    assert!(song.text.contains("#*refrão"));
    let directives = crate::core::markup::parse_slide_directives(
        "//ref: Test\n//ref-pos: below\n//ref-align: right\n//ref-size: small\n//va: centro",
    );
    assert_eq!(directives.ref_pos.as_deref(), Some("below"));
    assert_eq!(directives.valign.as_deref(), Some("center"));
    assert_eq!(crate::core::markup::parse_hex_color("áx"), None);
    assert_eq!(crate::core::markup::parse_hex_color("日本"), None);
}
