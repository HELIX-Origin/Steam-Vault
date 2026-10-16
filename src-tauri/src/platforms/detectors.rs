//! Per-platform detectors.
//!
//! Each detector returns a [`Detection`]: the games it found plus the local
//! source paths it consulted, and an optional note explaining a limitation.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;

use super::{EmulatorConfig, Platform, PlatformGame, PlatformScan, PlatformStatus, ScanConfig};

/// The outcome of one detector run.
pub(crate) struct Detection {
    pub(crate) games: Vec<PlatformGame>,
    pub(crate) roots: Vec<String>,
    pub(crate) note: Option<String>,
}

impl Detection {
    pub(crate) fn empty() -> Self {
        Detection {
            games: Vec::new(),
            roots: Vec::new(),
            note: None,
        }
    }
}

/// Platforms surfaced in the UI, in display order.
const ORDER: [Platform; 11] = [
    Platform::Gog,
    Platform::Epic,
    Platform::Itch,
    Platform::Ubisoft,
    Platform::Ea,
    Platform::BattleNet,
    Platform::Amazon,
    Platform::Rockstar,
    Platform::Xbox,
    Platform::Emulator,
    Platform::Other,
];

/// Scan every supported non-Steam platform on this machine.
///
/// Detection honours the user's per-platform preferences ([`ScanConfig`]) and
/// folds in two extra sources so games installed outside a launcher's default
/// location — notably on an external drive — are still discovered:
/// explicitly configured folders, and well-known game folders found on every
/// mounted drive.
pub fn scan_all(config: &ScanConfig) -> PlatformScan {
    let mut by_platform: BTreeMap<Platform, Detection> = BTreeMap::new();
    let mounted_drives = if config.include_other_drives {
        drive_roots()
    } else {
        Vec::new()
    };

    // OS-specific platform launchers (Windows, macOS, Linux)
    #[cfg(target_os = "windows")]
    {
        for (platform, detection) in
            crate::platforms::windows::detect_platform_launchers(config, &mounted_drives)
        {
            merge(&mut by_platform, platform, detection);
        }
    }
    #[cfg(target_os = "macos")]
    {
        for (platform, detection) in
            crate::platforms::macos::detect_platform_launchers(config, &mounted_drives)
        {
            merge(&mut by_platform, platform, detection);
        }
    }
    #[cfg(target_os = "linux")]
    {
        for (platform, detection) in
            crate::platforms::linux::detect_platform_launchers(config, &mounted_drives)
        {
            merge(&mut by_platform, platform, detection);
        }
    }

    if config.is_enabled(Platform::Itch) {
        merge(&mut by_platform, Platform::Itch, detect_itch());
    }
    if config.is_enabled(Platform::Emulator) {
        let extra_roots = config.roots_for(Platform::Emulator);
        merge(
            &mut by_platform,
            Platform::Emulator,
            detect_user_emulators(&config.emulator_configs, &extra_roots),
        );
    }

    // User-configured folders plus well-known folders on every mounted drive.
    for platform in ORDER {
        if !config.is_enabled(platform) || platform == Platform::Emulator {
            continue;
        }
        let mut roots = config.roots_for(platform);
        if config.include_other_drives
            && platform != Platform::Xbox
            && platform != Platform::Emulator
        {
            roots.extend(drive_candidates(platform, &mounted_drives));
        }
        let mut norm_roots: Vec<String> = roots.into_iter().map(|r| normalize_path(&r)).collect();
        norm_roots.retain(|r| !is_blacklisted_root(Path::new(r)));
        norm_roots.sort();
        norm_roots.dedup();
        if !norm_roots.is_empty() {
            for root in &norm_roots {
                merge(
                    &mut by_platform,
                    platform,
                    detect_extra_root(platform, root),
                );
            }
        }
    }

    let mut games = Vec::new();
    let mut platforms = Vec::new();
    let mut seen: BTreeSet<(Platform, String)> = BTreeSet::new();

    for platform in ORDER {
        let enabled = config.is_enabled(platform);
        let detection = if enabled {
            by_platform
                .remove(&platform)
                .unwrap_or_else(Detection::empty)
        } else {
            Detection::empty()
        };

        let roots = detection.roots;
        let note = detection.note;
        let mut count = 0;
        for game in detection.games {
            if is_game_candidate(&game.name, game.exe.as_deref(), game.install_dir.as_deref())
                && seen.insert((platform, dedup_key(&game)))
            {
                games.push(game);
                count += 1;
            }
        }
        let available = !roots.is_empty() || count > 0;

        platforms.push(PlatformStatus {
            platform,
            label: platform.label().to_string(),
            available,
            count,
            roots,
            notes: if enabled {
                note
            } else {
                Some("Disabled in Settings.".into())
            },
        });
    }

    PlatformScan { games, platforms }
}

/// Standardize Windows/POSIX path separators, eliminate trailing slashes, and trim whitespace.
pub fn normalize_path(path: &str) -> String {
    let mut s = path.trim().replace('/', "\\");
    while s.ends_with('\\') && s.len() > 3 {
        s.pop();
    }
    s
}

/// Well-known install folder names per platform, searched on every drive.
fn drive_folder_names(platform: Platform) -> &'static [&'static str] {
    match platform {
        Platform::Gog => &["GOG Games"],
        Platform::Epic => &["Epic Games"],
        Platform::Itch => &["itch\\games", "itch"],
        Platform::Ubisoft => &[
            "Ubisoft Games",
            "Ubisoft Game Launcher\\games",
            "Games\\Ubisoft\\Ubisoft Game Launcher\\games",
            "Games\\Ubisoft",
        ],
        Platform::Ea => &[
            "EA Games",
            "Origin Games",
            "Games\\EA Games",
            "Games\\Origin Games",
        ],
        Platform::BattleNet => &["Battle.net", "Games\\Battle.net"],
        Platform::Amazon => &["Amazon Games", "Games\\Amazon Games"],
        Platform::Rockstar => &["Rockstar Games", "Games\\Rockstar Games"],
        Platform::Xbox => &["XboxGames"],
        Platform::Emulator | Platform::Steam | Platform::Other => &[],
    }
}

/// Check if a drive or volume represents a cloud sync provider (Google Drive, OneDrive, etc.).
pub fn is_cloud_disk(name: &str, mount: &str) -> bool {
    let n = name.to_lowercase();
    let m = mount.to_lowercase();
    n.contains("google drive")
        || n.contains("onedrive")
        || n.contains("dropbox")
        || n.contains("icloud")
        || n.contains("proton drive")
        || n.contains("pcloud")
        || n.contains("box sync")
        || n.contains("nextcloud")
        || n.contains("owncloud")
        || n.contains("mega")
        || m.contains("google drive")
        || m.contains("onedrive")
        || m.contains("dropbox")
        || m.contains("icloud")
}

