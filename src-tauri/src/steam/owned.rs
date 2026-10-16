//! Persisted cache of the user's **entire owned Steam library**.
//!
//! Installing only games that are on disk would miss most of a large library,
//! so the Steam Web API's `IPlayerService/GetOwnedGames` result is cached here
//! and merged into the Library view. Owned-but-not-installed games are
//! first-class citizens: they can be categorized and placed into collections
//! even though they are not currently installed.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

/// One owned game, normalized from `GetOwnedGames`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnedGameRecord {
    pub app_id: u32,
    pub name: String,
    pub playtime_minutes: u64,
    pub playtime_two_weeks: u64,
    pub icon_url: Option<String>,
}

/// The cached owned library and when it was last refreshed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnedLibrary {
    pub fetched_at: Option<i64>,
    pub steam_id64: Option<String>,
    pub games: Vec<OwnedGameRecord>,
}

/// `…/universal-game-launcher/owned-games.json` under the OS config directory.
pub fn path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let new_path = base.join("universal-game-launcher").join("owned-games.json");
    if new_path.exists() {
        return new_path;
    }
    let legacy_path = base.join("steam-vault").join("owned-games.json");
    if legacy_path.exists() {
        return legacy_path;
    }
    new_path
}

/// Load the cached owned library, or an empty one when absent.
pub fn load() -> OwnedLibrary {
    std::fs::read_to_string(path())
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Persist the owned library to disk.
pub fn save(library: &OwnedLibrary) -> AppResult<()> {
    let path = path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string(&library)?)?;
    Ok(())
}

/// Build the public icon URL Steam exposes for an app's icon hash.
pub fn icon_url(app_id: u32, hash: &str) -> String {
    format!("https://media.steampowered.com/steamcommunity/public/images/apps/{app_id}/{hash}.jpg")
}
