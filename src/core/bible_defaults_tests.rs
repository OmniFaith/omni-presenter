use super::*;

#[test]
fn bible_defaults_match_the_requested_reference_layout_and_typography() {
    let settings = BibleSettings::default();
    assert_eq!(
        settings.reference_position,
        BibleReferencePosition::BelowVerse
    );
    assert_eq!(settings.reference_alignment, BibleReferenceAlignment::Right);
    assert_eq!(settings.alignment, BibleVerseAlignment::Center);
    assert_eq!(settings.vertical_alignment, BibleVerticalAlignment::Center);
    assert_eq!(settings.separator, BibleSeparator::Dot);
    assert_eq!(settings.verse_number_style, BibleVerseNumberStyle::Bracket);
    assert_eq!(settings.reference_size, BibleReferenceSize::Normal);
    assert!(settings.range_number_only);
    assert!(settings.show_verse_numbers);
    assert!(settings.quotes_around_verse);
    assert!(!settings.show_version);
    assert!(!settings.use_abbreviation);
    assert!(!settings.uppercase_reference);
    assert!(!settings.italic_reference);
    assert_eq!(
        BibleReferencePosition::default(),
        settings.reference_position
    );
    assert_eq!(BibleSeparator::default(), settings.separator);
    assert_eq!(
        serde_json::from_str::<BibleSettings>("{}").unwrap(),
        settings
    );
}

#[test]
fn explicit_saved_choices_and_legacy_highlight_values_are_preserved() {
    let settings: BibleSettings = serde_json::from_str(
        r#"{
        "reference_position": "Bottom", "separator": "Colon",
        "range_number_only": false, "quotes_around_verse": false,
        "reference_highlight": "badge"
    }"#,
    )
    .unwrap();
    assert_eq!(settings.reference_position, BibleReferencePosition::Bottom);
    assert_eq!(settings.separator, BibleSeparator::Colon);
    assert!(!settings.range_number_only);
    assert!(!settings.quotes_around_verse);
    assert_eq!(settings.reference_highlight, BibleReferenceHighlight::Badge);
    let restored: BibleSettings =
        serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
    assert_eq!(restored, settings);
}

#[test]
fn default_reference_uses_a_dot_below_the_quoted_verse() {
    let (_, content) = format_bible_slide(
        &BibleSettings::default(),
        "Romanos",
        "Rm",
        "NTLH",
        8,
        28,
        28,
        &["Tudo coopera para o bem.".into()],
    );
    assert!(content.contains("//ref-pos: below\n"));
    let visible = crate::core::slides::strip_directive_commands(&content);
    assert_eq!(visible, "“Tudo coopera para o bem.”\nRomanos 8.28");
}
