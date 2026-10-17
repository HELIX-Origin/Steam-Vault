//! External game-platform detection (GOG, Epic, itch.io, and more).
//!
//! Steam Vault discovers games installed by other launchers so they can be
//! imported into Steam as non-Steam shortcuts and organized into collections.
//! Detection reads authoritative local sources — launcher manifests and the
//! Windows *Uninstall* registry — and never scrapes a storefront or launches a
//! launcher process.
//!
//! Discovered games are attributed to a [`Platform`], and after import the
//! appid → platform mapping is persisted so the Library can keep grouping
//! shortcuts by their source storefront.

pub mod emulation;
pub mod epic;
pub mod promotions;
pub mod tools;

#[allow(unused_imports)]
pub use emulation::{
    normalize_game_title, normalize_rom_title, scaffold_emulation_folders, scan_emulation_roms,
    ScaffoldReport, SystemFolderSpec, SYSTEM_FOLDER_SPECS,
};
pub use epic::{
    get_eos_overlay_status, get_legendary_auth_status, install_eos_overlay,
    install_legendary_cli, legendary_authenticate, legendary_logout, EosOverlayStatus,
    LegendaryAuthStatus,
};
pub use tools::{
    check_binary_installed, detect_all_launchers, install_winget_package,
    is_winget_available, managed_launchers_dir, managed_tools_dir, LauncherStatus,
    ManagedDirectories,
};
pub use promotions::{FeaturedStoreItem, FreeGamePromotion, PatchNote, StoreDeal};

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

/// Per-platform detection preferences (persisted in `settings.json`).
///
/// When no entry exists for a platform it is scanned with its default,
/// built-in sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformSource {
    /// Stable platform id (see [`Platform::id`]).
    pub id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Extra folders to scan for this platform (for example on an external
    /// drive that is not registered with the launcher's own manifest store).
    #[serde(default)]
    pub roots: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// Options controlling one [`scan_all`] pass.
#[derive(Debug, Clone, Default)]
pub struct ScanConfig {
    /// Per-platform enable flags and extra folders, from settings.
    pub sources: Vec<PlatformSource>,
    /// Also look on every mounted drive for well-known game folders.
    pub include_other_drives: bool,
}

impl ScanConfig {
    /// Whether a platform should be scanned at all (defaults to enabled).
    pub fn is_enabled(&self, platform: Platform) -> bool {
        self.sources
            .iter()
            .find(|source| source.id == platform.id())
            .map(|source| source.enabled)
            .unwrap_or(true)
    }

    /// Folders explicitly configured for a platform.
    pub fn roots_for(&self, platform: Platform) -> Vec<String> {
        self.sources
            .iter()
            .filter(|source| source.id == platform.id())
            .flat_map(|source| source.roots.iter().cloned())
            .collect()
    }
}

/// A storefront / launcher Universal Game Launcher can detect games from.
///
/// Serialized with the same stable ids surfaced by [`Platform::id`]
/// (`steam`, `gog`, `epic`, `battlenet`, …) so the frontend can index it
/// directly.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    #[default]
    Steam,
    Gog,
    Epic,
    Itch,
    Ubisoft,
    Ea,
    BattleNet,
    Amazon,
    Xbox,
    Rockstar,
    EmulationStation,
    Standalone,
    Other,
}

impl Platform {
    /// Stable machine id (used in persisted maps and the frontend).
    pub fn id(self) -> &'static str {
        match self {
            Platform::Steam => "steam",
            Platform::Gog => "gog",
            Platform::Epic => "epic",
            Platform::Itch => "itch",
            Platform::Ubisoft => "ubisoft",
            Platform::Ea => "ea",
            Platform::BattleNet => "battlenet",
            Platform::Amazon => "amazon",
            Platform::Xbox => "xbox",
            Platform::Rockstar => "rockstar",
            Platform::EmulationStation => "emulationstation",
            Platform::Standalone => "standalone",
            Platform::Other => "other",
        }
    }

    /// Human-readable label for the UI.
    pub fn label(self) -> &'static str {
        match self {
            Platform::Steam => "Steam",
            Platform::Gog => "GOG",
            Platform::Epic => "Epic Games",
            Platform::Itch => "itch.io",
            Platform::Ubisoft => "Ubisoft Connect",
            Platform::Ea => "EA",
            Platform::BattleNet => "Battle.net",
            Platform::Amazon => "Amazon Games",
            Platform::Xbox => "Xbox / Microsoft Store",
            Platform::Rockstar => "Rockstar",
            Platform::EmulationStation => "EmulationStation",
            Platform::Standalone => "Standalone & Local",
            Platform::Other => "Other",
        }
    }

    pub fn from_id(id: &str) -> Option<Platform> {
        Some(match id {
            "steam" => Platform::Steam,
            "gog" => Platform::Gog,
            "epic" => Platform::Epic,
            "itch" => Platform::Itch,
            "ubisoft" => Platform::Ubisoft,
            "ea" => Platform::Ea,
            "battlenet" => Platform::BattleNet,
            "amazon" => Platform::Amazon,
            "xbox" => Platform::Xbox,
            "rockstar" => Platform::Rockstar,
            "emulationstation" => Platform::EmulationStation,
            "standalone" => Platform::Standalone,
            "other" => Platform::Other,
            _ => return None,
        })
    }
}

