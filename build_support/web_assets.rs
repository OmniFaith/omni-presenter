use std::{ffi::OsStr, io, path::Path, process::Command};

pub fn compile_web_assets(manifest: &Path, bun: &OsStr) -> io::Result<()> {
    let status = Command::new(bun)
        .current_dir(manifest.join("web"))
        .args(["run", "build"])
        .status()
        .map_err(|error| {
            io::Error::new(error.kind(), format!(
                "Could not start the web build with {bun:?}: {error}. Install Bun and run `bun install --frozen-lockfile` in web/."
            ))
        })?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "Web asset compilation failed ({status}). Refusing to use potentially stale assets/web files."
        )));
    }
    Ok(())
}
