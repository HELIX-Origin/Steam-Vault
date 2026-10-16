//! Drop-in Plugin Manifest schema.
//!
//! Every plugin is self-contained in its own folder and defines its capabilities
//! via a `plugin.json` (or `manifest.json`) file without requiring any hardcoded
//! changes to the core application.

use serde::{Deserialize, Serialize};

/// Canonical plugin category matching GOG Galaxy's modular plugin types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PluginType {
    /// Discovers and launches games for a gaming platform or storefront.
    Platform,
    /// Enriches games with descriptions, tags, genres, and store CDN artwork.
    Metadata,
    /// Enhances the UI with video previews, trailers, or ambient audio.
    Media,
    /// Background utility, manager tool, or drive analysis integration.
    Utility,
}

impl Default for PluginType {
    fn default() -> Self {
        PluginType::Utility
    }
}

/// Optional platform integration configuration declared by platform plugins.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginPlatformConfig {
    /// Canonical platform identifier (e.g. "steam", "gog", "epic", "itch", "xbox", etc.)
    pub platform_id: String,
    /// URI scheme patterns for launching games (e.g. "steam://rungameid/{appId}")
    #[serde(default)]
    pub launch_protocols: Vec<String>,
    /// Known directories searched for games or library folders
    #[serde(default)]
    pub known_folders: Vec<String>,
    /// Main executable name candidates
    #[serde(default)]
    pub executable_names: Vec<String>,
}

/// Store CDN artwork template URLs declared by metadata plugins.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginArtworkConfig {
    /// Cover capsule artwork URL template (supports `{appId}`, `{name}`, etc.)
    pub capsule_url: Option<String>,
    /// Hero banner artwork URL template
    pub hero_url: Option<String>,
    /// Transparent logo artwork URL template
    pub logo_url: Option<String>,
}

/// Optional metadata and catalog configuration declared by metadata plugins.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginMetadataConfig {
    /// Query endpoint URL
    pub api_url: Option<String>,
    /// CDN artwork templates
    #[serde(default)]
    pub artwork: Option<PluginArtworkConfig>,
}

/// Schema for `plugin.json` contained inside each self-contained plugin directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default, rename = "type")]
    pub plugin_type: PluginType,
    pub author: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub homepage: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
    /// Optional executable or script entrypoint for dynamic execution (e.g. `main.js`, `main.py`, `run.exe`)
    pub entrypoint: Option<String>,
    /// Optional platform discovery & launch configuration
    pub platform: Option<PluginPlatformConfig>,
    /// Optional metadata provider configuration
    pub metadata: Option<PluginMetadataConfig>,
}

/// Resolved plugin summary passed to the Svelte UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub plugin_type: PluginType,
    pub enabled: bool,
    pub builtin: bool,
    pub icon: Option<String>,
    pub homepage: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
    pub path: String,
}

/// Store-provided artwork URLs resolved from official store CDNs or catalog endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreArtwork {
    pub capsule_image: Option<String>,
    pub hero_image: Option<String>,
    pub logo_image: Option<String>,
}
