//! macOS-specific platform and launcher detectors.
//!
//! Authoritative detection of installed games on macOS via `/Applications`,
//! `~/Applications`, `~/Library/Application Support`, Info.plist metadata,
//! GOG Galaxy macOS database, and Epic Games manifests.

use std::path::{Path, PathBuf};

use crate::platforms::detectors::{
    find_executable, is_blacklisted_root, is_game_candidate, normalize_path,
};
use crate::platforms::{Detection, Platform, PlatformGame, ScanConfig};

/// Detect all macOS-native platform launchers and games.
pub fn detect_platform_launchers(
    config: &ScanConfig,
    _mounted_drives: &[PathBuf],
) -> Vec<(Platform, Detection)> {
    let mut results = Vec::new();

    if config.is_enabled(Platform::Gog) {
        results.push((Platform::Gog, detect_gog_macos()));
    }
    if config.is_enabled(Platform::Epic) {
        results.push((Platform::Epic, detect_epic_macos()));
    }
    if config.is_enabled(Platform::Itch) {
        results.push((Platform::Itch, detect_itch_macos()));
    }
    if config.is_enabled(Platform::BattleNet) {
        results.push((Platform::BattleNet, detect_battlenet_macos()));
    }

    // Generic macOS .app bundle scanning in /Applications and ~/Applications
    let apps_detection = scan_macos_applications();
    if !apps_detection.games.is_empty() && config.is_enabled(Platform::Other) {
        results.push((Platform::Other, apps_detection));
    }

    results
}

