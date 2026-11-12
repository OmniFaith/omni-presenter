#[test]
fn rust_modules_stay_within_the_requested_line_limit() {
    fn visit(path: &std::path::Path, oversized: &mut Vec<String>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, oversized);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let lines = std::fs::read_to_string(&path).unwrap().lines().count();
                if lines > 1000 {
                    oversized.push(format!("{}: {lines} lines", path.display()));
                }
            }
        }
    }
    let mut oversized = Vec::new();
    visit(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut oversized,
    );
    assert!(
        oversized.is_empty(),
        "Split these modules by responsibility:\n{}",
        oversized.join("\n")
    );
}

#[test]
fn new_core_modules_do_not_depend_directly_on_ui_or_runtime_services() {
    // A lightweight source guard, not a full dependency/AST analysis. Existing
    // application-state adapters remain explicit exceptions until migrated.
    const LEGACY_ADAPTERS: &[&str] = &[
        "state/mod.rs",
        "state/lyrics_search.rs",
        "state/web_media.rs",
        "interface_theme.rs", // UI palette assertions in its inline test module.
    ];
    fn visit(root: &std::path::Path, path: &std::path::Path, violations: &mut Vec<String>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, violations);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let relative = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                let name = path.file_name().unwrap().to_string_lossy();
                if name == "tests.rs"
                    || name.ends_with("_tests.rs")
                    || relative.contains("/tests/")
                    || LEGACY_ADAPTERS.contains(&relative.as_str())
                {
                    continue;
                }
                let source = std::fs::read_to_string(&path).unwrap();
                if source.contains("crate::ui::") || source.contains("crate::services::") {
                    violations.push(relative);
                }
            }
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/core");
    let mut violations = Vec::new();
    visit(&root, &root, &mut violations);
    assert!(
        violations.is_empty(),
        "Keep domain modules independent of UI/runtime adapters: {violations:?}"
    );
}
