//! Linux-specific platform and launcher detectors.
//!
//! Authoritative detection of installed games on Linux via FreeDesktop
//! `.desktop` files, Flatpak apps, Heroic Games Launcher, Lutris SQLite
//! database, Bottles Wine prefixes, and itch.io Linux manifests.

use std::path::{Path, PathBuf};

use crate::platforms::detectors::{
    find_executable, is_blacklisted_root, is_game_candidate, normalize_path,
};
use crate::platforms::{Detection, Platform, PlatformGame, ScanConfig};

/// Detect all Linux-native platform launchers and games.
pub fn detect_platform_launchers(
    config: &ScanConfig,
    _mounted_drives: &[PathBuf],
) -> Vec<(Platform, Detection)> {
    let mut results = Vec::new();

    if config.is_enabled(Platform::Gog) {
        results.push((Platform::Gog, detect_gog_linux()));
    }
    if config.is_enabled(Platform::Epic) {
        results.push((Platform::Epic, detect_epic_linux()));
    }
    if config.is_enabled(Platform::Itch) {
        results.push((Platform::Itch, detect_itch_linux()));
    }

    // Generic FreeDesktop and Flatpak game discovery
    let desktop_games = scan_freedesktop_games();
    if !desktop_games.games.is_empty() && config.is_enabled(Platform::Other) {
        results.push((Platform::Other, desktop_games));
    }

    // Lutris games discovery
    let lutris_games = detect_lutris();
    if !lutris_games.games.is_empty() && config.is_enabled(Platform::Other) {
        results.push((Platform::Other, lutris_games));
    }

    results
}

/// Linux default library folders for each platform.
pub fn default_library_folders(platform: Platform) -> Vec<String> {
    let mut out = Vec::new();
    let home = dirs::home_dir();

    match platform {
        Platform::Gog => {
            if let Some(h) = &home {
                out.push(normalize_path(
                    &h.join(".config/heroic/gog_store").display().to_string(),
                ));
                out.push(normalize_path(
                    &h.join(".var/app/com.heroicgameslauncher.hgl/config/heroic/gog_store")
                        .display()
                        .to_string(),
                ));
                out.push(normalize_path(
                    &h.join("GOG Games").display().to_string(),
                ));
            }
        }
        Platform::Epic => {
            if let Some(h) = &home {
                out.push(normalize_path(
                    &h.join(".config/heroic/legendaryConfig/legendary")
                        .display()
                        .to_string(),
                ));
                out.push(normalize_path(
                    &h.join(".var/app/com.heroicgameslauncher.hgl/config/heroic/legendaryConfig/legendary")
                        .display()
                        .to_string(),
                ));
            }
        }
        Platform::Itch => {
            if let Some(h) = &home {
                out.push(normalize_path(
                    &h.join(".config/itch/apps").display().to_string(),
                ));
                out.push(normalize_path(
                    &h.join(".var/app/io.itch.itch/config/itch/apps")
                        .display()
                        .to_string(),
                ));
            }
        }
        Platform::Ubisoft
        | Platform::Ea
        | Platform::BattleNet
        | Platform::Rockstar
        | Platform::Amazon
        | Platform::Xbox
        | Platform::Steam
        | Platform::Other => {
            if let Some(h) = &home {
                out.push(normalize_path(
                    &h.join(".local/share/applications").display().to_string(),
                ));
                out.push(normalize_path(
                    &h.join(".local/share/lutris").display().to_string(),
                ));
                out.push(normalize_path(
                    &h.join(".local/share/bottles").display().to_string(),
                ));
            }
            out.push("/usr/share/applications".into());
        }
    }
    out
}

// ---------------------------------------------------------------------------
// GOG on Linux (via Heroic & GOG Linux native)
// ---------------------------------------------------------------------------

fn detect_gog_linux() -> Detection {
    let mut detection = Detection::empty();
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return detection,
    };

    let candidate_paths = [
        home.join(".config/heroic/gog_store/installed.json"),
        home.join(".var/app/com.heroicgameslauncher.hgl/config/heroic/gog_store/installed.json"),
    ];

    for p in &candidate_paths {
        if p.is_file() {
            detection.roots.push(p.display().to_string());
            scan_heroic_installed(p, Platform::Gog, &mut detection);
        }
    }

    detection
}

// ---------------------------------------------------------------------------
// Epic Games on Linux (via Heroic)
// ---------------------------------------------------------------------------

