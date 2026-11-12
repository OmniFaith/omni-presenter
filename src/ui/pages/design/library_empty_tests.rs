use super::*;

fn frame(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    let mut deck = SlideDeck::default();
    deck.designs.clear();
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(520.0, 480.0),
            )),
            events,
            ..Default::default()
        },
        |ui| render_selection(ui, state, &mut deck, &mut false, None),
    );
    output.textures_delta.clear();
    output
}

fn text_position(output: &egui::FullOutput, label: &str) -> egui::Pos2 {
    output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == label => {
                Some(text.pos + text.galley.rect.center().to_vec2())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("Missing UI text: {label}"))
}

#[test]
fn empty_design_library_uses_centered_library_title_hint_and_action() {
    let ctx = egui::Context::default();
    let mut state = ControlPanelState::default();
    let language = state.settings.language;
    let output = frame(&ctx, &mut state, vec![]);
    let action = language.format("design-create-design", &[("icon", PLUS)]);
    for label in [
        language.text("common-empty"),
        language.text("design-your-designs-will-appear-here-automatically"),
        &action,
    ] {
        assert!((text_position(&output, label).x - 260.0).abs() < 2.0);
    }
}

#[test]
fn empty_design_search_can_be_cleared_using_the_shared_action() {
    let ctx = egui::Context::default();
    let mut state = ControlPanelState {
        search_query: "missing-design".into(),
        ..Default::default()
    };
    let output = frame(&ctx, &mut state, vec![]);
    let position = text_position(
        &output,
        state.settings.language.text("library-clear-search"),
    );
    let _ = frame(
        &ctx,
        &mut state,
        vec![
            egui::Event::PointerMoved(position),
            egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
            egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            },
        ],
    );
    assert!(state.search_query.is_empty());
}
