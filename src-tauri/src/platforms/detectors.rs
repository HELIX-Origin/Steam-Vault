//! Per-platform detectors.
//!
//! Each detector returns a [`Detection`]: the games it found plus the local
//! source paths it consulted, and an optional note explaining a limitation.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

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
const ORDER: [Platform; 10] = [
    Platform::Gog,
    Platform::Epic,
    Platform::Itch,
    Platform::Ubisoft,
    Platform::Ea,
    Platform::BattleNet,
    Platform::Amazon,
    Platform::Rockstar,
    Platform::Xbox,
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
        if config.include_other_drives {
            roots.extend(drive_candidates(platform));
        }
        roots.sort();
        roots.dedup();
        if !roots.is_empty() {
            merge(
                &mut by_platform,
                platform,
                detect_custom_roots(platform, &roots),
            );
        }
    }

    if config.is_enabled(Platform::Xbox) {
        let mut xbox = Detection::empty();
        xbox.note = Some(
            "Microsoft Store / Game Pass titles are not registered in the classic Uninstall \
             registry, so only folders named XboxGames on your drives are detected."
                .into(),
        );
        merge(&mut by_platform, Platform::Xbox, xbox);
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
            if seen.insert((platform, dedup_key(&game))) {
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
        Platform::Gog => &["GOG Games", "GOG Galaxy", "Games\\GOG"],
        Platform::Epic => &["Epic Games", "Program Files\\Epic Games"],
        Platform::Itch => &["itch", "Games\\itch"],
        Platform::Ubisoft => &["Ubisoft", "Ubisoft Game Launcher", "Games\\Ubisoft"],
        Platform::Ea => &["EA Games", "Origin Games", "Program Files\\EA Games"],
        Platform::BattleNet => &["Battle.net", "Games\\Battle.net"],
        Platform::Amazon => &["Amazon Games", "Games\\Amazon Games"],
        Platform::Rockstar => &["Rockstar Games", "Games\\Rockstar Games"],
        Platform::Xbox => &["XboxGames", "Games\\XboxGames"],
        Platform::Steam | Platform::Other => &[],
    }
}

/// Every mounted drive root (`C:\`, `D:\`, …). Windows only; elsewhere empty.
fn drive_roots() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        ('A'..='Z')
            .map(|letter| PathBuf::from(format!("{letter}:\\")))
            .filter(|path| path.is_dir())
            .collect()
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

/// Well-known folders for a platform that exist on any mounted drive.
fn drive_candidates(platform: Platform) -> Vec<String> {
    let names = drive_folder_names(platform);
    if names.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for drive in drive_roots() {
        for name in names {
            let candidate = drive.join(name);
            if candidate.is_dir() {
                out.push(candidate.display().to_string());
            }
        }
    }
    out
}

/// Discover one game per immediate subdirectory of each configured folder.
fn detect_custom_roots(platform: Platform, roots: &[String]) -> Detection {
    let mut detection = Detection::empty();
    for root in roots {
        detection.roots.push(root.clone());
        let Ok(entries) = std::fs::read_dir(Path::new(root)) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            detection.games.push(PlatformGame {
                name: entry.file_name().to_string_lossy().to_string(),
                platform,
                platform_id: None,
                install_dir: Some(path.display().to_string()),
                exe: find_executable(&path, 3),
                size_on_disk: None,
                launch_uri: None,
                source: Some("Folder scan".into()),
            });
        }
    }
    detection
}

/// A stable identity for de-duplicating the same game across detectors.
fn dedup_key(game: &PlatformGame) -> String {
    game.install_dir
        .clone()
        .or_else(|| game.exe.clone())
        .unwrap_or_else(|| game.name.clone())
        .to_lowercase()
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

/// Find the shallowest `.exe` beneath `dir`, up to `depth` levels deep.
fn find_executable(dir: &Path, depth: usize) -> Option<String> {
    let mut queue = vec![(dir.to_path_buf(), 0usize)];
    while let Some((current, level)) = queue.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        let mut subdirs = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if level < depth {
                    subdirs.push((path, level + 1));
                }
            } else if path
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("exe"))
                .unwrap_or(false)
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
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let is_item = path
                        .extension()
                        .map(|ext| ext.eq_ignore_ascii_case("item"))
                        .unwrap_or(false);
                    if is_item {
                        if let Some(game) = epic_from_manifest(&path) {
                            detection.games.push(game);
                        }
                    }
                }
            }
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
            .get("CatalogItemId")
            .and_then(|v| v.as_str())
            .map(|id| format!("com.epicgames.launcher://apps/{id}?action=launch")),
        source: Some("Epic manifest".into()),
    })
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

    let mut dirs_to_scan = Vec::new();
    if let Some(appdata) = env_dir("APPDATA") {
        dirs_to_scan.push(appdata.join("itch").join("apps"));
    }
    if let Some(config) = dirs::config_dir() {
        dirs_to_scan.push(config.join("itch").join("apps"));
    }

    for dir in dirs_to_scan {
        if !dir.is_dir() {
            continue;
        }
        detection.roots.push(dir.display().to_string());
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            detection.games.push(PlatformGame {
                name: entry.file_name().to_string_lossy().to_string(),
                platform: Platform::Itch,
                platform_id: None,
                install_dir: Some(path.display().to_string()),
                exe: find_executable(&path, 2),
                size_on_disk: None,
                launch_uri: None,
                source: Some("itch app".into()),
            });
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
                    });
                }
            }
        }
    }

    detection
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
        let bucket = buckets.entry(platform).or_insert_with(Detection::empty);
        if bucket.roots.is_empty() {
            bucket.roots.push("Uninstall registry".into());
        }
        bucket.games.push(PlatformGame {
            name: entry.display_name,
            platform,
            platform_id: None,
            install_dir: entry.install_location,
            exe: entry.display_icon.as_deref().and_then(icon_executable),
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