/// A game discovered via a non-Steam platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformGame {
    pub name: String,
    pub platform: Platform,
    /// The platform's own id for the game (e.g. an Epic `AppName`, GOG game id).
    pub platform_id: Option<String>,
    pub install_dir: Option<String>,
    /// Launch executable, absolute where known.
    pub exe: Option<String>,
    pub size_on_disk: Option<u64>,
    /// A storefront launch URI where available.
    pub launch_uri: Option<String>,
    /// Which local source produced this entry (for transparency in the UI).
    pub source: Option<String>,
    /// Specific console platform for emulated games (GameCube, Wii, PlayStation, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub console: Option<String>,
    #[serde(default = "default_true")]
    pub installed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uninstall_uri: Option<String>,
}

impl Default for PlatformGame {
    fn default() -> Self {
        Self {
            name: String::new(),
            platform: Platform::Steam,
            platform_id: None,
            install_dir: None,
            exe: None,
            size_on_disk: None,
            launch_uri: None,
            source: None,
            console: None,
            installed: true,
            install_uri: None,
            uninstall_uri: None,
        }
    }
}

/// Per-platform scan summary, so the UI can explain what was (not) found.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformStatus {
    pub platform: Platform,
    pub label: String,
    /// Whether the platform's local data source exists on this machine.
    pub available: bool,
    pub count: usize,
    /// Directories / registry paths that were consulted.
    pub roots: Vec<String>,
    pub notes: Option<String>,
}

/// Result of a full platform scan.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformScan {
    pub games: Vec<PlatformGame>,
    pub platforms: Vec<PlatformStatus>,
}

// ---------------------------------------------------------------------------
// appid → platform map
//
// Non-Steam shortcuts carry no platform information on their own, so after an
// import we remember which storefront each appid came from. This keeps the
// Library grouped by source and powers the "Platform" categorization option.
// ---------------------------------------------------------------------------

/// `…/universal-game-launcher/platform-map.json` under the OS config directory.
pub fn map_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let new_path = base.join("universal-game-launcher").join("platform-map.json");
    if new_path.exists() {
        return new_path;
    }
    let legacy_path = base.join("steam-vault").join("platform-map.json");
    if legacy_path.exists() {
        return legacy_path;
    }
    new_path
}

/// Load the persisted appid → platform map.
pub fn load_map() -> BTreeMap<u32, Platform> {
    std::fs::read_to_string(map_path())
        .ok()
        .and_then(|text| serde_json::from_str::<BTreeMap<String, String>>(&text).ok())
        .map(|map| {
            map.into_iter()
                .filter_map(|(key, value)| key.parse::<u32>().ok().zip(Platform::from_id(&value)))
                .collect()
        })
        .unwrap_or_default()
}

/// Set or clear the platform mapping for a single game.
///
/// When `platform` is `Some`, sets or overrides the attribution. When `None`,
/// removes any existing mapping for this game.
pub fn set_override(app_id: u32, platform: Option<Platform>) -> AppResult<()> {
    let mut map = load_map();
    match platform {
        Some(p) => {
            map.insert(app_id, p);
        }
        None => {
            map.remove(&app_id);
        }
    }
    save_map(&map)
}

/// Merge new appid → platform pairs into the persisted map.
pub fn record(pairs: &[(u32, Platform)]) -> AppResult<()> {
    if pairs.is_empty() {
        return Ok(());
    }
    let mut map = load_map();
    for (app_id, platform) in pairs {
        map.insert(*app_id, *platform);
    }
    save_map(&map)
}

fn save_map(map: &BTreeMap<u32, Platform>) -> AppResult<()> {
    let serializable: BTreeMap<String, String> = map
        .iter()
        .map(|(app_id, platform)| (app_id.to_string(), platform.id().to_string()))
        .collect();
    let path = map_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(&serializable)?)?;
    Ok(())
}
