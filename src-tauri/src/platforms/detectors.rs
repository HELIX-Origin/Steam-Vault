//! Per-platform detectors.
//!
//! Each detector returns a [`Detection`]: the games it found plus the local
//! source paths it consulted, and an optional note explaining a limitation.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;

use super::{Platform, PlatformGame, PlatformScan, PlatformStatus, ScanConfig};

/// The outcome of one detector run.
struct Detection {
    games: Vec<PlatformGame>,
    roots: Vec<String>,
    note: Option<String>,
}

impl Detection {
    fn empty() -> Self {
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

    if config.is_enabled(Platform::Gog) {
        merge(&mut by_platform, Platform::Gog, detect_gog());
    }
    if config.is_enabled(Platform::Epic) {
        merge(&mut by_platform, Platform::Epic, detect_epic());
    }
    if config.is_enabled(Platform::Itch) {
        merge(&mut by_platform, Platform::Itch, detect_itch());
    }
    if config.is_enabled(Platform::Amazon) {
        merge(&mut by_platform, Platform::Amazon, detect_amazon());
    }
    if config.is_enabled(Platform::Xbox) {
        merge(
            &mut by_platform,
            Platform::Xbox,
            detect_xbox(&mounted_drives),
        );
    }
    if config.is_enabled(Platform::Emulator) {
        merge(
            &mut by_platform,
            Platform::Emulator,
            detect_emulators(&mounted_drives),
        );
    }
    for (platform, detection) in detect_registry_based() {
        if config.is_enabled(platform) {
            merge(&mut by_platform, platform, detection);
        }
    }

    // User-configured folders plus well-known folders on every mounted drive.
    for platform in ORDER {
        if !config.is_enabled(platform) {
            continue;
        }
        let mut roots = config.roots_for(platform);
        if config.include_other_drives
            && platform != Platform::Xbox
            && platform != Platform::Emulator
        {
            roots.extend(drive_candidates(platform, &mounted_drives));
        }
        roots.sort();
        roots.dedup();
        if !roots.is_empty() {
            for root in &roots {
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

/// Well-known install folder names per platform, searched on every drive.
fn drive_folder_names(platform: Platform) -> &'static [&'static str] {
    match platform {
        Platform::Gog => &["GOG Games"],
        Platform::Epic => &["Epic Games"],
        Platform::Itch => &["itch"],
        Platform::Ubisoft => &["Ubisoft Games", "Ubisoft Game Launcher\\games"],
        Platform::Ea => &["EA Games", "Origin Games"],
        Platform::BattleNet => &["Battle.net"],
        Platform::Amazon => &["Amazon Games"],
        Platform::Rockstar => &["Rockstar Games"],
        Platform::Xbox => &["XboxGames"],
        Platform::Emulator => &["Emulators", "Roms", "RetroArch", "Dolphin", "PCSX2"],
        Platform::Steam | Platform::Other => &[],
    }
}

/// Every mounted drive root (`C:\`, `D:\`, …) using sysinfo to avoid probing unmounted drive letters.
fn drive_roots() -> Vec<PathBuf> {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let mut out: Vec<PathBuf> = disks
        .iter()
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
                out.push(candidate.display().to_string());
            }
        }
    }
    out
}

/// Existing locations for a platform, used to autofill Settings.
///
/// The primary entries are the launcher's config / app locations: just as Steam
/// keeps its library folders in `libraryfolders.vdf`, most launchers record
/// their installed games in their own config, so loading that path is usually
/// enough to find games. Game install folders are included as well, because
/// they are the fallback when a launcher's config is missing or incorrect, and
/// the only option for local games that have no shared launcher.
pub fn library_folders(platform: Platform) -> Vec<String> {
    let mut out: Vec<String> = config_dirs(platform)
        .into_iter()
        .map(|path| path.display().to_string())
        .collect();

    // Library folders the platform's own detector already reads.
    match platform {
        Platform::Itch => {
            if let Some(appdata) = env_dir("APPDATA") {
                out.push(appdata.join("itch").join("apps").display().to_string());
                let db_path = appdata.join("itch").join("db").join("butler.db");
                if let Ok(conn) = rusqlite::Connection::open_with_flags(
                    &db_path,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
                ) {
                    if let Ok(mut stmt) = conn.prepare("SELECT path FROM install_locations") {
                        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
                            for path in rows.flatten() {
                                out.push(path);
                            }
                        }
                    }
                }
            }
            if let Some(config) = dirs::config_dir() {
                out.push(config.join("itch").join("apps").display().to_string());
                let db_path = config.join("itch").join("db").join("butler.db");
                if let Ok(conn) = rusqlite::Connection::open_with_flags(
                    &db_path,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
                ) {
                    if let Ok(mut stmt) = conn.prepare("SELECT path FROM install_locations") {
                        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
                            for path in rows.flatten() {
                                out.push(path);
                            }
                        }
                    }
                }
            }
        }
        Platform::Amazon => {
            if let Some(local) = env_dir("LOCALAPPDATA") {
                out.push(
                    local
                        .join("Amazon Games")
                        .join("Library")
                        .display()
                        .to_string(),
                );
            }
        }
        _ => {}
    }

    let mounted = drive_roots();
    out.extend(detector_roots(platform));
    // Game install folders stay useful when a launcher's config is missing or
    // incorrect, and for local games that have no shared launcher at all.
    out.extend(drive_candidates(platform, &mounted));

    // Only real directories are useful as config / library locations.
    out.retain(|path| Path::new(path).is_dir());
    out.sort();
    out.dedup();
    out
}

