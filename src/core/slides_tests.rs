use super::models::Song;
use super::slides::*;

#[test]
fn test_strip_comments() {
    let input = "// Intro: G D Em\nPorque Deus amou o mundo\n// Todos juntos\nDe tal maneira";
    let stripped = strip_comments(input);
    assert_eq!(stripped, "Porque Deus amou o mundo\nDe tal maneira");

    let only_comments = "// Linha 1\n  // Linha 2";
    assert_eq!(strip_comments(only_comments), "");

    let no_comments = "Sem nenhum comentário\nApenas louvor";
    assert_eq!(strip_comments(no_comments), no_comments);

    let inline_comment = "Nada havia em Sua aparência // bis\nPara o desejarmos";
    assert_eq!(strip_comments(inline_comment), "Nada havia em Sua aparência\nPara o desejarmos");

    let tag_and_comment = "#*Refrão\n// bis\nNada havia // nota";
    assert_eq!(strip_comments(tag_and_comment), "Nada havia");
}

#[test]
fn test_strophe_tags() {
    let strophe = "#*Refrão\nPorque Ele vive\nPosso crer no amanhã";
    assert_eq!(extract_strophe_tag(strophe), Some("Refrão".to_string()));
    assert_eq!(strip_strophe_tag(strophe), "Porque Ele vive\nPosso crer no amanhã");

    let changed = set_strophe_tag(strophe, Some("Ponte"));
    assert_eq!(extract_strophe_tag(&changed), Some("Ponte".to_string()));
    assert!(changed.starts_with("#*Ponte\n"));

    let removed = set_strophe_tag(&changed, None);
    assert_eq!(extract_strophe_tag(&removed), None);
    assert_eq!(removed, "Porque Ele vive\nPosso crer no amanhã");
}

#[test]
fn test_double_enter_splitting() {
    let text = "Estrofe 1\nLinha 2\n\nEstrofe 2\nLinha 2\n\n\nEstrofe 3";
    let slides = parse_slides_from_text(text);
    assert_eq!(slides.len(), 3);
    assert_eq!(slides[0], "Estrofe 1\nLinha 2");
    assert_eq!(slides[1], "Estrofe 2\nLinha 2");
    assert_eq!(slides[2], "Estrofe 3");
}

#[test]
fn test_navigation_and_hiding() {
    let mut deck = SlideDeck::new("Slide 1\n\nSlide 2\n\nSlide 3");
    assert_eq!(deck.current_slide_text(), "Slide 1");
    assert_eq!(deck.projected_text(), "Slide 1");

    // Hide with ESC
    deck.toggle_hide_content();
    assert!(deck.is_content_hidden());
    assert_eq!(deck.projected_text(), "");

    // Next slide restores visibility
    deck.next_slide();
    assert_eq!(deck.current_index(), 1);
    assert!(!deck.is_content_hidden());
    assert_eq!(deck.projected_text(), "Slide 2");
}

#[test]
fn test_song_selection_only_changes_preview_not_live() {
    let songs = vec![
        Song::new(
            "Deus de Promessas",
            "Davi Sacer",
            "Sei que os Teus olhos\nSempre atentos\npermanecem em mim",
        ),
        Song::new(
            "Isaías 53",
            "Projeto Sola",
            "Ele não tinha qualquer beleza\nOu majestade pra nos atrair\n\nMais rejeitado dentre os homens\nAinda assim carregou nossas dores\nO Seu castigo nos traz paz\nE a esperança não se esvai",
        ),
    ];
    let mut deck = SlideDeck::from_catalog(songs, Vec::new());
    deck.project_song(1);
    deck.select_live_slide(1);
    assert_eq!(deck.live_title(), "Isaías 53");
    assert_eq!(deck.title(), "Isaías 53");

    // Operator selects song 0 ("Deus de Promessas") with a single click
    deck.preview_song(0);

    // Preview title updates to "Deus de Promessas"
    assert_eq!(deck.title(), "Deus de Promessas");
    assert_eq!(deck.preview_slide_text(), "Deus de Promessas\nDavi Sacer");

    // But LIVE is STILL "Isaías 53" completely untouched!
    assert_eq!(deck.live_title(), "Isaías 53");
    assert_eq!(
        deck.live_slide_text(),
        "Ele não tinha qualquer beleza\nOu majestade pra nos atrair"
    );
    assert_eq!(deck.projected_text(), deck.live_slide_text());

    // Now operator projects the preview (double click or "Projetar" button)
    deck.project_preview();

    // Now and ONLY now does LIVE update to "Deus de Promessas"!
    assert_eq!(deck.live_title(), "Deus de Promessas");
    assert_eq!(deck.live_slide_text(), "Deus de Promessas\nDavi Sacer");
    assert_eq!(deck.projected_text(), deck.live_slide_text());
}

