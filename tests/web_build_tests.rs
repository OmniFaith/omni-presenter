#[path = "../build_support/web_assets.rs"]
mod web_assets;

use std::{ffi::OsStr, path::PathBuf};

fn fixture(script: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("omni_web_build_{}_{unique}", std::process::id()));
    std::fs::create_dir_all(path.join("web")).unwrap();
    let package = serde_json::json!({"scripts": {"build": script}});
    std::fs::write(path.join("web/package.json"), package.to_string()).unwrap();
    path
}

#[test]
fn missing_bun_is_not_silently_ignored() {
    let path = fixture("bun --version");
    let missing = path.join("missing-bun");
    assert!(web_assets::compile_web_assets(&path, missing.as_os_str()).is_err());
    std::fs::remove_file(path.join("web/package.json")).unwrap();
    std::fs::remove_dir(path.join("web")).unwrap();
    std::fs::remove_dir(path).unwrap();
}

#[test]
fn failed_web_build_is_not_silently_ignored() {
    let path = fixture("bun -e 'process.exit(7)'");
    let bun = std::env::var_os("BUN").unwrap_or_else(|| "bun".into());
    assert!(web_assets::compile_web_assets(&path, &bun).is_err());
    std::fs::remove_file(path.join("web/package.json")).unwrap();
    std::fs::remove_dir(path.join("web")).unwrap();
    std::fs::remove_dir(path).unwrap();
}

#[test]
fn successful_web_build_is_accepted() {
    let path = fixture("bun --version");
    let bun = std::env::var_os("BUN").unwrap_or_else(|| OsStr::new("bun").to_owned());
    web_assets::compile_web_assets(&path, &bun).unwrap();
    std::fs::remove_file(path.join("web/package.json")).unwrap();
    std::fs::remove_dir(path.join("web")).unwrap();
    std::fs::remove_dir(path).unwrap();
}
