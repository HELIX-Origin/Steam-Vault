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

        let standard_platforms = [
            "gog", "epic", "itch", "ea", "ubisoft", "xbox", "battlenet", "amazon", "rockstar", "standalone", "other"
        ];

        let mut seen_platforms = std::collections::HashSet::new();

        // 1. Scan explicitly configured sources from settings
        for source in &config.sources {
            seen_platforms.insert(source.id.clone());
            if !source.enabled || !self.is_enabled(&source.id, enabled_map) {
                continue;
            }

            let detection = scan_platform_roots(
                &source.id,
                &source.roots,
                &[],
                true,
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

        // 2. Scan standard platforms that were not explicitly configured
        for plat_id in standard_platforms {
            if seen_platforms.contains(plat_id) {
                continue;
            }
            if !self.is_enabled(plat_id, enabled_map) {
                continue;
            }

            let detection = scan_platform_roots(
                plat_id,
                &[],
                &[],
                true,
            );

            let count = detection.games.len();
            let is_available = !detection.roots.is_empty() || count > 0;
            all_games.extend(detection.games);
            all_statuses.push(crate::platforms::PlatformStatus {
                platform: crate::platforms::Platform::from_id(plat_id).unwrap_or(crate::platforms::Platform::Other),
                label: plat_id.to_string(),
                available: is_available,
                count,
                roots: detection.roots,
                notes: detection.note,
            });
        }

        // Also scan EmulationStation ROM directories across all drives (including archive extraction & rom.ini indexing)
        if let Ok(rom_games) = crate::platforms::scan_emulation_roms("", true) {
            let rom_count = rom_games.len();
            if rom_count > 0 {
                let rom_roots = crate::platforms::emulation::detect_rom_directories();
                all_statuses.push(crate::platforms::PlatformStatus {
                    platform: crate::platforms::Platform::EmulationStation,
                    label: "EmulationStation".to_string(),
                    available: true,
                    count: rom_count,
                    roots: rom_roots,
                    notes: None,
                });
                all_games.extend(rom_games);
            }
        }

        // Deduplicate games by (platform, canonical_title_key), prioritizing installed games
        all_games.sort_by_key(|g| (!g.installed, g.name.clone()));
        let mut deduped_games = Vec::new();
        let mut seen_keys = std::collections::HashSet::new();
        for game in all_games {
            let key = format!(
                "{}:{}:{}",
                game.platform.id(),
                game.console.as_deref().unwrap_or("").to_lowercase(),
                crate::plugins::title_normalizer::canonical_title_key(&game.name)
            );
            if !seen_keys.contains(&key) {
                seen_keys.insert(key);
                deduped_games.push(game);
            }
        }

        PlatformScan {
            games: deduped_games,
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

/// Check if an executable filename represents a launcher client, uninstaller, crash handler, or setup helper.
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
        || lower.starts_with("notification_helper")
        || lower.ends_with("_cleanup.exe")
        || lower == "nwjc.exe"
        || lower == "steamcmd.exe"
        || lower == "python.exe"
        || lower == "pythonw.exe"
        || lower == "butler.exe"
        || lower == "legendary.exe"
        || lower == "epiconlineservicesinstaller.exe"
        || lower == "gamelauncher.exe"
        || lower == "ubisoftgamelauncher.exe"
        || lower == "ubisoftgamelauncher64.exe"
        || lower == "ubisoftconnect.exe"
        || lower == "upc.exe"
        || lower == "uplay.exe"
        || lower == "uplayservice.exe"
        || lower == "uplaywebcore.exe"
        || lower == "eadesktop.exe"
        || lower == "ealauncher.exe"
        || lower == "eabackgroundservice.exe"
        || lower == "origin.exe"
        || lower == "galaxyclient.exe"
        || lower == "galaxyclient service.exe"
        || lower == "epicgameslauncher.exe"
        || lower == "battle.net.exe"
        || lower == "battle.net launcher.exe"
        || lower == "rockstarlauncher.exe"
        || lower == "rockstarservice.exe"
        || lower == "socialclubhelper.exe"
}

/// Check if a subfolder name is a launcher application folder rather than an installed game.
fn is_launcher_application_folder(folder_name: &str, folder_path: &Path) -> bool {
    let lower = folder_name.trim().to_lowercase();
    if [
        "ubisoft game launcher",
        "ubisoft connect",
        "gog galaxy",
        "ea desktop",
        "origin",
        "epic games launcher",
        "launcher",
        "legendary",
        "broth",
        "downloads",
        "rockstar games launcher",
        "rockstar games social club",
        "battle.net",
        "steam",
        "installers",
        "redist",
        "directx",
        "support",
    ]
    .contains(&lower.as_str())
    {
        return true;
    }

    // Check if the directory itself is a launcher client or CLI tool directory
    if folder_path.join("upc.exe").is_file()
        || folder_path.join("UbisoftGameLauncher.exe").is_file()
        || folder_path.join("UbisoftConnect.exe").is_file()
        || folder_path.join("GalaxyClient.exe").is_file()
        || folder_path.join("EADesktop.exe").is_file()
        || folder_path.join("legendary.exe").is_file()
        || folder_path.join("EpicOnlineServicesInstaller.exe").is_file()
        || folder_path.join("Portal").join("Binaries").is_dir()
    {
        return true;
    }

    false
}

/// Scans a single directory representing an installed game to find its primary executable and size.
fn inspect_game_folder(
    folder: &Path,
    executable_names: &[String],
) -> Option<(PathBuf, u64)> {
    fn scan_recursive(
        dir: &Path,
        executable_names: &[String],
        depth: usize,
        max_depth: usize,
        best_exe: &mut Option<PathBuf>,
        largest_size: &mut u64,
        total_size: &mut u64,
    ) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let size = entry.metadata().ok().map(|m| m.len()).unwrap_or(0);
                *total_size += size;

                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    #[cfg(windows)]
                    let is_exec = name.to_lowercase().ends_with(".exe");
                    #[cfg(not(windows))]
                    let is_exec = !name.contains('.') || name.ends_with(".bin") || name.ends_with(".app");

                    if is_exec && !is_ignored_executable(name) {
                        let lower_name = name.to_lowercase();
                        let is_explicit = executable_names.iter().any(|e| e.to_lowercase() == lower_name);

                        if is_explicit {
                            *best_exe = Some(path.clone());
                            *largest_size = u64::MAX;
                        } else if size > *largest_size && *largest_size != u64::MAX {
                            *best_exe = Some(path.clone());
                            *largest_size = size;
                        }
                    }
                }
            } else if path.is_dir() && depth < max_depth {
                let folder_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                if !folder_name.starts_with('.')
                    && !folder_name.starts_with('$')
                    && !["temp", "download", "cache", "locales", "swiftshader", "node_modules"].contains(&folder_name.as_str())
                {
                    scan_recursive(&path, executable_names, depth + 1, max_depth, best_exe, largest_size, total_size);
                }
            }
        }
    }

    let mut best_exe: Option<PathBuf> = None;
    let mut largest_exe_size = 0u64;
    let mut total_size = 0u64;

    scan_recursive(folder, executable_names, 0, 3, &mut best_exe, &mut largest_exe_size, &mut total_size);

    best_exe.map(|exe| (exe, total_size))
}

