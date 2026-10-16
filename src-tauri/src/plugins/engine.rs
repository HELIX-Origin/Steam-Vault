//! Dedicated Community & First-Party Plugin Engine.
//!
//! Loads, discovers, and manages self-contained drop-in plugins dynamically from disk
//! without hardcoding plugin implementations into the core application.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::error::{AppError, AppResult};
use crate::platforms::{PlatformScan, ScanConfig};

use super::defaults::deploy_default_plugins;
use super::manifest::{PluginInfo, PluginManifest, PluginType, StoreArtwork};

/// A loaded, active plugin running inside the launcher.
#[derive(Debug, Clone)]
pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub path: PathBuf,
}

/// The dedicated plugin engine managing all installed plugins.
#[derive(Debug, Clone, Default)]
pub struct PluginEngine {
    pub plugins: Vec<LoadedPlugin>,
}

/// Returns the root plugins directory: `<data_dir>/universal-game-launcher/plugins/`.
pub fn plugins_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("universal-game-launcher").join("plugins");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
        let readme = dir.join("README.txt");
        if !readme.exists() {
            let _ = fs::write(
                &readme,
                "Universal Game Launcher — Community Plugins Directory\r\n\
                 \r\n\
                 Drop your community plugin folders here. Each plugin folder must contain\r\n\
                 a valid `plugin.json` manifest specifying its id, name, version, and type:\r\n\
                 - 'platform': Community game platform integration (e.g. Battle.net, Amazon, PlayStation)\r\n\
                 - 'metadata': Community metadata provider\r\n\
                 - 'media': Video trailer or audio media enhancement\r\n\
                 - 'utility': Community tool or launcher extension\r\n",
            );
        }
    }
    dir
}

impl PluginEngine {
    /// Initializes the engine, deploying default drop-in plugins if needed,
    /// and scanning for all installed plugins.
    pub fn init() -> Self {
        let dir = plugins_dir();
        // Unpack standard drop-in plugins into disk if missing
        deploy_default_plugins(&dir);

        let mut engine = PluginEngine {
            plugins: Vec::new(),
        };
        engine.reload();
        engine
    }

