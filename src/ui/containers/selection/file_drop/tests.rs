use super::super::*;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Debug)]
struct LocalFile(PathBuf);
impl egui::DroppedFile for LocalFile {
    fn path(&self) -> &Path {
        &self.0
    }
    fn bytes(&self) -> Result<Vec<u8>, String> {
        panic!("File drops must not read media bytes on the UI thread")
    }
}
fn frame(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    position: egui::Pos2,
    file: Option<PathBuf>,
) -> bool {
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(400.0, 400.0),
        )),
        events: vec![egui::Event::PointerMoved(position)],
        ..Default::default()
    };
    if let Some(file) = file {
        input.dropped_files.push(Arc::new(LocalFile(file)));
    }
    let mut remaining = false;
    let mut output = ctx.run_ui(input, |ui| {
        super::handle(
            ui,
            state,
            egui::Rect::from_min_max(egui::pos2(10.0, 10.0), egui::pos2(220.0, 350.0)),
        );
        remaining = ui.input(|input| !input.raw.dropped_files.is_empty());
    });
    output.textures_delta.clear();
    remaining
}
fn finish(ctx: &egui::Context, state: &mut ControlPanelState, db: &AppDb) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while state.media_import_task.is_some() {
        poll_media_import(ctx, state, Some(db));
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn dropping_a_file_imports_once_and_opens_media_without_reading_image_bytes() {
    let ctx = egui::Context::default();
    let db = AppDb::open_in_memory().unwrap();
    let mut state = ControlPanelState::default();
    state.selected_tab = 4;
    let path = std::env::temp_dir().join(format!("{}.png", crate::core::preferences::new_id("drop-test")));
    image::RgbaImage::new(2, 2).save(&path).unwrap();
    let inside = egui::pos2(80.0, 80.0);
    frame(&ctx, &mut state, inside, None);
    assert!(!frame(&ctx, &mut state, inside, Some(path.clone())));
    assert_eq!(state.selected_tab, 1);
    finish(&ctx, &mut state, &db);
    assert_eq!(state.media_items.len(), 1);
    assert_eq!(db.get_all_media_items().unwrap().len(), 1);
    frame(&ctx, &mut state, inside, Some(path.clone()));
    finish(&ctx, &mut state, &db);
    assert_eq!(state.media_items.len(), 1);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn a_drop_outside_the_library_is_not_consumed_or_imported() {
    let ctx = egui::Context::default();
    let mut state = ControlPanelState::default();
    let outside = egui::pos2(320.0, 80.0);
    frame(&ctx, &mut state, outside, None);
    assert!(frame(
        &ctx,
        &mut state,
        outside,
        Some(PathBuf::from("file.mp4"))
    ));
    assert!(state.media_import_task.is_none());
    assert_eq!(state.selected_tab, 0);
}
