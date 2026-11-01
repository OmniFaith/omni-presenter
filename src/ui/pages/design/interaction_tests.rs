use super::*;
use egui::{Color32, Event, PointerButton, Pos2, Rect, Shape};

fn input(ctx: &egui::Context, events: Vec<Event>) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(300.0, 720.0))),
        time: Some(ctx.input(|input| input.time) + 0.1),
        events,
        ..Default::default()
    }
}
fn pointer(position: Pos2, button: PointerButton, pressed: bool) -> Event {
    Event::PointerButton {
        pos: position,
        button,
        pressed,
        modifiers: Default::default(),
    }
}
fn click(position: Pos2, button: PointerButton) -> Vec<Event> {
    vec![
        Event::PointerMoved(position),
        pointer(position, button, true),
        pointer(position, button, false),
    ]
}
fn text_position(output: &egui::FullOutput, label: &str) -> Pos2 {
    output
        .shapes
        .iter()
        .rev()
        .find_map(|shape| match &shape.shape {
            Shape::Text(text) if text.galley.job.text == label => {
                Some(text.pos + text.galley.rect.center().to_vec2())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("Missing UI text: {label}"))
}
fn setup() -> (
    egui::Context,
    AppDb,
    SlideDeck,
    DesignEditor,
    ControlPanelState,
) {
    let ctx = egui::Context::default();
    crate::ui::i18n::install(&ctx, crate::core::i18n::Language::Portuguese);
    for theme in [egui::Theme::Light, egui::Theme::Dark] {
        ctx.style_mut_of(theme, |style| style.animation_time = 0.0);
    }
    ctx.data_mut(|data| data.insert_temp(egui::Id::unique("design_gpu_ready"), true));
    let db = AppDb::open_in_memory().unwrap();
    let mut deck = SlideDeck::new("Conteúdo ao vivo");
    let mut editor = DesignEditor::default();
    editor.new_scene(0, &db, &mut deck);
    editor.add_layer(LayerContent::Shape { radius: 8.0 });
    editor.scene.as_mut().unwrap().layers[0].name = "Camada A".into();
    editor.add_layer(LayerContent::Shape { radius: 8.0 });
    editor.scene.as_mut().unwrap().layers[1].name = "Camada B".into();
    let mut state = ControlPanelState::default();
    state.theme = AppTheme::dark();
    (ctx, db, deck, editor, state)
}
fn sidebar_frame(
    ctx: &egui::Context,
    events: Vec<Event>,
    db: &AppDb,
    deck: &mut SlideDeck,
    editor: &mut DesignEditor,
    state: &mut ControlPanelState,
) -> egui::FullOutput {
    let mut output = ctx.run_ui(input(ctx, events), |ui| {
        let theme = state.theme.clone();
        sidebar::render(ui, editor, deck, db, &theme, state);
    });
    output.textures_delta.clear();
    output
}
fn row_color(output: &egui::FullOutput, point: Pos2) -> Color32 {
    output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            Shape::Rect(rect)
                if rect.rect.contains(point)
                    && rect.rect.width() > 200.0
                    && (36.0..=39.0).contains(&rect.rect.height()) =>
            {
                Some(rect.fill)
            }
            _ => None,
        })
        .expect("Layer row background")
}

