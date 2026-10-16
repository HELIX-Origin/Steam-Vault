//! Windows-specific platform and launcher detectors.
//!
//! Authoritative detection of installed games via Windows Registry,
//! ProgramData manifests, GOG Galaxy SQLite database, EA/Ubisoft/Rockstar
//! registries, and Microsoft Store / Xbox Package Manager.

#[cfg(target_os = "windows")]
mod imp {
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};

    use winreg::enums::*;
    use winreg::RegKey;

    use crate::platforms::detectors::{
        env_dir, extract_package_family_name, find_executable, icon_executable,
        is_blacklisted_root, is_game_candidate, join_exe, normalize_path,
        platform_for_publisher,
    };
    use crate::platforms::registry;
    use crate::platforms::{Detection, Platform, PlatformGame, ScanConfig};

    /// Detect all Windows-native platform launchers and games.
    pub fn detect_platform_launchers(
        config: &ScanConfig,
        mounted_drives: &[PathBuf],
    ) -> Vec<(Platform, Detection)> {
        let mut results = Vec::new();

        if config.is_enabled(Platform::Gog) {
            results.push((Platform::Gog, detect_gog()));
        }
        if config.is_enabled(Platform::Epic) {
            results.push((Platform::Epic, detect_epic()));
        }
        if config.is_enabled(Platform::Ubisoft) {
            results.push((Platform::Ubisoft, detect_ubisoft()));
        }
        if config.is_enabled(Platform::Ea) {
            results.push((Platform::Ea, detect_ea()));
        }
        if config.is_enabled(Platform::BattleNet) {
            results.push((Platform::BattleNet, detect_battlenet()));
        }
        if config.is_enabled(Platform::Rockstar) {
            results.push((Platform::Rockstar, detect_rockstar()));
        }
        if config.is_enabled(Platform::Amazon) {
            results.push((Platform::Amazon, detect_amazon()));
        }
        if config.is_enabled(Platform::Xbox) {
            results.push((Platform::Xbox, detect_xbox(mounted_drives)));
        }

        // Generic Windows registry uninstaller detection
        let reg_based = detect_registry_based();
        for (platform, detection) in reg_based {
            if config.is_enabled(platform) {
                results.push((platform, detection));
            }
        }

        results
    }

    /// Windows default library folders for each platform.
    pub fn default_library_folders(platform: Platform) -> Vec<String> {
        let mut out = Vec::new();
        match platform {
            Platform::Gog => {
                if let Some(pf) = env_dir("ProgramFiles") {
                    out.push(normalize_path(&pf.join("GOG Galaxy\\Games").display().to_string()));
                    out.push(normalize_path(&pf.join("GOG Games").display().to_string()));
                }
                if let Some(pfx86) = env_dir("ProgramFiles(x86)") {
                    out.push(normalize_path(&pfx86.join("GOG Galaxy\\Games").display().to_string()));
                    out.push(normalize_path(&pfx86.join("GOG Games").display().to_string()));
                }
            }
            Platform::Epic => {
                if let Some(pf) = env_dir("ProgramFiles") {
                    out.push(normalize_path(&pf.join("Epic Games").display().to_string()));
                }
            }
            Platform::Ubisoft => {
                if let Some(pfx86) = env_dir("ProgramFiles(x86)") {
                    out.push(normalize_path(
                        &pfx86.join("Ubisoft\\Ubisoft Game Launcher\\games").display().to_string(),
                    ));
                }
            }
            Platform::Ea => {
                if let Some(pf) = env_dir("ProgramFiles") {
                    out.push(normalize_path(&pf.join("EA Games").display().to_string()));
                }
                if let Some(pfx86) = env_dir("ProgramFiles(x86)") {
                    out.push(normalize_path(&pfx86.join("Origin Games").display().to_string()));
                }
            }
            Platform::BattleNet => {
                if let Some(pfx86) = env_dir("ProgramFiles(x86)") {
                    out.push(normalize_path(&pfx86.join("Battle.net").display().to_string()));
                }
            }
            Platform::Amazon => {
                if let Some(local) = env_dir("LOCALAPPDATA") {
                    out.push(normalize_path(&local.join("Amazon Games\\Library").display().to_string()));
                }
            }
            Platform::Rockstar => {
                if let Some(pf) = env_dir("ProgramFiles") {
                    out.push(normalize_path(&pf.join("Rockstar Games").display().to_string()));
                }
            }
            Platform::Xbox => {
                if let Some(sys_drive) = std::env::var_os("SystemDrive").map(PathBuf::from) {
                    out.push(normalize_path(&sys_drive.join("XboxGames").display().to_string()));
                }
            }
            Platform::Itch => {
                if let Some(appdata) = env_dir("APPDATA") {
                    out.push(normalize_path(&appdata.join("itch\\apps").display().to_string()));
                }
            }
            Platform::Emulator | Platform::Steam | Platform::Other => {}
        }
        out
    }

    // ---------------------------------------------------------------------------
    // GOG Galaxy & Registry
    // ---------------------------------------------------------------------------

    fn detect_gog() -> Detection {
        let mut detection = Detection::empty();
        let mut seen_ids = BTreeSet::new();

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
                    let mut installed_ids = BTreeSet::new();
                    if let Ok(mut stmt) = conn.prepare("SELECT productId FROM InstalledProducts") {
                        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, i64>(0)) {
                            for id in rows.flatten() {
                                installed_ids.insert(id.to_string());
                            }
                        }
                    }

                    let reg_installs: BTreeMap<String, registry::GogInstall> =
                        registry::gog_installs()
                            .into_iter()
                            .map(|g| (g.id.clone(), g))
                            .collect();

                    if let Ok(mut stmt) = conn.prepare(
                        "SELECT gp.productId, gp.title, gp.releaseKey \
                         FROM GamePieces gp \
                         JOIN Products p ON gp.productId = p.id \
                         WHERE gp.originalTitle IS NOT NULL",
                    ) {
                        if let Ok(rows) = stmt.query_map([], |row| {
                            let product_id: i64 = row.get(0)?;
                            let title: String = row.get(1)?;
                            let release_key: Option<String> = row.get(2)?;
                            Ok((product_id.to_string(), title, release_key))
                        }) {
                            for (product_id, title, _release_key) in rows.flatten() {
                                if !seen_ids.insert(product_id.clone()) {
                                    continue;
                                }
                                let is_installed = installed_ids.contains(&product_id);
                                let reg = reg_installs.get(&product_id);
                                let (exe, dir) = if let Some(r) = reg {
                                    (r.exe.clone(), r.path.clone())
                                } else {
                                    (None, None)
                                };

                                detection.games.push(PlatformGame {
                                    name: title,
                                    platform: Platform::Gog,
                                    platform_id: Some(product_id.clone()),
                                    install_dir: dir,
                                    exe,
                                    installed: is_installed,
                                    launch_uri: Some(format!("goggalaxy://openGameView/{product_id}")),
                                    source: Some("GOG Galaxy".into()),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
            }
        }

        // 2. Fall back to Registry for any installed GOG games not in DB
        let reg_installs = registry::gog_installs();
        if !reg_installs.is_empty() {
            detection
                .roots
                .push("HKEY_LOCAL_MACHINE\\SOFTWARE\\GOG.com\\Games".into());
        }
        for install in reg_installs {
            if !seen_ids.insert(install.id.clone()) {
                continue;
            }
            detection.games.push(PlatformGame {
                name: install.name,
                platform: Platform::Gog,
                platform_id: Some(install.id.clone()),
                install_dir: install.path,
                exe: install.exe,
                installed: true,
                launch_uri: Some(format!("goggalaxy://openGameView/{}", install.id)),
                source: Some("Windows Registry".into()),
                ..Default::default()
            });
        }

        detection
    }

    // ---------------------------------------------------------------------------
    // Epic Games
    // ---------------------------------------------------------------------------

    fn detect_epic() -> Detection {
        let mut detection = Detection::empty();
        if let Some(program_data) = env_dir("ProgramData") {
            let manifest_dir = program_data
                .join("Epic")
                .join("EpicGamesLauncher")
                .join("Data")
                .join("Manifests");
            if manifest_dir.is_dir() {
                detection.roots.push(manifest_dir.display().to_string());
                detection.games.extend(epic_manifests(&manifest_dir));
            }
        }
        detection
    }

    fn epic_from_manifest(path: &Path) -> Option<PlatformGame> {
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
            Some(join_exe(install_loc, launch_exe))
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
            launch_uri: Some(format!("com.epicgames.launcher://apps/{app_name}?action=launch&silent=true")),
            source: Some("Epic Manifest".into()),
            ..Default::default()
        })
    }

    fn epic_manifests(dir: &Path) -> Vec<PlatformGame> {
        let mut games = Vec::new();
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return games,
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().is_some_and(|ext| ext == "item") {
                if let Some(game) = epic_from_manifest(&p) {
                    games.push(game);
                }
            }
        }
        games
    }

    // ---------------------------------------------------------------------------
    // Ubisoft Connect
    // ---------------------------------------------------------------------------

    fn detect_ubisoft() -> Detection {
        let mut detection = Detection::empty();
        let mut seen_dirs = BTreeSet::new();

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
            let exe = find_executable(dir_path, 2);
            let game_name = install
                .name
                .or_else(|| {
                    dir_path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| install.id.clone());

            if is_game_candidate(&game_name, exe.as_deref(), Some(&norm_dir)) {
                detection.games.push(PlatformGame {
                    name: game_name,
                    platform: Platform::Ubisoft,
                    platform_id: Some(install.id.clone()),
                    install_dir: Some(norm_dir),
                    exe,
                    installed: true,
                    launch_uri: Some(format!("uplay://launch/{}/0", install.id)),
                    source: Some("Ubisoft Connect".into()),
                    ..Default::default()
                });
            }
        }

        detection
    }

    // ---------------------------------------------------------------------------
    // EA / Origin
    // ---------------------------------------------------------------------------

    fn detect_ea() -> Detection {
        let mut detection = Detection::empty();
        let mut seen_dirs = BTreeSet::new();

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
            let game_name = if !install.name.is_empty() {
                install.name.clone()
            } else {
                dir_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&install.name)
                    .to_string()
            };

            if is_game_candidate(&game_name, exe.as_deref(), Some(&norm_dir)) {
                detection.games.push(PlatformGame {
                    name: game_name.clone(),
                    platform: Platform::Ea,
                    platform_id: Some(game_name),
                    install_dir: Some(norm_dir),
                    exe,
                    installed: true,
                    launch_uri: None,
                    source: Some("EA App".into()),
                    ..Default::default()
                });
            }
        }

        detection
    }

    // ---------------------------------------------------------------------------
    // Battle.net
    // ---------------------------------------------------------------------------

    fn detect_battlenet() -> Detection {
        // Blizzard installs register themselves in the Windows Uninstall registry
        // with Publisher "Blizzard Entertainment" and are captured by detect_registry_based().
        Detection::empty()
    }

    // ---------------------------------------------------------------------------
    // Rockstar Games
    // ---------------------------------------------------------------------------

    fn detect_rockstar() -> Detection {
        #[allow(unused_mut)]
        let mut detection = Detection::empty();
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
                                    name: sub_name.clone(),
                                    platform: Platform::Rockstar,
                                    platform_id: Some(sub_name),
                                    install_dir: Some(norm),
                                    exe,
                                    installed: true,
                                    source: Some("Rockstar Games".into()),
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
    // Amazon Games
    // ---------------------------------------------------------------------------

    fn detect_amazon() -> Detection {
        let mut detection = Detection::empty();
        if let Some(local) = env_dir("LOCALAPPDATA") {
            let db_path = local
                .join("Amazon Games")
                .join("Data")
                .join("Games")
                .join("Sql")
                .join("app.db");
            if db_path.is_file() {
                detection.roots.push(db_path.display().to_string());
                if let Ok(conn) = rusqlite::Connection::open_with_flags(
                    &db_path,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                ) {
                    if let Ok(mut stmt) = conn.prepare(
                        "SELECT Id, ProductTitle, InstallDirectory FROM DbSet WHERE InstallDirectory IS NOT NULL",
                    ) {
                        if let Ok(rows) = stmt.query_map([], |row| {
                            let id: String = row.get(0)?;
                            let title: String = row.get(1)?;
                            let install_dir: String = row.get(2)?;
                            Ok((id, title, install_dir))
                        }) {
                            for (id, title, install_dir) in rows.flatten() {
                                let norm = normalize_path(&install_dir);
                                let p = Path::new(&norm);
                                if p.is_dir() {
                                    let exe = find_executable(p, 2);
                                    detection.games.push(PlatformGame {
                                        name: title,
                                        platform: Platform::Amazon,
                                        platform_id: Some(id.clone()),
                                        install_dir: Some(norm),
                                        exe,
                                        installed: true,
                                        launch_uri: Some(format!("amazon-games://play/{id}")),
                                        source: Some("Amazon Games".into()),
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
    // Xbox & Microsoft Store
    // ---------------------------------------------------------------------------

    fn detect_xbox(mounted_drives: &[PathBuf]) -> Detection {
        let mut detection = Detection::empty();
        scan_xbox_registry(&mut detection);

        for drive in mounted_drives {
            let root = drive.join("XboxGames");
            if root.is_dir() {
                detection.roots.push(normalize_path(&root.display().to_string()));
                scan_xbox_dir(&root, &mut detection);
            }
        }
        detection
    }

    fn scan_xbox_registry(detection: &mut Detection) {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = "Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\AppModel\\Repository\\Packages";
        if let Ok(key) = hkcu.open_subkey(path) {
            for pkg_name in key.enum_keys().flatten() {
                if let Ok(pkg_key) = key.open_subkey(&pkg_name) {
                    let display_name: String = pkg_key.get_value("DisplayName").unwrap_or_default();
                    let pfn: String = pkg_key
                        .get_value("PackageFamilyName")
                        .ok()
                        .or_else(|| extract_package_family_name(&pkg_name))
                        .unwrap_or_default();
                    let p_root: String = pkg_key.get_value("PackageRootFolder").unwrap_or_default();

                    if !display_name.is_empty() && !pfn.is_empty() && !p_root.is_empty() {
                        let norm = normalize_path(&p_root);
                        let p = Path::new(&norm);
                        if p.is_dir() && !display_name.starts_with('@') {
                            let is_game = if let Ok(app_key) =
                                pkg_key.open_subkey("App\\Capabilities\\URLAssociations")
                            {
                                app_key.enum_values().flatten().any(|(name, _)| {
                                    let lower = name.to_lowercase();
                                    lower.contains("xbox")
                                        || lower.contains("solitaire")
                                        || lower.contains("mahjong")
                                        || lower.contains("casual")
                                        || lower.contains("game")
                                })
                            } else {
                                false
                            };
                            let has_mgc = p.join("MicrosoftGame.config").is_file()
                                || p.join("Content\\MicrosoftGame.config").is_file();
                            if is_game || has_mgc {
                                let exe = find_executable(p, 2);
                                detection.games.push(PlatformGame {
                                    name: display_name,
                                    platform: Platform::Xbox,
                                    platform_id: Some(pfn.clone()),
                                    install_dir: Some(norm),
                                    exe,
                                    installed: true,
                                    launch_uri: Some(format!("shell:AppsFolder\\{pfn}!App")),
                                    source: Some("Xbox / Microsoft Store".into()),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    fn scan_xbox_dir(dir: &Path, detection: &mut Detection) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() && !is_blacklisted_root(&p) {
                let name = p
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let exe = find_executable(&p, 2);
                let norm = normalize_path(&p.display().to_string());
                detection.games.push(PlatformGame {
                    name: name.clone(),
                    platform: Platform::Xbox,
                    platform_id: Some(name),
                    install_dir: Some(norm),
                    exe,
                    installed: true,
                    source: Some("Xbox Library Folder".into()),
                    ..Default::default()
                });
            }
        }
    }

    // ---------------------------------------------------------------------------
    // Windows Registry generic uninstall entries
    // ---------------------------------------------------------------------------

    fn detect_registry_based() -> Vec<(Platform, Detection)> {
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
                uninstall_uri: entry.uninstall_string,
                ..Default::default()
            });
        }
        buckets.into_iter().collect()
    }
}

#[cfg(target_os = "windows")]
pub use imp::*;

#[cfg(not(target_os = "windows"))]
mod imp {
    use std::path::PathBuf;
    use crate::platforms::{Detection, Platform, ScanConfig};

    pub fn detect_platform_launchers(
        _config: &ScanConfig,
        _mounted_drives: &[PathBuf],
    ) -> Vec<(Platform, Detection)> {
        Vec::new()
    }

    pub fn default_library_folders(_platform: Platform) -> Vec<String> {
        Vec::new()
    }
}

#[cfg(not(target_os = "windows"))]
pub use imp::*;
