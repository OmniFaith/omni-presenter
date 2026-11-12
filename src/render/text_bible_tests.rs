use super::*;
use crate::core::bible::{BibleReferencePosition, BibleSettings, format_bible_slide};

#[test]
fn top_and_bottom_bible_references_do_not_move_the_centered_verse() {
    for size in [egui::vec2(832.0, 468.0), egui::vec2(1280.0, 720.0)] {
        let screen = Rect::from_min_size(egui::pos2(40.0, 60.0), size);
        let ctx = egui::Context::default();
        let theme = PresentationTheme {
            shadow_enabled: false,
            box_y: 0.3,
            box_h: 0.6,
            ..PresentationTheme::default_themes().remove(0)
        };
        let mut verse_bounds = Vec::new();
        for position in [BibleReferencePosition::Top, BibleReferencePosition::Bottom] {
            let settings = BibleSettings {
                reference_position: position,
                show_version: true,
                ..Default::default()
            };
            let (_, content) = format_bible_slide(
                &settings, "Salmos", "Sl", "NTLH", 39, 4, 4,
                &["Ó Senhor Deus, quanto tempo ainda vou viver? Mostra-me como é passageira a minha vida. Quando é que vou morrer?".into()],
            );
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, screen.max.to_vec2())),
                    ..Default::default()
                },
                |ui| render(ui.painter(), screen, &theme, &content),
            );
            let texts: Vec<_> = output.shapes.iter().filter_map(|s| {
                if let egui::Shape::Text(t) = &s.shape {
                    Some((s.clip_rect, t.visual_bounding_rect()))
                } else {
                    None
                }
            }).collect();
            output.textures_delta.clear();
            assert_eq!(texts.len(), 2);
            let (clip, verse) = texts[0];
            let (reference_clip, reference) = texts[1];
            assert!(clip.contains_rect(verse), "Verse is clipped: {texts:?}");
            assert!(reference_clip.contains_rect(reference), "Reference is clipped: {texts:?}");
            assert!(!verse.intersects(reference), "Reference overlaps verse: {texts:?}");
            assert!(
                (verse.center().y - screen.center().y).abs() < 16.0 * size.y / 720.0,
                "Verse must be centered on the slide: {verse:?}, {screen:?}"
            );
            match position {
                BibleReferencePosition::Top => assert!(reference.bottom() < verse.top()),
                _ => assert!(reference.top() > verse.bottom()),
            }
            verse_bounds.push(verse);
        }
        assert_eq!(verse_bounds[0], verse_bounds[1], "Moving the reference must not move the verse");
    }
}