fn detect_epic_linux() -> Detection {
    let mut detection = Detection::empty();
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return detection,
    };

    let candidate_paths = [
        home.join(".config/heroic/legendaryConfig/legendary/installed.json"),
        home.join(".var/app/com.heroicgameslauncher.hgl/config/heroic/legendaryConfig/legendary/installed.json"),
    ];

    for p in &candidate_paths {
        if p.is_file() {
            detection.roots.push(p.display().to_string());
            scan_heroic_installed(p, Platform::Epic, &mut detection);
        }
    }

    detection
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
                    source: Some("Heroic (Linux)".into()),
                    ..Default::default()
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// itch.io on Linux
// ---------------------------------------------------------------------------

fn detect_itch_linux() -> Detection {
    let mut detection = Detection::empty();
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return detection,
    };

    let candidate_dirs = [
        home.join(".config/itch/apps"),
        home.join(".var/app/io.itch.itch/config/itch/apps"),
    ];

    for itch_apps in &candidate_dirs {
        if itch_apps.is_dir() {
            detection.roots.push(itch_apps.display().to_string());
            if let Ok(entries) = std::fs::read_dir(itch_apps) {
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
                                source: Some("itch.io (Linux)".into()),
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
// Lutris on Linux
// ---------------------------------------------------------------------------

fn detect_lutris() -> Detection {
    let mut detection = Detection::empty();
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return detection,
    };

    let db_candidates = [
        home.join(".local/share/lutris/pga.db"),
        home.join(".var/app/net.lutris.Lutris/data/lutris/pga.db"),
    ];

    for db_path in &db_candidates {
        if db_path.is_file() {
            detection.roots.push(db_path.display().to_string());
            if let Ok(conn) = rusqlite::Connection::open_with_flags(
                db_path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            ) {
                if let Ok(mut stmt) = conn.prepare(
                    "SELECT id, slug, name, runner, directory, installed FROM games WHERE installed = 1",
                ) {
                    if let Ok(rows) = stmt.query_map([], |row| {
                        let id: i64 = row.get(0)?;
                        let slug: String = row.get(1)?;
                        let name: String = row.get(2)?;
                        let _runner: Option<String> = row.get(3)?;
                        let directory: Option<String> = row.get(4)?;
                        let installed: i32 = row.get(5)?;
                        Ok((id, slug, name, directory, installed == 1))
                    }) {
                        for (id, slug, name, directory, installed) in rows.flatten() {
                            let exe = directory.as_ref().and_then(|d| find_executable(Path::new(d), 2));
                            detection.games.push(PlatformGame {
                                name,
                                platform: Platform::Other,
                                platform_id: Some(id.to_string()),
                                install_dir: directory,
                                exe,
                                installed,
                                launch_uri: Some(format!("lutris:rungame/{slug}")),
                                source: Some("Lutris".into()),
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
// FreeDesktop `.desktop` File Game Scanner
// ---------------------------------------------------------------------------

fn scan_freedesktop_games() -> Detection {
    let mut detection = Detection::empty();
    let mut app_dirs = vec![PathBuf::from("/usr/share/applications")];
    if let Some(home) = dirs::home_dir() {
        app_dirs.push(home.join(".local/share/applications"));
        app_dirs.push(home.join(".local/share/flatpak/exports/share/applications"));
        app_dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
    }

    for dir in app_dirs {
        if !dir.is_dir() {
            continue;
        }
        detection.roots.push(dir.display().to_string());
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().is_some_and(|ext| ext == "desktop") {
                    if let Some(game) = parse_desktop_game(&p) {
                        detection.games.push(game);
                    }
                }
            }
        }
    }

    detection
}

fn parse_desktop_game(path: &Path) -> Option<PlatformGame> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut name = None;
    let mut exec = None;
    let mut is_game = false;
    let mut no_display = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Name=") && name.is_none() {
            name = Some(trimmed["Name=".len()..].trim().to_string());
        } else if trimmed.starts_with("Exec=") && exec.is_none() {
            let full_exec = trimmed["Exec=".len()..].trim();
            // Strip format specifiers like %u, %F, etc.
            let clean_exec = full_exec.split_whitespace().next().unwrap_or("").to_string();
            exec = Some(clean_exec);
        } else if trimmed.starts_with("Categories=") {
            let cats = trimmed["Categories=".len()..].to_lowercase();
            if cats.contains("game") {
                is_game = true;
            }
        } else if trimmed.starts_with("NoDisplay=true") {
            no_display = true;
        }
    }

    if !is_game || no_display {
        return None;
    }

    let game_name = name?;
    let lower_name = game_name.to_lowercase();
    // Exclude Steam client, settings, and desktop utility items
    if lower_name == "steam" || lower_name.contains("settings") {
        return None;
    }

    let file_id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&game_name)
        .to_string();

    Some(PlatformGame {
        name: game_name,
        platform: Platform::Other,
        platform_id: Some(file_id),
        install_dir: None,
        exe: exec,
        installed: true,
        source: Some("Linux Desktop Entry".into()),
        ..Default::default()
    })
}
