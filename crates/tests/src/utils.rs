use std::path::{Path, PathBuf};

// AI generated function, then modified
/// Returns the absolute path to the workspace root.
///
/// Starts at the current crate's manifest dir (known at compile time)
/// and walks upward looking for the outermost Cargo.toml that has a
/// `[workspace]` table. Falls back to CARGO_MANIFEST_DIR itself if
/// no workspace root is found (e.g. single-crate project).
pub fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    let mut current = manifest_dir;
    let mut found: Option<PathBuf> = None;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.is_file()
            && let Ok(contents) = std::fs::read_to_string(&cargo_toml)
            && contents.contains("[workspace]")
        {
            found = Some(current.to_path_buf());
            break;
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    found.unwrap_or_else(|| manifest_dir.to_path_buf())
}

/// Convenience: build a path relative to the workspace root.
pub fn workspace_path(relative: impl AsRef<Path>) -> PathBuf {
    workspace_root().join(relative)
}
