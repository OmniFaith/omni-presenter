//! Portable, versioned workspace archives. Files are streamed, never extracted by archive paths.
use super::db::{
    AppDb,
    transfer::{Cell, Table},
};
use crate::core::preferences::AppSettings;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

const MANIFEST_LIMIT: u64 = 128 * 1024 * 1024;
const TOTAL_LIMIT: u64 = 64 * 1024 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Package {
    pub format: String,
    pub version: u32,
    pub application_version: String,
    pub tables: Vec<Table>,
    /// Original path -> generated ZIP entry (never an extraction destination).
    pub assets: BTreeMap<String, String>,
    pub missing: Vec<String>,
}

fn asset_key(key: &str) -> bool {
    matches!(
        key,
        "path" | "file_path" | "bg_image_path" | "recent_wallpapers"
    )
}

fn walk(value: &mut Value, key: &str, visit: &mut impl FnMut(&mut String)) {
    match value {
        Value::String(s) if asset_key(key) => visit(s),
        Value::Array(items) => {
            for v in items {
                walk(v, key, visit);
            }
        }
        Value::Object(map) => {
            // Media playlist/quick-bar entries carry their path in content/target.
            let media = map.get("kind").and_then(Value::as_str) == Some("Media");
            for (k, v) in map {
                walk(
                    v,
                    if media && matches!(k.as_str(), "content" | "target") {
                        "path"
                    } else {
                        k
                    },
                    visit,
                );
            }
        }
        _ => {}
    }
}

