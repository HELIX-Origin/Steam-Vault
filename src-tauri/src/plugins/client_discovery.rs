//! Authoritative local client discovery for owned games across desktop platforms.
//!
//! Queries installed client databases and manifests:
//! - GOG Galaxy (`galaxy-2.0.db`)
//! - itch.io Butler (`butler.db`)
//! - Epic Games (Legendary metadata & official `.item` manifests)
//! - EA Desktop / EA App (Logs & InstallData)
//! - Ubisoft Connect (`configurations` & `cache/ownership/*` protobuf stores)
//! - Xbox / Microsoft Store (`AsyncCache.db` & `*_PersistentEntitlementsCache.json`)
//! - Battle.net (`CachedData.db`, `Battle.net.config`, & local product stores)
//!
//! Invariants:
//! - All database operations are STRICTLY read-only (`rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY`).
//! - Never modify, lock, or write to external client databases or files.
//! - GOG Galaxy discovery MUST filter strictly to genuine GOG titles (`releaseKey LIKE 'gog_%'`)
//!   to prevent pulling non-GOG linked games into the GOG provider integration.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};
use serde_json::Value;

use crate::platforms::{normalize_game_title, Platform, PlatformGame};

/// Check standard installation directories on disk for an executable.
pub fn locate_executable_in_folder(folder: &Path) -> Option<PathBuf> {
    if !folder.is_dir() {
        return None;
    }

    let mut candidate_dirs = vec![folder.to_path_buf()];
    for sub in &["bin", "Binaries/Win64", "bin/x64", "bin/win64", "bin/x86", "x64", "x86"] {
        let sub_path = folder.join(sub);
        if sub_path.is_dir() {
            candidate_dirs.push(sub_path);
        }
    }

    let mut best_exe: Option<PathBuf> = None;
    let mut largest_size = 0u64;

    for check_dir in candidate_dirs {
        if let Ok(entries) = fs::read_dir(&check_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    #[cfg(windows)]
                    let is_exec = name.ends_with(".exe");
                    #[cfg(not(windows))]
                    let is_exec = !name.contains('.') || name.ends_with(".bin") || name.ends_with(".app");

                    if is_exec {
                        let is_unwanted = name.starts_with("unins")
                            || name.starts_with("uninstall")
                            || name.starts_with("crashpad")
                            || name.starts_with("unitycrashhandler")
                            || name.starts_with("crashreport")
                            || name.starts_with("setup")
                            || name.starts_with("dxsetup")
                            || name.starts_with("vcredist")
                            || name.starts_with("notification_helper")
                            || name.ends_with("_cleanup.exe");

                        if !is_unwanted {
                            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                            if size > largest_size {
                                largest_size = size;
                                best_exe = Some(path);
                            }
                        }
                    }
                }
            }
        }
    }

    best_exe
}

/// Reads a protobuf varint from a byte slice.
fn read_varint(buf: &[u8], mut offset: usize) -> Option<(u64, usize)> {
    let mut res = 0u64;
    let mut shift = 0;
    while offset < buf.len() {
        let b = buf[offset];
        offset += 1;
        res |= ((b & 0x7f) as u64) << shift;
        if (b & 0x80) == 0 {
            return Some((res, offset));
        }
        shift += 7;
        if shift > 64 {
            return None;
        }
    }
    None
}

// ---------------------------------------------------------------------------
// GOG Galaxy Client Discovery (galaxy-2.0.db)
// ---------------------------------------------------------------------------