/// macOS default library folders for each platform.
pub fn default_library_folders(platform: Platform) -> Vec<String> {
    let mut out = Vec::new();
    let home = dirs::home_dir();

    match platform {
        Platform::Gog => {
            out.push("/Applications".into());
            out.push("/Applications/GOG".into());
            if let Some(h) = &home {
                out.push(normalize_path(
                    &h.join("Applications").display().to_string(),
                ));
                out.push(normalize_path(
                    &h.join("Library/Application Support/GOG.com/Galaxy/storage")
                        .display()
                        .to_string(),
                ));
            }
        }
        Platform::Epic => {
            out.push("/Users/Shared/Epic Games".into());
            if let Some(h) = &home {
                out.push(normalize_path(
                    &h.join("Library/Application Support/Epic/EpicGamesLauncher/Data/Manifests")
                        .display()
                        .to_string(),
                ));
            }
        }
        Platform::Itch => {
            if let Some(h) = &home {
                out.push(normalize_path(
                    &h.join("Library/Application Support/itch/apps")
                        .display()
                        .to_string(),
                ));
            }
        }
        Platform::BattleNet => {
            out.push("/Applications/Battle.net".into());
        }
        Platform::Emulator => {
            if let Some(h) = &home {
                out.push(normalize_path(
                    &h.join("Library/Application Support/OpenEmu/Game Library")
                        .display()
                        .to_string(),
                ));
                out.push(normalize_path(
                    &h.join("Library/Application Support/RetroArch")
                        .display()
                        .to_string(),
                ));
            }
        }
        Platform::Ubisoft
        | Platform::Ea
        | Platform::Rockstar
        | Platform::Amazon
        | Platform::Xbox
        | Platform::Steam
        | Platform::Other => {
            out.push("/Applications".into());
            if let Some(h) = &home {
                out.push(normalize_path(
                    &h.join("Applications").display().to_string(),
                ));
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// GOG Galaxy on macOS
// ---------------------------------------------------------------------------

fn detect_gog_macos() -> Detection {
    let mut detection = Detection::empty();
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return detection,
    };

    // 1. GOG Galaxy 2.0 SQLite database
    let db_path = home.join("Library/Application Support/GOG.com/Galaxy/storage/galaxy-2.0.db");
    if db_path.is_file() {
        detection.roots.push(db_path.display().to_string());
        if let Ok(conn) = rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        ) {
            let mut installed_ids = std::collections::BTreeSet::new();
            if let Ok(mut stmt) = conn.prepare("SELECT productId FROM InstalledProducts") {
                if let Ok(rows) = stmt.query_map([], |row| row.get::<_, i64>(0)) {
                    for id in rows.flatten() {
                        installed_ids.insert(id.to_string());
                    }
                }
            }

            if let Ok(mut stmt) = conn.prepare(
                "SELECT gp.productId, gp.title \
                 FROM GamePieces gp \
                 JOIN Products p ON gp.productId = p.id \
                 WHERE gp.originalTitle IS NOT NULL",
            ) {
                if let Ok(rows) = stmt.query_map([], |row| {
                    let product_id: i64 = row.get(0)?;
                    let title: String = row.get(1)?;
                    Ok((product_id.to_string(), title))
                }) {
                    for (product_id, title) in rows.flatten() {
                        let is_installed = installed_ids.contains(&product_id);
                        detection.games.push(PlatformGame {
                            name: title,
                            platform: Platform::Gog,
                            platform_id: Some(product_id.clone()),
                            install_dir: None,
                            exe: None,
                            installed: is_installed,
                            launch_uri: Some(format!("goggalaxy://openGameView/{product_id}")),
                            source: Some("GOG Galaxy (macOS)".into()),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    // 2. Heroic Games Launcher on macOS (GOG store)
    let heroic_gog = home.join("Library/Application Support/heroic/gog_store/installed.json");
    if heroic_gog.is_file() {
        detection.roots.push(heroic_gog.display().to_string());
        scan_heroic_installed(&heroic_gog, Platform::Gog, &mut detection);
    }

    detection
}

// ---------------------------------------------------------------------------
// Epic Games on macOS
// ---------------------------------------------------------------------------

fn detect_epic_macos() -> Detection {
    let mut detection = Detection::empty();
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return detection,
    };

    // 1. Official Epic Games Launcher manifests on macOS
    let manifest_dir = home.join("Library/Application Support/Epic/EpicGamesLauncher/Data/Manifests");
    if manifest_dir.is_dir() {
        detection.roots.push(manifest_dir.display().to_string());
        if let Ok(entries) = std::fs::read_dir(&manifest_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().is_some_and(|ext| ext == "item") {
                    if let Some(game) = epic_from_item(&p) {
                        detection.games.push(game);
                    }
                }
            }
        }
    }

    // 2. Heroic Games Launcher on macOS (Epic store)
    let heroic_epic = home.join("Library/Application Support/heroic/legendaryConfig/legendary/installed.json");
    if heroic_epic.is_file() {
        detection.roots.push(heroic_epic.display().to_string());
        scan_heroic_installed(&heroic_epic, Platform::Epic, &mut detection);
    }

    detection
}

fn epic_from_item(path: &Path) -> Option<PlatformGame> {
    let text = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&text).ok()?;
    let display_name = json.get("DisplayName")?.as_str()?;
    let app_name = json.get("AppName")?.as_str()?;
    let install_loc = json
        .get("InstallLocation")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let launch_exe = json
        .get("LaunchExecutable")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let exe_path = if !install_loc.is_empty() && !launch_exe.is_empty() {
        Some(format!("{install_loc}/{launch_exe}"))
    } else {
        None
    };

    Some(PlatformGame {
        name: display_name.to_string(),
        platform: Platform::Epic,
        platform_id: Some(app_name.to_string()),
        install_dir: if install_loc.is_empty() {
            None
        } else {
            Some(install_loc.to_string())
        },
        exe: exe_path,
        installed: true,
        launch_uri: Some(format!(
            "com.epicgames.launcher://apps/{app_name}?action=launch&silent=true"
        )),
        source: Some("Epic Games (macOS)".into()),
        ..Default::default()
    })
}

fn scan_heroic_installed(path: &Path, platform: Platform, detection: &mut Detection) {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return,
    };
    let json: serde_json::Value = match serde_json::from_str(&text) {
        Ok(j) => j,
        Err(_) => return,
    };
    let items = json
        .get("installed")
        .and_then(|v| v.as_array())
        .or_else(|| json.as_array());

    if let Some(arr) = items {
        for entry in arr {
            let app_name = entry
                .get("appName")
                .or_else(|| entry.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let title = entry
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or(app_name);
            let install_path = entry
                .get("install_path")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let exe = entry
                .get("executable")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if !app_name.is_empty() {
                let full_exe = if !install_path.is_empty() && !exe.is_empty() {
                    Some(format!("{install_path}/{exe}"))
                } else if !install_path.is_empty() {
                    find_executable(Path::new(install_path), 2)
                } else {
                    None
                };

                detection.games.push(PlatformGame {
                    name: title.to_string(),
                    platform,
                    platform_id: Some(app_name.to_string()),
                    install_dir: if install_path.is_empty() {
                        None
                    } else {
                        Some(install_path.to_string())
                    },
                    exe: full_exe,
                    installed: true,
                    launch_uri: Some(format!("heroic://launch/{app_name}")),
                    source: Some("Heroic (macOS)".into()),
                    ..Default::default()
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// itch.io on macOS
// ---------------------------------------------------------------------------

fn detect_itch_macos() -> Detection {
    let mut detection = Detection::empty();
    if let Some(home) = dirs::home_dir() {
        let itch_apps = home.join("Library/Application Support/itch/apps");
        if itch_apps.is_dir() {
            detection.roots.push(itch_apps.display().to_string());
            if let Ok(entries) = std::fs::read_dir(&itch_apps) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() && !is_blacklisted_root(&p) {
                        let name = p
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_string();
                        let exe = find_executable(&p, 3);
                        let norm = normalize_path(&p.display().to_string());
                        if is_game_candidate(&name, exe.as_deref(), Some(&norm)) {
                            detection.games.push(PlatformGame {
                                name: name.clone(),
                                platform: Platform::Itch,
                                platform_id: Some(name),
                                install_dir: Some(norm),
                                exe,
                                installed: true,
                                source: Some("itch.io (macOS)".into()),
                                ..Default::default()
                            });
                        }
                    }
                }
            }
        }
    }
    detection
}

// ---------------------------------------------------------------------------
// Battle.net on macOS
// ---------------------------------------------------------------------------

fn detect_battlenet_macos() -> Detection {
    let mut detection = Detection::empty();
    let bnet_app = Path::new("/Applications/Battle.net.app");
    if bnet_app.is_dir() {
        detection.roots.push("/Applications/Battle.net.app".into());
    }
    detection
}

// ---------------------------------------------------------------------------
// Generic macOS Applications (/Applications and ~/Applications)
// ---------------------------------------------------------------------------

fn scan_macos_applications() -> Detection {
    let mut detection = Detection::empty();
    let mut search_dirs = vec![PathBuf::from("/Applications")];
    if let Some(home) = dirs::home_dir() {
        search_dirs.push(home.join("Applications"));
    }

    for app_dir in search_dirs {
        if !app_dir.is_dir() {
            continue;
        }
        detection.roots.push(app_dir.display().to_string());
        if let Ok(entries) = std::fs::read_dir(&app_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() && p.extension().is_some_and(|ext| ext == "app") {
                    let app_name = p
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();

                    // Check if it's a known non-game application
                    let lower = app_name.to_lowercase();
                    if lower.contains("safari")
                        || lower.contains("chrome")
                        || lower.contains("firefox")
                        || lower.contains("utilities")
                        || lower.contains("xcode")
                        || lower.contains("system preferences")
                        || lower.contains("calculator")
                        || lower.contains("terminal")
                    {
                        continue;
                    }

                    let exe = p.join("Contents/MacOS").join(&app_name);
                    let exe_str = if exe.is_file() {
                        Some(exe.display().to_string())
                    } else {
                        find_executable(&p.join("Contents/MacOS"), 1)
                    };

                    detection.games.push(PlatformGame {
                        name: app_name.clone(),
                        platform: Platform::Other,
                        platform_id: Some(app_name),
                        install_dir: Some(p.display().to_string()),
                        exe: exe_str,
                        installed: true,
                        source: Some("macOS Applications".into()),
                        ..Default::default()
                    });
                }
            }
        }
    }

    detection
}
