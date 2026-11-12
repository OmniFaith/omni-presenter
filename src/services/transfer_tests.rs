use super::*;

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(crate::core::preferences::new_id("omni_transfer_test"));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn portable_roundtrip_restores_settings_catalogs_and_real_assets_with_recovery() {
    let directory = Directory::new();
    let db = AppDb::open_in_memory().unwrap();
    let settings = AppSettings::default();
    let asset = directory.0.join("photo.png");
    std::fs::write(&asset, b"image bytes").unwrap();
    db.add_song("Portable", "Artist", "//c: #FFF\n**Lyrics**")
        .unwrap();
    db.add_media_item("Photo", &asset.to_string_lossy(), "image", "Media folder")
        .unwrap();
    let archive = directory.0.join("workspace.zip");
    let package = export(&db, &settings, &archive).unwrap();
    assert_eq!(package.assets.len(), 1);
    assert_eq!(
        inspect(&archive, &db).unwrap().tables.len(),
        super::super::db::transfer::TABLES.len()
    );
    let target = AppDb::open_in_memory().unwrap();
    target.add_song("Before restore", "", "old").unwrap();
    let recovery = directory.0.join("recovery.zip");
    restore(
        &target,
        &settings,
        &archive,
        &recovery,
        &directory.0.join("assets"),
    )
    .unwrap();
    assert!(recovery.is_file());
    assert!(
        inspect(&recovery, &target)
            .unwrap()
            .tables
            .iter()
            .find(|t| t.name == "songs")
            .unwrap()
            .rows
            .iter()
            .flatten()
            .any(|c| matches!(c, Cell::Text(t) if t == "Before restore"))
    );
    assert_eq!(target.load_settings().unwrap(), Some(settings));
    assert!(
        target
            .get_all_songs()
            .unwrap()
            .iter()
            .any(|s| s.title == "Portable")
    );
    assert!(
        !target
            .get_all_songs()
            .unwrap()
            .iter()
            .any(|s| s.title == "Before restore")
    );
    let media = target.get_all_media_items().unwrap();
    let imported = media.iter().find(|m| m.title == "Photo").unwrap();
    assert_ne!(imported.file_path, asset.to_string_lossy());
    std::fs::remove_file(asset).unwrap();
    assert_eq!(std::fs::read(&imported.file_path).unwrap(), b"image bytes");
}

#[test]
fn invalid_schema_and_failed_insert_never_partially_replace_the_database() {
    let db = AppDb::open_in_memory().unwrap();
    db.add_song("Keep me", "", "text").unwrap();
    let snapshot = db.transfer_snapshot().unwrap();
    let mut bad = snapshot.clone();
    bad[0].name = "songs; DROP TABLE themes".into();
    assert!(db.restore_transfer(&bad).is_err());
    let mut bad = snapshot.clone();
    let table = bad.iter_mut().find(|t| t.name == "songs").unwrap();
    table.rows.push(table.rows[0].clone()); // Duplicate PK fails after earlier table deletes.
    assert!(db.restore_transfer(&bad).is_err());
    assert_eq!(
        serde_json::to_value(db.transfer_snapshot().unwrap()).unwrap(),
        serde_json::to_value(snapshot).unwrap()
    );
}

