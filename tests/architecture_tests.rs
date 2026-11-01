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