/// Well-known launcher config / app directories per platform.
///
/// These are where a launcher stores the record of its installed games (the
/// equivalent of Steam's `libraryfolders.vdf`). Paths that do not exist are
/// dropped by [`library_folders`].
fn config_dirs(platform: Platform) -> Vec<PathBuf> {
    let mut out = Vec::new();
    match platform {
        Platform::Gog => {
            if let Some(program_data) = env_dir("ProgramData") {
                out.push(program_data.join("GOG.com").join("Galaxy").join("storage"));
            }
            if let Some(local) = env_dir("LOCALAPPDATA") {
                out.push(local.join("GOG.com").join("Galaxy").join("storage"));
            }
        }
        Platform::Itch => {
            if let Some(appdata) = env_dir("APPDATA") {
                out.push(appdata.join("itch"));
            }
        }
        Platform::Ubisoft => {
            if let Some(local) = env_dir("LOCALAPPDATA") {
                out.push(local.join("Ubisoft Game Launcher"));
            }
        }
        Platform::Ea => {
            if let Some(program_data) = env_dir("ProgramData") {
                out.push(program_data.join("EA Desktop"));
            }
        }
        Platform::BattleNet => {
            if let Some(appdata) = env_dir("APPDATA") {
                out.push(appdata.join("Battle.net"));
            }
        }
        Platform::Rockstar => {
            if let Some(local) = env_dir("LOCALAPPDATA") {
                out.push(local.join("Rockstar Games").join("Launcher"));
            }
        }
        Platform::Xbox => {
            if let Some(local) = env_dir("LOCALAPPDATA") {
                out.push(local.join("Microsoft").join("XboxGames"));
            }
        }
        Platform::Steam
        | Platform::Epic
        | Platform::Amazon
        | Platform::Emulator
        | Platform::Other => {}
    }
    out
}

/// The raw source paths a single platform's detector consults.
fn detector_roots(platform: Platform) -> Vec<String> {
    match platform {
        Platform::Gog => detect_gog().roots,
        Platform::Epic => detect_epic().roots,
        Platform::Itch => detect_itch().roots,
        Platform::Amazon => detect_amazon().roots,
        Platform::Xbox => detect_xbox(&drive_roots()).roots,
        Platform::Emulator => detect_emulators(&drive_roots()).roots,
        Platform::Steam => Vec::new(),
        _ => detect_registry_based()
            .into_iter()
            .filter(|(candidate, _)| *candidate == platform)
            .flat_map(|(_, detection)| detection.roots)
            .collect(),
    }
}

