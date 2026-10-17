//! Native Plugin Engine.
//!
//! Manages native plugin enabled states and generic platform directory scanning
//! without any external JSON fallbacks, disk manifest extraction, or reverse-compatibility clutter.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::platforms::{PlatformGame, PlatformScan, ScanConfig};
use super::manifest::{PluginInfo, PluginType};

/// The dedicated plugin engine managing plugin execution and state.
#[derive(Debug, Clone, Default)]
pub struct PluginEngine {
    pub enabled: HashMap<String, bool>,
}

impl PluginEngine {
    /// Initializes the plugin engine.
    pub fn init() -> Self {
        Self {
            enabled: HashMap::new(),
        }
    }

    /// Reloads in-memory engine state.
    pub fn reload(&mut self) {}

    /// Returns list of active plugin states for the UI.
    pub fn list_plugins(&self, enabled_map: &HashMap<String, bool>) -> Vec<PluginInfo> {
        enabled_map
            .iter()
            .map(|(id, enabled)| PluginInfo {
                id: id.clone(),
                name: id.clone(),
                version: "1.0.0".to_string(),
                author: Some("HELIX Origin".to_string()),
                description: None,
                plugin_type: PluginType::Platform,
                enabled: *enabled,
                builtin: true,
                icon: None,
                homepage: None,
                features: vec![],
                path: format!("native://src/lib/plugins/{id}"),
                settings: None,
            })
            .collect()
    }

    /// Whether a specific plugin is currently enabled.
    pub fn is_enabled(&self, id: &str, enabled_map: &HashMap<String, bool>) -> bool {
        enabled_map.get(id).copied().unwrap_or(true)
    }

    /// Scans installed games across platforms using configured platform roots.
    pub fn scan_platforms(&self, config: &ScanConfig, enabled_map: &HashMap<String, bool>) -> PlatformScan {
        let mut all_games = Vec::new();
        let mut all_statuses = Vec::new();

        for source in &config.sources {
            if !source.enabled || !self.is_enabled(&source.id, enabled_map) {
                continue;
            }

            let detection = scan_platform_roots(
                &source.id,
                &source.roots,
                &[],
                config.include_other_drives,
            );

            let count = detection.games.len();
            all_games.extend(detection.games);
            all_statuses.push(crate::platforms::PlatformStatus {
                platform: crate::platforms::Platform::from_id(&source.id).unwrap_or(crate::platforms::Platform::Other),
                label: source.id.clone(),
                available: !detection.roots.is_empty(),
                count,
                roots: detection.roots,
                notes: detection.note,
            });
        }

        PlatformScan {
            games: all_games,
            platforms: all_statuses,
        }
    }
}

/// The outcome of one platform scan run.
#[derive(Debug, Clone, Default)]
pub struct Detection {
    pub games: Vec<PlatformGame>,
    pub roots: Vec<String>,
    pub note: Option<String>,
}

/// Normalizes a filesystem path: resolves relative components and standardizes separators.
pub fn normalize_path(p: &str) -> String {
    p.trim().trim_end_matches(['/', '\\']).to_string()
}

/// Check if a drive or volume represents a cloud sync virtual mount (Google Drive, OneDrive, etc.).
pub fn is_cloud_disk(name: &str, mount: &str) -> bool {
    let n = name.to_lowercase();
    let m = mount.to_lowercase();
    n.contains("google drive")
        || n.contains("onedrive")
        || n.contains("dropbox")
        || n.contains("icloud")
        || m.contains("google drive")
        || m.contains("onedrive")
        || m.contains("dropbox")
        || m.contains("icloud")
}

