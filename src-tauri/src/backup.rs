//! Timestamped backups of Steam files before any write.
//!
//! Every destructive operation copies the original into a per-user backup
//! directory (`<app data>/steam-vault/backups/<timestamp>/…`) so a bad edit can
//! always be reverted manually.

use std::path::{Path, PathBuf};

use crate::error::AppResult;

/// Root directory where backups are accumulated.
pub fn backup_root() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("steam-vault")
        .join("backups")
}

/// Copy `path` into a timestamped backup folder. Returns the destination path.
/// A missing source is a no-op that returns the intended destination.
pub fn backup_file(path: &Path) -> AppResult<PathBuf> {
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S-%3f").to_string();
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let destination = backup_root().join(timestamp).join(file_name);

    if !path.is_file() {
        return Ok(destination);
    }

    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(path, &destination)?;
    Ok(destination)
}