fn gog_galaxy_db_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(windows)]
    {
        if let Ok(pd) = std::env::var("ProgramData") {
            paths.push(PathBuf::from(pd).join("GOG.com").join("Galaxy").join("storage").join("galaxy-2.0.db"));
        }
        paths.push(PathBuf::from(r"C:\ProgramData\GOG.com\Galaxy\storage\galaxy-2.0.db"));

        if let Some(local_appdata) = dirs::data_local_dir() {
            paths.push(local_appdata.join("GOG.com").join("Galaxy").join("storage").join("galaxy-2.0.db"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join("Library/Application Support/GOG.com/Galaxy/storage/galaxy-2.0.db"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(data) = dirs::data_dir() {
            paths.push(data.join("GOG.com/Galaxy/storage/galaxy-2.0.db"));
        }
    }

    paths
}

pub fn discover_gog_client_games() -> Vec<PlatformGame> {
    let mut discovered = Vec::new();
    let mut db_path_opt: Option<PathBuf> = None;

    for candidate in gog_galaxy_db_paths() {
        if candidate.is_file() {
            db_path_opt = Some(candidate);
            break;
        }
    }

    let Some(db_path) = db_path_opt else {
        return discovered;
    };

    let conn = match Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Could not open GOG Galaxy database at {}: {e}", db_path.display());
            return discovered;
        }
    };

    let query = "
        SELECT
            lr.releaseKey,
            gp.value,
            ibp.productId,
            ibp.installationPath
        FROM LibraryReleases lr
        JOIN GamePieces gp ON gp.releaseKey = lr.releaseKey
        LEFT JOIN InstalledBaseProducts ibp ON ibp.productId = CAST(SUBSTR(lr.releaseKey, 5) AS INTEGER)
        WHERE lr.releaseKey LIKE 'gog_%'
          AND gp.gamePieceTypeId = (SELECT id FROM GamePieceTypes WHERE type = 'title')
    ";

    let mut stmt = match conn.prepare(query) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Warning: Failed to prepare GOG Galaxy query: {e}");
            return discovered;
        }
    };

    let mut seen_ids = HashSet::new();

    let rows = stmt.query_map([], |row| {
        let release_key: String = row.get(0)?;
        let piece_value: String = row.get(1)?;
        let installed_id: Option<i64> = row.get(2)?;
        let install_path: Option<String> = row.get(3)?;
        Ok((release_key, piece_value, installed_id, install_path))
    });

    let Ok(rows) = rows else {
        return discovered;
    };

    for row in rows.flatten() {
        let (release_key, piece_value, installed_id, install_path) = row;

        let product_id = release_key.trim_start_matches("gog_");
        if product_id.is_empty() || !seen_ids.insert(product_id.to_string()) {
            continue;
        }

        let raw_title = serde_json::from_str::<Value>(&piece_value)
            .ok()
            .and_then(|v| v.get("title").and_then(Value::as_str).map(str::to_string))
            .unwrap_or(piece_value);

        let cleaned_title = raw_title
            .trim_end_matches(" - Amazon Prime")
            .trim_end_matches(" Amazon Prime")
            .trim();

        let clean_name = normalize_game_title(cleaned_title);
        let final_name = if clean_name.is_empty() {
            cleaned_title.to_string()
        } else {
            clean_name
        };

        let is_installed = installed_id.is_some()
            && install_path
                .as_ref()
                .map(|p| Path::new(p).exists())
                .unwrap_or(false);

        let (install_dir, exe, size_on_disk) = if is_installed {
            let path_buf = install_path.as_ref().map(PathBuf::from);
            let exe_path = path_buf.as_deref().and_then(locate_executable_in_folder);
            let size = exe_path
                .as_ref()
                .and_then(|p| p.metadata().ok())
                .map(|m| m.len());
            (
                install_path,
                exe_path.map(|p| p.display().to_string()),
                size,
            )
        } else {
            (None, None, None)
        };

        discovered.push(PlatformGame {
            name: final_name,
            platform: Platform::Gog,
            platform_id: Some(format!("gog:{product_id}")),
            install_dir,
            exe,
            size_on_disk,
            launch_uri: Some(format!("goggalaxy://openGameView/{product_id}")),
            source: Some("GOG Galaxy Client Database".to_string()),
            console: None,
            installed: is_installed,
            install_uri: Some(format!("goggalaxy://openGameView/{product_id}")),
            uninstall_uri: None,
        });
    }

    discovered
}

// ---------------------------------------------------------------------------
// itch.io Client Discovery (butler.db)
// ---------------------------------------------------------------------------

fn itch_butler_db_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(windows)]
    {
        if let Some(appdata) = dirs::data_dir() {
            paths.push(appdata.join("itch").join("db").join("butler.db"));
        }
        if let Some(local_appdata) = dirs::data_local_dir() {
            paths.push(local_appdata.join("itch").join("db").join("butler.db"));
        }
    }

    #[cfg(not(windows))]
    {
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".config/itch/db/butler.db"));
        }
    }

    paths
}

pub fn discover_itch_client_games() -> Vec<PlatformGame> {
    let mut discovered = Vec::new();
    let mut db_path_opt: Option<PathBuf> = None;

    for candidate in itch_butler_db_paths() {
        if candidate.is_file() {
            db_path_opt = Some(candidate);
            break;
        }
    }

    let Some(db_path) = db_path_opt else {
        return discovered;
    };

    let conn = match Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Could not open itch butler database at {}: {e}", db_path.display());
            return discovered;
        }
    };

    let query = "
        SELECT
            g.id,
            g.title,
            c.id,
            c.install_folder_name
        FROM games g
        LEFT JOIN caves c ON c.game_id = g.id
        WHERE g.classification = 'game' OR g.classification IS NULL
    ";

    let mut stmt = match conn.prepare(query) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Warning: Failed to prepare itch butler query: {e}");
            return discovered;
        }
    };

    let mut seen_ids = HashSet::new();

    let rows = stmt.query_map([], |row| {
        let game_id: i64 = row.get(0)?;
        let title: String = row.get(1)?;
        let cave_id: Option<String> = row.get(2)?;
        let folder_name: Option<String> = row.get(3)?;
        Ok((game_id, title, cave_id, folder_name))
    });

    let Ok(rows) = rows else {
        return discovered;
    };

    let mut search_dirs = Vec::new();
    if let Some(appdata) = dirs::data_dir() {
        search_dirs.push(appdata.join("itch").join("apps"));
    }
    if let Some(local_appdata) = dirs::data_local_dir() {
        search_dirs.push(local_appdata.join("itch").join("apps"));
    }

    for row in rows.flatten() {
        let (game_id, raw_title, cave_id, folder_name) = row;

        if !seen_ids.insert(game_id) {
            continue;
        }

        let clean_name = normalize_game_title(&raw_title);
        let final_name = if clean_name.is_empty() {
            raw_title
        } else {
            clean_name
        };

        let mut is_installed = cave_id.is_some();
        let mut install_dir = None;
        let mut exe = None;
        let mut size_on_disk = None;

        if let Some(ref sub) = folder_name {
            for sdir in &search_dirs {
                let candidate_folder = sdir.join(sub);
                if candidate_folder.is_dir() {
                    let exe_path = locate_executable_in_folder(&candidate_folder);
                    size_on_disk = exe_path
                        .as_ref()
                        .and_then(|p| p.metadata().ok())
                        .map(|m| m.len());
                    exe = exe_path.map(|p| p.display().to_string());
                    install_dir = Some(candidate_folder.display().to_string());
                    is_installed = true;
                    break;
                }
            }
        }

        discovered.push(PlatformGame {
            name: final_name,
            platform: Platform::Itch,
            platform_id: Some(format!("itch:{game_id}")),
            install_dir,
            exe,
            size_on_disk,
            launch_uri: Some(format!("itch://games/{game_id}")),
            source: Some("itch.io Client Database".to_string()),
            console: None,
            installed: is_installed,
            install_uri: Some(format!("itch://games/{game_id}")),
            uninstall_uri: None,
        });
    }

    discovered
}

