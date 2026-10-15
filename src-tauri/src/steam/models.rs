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
    pub release_date: Option<String>,
    pub release_year: Option<i32>,
    pub short_description: Option<String>,
    pub header_image: Option<String>,
    pub capsule_image: Option<String>,
    pub user_score: Option<f64>,
    pub fetched_at: Option<i64>,
}