/// Discovers mounted physical disk and partition roots on the host machine.
pub fn drive_roots() -> Vec<PathBuf> {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let mut out: Vec<PathBuf> = disks
        .iter()
        .filter(|d| !is_cloud_disk(&d.name().to_string_lossy(), &d.mount_point().to_string_lossy()))
        .map(|d| d.mount_point().to_path_buf())
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Check if an executable filename represents an uninstaller, crash handler, or setup helper.
fn is_ignored_executable(file_name: &str) -> bool {
    let lower = file_name.to_lowercase();
    lower.starts_with("unins")
        || lower.starts_with("uninstall")
        || lower.starts_with("crashpad")
        || lower.starts_with("unitycrashhandler")
        || lower.starts_with("crashreport")
        || lower.starts_with("setup")
        || lower.starts_with("update")
        || lower.starts_with("vcredist")
        || lower.starts_with("dxsetup")
        || lower == "gamelauncher.exe"
}

/// Scans a single directory representing an installed game to find its primary executable and size.
fn inspect_game_folder(
    folder: &Path,
    executable_names: &[String],
) -> Option<(PathBuf, u64)> {
    let mut best_exe: Option<PathBuf> = None;
    let mut largest_exe_size = 0u64;
    let mut total_size = 0u64;

    let entries = fs::read_dir(folder).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let meta = entry.metadata().ok();
            let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
            total_size += size;

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                #[cfg(windows)]
                let is_exec = name.to_lowercase().ends_with(".exe");
                #[cfg(not(windows))]
                let is_exec = !name.contains('.') || name.ends_with(".bin") || name.ends_with(".app");

                if is_exec && !is_ignored_executable(name) {
                    let lower_name = name.to_lowercase();
                    let is_explicit = executable_names.iter().any(|e| e.to_lowercase() == lower_name);

                    if is_explicit {
                        best_exe = Some(path.clone());
                        largest_exe_size = u64::MAX;
                    } else if size > largest_exe_size {
                        best_exe = Some(path.clone());
                        largest_exe_size = size;
                    }
                }
            }
        } else if path.is_dir() {
            let folder_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
            if ["bin", "binaries", "game", "x64", "win64"].contains(&folder_name.as_str()) {
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub in sub_entries.flatten() {
                        let sub_path = sub.path();
                        if sub_path.is_file() {
                            let size = sub.metadata().map(|m| m.len()).unwrap_or(0);
                            total_size += size;
                            if let Some(sub_name) = sub_path.file_name().and_then(|n| n.to_str()) {
                                #[cfg(windows)]
                                let is_exec = sub_name.to_lowercase().ends_with(".exe");
                                #[cfg(not(windows))]
                                let is_exec = !sub_name.contains('.');

                                if is_exec && !is_ignored_executable(sub_name) && size > largest_exe_size && largest_exe_size != u64::MAX {
                                    best_exe = Some(sub_path);
                                    largest_exe_size = size;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    best_exe.map(|exe| (exe, total_size))
}

/// Dynamically scans directories configured for a platform plugin.
pub fn scan_platform_roots(
    platform_id: &str,
    roots: &[String],
    executable_names: &[String],
    include_other_drives: bool,
) -> Detection {
    let mut verified_roots = Vec::new();
    let mut games = Vec::new();

    let mut all_candidate_roots: Vec<PathBuf> = roots.iter().map(PathBuf::from).collect();
    if include_other_drives {
        let mounted = drive_roots();
        for root in roots {
            let p = Path::new(root);
            if let Ok(relative) = p.strip_prefix(p.components().next().map(|c| c.as_os_str()).unwrap_or_default()) {
                for drive in &mounted {
                    let alt = drive.join(relative);
                    if alt.is_dir() && !all_candidate_roots.contains(&alt) {
                        all_candidate_roots.push(alt);
                    }
                }
            }
        }
    }

    for root in all_candidate_roots {
        if !root.is_dir() {
            continue;
        }

        let root_str = normalize_path(&root.display().to_string());
        if !verified_roots.contains(&root_str) {
            verified_roots.push(root_str);
        }

        if let Ok(entries) = fs::read_dir(&root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let folder_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();

                    if folder_name.starts_with('.')
                        || folder_name.starts_with('$')
                        || folder_name.eq_ignore_ascii_case("temp")
                        || folder_name.eq_ignore_ascii_case("download")
                        || folder_name.eq_ignore_ascii_case("cache")
                    {
                        continue;
                    }

                    if let Some((exe_path, size)) = inspect_game_folder(&path, executable_names) {
                        let game_title = folder_name;
                        games.push(PlatformGame {
                            name: game_title,
                            platform: crate::platforms::Platform::from_id(platform_id).unwrap_or(crate::platforms::Platform::Other),
                            platform_id: Some(format!("{}:{}", platform_id, path.display())),
                            install_dir: Some(path.display().to_string()),
                            exe: Some(exe_path.display().to_string()),
                            size_on_disk: Some(size),
                            launch_uri: None,
                            source: Some(format!("Dynamic {} Plugin Scanner", platform_id)),
                            console: None,
                            installed: true,
                            install_uri: None,
                            uninstall_uri: None,
                        });
                    }
                }
            }
        }
    }

    Detection {
        games,
        roots: verified_roots,
        note: None,
    }
}
