//! Local application settings and secrets.
//!
//! Settings live in the OS config directory (`<config>/steam-vault/settings.json`).
//! API keys are stored here on the user's own machine only and are **never**
//! committed. The file is created with best-effort restrictive permissions.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

/// Persisted application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub steam_root: Option<String>,
    pub account_id3: Option<String>,
    pub steam_api_key: Option<String>,
    pub steamgriddb_api_key: Option<String>,
    pub igdb_client_id: Option<String>,
    pub igdb_client_secret: Option<String>,
    pub country_code: String,
    pub language: String,
    #[serde(default = "default_true")]
    pub auto_backup: bool,
    /// Enable Steam Sync & Auto-Categorizer (Depressurizer Mode).
    /// When disabled, Steam-Vault functions purely in Standalone Universal Library Mode.
    #[serde(default = "default_true")]
    pub enable_steam_sync: bool,
    #[serde(default)]
    pub auto_cats: Vec<crate::autocat::AutoCat>,
    /// Non-Steam source detection preferences (enable flags + extra folders).
    #[serde(default)]
    pub platform_sources: Vec<crate::platforms::PlatformSource>,
    /// Explicitly configured emulators and their ROM directories.
    #[serde(default)]
    pub emulator_configs: Vec<crate::platforms::EmulatorConfig>,
    /// Optional user-configured path to Legendary CLI binary or directory.
    #[serde(default)]
    pub legendary_path: Option<String>,
    /// Optional free game promotion notifications for enabled storefront platforms.
    #[serde(default)]
    pub enable_free_game_notifications: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            steam_root: None,
            account_id3: None,
            steam_api_key: None,
            steamgriddb_api_key: None,
            igdb_client_id: None,
            igdb_client_secret: None,
            country_code: "us".into(),
            language: "en".into(),
            auto_backup: true,
            enable_steam_sync: true,
            auto_cats: Vec::new(),
            platform_sources: Vec::new(),
            emulator_configs: Vec::new(),
            legendary_path: None,
            enable_free_game_notifications: false,
        }
    }
}

/// `…/steam-vault/settings.json` under the OS config directory.
pub fn settings_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("steam-vault")
        .join("settings.json")
}

/// Load settings, falling back to defaults when the file is absent or invalid.
pub fn load() -> Settings {
    let path = settings_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Persist settings to disk.
pub fn save(settings: &Settings) -> AppResult<()> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(settings)?;
    std::fs::write(&path, text)?;
    Ok(())
}