/// Filter out non-game software, launchers, tools, and system packages.
pub fn is_game_candidate(name: &str, exe: Option<&str>, install_dir: Option<&str>) -> bool {
    let n = name.trim().to_lowercase();
    if n.is_empty() {
        return false;
    }

    // 1. Direct blacklisted titles & launchers
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
    ];
    if SOFTWARE_KEYWORDS.iter().any(|&kw| n.contains(kw)) {
        return false;
    }

    // 3. Check executable name (if present)
    if let Some(exe_str) = exe {
        let exe_lower = exe_str.to_lowercase();
        let exe_name = Path::new(&exe_lower)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if exe_name.starts_with("unins")
            || exe_name.starts_with("setup")
            || exe_name.starts_with("installer")
            || exe_name.starts_with("dxsetup")
            || exe_name == "eaappinstaller.exe"
            || exe_name == "ubisoftconnect.exe"
            || exe_name == "galaxyclient.exe"
            || exe_name == "epicgameslauncher.exe"
        {
            return false;
        }
    }

    // 4. Check if directory is an ignored directory name
    if let Some(dir_str) = install_dir {
        let dir_path = Path::new(dir_str);
        if let Some(folder_name) = dir_path.file_name().and_then(|s| s.to_str()) {
            let folder_lower = folder_name.to_lowercase();
            const IGNORED_FOLDERS: &[&str] = &[
                "launcher",
                "directxredist",
                "gameinputredist",
                "gamesave",
                "social club",
                "crashreportclient",
                "redist",
                "_commonredist",
                "support",
                "prerequisites",
                "engine",
                "common",
                "bin",
                "docs",
                "lib",
                "temp",
                "blob_storage",
                "cache",
                "broth",
                "games",
                "apps",
            ];
            if IGNORED_FOLDERS.contains(&folder_lower.as_str()) {
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
        let emu_map = locate_installed_emulators(&[]);
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
                            });
                        }
                    }
                }
                continue;
            }

            let exe = find_executable(&path, 3);
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
                });
            }
        }
    }
    detection
}

/// A stable identity for de-duplicating the same game across detectors.
fn dedup_key(game: &PlatformGame) -> String {
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
fn env_dir(var: &str) -> Option<PathBuf> {
    std::env::var_os(var)
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
}

fn join_exe(dir: &str, relative: &str) -> String {
    let candidate = Path::new(relative);
    if candidate.is_absolute() {
        relative.to_string()
    } else {
        Path::new(dir).join(relative).display().to_string()
    }
}

/// Check if an executable is an uninstaller, setup, or support utility.
fn is_executable_blacklisted(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
        return true;
    };
    let n = name.to_lowercase();
    n.starts_with("unins")
        || n.starts_with("setup")
        || n.starts_with("installer")
        || n.starts_with("cleanup")
        || n.starts_with("dxsetup")
        || n.starts_with("crashreport")
        || n.starts_with("eaappinstaller")
        || n.starts_with("touchup")
        || n.starts_with("activation")
        || n == "easervice.exe"
}

/// Find the shallowest `.exe` beneath `dir`, up to `depth` levels deep.
fn find_executable(dir: &Path, depth: usize) -> Option<String> {
    // 1. Fast path: check if dir itself has a valid game executable (depth 0)
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .map(|ext| ext.eq_ignore_ascii_case("exe"))
                    .unwrap_or(false)
                && !is_executable_blacklisted(&path)
            {
                return Some(path.display().to_string());
            }
        }
    }

    if depth == 0 {
        return None;
    }

    // 2. BFS for subdirectories up to depth
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
                {
                    continue;
                }
                if level < depth {
                    subdirs.push((path, level + 1));
                }
            } else if path
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("exe"))
                .unwrap_or(false)
                && !is_executable_blacklisted(&path)
            {
                return Some(path.display().to_string());
            }
        }
        queue.extend(subdirs);
    }
    None
}

// ---------------------------------------------------------------------------
// GOG
// ---------------------------------------------------------------------------

