use super::*;
use crate::core::{design::DesignLayer, models::fonts::THEME_FONT};

fn position(output: &egui::FullOutput, label: &str) -> egui::Pos2 {
    output
        .shapes
        .iter()
        .find_map(|shape| {
            if let egui::Shape::Text(text) = &shape.shape
                && text.galley.job.text.contains(label)
            {
                Some(text.pos + text.galley.rect.center().to_vec2())
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("control not found: {label}"))
}

fn click(input: &egui::RawInput, point: egui::Pos2) -> egui::RawInput {
    let mut input = input.clone();
    input.events.push(egui::Event::PointerMoved(point));
    for pressed in [true, false] {
        input.events.push(egui::Event::PointerButton {
            pos: point,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
    }
    input
}

#[test]
fn text_inspector_selects_theme_default_and_numeric_weight_below_the_font() {
    let ctx = egui::Context::default();
    crate::ui::i18n::install(&ctx, crate::core::i18n::Language::Portuguese);
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(320.0, 1400.0),
        )),
        ..Default::default()
    };
    let mut layer = DesignLayer::new(LayerContent::Text("Texto de exemplo".into()));
    let render = |input, layer: &mut DesignLayer| {
        let mut output = ctx.run_ui(input, |ui| {
            render_text_tab(ui, layer, 0, Some("Montserrat"), &AppTheme::dark(), &mut false);
        });
        output.textures_delta.clear();
        output
    };
    let output = render(input.clone(), &mut layer);
    let font_position = position(&output, "Padrão da interface");
    assert!(position(&output, "Peso").y > font_position.y);
    render(click(&input, font_position), &mut layer);
    let output = render(input.clone(), &mut layer);
    render(
        click(&input, position(&output, "Padrão do tema")),
        &mut layer,
    );
    assert_eq!(layer.font_family, THEME_FONT);
    let output = render(input.clone(), &mut layer);
    render(click(&input, position(&output, "400 — Normal")), &mut layer);
    let output = render(input.clone(), &mut layer);
    render(click(&input, position(&output, "300 — Clara")), &mut layer);
    assert_eq!(layer.font_weight, Some(300));
    assert!(!layer.bold);
}