#[test]
fn selected_layer_highlights_over_its_label_and_context_actions_keep_undo() {
    let (ctx, db, mut deck, mut editor, mut state) = setup();
    sidebar_frame(&ctx, vec![], &db, &mut deck, &mut editor, &mut state);
    let idle = sidebar_frame(&ctx, vec![], &db, &mut deck, &mut editor, &mut state);
    let b = text_position(&idle, "Camada B");
    let base = row_color(&idle, b);
    let hovered = sidebar_frame(
        &ctx,
        vec![Event::PointerMoved(b)],
        &db,
        &mut deck,
        &mut editor,
        &mut state,
    );
    assert_ne!(base, row_color(&hovered, b));
    let held = sidebar_frame(
        &ctx,
        vec![pointer(b, PointerButton::Primary, true)],
        &db,
        &mut deck,
        &mut editor,
        &mut state,
    );
    assert_ne!(row_color(&hovered, b), row_color(&held, b));
    sidebar_frame(
        &ctx,
        vec![pointer(b, PointerButton::Primary, false)],
        &db,
        &mut deck,
        &mut editor,
        &mut state,
    );
    let a = text_position(&idle, "Camada A");
    sidebar_frame(
        &ctx,
        click(a, PointerButton::Secondary),
        &db,
        &mut deck,
        &mut editor,
        &mut state,
    );
    let menu = sidebar_frame(&ctx, vec![], &db, &mut deck, &mut editor, &mut state);
    let lock = text_position(&menu, "Bloquear camada");
    text_position(&menu, "Ocultar camada");
    sidebar_frame(
        &ctx,
        click(lock, PointerButton::Primary),
        &db,
        &mut deck,
        &mut editor,
        &mut state,
    );
    assert!(editor.scene.as_ref().unwrap().layers[0].locked);
    assert_eq!(editor.selected, Some(0));
    editor.history(false);
    assert!(!editor.scene.as_ref().unwrap().layers[0].locked);
    assert_eq!(deck.live_slide_text(), "Conteúdo ao vivo");
}

#[test]
fn blank_design_library_context_menu_creates_a_persisted_design() {
    let (ctx, db, mut deck, _, mut state) = setup();
    state.selected_tab = 4;
    let mut render = |events| {
        let mut output = ctx.run_ui(input(&ctx, events), |ui| {
            library::render_selection(ui, &mut state, &mut deck, &mut false, Some(&db))
        });
        output.textures_delta.clear();
        output
    };
    render(vec![]);
    render(click(egui::pos2(130.0, 450.0), PointerButton::Secondary));
    let menu = render(vec![]);
    let folder = text_position(&menu, "Nova pasta...");
    let create = menu
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            Shape::Text(text)
                if text.galley.job.text == "Novo design"
                    && (text.pos.y - folder.y).abs() < 100.0 =>
            {
                Some(text.pos + text.galley.rect.center().to_vec2())
            }
            _ => None,
        })
        .unwrap_or_else(|| text_position(&menu, "Novo design"));
    render(vec![Event::PointerMoved(create)]);
    render(vec![pointer(create, PointerButton::Primary, true)]);
    render(vec![pointer(create, PointerButton::Primary, false)]);
    assert_eq!(deck.designs.len(), 2);
    assert_eq!(db.load_designs().unwrap().len(), 2);
    assert_eq!(
        state.active_sidebar_page,
        crate::core::state::SidebarPage::Design
    );
    assert_ne!(deck.preview_design.as_ref().unwrap().id, deck.designs[0].id);
    assert_eq!(deck.live_slide_text(), "Conteúdo ao vivo");
}

#[test]
fn locked_layer_context_commands_preserve_content_and_unlock_is_undoable() {
    let (_, _, _, mut editor, _) = setup();
    editor.scene.as_mut().unwrap().layers[0].locked = true;
    let before = editor.scene.clone().unwrap();
    menus::apply_layer(&mut editor, 0, menus::LayerAction::Delete);
    menus::apply_layer(&mut editor, 0, menus::LayerAction::Forward);
    assert_eq!(editor.scene.as_ref(), Some(&before));
    menus::apply_layer(&mut editor, 0, menus::LayerAction::Lock);
    assert!(!editor.scene.as_ref().unwrap().layers[0].locked);
    editor.history(false);
    assert_eq!(editor.scene.as_ref(), Some(&before));
}