fn detect_gog() -> Detection {
    let mut detection = Detection::empty();

    #[cfg(windows)]
    {
        use crate::platforms::registry;
        let installs = registry::gog_installs();
        if !installs.is_empty() {
            detection
                .roots
                .push("HKEY_LOCAL_MACHINE\\SOFTWARE\\GOG.com\\Games".into());
        }
        for install in installs {
            detection.games.push(PlatformGame {
                name: install.name,
                platform: Platform::Gog,
                platform_id: Some(install.id),
                install_dir: install.path.clone(),
                exe: install.exe.or(install.path),
                size_on_disk: None,
                launch_uri: None,
                source: Some("GOG Galaxy registry".into()),
            });
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

    // Linux / macOS: legendary / Heroic.
    if let Some(config) = dirs::config_dir() {
        for candidate in [
            config
                .join("heroic")
                .join("legendaryConfig")
                .join("legendary"),
            config.join("legendary"),
        ] {
            let file = candidate.join("installed.json");
            if file.is_file() {
                detection.roots.push(file.display().to_string());
                detection
                    .games
                    .extend(heroic_installed(&file, Platform::Epic));
            }
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
        launch_uri: value
            .get("AppName")
            .or_else(|| value.get("CatalogItemId"))
            .and_then(|v| v.as_str())
            .map(|id| format!("com.epicgames.launcher://apps/{id}?action=launch")),
        source: Some("Epic manifest".into()),
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
                                        #[cfg(windows)]
                                        let matches_os =
                                            flavor == "windows" || rel_path.ends_with(".exe");
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

        // Skip internal/container directories
        if folder_name.starts_with('.')
            || folder_lower == "games"
            || folder_lower == "apps"
            || folder_lower == "broth"
            || folder_lower == "prereqs"
            || folder_lower == "cache"
            || folder_lower == "blob_storage"
            || folder_lower == "db"
            || folder_lower == "logs"
            || folder_lower == "partitions"
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
        });
    }
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

fn detect_xbox(mounted_drives: &[PathBuf]) -> Detection {
    let mut detection = Detection::empty();

    for drive in mounted_drives {
        for sub in &["XboxGames", "Games\\XboxGames"] {
            let candidate = drive.join(sub);
            if candidate.is_dir() {
                scan_xbox_dir(&candidate, &mut detection);
            }
        }
    }

    if detection.roots.is_empty() {
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
                        break;
                    }
                }
            }
            let manifest_path = s_dir.join("appxmanifest.xml");
            if manifest_path.is_file() {
                if let Ok(xml) = std::fs::read_to_string(&manifest_path) {
                    if let Some(meta) = parse_appx_manifest(&xml, s_dir) {
                        if let Some(n) = meta.name {
                            game_name = n;
                        }
                        if let Some(e) = meta.exe {
                            exe_path = Some(e);
                        }
                        platform_id = meta.id;
                        launch_uri = meta.launch_uri;
                        break;
                    }
                }
            }
        }

        if exe_path.is_none() {
            exe_path = find_executable(&path, 3);
        }

        detection.games.push(PlatformGame {
            name: game_name,
            platform: Platform::Xbox,
            platform_id,
            install_dir: Some(path.display().to_string()),
            exe: exe_path,
            size_on_disk: None,
            launch_uri,
            source: Some("Xbox Game Pass / Microsoft Store".into()),
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
    let name = extract_xml_tag(xml, "DisplayName").or_else(|| extract_xml_tag(xml, "IdForDisplay"));

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
    let name = extract_xml_tag(xml, "DisplayName");

    let id = if let Some(start) = xml.find("<Identity ") {
        let end = xml[start..].find('>')?;
        extract_xml_attr(&xml[start..start + end], "Name")
    } else {
        None
    };

    let launch_uri = id
        .as_ref()
        .map(|id_str| format!("shell:AppsFolder\\{id_str}!App"));

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
        id,
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

fn detect_emulators(mounted_drives: &[PathBuf]) -> Detection {
    let mut detection = Detection::empty();
    let emu_map = locate_installed_emulators(mounted_drives);

    // Scan mounted drives for ROMs folders
    for drive in mounted_drives {
        for sub in &["Roms", "Games\\Roms", "Emulators", "Games\\Emulators"] {
            let candidate = drive.join(sub);
            if candidate.is_dir() {
                scan_emulator_dir(&candidate, &mut detection, &emu_map);
            }
        }
    }

    if detection.roots.is_empty() {
        detection.note =
            Some("No ROM or emulator folders found. Add your ROM folder in Settings.".into());
    }

    detection
}

/// Locate all installed emulator executables once into a fast lookup map.
fn locate_installed_emulators(mounted_drives: &[PathBuf]) -> BTreeMap<&'static str, String> {
    let base_search_dirs = [
        env_dir("LOCALAPPDATA"),
        env_dir("APPDATA"),
        env_dir("ProgramFiles"),
        env_dir("ProgramFiles(x86)"),
    ];

    let mut map = BTreeMap::new();
    for emu in KNOWN_EMULATORS {
        let mut found = None;

        // Check well-known install locations
        for base in base_search_dirs.iter().flatten() {
            for sub in emu.dirs {
                let emu_dir = base.join(sub);
                for exe in emu.exes {
                    let exe_path = emu_dir.join(exe);
                    if exe_path.is_file() {
                        found = Some(exe_path.display().to_string());
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
            }
            if found.is_some() {
                break;
            }
        }

        // If not in standard base directories, check mounted drive emulator folders
        if found.is_none() {
            for drive in mounted_drives {
                for emu_dir_name in emu.dirs {
                    for candidate in &[
                        drive.join("Emulators").join(emu_dir_name),
                        drive.join("Games").join("Emulators").join(emu_dir_name),
                        drive.join(emu_dir_name),
                    ] {
                        for exe in emu.exes {
                            let exe_path = candidate.join(exe);
                            if exe_path.is_file() {
                                found = Some(exe_path.display().to_string());
                                break;
                            }
                        }
                        if found.is_some() {
                            break;
                        }
                    }
                    if found.is_some() {
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
            }
        }

        if let Some(exe_path) = found {
            map.insert(emu.name, exe_path);
        }
    }

    map
}

/// Recursively scan a directory for ROM files up to depth 4.
fn scan_emulator_dir(
    dir: &Path,
    detection: &mut Detection,
    emu_map: &BTreeMap<&'static str, String>,
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
    emu_map: &BTreeMap<&'static str, String>,
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
            if folder_name.starts_with('.')
                || folder_name.eq_ignore_ascii_case("cache")
                || folder_name.eq_ignore_ascii_case("temp")
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

        // Check if file is a known ROM extension
        let matched_emu = KNOWN_EMULATORS.iter().find(|emu| {
            emu.rom_exts
                .iter()
                .any(|e| e.eq_ignore_ascii_case(&ext_lower))
        });

        let Some(emu) = matched_emu else {
            continue;
        };

        let raw_name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let name = crate::metadata::vndb::clean_search_term(&raw_name);
        if name.is_empty() {
            continue;
        }

        let rom_path_str = path.display().to_string();
        let size = entry.metadata().ok().map(|m| m.len());

        let (emu_name, exe, launch_uri) = match emu_map.get(emu.name) {
            Some(exe_path) => {
                let launch_options = (emu.launch_arg)(&rom_path_str);
                (emu.name, Some(exe_path.clone()), Some(launch_options))
            }
            None => (
                "ROM (configure emulator in Settings)",
                None,
                Some(format!("\"{rom_path_str}\"")),
            ),
        };

        detection.games.push(PlatformGame {
            name,
            platform: Platform::Emulator,
            platform_id: Some(raw_name),
            install_dir: path.parent().map(|p| p.display().to_string()),
            exe,
            size_on_disk: size,
            launch_uri,
            source: Some(emu_name.into()),
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
        });
    }
    buckets.into_iter().collect()
}

#[cfg(not(windows))]
fn detect_registry_based() -> Vec<(Platform, Detection)> {
    Vec::new()
}

/// Map an uninstall `Publisher` to a platform, or `None` if unrecognised.
fn platform_for_publisher(publisher: &str) -> Option<Platform> {
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
fn icon_executable(icon: &str) -> Option<String> {
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
}