#[test]
fn json_paths_are_relocated_without_replacing_lyric_text_and_urls() {
    let mut tables = vec![Table { name: "design_documents".into(), columns: vec!["value".into()], rows: vec![vec![Cell::Text(r#"{"layers":[{"Image":{"path":"/old/a.png"}}],"text":"/old/a.png","playlist":[{"kind":"Media","content":"/old/a.png"}],"url":"https://example.org"}"#.into())]] }];
    visit_paths(&mut tables, |s| {
        if s == "/old/a.png" {
            *s = "/new/a.png".into();
        }
    })
    .unwrap();
    let Cell::Text(text) = &tables[0].rows[0][0] else {
        panic!()
    };
    let value: Value = serde_json::from_str(text).unwrap();
    assert_eq!(value["layers"][0]["Image"]["path"], "/new/a.png");
    assert_eq!(value["playlist"][0]["content"], "/new/a.png");
    assert_eq!(value["text"], "/old/a.png");
    assert_eq!(value["url"], "https://example.org");
}

#[test]
fn unknown_versions_and_unsafe_assets_are_rejected_without_writes() {
    let directory = Directory::new();
    let db = AppDb::open_in_memory().unwrap();
    let path = directory.0.join("bad.zip");
    let mut package = Package {
        format: "omni-presenter-workspace".into(),
        version: 2,
        application_version: "test".into(),
        tables: db.transfer_snapshot().unwrap(),
        assets: BTreeMap::new(),
        missing: Vec::new(),
    };
    let write = |package: &Package| {
        let mut zip = ZipWriter::new(File::create(&path).unwrap());
        zip.start_file("manifest.json", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(&serde_json::to_vec(package).unwrap())
            .unwrap();
        zip.finish().unwrap();
    };
    write(&package);
    assert!(inspect(&path, &db).is_err());
    package.version = 1;
    package
        .assets
        .insert("original".into(), "assets/../../escape".into());
    write(&package);
    assert!(inspect(&path, &db).is_err());
}

#[test]
fn lyrics_import_is_additive_and_preserves_existing_homonyms() {
    let db = AppDb::open_in_memory().unwrap();
    db.add_song("Same title", "", "Original").unwrap();
    let song = crate::core::models::Song {
        id: None,
        title: "Same title".into(),
        artist: "".into(),
        text: "Imported".into(),
        folder: "New folder".into(),
        tags: "import".into(),
    };
    db.import_lyrics(&[song]).unwrap();
    let songs = db.get_all_songs().unwrap();
    assert!(songs.iter().any(|s| s.text == "Original"));
    assert!(
        songs
            .iter()
            .any(|s| s.text == "Imported" && s.tags == "import")
    );
    assert!(db.get_folders().unwrap().contains(&"New folder".into()));
}

#[test]
fn failed_recovery_export_aborts_before_replacing_anything() {
    let directory = Directory::new();
    let db = AppDb::open_in_memory().unwrap();
    let settings = AppSettings::default();
    let archive = directory.0.join("source.zip");
    export(&db, &settings, &archive).unwrap();
    db.add_song("Keep original", "", "Before import").unwrap();
    let original = serde_json::to_value(db.transfer_snapshot().unwrap()).unwrap();
    let recovery = directory.0.join("does-not-exist/recovery.zip");
    assert!(
        restore(
            &db,
            &settings,
            &archive,
            &recovery,
            &directory.0.join("assets")
        )
        .is_err()
    );
    assert_eq!(
        serde_json::to_value(db.transfer_snapshot().unwrap()).unwrap(),
        original
    );
    assert!(!directory.0.join("assets").exists());
}

#[test]
fn missing_local_files_are_reported_but_online_media_is_not_downloaded() {
    let directory = Directory::new();
    let db = AppDb::open_in_memory().unwrap();
    let missing = directory
        .0
        .join("missing.png")
        .to_string_lossy()
        .into_owned();
    db.add_media_item("Missing", &missing, "image", "").unwrap();
    db.add_media_item("Online", "https://example.org/video.mp4", "video", "")
        .unwrap();
    let package = export(
        &db,
        &AppSettings::default(),
        &directory.0.join("backup.zip"),
    )
    .unwrap();
    assert_eq!(package.missing, vec![missing]);
    assert!(package.assets.is_empty());
}

#[test]
fn unicode_zip_names_are_checked_against_the_manifest_without_modifying_workspace() {
    let directory = Directory::new();
    let db = AppDb::open_in_memory().unwrap();
    let path = directory.0.join("names.zip");
    let mut package = export(&db, &AppSettings::default(), &path).unwrap();
    let name = "invalid-é.txt";
    let entry = format!("assets/{name}");
    package.assets.insert("original.txt".into(), entry.clone());
    let mut zip = ZipWriter::new(File::create(&path).unwrap());
    zip.start_file("manifest.json", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(&serde_json::to_vec(&package).unwrap())
        .unwrap();
    zip.start_file(&entry, SimpleFileOptions::default())
        .unwrap();
    zip.write_all(b"test").unwrap();
    zip.finish().unwrap();
    // Valid Unicode asset names must still work with zip 9's fallible decoder.
    assert!(inspect(&path, &db).is_ok());
    let before = serde_json::to_value(db.transfer_snapshot().unwrap()).unwrap();
    let mut bytes = std::fs::read(&path).unwrap();
    let positions: Vec<_> = bytes
        .windows(entry.len())
        .enumerate()
        .filter(|(i, value)| {
            *value == entry.as_bytes()
                && ((*i >= 30 && &bytes[i - 30..i - 26] == b"PK\x03\x04")
                    || (*i >= 46 && &bytes[i - 46..i - 42] == b"PK\x01\x02"))
        })
        .map(|(i, _)| i)
        .collect();
    assert_eq!(positions.len(), 2); // Only headers, never the manifest contents.
    for position in positions {
        bytes[position + "assets/invalid-".len()] = 0xff;
    }
    std::fs::write(&path, bytes).unwrap();
    // zip may decode non-UTF-8 bytes as CP437. Regardless of that fallback,
    // a renamed asset must not satisfy the original manifest reference.
    assert!(inspect(&path, &db).is_err());
    assert_eq!(
        serde_json::to_value(db.transfer_snapshot().unwrap()).unwrap(),
        before
    );
}

#[test]
fn complete_backup_retains_profiles_playlists_designs_bibles_themes_and_linked_songs() {
    let directory = Directory::new();
    let db = AppDb::open_in_memory().unwrap();
    let mut settings = AppSettings::default();
    settings.operators[0].name = "Ana · projeção".into();
    settings.operators[0].avatar = Some(vec![1, 2, 3]);
    settings.services[0].playlist = crate::core::state::ControlPanelState::default().playlist;
    settings.services[0].bible_shortcuts = vec![crate::core::bible::DEFAULT_BIBLE.into()];
    let first = db.add_song("First version", "", "Original").unwrap();
    let second = db.add_song("Second version", "", "Translation").unwrap();
    db.set_song_versions(first, &[second]).unwrap();
    db.add_custom_text("Notice", "Welcome", "Notices").unwrap();
    db.add_saved_verse("John 3:16", "NVI", "Verse text", "Verses")
        .unwrap();
    let design = crate::core::design::DesignDocument::template(2);
    db.save_design(&design).unwrap();
    let bible = crate::core::bible::BibleLibrary::default()
        .get(crate::core::bible::DEFAULT_BIBLE)
        .unwrap()
        .clone();
    db.save_bible_translation(&bible).unwrap();
    let themes = db.get_all_themes().unwrap();
    let archive = directory.0.join("complete.zip");
    export(&db, &settings, &archive).unwrap();
    let target = AppDb::open_in_memory().unwrap();
    restore(
        &target,
        &AppSettings::default(),
        &archive,
        &directory.0.join("recovery.zip"),
        &directory.0.join("assets"),
    )
    .unwrap();
    assert_eq!(target.load_settings().unwrap(), Some(settings));
    assert_eq!(target.song_version_ids(first).unwrap(), vec![second]);
    assert_eq!(target.load_designs().unwrap(), vec![design]);
    assert_eq!(target.get_all_themes().unwrap(), themes);
    assert_eq!(target.get_all_custom_texts().unwrap()[0].content, "Welcome");
    assert_eq!(target.get_all_saved_verses().unwrap()[0].text, "Verse text");
    assert_eq!(target.get_bible_translations().unwrap()[0].id, bible.id);
}