// ---------------------------------------------------------------------------
// Epic Games / Legendary Discovery
// ---------------------------------------------------------------------------

pub fn discover_epic_client_games() -> Vec<PlatformGame> {
    let mut discovered = Vec::new();
    let mut seen_ids = HashSet::new();

    // 1. Check Legendary metadata directory (~/.config/legendary/metadata/*.json)
    if let Some(home) = dirs::home_dir() {
        let meta_dir = home.join(".config").join("legendary").join("metadata");
        let installed_file = home.join(".config").join("legendary").join("installed.json");

        let installed_apps: HashSet<String> = if installed_file.is_file() {
            fs::read_to_string(&installed_file)
                .ok()
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                .and_then(|v| v.as_object().map(|obj| obj.keys().cloned().collect()))
                .unwrap_or_default()
        } else {
            HashSet::new()
        };

        if meta_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&meta_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("json") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                                let app_name = json
                                    .get("app_name")
                                    .and_then(Value::as_str)
                                    .unwrap_or("");
                                if app_name.is_empty() || !seen_ids.insert(app_name.to_string()) {
                                    continue;
                                }

                                let app_title = json
                                    .get("app_title")
                                    .or_else(|| json.get("title"))
                                    .and_then(Value::as_str)
                                    .unwrap_or("");
                                if app_title.is_empty() {
                                    continue;
                                }

                                let is_dlc = json
                                    .get("metadata")
                                    .and_then(|m| m.get("isDlc"))
                                    .and_then(Value::as_bool)
                                    .unwrap_or(false);
                                let categories: Vec<String> = json
                                    .get("metadata")
                                    .and_then(|m| m.get("categories"))
                                    .and_then(Value::as_array)
                                    .map(|arr| {
                                        arr.iter()
                                            .filter_map(|c| c.get("path").and_then(Value::as_str))
                                            .map(str::to_string)
                                            .collect()
                                    })
                                    .unwrap_or_default();
                                let is_addon = categories.iter().any(|c| {
                                    c == "addons"
                                        || c == "digitalextras"
                                        || c == "engines"
                                        || c == "engines/ue4"
                                });

                                if is_dlc || is_addon {
                                    continue;
                                }

                                let is_installed = installed_apps.contains(app_name);
                                let clean_name = normalize_game_title(app_title);
                                let final_name = if clean_name.is_empty() {
                                    app_title.to_string()
                                } else {
                                    clean_name
                                };

                                discovered.push(PlatformGame {
                                    name: final_name,
                                    platform: Platform::Epic,
                                    platform_id: Some(format!("epic:{app_name}")),
                                    install_dir: None,
                                    exe: None,
                                    size_on_disk: None,
                                    launch_uri: Some(format!(
                                        "com.epicgames.launcher://apps/{app_name}?action=launch"
                                    )),
                                    source: Some("Legendary Client Metadata".to_string()),
                                    console: None,
                                    installed: is_installed,
                                    install_uri: Some(format!(
                                        "com.epicgames.launcher://apps/{app_name}?action=launch"
                                    )),
                                    uninstall_uri: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Check Epic Games Launcher manifests (%ProgramData%\Epic\EpicGamesLauncher\Data\Manifests\*.item)
    #[cfg(windows)]
    {
        if let Ok(pd) = std::env::var("ProgramData") {
            let manifest_dir = PathBuf::from(pd)
                .join("Epic")
                .join("EpicGamesLauncher")
                .join("Data")
                .join("Manifests");
            if manifest_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&manifest_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) == Some("item") {
                            if let Ok(content) = fs::read_to_string(&path) {
                                if let Ok(json) = serde_json::from_str::<Value>(&content) {
                                    let app_name = json
                                        .get("AppName")
                                        .and_then(Value::as_str)
                                        .unwrap_or("");
                                    let display_name = json
                                        .get("DisplayName")
                                        .and_then(Value::as_str)
                                        .unwrap_or("");
                                    let install_loc = json
                                        .get("InstallLocation")
                                        .and_then(Value::as_str);

                                    if !app_name.is_empty() && !display_name.is_empty() {
                                        let clean_name = normalize_game_title(display_name);
                                        let final_name = if clean_name.is_empty() {
                                            display_name.to_string()
                                        } else {
                                            clean_name
                                        };

                                        let is_installed = install_loc
                                            .map(|p| Path::new(p).exists())
                                            .unwrap_or(false);

                                        let exe = install_loc
                                            .map(Path::new)
                                            .and_then(locate_executable_in_folder)
                                            .map(|p| p.display().to_string());

                                        if seen_ids.insert(app_name.to_string()) {
                                            discovered.push(PlatformGame {
                                                name: final_name,
                                                platform: Platform::Epic,
                                                platform_id: Some(format!("epic:{app_name}")),
                                                install_dir: install_loc.map(str::to_string),
                                                exe,
                                                size_on_disk: None,
                                                launch_uri: Some(format!(
                                                    "com.epicgames.launcher://apps/{app_name}?action=launch"
                                                )),
                                                source: Some("Epic Games Launcher Manifest".to_string()),
                                                console: None,
                                                installed: is_installed,
                                                install_uri: Some(format!(
                                                    "com.epicgames.launcher://apps/{app_name}?action=launch"
                                                )),
                                                uninstall_uri: None,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    discovered
}

// ---------------------------------------------------------------------------
// EA Desktop / Origin Client Discovery
// ---------------------------------------------------------------------------

fn is_ea_dlc_or_addon(slug: &str) -> bool {
    let s = slug.to_lowercase();
    s.contains("-dlc")
        || s.contains("-pack")
        || s.contains("-content")
        || s.contains("-expansion")
        || s.contains("-upgrade")
        || s.contains("-bundle")
        || s.contains("-livery")
        || s.contains("-armor")
        || s.contains("-rewards")
        || s.contains("-mount")
        || s.contains("-weapons")
        || s.contains("-texture-pack")
        || s.contains("catchup")
        || s.contains("high-resolution")
        || s.contains("flames-of-the-inquisition")
        || s.contains("mini-john-cooper")
        || s.contains("infiniti-q60")
        || s.contains("bmw-m3")
        || s.contains("dodge-challenger")
        || s.contains("porsche-911")
        || s.contains("aston-martin")
        || s.contains("lamborghini")
        || s.contains("ferrari")
        || s.contains("jaguar")
        || s.contains("koenigsegg")
        || s.contains("black-emporium")
        || s.contains("exiled-prince")
        || s.contains("mark-of-the-assassin")
        || s.contains("legacy")
}

fn ea_slug_to_clean_title(slug: &str) -> String {
    let s = slug.trim().to_lowercase();
    match s.as_str() {
        "battlefield-1" => "Battlefield 1".to_string(),
        "battlefield-4" => "Battlefield 4".to_string(),
        "battlefield-5" => "Battlefield V".to_string(),
        "battlefield-2042" | "battlefield-2042-nvidia-ultimate-play-bundle-pc-ww-code-redemption-only" => "Battlefield 2042".to_string(),
        "burnout-paradise-remastered" => "Burnout Paradise Remastered".to_string(),
        "dead-space-2" => "Dead Space 2".to_string(),
        "dragon-age-2" => "Dragon Age II".to_string(),
        "dragon-age-inquisition" => "Dragon Age: Inquisition".to_string(),
        "need-for-speed-heat" => "Need for Speed Heat".to_string(),
        "need-for-speed-hot-pursuit-remastered" => "Need for Speed: Hot Pursuit Remastered".to_string(),
        "need-for-speed-most-wanted" => "Need for Speed: Most Wanted".to_string(),
        "need-for-speed-payback" => "Need for Speed Payback".to_string(),
        "need-for-speed-rivals" => "Need for Speed Rivals".to_string(),
        "star-wars-battlefront-2" => "Star Wars Battlefront II".to_string(),
        "star-wars-jedi-fallen-order" => "Star Wars Jedi: Fallen Order".to_string(),
        "the-sims-4" => "The Sims 4".to_string(),
        _ => {
            slug.split('-')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}

/// Discover all owned and installed EA games from EA Desktop client logs and manifests.
pub fn discover_ea_client_games() -> Vec<PlatformGame> {
    let mut discovered = Vec::new();
    let mut seen_canonical = HashSet::new();

    #[cfg(windows)]
    {
        let mut ea_offers: HashMap<String, String> = HashMap::new();

        // 1. Scan EA Desktop Logs for authoritative owned offerId and slug pairs
        if let Some(local_appdata) = dirs::data_local_dir() {
            let logs_dir = local_appdata.join("Electronic Arts").join("EA Desktop").join("Logs");
            if logs_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&logs_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) == Some("log") {
                            if let Ok(content) = fs::read_to_string(&path) {
                                for line in content.lines() {
                                    // Match: offerId=[<id>] slug=[<slug>]
                                    if let Some(idx_offer) = line.find("offerId=[") {
                                        let rest = &line[idx_offer + 9..];
                                        if let Some(end_offer) = rest.find(']') {
                                            let offer_id = &rest[..end_offer];
                                            if let Some(idx_slug) = rest.find("slug=[") {
                                                let slug_rest = &rest[idx_slug + 6..];
                                                if let Some(end_slug) = slug_rest.find(']') {
                                                    let slug = &slug_rest[..end_slug];
                                                    if !is_ea_dlc_or_addon(slug) {
                                                        ea_offers.entry(slug.to_string()).or_insert_with(|| offer_id.to_string());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Scan EA Desktop InstallData directory (C:\ProgramData\EA Desktop\InstallData\*)
        let install_data_dir = PathBuf::from(r"C:\ProgramData\EA Desktop\InstallData");
        if install_data_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&install_data_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let folder_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();

                        if !folder_name.is_empty() {
                            let slug = folder_name.to_lowercase().replace(' ', "-");
                            if !is_ea_dlc_or_addon(&slug) {
                                ea_offers.entry(slug).or_insert_with(|| folder_name);
                            }
                        }
                    }
                }
            }
        }

        // 3. For each discovered owned game, check installation status across drives
        let drive_list = crate::plugins::engine::drive_roots();
        let subfolders = crate::plugins::engine::default_platform_subfolders("ea");

        for (slug, offer_id) in ea_offers {
            let title = ea_slug_to_clean_title(&slug);
            let canonical = crate::plugins::title_normalizer::canonical_title_key(&title);
            if !seen_canonical.insert(canonical) {
                continue;
            }

            let mut found_dir = None;
            let mut found_exe = None;

            for drive in &drive_list {
                for sub in &subfolders {
                    let candidate1 = drive.join(sub).join(&title);
                    let candidate2 = drive.join(sub).join(&offer_id);
                    let candidate3 = drive.join(sub).join(title.replace(':', ""));

                    for cand in &[candidate1, candidate2, candidate3] {
                        if cand.is_dir() {
                            let exe = locate_executable_in_folder(cand);
                            found_dir = Some(cand.display().to_string());
                            found_exe = exe.map(|p| p.display().to_string());
                            break;
                        }
                    }
                    if found_dir.is_some() {
                        break;
                    }
                }
                if found_dir.is_some() {
                    break;
                }
            }

            let is_installed = found_dir.is_some();
            let size_on_disk = found_exe
                .as_ref()
                .and_then(|p| fs::metadata(p).ok())
                .map(|m| m.len());

            discovered.push(PlatformGame {
                name: title,
                platform: Platform::Ea,
                platform_id: Some(format!("ea:{offer_id}")),
                install_dir: found_dir,
                exe: found_exe,
                size_on_disk,
                launch_uri: Some(format!("origin2://game/launch?offerIds={offer_id}")),
                source: Some("EA Desktop Client Logs & Manifests".to_string()),
                console: None,
                installed: is_installed,
                install_uri: Some(format!("origin2://game/launch?offerIds={offer_id}")),
                uninstall_uri: None,
            });
        }
    }

    discovered
}

// ---------------------------------------------------------------------------
// Ubisoft Connect Client Discovery (configurations & cache/ownership/*)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct UbisoftConfig {
    name: String,
    has_start_game: bool,
    is_dlc: bool,
}

pub fn discover_ubisoft_client_games() -> Vec<PlatformGame> {
    let mut discovered = Vec::new();

    #[cfg(windows)]
    {
        let mut configs: HashMap<u64, UbisoftConfig> = HashMap::new();
        let mut owned_ids: HashSet<u64> = HashSet::new();

        if let Some(local_appdata) = dirs::data_local_dir() {
            let ubi_dir = local_appdata.join("Ubisoft Game Launcher");

            // 1. Parse configurations protobuf store
            let cfg_path = ubi_dir.join("cache").join("configuration").join("configurations");
            if let Ok(raw) = fs::read(&cfg_path) {
                let mut pos = 0;
                while pos < raw.len() {
                    let Some((tag, next_pos)) = read_varint(&raw, pos) else { break; };
                    pos = next_pos;
                    let wire = tag & 7;
                    if wire == 0 {
                        let Some((_, next_pos)) = read_varint(&raw, pos) else { break; };
                        pos = next_pos;
                    } else if wire == 2 {
                        let Some((slen, next_pos)) = read_varint(&raw, pos) else { break; };
                        pos = next_pos;
                        let end = pos + slen as usize;
                        if end > raw.len() { break; }
                        let sub_bytes = &raw[pos..end];
                        pos = end;

                        let mut sp = 0;
                        let mut gid = None;
                        let mut uid = None;
                        let mut yaml_content: Option<String> = None;

                        while sp < sub_bytes.len() {
                            let Some((stag, snext)) = read_varint(sub_bytes, sp) else { break; };
                            sp = snext;
                            let swire = stag & 7;
                            let sfield = stag >> 3;
                            if swire == 0 {
                                let Some((sval, snext)) = read_varint(sub_bytes, sp) else { break; };
                                sp = snext;
                                if sfield == 1 {
                                    gid = Some(sval);
                                } else if sfield == 2 {
                                    uid = Some(sval);
                                }
                            } else if swire == 2 {
                                let Some((sslen, snext)) = read_varint(sub_bytes, sp) else { break; };
                                sp = snext;
                                let send = sp + sslen as usize;
                                if send > sub_bytes.len() { break; }
                                if sfield == 3 {
                                    yaml_content = String::from_utf8(sub_bytes[sp..send].to_vec()).ok();
                                }
                                sp = send;
                            } else {
                                break;
                            }
                        }

                        if let (Some(target_id), Some(content)) = (gid.or(uid), yaml_content) {
                            let has_start_game = content.contains("start_game:") || content.contains("executables:");
                            let is_dlc = content.contains("addon: yes")
                                || content.contains("type: dlc")
                                || content.contains("is_visible: no");

                            let mut name = String::new();
                            for line in content.lines() {
                                let trimmed = line.trim();
                                if let Some(n) = trimmed.strip_prefix("name:") {
                                    name = n.trim().trim_matches(|c| c == '"' || c == '\'').to_string();
                                    break;
                                }
                            }

                            if !name.is_empty()
                                && name != "GAMENAME"
                                && name != "NAME"
                                && name != "l1"
                                && !name.to_lowercase().starts_with("dlc")
                            {
                                configs.insert(target_id, UbisoftConfig {
                                    name,
                                    has_start_game,
                                    is_dlc,
                                });
                            }
                        }
                    } else {
                        break;
                    }
                }
            }

            // 2. Parse ownership stores (cache/ownership/*)
            let own_dir = ubi_dir.join("cache").join("ownership");
            if own_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&own_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            if let Ok(data) = fs::read(&path) {
                                // Find sequence of 0x08 <varint> 0x10 <varint> where both varints match
                                let mut i = 0;
                                while i + 5 < data.len() {
                                    if data[i] == 0x08 {
                                        if let Some((v1, np1)) = read_varint(&data, i + 1) {
                                            if np1 < data.len() && data[np1] == 0x10 {
                                                if let Some((v2, _)) = read_varint(&data, np1 + 1) {
                                                    if v1 == v2 && v1 > 0 {
                                                        owned_ids.insert(v1);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    i += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Query Windows Registry for installed Ubisoft games
        let mut installed_paths: HashMap<u64, String> = HashMap::new();
        {
            use winreg::enums::HKEY_LOCAL_MACHINE;
            use winreg::RegKey;

            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            if let Ok(installs) = hklm.open_subkey(r"SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs") {
                for key_name in installs.enum_keys().flatten() {
                    if let Ok(id) = key_name.parse::<u64>() {
                        if let Ok(sub) = installs.open_subkey(&key_name) {
                            if let Ok(dir) = sub.get_value::<String, _>("InstallDir") {
                                installed_paths.insert(id, dir);
                            }
                        }
                    }
                }
            }
        }

        // 4. Match owned genuine games and check filesystem
        let drive_list = crate::plugins::engine::drive_roots();
        let subfolders = crate::plugins::engine::default_platform_subfolders("ubisoft");
        let mut seen_canonical = HashSet::new();

        for pid in owned_ids {
            if let Some(cfg) = configs.get(&pid) {
                if !cfg.has_start_game || cfg.is_dlc {
                    continue;
                }

                let clean_name = normalize_game_title(&cfg.name);
                let final_name = if clean_name.is_empty() {
                    cfg.name.clone()
                } else {
                    clean_name
                };

                let canonical = crate::plugins::title_normalizer::canonical_title_key(&final_name);
                if !seen_canonical.insert(canonical) {
                    continue;
                }

                let mut found_dir = installed_paths.get(&pid).cloned();
                let mut found_exe = None;

                if let Some(ref dir_str) = found_dir {
                    let p = Path::new(dir_str);
                    if p.is_dir() {
                        found_exe = locate_executable_in_folder(p).map(|e| e.display().to_string());
                    } else {
                        found_dir = None;
                    }
                }

                if found_dir.is_none() {
                    for drive in &drive_list {
                        for sub in &subfolders {
                            let cand1 = drive.join(sub).join(&final_name);
                            let cand2 = drive.join(sub).join(final_name.replace(':', ""));
                            let cand3 = drive.join(sub).join(&cfg.name);

                            for cand in &[cand1, cand2, cand3] {
                                if cand.is_dir() {
                                    let exe = locate_executable_in_folder(cand);
                                    found_dir = Some(cand.display().to_string());
                                    found_exe = exe.map(|p| p.display().to_string());
                                    break;
                                }
                            }
                            if found_dir.is_some() {
                                break;
                            }
                        }
                        if found_dir.is_some() {
                            break;
                        }
                    }
                }

                let is_installed = found_dir.is_some();
                let size_on_disk = found_exe
                    .as_ref()
                    .and_then(|p| fs::metadata(p).ok())
                    .map(|m| m.len());

                discovered.push(PlatformGame {
                    name: final_name,
                    platform: Platform::Ubisoft,
                    platform_id: Some(format!("ubisoft:{pid}")),
                    install_dir: found_dir,
                    exe: found_exe,
                    size_on_disk,
                    launch_uri: Some(format!("uplay://launch/{pid}/0")),
                    source: Some("Ubisoft Connect Client Database".to_string()),
                    console: None,
                    installed: is_installed,
                    install_uri: Some(format!("uplay://launch/{pid}/0")),
                    uninstall_uri: None,
                });
            }
        }
    }

    discovered
}

// ---------------------------------------------------------------------------
// Xbox / Microsoft Store Client Discovery
// ---------------------------------------------------------------------------

pub fn discover_xbox_client_games() -> Vec<PlatformGame> {
    let mut discovered = Vec::new();

    #[cfg(windows)]
    {
        let mut game_product_ids: HashSet<String> = HashSet::new();

        // 1. Read persistent entitlements from Windows Store LocalCache
        if let Some(local_appdata) = dirs::data_local_dir() {
            let ws_cache_dir = local_appdata.join("Packages").join("Microsoft.WindowsStore_8wekyb3d8bbwe").join("LocalCache");
            if ws_cache_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&ws_cache_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if fname.ends_with("_PersistentEntitlementsCache.json") {
                            if let Ok(content) = fs::read_to_string(&path) {
                                if let Ok(json) = serde_json::from_str::<Value>(&content) {
                                    if let Some(ents) = json.get("Entitlements").and_then(Value::as_array) {
                                        for e in ents {
                                            let pkind = e.get("ProductKind").and_then(Value::as_str).unwrap_or("");
                                            let pfamily = e.get("ProductFamily").and_then(Value::as_str).unwrap_or("");
                                            if (pkind.eq_ignore_ascii_case("game") || pfamily.eq_ignore_ascii_case("games"))
                                                && e.get("Status").and_then(Value::as_str) == Some("Active")
                                            {
                                                if let Some(pid) = e.get("ProductId").and_then(Value::as_str) {
                                                    if !pid.is_empty() {
                                                        game_product_ids.insert(pid.to_uppercase());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 2. Read Xbox App AsyncCache.db for exact game titles and metadata
            let xbox_db_path = local_appdata
                .join("Packages")
                .join("Microsoft.GamingApp_8wekyb3d8bbwe")
                .join("LocalState")
                .join("AsyncCache.db");

            if xbox_db_path.is_file() {
                if let Ok(conn) = Connection::open_with_flags(&xbox_db_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
                    let query = "SELECT key, value FROM AsyncCache WHERE scope = 'product_summary'";
                    if let Ok(mut stmt) = conn.prepare(query) {
                        let rows = stmt.query_map([], |row| {
                            let key: String = row.get(0)?;
                            let val: String = row.get(1)?;
                            Ok((key, val))
                        });

                        let drive_list = crate::plugins::engine::drive_roots();
                        let subfolders = crate::plugins::engine::default_platform_subfolders("xbox");
                        let mut seen_canonical = HashSet::new();

                        if let Ok(rows) = rows {
                            for row in rows.flatten() {
                                let (key, val) = row;
                                let pid_upper = key.to_uppercase();

                                // Include if entitlement exists or if productKind is Game in summary
                                let is_entitled = game_product_ids.contains(&pid_upper);

                                if let Ok(parsed) = serde_json::from_str::<Value>(&val) {
                                    let data = parsed.get("data").unwrap_or(&parsed);
                                    let pkind = data.get("productKind").and_then(Value::as_str).unwrap_or("");
                                    let is_game = pkind.eq_ignore_ascii_case("game");

                                    if !is_entitled && !is_game {
                                        continue;
                                    }

                                    let raw_title = data
                                        .get("title")
                                        .or_else(|| data.get("shortTitle"))
                                        .or_else(|| data.get("productTitle"))
                                        .and_then(Value::as_str)
                                        .unwrap_or("");

                                    if raw_title.is_empty() {
                                        continue;
                                    }

                                    // Reject sound tracks, addons, expansions, and DLCs
                                    let lower_title = raw_title.to_lowercase();
                                    if lower_title.contains("soundtrack")
                                        || lower_title.contains("original soundtrack")
                                        || lower_title.contains("season pass")
                                        || lower_title.contains("expansion pass")
                                        || lower_title.contains("artbook")
                                    {
                                        continue;
                                    }

                                    let clean_name = normalize_game_title(raw_title);
                                    let final_name = if clean_name.is_empty() {
                                        raw_title.to_string()
                                    } else {
                                        clean_name
                                    };

                                    let canonical = crate::plugins::title_normalizer::canonical_title_key(&final_name);
                                    if !seen_canonical.insert(canonical) {
                                        continue;
                                    }

                                    // Check installation on disk
                                    let mut found_dir = None;
                                    let mut found_exe = None;

                                    for drive in &drive_list {
                                        for sub in &subfolders {
                                            let cand1 = drive.join(sub).join(&final_name);
                                            let cand2 = drive.join(sub).join(final_name.replace(':', ""));
                                            let cand3 = drive.join(sub).join(raw_title);

                                            for cand in &[cand1, cand2, cand3] {
                                                if cand.is_dir() {
                                                    let exe = locate_executable_in_folder(cand);
                                                    found_dir = Some(cand.display().to_string());
                                                    found_exe = exe.map(|p| p.display().to_string());
                                                    break;
                                                }
                                            }
                                            if found_dir.is_some() {
                                                break;
                                            }
                                        }
                                        if found_dir.is_some() {
                                            break;
                                        }
                                    }

                                    let is_installed = found_dir.is_some();
                                    let size_on_disk = found_exe
                                        .as_ref()
                                        .and_then(|p| fs::metadata(p).ok())
                                        .map(|m| m.len());

                                    let store_uri = format!("ms-windows-store://pdp/?ProductId={pid_upper}");

                                    discovered.push(PlatformGame {
                                        name: final_name,
                                        platform: Platform::Xbox,
                                        platform_id: Some(format!("xbox:{pid_upper}")),
                                        install_dir: found_dir,
                                        exe: found_exe,
                                        size_on_disk,
                                        launch_uri: Some(store_uri.clone()),
                                        source: Some("Xbox / Microsoft Store Entitlements".to_string()),
                                        console: None,
                                        installed: is_installed,
                                        install_uri: Some(store_uri),
                                        uninstall_uri: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    discovered
}

// ---------------------------------------------------------------------------
// Battle.net Client Discovery (CachedData.db & Battle.net.config)
// ---------------------------------------------------------------------------

fn bnet_code_to_meta(code: &str) -> Option<(&'static str, &'static str)> {
    let lower = code.trim().to_lowercase();
    match lower.as_str() {
        "diablo3" | "d3" => Some(("Diablo III", "D3")),
        "diablo4" | "d4" => Some(("Diablo IV", "D4")),
        "d2r" | "osi" => Some(("Diablo II: Resurrected", "OSI")),
        "war3r" | "w3" => Some(("Warcraft III: Reforged", "W3")),
        "overwatch" | "pro" => Some(("Overwatch 2", "Pro")),
        "wow" => Some(("World of Warcraft", "WoW")),
        "wow_classic" => Some(("World of Warcraft Classic", "WoW")),
        "starcraft2" | "s2" => Some(("StarCraft II", "S2")),
        "starcraft" | "s1" => Some(("StarCraft: Remastered", "S1")),
        "hearthstone" | "wtcg" => Some(("Hearthstone", "WTCG")),
        "heroes" | "hero" => Some(("Heroes of the Storm", "Hero")),
        "odin" | "call_of_duty_mw" => Some(("Call of Duty: Modern Warfare", "ODIN")),
        "lazo" | "call_of_duty_bocw" => Some(("Call of Duty: Black Ops Cold War", "LAZO")),
        "fore" | "call_of_duty_vanguard" => Some(("Call of Duty: Vanguard", "FORE")),
        "jup" | "call_of_duty_mw2" => Some(("Call of Duty: Modern Warfare II", "JUP")),
        "wlby" | "crash4" => Some(("Crash Bandicoot 4: It's About Time", "WLBY")),
        _ => None,
    }
}

pub fn discover_battlenet_client_games() -> Vec<PlatformGame> {
    let mut discovered = Vec::new();

    #[cfg(windows)]
    {
        let mut bnet_codes: HashSet<String> = HashSet::new();

        // 1. Read CachedData.db for active service objects and breaking news URLs
        if let Some(local_appdata) = dirs::data_local_dir() {
            let db_path = local_appdata.join("Battle.net").join("CachedData.db");
            if db_path.is_file() {
                if let Ok(conn) = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
                    let query = "SELECT url FROM remote_objects";
                    if let Ok(mut stmt) = conn.prepare(query) {
                        let rows = stmt.query_map([], |row| row.get::<_, String>(0));
                        if let Ok(rows) = rows {
                            for url in rows.flatten() {
                                // Extract service code from https://breaking-news.support.blizzard.com/service/<code_here>/live/...
                                if let Some(idx) = url.find("/service/") {
                                    let rest = &url[idx + 9..];
                                    if let Some(slash_idx) = rest.find('/') {
                                        let code = &rest[..slash_idx];
                                        if code != "app" {
                                            bnet_codes.insert(code.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Read Battle.net.config for configured and played game clients
        if let Some(appdata) = dirs::data_dir() {
            let cfg_path = appdata.join("Battle.net").join("Battle.net.config");
            if cfg_path.is_file() {
                if let Ok(content) = fs::read_to_string(&cfg_path) {
                    if let Ok(json) = serde_json::from_str::<Value>(&content) {
                        if let Some(games_obj) = json.get("Games").and_then(Value::as_object) {
                            for (k, _) in games_obj {
                                if k != "battle_net" {
                                    bnet_codes.insert(k.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Match known Blizzard titles and check disk installation
        let drive_list = crate::plugins::engine::drive_roots();
        let subfolders = crate::plugins::engine::default_platform_subfolders("battlenet");
        let mut seen_canonical = HashSet::new();

        for code in bnet_codes {
            if let Some((title, launch_code)) = bnet_code_to_meta(&code) {
                let canonical = crate::plugins::title_normalizer::canonical_title_key(title);
                if !seen_canonical.insert(canonical) {
                    continue;
                }

                let mut found_dir = None;
                let mut found_exe = None;

                for drive in &drive_list {
                    for sub in &subfolders {
                        let cand1 = drive.join(sub).join(title);
                        let cand2 = drive.join(sub).join(title.replace(':', ""));
                        let cand3 = drive.join(sub).join(&code);

                        for cand in &[cand1, cand2, cand3] {
                            if cand.is_dir() {
                                let exe = locate_executable_in_folder(cand);
                                found_dir = Some(cand.display().to_string());
                                found_exe = exe.map(|p| p.display().to_string());
                                break;
                            }
                        }
                        if found_dir.is_some() {
                            break;
                        }
                    }
                    if found_dir.is_some() {
                        break;
                    }
                }

                let is_installed = found_dir.is_some();
                let size_on_disk = found_exe
                    .as_ref()
                    .and_then(|p| fs::metadata(p).ok())
                    .map(|m| m.len());

                let bnet_uri = format!("battlenet://{launch_code}");

                discovered.push(PlatformGame {
                    name: title.to_string(),
                    platform: Platform::BattleNet,
                    platform_id: Some(format!("battlenet:{launch_code}")),
                    install_dir: found_dir,
                    exe: found_exe,
                    size_on_disk,
                    launch_uri: Some(bnet_uri.clone()),
                    source: Some("Battle.net Client Database".to_string()),
                    console: None,
                    installed: is_installed,
                    install_uri: Some(bnet_uri),
                    uninstall_uri: None,
                });
            }
        }
    }

    discovered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_discovery_runs_cleanly() {
        let gog_games = discover_gog_client_games();
        println!("Discovered GOG games: {}", gog_games.len());
        for g in gog_games.iter().take(3) {
            assert_eq!(g.platform, Platform::Gog);
            assert!(g.platform_id.as_ref().unwrap().starts_with("gog:"));
        }

        let itch_games = discover_itch_client_games();
        println!("Discovered itch games: {}", itch_games.len());
        for g in itch_games.iter().take(3) {
            assert_eq!(g.platform, Platform::Itch);
            assert!(g.platform_id.as_ref().unwrap().starts_with("itch:"));
        }

        let epic_games = discover_epic_client_games();
        println!("Discovered Epic games: {}", epic_games.len());
        for g in epic_games.iter().take(3) {
            assert_eq!(g.platform, Platform::Epic);
            assert!(g.platform_id.as_ref().unwrap().starts_with("epic:"));
        }

        let ea_games = discover_ea_client_games();
        println!("Discovered EA games: {}", ea_games.len());
        for g in ea_games.iter().take(3) {
            assert_eq!(g.platform, Platform::Ea);
            assert!(g.platform_id.as_ref().unwrap().starts_with("ea:"));
        }

        let ubi_games = discover_ubisoft_client_games();
        println!("Discovered Ubisoft games: {}", ubi_games.len());
        for g in ubi_games.iter().take(3) {
            assert_eq!(g.platform, Platform::Ubisoft);
            assert!(g.platform_id.as_ref().unwrap().starts_with("ubisoft:"));
        }

        let xbox_games = discover_xbox_client_games();
        println!("Discovered Xbox games: {}", xbox_games.len());
        for g in xbox_games.iter().take(3) {
            assert_eq!(g.platform, Platform::Xbox);
            assert!(g.platform_id.as_ref().unwrap().starts_with("xbox:"));
        }

        let bnet_games = discover_battlenet_client_games();
        println!("Discovered Battle.net games: {}", bnet_games.len());
        for g in bnet_games.iter().take(3) {
            assert_eq!(g.platform, Platform::BattleNet);
            assert!(g.platform_id.as_ref().unwrap().starts_with("battlenet:"));
        }
    }
}
