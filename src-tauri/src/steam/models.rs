//! Domain models shared between the Rust backend and the Svelte frontend.
//!
//! All structs serialize to `camelCase` JSON so the TypeScript layer can consume
//! them without a mapping step.

use serde::{Deserialize, Serialize};

/// A discovered Steam installation: its root, library folders, and accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamInstall {
    pub root: String,
    pub libraries: Vec<SteamLibrary>,
    pub accounts: Vec<SteamAccount>,
}

/// A single Steam library folder registered in `libraryfolders.vdf`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamLibrary {
    pub index: u32,
    pub path: String,
    pub label: Option<String>,
    #[serde(default)]
    pub app_ids: Vec<u32>,
}

/// A Steam account present in `userdata/` and/or `loginusers.vdf`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamAccount {
    pub id3: String,
    pub id64: Option<String>,
    pub account_name: Option<String>,
    pub persona_name: Option<String>,
}

/// Whether a library entry is a real Steam app or a non-Steam shortcut.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GameKind {
    #[default]
    Steam,
    Shortcut,
}

/// A single entry in the user's library (Steam game or non-Steam shortcut).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub app_id: u32,
    pub name: String,
    #[serde(default)]
    pub kind: GameKind,
    pub library_path: Option<String>,
    pub install_dir: Option<String>,
    pub size_on_disk: u64,
    pub installed: bool,
    pub last_played: Option<i64>,
    pub playtime_minutes: Option<u64>,
    /// Names of the collections this game belongs to (resolved for display).
    #[serde(default)]
    pub collections: Vec<String>,
    pub favorite: bool,
    pub hidden: bool,
    /// Source platform: Steam for real apps, the storefront for imported shortcuts.
    #[serde(default)]
    pub platform: crate::platforms::Platform,
    // Non-Steam shortcut specifics.
    pub shortcut_id: Option<u32>,
    pub exe: Option<String>,
    pub start_dir: Option<String>,
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub launch_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uninstall_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capsule_image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hero_image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub console: Option<String>,
}

/// A Steam collection (a.k.a. category), modern or legacy-derived.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub dynamic: bool,
    #[serde(default)]
    pub added: Vec<u32>,
    #[serde(default)]
    pub removed: Vec<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter_spec: Option<serde_json::Value>,
}

/// Normalized metadata for a game from any provider.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppMeta {
    #[serde(default, alias = "app_id")]
    pub app_id: u32,
    pub name: Option<String>,
    #[serde(default)]
    pub developers: Vec<String>,
    #[serde(default)]
    pub publishers: Vec<String>,
    #[serde(default)]
    pub genres: Vec<String>,
    /// Steam store feature flags, e.g. "Single-player", "Steam Cloud".
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, alias = "release_date")]
    pub release_date: Option<String>,
    #[serde(default, alias = "release_year")]
    pub release_year: Option<i32>,
    #[serde(default, alias = "short_description")]
    pub short_description: Option<String>,
    #[serde(default, alias = "header_image")]
    pub header_image: Option<String>,
    #[serde(default, alias = "capsule_image")]
    pub capsule_image: Option<String>,
    #[serde(default, alias = "hero_image")]
    pub hero_image: Option<String>,
    #[serde(default, alias = "logo_image")]
    pub logo_image: Option<String>,
    #[serde(default, alias = "icon_image")]
    pub icon_image: Option<String>,
    #[serde(default, alias = "user_score")]
    pub user_score: Option<f64>,
    #[serde(default, alias = "fetched_at")]
    pub fetched_at: Option<i64>,
    #[serde(default)]
    pub screenshots: Vec<String>,
    #[serde(default)]
    pub movies: Vec<String>,
    #[serde(default)]
    pub locales: std::collections::HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub providers: std::collections::HashMap<String, serde_json::Value>,
    #[serde(default, alias = "user_enhanced")]
    pub user_enhanced: Option<bool>,
    #[serde(default, alias = "enhanced_by")]
    pub enhanced_by: Vec<String>,
}

impl AppMeta {
    /// Enrich missing top-level imagery and metadata fields by falling back
    /// to provider-specific records (Steam, GOG, PCGamingWiki, IGDB, VNDB).
    pub fn enrich_from_providers(&mut self) {
        if let Some(steam) = self.providers.get("steam") {
            if self.capsule_image.is_none() {
                self.capsule_image = steam.get("capsuleImage").and_then(|v| v.as_str()).map(String::from);
            }
            if self.hero_image.is_none() {
                self.hero_image = steam.get("heroImage").and_then(|v| v.as_str()).map(String::from);
            }
            if self.logo_image.is_none() {
                self.logo_image = steam.get("logoImage").and_then(|v| v.as_str()).map(String::from);
            }
            if self.icon_image.is_none() {
                self.icon_image = steam.get("iconImage").or_else(|| steam.get("icon")).and_then(|v| v.as_str()).map(String::from);
            }
            if self.header_image.is_none() {
                self.header_image = steam.get("headerImage").and_then(|v| v.as_str()).map(String::from);
            }
            if self.short_description.is_none() {
                self.short_description = steam.get("shortDescription").and_then(|v| v.as_str()).map(String::from);
            }
        }
        if let Some(gog) = self.providers.get("gog") {
            if self.capsule_image.is_none() {
                self.capsule_image = gog.get("coverVertical").and_then(|v| v.as_str()).map(String::from);
            }
            if self.header_image.is_none() {
                self.header_image = gog.get("coverHorizontal").and_then(|v| v.as_str()).map(String::from);
            }
            if self.hero_image.is_none() {
                self.hero_image = gog.get("galaxyBackgroundImage").and_then(|v| v.as_str()).map(String::from);
            }
            if self.logo_image.is_none() {
                self.logo_image = gog.get("logo").and_then(|v| v.as_str()).map(String::from);
            }
            if self.icon_image.is_none() {
                self.icon_image = gog.get("icon").and_then(|v| v.as_str()).map(String::from);
            }
        }
        if let Some(pcgw) = self.providers.get("pcgamingwiki") {
            if self.header_image.is_none() {
                self.header_image = pcgw.get("coverImage").and_then(|v| v.as_str()).map(String::from);
            }
        }
        if let Some(igdb) = self.providers.get("igdb") {
            if self.capsule_image.is_none() {
                self.capsule_image = igdb.get("coverUrl").and_then(|v| v.as_str()).map(String::from);
            }
            if self.short_description.is_none() {
                self.short_description = igdb.get("summary").and_then(|v| v.as_str()).map(String::from);
            }
        }
    }
}