#[test]
fn test_strophe_reordering_and_manipulation() {
    let mut deck = SlideDeck::new("Estrofe A\n\nEstrofe B\n\nEstrofe C");
    assert_eq!(deck.slides().len(), 3);
    assert_eq!(deck.slides()[0], "Estrofe A");
    assert_eq!(deck.slides()[1], "Estrofe B");
    assert_eq!(deck.slides()[2], "Estrofe C");

    // Move B up -> [B, A, C]
    deck.move_slide_up(1);
    assert_eq!(deck.slides()[0], "Estrofe B");
    assert_eq!(deck.slides()[1], "Estrofe A");
    assert_eq!(deck.slides()[2], "Estrofe C");

    // Move B down -> [A, B, C]
    deck.move_slide_down(0);
    assert_eq!(deck.slides()[0], "Estrofe A");
    assert_eq!(deck.slides()[1], "Estrofe B");

    // Duplicate B -> [A, B, B, C]
    deck.duplicate_slide(1);
    assert_eq!(deck.slides().len(), 4);
    assert_eq!(deck.slides()[1], "Estrofe B");
    assert_eq!(deck.slides()[2], "Estrofe B");

    // Delete duplicated strophe -> [A, B, C]
    deck.delete_slide(2);
    assert_eq!(deck.slides().len(), 3);
    assert_eq!(deck.slides()[2], "Estrofe C");

    // Update strophe B
    deck.update_slide_text(1, "Estrofe B Modificada");
    assert_eq!(deck.slides()[1], "Estrofe B Modificada");
}

#[test]
fn test_strip_directive_commands() {
    let text = "//t: 60\n//c: #FF0000\n// Intro: G D Em\nBless the Lord, oh my soul\n//f: Inter";
    let cleaned = strip_directive_commands(text);
    assert_eq!(cleaned, "// Intro: G D Em\nBless the Lord, oh my soul");
}

#[test]
fn test_custom_slides_with_media_sequential_navigation() {
    let mut deck = SlideDeck::default();

    let slides = vec![
        "Verse 1 of song".to_string(),
        "".to_string(), // video slide (empty text)
        "Romans 8:28".to_string(), // bible text
    ];
    let media = vec![
        None,
        Some(MediaOutput {
            path: "/path/to/video.mp4".into(),
            kind: "video".into(),
        }),
        None,
    ];

    deck.load_custom_slides_with_media(
        "Combined Service".into(),
        "Playlist".into(),
        slides,
        media,
    );

    // Initial preview is slide 0
    assert_eq!(deck.preview_media, None);
    assert_eq!(deck.preview_slide_text(), "Verse 1 of song");

    // Project the custom slides
    deck.project_preview();
    assert_eq!(deck.live_title(), "Combined Service");
    assert_eq!(deck.live_index(), 0);
    assert_eq!(deck.live_media, None);
    assert_eq!(deck.live_slide_text(), "Verse 1 of song");

    // Advance to slide 1 (video)
    assert!(deck.next_slide());
    assert_eq!(deck.live_index(), 1);
    assert_eq!(
        deck.live_media,
        Some(MediaOutput {
            path: "/path/to/video.mp4".into(),
            kind: "video".into(),
        })
    );
    assert_eq!(deck.live_slide_text(), "");

    // Advance to slide 2 (bible text)
    assert!(deck.next_slide());
    assert_eq!(deck.live_index(), 2);
    assert_eq!(deck.live_media, None);
    assert_eq!(deck.live_slide_text(), "Romans 8:28");

    // Go back to slide 1 (video)
    deck.prev_slide();
    assert_eq!(deck.live_index(), 1);
    assert_eq!(
        deck.live_media,
        Some(MediaOutput {
            path: "/path/to/video.mp4".into(),
            kind: "video".into(),
        })
    );

    // Go back to slide 0 (song)
    deck.prev_slide();
    assert_eq!(deck.live_index(), 0);
    assert_eq!(deck.live_media, None);
}