#[test]
fn selected_designs_have_hover_and_hold_feedback_in_list_and_grid() {
    for list in [true, false] {
        let (ctx, db, mut deck, _, mut state) = setup();
        state.design_list_view = list;
        state.selected_design_ids.insert(deck.designs[0].id.clone());
        let name = deck.designs[0].name.clone();
        let mut render = |events| {
            let mut output = ctx.run_ui(input(&ctx, events), |ui| {
                library::render_selection(ui, &mut state, &mut deck, &mut false, Some(&db))
            });
            output.textures_delta.clear();
            output
        };
        render(vec![]);
        let idle = render(vec![]);
        if list {
            let point = text_position(&idle, &name);
            let fill = |output: &egui::FullOutput| {
                output
                    .shapes
                    .iter()
                    .find_map(|shape| match &shape.shape {
                        Shape::Rect(rect)
                            if rect.fill.a() > 0
                                && rect.rect.contains(point)
                                && rect.rect.width() > 200.0
                                && (44.0..=47.0).contains(&rect.rect.height()) =>
                        {
                            Some(rect.fill)
                        }
                        _ => None,
                    })
                    .unwrap()
            };
            let hover = render(vec![Event::PointerMoved(point)]);
            let hold = render(vec![pointer(point, PointerButton::Primary, true)]);
            assert_ne!(fill(&idle), fill(&hover));
            assert_ne!(fill(&hover), fill(&hold));
        } else {
            let thumb = idle
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    Shape::Rect(rect)
                        if rect.fill == Color32::from_rgb(20, 24, 30)
                            && rect.rect.width() > 200.0 =>
                    {
                        Some(rect.rect)
                    }
                    _ => None,
                })
                .unwrap();
            let tint = |output: &egui::FullOutput| {
                output
                    .shapes
                    .iter()
                    .filter_map(|shape| match &shape.shape {
                        Shape::Rect(rect)
                            if rect.rect == thumb && rect.fill != Color32::from_rgb(20, 24, 30) =>
                        {
                            Some(rect.fill.a())
                        }
                        _ => None,
                    })
                    .max()
                    .unwrap_or(0)
            };
            let hover = render(vec![Event::PointerMoved(thumb.center())]);
            let hold = render(vec![pointer(thumb.center(), PointerButton::Primary, true)]);
            assert!(tint(&hover) > tint(&idle));
            assert!(tint(&hold) > tint(&hover));
        }
    }
}

#[test]
fn canvas_objects_highlight_on_hover_and_hold_and_open_the_layer_context_menu() {
    let (ctx, _, deck, mut editor, state) = setup();
    let mut render = |events| {
        let mut output = ctx.run_ui(input(&ctx, events), |ui| {
            canvas::render(ui, &mut editor, &state.theme, &state, &deck, false)
        });
        output.textures_delta.clear();
        output
    };
    render(vec![]);
    let idle = render(vec![]);
    let canvas = idle
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            Shape::Rect(rect)
                if rect.stroke.color == state.theme.border_default
                    && rect.rect.width() > 200.0
                    && rect.rect.height() > 100.0 =>
            {
                Some(rect.rect)
            }
            _ => None,
        })
        .unwrap();
    // Shape layers overlap by default; the topmost one must own the interaction.
    let target = Rect::from_min_size(
        canvas.min + egui::vec2(canvas.width() * 0.15, canvas.height() * 0.25),
        egui::vec2(canvas.width() * 0.7, canvas.height() * 0.5),
    );
    let point = target.center();
    let tint = |output: &egui::FullOutput| {
        output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                Shape::Rect(rect)
                    if rect.rect.contains(point)
                        && rect.fill.a() > 0
                        && rect.fill.a() < 100
                        && rect.rect.width() > 30.0 =>
                {
                    Some(rect.fill.a())
                }
                _ => None,
            })
            .max()
            .unwrap_or(0)
    };
    let hovered = render(vec![Event::PointerMoved(point)]);
    let held = render(vec![pointer(point, PointerButton::Primary, true)]);
    assert!(tint(&hovered) > tint(&idle));
    assert!(tint(&held) > tint(&hovered));
    render(vec![pointer(point, PointerButton::Primary, false)]);
    render(click(point, PointerButton::Secondary));
    let menu = render(vec![]);
    text_position(&menu, "Duplicar camada");
    text_position(&menu, "Bloquear camada");
    assert_eq!(editor.selected, Some(1));
}