fn visit_paths(tables: &mut [Table], mut visit: impl FnMut(&mut String)) -> Result<(), String> {
    for table in tables {
        for row in &mut table.rows {
            for (column, cell) in table.columns.iter().zip(row) {
                if let Cell::Text(text) = cell {
                    if asset_key(column) {
                        visit(text);
                    } else if matches!(column.as_str(), "value" | "options") {
                        let mut value: Value =
                            serde_json::from_str(text).map_err(|e| e.to_string())?;
                        walk(&mut value, "", &mut visit);
                        *text = serde_json::to_string(&value).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// The supplied settings snapshot captures unsaved profile state and the Settings draft.
pub fn export(db: &AppDb, settings: &AppSettings, destination: &Path) -> Result<Package, String> {
    let mut tables = db.transfer_snapshot()?;
    let settings_table = tables
        .iter_mut()
        .find(|t| t.name == "app_settings")
        .ok_or("transfer-invalid")?;
    settings_table.rows = vec![vec![
        Cell::Integer(1),
        Cell::Text(serde_json::to_string(settings).map_err(err)?),
    ]];
    let mut package = Package {
        format: "omni-presenter-workspace".into(),
        version: 1,
        application_version: env!("CARGO_PKG_VERSION").into(),
        tables,
        assets: BTreeMap::new(),
        missing: Vec::new(),
    };
    visit_paths(&mut package.tables, |s| {
        if s.is_empty() || s.starts_with("http://") || s.starts_with("https://") {
            return;
        }
        if Path::new(s).is_file() {
            if !package.assets.contains_key(s) {
                let extension = Path::new(s)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("bin");
                let extension: String = extension
                    .chars()
                    .filter(char::is_ascii_alphanumeric)
                    .take(12)
                    .collect();
                package.assets.insert(
                    s.clone(),
                    format!("assets/{}.{}", package.assets.len(), extension),
                );
            }
        } else if !package.missing.contains(s) {
            package.missing.push(s.clone());
        }
    })?;
    // Write alongside the destination, then atomically replace only after success.
    let temporary = destination.with_file_name(crate::core::preferences::new_id("omni_export_tmp"));
    let result = (|| {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(err)?;
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        let manifest = serde_json::to_vec(&package).map_err(err)?;
        if manifest.len() as u64 > MANIFEST_LIMIT {
            return Err("transfer-invalid".into());
        }
        zip.start_file("manifest.json", options).map_err(err)?;
        zip.write_all(&manifest).map_err(err)?;
        let mut total = manifest.len() as u64;
        for (source, entry) in &package.assets {
            zip.start_file(
                entry,
                options.compression_method(zip::CompressionMethod::Stored),
            )
            .map_err(err)?;
            let file = File::open(source).map_err(err)?;
            total +=
                std::io::copy(&mut file.take(TOTAL_LIMIT - total + 1), &mut zip).map_err(err)?;
            if total > TOTAL_LIMIT {
                return Err("transfer-invalid".into());
            }
        }
        zip.finish().map_err(err)?.sync_all().map_err(err)?;
        std::fs::rename(&temporary, destination).map_err(err)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result?;
    Ok(package)
}

pub fn inspect(path: &Path, db: &AppDb) -> Result<Package, String> {
    let mut archive = ZipArchive::new(File::open(path).map_err(err)?).map_err(err)?;
    if archive.len() > 100_000 {
        return Err("transfer-invalid".into());
    }
    let mut total = 0_u64;
    let mut names = std::collections::HashSet::new();
    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(err)?;
        total = total.checked_add(entry.size()).ok_or("transfer-invalid")?;
        // zip 9 decodes names fallibly; propagate failures before checking for
        // duplicate names and matching entries against the manifest's assets.
        let name = entry.name().map_err(err)?.into_owned();
        if total > TOTAL_LIMIT || !names.insert(name) {
            return Err("transfer-invalid".into());
        }
    }
    let manifest = archive.by_name("manifest.json").map_err(err)?;
    if manifest.size() > MANIFEST_LIMIT {
        return Err("transfer-invalid".into());
    }
    let mut bytes = Vec::new();
    manifest
        .take(MANIFEST_LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(err)?;
    if bytes.len() as u64 > MANIFEST_LIMIT {
        return Err("transfer-invalid".into());
    }
    let package: Package = serde_json::from_slice(&bytes).map_err(err)?;
    if package.format != "omni-presenter-workspace" || package.version != 1 {
        return Err("transfer-invalid".into());
    }
    for entry in package.assets.values() {
        if !entry.starts_with("assets/")
            || entry.contains("..")
            || entry.contains('\\')
            || !names.contains(entry)
        {
            return Err("transfer-invalid".into());
        }
    }
    db.validate_transfer(&package.tables)?;
    Ok(package)
}

/// Restore runs only after confirmation and after the app has suspended writes/servers.
/// A recovery archive is mandatory. Files live in a new owned directory, never source paths.
pub fn restore(
    db: &AppDb,
    settings: &AppSettings,
    source: &Path,
    recovery: &Path,
    assets_root: &Path,
) -> Result<PathBuf, String> {
    let mut package = inspect(source, db)?;
    export(db, settings, recovery)?;
    std::fs::create_dir_all(assets_root).map_err(err)?;
    let directory = assets_root.join(crate::core::preferences::new_id("restored"));
    std::fs::create_dir(&directory).map_err(err)?;
    let result = (|| {
        let mut archive = ZipArchive::new(File::open(source).map_err(err)?).map_err(err)?;
        let mut replacements = BTreeMap::new();
        let mut total = 0_u64;
        for (original, name) in &package.assets {
            let mut entry = archive.by_name(name).map_err(err)?;
            let extension = Path::new(name)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("bin");
            let extension: String = extension
                .chars()
                .filter(char::is_ascii_alphanumeric)
                .take(12)
                .collect();
            let target = directory.join(format!("{}.{}", replacements.len(), extension));
            let mut output = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&target)
                .map_err(err)?;
            let copied = std::io::copy(
                &mut entry.by_ref().take(TOTAL_LIMIT - total + 1),
                &mut output,
            )
            .map_err(err)?;
            total += copied;
            if total > TOTAL_LIMIT || copied != entry.size() {
                return Err("transfer-invalid".into());
            }
            replacements.insert(original.clone(), target.to_string_lossy().into_owned());
        }
        visit_paths(&mut package.tables, |s| {
            if let Some(path) = replacements.get(s) {
                *s = path.clone();
            }
        })?;
        db.restore_transfer(&package.tables)
    })();
    if result.is_err() {
        // Only the fresh, uniquely generated directory owned by this operation is removed.
        let _ = std::fs::remove_dir_all(&directory);
    }
    result?;
    Ok(recovery.to_path_buf())
}

#[cfg(test)]
#[path = "transfer_tests.rs"]
mod tests;