    /// Dynamically re-scans the plugins directory and loads all valid manifests.
    pub fn reload(&mut self) {
        let dir = plugins_dir();
        let mut loaded = Vec::new();

        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let index_path = path.join("index.json");
                let manifest_path = path.join("plugin.json");
                let alt_manifest = path.join("manifest.json");
                let target = if index_path.is_file() {
                    index_path
                } else if manifest_path.is_file() {
                    manifest_path
                } else if alt_manifest.is_file() {
                    alt_manifest
                } else {
                    continue;
                };

                if let Ok(content) = fs::read_to_string(&target) {
                    if let Ok(manifest) = serde_json::from_str::<PluginManifest>(&content) {
                        loaded.push(LoadedPlugin { manifest, path });
                    }
                }
            }
        }

        loaded.sort_by(|a, b| a.manifest.name.to_lowercase().cmp(&b.manifest.name.to_lowercase()));
        self.plugins = loaded;
    }

    /// Installs a community plugin package from a GitHub repository or hosted `index.json` entry point.
    pub async fn install_plugin_package(
        input_url: &str,
        client: &reqwest::Client,
    ) -> AppResult<String> {
        let (raw_index_url, base_raw_url) = resolve_github_plugin_urls(input_url)
            .ok_or_else(|| AppError::Invalid(format!("Invalid GitHub repository URL or entry point: '{input_url}'")))?;

        // Try downloading index.json; fallback to plugin.json if index.json is 404
        let resp = client.get(&raw_index_url).send().await?;
        let content = if resp.status().is_success() {
            resp.text().await?
        } else {
            // Try fallback to plugin.json
            let alt_url = raw_index_url.replace("/index.json", "/plugin.json");
            let alt_resp = client.get(&alt_url).send().await?;
            if !alt_resp.status().is_success() {
                // If main branch failed, try master branch
                let master_url = raw_index_url.replace("/main/index.json", "/master/index.json");
                let master_resp = client.get(&master_url).send().await?;
                if master_resp.status().is_success() {
                    master_resp.text().await?
                } else {
                    return Err(AppError::NotFound(format!(
                        "Could not find index.json or plugin.json at hosted entry point: {raw_index_url}"
                    )));
                }
            } else {
                alt_resp.text().await?
            }
        };

        let manifest: PluginManifest = serde_json::from_str(&content).map_err(|e| {
            AppError::Invalid(format!("Failed to parse plugin index.json manifest: {e}"))
        })?;

        let plugin_id = manifest.id.trim().to_string();
        if plugin_id.is_empty() {
            return Err(AppError::Invalid("Plugin manifest must contain a valid 'id'".into()));
        }

        let dir = plugins_dir();
        let plugin_dir = dir.join(&plugin_id);
        fs::create_dir_all(&plugin_dir)?;

        // Save index.json
        let target_manifest = plugin_dir.join("index.json");
        fs::write(target_manifest, &content)?;

        // If manifest declares an entrypoint script (e.g. index.js, main.py), attempt to download it
        if let Some(ref entry) = manifest.entrypoint {
            let entry_file = entry.trim_matches('/').trim_matches('\\');
            let entry_download_url = format!("{base_raw_url}/{entry_file}");
            if let Ok(entry_resp) = client.get(&entry_download_url).send().await {
                if entry_resp.status().is_success() {
                    if let Ok(bytes) = entry_resp.bytes().await {
                        let _ = fs::write(plugin_dir.join(entry_file), bytes);
                    }
                }
            }
        }

        Ok(plugin_id)
    }

    /// Lists all installed plugins with their enabled status for the frontend UI.
    pub fn list_plugins(&self, enabled_map: &HashMap<String, bool>) -> Vec<PluginInfo> {
        self.plugins
            .iter()
            .map(|lp| {
                let m = &lp.manifest;
                let enabled = enabled_map.get(&m.id).copied().unwrap_or(true);
                let is_builtin = m.author.as_deref() == Some("HELIX-Origin");
                PluginInfo {
                    id: m.id.clone(),
                    name: m.name.clone(),
                    version: m.version.clone(),
                    author: m.author.clone(),
                    description: m.description.clone(),
                    plugin_type: m.plugin_type,
                    enabled,
                    builtin: is_builtin,
                    icon: m.icon.clone(),
                    homepage: m.homepage.clone(),
                    features: m.features.clone(),
                    path: lp.path.to_string_lossy().to_string(),
                }
            })
            .collect()
    }

    /// Check if a given plugin is enabled.
    pub fn is_enabled(&self, plugin_id: &str, enabled_map: &HashMap<String, bool>) -> bool {
        enabled_map.get(plugin_id).copied().unwrap_or(true)
    }

    /// Resolves store CDN artwork URLs for a game from loaded metadata and platform plugins.
    pub fn resolve_store_artwork(
        &self,
        platform_id: &str,
        app_id: u32,
        game_name: &str,
        enabled_map: &HashMap<String, bool>,
    ) -> Option<StoreArtwork> {
        let app_id_str = app_id.to_string();

        for lp in &self.plugins {
            if !self.is_enabled(&lp.manifest.id, enabled_map) {
                continue;
            }

            // Check if plugin matches platform or is a metadata provider for it
            let matches_platform = lp
                .manifest
                .platform
                .as_ref()
                .map_or(false, |p| p.platform_id.eq_ignore_ascii_case(platform_id));
            let is_metadata = lp.manifest.plugin_type == PluginType::Metadata;

            if !matches_platform && !is_metadata {
                continue;
            }

            // Check metadata artwork configuration
            if let Some(ref meta) = lp.manifest.metadata {
                if let Some(ref art) = meta.artwork {
                    let capsule = art.capsule_url.as_ref().map(|url| {
                        url.replace("{appId}", &app_id_str)
                            .replace("{name}", game_name)
                    });
                    let hero = art.hero_url.as_ref().map(|url| {
                        url.replace("{appId}", &app_id_str)
                            .replace("{name}", game_name)
                    });
                    let logo = art.logo_url.as_ref().map(|url| {
                        url.replace("{appId}", &app_id_str)
                            .replace("{name}", game_name)
                    });

                    if capsule.is_some() || hero.is_some() || logo.is_some() {
                        return Some(StoreArtwork {
                            capsule_image: capsule,
                            hero_image: hero,
                            logo_image: logo,
                        });
                    }
                }
            }
        }

        None
    }

    /// Resolves a launch URI pattern for a game using loaded platform plugins.
    pub fn resolve_launch_uri(
        &self,
        platform_id: &str,
        app_id: u32,
        app_name: &str,
        enabled_map: &HashMap<String, bool>,
    ) -> Option<String> {
        let app_id_str = app_id.to_string();

        for lp in &self.plugins {
            if !self.is_enabled(&lp.manifest.id, enabled_map) {
                continue;
            }

            if let Some(ref plat) = lp.manifest.platform {
                if plat.platform_id.eq_ignore_ascii_case(platform_id) {
                    if let Some(proto) = plat.launch_protocols.first() {
                        return Some(
                            proto
                                .replace("{appId}", &app_id_str)
                                .replace("{appName}", app_name),
                        );
                    }
                }
            }
        }

        None
    }

    /// Scans installed games across platforms using enabled platform plugins.
    pub fn scan_platforms(&self, config: &ScanConfig, enabled_map: &HashMap<String, bool>) -> PlatformScan {
        let mut scan = crate::platforms::scan_all(config);

        // Filter out platforms whose plugin is disabled
        scan.platforms.retain(|status| {
            let plugin_id = format!("platform-{}", status.platform.id());
            self.is_enabled(&plugin_id, enabled_map)
        });

        scan
    }

    /// Dynamic entrypoint execution for plugins that provide a custom script or binary.
    pub fn execute_entrypoint(
        &self,
        plugin_id: &str,
        action: &str,
        payload_json: &str,
    ) -> AppResult<String> {
        let plugin = self
            .plugins
            .iter()
            .find(|p| p.manifest.id == plugin_id)
            .ok_or_else(|| AppError::NotFound(format!("Plugin '{plugin_id}' not found")))?;

        let entry = plugin
            .manifest
            .entrypoint
            .as_ref()
            .ok_or_else(|| AppError::Invalid(format!("Plugin '{plugin_id}' has no entrypoint")))?;

        let entry_path = plugin.path.join(entry);
        if !entry_path.exists() {
            return Err(AppError::NotFound(format!(
                "Entrypoint file '{}' not found in plugin '{}'",
                entry_path.display(),
                plugin_id
            )));
        }

        // Spawn entrypoint process with arguments [action, payload_json]
        let mut cmd = std::process::Command::new(&entry_path);
        cmd.current_dir(&plugin.path);
        cmd.arg(action);
        cmd.arg(payload_json);

        let output = cmd.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Invalid(format!(
                "Plugin '{}' error (code {:?}): {}",
                plugin_id,
                output.status.code(),
                stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Normalizes a GitHub repository URL or shorthand ("owner/repo") to (raw_index_url, base_raw_url).
pub fn resolve_github_plugin_urls(input: &str) -> Option<(String, String)> {
    let trimmed = input.trim();

    // 1. Direct raw URL to index.json or plugin.json
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        if trimmed.ends_with("/index.json") || trimmed.ends_with("/plugin.json") {
            let base = trimmed.rsplit_once('/')?.0.to_string();
            return Some((trimmed.to_string(), base));
        }
    }

    // 2. GitHub repository URL or shorthand
    let github_prefix = "https://github.com/";
    let clean = if let Some(rest) = trimmed.strip_prefix(github_prefix) {
        rest.trim_end_matches('/')
    } else if let Some(rest) = trimmed.strip_prefix("http://github.com/") {
        rest.trim_end_matches('/')
    } else if !trimmed.contains("://") && trimmed.contains('/') {
        trimmed.trim_end_matches('/')
    } else {
        return None;
    };

    let parts: Vec<&str> = clean.split('/').collect();
    if parts.len() >= 2 {
        let owner = parts[0];
        let repo = parts[1];
        let branch = if parts.len() >= 4 && parts[2] == "tree" {
            parts[3]
        } else {
            "main"
        };

        let base_raw_url = format!("https://raw.githubusercontent.com/{owner}/{repo}/{branch}");
        let raw_index_url = format!("{base_raw_url}/index.json");
        return Some((raw_index_url, base_raw_url));
    }

    None
}