/// Every mounted drive root (`C:\`, `D:\`, …) using sysinfo to avoid probing unmounted drive letters.
pub(crate) fn drive_roots() -> Vec<PathBuf> {
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

/// Well-known folders for a platform that exist on any mounted drive.
fn drive_candidates(platform: Platform, mounted_drives: &[PathBuf]) -> Vec<String> {
    let names = drive_folder_names(platform);
    if names.is_empty() || mounted_drives.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for drive in mounted_drives {
        for name in names {
            let candidate = drive.join(name);
            if candidate.is_dir() {
                out.push(normalize_path(&candidate.display().to_string()));
            }
        }
    }
    out
}

/// Check if a path is a launcher's internal data, cache, or config directory (not a game library).
pub fn is_blacklisted_root(path: &Path) -> bool {
    let p = path.to_string_lossy().to_lowercase();
    p.ends_with("\\appdata\\roaming\\itch")
        || p.ends_with("/appdata/roaming/itch")
        || p.ends_with("\\appdata\\local\\itch")
        || p.ends_with("/appdata/local/itch")
        || p.contains("\\programdata\\ea desktop")
        || p.contains("/programdata/ea desktop")
        || p.contains("\\electronic arts\\ea desktop")
        || p.contains("/electronic arts/ea desktop")
        || p.contains("\\appdata\\local\\electronic arts")
        || p.contains("/appdata/local/electronic arts")
        || p.contains("\\gog.com\\galaxy\\storage")
        || p.contains("/gog.com/galaxy/storage")
        || ((p.contains("\\ubisoft game launcher") || p.contains("/ubisoft game launcher"))
            && !p.contains("\\games") && !p.contains("/games"))
        || p.ends_with("\\appdata\\roaming\\battle.net")
        || p.ends_with("/appdata/roaming/battle.net")
        || p.contains("\\rockstar games\\launcher")
        || p.contains("/rockstar games/launcher")
}

/// Discover EA game install directories from user and machine configs.
fn ea_library_dirs() -> Vec<String> {
    let mut out = Vec::new();

    // 1. User config: %LOCALAPPDATA%\Electronic Arts\EA Desktop\user_*.ini
    if let Some(local) = env_dir("LOCALAPPDATA") {
        let ea_user_dir = local.join("Electronic Arts").join("EA Desktop");
        if let Ok(entries) = std::fs::read_dir(&ea_user_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("user_") && name.ends_with(".ini") {
                    if let Ok(text) = std::fs::read_to_string(entry.path()) {
                        for line in text.lines() {
                            if let Some(rest) = line.strip_prefix("user.downloadinplacedir") {
                                let val = rest.trim_start_matches(['=', ' ', '\t']).trim();
                                if !val.is_empty() {
                                    out.push(val.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Machine config: %ProgramData%\Electronic Arts\EA Desktop\machine.ini or %ProgramData%\EA Desktop\machine.ini
    if let Some(prog_data) = env_dir("ProgramData") {
        for candidate in &[
            prog_data
                .join("Electronic Arts")
                .join("EA Desktop")
                .join("machine.ini"),
            prog_data.join("EA Desktop").join("machine.ini"),
        ] {
            if let Ok(text) = std::fs::read_to_string(candidate) {
                for line in text.lines() {
                    if let Some(rest) = line.strip_prefix("machine.downloadinplacedir") {
                        let val = rest.trim_start_matches(['=', ' ', '\t']).trim();
                        if !val.is_empty() {
                            out.push(val.to_string());
                        }
                    }
                }
            }
        }
    }

    // 3. Well-known install folders
    if let Some(pf) = env_dir("ProgramFiles") {
        out.push(pf.join("EA Games").display().to_string());
    }
    if let Some(pfx86) = env_dir("ProgramFiles(x86)") {
        out.push(pfx86.join("Origin Games").display().to_string());
    }

    out
}

/// Discover ROM directories from popular emulator configurations.
fn emulator_library_dirs() -> Vec<String> {
    let mut out = Vec::new();

    // 1. Dolphin: %APPDATA%\Dolphin Emulator\Config\Dolphin.ini or Documents\Dolphin Emulator\Config\Dolphin.ini
    let mut dolphin_inis = Vec::new();
    if let Some(appdata) = env_dir("APPDATA") {
        dolphin_inis.push(
            appdata
                .join("Dolphin Emulator")
                .join("Config")
                .join("Dolphin.ini"),
        );
    }
    if let Some(docs) = dirs::document_dir() {
        dolphin_inis.push(
            docs.join("Dolphin Emulator")
                .join("Config")
                .join("Dolphin.ini"),
        );
    }
    for ini in dolphin_inis {
        if let Ok(text) = std::fs::read_to_string(&ini) {
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("ISOPath") {
                    if let Some(idx) = trimmed.find('=') {
                        let path_str = trimmed[idx + 1..].trim();
                        if !path_str.is_empty() && Path::new(path_str).is_dir() {
                            out.push(path_str.to_string());
                        }
                    }
                }
            }
        }
    }

    // 2. PCSX2: inis\PCSX2.ini in Documents\PCSX2, APPDATA\PCSX2, or LOCALAPPDATA\PCSX2
    let mut pcsx2_inis = Vec::new();
    if let Some(docs) = dirs::document_dir() {
        pcsx2_inis.push(docs.join("PCSX2").join("inis").join("PCSX2.ini"));
    }
    if let Some(appdata) = env_dir("APPDATA") {
        pcsx2_inis.push(appdata.join("PCSX2").join("inis").join("PCSX2.ini"));
    }
    if let Some(local) = env_dir("LOCALAPPDATA") {
        pcsx2_inis.push(local.join("PCSX2").join("inis").join("PCSX2.ini"));
    }
    for ini in pcsx2_inis {
        if let Ok(text) = std::fs::read_to_string(&ini) {
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("Games =") || trimmed.starts_with("RecursivePaths =") {
                    if let Some(idx) = trimmed.find('=') {
                        let path_str = trimmed[idx + 1..].trim();
                        if !path_str.is_empty() && Path::new(path_str).is_dir() {
                            out.push(path_str.to_string());
                        }
                    }
                }
            }
        }
    }

    // 3. DuckStation: settings.ini in Documents\DuckStation or APPDATA\DuckStation
    let mut duck_inis = Vec::new();
    if let Some(docs) = dirs::document_dir() {
        duck_inis.push(docs.join("DuckStation").join("settings.ini"));
    }
    if let Some(appdata) = env_dir("APPDATA") {
        duck_inis.push(appdata.join("DuckStation").join("settings.ini"));
    }
    for ini in duck_inis {
        if let Ok(text) = std::fs::read_to_string(&ini) {
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("RecursivePaths") {
                    if let Some(idx) = trimmed.find('=') {
                        let path_str = trimmed[idx + 1..].trim();
                        if !path_str.is_empty() && Path::new(path_str).is_dir() {
                            out.push(path_str.to_string());
                        }
                    }
                }
            }
        }
    }

    // 4. Ryujinx: %APPDATA%\Ryujinx\Config.json
    if let Some(appdata) = env_dir("APPDATA") {
        let ryu_cfg = appdata.join("Ryujinx").join("Config.json");
        if let Ok(text) = std::fs::read_to_string(&ryu_cfg) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(dirs) = val.get("game_dirs").and_then(|v| v.as_array()) {
                    for d in dirs {
                        if let Some(p) = d.as_str() {
                            if Path::new(p).is_dir() {
                                out.push(p.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // 5. Cemu: %LOCALAPPDATA%\Cemu\settings.xml
    if let Some(local) = env_dir("LOCALAPPDATA") {
        let cemu_xml = local.join("Cemu").join("settings.xml");
        if let Ok(text) = std::fs::read_to_string(&cemu_xml) {
            let mut start = 0;
            while let Some(open) = text[start..].find("<Path>") {
                let tag_start = start + open + 6;
                if let Some(close) = text[tag_start..].find("</Path>") {
                    let path_str = text[tag_start..tag_start + close].trim();
                    if !path_str.is_empty() && Path::new(path_str).is_dir() {
                        out.push(path_str.to_string());
                    }
                    start = tag_start + close + 7;
                } else {
                    break;
                }
            }
        }
    }

    // 6. RetroArch: %APPDATA%\RetroArch\retroarch.cfg
    if let Some(appdata) = env_dir("APPDATA") {
        let ra_cfg = appdata.join("RetroArch").join("retroarch.cfg");
        if let Ok(text) = std::fs::read_to_string(&ra_cfg) {
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("rgui_browser_directory")
                    || trimmed.starts_with("content_directory")
                {
                    if let Some(idx) = trimmed.find('=') {
                        let path_str = trimmed[idx + 1..].trim().trim_matches('"');
                        if !path_str.is_empty() && Path::new(path_str).is_dir() {
                            out.push(path_str.to_string());
                        }
                    }
                }
            }
        }
    }

    out
}

/// Existing locations for a platform, used to autofill Settings.
///
/// Returns actual game install directories where games or ROMs reside. Launcher
/// internal configuration/cache folders are explicitly excluded.
pub fn library_folders(platform: Platform) -> Vec<String> {
    #[cfg(target_os = "windows")]
    let mut out = crate::platforms::windows::default_library_folders(platform);

    #[cfg(target_os = "macos")]
    let mut out = crate::platforms::macos::default_library_folders(platform);

    #[cfg(target_os = "linux")]
    let mut out = crate::platforms::linux::default_library_folders(platform);

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let mut out: Vec<String> = Vec::new();

    if platform == Platform::Emulator {
        out.extend(emulator_library_dirs());
    }

    let mounted = drive_roots();
    out.extend(drive_candidates(platform, &mounted));

    // Normalize all paths (standardize slashes, remove trailing slashes)
    let mut normalized: Vec<String> = out.into_iter().map(|p| normalize_path(&p)).collect();

    // Exclude launcher internal directories, non-existent folders, and empty folders without any game files
    normalized.retain(|path| {
        let p = Path::new(path);
        if !p.is_dir() || is_blacklisted_root(p) {
            return false;
        }
        // Folder must contain at least one entry (e.g. game folder or files)
        if let Ok(mut it) = std::fs::read_dir(p) {
            it.next().is_some()
        } else {
            false
        }
    });
    normalized.sort();
    normalized.dedup();
    normalized
}

/// Filter out non-game software, launchers, tools, and system packages.
pub fn is_game_candidate(name: &str, exe: Option<&str>, install_dir: Option<&str>) -> bool {
    let n = name.trim().to_lowercase();
    if n.is_empty() {
        return false;
    }

    // Hex hash filter (e.g. 530c11479fe252fc5aabc24935b9776d4900eb3ba58fdc271e0d6229413ad40e)
    if n.len() >= 16 && n.chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }

    // 1. Direct blacklisted titles & launchers & cache folders
    const BLACKLISTED_TITLES: &[&str] = &[
        "gog galaxy",
        "epic games launcher",
        "epic online services",
        "ea app",
        "ea desktop",
        "origin",
        "ubisoft connect",
        "ubisoft game launcher",
        "uplay",
        "battle.net",
        "battle.net launcher",
        "blizzard battle.net",
        "rockstar games launcher",
        "rockstar games social club",
        "social club",
        "amazon games",
        "amazon games app",
        "riot client",
        "riot vanguard",
        "steam",
        "itch",
        "itch.io",
        "butler",
        "gamesave",
        "gamingservices",
        "microsoft gameinput",
        "directxredist",
        "gameinputredist",
        "installdata",
        "code cache",
        "local storage",
        "session storage",
        "shared dictionary",
        "partitions",
        "network",
        "db",
        "logs",
        "log",
        "plr",
        "prereqs",
        "prerequisites",
        "__macosx",
        "downloads",
        "download",
        "support",
        "redist",
        "common",
        "bin",
        "docs",
        "lib",
        "temp",
        "tmp",
        "blob_storage",
        "cache",
        "broth",
        "games",
        "apps",
        "webrtc",
        "gpucache",
        "indexeddb",
        "crashpad",
        "crashdump",
        "cleanup",
        "touchup",
        "__installer",
        "license",
        "licenses",
        "autoupdate",
        "telemetry",
        "node_modules",
    ];
    if BLACKLISTED_TITLES
        .iter()
        .any(|&b| n == b || n == format!("{b} app") || n == format!("{b} client"))
    {
        return false;
    }

    // 2. Blacklisted keywords for tools, redistributables, runtimes, and services
    const SOFTWARE_KEYWORDS: &[&str] = &[
        "redistributable",
        "directx",
        "gameinput",
        "installer",
        "unins000",
        "prerequisite",
        "runtime",
        "sdk",
        "supportassist",
        "driver",
        "dedicated server",
        "benchmark",
        "anti-cheat",
        "vanguard",
        "easyanticheat",
        "battleye",
        "touchup",
        "cleanup",
    ];
    if SOFTWARE_KEYWORDS.iter().any(|&kw| n.contains(kw)) {
        return false;
    }

    // 3. Check executable name (if present)
    if let Some(exe_str) = exe {
        if is_executable_blacklisted(Path::new(exe_str)) {
            return false;
        }
    }

    // 4. Check if directory is an ignored directory name or inside cache
    if let Some(dir_str) = install_dir {
        let dir_path = Path::new(dir_str);
        if is_blacklisted_root(dir_path) {
            return false;
        }
        if let Some(folder_name) = dir_path.file_name().and_then(|s| s.to_str()) {
            let folder_lower = folder_name.to_lowercase();
            if folder_lower.len() >= 16 && folder_lower.chars().all(|c| c.is_ascii_hexdigit()) {
                return false;
            }
            if BLACKLISTED_TITLES.contains(&folder_lower.as_str()) {
                return false;
            }
        }
    }

    true
}

/// Detect games from a user-configured extra location.
fn detect_extra_root(platform: Platform, root: &str) -> Detection {
    if platform == Platform::Epic {
        let games = epic_manifests(Path::new(root));
        if !games.is_empty() {
            return Detection {
                games,
                roots: vec![root.to_string()],
                note: None,
            };
        }
    }
    if platform == Platform::Xbox {
        let mut detection = Detection::empty();
        scan_xbox_dir(Path::new(root), &mut detection);
        if !detection.games.is_empty() {
            return detection;
        }
    }
    if platform == Platform::Itch {
        let mut detection = Detection::empty();
        scan_itch_dir(Path::new(root), &mut detection);
        if !detection.games.is_empty() {
            return detection;
        }
    }
    if platform == Platform::Emulator {
        let mut detection = Detection::empty();
        let emu_map = BTreeMap::new();
        scan_emulator_dir(Path::new(root), &mut detection, &emu_map);
        if !detection.games.is_empty() {
            return detection;
        }
    }
    detect_custom_roots(platform, &[root.to_string()])
}

/// Discover one game per immediate subdirectory of each configured folder.
fn detect_custom_roots(platform: Platform, roots: &[String]) -> Detection {
    let mut detection = Detection::empty();
    for root in roots {
        let root_path = Path::new(root);
        if !root_path.is_dir() {
            continue;
        }
        detection.roots.push(root.clone());
        let Ok(entries) = std::fs::read_dir(root_path) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let name_lower = name.to_lowercase();

            // If entry is named "games" or "apps", dive into it!
            if name_lower == "games" || name_lower == "apps" {
                if let Ok(sub_entries) = std::fs::read_dir(&path) {
                    for sub in sub_entries.flatten() {
                        let sub_path = sub.path();
                        if !sub_path.is_dir() {
                            continue;
                        }
                        let sub_name = sub.file_name().to_string_lossy().to_string();
                        let sub_exe = find_executable(&sub_path, 3);
                        // Folder scan candidates MUST have an executable or recognized indie entrypoint
                        if sub_exe.is_none() {
                            continue;
                        }
                        let sub_dir = Some(sub_path.display().to_string());
                        if is_game_candidate(&sub_name, sub_exe.as_deref(), sub_dir.as_deref()) {
                            detection.games.push(PlatformGame {
                                name: sub_name,
                                platform,
                                platform_id: None,
                                install_dir: sub_dir,
                                exe: sub_exe,
                                size_on_disk: None,
                                launch_uri: None,
                                source: Some("Folder scan".into()),
                                console: None,
                                ..Default::default()
                            });
                        }
                    }
                }
                continue;
            }

            let exe = find_executable(&path, 3);
            // Folder scan candidates MUST have an executable or recognized indie entrypoint
            if exe.is_none() {
                continue;
            }
            let dir = Some(path.display().to_string());
            if is_game_candidate(&name, exe.as_deref(), dir.as_deref()) {
                detection.games.push(PlatformGame {
                    name,
                    platform,
                    platform_id: None,
                    install_dir: dir,
                    exe,
                    size_on_disk: None,
                    launch_uri: None,
                    source: Some("Folder scan".into()),
                    console: None,
                    ..Default::default()
                });
            }
        }
    }
    detection
}

/// A stable identity for de-duplicating the same game across detectors.
fn dedup_key(game: &PlatformGame) -> String {
    if let Some(ref id) = game.platform_id {
        return format!("id:{id}");
    }
    let key = game
        .install_dir
        .as_deref()
        .or(game.exe.as_deref())
        .unwrap_or(&game.name);
    key.replace('/', "\\").trim_end_matches('\\').to_lowercase()
}

fn merge(by: &mut BTreeMap<Platform, Detection>, platform: Platform, mut detection: Detection) {
    let entry = by.entry(platform).or_insert_with(Detection::empty);
    entry.games.append(&mut detection.games);
    entry.roots.append(&mut detection.roots);
    entry.roots.sort();
    entry.roots.dedup();
    if entry.note.is_none() {
        entry.note = detection.note;
    }
}

/// Resolve an environment variable to an existing directory.
pub(crate) fn env_dir(var: &str) -> Option<PathBuf> {
    std::env::var_os(var)
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
}

pub(crate) fn join_exe(dir: &str, relative: &str) -> String {
    let candidate = Path::new(relative);
    if candidate.is_absolute() {
        relative.to_string()
    } else {
        Path::new(dir).join(relative).display().to_string()
    }
}

/// Check if an executable is an uninstaller, setup, crash handler, or support utility.
fn is_executable_blacklisted(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
        return true;
    };
    let n = name.to_lowercase();
    n.starts_with("unins")
        || n.starts_with("setup")
        || n.starts_with("installer")
        || n.starts_with("cleanup")
        || n.starts_with("touchup")
        || n.starts_with("dxsetup")
        || n.starts_with("crashreport")
        || n.starts_with("eaappinstaller")
        || n.starts_with("ealauncher")
        || n.starts_with("eaconnect")
        || n.starts_with("activation")
        || n.starts_with("unitycrashhandler")
        || n.starts_with("crashpad_handler")
        || n.starts_with("cef_subprocess")
        || n.starts_with("notification_helper")
        || n.starts_with("vc_redist")
        || n.starts_with("vcredist")
        || n.starts_with("dotnet")
        || n.starts_with("werfault")
        || n == "easervice.exe"
        || n == "butler.exe"
        || n == "epicgameslauncher.exe"
        || n == "galaxyclient.exe"
        || n == "ubisoftconnect.exe"
}

/// Find the most suitable game executable or indie engine launch file beneath `dir`.
///
/// Uses candidate scoring:
/// - Exact or partial match with the game's folder name: +100
/// - Accompanied by Unity `*_Data` or Godot `*.pck`: +90
/// - Standard launcher names (`Game.exe`, `Launch.exe`, etc.): +80
/// - Root-level executable: +40
/// - Immediate subdirectory (`bin/`, `bin_ship/`): +20
/// - Disqualifies non-game utilities (crash handlers, notification helpers, setups).
/// - Fallback to indie engine entrypoints (`.bat`, `.cmd`, `.love`, `.jar`, `.py`, `.html`).
pub(crate) fn find_executable(dir: &Path, depth: usize) -> Option<String> {
    let mut candidates: Vec<(i32, String)> = Vec::new();
    let folder_stem = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    let clean_folder = folder_stem.replace(|c: char| !c.is_alphanumeric(), "");

    let mut queue = vec![(dir.to_path_buf(), 0usize)];
    while let Some((current, level)) = queue.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        let mut subdirs = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dname = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if dname.starts_with('.')
                    || dname.starts_with("__installer")
                    || dname.starts_with("support")
                    || dname.starts_with("redist")
                    || dname == "crashreport"
                    || dname == "node_modules"
                    || dname == "cache"
                    || dname == "temp"
                    || dname == "tmp"
                    || dname == "logs"
                    || dname == "prereqs"
                    || dname == "prerequisites"
                {
                    continue;
                }
                if level < depth {
                    subdirs.push((path, level + 1));
                }
            } else if path.is_file() {
                let ext = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                let clean_stem = stem.replace(|c: char| !c.is_alphanumeric(), "");

                if ext == "exe" {
                    if is_executable_blacklisted(&path) {
                        continue;
                    }

                    let mut score = 50;

                    // Match folder name
                    if !clean_folder.is_empty()
                        && (clean_stem == clean_folder
                            || clean_stem.starts_with(&clean_folder)
                            || clean_folder.starts_with(&clean_stem))
                    {
                        score += 100;
                    }

                    // Common game launcher names
                    if stem == "game"
                        || stem == "launch"
                        || stem == "launcher"
                        || stem == "start"
                        || stem == "play"
                    {
                        score += 80;
                    }

                    // Level bonus / penalty
                    if level == 0 {
                        score += 40;
                    } else if level == 1 {
                        score += 20;
                    } else {
                        score -= 20;
                    }

                    // Check for Unity / Godot companion markers
                    if current.join(format!("{stem}_Data")).is_dir() {
                        score += 90;
                    }
                    if current.join(format!("{stem}.pck")).is_file() {
                        score += 90;
                    }

                    // Prefer 64-bit
                    if stem.ends_with("64") || stem.ends_with("x64") || stem.ends_with("win64") {
                        score += 10;
                    }

                    // Penalize lib/ internal binaries (like Ren'Py python.exe)
                    let p_str = path.to_string_lossy().to_lowercase();
                    if p_str.contains("\\lib\\")
                        || p_str.contains("/lib/")
                        || p_str.contains("\\lib32\\")
                        || p_str.contains("\\lib64\\")
                    {
                        score -= 60;
                    }

                    // Penalize config / tool keywords
                    if stem.contains("config")
                        || stem.contains("settings")
                        || stem.contains("tool")
                        || stem.contains("editor")
                        || stem.contains("update")
                        || stem.contains("server")
                        || stem.contains("dedicated")
                        || stem.contains("benchmark")
                    {
                        score -= 80;
                    }

                    candidates.push((score, path.display().to_string()));
                } else if ext == "bat" || ext == "cmd" {
                    if !stem.starts_with("unins") && !stem.starts_with("setup") {
                        let score = if level == 0 { 40 } else { 20 };
                        candidates.push((score, path.display().to_string()));
                    }
                } else if ext == "love" || ext == "jar" {
                    let score = if level == 0 { 35 } else { 15 };
                    candidates.push((score, path.display().to_string()));
                } else if ext == "py" && (stem == "main" || stem == "run" || stem == "game") {
                    let score = if level == 0 { 30 } else { 10 };
                    candidates.push((score, path.display().to_string()));
                } else if ext == "html" && stem == "index" {
                    let score = if level == 0 { 25 } else { 5 };
                    candidates.push((score, path.display().to_string()));
                }
            }
        }
        queue.extend(subdirs);
    }

    candidates.sort_by_key(|a| std::cmp::Reverse(a.0));
    candidates.into_iter().next().map(|(_, path)| path)
}

// ---------------------------------------------------------------------------
// GOG
// ---------------------------------------------------------------------------

fn detect_gog() -> Detection {
    let mut detection = Detection::empty();

    #[cfg(windows)]
    {
        use crate::platforms::registry;
        let mut seen_ids = std::collections::BTreeSet::new();

        // 1. Try GOG Galaxy 2.0 SQLite database for complete owned library + install status
        if let Some(program_data) = env_dir("ProgramData") {
            let db_path = program_data
                .join("GOG.com")
                .join("Galaxy")
                .join("storage")
                .join("galaxy-2.0.db");
            if db_path.is_file() {
                detection.roots.push(db_path.display().to_string());
                if let Ok(conn) = rusqlite::Connection::open_with_flags(
                    &db_path,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                ) {
                    // Collect installed product IDs
                    let mut installed_ids = std::collections::BTreeSet::new();
                    if let Ok(mut stmt) = conn.prepare("SELECT productId FROM InstalledProducts") {
                        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, i64>(0)) {
                            for id in rows.flatten() {
                                installed_ids.insert(id.to_string());
                            }
                        }
                    }

                    // Map of registry installs for local path/exe resolution
                    let reg_installs: std::collections::BTreeMap<String, registry::GogInstall> =
                        registry::gog_installs()
                            .into_iter()
                            .map(|inst| (inst.id.clone(), inst))
                            .collect();

                    // Query LibraryReleases joined with GamePieces (304 = title)
                    let query = "SELECT lr.releaseKey, gp.value \
                                 FROM LibraryReleases lr \
                                 LEFT JOIN GamePieces gp ON gp.releaseKey = lr.releaseKey AND gp.gamePieceTypeId = 304 \
                                 WHERE lr.releaseKey LIKE 'gog_%'";
                    if let Ok(mut stmt) = conn.prepare(query) {
                        let rows_result = stmt.query_map([], |row| {
                            let key: String = row.get(0)?;
                            let val: Option<String> = row.get(1)?;
                            Ok((key, val))
                        });

                        if let Ok(rows) = rows_result {
                            for (release_key, val_json) in rows.flatten() {
                                let product_id = release_key
                                    .strip_prefix("gog_")
                                    .unwrap_or(&release_key)
                                    .to_string();
                                seen_ids.insert(product_id.clone());

                                let title = val_json
                                    .as_deref()
                                    .and_then(|txt| serde_json::from_str::<serde_json::Value>(txt).ok())
                                    .and_then(|v| v.get("title").and_then(|t| t.as_str()).map(str::to_string))
                                    .unwrap_or_else(|| product_id.clone());

                                let is_installed = installed_ids.contains(&product_id)
                                    || reg_installs.contains_key(&product_id);

                                let reg = reg_installs.get(&product_id);
                                let install_dir = reg.and_then(|r| r.path.clone());
                                let exe = reg.and_then(|r| r.exe.clone().or_else(|| r.path.clone()));

                                detection.games.push(PlatformGame {
                                    name: title,
                                    platform: Platform::Gog,
                                    platform_id: Some(product_id.clone()),
                                    install_dir,
                                    exe,
                                    size_on_disk: None,
                                    launch_uri: Some(format!("goggalaxy://openGameView/{product_id}")),
                                    source: Some("GOG Galaxy library".into()),
                                    console: None,
                                    installed: is_installed,
                                    install_uri: Some(format!("goggalaxy://installGame/{product_id}")),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
            }
        }

        // 2. Add any GOG games from the registry not already seen in Galaxy DB (e.g. standalone installers)
        let installs = registry::gog_installs();
        if !installs.is_empty() && detection.roots.is_empty() {
            detection
                .roots
                .push("HKEY_LOCAL_MACHINE\\SOFTWARE\\GOG.com\\Games".into());
        }
        for install in installs {
            if seen_ids.insert(install.id.clone()) {
                detection.games.push(PlatformGame {
                    name: install.name,
                    platform: Platform::Gog,
                    platform_id: Some(install.id.clone()),
                    install_dir: install.path.clone(),
                    exe: install.exe.or(install.path),
                    size_on_disk: None,
                    launch_uri: Some(format!("goggalaxy://openGameView/{}", install.id)),
                    source: Some("GOG Galaxy registry".into()),
                    console: None,
                    installed: true,
                    install_uri: Some(format!("goggalaxy://installGame/{}", install.id)),
                    ..Default::default()
                });
            }
        }
    }

    // Linux / macOS: Heroic's GOG store.
    if let Some(config) = dirs::config_dir() {
        let file = config
            .join("heroic")
            .join("gog_store")
            .join("installed.json");
        if file.is_file() {
            detection.roots.push(file.display().to_string());
            detection
                .games
                .extend(heroic_installed(&file, Platform::Gog));
        }
    }

    detection
}

// ---------------------------------------------------------------------------
// Epic Games
// ---------------------------------------------------------------------------

fn detect_epic() -> Detection {
    let mut detection = Detection::empty();

    if let Some(program_data) = env_dir("ProgramData") {
        let dir = program_data
            .join("Epic")
            .join("EpicGamesLauncher")
            .join("Data")
            .join("Manifests");
        if dir.is_dir() {
            detection.roots.push(dir.display().to_string());
            detection.games.extend(epic_manifests(&dir));
        }
    }

    // Legendary / Heroic installed.json locations (covers Linux, macOS, and Windows ~/.config/legendary)
    let mut candidate_dirs = Vec::new();
    if let Some(config) = dirs::config_dir() {
        candidate_dirs.push(
            config
                .join("heroic")
                .join("legendaryConfig")
                .join("legendary"),
        );
        candidate_dirs.push(config.join("legendary"));
    }
    if let Some(home) = dirs::home_dir() {
        candidate_dirs.push(home.join(".config").join("legendary"));
        candidate_dirs.push(
            home.join(".config")
                .join("heroic")
                .join("legendaryConfig")
                .join("legendary"),
        );
    }

    for candidate in candidate_dirs {
        let file = candidate.join("installed.json");
        if file.is_file() {
            detection.roots.push(file.display().to_string());
            detection
                .games
                .extend(heroic_installed(&file, Platform::Epic));
        }
    }

    detection
}

fn epic_from_manifest(path: &Path) -> Option<PlatformGame> {
    let text = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    if value
        .get("bIsIncompleteInstall")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return None;
    }
    let name = value
        .get("DisplayName")
        .and_then(|v| v.as_str())?
        .to_string();
    let install = value
        .get("InstallLocation")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let exe_rel = value
        .get("LaunchExecutable")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let exe = match (&install, exe_rel.is_empty()) {
        (Some(dir), false) => Some(join_exe(dir, exe_rel)),
        _ => install.clone(),
    };
    let app_id = value
        .get("AppName")
        .or_else(|| value.get("CatalogItemId"))
        .and_then(|v| v.as_str());
    Some(PlatformGame {
        name,
        platform: Platform::Epic,
        platform_id: value
            .get("AppName")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        install_dir: install,
        exe,
        size_on_disk: value.get("InstallSize").and_then(|v| v.as_u64()),
        launch_uri: app_id.map(|id| format!("com.epicgames.launcher://apps/{id}?action=launch")),
        source: Some("Epic manifest".into()),
        console: None,
        installed: true,
        install_uri: app_id.map(|id| format!("com.epicgames.launcher://apps/{id}?action=install")),
        ..Default::default()
    })
}

/// Read every Epic `*.item` manifest in a folder into platform games.
fn epic_manifests(dir: &Path) -> Vec<PlatformGame> {
    let mut games = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return games;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_item = path
            .extension()
            .map(|ext| ext.eq_ignore_ascii_case("item"))
            .unwrap_or(false);
        if is_item {
            if let Some(game) = epic_from_manifest(&path) {
                games.push(game);
            }
        }
    }
    games
}

/// Parse a legendary / Heroic `installed.json` (a map of `app_name` → record).
fn heroic_installed(path: &Path, platform: Platform) -> Vec<PlatformGame> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };
    let Some(map) = value.as_object() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (key, entry) in map {
        let title = entry
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or(key)
            .to_string();
        let install = entry
            .get("install_path")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let exe_rel = entry
            .get("executable")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let exe = match (&install, exe_rel.is_empty()) {
            (Some(dir), false) => Some(join_exe(dir, exe_rel)),
            _ => install.clone(),
        };
        out.push(PlatformGame {
            name: title,
            platform,
            platform_id: Some(key.clone()),
            install_dir: install,
            exe,
            size_on_disk: None,
            launch_uri: if platform == Platform::Epic {
                Some(format!("legendary://{key}"))
            } else {
                None
            },
            source: Some("legendary / Heroic".into()),
            console: None,
            installed: true,
            ..Default::default()
        });
    }
    out
}

// ---------------------------------------------------------------------------
// itch.io
// ---------------------------------------------------------------------------

fn detect_itch() -> Detection {
    let mut detection = Detection::empty();

    // 1. Authoritative: read itch's local SQLite database (butler.db)
    let mut db_paths = Vec::new();
    if let Some(appdata) = env_dir("APPDATA") {
        db_paths.push(appdata.join("itch").join("db").join("butler.db"));
    }
    if let Some(config) = dirs::config_dir() {
        db_paths.push(config.join("itch").join("db").join("butler.db"));
    }

    for db_path in db_paths {
        if !db_path.is_file() {
            continue;
        }
        if let Ok(conn) = rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ) {
            detection.roots.push(db_path.display().to_string());
            let query = "
                SELECT c.id, c.game_id, g.title, g.classification, c.installed_size, c.verdict, loc.path
                FROM caves c
                JOIN games g ON c.game_id = g.id
                LEFT JOIN install_locations loc ON c.install_location_id = loc.id
            ";
            if let Ok(mut stmt) = conn.prepare(query) {
                let rows = stmt.query_map([], |row| {
                    let cave_id: String = row.get(0)?;
                    let game_id: i64 = row.get(1)?;
                    let title: String = row.get(2)?;
                    let classification: Option<String> = row.get(3)?;
                    let size: Option<i64> = row.get(4)?;
                    let verdict: Option<String> = row.get(5)?;
                    let loc_path: Option<String> = row.get(6)?;
                    Ok((
                        cave_id,
                        game_id,
                        title,
                        classification,
                        size,
                        verdict,
                        loc_path,
                    ))
                });
                if let Ok(mapped) = rows {
                    for r in mapped.flatten() {
                        let (_cave_id, game_id, title, classification, size, verdict_str, loc_path) =
                            r;
                        // Focus on games: filter out tools, soundtracks, books, comics, assets
                        if let Some(ref cls) = classification {
                            let cls_lower = cls.to_lowercase();
                            if cls_lower != "game" && !cls_lower.is_empty() {
                                continue;
                            }
                        }

                        let mut install_dir = None;
                        let mut exe_path = None;

                        if let Some(ref v_str) = verdict_str {
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(v_str) {
                                if let Some(base) = v.get("basePath").and_then(|b| b.as_str()) {
                                    install_dir = Some(base.to_string());
                                }
                                if let Some(candidates) =
                                    v.get("candidates").and_then(|c| c.as_array())
                                {
                                    for cand in candidates {
                                        let flavor = cand
                                            .get("flavor")
                                            .and_then(|f| f.as_str())
                                            .unwrap_or("");
                                        let is_gui = cand
                                            .get("windowsInfo")
                                            .and_then(|w| w.get("gui"))
                                            .and_then(|g| g.as_bool())
                                            .unwrap_or(false);
                                        let rel_path =
                                            cand.get("path").and_then(|p| p.as_str()).unwrap_or("");

                                        if is_executable_blacklisted(Path::new(rel_path)) {
                                            continue;
                                        }

                                        #[cfg(windows)]
                                        let matches_os = flavor == "windows"
                                            || rel_path.ends_with(".exe")
                                            || flavor == "html"
                                            || rel_path.ends_with(".html")
                                            || flavor == "love"
                                            || rel_path.ends_with(".love")
                                            || flavor == "jar"
                                            || rel_path.ends_with(".jar")
                                            || rel_path.ends_with(".bat")
                                            || rel_path.ends_with(".cmd")
                                            || rel_path.ends_with(".py");
                                        #[cfg(not(windows))]
                                        let matches_os = flavor != "windows";

                                        if matches_os && (is_gui || exe_path.is_none()) {
                                            if let Some(ref base) = install_dir {
                                                let full = Path::new(base).join(rel_path);
                                                if full.is_file() {
                                                    exe_path = Some(full.display().to_string());
                                                    if is_gui {
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if exe_path.is_none() {
                            if let Some(ref dir) = install_dir {
                                exe_path = find_executable(Path::new(dir), 3);
                            }
                        }

                        if let Some(ref loc) = loc_path {
                            if !detection.roots.contains(loc) {
                                detection.roots.push(loc.clone());
                            }
                        }

                        let size_on_disk =
                            size.and_then(|s| if s > 0 { Some(s as u64) } else { None });

                        detection.games.push(PlatformGame {
                            name: title,
                            platform: Platform::Itch,
                            platform_id: Some(game_id.to_string()),
                            install_dir,
                            exe: exe_path,
                            size_on_disk,
                            launch_uri: Some(format!("itch://games/{game_id}")),
                            source: Some("itch app (butler.db)".into()),
                            console: None,
                            installed: true,
                            install_uri: Some(format!("itch://games/{game_id}")),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    // 2. Folder fallback for any standalone or extra itch directories
    let mut dirs_to_scan = Vec::new();
    if let Some(appdata) = env_dir("APPDATA") {
        dirs_to_scan.push(appdata.join("itch").join("apps"));
    }
    if let Some(config) = dirs::config_dir() {
        dirs_to_scan.push(config.join("itch").join("apps"));
    }

    for dir in dirs_to_scan {
        if dir.is_dir() {
            scan_itch_dir(&dir, &mut detection);
        }
    }

    if detection.roots.is_empty() {
        detection.note = Some("No itch app install records or libraries found.".into());
    }

    detection
}

/// Recursively scan an itch library directory, navigating into "games" or "apps"
/// containers instead of treating them as single game items.
fn scan_itch_dir(dir: &Path, detection: &mut Detection) {
    if !dir.is_dir() {
        return;
    }

    // If the directory has a "games" or "apps" subfolder, scan that subfolder
    // instead of treating the container folder itself as a game.
    let games_sub = dir.join("games");
    let apps_sub = dir.join("apps");
    let target_dir = if games_sub.is_dir() {
        &games_sub
    } else if apps_sub.is_dir() {
        &apps_sub
    } else {
        dir
    };

    let root_str = target_dir.display().to_string();
    if !detection.roots.contains(&root_str) {
        detection.roots.push(root_str);
    }

    let Ok(entries) = std::fs::read_dir(target_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let folder_name = entry.file_name().to_string_lossy().to_string();
        let folder_lower = folder_name.to_lowercase();

        // Skip internal/container directories and cache
        if folder_name.starts_with('.')
            || folder_lower == "games"
            || folder_lower == "apps"
            || folder_lower == "broth"
            || folder_lower == "prereqs"
            || folder_lower == "prerequisites"
            || folder_lower == "cache"
            || folder_lower == "blob_storage"
            || folder_lower == "db"
            || folder_lower == "logs"
            || folder_lower == "partitions"
            || folder_lower == "code cache"
            || folder_lower == "local storage"
            || folder_lower == "session storage"
            || folder_lower == "shared dictionary"
            || folder_lower == "network"
            || folder_lower == "downloads"
            || folder_lower == "__macosx"
            || (folder_lower.len() >= 16 && folder_lower.chars().all(|c| c.is_ascii_hexdigit()))
        {
            continue;
        }

        // Try reading .itch/receipt.json.gz, .itch/receipt.json, receipt.json
        let mut game_name = folder_name.clone();
        let mut platform_id = None;
        let mut launch_uri = None;
        let mut is_game = true;

        if let Some(itch_meta) = crate::metadata::local::inspect_local_game(Some(&path), None) {
            if let Some(n) = itch_meta.name {
                game_name = n;
            }
        }

        let receipt_gz = path.join(".itch").join("receipt.json.gz");
        let receipt_plain = path.join(".itch").join("receipt.json");
        let receipt_alt = path.join("receipt.json");

        let mut parsed_receipt: Option<serde_json::Value> = None;
        if receipt_gz.is_file() {
            if let Ok(file) = std::fs::File::open(&receipt_gz) {
                let mut decoder = GzDecoder::new(file);
                let mut text = String::new();
                if decoder.read_to_string(&mut text).is_ok() {
                    parsed_receipt = serde_json::from_str(&text).ok();
                }
            }
        }
        if parsed_receipt.is_none() {
            for p in &[receipt_plain, receipt_alt] {
                if let Ok(text) = std::fs::read_to_string(p) {
                    if let Ok(v) = serde_json::from_str(&text) {
                        parsed_receipt = Some(v);
                        break;
                    }
                }
            }
        }

        if let Some(v) = parsed_receipt {
            if let Some(game) = v.get("game") {
                if let Some(cls) = game.get("classification").and_then(|c| c.as_str()) {
                    if cls.to_lowercase() != "game" && !cls.is_empty() {
                        is_game = false;
                    }
                }
                if let Some(id) = game.get("id").and_then(|id| id.as_i64()) {
                    platform_id = Some(id.to_string());
                    launch_uri = Some(format!("itch://games/{id}"));
                }
                if let Some(title) = game.get("title").and_then(|t| t.as_str()) {
                    game_name = title.to_string();
                }
            }
        }

        if !is_game {
            continue;
        }

        if launch_uri.is_none() {
            launch_uri = Some(format!("itch://games/{folder_name}"));
        }

        let exe = find_executable(&path, 3);
        if exe.is_none() && platform_id.is_none() {
            continue;
        }

        detection.games.push(PlatformGame {
            name: game_name,
            platform: Platform::Itch,
            platform_id,
            install_dir: Some(path.display().to_string()),
            exe,
            size_on_disk: None,
            launch_uri,
            source: Some("itch library folder".into()),
            console: None,
            installed: true,
            ..Default::default()
        });
    }
}

// ---------------------------------------------------------------------------
// Ubisoft Connect
// ---------------------------------------------------------------------------

fn detect_ubisoft() -> Detection {
    #[allow(unused_mut)]
    let mut detection = Detection::empty();

    #[cfg(windows)]
    {
        use crate::platforms::registry;
        let mut seen_dirs = std::collections::BTreeSet::new();

        // 1. Registry installs: HKLM\SOFTWARE\Ubisoft\Launcher\Installs\<id>
        let ubi_installs = registry::ubisoft_installs();
        if !ubi_installs.is_empty() {
            detection
                .roots
                .push("HKEY_LOCAL_MACHINE\\SOFTWARE\\Ubisoft\\Launcher\\Installs".into());
        }
        for install in ubi_installs {
            let norm_dir = normalize_path(&install.install_dir);
            let dir_path = Path::new(&norm_dir);
            if !dir_path.is_dir() {
                continue;
            }
            if !seen_dirs.insert(norm_dir.to_lowercase()) {
                continue;
            }
            let game_name = dir_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&install.id)
                .to_string();
            let exe = find_executable(dir_path, 2);
            let launch_uri = Some(format!("uplay://launch/{}/0", install.id));
            detection.games.push(PlatformGame {
                name: game_name,
                platform: Platform::Ubisoft,
                platform_id: Some(install.id.clone()),
                install_dir: Some(norm_dir),
                exe,
                size_on_disk: None,
                launch_uri,
                source: Some("Ubisoft Connect registry".into()),
                console: None,
                installed: true,
                install_uri: Some(format!("uplay://install/{}", install.id)),
                ..Default::default()
            });
        }

        // 2. Uninstall registry entries where Publisher is Ubisoft or uninstall_string has uplay://
        for uninst in registry::uninstall_entries() {
            let is_ubi = uninst
                .publisher
                .as_deref()
                .map(|p| p.to_lowercase().contains("ubisoft"))
                .unwrap_or(false)
                || uninst
                    .uninstall_string
                    .as_deref()
                    .map(|u| u.to_lowercase().contains("uplay://"))
                    .unwrap_or(false);
            if !is_ubi {
                continue;
            }
            let Some(loc) = uninst.install_location else {
                continue;
            };
            let norm_dir = normalize_path(&loc);
            let dir_path = Path::new(&norm_dir);
            if !dir_path.is_dir() {
                continue;
            }
            if !seen_dirs.insert(norm_dir.to_lowercase()) {
                continue;
            }
            let game_name = uninst.display_name;
            let exe = find_executable(dir_path, 2);
            let game_id = uninst.uninstall_string.as_deref().and_then(|s| {
                if let Some(pos) = s.find("uplay://uninstall/") {
                    let rest = &s[pos + 18..];
                    Some(rest.trim_matches(['"', ' ', '\t', '/']).to_string())
                } else {
                    None
                }
            });
            let launch_uri = game_id.as_ref().map(|id| format!("uplay://launch/{id}/0"));
            let install_uri = game_id.as_ref().map(|id| format!("uplay://install/{id}"));
            detection.games.push(PlatformGame {
                name: game_name,
                platform: Platform::Ubisoft,
                platform_id: game_id,
                install_dir: Some(norm_dir),
                exe,
                size_on_disk: None,
                launch_uri,
                source: Some("Ubisoft Uninstall registry".into()),
                console: None,
                installed: true,
                install_uri,
                ..Default::default()
            });
        }

        // 3. Ubisoft Launcher root games/ folder
        if let Some(launcher_dir) = registry::ubisoft_launcher_dir() {
            let games_dir = Path::new(&launcher_dir).join("games");
            if games_dir.is_dir() {
                let games_dir_str = normalize_path(&games_dir.display().to_string());
                detection.roots.push(games_dir_str);
                if let Ok(entries) = std::fs::read_dir(&games_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if !path.is_dir() {
                            continue;
                        }
                        let norm = normalize_path(&path.display().to_string());
                        if !seen_dirs.insert(norm.to_lowercase()) {
                            continue;
                        }
                        let name = entry.file_name().to_string_lossy().to_string();
                        let exe = find_executable(&path, 2);
                        if exe.is_none() {
                            continue;
                        }
                        detection.games.push(PlatformGame {
                            name,
                            platform: Platform::Ubisoft,
                            platform_id: None,
                            install_dir: Some(norm),
                            exe,
                            size_on_disk: None,
                            launch_uri: None,
                            source: Some("Ubisoft games folder".into()),
                            console: None,
                            installed: true,
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    detection
}

// ---------------------------------------------------------------------------
// EA / Origin
// ---------------------------------------------------------------------------

fn detect_ea() -> Detection {
    #[allow(unused_mut)]
    let mut detection = Detection::empty();

    #[cfg(windows)]
    {
        use crate::platforms::registry;
        let mut seen_dirs = std::collections::BTreeSet::new();

        // 1. Registry installs: BioWare, Electronic Arts, EA Games
        let ea_installs = registry::ea_installs();
        for install in ea_installs {
            let norm_dir = normalize_path(&install.install_dir);
            let dir_path = Path::new(&norm_dir);
            if !dir_path.is_dir() {
                continue;
            }
            if !seen_dirs.insert(norm_dir.to_lowercase()) {
                continue;
            }
            let exe = find_executable(dir_path, 2);
            let game_name = dir_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&install.name)
                .to_string();
            detection.games.push(PlatformGame {
                name: game_name,
                platform: Platform::Ea,
                platform_id: None,
                install_dir: Some(norm_dir),
                exe,
                size_on_disk: None,
                launch_uri: None,
                source: Some("EA registry".into()),
                console: None,
                installed: true,
                ..Default::default()
            });
        }

        // 2. Uninstall registry entries where Publisher is Electronic Arts / EA / BioWare
        for uninst in registry::uninstall_entries() {
            let is_ea = uninst
                .publisher
                .as_deref()
                .map(|p| {
                    let low = p.to_lowercase();
                    low.contains("electronic arts") || low == "ea" || low == "bioware"
                })
                .unwrap_or(false)
                || uninst
                    .uninstall_string
                    .as_deref()
                    .map(|u| u.to_lowercase().contains("eainstaller"))
                    .unwrap_or(false);

            if !is_ea {
                continue;
            }
            let Some(loc) = uninst.install_location else {
                continue;
            };
            let norm_dir = normalize_path(&loc);
            let dir_path = Path::new(&norm_dir);
            if !dir_path.is_dir() {
                continue;
            }
            if !seen_dirs.insert(norm_dir.to_lowercase()) {
                continue;
            }
            let exe = uninst
                .display_icon
                .as_deref()
                .map(|i| i.trim_matches('"').trim())
                .filter(|i| Path::new(i).is_file())
                .map(String::from)
                .or_else(|| find_executable(dir_path, 2));

            let clean_name = uninst
                .display_name
                .replace('™', "")
                .replace('®', "")
                .trim()
                .to_string();

            detection.games.push(PlatformGame {
                name: if clean_name.is_empty() {
                    uninst.display_name
                } else {
                    clean_name
                },
                platform: Platform::Ea,
                platform_id: None,
                install_dir: Some(norm_dir),
                exe,
                size_on_disk: None,
                launch_uri: None,
                source: Some("EA Uninstall registry".into()),
                console: None,
                installed: true,
                ..Default::default()
            });
        }

        // 3. EA Desktop user_*.ini and machine.ini library roots
        for lib_dir in ea_library_dirs() {
            let norm_lib = normalize_path(&lib_dir);
            let lib_path = Path::new(&norm_lib);
            if !lib_path.is_dir() {
                continue;
            }
            detection.roots.push(norm_lib.clone());
            if let Ok(entries) = std::fs::read_dir(lib_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let norm = normalize_path(&path.display().to_string());
                    if !seen_dirs.insert(norm.to_lowercase()) {
                        continue;
                    }
                    let exe = find_executable(&path, 2);
                    if exe.is_none() {
                        continue;
                    }
                    let name = entry.file_name().to_string_lossy().to_string();
                    detection.games.push(PlatformGame {
                        name,
                        platform: Platform::Ea,
                        platform_id: None,
                        install_dir: Some(norm),
                        exe,
                        size_on_disk: None,
                        launch_uri: None,
                        source: Some("EA library folder".into()),
                        console: None,
                        installed: true,
                        ..Default::default()
                    });
                }
            }
        }
    }

    detection
}

// ---------------------------------------------------------------------------
// Battle.net
// ---------------------------------------------------------------------------

fn detect_battlenet() -> Detection {
    let mut detection = Detection::empty();
    if let Some(appdata) = env_dir("APPDATA") {
        let cfg = appdata.join("Battle.net").join("Battle.net.config");
        if let Ok(text) = std::fs::read_to_string(cfg) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(path_str) = val
                    .pointer("/Client/Install/DefaultInstallPath")
                    .and_then(|v| v.as_str())
                {
                    let norm = normalize_path(path_str);
                    let p = Path::new(&norm);
                    if p.is_dir() {
                        detection.roots.push(norm.clone());
                        if let Ok(entries) = std::fs::read_dir(p) {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if !path.is_dir() {
                                    continue;
                                }
                                if let Some(exe) = find_executable(&path, 2) {
                                    let name = entry.file_name().to_string_lossy().to_string();
                                    detection.games.push(PlatformGame {
                                        name,
                                        platform: Platform::BattleNet,
                                        platform_id: None,
                                        install_dir: Some(normalize_path(&path.display().to_string())),
                                        exe: Some(exe),
                                        size_on_disk: None,
                                        launch_uri: None,
                                        source: Some("Battle.net folder".into()),
                                        console: None,
                                        installed: true,
                                        ..Default::default()
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    detection
}

// ---------------------------------------------------------------------------
// Rockstar Games
// ---------------------------------------------------------------------------

fn detect_rockstar() -> Detection {
    #[allow(unused_mut)]
    let mut detection = Detection::empty();
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        for view in [KEY_WOW64_32KEY, KEY_WOW64_64KEY] {
            if let Ok(key) =
                hklm.open_subkey_with_flags("SOFTWARE\\Rockstar Games", KEY_READ | view)
            {
                for sub_name in key.enum_keys().flatten() {
                    if let Ok(sub) = key.open_subkey(&sub_name) {
                        if let Ok(install_folder) =
                            sub.get_value::<String, _>("InstallFolder")
                        {
                            let norm = normalize_path(&install_folder);
                            let p = Path::new(&norm);
                            if p.is_dir() {
                                let exe = find_executable(p, 2);
                                detection.games.push(PlatformGame {
                                    name: sub_name,
                                    platform: Platform::Rockstar,
                                    platform_id: None,
                                    install_dir: Some(norm),
                                    exe,
                                    size_on_disk: None,
                                    launch_uri: None,
                                    source: Some("Rockstar registry".into()),
                                    console: None,
                                    installed: true,
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    detection
}

// ---------------------------------------------------------------------------
// Amazon Games
// ---------------------------------------------------------------------------

fn detect_amazon() -> Detection {
    let mut detection = Detection::empty();

    if let Some(local) = env_dir("LOCALAPPDATA") {
        let dir = local.join("Amazon Games").join("Library");
        if dir.is_dir() {
            detection.roots.push(dir.display().to_string());
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    detection.games.push(PlatformGame {
                        name: entry.file_name().to_string_lossy().to_string(),
                        platform: Platform::Amazon,
                        platform_id: None,
                        install_dir: Some(path.display().to_string()),
                        exe: find_executable(&path, 3),
                        size_on_disk: None,
                        launch_uri: None,
                        source: Some("Amazon Games library".into()),
                        console: None,
                        installed: true,
                        ..Default::default()
                    });
                }
            }
        }
    }

    detection
}

// ---------------------------------------------------------------------------
// Microsoft Store / Xbox Game Pass
// ---------------------------------------------------------------------------

struct XboxMeta {
    name: Option<String>,
    exe: Option<String>,
    id: Option<String>,
    launch_uri: Option<String>,
}

fn unescape_xml(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

pub(crate) fn extract_package_family_name(full_name: &str) -> Option<String> {
    let parts: Vec<&str> = full_name.split('_').collect();
    if parts.len() >= 5 {
        let name = parts.first()?;
        let pub_id = parts.last()?;
        if !name.is_empty() && !pub_id.is_empty() {
            return Some(format!("{name}_{pub_id}"));
        }
    }
    None
}

#[cfg(windows)]
fn scan_xbox_registry(detection: &mut Detection) {
    use winreg::enums::*;
    use winreg::RegKey;

    let subkey_path = "SOFTWARE\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\AppModel\\Repository\\Packages";
    let hives = [
        RegKey::predef(HKEY_CURRENT_USER),
        RegKey::predef(HKEY_LOCAL_MACHINE),
    ];

    let mut seen_packages = std::collections::HashSet::new();

    for hive in &hives {
        let Ok(packages_key) = hive.open_subkey_with_flags(subkey_path, KEY_READ) else {
            continue;
        };

        for pkg_name in packages_key.enum_keys().flatten() {
            if !seen_packages.insert(pkg_name.clone()) {
                continue;
            }

            let Ok(pkg_key) = packages_key.open_subkey_with_flags(&pkg_name, KEY_READ) else {
                continue;
            };

            let root_str: String = match pkg_key.get_value("PackageRootFolder") {
                Ok(r) => r,
                Err(_) => continue,
            };

            let root_path = Path::new(&root_str);
            if !root_path.is_dir() {
                continue;
            }

            let has_mgc = root_path.join("MicrosoftGame.config").is_file();
            let has_xbox_cfg = root_path.join("xboxservices.config").is_file()
                || root_path.join("XboxServices.config").is_file();

            let manifest_path = root_path.join("AppxManifest.xml");
            let mut manifest_content = None;
            let mut manifest_is_game = false;

            if manifest_path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                    if content.contains("xboxliveapp")
                        || content.contains("ms-xbl-")
                        || content.contains("TargetDeviceFamily Name=\"Windows.Xbox\"")
                        || content.contains("SubModel=\"Game\"")
                        || content.contains("Category=\"windows.game\"")
                    {
                        manifest_is_game = true;
                    }
                    manifest_content = Some(content);
                }
            }

            let mut has_xbox_protocol = false;
            if let Ok(app_key) =
                pkg_key.open_subkey_with_flags("App\\Capabilities\\URLAssociations", KEY_READ)
            {
                for (name, _) in app_key.enum_values().flatten() {
                    let lower = name.to_lowercase();
                    if lower.contains("xbox") {
                        has_xbox_protocol = true;
                        break;
                    }
                }
            }

            if !has_mgc && !has_xbox_cfg && !manifest_is_game && !has_xbox_protocol {
                continue;
            }

            if detection
                .games
                .iter()
                .any(|g| g.install_dir.as_deref() == Some(&root_str))
            {
                continue;
            }

            let mut meta = None;
            if has_mgc {
                if let Ok(xml) = std::fs::read_to_string(root_path.join("MicrosoftGame.config")) {
                    meta = parse_microsoft_game_config(&xml, root_path);
                }
            }
            if meta.is_none() {
                if let Some(ref xml) = manifest_content {
                    meta = parse_appx_manifest(xml, root_path);
                }
            }

            let reg_display_name: Option<String> = pkg_key
                .get_value("DisplayName")
                .ok()
                .map(|s: String| s.trim().to_string())
                .filter(|s| !s.is_empty() && !s.starts_with('@'));

            let mut game_name = reg_display_name
                .or_else(|| meta.as_ref().and_then(|m| m.name.clone()))
                .unwrap_or_else(|| {
                    pkg_name
                        .split('_')
                        .next()
                        .unwrap_or(&pkg_name)
                        .to_string()
                });

            game_name = unescape_xml(game_name.trim());

            let family_name = extract_package_family_name(&pkg_name)
                .or_else(|| meta.as_ref().and_then(|m| m.id.clone()));

            let launch_uri = meta
                .as_ref()
                .and_then(|m| m.launch_uri.clone())
                .or_else(|| family_name.as_ref().map(|fam| format!("shell:AppsFolder\\{fam}!App")));

            let exe_path = meta.and_then(|m| m.exe).or_else(|| find_executable(root_path, 2));

            let root_dir_str = root_path
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| root_str.clone());
            if !detection.roots.contains(&root_dir_str) {
                detection.roots.push(root_dir_str);
            }

            detection.games.push(PlatformGame {
                name: game_name,
                platform: Platform::Xbox,
                platform_id: family_name,
                install_dir: Some(root_str),
                exe: exe_path,
                size_on_disk: None,
                launch_uri,
                source: Some("Xbox Game Pass / Microsoft Store".into()),
                console: None,
                installed: true,
                ..Default::default()
            });
        }
    }
}

fn detect_xbox(mounted_drives: &[PathBuf]) -> Detection {
    let mut detection = Detection::empty();

    #[cfg(windows)]
    scan_xbox_registry(&mut detection);

    for drive in mounted_drives {
        for sub in &[
            "XboxGames",
            "Games\\XboxGames",
            "Program Files\\WindowsApps",
            "WindowsApps",
        ] {
            let candidate = drive.join(sub);
            if candidate.is_dir() {
                scan_xbox_dir(&candidate, &mut detection);
            }
        }
    }

    if detection.roots.is_empty() && detection.games.is_empty() {
        detection.note = Some("No XboxGames library folders found on mounted drives.".into());
    }

    detection
}

fn scan_xbox_dir(dir: &Path, detection: &mut Detection) {
    let root_str = dir.display().to_string();
    if !detection.roots.contains(&root_str) {
        detection.roots.push(root_str);
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let folder_name = entry.file_name().to_string_lossy().to_string();
        if folder_name.starts_with('.') || folder_name.eq_ignore_ascii_case("GameSave") {
            continue;
        }

        let content_dir = path.join("Content");
        let search_dirs = if content_dir.is_dir() {
            vec![content_dir, path.clone()]
        } else {
            vec![path.clone()]
        };

        let mut game_name = folder_name.clone();
        let mut exe_path = None;
        let mut platform_id = None;
        let mut launch_uri = None;
        let mut recognized = false;

        for s_dir in &search_dirs {
            let config_path = s_dir.join("MicrosoftGame.config");
            if config_path.is_file() {
                if let Ok(xml) = std::fs::read_to_string(&config_path) {
                    if let Some(meta) = parse_microsoft_game_config(&xml, s_dir) {
                        if let Some(n) = meta.name {
                            game_name = n;
                        }
                        if let Some(e) = meta.exe {
                            exe_path = Some(e);
                        }
                        platform_id = meta.id;
                        launch_uri = meta.launch_uri;
                        recognized = true;
                        break;
                    }
                }
            }

            let has_xbox_svc = s_dir.join("xboxservices.config").is_file()
                || s_dir.join("XboxServices.config").is_file();

            let manifest_path = s_dir.join("appxmanifest.xml");
            if manifest_path.is_file() {
                if let Ok(xml) = std::fs::read_to_string(&manifest_path) {
                    let has_xbox_marker = xml.contains("xboxliveapp")
                        || xml.contains("ms-xbl-")
                        || xml.contains("TargetDeviceFamily Name=\"Windows.Xbox\"")
                        || xml.contains("Category=\"windows.game\"")
                        || xml.contains("SubModel=\"Game\"");

                    if has_xbox_svc || has_xbox_marker {
                        if let Some(meta) = parse_appx_manifest(&xml, s_dir) {
                            if let Some(n) = meta.name {
                                game_name = n;
                            }
                            if let Some(e) = meta.exe {
                                exe_path = Some(e);
                            }
                            platform_id = meta.id;
                            launch_uri = meta.launch_uri;
                            recognized = true;
                            break;
                        }
                    }
                }
            }
        }

        // If scanning a dedicated XboxGames directory, assume candidate game
        let parent_name = dir.file_name().and_then(|f| f.to_str()).unwrap_or("");
        if !recognized && !parent_name.eq_ignore_ascii_case("XboxGames") {
            continue;
        }

        let install_str = path.display().to_string();
        if detection
            .games
            .iter()
            .any(|g| g.install_dir.as_deref() == Some(&install_str))
        {
            continue;
        }

        if exe_path.is_none() {
            exe_path = find_executable(&path, 3);
        }

        detection.games.push(PlatformGame {
            name: unescape_xml(&game_name),
            platform: Platform::Xbox,
            platform_id,
            install_dir: Some(install_str),
            exe: exe_path,
            size_on_disk: None,
            launch_uri,
            source: Some("Xbox Game Pass / Microsoft Store".into()),
            console: None,
            installed: true,
            ..Default::default()
        });
    }
}

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open_tag = format!("<{tag}>");
    let close_tag = format!("</{tag}>");
    let start = xml.find(&open_tag)? + open_tag.len();
    let end = xml[start..].find(&close_tag)?;
    let val = xml[start..start + end].trim();
    if val.is_empty() {
        None
    } else {
        Some(val.to_string())
    }
}

fn extract_xml_attr(tag_content: &str, attr: &str) -> Option<String> {
    let pattern = format!("{attr}=\"");
    let start = tag_content.find(&pattern)? + pattern.len();
    let end = tag_content[start..].find('"')?;
    let val = tag_content[start..start + end].trim();
    if val.is_empty() {
        None
    } else {
        Some(val.to_string())
    }
}

fn parse_microsoft_game_config(xml: &str, dir: &Path) -> Option<XboxMeta> {
    let name = extract_xml_tag(xml, "DisplayName")
        .or_else(|| extract_xml_tag(xml, "IdForDisplay"))
        .map(|s| unescape_xml(&s));

    let id = extract_xml_tag(xml, "TitleId").or_else(|| {
        let start = xml.find("<Identity ")?;
        let end = xml[start..].find('>')?;
        extract_xml_attr(&xml[start..start + end], "Name")
    });

    let launch_uri = id
        .as_ref()
        .map(|id_str| format!("shell:AppsFolder\\{id_str}!App"));

    let exe = if let Some(start) = xml.find("<Executable ") {
        let end = xml[start..].find('>')?;
        let exe_name = extract_xml_attr(&xml[start..start + end], "Name")?;
        let joined = dir.join(&exe_name);
        if joined.is_file() {
            Some(joined.display().to_string())
        } else {
            Some(exe_name)
        }
    } else {
        find_executable(dir, 2)
    };

    Some(XboxMeta {
        name,
        exe,
        id,
        launch_uri,
    })
}

fn parse_appx_manifest(xml: &str, dir: &Path) -> Option<XboxMeta> {
    let name = extract_xml_tag(xml, "DisplayName")
        .or_else(|| {
            let start = xml.find("<uap:VisualElements ")?;
            let end = xml[start..].find('>')?;
            extract_xml_attr(&xml[start..start + end], "DisplayName")
        })
        .or_else(|| {
            let start = xml.find("<VisualElements ")?;
            let end = xml[start..].find('>')?;
            extract_xml_attr(&xml[start..start + end], "DisplayName")
        })
        .map(|s| unescape_xml(&s));

    let app_id = if let Some(start) = xml.find("<Application ") {
        let end = xml[start..].find('>')?;
        extract_xml_attr(&xml[start..start + end], "Id")
    } else {
        None
    }
    .unwrap_or_else(|| "App".to_string());

    let identity_name = if let Some(start) = xml.find("<Identity ") {
        let end = xml[start..].find('>')?;
        extract_xml_attr(&xml[start..start + end], "Name")
    } else {
        None
    };

    let dir_name = dir.file_name().and_then(|f| f.to_str()).unwrap_or("");
    let family_name = extract_package_family_name(dir_name).or(identity_name);

    let launch_uri = family_name
        .as_ref()
        .map(|fam| format!("shell:AppsFolder\\{fam}!{app_id}"));

    let exe = if let Some(start) = xml.find("<Application ") {
        let end = xml[start..].find('>')?;
        let exe_name = extract_xml_attr(&xml[start..start + end], "Executable")?;
        let joined = dir.join(&exe_name);
        if joined.is_file() {
            Some(joined.display().to_string())
        } else {
            Some(exe_name)
        }
    } else {
        find_executable(dir, 2)
    };

    Some(XboxMeta {
        name,
        exe,
        id: family_name,
        launch_uri,
    })
}

// ---------------------------------------------------------------------------
// Emulators & ROMs
// ---------------------------------------------------------------------------

struct EmulatorDef {
    name: &'static str,
    exes: &'static [&'static str],
    dirs: &'static [&'static str],
    rom_exts: &'static [&'static str],
    launch_arg: fn(&str) -> String,
}

const KNOWN_EMULATORS: &[EmulatorDef] = &[
    EmulatorDef {
        name: "Dolphin (GameCube/Wii)",
        exes: &["Dolphin.exe"],
        dirs: &["Dolphin", "Dolphin Emulator", "Dolphin-x64"],
        rom_exts: &["iso", "gcm", "gcz", "ciso", "wbfs", "rvz", "wad"],
        launch_arg: |p| format!("-b -e \"{p}\""),
    },
    EmulatorDef {
        name: "Project64 / Mupen64 (N64)",
        exes: &["Project64.exe", "mupen64plus.exe", "simple64-gui.exe"],
        dirs: &["Project64", "Mupen64Plus", "simple64"],
        rom_exts: &["z64", "n64", "v64"],
        launch_arg: |p| format!("\"{p}\""),
    },
    EmulatorDef {
        name: "PCSX2 (PlayStation 2)",
        exes: &["pcsx2-qtx64.exe", "pcsx2.exe"],
        dirs: &["PCSX2", "PCSX2 2.0"],
        rom_exts: &["iso", "bin", "chd", "cso", "gz"],
        launch_arg: |p| format!("-batch -nogui \"{p}\""),
    },
    EmulatorDef {
        name: "DuckStation (PlayStation 1)",
        exes: &[
            "duckstation-qt-x64-ReleaseLTCG.exe",
            "duckstation-nogui.exe",
            "duckstation.exe",
        ],
        dirs: &["DuckStation"],
        rom_exts: &["chd", "cue", "pbp"],
        launch_arg: |p| format!("-batch \"{p}\""),
    },
    EmulatorDef {
        name: "PPSSPP (PSP)",
        exes: &["PPSSPPWindows64.exe", "PPSSPPWindows.exe", "PPSSPP.exe"],
        dirs: &["PPSSPP"],
        rom_exts: &["cso", "pbp", "iso"],
        launch_arg: |p| format!("\"{p}\""),
    },
    EmulatorDef {
        name: "Citra / Lime3DS (3DS)",
        exes: &["citra-qt.exe", "lime-qt.exe", "citra.exe"],
        dirs: &["Citra", "Lime3DS"],
        rom_exts: &["3ds", "cia", "cxi"],
        launch_arg: |p| format!("\"{p}\""),
    },
    EmulatorDef {
        name: "Cemu (Wii U)",
        exes: &["Cemu.exe"],
        dirs: &["Cemu"],
        rom_exts: &["rpx", "wua", "wud", "wux"],
        launch_arg: |p| format!("-f -g \"{p}\""),
    },
    EmulatorDef {
        name: "Ryujinx (Switch)",
        exes: &["Ryujinx.exe", "Ryujinx.Ava.exe"],
        dirs: &["Ryujinx"],
        rom_exts: &["nsp", "xci"],
        launch_arg: |p| format!("\"{p}\""),
    },
    EmulatorDef {
        name: "mGBA (Game Boy Advance)",
        exes: &["mGBA.exe"],
        dirs: &["mGBA"],
        rom_exts: &["gba", "gbc", "gb"],
        launch_arg: |p| format!("\"{p}\""),
    },
    EmulatorDef {
        name: "Snes9x (SNES)",
        exes: &["snes9x-x64.exe", "snes9x.exe"],
        dirs: &["Snes9x"],
        rom_exts: &["sfc", "smc"],
        launch_arg: |p| format!("\"{p}\""),
    },
    EmulatorDef {
        name: "RetroArch",
        exes: &["retroarch.exe"],
        dirs: &["RetroArch", "RetroArch-Win64"],
        rom_exts: &[
            "nes", "sfc", "smc", "z64", "n64", "gba", "gbc", "gb", "iso", "chd", "cue",
        ],
        launch_arg: |p| format!("\"{p}\""),
    },
];

/// Map explicitly configured emulators to their executable paths.
fn map_configured_emulators(configs: &[EmulatorConfig]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for emu in configs {
        let name = emu.name.trim();
        let exe = emu.executable_path.trim();
        if !name.is_empty() && !exe.is_empty() {
            map.insert(name.to_string(), exe.to_string());
        }
    }
    map
}

/// Detect emulated ROMs strictly from user-configured emulators and folders.
///
/// Automatic detection of emulator installation locations or filesystem ROM scanning
/// is deliberately disabled to protect user privacy and respect anti-piracy policies.
fn detect_user_emulators(configs: &[EmulatorConfig], extra_roots: &[String]) -> Detection {
    let mut detection = Detection::empty();
    if configs.is_empty() && extra_roots.is_empty() {
        detection.note = Some(
            "Emulated games are not auto-detected. Add your emulator and ROM folder in Settings.".into(),
        );
        return detection;
    }

    let emu_map = map_configured_emulators(configs);

    // Scan ROM directories configured per emulator
    for emu in configs {
        for rom_dir in &emu.rom_directories {
            let path = Path::new(rom_dir);
            if path.is_dir() {
                scan_emulator_dir(path, &mut detection, &emu_map);
            }
        }
    }

    // Scan any additional ROM directories configured by the user
    for extra in extra_roots {
        let path = Path::new(extra);
        if path.is_dir() {
            scan_emulator_dir(path, &mut detection, &emu_map);
        }
    }

    if detection.games.is_empty() && detection.roots.is_empty() {
        detection.note = Some(
            "No ROM files found in the configured emulator directories.".into(),
        );
    }

    detection
}

/// Determine the specific console platform (e.g. GameCube, Wii, PlayStation, etc.) for a ROM file.
///
/// Supports EmulationStation Desktop Edition (ES-DE) folder hierarchy (`ROMs/<system>/...`)
/// as well as standard file extension heuristics.
pub fn determine_console_platform(path: &Path, emu_name: Option<&str>) -> Option<String> {
    // 1. EmulationStation (ES-DE) directory match
    if let Some(parent) = path.parent() {
        let parent_name = parent
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        match parent_name.as_str() {
            "nes" | "famicom" => return Some("NES".to_string()),
            "snes" | "sfc" => return Some("Super Nintendo".to_string()),
            "n64" | "n64dd" => return Some("Nintendo 64".to_string()),
            "gc" | "gamecube" => return Some("GameCube".to_string()),
            "wii" => return Some("Wii".to_string()),
            "wiiu" => return Some("Wii U".to_string()),
            "switch" => return Some("Nintendo Switch".to_string()),
            "gb" => return Some("Game Boy".to_string()),
            "gbc" => return Some("Game Boy Color".to_string()),
            "gba" => return Some("Game Boy Advance".to_string()),
            "nds" => return Some("Nintendo DS".to_string()),
            "n3ds" => return Some("Nintendo 3DS".to_string()),
            "virtualboy" => return Some("Virtual Boy".to_string()),
            "gameandwatch" => return Some("Game & Watch".to_string()),
            "psx" | "ps1" => return Some("PlayStation".to_string()),
            "ps2" => return Some("PlayStation 2".to_string()),
            "ps3" => return Some("PlayStation 3".to_string()),
            "psp" => return Some("PlayStation Portable".to_string()),
            "psvita" => return Some("PlayStation Vita".to_string()),
            "mastersystem" => return Some("Sega Master System".to_string()),
            "genesis" | "megadrive" => return Some("Sega Genesis".to_string()),
            "segacd" | "megacd" => return Some("Sega CD".to_string()),
            "sega32x" => return Some("Sega 32X".to_string()),
            "saturn" => return Some("Sega Saturn".to_string()),
            "dreamcast" => return Some("Dreamcast".to_string()),
            "gamegear" => return Some("Sega Game Gear".to_string()),
            "arcade" | "mame" | "fbneo" => return Some("Arcade".to_string()),
            "neogeo" => return Some("Neo Geo".to_string()),
            "ngp" => return Some("Neo Geo Pocket".to_string()),
            "ngpc" => return Some("Neo Geo Pocket Color".to_string()),
            "wonderswan" => return Some("WonderSwan".to_string()),
            "wonderswancolor" => return Some("WonderSwan Color".to_string()),
            "pcengine" | "tg16" => return Some("TurboGrafx-16".to_string()),
            "atari2600" => return Some("Atari 2600".to_string()),
            "atari5200" => return Some("Atari 5200".to_string()),
            "atari7800" => return Some("Atari 7800".to_string()),
            "atarijaguar" => return Some("Atari Jaguar".to_string()),
            "atarilynx" => return Some("Atari Lynx".to_string()),
            "atarist" => return Some("Atari ST".to_string()),
            "3do" => return Some("3DO".to_string()),
            "colecovision" => return Some("ColecoVision".to_string()),
            "intellivision" => return Some("Intellivision".to_string()),
            "msx" | "msx1" => return Some("MSX".to_string()),
            "msx2" | "msxturbor" => return Some("MSX2".to_string()),
            "c64" => return Some("Commodore 64".to_string()),
            "amiga" | "amiga1200" | "amiga600" => return Some("Amiga".to_string()),
            "dos" => return Some("DOS".to_string()),
            _ => {}
        }
    }

    let ext = path
        .extension()
        .and_then(|s| s.to_str())?
        .to_ascii_lowercase();
    let p_str = path.to_string_lossy().to_ascii_lowercase();
    let emu_lower = emu_name.unwrap_or("").to_ascii_lowercase();

    match ext.as_str() {
        "gcm" => Some("GameCube".to_string()),
        "rvz" | "wbfs" | "wad" | "ciso" => Some("Wii".to_string()),
        "z64" | "n64" | "v64" => Some("Nintendo 64".to_string()),
        "sfc" | "smc" => Some("Super Nintendo".to_string()),
        "nes" => Some("NES".to_string()),
        "gba" => Some("Game Boy Advance".to_string()),
        "gbc" => Some("Game Boy Color".to_string()),
        "gb" => Some("Game Boy".to_string()),
        "nds" => Some("Nintendo DS".to_string()),
        "3ds" | "cia" | "cxi" => Some("Nintendo 3DS".to_string()),
        "nsp" | "xci" => Some("Nintendo Switch".to_string()),
        "rpx" | "wua" | "wud" | "wux" => Some("Wii U".to_string()),
        "cso" => Some("PlayStation Portable".to_string()),
        "pbp" => {
            if p_str.contains("psp") || emu_lower.contains("psp") {
                Some("PlayStation Portable".to_string())
            } else {
                Some("PlayStation".to_string())
            }
        }
        "md" | "gen" | "smd" => Some("Sega Genesis".to_string()),
        "gg" => Some("Sega Game Gear".to_string()),
        "sms" => Some("Sega Master System".to_string()),
        "a26" => Some("Atari 2600".to_string()),
        "iso" => {
            if emu_lower.contains("dolphin")
                || p_str.contains("dolphin")
                || p_str.contains("gamecube")
                || p_str.contains("wii")
            {
                if p_str.contains("gamecube") || p_str.contains("gcn") {
                    Some("GameCube".to_string())
                } else if p_str.contains("wii") {
                    Some("Wii".to_string())
                } else {
                    let file_size = std::fs::metadata(path).ok().map(|m| m.len()).unwrap_or(0);
                    if file_size > 0 && file_size <= 1_500_000_000 {
                        Some("GameCube".to_string())
                    } else {
                        Some("Wii".to_string())
                    }
                }
            } else if emu_lower.contains("pcsx2")
                || p_str.contains("pcsx2")
                || p_str.contains("ps2")
            {
                Some("PlayStation 2".to_string())
            } else if emu_lower.contains("duckstation")
                || p_str.contains("duckstation")
                || p_str.contains("ps1")
                || p_str.contains("psx")
            {
                Some("PlayStation".to_string())
            } else if emu_lower.contains("ppsspp") || p_str.contains("psp") {
                Some("PlayStation Portable".to_string())
            } else {
                None
            }
        }
        "chd" | "cue" | "bin" => {
            if emu_lower.contains("pcsx2") || p_str.contains("ps2") {
                Some("PlayStation 2".to_string())
            } else if emu_lower.contains("duckstation")
                || p_str.contains("duckstation")
                || p_str.contains("ps1")
                || p_str.contains("psx")
            {
                Some("PlayStation".to_string())
            } else if p_str.contains("dreamcast") {
                Some("Dreamcast".to_string())
            } else if p_str.contains("saturn") {
                Some("Sega Saturn".to_string())
            } else if p_str.contains("genesis") || p_str.contains("megadrive") {
                Some("Sega Genesis".to_string())
            } else if emu_lower.contains("pcsx2") {
                Some("PlayStation 2".to_string())
            } else if emu_lower.contains("duckstation") {
                Some("PlayStation".to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Recursively scan a directory for ROM files up to depth 4.
fn scan_emulator_dir(
    dir: &Path,
    detection: &mut Detection,
    emu_map: &BTreeMap<String, String>,
) {
    let root_str = dir.display().to_string();
    if !detection.roots.contains(&root_str) {
        detection.roots.push(root_str);
    }
    scan_roms_recursive(dir, 0, 4, detection, emu_map);
}

fn scan_roms_recursive(
    current: &Path,
    depth: usize,
    max_depth: usize,
    detection: &mut Detection,
    emu_map: &BTreeMap<String, String>,
) {
    if depth > max_depth {
        return;
    }
    let Ok(entries) = std::fs::read_dir(current) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let folder_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let f_lower = folder_name.to_lowercase();
            if folder_name.starts_with('.')
                || f_lower == "sys"
                || f_lower == "system"
                || f_lower == "assets"
                || f_lower == "shaders"
                || f_lower == "bios"
                || f_lower == "plugins"
                || f_lower == "lang"
                || f_lower == "languages"
                || f_lower == "docs"
                || f_lower == "user"
                || f_lower == "config"
                || f_lower == "paks"
                || f_lower == "themes"
                || f_lower == "dump"
                || f_lower == "load"
                || f_lower == "profiles"
                || f_lower == "cache"
                || f_lower == "temp"
                || f_lower == "tmp"
                || f_lower == "logs"
            {
                continue;
            }
            scan_roms_recursive(&path, depth + 1, max_depth, detection, emu_map);
            continue;
        }

        let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
            continue;
        };
        let ext_lower = ext.to_ascii_lowercase();

        let raw_name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let name_lower = raw_name.to_lowercase();

        // Skip known system/firmware/internal tools
        if name_lower.starts_with("codehandler")
            || name_lower.starts_with("dsp_coef")
            || name_lower.starts_with("dsp_rom")
            || name_lower.starts_with("font")
            || name_lower.starts_with("romfont")
            || name_lower.starts_with("scph")
            || name_lower.starts_with("bios")
            || name_lower.starts_with("firmware")
            || name_lower.starts_with("keys")
            || name_lower.starts_with("titlekey")
            || name_lower == "misc"
        {
            continue;
        }

        // For .bin files, require size >= 10 MB (disc image vs small internal binaries)
        let file_len = entry.metadata().ok().map(|m| m.len()).unwrap_or(0);
        if ext_lower == "bin" && file_len < 10 * 1024 * 1024 {
            continue;
        }

        // Check if file is a known ROM extension
        let matched_emu = KNOWN_EMULATORS.iter().find(|emu| {
            emu.rom_exts
                .iter()
                .any(|e| e.eq_ignore_ascii_case(&ext_lower))
        });

        let Some(emu) = matched_emu else {
            continue;
        };

        let name = crate::metadata::vndb::clean_search_term(&raw_name);
        if name.is_empty() {
            continue;
        }

        let rom_path_str = path.display().to_string();
        let size = Some(file_len);

        let emu_name_lower = emu.name.to_lowercase();
        let configured_match = emu_map.iter().find_map(|(k, v)| {
            let k_low = k.to_lowercase();
            if k_low == emu_name_lower || emu_name_lower.contains(&k_low) || k_low.contains(&emu_name_lower) {
                Some((k.as_str(), v.clone()))
            } else {
                None
            }
        });

        let (emu_name, exe, launch_uri) = match configured_match {
            Some((display_name, exe_path)) => {
                let launch_options = (emu.launch_arg)(&rom_path_str);
                (display_name, Some(exe_path), Some(launch_options))
            }
            None => (
                "ROM (configure emulator in Settings)",
                None,
                Some(format!("\"{rom_path_str}\"")),
            ),
        };

        let console = determine_console_platform(&path, Some(emu.name));
        let display_source = match &console {
            Some(c) => format!("{emu_name} · {c}"),
            None => emu_name.to_string(),
        };

        detection.games.push(PlatformGame {
            name,
            platform: Platform::Emulator,
            platform_id: Some(raw_name),
            install_dir: path.parent().map(|p| p.display().to_string()),
            exe,
            size_on_disk: size,
            launch_uri,
            source: Some(display_source),
            console,
            installed: true,
            ..Default::default()
        });
    }
}

// ---------------------------------------------------------------------------
// Registry-based (Ubisoft, EA, Battle.net, Rockstar, and other studios)
// ---------------------------------------------------------------------------

#[cfg(windows)]
fn detect_registry_based() -> Vec<(Platform, Detection)> {
    use crate::platforms::registry;

    let mut buckets: BTreeMap<Platform, Detection> = BTreeMap::new();
    for entry in registry::uninstall_entries() {
        let Some(publisher) = entry.publisher.as_deref() else {
            continue;
        };
        let Some(platform) = platform_for_publisher(publisher) else {
            continue;
        };
        let exe = entry.display_icon.as_deref().and_then(icon_executable);
        if !is_game_candidate(
            &entry.display_name,
            exe.as_deref(),
            entry.install_location.as_deref(),
        ) {
            continue;
        }
        let bucket = buckets.entry(platform).or_insert_with(Detection::empty);
        if bucket.roots.is_empty() {
            bucket.roots.push("Uninstall registry".into());
        }
        bucket.games.push(PlatformGame {
            name: entry.display_name,
            platform,
            platform_id: None,
            install_dir: entry.install_location,
            exe,
            size_on_disk: None,
            launch_uri: None,
            source: Some(format!("Uninstall registry · {publisher}")),
            console: None,
            installed: true,
            ..Default::default()
        });
    }
    buckets.into_iter().collect()
}

#[cfg(not(windows))]
fn detect_registry_based() -> Vec<(Platform, Detection)> {
    Vec::new()
}

/// Map an uninstall `Publisher` to a platform, or `None` if unrecognised.
pub(crate) fn platform_for_publisher(publisher: &str) -> Option<Platform> {
    let p = publisher.to_lowercase();
    if p.contains("gog") {
        Some(Platform::Gog)
    } else if p.contains("epic games") {
        Some(Platform::Epic)
    } else if p.contains("ubisoft") {
        Some(Platform::Ubisoft)
    } else if p.contains("electronic arts") || p.contains("ea games") {
        Some(Platform::Ea)
    } else if p.contains("blizzard") {
        Some(Platform::BattleNet)
    } else if p.contains("rockstar") {
        Some(Platform::Rockstar)
    } else if p.contains("amazon") {
        Some(Platform::Amazon)
    } else if [
        "bethesda",
        "riot games",
        "wargaming",
        "cd projekt",
        "square enix",
        "bandai namco",
        "sega",
        "take-two",
        "paradox",
        "thq nordic",
    ]
    .iter()
    .any(|needle| p.contains(needle))
    {
        Some(Platform::Other)
    } else {
        None
    }
}

/// Turn a `DisplayIcon` value (`"C:\game\game.exe",0`) into an executable path.
pub(crate) fn icon_executable(icon: &str) -> Option<String> {
    let first = icon.split(',').next().unwrap_or(icon).trim();
    let path = first.trim_matches('"');
    if path.to_lowercase().ends_with(".exe") {
        Some(path.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_microsoft_game_config() {
        let xml = r#"
            <Game configVersion="1">
                <Identity Name="SampleStudio.SampleGame" Publisher="CN=Sample" Version="1.0.0.0"/>
                <TitleId>12345678</TitleId>
                <ExecutableList>
                    <Executable Name="SampleGame.exe" TargetDeviceFamily="PC"/>
                </ExecutableList>
                <ShellVisuals DefaultDisplayName="Sample Game"/>
                <DisplayName>Sample Game</DisplayName>
            </Game>
        "#;
        let meta = parse_microsoft_game_config(xml, Path::new("C:\\Games\\Sample")).unwrap();
        assert_eq!(meta.name.as_deref(), Some("Sample Game"));
        assert_eq!(meta.id.as_deref(), Some("12345678"));
        assert_eq!(
            meta.launch_uri.as_deref(),
            Some("shell:AppsFolder\\12345678!App")
        );
        assert_eq!(meta.exe.as_deref(), Some("SampleGame.exe"));
    }

    #[test]
    fn parses_appx_manifest() {
        let xml = r#"
            <Package>
                <Identity Name="Microsoft.MinecraftUWP" Publisher="CN=Microsoft" Version="1.19.0.0"/>
                <Properties>
                    <DisplayName>Minecraft for Windows</DisplayName>
                </Properties>
                <Applications>
                    <Application Id="App" Executable="Minecraft.Windows.exe">
                        <VisualElements DisplayName="Minecraft for Windows"/>
                    </Application>
                </Applications>
            </Package>
        "#;
        let meta = parse_appx_manifest(xml, Path::new("C:\\Games\\Minecraft")).unwrap();
        assert_eq!(meta.name.as_deref(), Some("Minecraft for Windows"));
        assert_eq!(meta.id.as_deref(), Some("Microsoft.MinecraftUWP"));
        assert_eq!(
            meta.launch_uri.as_deref(),
            Some("shell:AppsFolder\\Microsoft.MinecraftUWP!App")
        );
        assert_eq!(meta.exe.as_deref(), Some("Minecraft.Windows.exe"));
    }

    #[test]
    fn extracts_package_family_name_and_unescapes_xml() {
        assert_eq!(
            extract_package_family_name("Microsoft.MicrosoftSolitaireCollection_4.27.9181.0_x64__8wekyb3d8bbwe"),
            Some("Microsoft.MicrosoftSolitaireCollection_8wekyb3d8bbwe".to_string())
        );
        assert_eq!(
            extract_package_family_name("Microsoft.Studios.Wordament_4.7.7281.0_x64__8wekyb3d8bbwe"),
            Some("Microsoft.Studios.Wordament_8wekyb3d8bbwe".to_string())
        );
        assert_eq!(
            extract_package_family_name("Microsoft.MicrosoftMahjong_4.9.9090.0_x64__8wekyb3d8bbwe"),
            Some("Microsoft.MicrosoftMahjong_8wekyb3d8bbwe".to_string())
        );
        assert_eq!(unescape_xml("Solitaire &amp; Casual Games"), "Solitaire & Casual Games");

        let xml = r#"
            <Package>
                <Identity Name="Microsoft.MicrosoftSolitaireCollection" Publisher="CN=Microsoft Corporation" Version="4.27.9181.0"/>
                <Properties>
                    <DisplayName>Solitaire &amp; Casual Games</DisplayName>
                </Properties>
                <Applications>
                    <Application Id="App" Executable="Solitaire.exe">
                        <uap:VisualElements DisplayName="Solitaire &amp; Casual Games"/>
                    </Application>
                </Applications>
            </Package>
        "#;
        let meta = parse_appx_manifest(
            xml,
            Path::new("C:\\MockApps\\Microsoft.MicrosoftSolitaireCollection_4.27.9181.0_x64__8wekyb3d8bbwe"),
        )
        .unwrap();
        assert_eq!(meta.name.as_deref(), Some("Solitaire & Casual Games"));
        assert_eq!(
            meta.id.as_deref(),
            Some("Microsoft.MicrosoftSolitaireCollection_8wekyb3d8bbwe")
        );
        assert_eq!(
            meta.launch_uri.as_deref(),
            Some("shell:AppsFolder\\Microsoft.MicrosoftSolitaireCollection_8wekyb3d8bbwe!App")
        );
        assert_eq!(
            meta.exe.as_deref(),
            Some("Solitaire.exe")
        );
    }

    #[test]
    fn emulator_definitions_cover_popular_rom_types() {
        assert!(KNOWN_EMULATORS.iter().any(|e| e.rom_exts.contains(&"iso")));
        assert!(KNOWN_EMULATORS.iter().any(|e| e.rom_exts.contains(&"z64")));
        assert!(KNOWN_EMULATORS.iter().any(|e| e.rom_exts.contains(&"gba")));
        assert!(KNOWN_EMULATORS.iter().any(|e| e.rom_exts.contains(&"sfc")));
        assert!(KNOWN_EMULATORS.iter().any(|e| e.rom_exts.contains(&"chd")));
    }

    #[test]
    fn is_game_candidate_filters_launchers_and_redists() {
        assert!(!is_game_candidate("Epic Games Launcher", None, None));
        assert!(!is_game_candidate("GOG GALAXY", Some("unins000.exe"), None));
        assert!(!is_game_candidate(
            "EA app",
            Some("EAappInstaller.exe"),
            None
        ));
        assert!(!is_game_candidate(
            "Ubisoft Connect",
            Some("UbisoftConnect.exe"),
            None
        ));
        assert!(!is_game_candidate(
            "DirectXRedist",
            Some("DXSETUP.exe"),
            Some("C:\\Games\\DirectXRedist")
        ));
        assert!(!is_game_candidate(
            "GameSave",
            None,
            Some("C:\\XboxGames\\GameSave")
        ));
        assert!(!is_game_candidate(
            "games",
            None,
            Some("E:\\Games\\itch\\games")
        ));
        assert!(!is_game_candidate(
            "apps",
            None,
            Some("C:\\Users\\AppData\\itch\\apps")
        ));
        assert!(!is_game_candidate(
            "InstallData",
            None,
            Some("C:\\ProgramData\\EA Desktop\\InstallData")
        ));
        assert!(!is_game_candidate(
            "Code Cache",
            None,
            Some("C:\\Users\\AppData\\itch\\Code Cache")
        ));
        assert!(!is_game_candidate(
            "db",
            None,
            Some("C:\\Users\\AppData\\itch\\db")
        ));
        assert!(!is_game_candidate(
            "Local Storage",
            None,
            Some("C:\\Users\\AppData\\itch\\Local Storage")
        ));
        assert!(!is_game_candidate(
            "logs",
            None,
            Some("C:\\Users\\AppData\\itch\\logs")
        ));
        assert!(!is_game_candidate(
            "Partitions",
            None,
            Some("C:\\Users\\AppData\\itch\\Partitions")
        ));
        assert!(!is_game_candidate(
            "prereqs",
            None,
            Some("C:\\Users\\AppData\\itch\\prereqs")
        ));
        assert!(!is_game_candidate(
            "Shared Dictionary",
            None,
            Some("C:\\Users\\AppData\\itch\\Shared Dictionary")
        ));
        assert!(!is_game_candidate(
            "530c11479fe252fc5aabc24935b9776d4900eb3ba58fdc271e0d6229413ad40e",
            None,
            None
        ));
        assert!(!is_game_candidate(
            "bff3034ad0fe5ad4201268c3aad502c0a6eb142c06437aed3437cddd8834fded",
            None,
            None
        ));

        // Valid games must pass
        assert!(is_game_candidate(
            "Genshin Impact",
            Some("launcher_epic.exe"),
            Some("E:\\Games\\GenshinImpact")
        ));
        assert!(is_game_candidate(
            "A House in the Rift",
            Some("ahitr.exe"),
            Some("E:\\Games\\itch\\games\\ahitr")
        ));
        assert!(is_game_candidate(
            "Dragon Age II",
            Some("DragonAge2.exe"),
            Some("E:\\Games\\Dragon Age II")
        ));
        assert!(is_game_candidate(
            "The Elder Scrolls Online",
            Some("eso.exe"),
            None
        ));
    }

    #[test]
    fn real_scan_filters_software_and_containers() {
        let scan = scan_all(&ScanConfig {
            sources: Vec::new(),
            include_other_drives: true,
            emulator_configs: Vec::new(),
        });
        for g in &scan.games {
            assert_ne!(g.name.to_lowercase(), "games");
            assert_ne!(g.name.to_lowercase(), "apps");
            assert_ne!(g.name, "Epic Games Launcher");
            assert_ne!(g.name, "GOG GALAXY");
            assert_ne!(g.name, "EA app");
            assert_ne!(g.name, "Ubisoft Connect");
            assert_ne!(g.name, "DirectXRedist");
            assert_ne!(g.name, "GameSave");
        }
    }

    #[test]
    fn determines_console_platforms_correctly() {
        assert_eq!(
            determine_console_platform(
                Path::new("C:\\Roms\\Melee.gcm"),
                Some("Dolphin (GameCube/Wii)")
            )
            .as_deref(),
            Some("GameCube")
        );
        assert_eq!(
            determine_console_platform(
                Path::new("C:\\Roms\\MarioKartWii.rvz"),
                Some("Dolphin (GameCube/Wii)")
            )
            .as_deref(),
            Some("Wii")
        );
        assert_eq!(
            determine_console_platform(Path::new("C:\\Roms\\Mario64.z64"), None).as_deref(),
            Some("Nintendo 64")
        );
        assert_eq!(
            determine_console_platform(Path::new("C:\\Roms\\ChronoTrigger.sfc"), None).as_deref(),
            Some("Super Nintendo")
        );
        assert_eq!(
            determine_console_platform(Path::new("C:\\Roms\\PokemonEmerald.gba"), None).as_deref(),
            Some("Game Boy Advance")
        );
        assert_eq!(
            determine_console_platform(
                Path::new("C:\\Roms\\Persona4.chd"),
                Some("PCSX2 (PlayStation 2)")
            )
            .as_deref(),
            Some("PlayStation 2")
        );
        assert_eq!(
            determine_console_platform(
                Path::new("C:\\Roms\\MGS.chd"),
                Some("DuckStation (PlayStation 1)")
            )
            .as_deref(),
            Some("PlayStation")
        );
    }

    #[test]
    fn test_ubisoft_ea_xbox_real_detection() {
        let config = ScanConfig {
            sources: vec![],
            include_other_drives: false,
            emulator_configs: Vec::new(),
        };
        let scan = scan_all(&config);
        let names: Vec<String> = scan.games.iter().map(|g| format!("[{:?}] {}", g.platform, g.name)).collect();
        println!("Detected games:\n{:#?}", names);
        // Verify Xbox detection if on Windows
        #[cfg(windows)]
        {
            let ubi_games: Vec<&str> = scan.games.iter().filter(|g| g.platform == Platform::Ubisoft).map(|g| g.name.as_str()).collect();
            let ea_games: Vec<&str> = scan.games.iter().filter(|g| g.platform == Platform::Ea).map(|g| g.name.as_str()).collect();
            let xbox_games: Vec<&str> = scan.games.iter().filter(|g| g.platform == Platform::Xbox).map(|g| g.name.as_str()).collect();
            println!("Ubisoft: {:?}", ubi_games);
            println!("EA: {:?}", ea_games);
            println!("Xbox: {:?}", xbox_games);
            if !ubi_games.is_empty() {
                assert!(ubi_games.iter().any(|n| n.contains("Assassin")));
            }
            if !ea_games.is_empty() {
                assert!(ea_games.iter().any(|n| n.contains("Dragon Age")));
            }
            if !xbox_games.is_empty() {
                assert!(xbox_games.iter().any(|n| n.contains("Solitaire") || n.contains("Mahjong") || n.contains("Word") || n.contains("Casual")));
            }

            let ubi_folders = library_folders(Platform::Ubisoft);
            let ea_folders = library_folders(Platform::Ea);
            println!("Ubisoft library folders: {:?}", ubi_folders);
            println!("EA library folders: {:?}", ea_folders);
            let mut ubi_set = std::collections::HashSet::new();
            for f in &ubi_folders {
                assert!(ubi_set.insert(f.to_lowercase()), "Duplicate Ubisoft folder found: {}", f);
            }
            let mut ea_set = std::collections::HashSet::new();
            for f in &ea_folders {
                assert!(ea_set.insert(f.to_lowercase()), "Duplicate EA folder found: {}", f);
            }
            // Ensure no empty or trailing-slash duplicate exists
            if !ea_folders.is_empty() {
                assert!(!ea_folders.contains(&"C:\\Program Files\\EA Games".to_string())); // Filtered out because it was empty
            }
        }
    }
}