/// Returns well-known relative or absolute game installation subfolders for a platform across drives.
pub fn default_platform_subfolders(platform_id: &str) -> Vec<&'static str> {
    match platform_id {
        "gog" => vec![
            "Games/GOG Galaxy/Games",
            "GOG Galaxy/Games",
            "GOG Games",
            "Games/GOG Games",
            "Program Files (x86)/GOG Galaxy/Games",
            "Program Files/GOG Galaxy/Games",
        ],
        "ea" => vec![
            "Games/EA Games",
            "EA Games",
            "Program Files/EA Games",
            "Program Files (x86)/Origin Games",
            "Origin Games",
        ],
        "epic" => vec![
            "Games/Epic Games",
            "Epic Games",
            "Program Files/Epic Games",
            "Program Files (x86)/Epic Games",
        ],
        "itch" => vec![
            "Games/itch/games",
            "itch/games",
            "itch/apps",
        ],
        "ubisoft" => vec![
            "Games/Ubisoft/Ubisoft Game Launcher/games",
            "Ubisoft/Ubisoft Game Launcher/games",
            "Games/Ubisoft Games",
            "Ubisoft Games",
            "Program Files (x86)/Ubisoft/Ubisoft Game Launcher/games",
            "Program Files/Ubisoft/Ubisoft Game Launcher/games",
        ],
        "xbox" => vec![
            "XboxGames",
            "Games/XboxGames",
            "Program Files/ModifiableWindowsApps",
        ],
        "battlenet" => vec![
            "Battle.net Games",
            "Games/Battle.net Games",
        ],
        "amazon" => vec![
            "Amazon Games/Library",
            "Games/Amazon Games",
        ],
        "rockstar" => vec![
            "Rockstar Games",
            "Games/Rockstar Games",
            "Program Files/Rockstar Games",
        ],
        "standalone" | "other" => vec![
            "Games/Square Enix",
            "Square Enix",
            "Program Files (x86)/SquareEnix",
            "Program Files/SquareEnix",
            "Games/Standalone",
            "Standalone Games",
        ],
        _ => vec![],
    }
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

    // 1. Authoritative local client database discovery for owned games
    match platform_id {
        "gog" => {
            games.extend(crate::plugins::client_discovery::discover_gog_client_games());
        }
        "itch" => {
            games.extend(crate::plugins::client_discovery::discover_itch_client_games());
        }
        "epic" => {
            games.extend(crate::plugins::client_discovery::discover_epic_client_games());
        }
        "ea" => {
            games.extend(crate::plugins::client_discovery::discover_ea_client_games());
        }
        "ubisoft" => {
            games.extend(crate::plugins::client_discovery::discover_ubisoft_client_games());
        }
        "xbox" => {
            games.extend(crate::plugins::client_discovery::discover_xbox_client_games());
        }
        "battlenet" => {
            games.extend(crate::plugins::client_discovery::discover_battlenet_client_games());
        }
        _ => {}
    }

    let mut all_candidate_roots: Vec<PathBuf> = roots.iter().map(PathBuf::from).collect();

    // 2. Across all mounted drives, search default platform folders
    let mounted = drive_roots();
    for sub in default_platform_subfolders(platform_id) {
        for drive in &mounted {
            let candidate = drive.join(sub);
            if candidate.is_dir() && !all_candidate_roots.contains(&candidate) {
                all_candidate_roots.push(candidate);
            }
        }
    }

    // 2. Also check User AppData directories (e.g. itch, Amazon)
    if platform_id == "itch" {
        if let Some(appdata) = dirs::data_dir() {
            let itch_apps = appdata.join("itch").join("apps");
            if itch_apps.is_dir() && !all_candidate_roots.contains(&itch_apps) {
                all_candidate_roots.push(itch_apps);
            }
        }
        if let Some(local_appdata) = dirs::data_local_dir() {
            let itch_apps = local_appdata.join("itch").join("apps");
            if itch_apps.is_dir() && !all_candidate_roots.contains(&itch_apps) {
                all_candidate_roots.push(itch_apps);
            }
        }
    } else if platform_id == "amazon" {
        if let Some(local_appdata) = dirs::data_local_dir() {
            let amz_lib = local_appdata.join("Amazon Games").join("Library");
            if amz_lib.is_dir() && !all_candidate_roots.contains(&amz_lib) {
                all_candidate_roots.push(amz_lib);
            }
        }
    }

    if include_other_drives {
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

        // If a user or config pointed directly at a `Ubisoft`, `Ubisoft Game Launcher`, or `itch` folder
        // that has a `games` child directory, scan `games` instead of the launcher client root!
        let effective_root = if (platform_id == "ubisoft"
            || platform_id == "itch"
            || root.ends_with("Ubisoft Game Launcher"))
            && root.join("games").is_dir()
        {
            root.join("games")
        } else if root.join("Ubisoft Game Launcher").join("games").is_dir() {
            root.join("Ubisoft Game Launcher").join("games")
        } else {
            root
        };

        let root_str = normalize_path(&effective_root.display().to_string());
        if !verified_roots.contains(&root_str) {
            verified_roots.push(root_str);
        }

        if let Ok(entries) = fs::read_dir(&effective_root) {
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
                        || is_launcher_application_folder(&folder_name, &path)
                    {
                        continue;
                    }

                    if let Some((exe_path, size)) = inspect_game_folder(&path, executable_names) {
                        // Prefer authoritative title from local manifest (e.g. itch receipt / RPG Maker / Godot / PE) if present,
                        // otherwise normalize the folder name across all platforms.
                        let raw_title = crate::metadata::local::inspect_local_game(Some(&path), Some(&exe_path))
                            .and_then(|m| m.name)
                            .filter(|n| !n.trim().is_empty())
                            .unwrap_or_else(|| folder_name.clone());
                        let game_title = crate::platforms::normalize_game_title(&raw_title);
                        let final_name = if game_title.is_empty() { folder_name } else { game_title };

                        let target_key = crate::plugins::title_normalizer::canonical_title_key(&final_name);
                        let mut found_existing = false;
                        for cg in &mut games {
                            if crate::plugins::title_normalizer::canonical_title_key(&cg.name) == target_key {
                                cg.installed = true;
                                cg.install_dir = Some(path.display().to_string());
                                cg.exe = Some(exe_path.display().to_string());
                                cg.size_on_disk = Some(size);
                                found_existing = true;
                                break;
                            }
                        }

                        if !found_existing {
                            games.push(PlatformGame {
                                name: final_name,
                                platform: crate::platforms::Platform::from_id(platform_id).unwrap_or(crate::platforms::Platform::Other),
                                platform_id: Some(format!("{}:{}", platform_id, path.display())),
                                install_dir: Some(path.display().to_string()),
                                exe: Some(exe_path.display().to_string()),
                                size_on_disk: Some(size),
                                launch_uri: None,
                                source: Some(format!("Dynamic {} Scanner", platform_id)),
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
    }

    Detection {
        games,
        roots: verified_roots,
        note: None,
    }
}
