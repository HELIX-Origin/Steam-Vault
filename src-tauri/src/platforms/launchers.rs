//! Multi-platform launcher installation detection and download URLs.
//!
//! Checks whether supported gaming platform launchers are actually installed
//! on the host system across standard directories, registries, application
//! bundles, and external/mounted storage drives.
//!
//! Also detects alternative native services and tools (e.g. Legendary / Epic
//! Online Services for Epic Games, DRM-free offline installers for GOG, Butler
//! for itch.io, Heroic Games Launcher, Nile for Amazon Games) allowing platforms
//! to be enabled and operated directly without requiring heavy official desktop clients.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::platforms::detectors::drive_roots;
use crate::platforms::EmulatorConfig;

/// User-accessible directory for managed helper binaries and tools.
/// User-accessible directory for managed helper binaries and tools.
///
/// On Windows: supports storing either in the application's install location (`<install_dir>/tools`)
/// if writable/present, or falling back cleanly to user AppData (`<local_app_data>/Steam-Vault/tools`).
/// On macOS: stores in `~/Library/Application Support/Steam-Vault/tools`.
/// On Linux: stores in XDG Data Home (`~/.local/share/steam-vault/tools`).
pub fn managed_tools_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        // 1. Check if the app's installation directory has a `tools` folder or is writable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(install_dir) = exe_path.parent() {
                let local_tools = install_dir.join("tools");
                if local_tools.is_dir() {
                    return local_tools;
                }
                // Test if install dir is writable (e.g. portable installation or dev mode)
                let test_probe = install_dir.join(".write_test");
                if std::fs::write(&test_probe, b"").is_ok() {
                    let _ = std::fs::remove_file(&test_probe);
                    let _ = std::fs::create_dir_all(&local_tools);
                    if local_tools.is_dir() {
                        return local_tools;
                    }
                }
            }
        }

        // 2. User AppData directory
        dirs::data_local_dir()
            .unwrap_or_else(|| dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("Steam-Vault")
            .join("tools")
    }
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Library/Application Support/Steam-Vault/tools")
    }
    #[cfg(target_os = "linux")]
    {
        dirs::data_local_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".local/share"))
            .join("steam-vault")
            .join("tools")
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        PathBuf::from("./tools")
    }
}

/// Target path for the built-in Legendary CLI binary.
pub fn legendary_binary_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        // Check installation location first if present
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(install_dir) = exe_path.parent() {
                let install_bin = install_dir.join("tools\\legendary.exe");
                if install_bin.is_file() {
                    return install_bin;
                }
            }
        }
        // Check AppData tools folder
        if let Some(local_app_data) = dirs::data_local_dir() {
            let app_data_bin = local_app_data.join("Steam-Vault\\tools\\legendary.exe");
            if app_data_bin.is_file() {
                return app_data_bin;
            }
        }
        managed_tools_dir().join("legendary.exe")
    }
    #[cfg(not(target_os = "windows"))]
    {
        managed_tools_dir().join("legendary")
    }
}

/// Download and install the Legendary CLI out of the box and initialize Epic Online Services.
pub async fn install_legendary_cli() -> AppResult<LauncherStatus> {
    let bin_dir = managed_tools_dir();
    let _ = std::fs::create_dir_all(&bin_dir);
    let target_path = legendary_binary_path();

    let download_url = if cfg!(target_os = "windows") {
        "https://github.com/legendary-gl/legendary/releases/latest/download/legendary.exe"
    } else if cfg!(target_os = "macos") {
        "https://github.com/legendary-gl/legendary/releases/latest/download/legendary_macos"
    } else {
        #[cfg(target_arch = "aarch64")]
        {
            "https://github.com/legendary-gl/legendary/releases/latest/download/legendary_linux_arm64"
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            "https://github.com/legendary-gl/legendary/releases/latest/download/legendary"
        }
    };

    let client = crate::metadata::http_client();
    let res = client
        .get(download_url)
        .header("User-Agent", "Steam-Vault/1.2.0")
        .send()
        .await
        .map_err(|e| AppError::msg(format!("Failed to download Legendary CLI: {e}")))?;

    if !res.status().is_success() {
        return Err(AppError::msg(format!(
            "Failed to download Legendary CLI (HTTP {})",
            res.status()
        )));
    }

    let bytes = res
        .bytes()
        .await
        .map_err(|e| AppError::msg(format!("Failed to read Legendary binary: {e}")))?;

    std::fs::write(&target_path, &bytes)
        .map_err(|e| AppError::msg(format!("Failed to save Legendary binary: {e}")))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&target_path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(&target_path, perms);
        }
    }

    // Initialize Epic Online Services overlay & service via Legendary
    let _ = std::process::Command::new(&target_path)
        .args(["eos-overlay", "install", "-y"])
        .output();

    Ok(LauncherStatus {
        id: "epic".to_string(),
        name: launcher_name("epic").to_string(),
        installed: true,
        executable_path: Some(target_path.display().to_string()),
        alternative_method: Some("Legendary CLI & EOS Service (Built-in)".to_string()),
        download_url: launcher_download_url("epic").to_string(),
        alternative_url: Some("https://github.com/legendary-gl/legendary".to_string()),
        alternative_label: Some("Legendary CLI & EOS Service".to_string()),
    })
}

/// Status of Legendary Epic Games authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegendaryAuthStatus {
    pub is_installed: bool,
    pub is_logged_in: bool,
    pub account_name: Option<String>,
    pub account_id: Option<String>,
    pub auth_url: String,
}

pub const EPIC_AUTH_URL: &str =
    "https://www.epicgames.com/id/login?redirectUrl=https%3A%2F%2Fwww.epicgames.com%2Fid%2Fapi%2Fredirect%3FclientId%3D34a2c53738e1452097026042d6528137%26responseType%3Dcode";

pub fn get_legendary_auth_status() -> LegendaryAuthStatus {
    let bin = legendary_binary_path();
    let is_installed = bin.is_file();
    if !is_installed {
        return LegendaryAuthStatus {
            is_installed: false,
            is_logged_in: false,
            account_name: None,
            account_id: None,
            auth_url: EPIC_AUTH_URL.to_string(),
        };
    }

    let output = std::process::Command::new(&bin)
        .arg("status")
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        let full = format!("{stdout}\n{stderr}");

        let is_logged_in = !full.contains("not logged in") && (full.contains("Logged in as") || full.contains("Account:"));
        let mut account_name = None;
        let mut account_id = None;

        for line in full.lines() {
            if line.contains("Logged in as") {
                if let Some(pos) = line.find("Logged in as") {
                    let rest = line[pos + 12..].trim();
                    if let Some(open_paren) = rest.find('(') {
                        account_name = Some(rest[..open_paren].trim().to_string());
                        if let Some(close_paren) = rest.find(')') {
                            account_id = Some(rest[open_paren + 1..close_paren].trim().to_string());
                        }
                    } else {
                        account_name = Some(rest.to_string());
                    }
                }
            } else if line.trim_start().starts_with("Account:") {
                let rest = line.trim_start()["Account:".len()..].trim();
                if let Some(open_paren) = rest.find('(') {
                    account_name = Some(rest[..open_paren].trim().to_string());
                    if let Some(close_paren) = rest.find(')') {
                        account_id = Some(rest[open_paren + 1..close_paren].trim().to_string());
                    }
                } else {
                    account_name = Some(rest.to_string());
                }
            }
        }

        LegendaryAuthStatus {
            is_installed: true,
            is_logged_in,
            account_name,
            account_id,
            auth_url: EPIC_AUTH_URL.to_string(),
        }
    } else {
        LegendaryAuthStatus {
            is_installed: true,
            is_logged_in: false,
            account_name: None,
            account_id: None,
            auth_url: EPIC_AUTH_URL.to_string(),
        }
    }
}

pub fn legendary_authenticate(code: Option<String>, import_from_launcher: bool) -> AppResult<LegendaryAuthStatus> {
    let bin = legendary_binary_path();
    if !bin.is_file() {
        return Err(AppError::msg("Legendary CLI is not installed. Please click '⚡ Install Legendary CLI' first."));
    }

    if import_from_launcher {
        let output = std::process::Command::new(&bin)
            .args(["auth", "--import"])
            .output()
            .map_err(|e| AppError::msg(format!("Failed to run legendary auth --import: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(AppError::msg(format!(
                "Failed to import Epic Games credentials: {stderr} {stdout}"
            )));
        }
    } else if let Some(auth_code) = code {
        let clean_code = auth_code.trim();
        let extracted_code = if clean_code.contains("authorizationCode") {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(clean_code) {
                v.get("authorizationCode")
                    .and_then(|c| c.as_str())
                    .unwrap_or(clean_code)
                    .to_string()
            } else {
                clean_code.to_string()
            }
        } else {
            clean_code.to_string()
        };

        if extracted_code.is_empty() {
            return Err(AppError::msg("Authorization code is empty."));
        }

        let output = std::process::Command::new(&bin)
            .args(["auth", "--code", &extracted_code])
            .output()
            .map_err(|e| AppError::msg(format!("Failed to run legendary auth --code: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(AppError::msg(format!(
                "Authentication failed: {stderr} {stdout}"
            )));
        }
    } else {
        return Err(AppError::msg("Authorization code is required. Please paste the code from your browser."));
    }

    Ok(get_legendary_auth_status())
}

pub fn legendary_logout() -> AppResult<LegendaryAuthStatus> {
    let bin = legendary_binary_path();
    if bin.is_file() {
        let _ = std::process::Command::new(&bin)
            .args(["auth", "--delete"])
            .output();
    }
    Ok(get_legendary_auth_status())
}

/// Status and metadata for a supported external game launcher.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherStatus {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub executable_path: Option<String>,
    pub alternative_method: Option<String>,
    pub download_url: String,
    pub alternative_url: Option<String>,
    pub alternative_label: Option<String>,
}

/// Official download URLs for supported launchers.
pub fn launcher_download_url(platform_id: &str) -> &'static str {
    match platform_id {
        "gog" => "https://www.gog.com/galaxy",
        "epic" => "https://store.epicgames.com/download",
        "itch" => "https://itch.io/app",
        "ubisoft" => "https://ubisoftconnect.com/",
        "ea" => "https://www.ea.com/ea-app",
        "battlenet" => "https://www.battle.net/download/getInstaller",
        "amazon" => "https://gaming.amazon.com/amazon-games-app",
        "rockstar" => "https://socialclub.rockstargames.com/rockstar-games-launcher",
        "xbox" => {
            #[cfg(target_os = "windows")]
            {
                "https://www.xbox.com/apps/xbox-app-for-pc"
            }
            #[cfg(not(target_os = "windows"))]
            {
                "https://www.xbox.com/play"
            }
        }
        "emulator" => "https://es-de.org/",
        _ => "",
    }
}

/// Alternative download URLs and tools that allow native play without official desktop launchers.
pub fn launcher_alternative_info(platform_id: &str) -> (Option<&'static str>, Option<&'static str>) {
    match platform_id {
        "epic" => (
            Some("https://github.com/legendary-gl/legendary"),
            Some("Legendary CLI & EOS Service (No Launcher Required)"),
        ),
        _ => (None, None),
    }
}

/// Friendly names for supported launchers.
pub fn launcher_name(platform_id: &str) -> &'static str {
    match platform_id {
        "gog" => "GOG Galaxy",
        "epic" => "Epic Games Launcher",
        "itch" => "itch.io",
        "ubisoft" => "Ubisoft Connect",
        "ea" => "EA App",
        "battlenet" => "Battle.net",
        "amazon" => "Amazon Games",
        "rockstar" => "Rockstar Games Launcher",
        "xbox" => "Xbox App",
        "emulator" => "Emulators & ROMs",
        "other" => "Other Studios",
        _ => "Custom Launcher",
    }
}

/// Detect the installed status of all supported non-Steam launchers across the OS.
pub fn detect_all_launchers(emulator_configs: &[EmulatorConfig]) -> Vec<LauncherStatus> {
    let supported_platforms = [
        "gog", "epic", "itch", "ubisoft", "ea", "battlenet", "amazon", "rockstar", "xbox",
        "emulator", "other",
    ];

    let mounted_drives = drive_roots();

    supported_platforms
        .into_iter()
        .map(|id| {
            let (installed, exe_path, alt_method) = match id {
                "other" => (true, None, Some("Direct Standalone Executable".to_string())),
                "emulator" => {
                    let (inst, exe) = detect_emulator_installed(emulator_configs, &mounted_drives);
                    let alt = if inst { Some("Configured / Detected Emulator".to_string()) } else { None };
                    (inst, exe, alt)
                }
                _ => detect_platform_launcher_installed(id, &mounted_drives),
            };

            let (alt_url, alt_label) = launcher_alternative_info(id);

            LauncherStatus {
                id: id.to_string(),
                name: launcher_name(id).to_string(),
                installed,
                executable_path: exe_path,
                alternative_method: alt_method,
                download_url: launcher_download_url(id).to_string(),
                alternative_url: alt_url.map(str::to_string),
                alternative_label: alt_label.map(str::to_string),
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Windows Detection Logic
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
fn detect_platform_launcher_installed(id: &str, mounted_drives: &[PathBuf]) -> (bool, Option<String>, Option<String>) {
    use winreg::enums::*;
    use winreg::RegKey;

    match id {
        "gog" => {
            // 1. Official GOG Galaxy client registry paths
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            for sub in [
                "SOFTWARE\\WOW6432Node\\GOG.com\\GalaxyClient\\paths",
                "SOFTWARE\\GOG.com\\GalaxyClient\\paths",
            ] {
                if let Ok(key) = hklm.open_subkey(sub) {
                    if let Ok(client) = key.get_value::<String, _>("client") {
                        let path = PathBuf::from(&client);
                        let exe = if path.is_file() {
                            path
                        } else {
                            path.join("GalaxyClient.exe")
                        };
                        if exe.is_file() {
                            return (true, Some(exe.display().to_string()), None);
                        }
                    }
                }
            }

            // 2. Drive & standard dir scans for GalaxyClient.exe
            for root in mounted_drives {
                for candidate in [
                    root.join("Program Files (x86)\\GOG Galaxy\\GalaxyClient.exe"),
                    root.join("Program Files\\GOG Galaxy\\GalaxyClient.exe"),
                    root.join("GOG Galaxy\\GalaxyClient.exe"),
                ] {
                    if candidate.is_file() {
                        return (true, Some(candidate.display().to_string()), None);
                    }
                }
            }

            // 3. Alternative method: Standalone DRM-Free GOG Game installs in Registry
            let reg_gog = crate::platforms::registry::gog_installs();
            if !reg_gog.is_empty() {
                let first_exe = reg_gog.first().and_then(|g| g.exe.clone());
                return (true, first_exe, Some("DRM-Free Standalone Registry".to_string()));
            }

            // 4. Alternative method: Heroic Games Launcher (supports GOG natively)
            if let Some(heroic_exe) = find_heroic_windows(mounted_drives) {
                return (true, Some(heroic_exe), Some("Heroic Games Launcher".to_string()));
            }

            (false, None, None)
        }
        "epic" => {
            // 1. Official Epic Games Launcher
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            for sub in [
                "SOFTWARE\\WOW6432Node\\EpicGames\\Unreal Engine",
                "SOFTWARE\\EpicGames\\Unreal Engine",
            ] {
                if let Ok(key) = hklm.open_subkey(sub) {
                    if let Ok(app_data) = key.get_value::<String, _>("AppDataPath") {
                        let p = PathBuf::from(&app_data);
                        let exe = p.join("Launcher\\Portal\\Binaries\\Win64\\EpicGamesLauncher.exe");
                        if exe.is_file() {
                            return (true, Some(exe.display().to_string()), None);
                        }
                    }
                }
            }

            for root in mounted_drives {
                for candidate in [
                    root.join("Program Files (x86)\\Epic Games\\Launcher\\Portal\\Binaries\\Win64\\EpicGamesLauncher.exe"),
                    root.join("Program Files\\Epic Games\\Launcher\\Portal\\Binaries\\Win64\\EpicGamesLauncher.exe"),
                    root.join("Program Files (x86)\\Epic Games\\Launcher\\Portal\\Binaries\\Win32\\EpicGamesLauncher.exe"),
                    root.join("Epic Games\\Launcher\\Portal\\Binaries\\Win64\\EpicGamesLauncher.exe"),
                ] {
                    if candidate.is_file() {
                        return (true, Some(candidate.display().to_string()), None);
                    }
                }
            }

            // 2. Alternative method: Legendary CLI (native Epic Games service CLI)
            if let Some(legendary_exe) = find_legendary_windows(mounted_drives) {
                return (true, Some(legendary_exe), Some("Legendary CLI (Native EOS Service)".to_string()));
            }

            // 3. Alternative method: Epic Online Services or Local Manifest Store
            if let Some(prog_data) = crate::platforms::detectors::env_dir("ProgramData") {
                let manifests = prog_data.join("Epic\\EpicGamesLauncher\\Data\\Manifests");
                if manifests.is_dir() {
                    let has_items = std::fs::read_dir(&manifests)
                        .map(|entries| entries.filter_map(Result::ok).any(|e| e.path().extension().is_some_and(|x| x == "item")))
                        .unwrap_or(false);
                    if has_items {
                        return (true, None, Some("Epic Online Services / Native Manifests".to_string()));
                    }
                }
            }

            // 4. Alternative method: Heroic Games Launcher (embeds Legendary for Epic)
            if let Some(heroic_exe) = find_heroic_windows(mounted_drives) {
                return (true, Some(heroic_exe), Some("Heroic (Legendary Backend)".to_string()));
            }

            (false, None, None)
        }
        "itch" => {
            // 1. Official itch desktop app
            if let Some(local_app_data) = dirs::data_local_dir() {
                let exe = local_app_data.join("itch\\itch.exe");
                if exe.is_file() {
                    return (true, Some(exe.display().to_string()), None);
                }
                let prog_exe = local_app_data.join("Programs\\itch\\itch.exe");
                if prog_exe.is_file() {
                    return (true, Some(prog_exe.display().to_string()), None);
                }
            }

            for root in mounted_drives {
                for candidate in [
                    root.join("Program Files\\itch\\itch.exe"),
                    root.join("Program Files (x86)\\itch\\itch.exe"),
                    root.join("itch\\itch.exe"),
                ] {
                    if candidate.is_file() {
                        return (true, Some(candidate.display().to_string()), None);
                    }
                }
            }

            // 2. Alternative method: Butler CLI or itch library directory
            if let Some(app_data) = dirs::data_dir() {
                let butler = app_data.join("itch\\bin\\butler.exe");
                if butler.is_file() {
                    return (true, Some(butler.display().to_string()), Some("Butler CLI".to_string()));
                }
                let apps_dir = app_data.join("itch\\apps");
                if apps_dir.is_dir() {
                    return (true, None, Some("DRM-Free Local Manifests".to_string()));
                }
            }

            (false, None, None)
        }
        "ubisoft" => {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            for sub in [
                "SOFTWARE\\WOW6432Node\\Ubisoft\\Launcher",
                "SOFTWARE\\Ubisoft\\Launcher",
            ] {
                if let Ok(key) = hklm.open_subkey(sub) {
                    if let Ok(dir) = key.get_value::<String, _>("InstallDir") {
                        let path = PathBuf::from(&dir);
                        for exe_name in ["UbisoftConnect.exe", "upc.exe"] {
                            let exe = path.join(exe_name);
                            if exe.is_file() {
                                return (true, Some(exe.display().to_string()), None);
                            }
                        }
                    }
                }
            }

            for root in mounted_drives {
                for candidate in [
                    root.join("Program Files (x86)\\Ubisoft\\Ubisoft Game Launcher\\UbisoftConnect.exe"),
                    root.join("Program Files (x86)\\Ubisoft\\Ubisoft Game Launcher\\upc.exe"),
                    root.join("Program Files\\Ubisoft\\Ubisoft Game Launcher\\UbisoftConnect.exe"),
                    root.join("Ubisoft\\Ubisoft Game Launcher\\UbisoftConnect.exe"),
                ] {
                    if candidate.is_file() {
                        return (true, Some(candidate.display().to_string()), None);
                    }
                }
            }
            (false, None, None)
        }
        "ea" => {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            for sub in [
                "SOFTWARE\\WOW6432Node\\Electronic Arts\\EA Desktop",
                "SOFTWARE\\Electronic Arts\\EA Desktop",
            ] {
                if let Ok(key) = hklm.open_subkey(sub) {
                    for val_name in ["ClientPath", "DesktopAppPath", "LauncherPath"] {
                        if let Ok(client) = key.get_value::<String, _>(val_name) {
                            let path = PathBuf::from(&client);
                            let exe = if path.is_file() {
                                path
                            } else {
                                path.join("EADesktop.exe")
                            };
                            if exe.is_file() {
                                return (true, Some(exe.display().to_string()), None);
                            }
                        }
                    }
                }
            }

            for root in mounted_drives {
                for candidate in [
                    root.join("Program Files\\Electronic Arts\\EA Desktop\\EA Desktop\\EADesktop.exe"),
                    root.join("Program Files\\Electronic Arts\\EA Desktop\\EA Desktop\\EALauncher.exe"),
                    root.join("Program Files (x86)\\Electronic Arts\\EA Desktop\\EA Desktop\\EADesktop.exe"),
                    root.join("Program Files (x86)\\Origin\\Origin.exe"),
                    root.join("Origin\\Origin.exe"),
                ] {
                    if candidate.is_file() {
                        return (true, Some(candidate.display().to_string()), None);
                    }
                }
            }
            (false, None, None)
        }
        "battlenet" => {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            for sub in [
                "SOFTWARE\\WOW6432Node\\Blizzard Entertainment\\Battle.net\\Capabilities",
                "SOFTWARE\\Blizzard Entertainment\\Battle.net\\Capabilities",
            ] {
                if let Ok(key) = hklm.open_subkey(sub) {
                    if let Ok(app_path) = key.get_value::<String, _>("ApplicationIcon") {
                        let trimmed = app_path.split(',').next().unwrap_or(&app_path).trim_matches('"');
                        let p = PathBuf::from(trimmed);
                        if p.is_file() {
                            return (true, Some(p.display().to_string()), None);
                        }
                    }
                }
            }

            for root in mounted_drives {
                for candidate in [
                    root.join("Program Files (x86)\\Battle.net\\Battle.net.exe"),
                    root.join("Program Files\\Battle.net\\Battle.net.exe"),
                    root.join("Battle.net\\Battle.net.exe"),
                ] {
                    if candidate.is_file() {
                        return (true, Some(candidate.display().to_string()), None);
                    }
                }
            }
            (false, None, None)
        }
        "amazon" => {
            if let Some(local_app_data) = dirs::data_local_dir() {
                let exe = local_app_data.join("Amazon Games\\App\\Amazon Games.exe");
                if exe.is_file() {
                    return (true, Some(exe.display().to_string()), None);
                }
            }

            for root in mounted_drives {
                for candidate in [
                    root.join("Program Files (x86)\\Amazon Games\\App\\Amazon Games.exe"),
                    root.join("Program Files\\Amazon Games\\App\\Amazon Games.exe"),
                    root.join("Amazon Games\\App\\Amazon Games.exe"),
                ] {
                    if candidate.is_file() {
                        return (true, Some(candidate.display().to_string()), None);
                    }
                }
            }

            // Alternative method: Nile CLI (Amazon Games native CLI)
            if let Some(nile_exe) = find_binary_on_drives("nile.exe", mounted_drives) {
                return (true, Some(nile_exe), Some("Nile CLI".to_string()));
            }

            (false, None, None)
        }
        "rockstar" => {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            for sub in [
                "SOFTWARE\\WOW6432Node\\Rockstar Games\\Launcher",
                "SOFTWARE\\Rockstar Games\\Launcher",
            ] {
                if let Ok(key) = hklm.open_subkey(sub) {
                    if let Ok(dir) = key.get_value::<String, _>("InstallFolder") {
                        let exe = PathBuf::from(&dir).join("Launcher.exe");
                        if exe.is_file() {
                            return (true, Some(exe.display().to_string()), None);
                        }
                    }
                }
            }

            for root in mounted_drives {
                for candidate in [
                    root.join("Program Files\\Rockstar Games\\Launcher\\Launcher.exe"),
                    root.join("Program Files (x86)\\Rockstar Games\\Launcher\\Launcher.exe"),
                    root.join("Rockstar Games\\Launcher\\Launcher.exe"),
                ] {
                    if candidate.is_file() {
                        return (true, Some(candidate.display().to_string()), None);
                    }
                }
            }
            (false, None, None)
        }
        "xbox" => {
            // Check Windows GamingApp / Xbox app package registration
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let path = "Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\AppModel\\Repository\\Packages";
            if let Ok(key) = hkcu.open_subkey(path) {
                for pkg_name in key.enum_keys().flatten() {
                    let lower = pkg_name.to_lowercase();
                    if lower.starts_with("microsoft.gamingapp") || lower.starts_with("microsoft.xboxapp") {
                        if let Ok(pkg_key) = key.open_subkey(&pkg_name) {
                            if let Ok(root_folder) = pkg_key.get_value::<String, _>("PackageRootFolder") {
                                let p = PathBuf::from(&root_folder);
                                let exe = p.join("XboxPcApp.exe");
                                if exe.is_file() {
                                    return (true, Some(exe.display().to_string()), None);
                                }
                                if p.is_dir() {
                                    return (true, Some(p.display().to_string()), None);
                                }
                            }
                        }
                    }
                }
            }
            (false, None, None)
        }
        _ => (false, None, None),
    }
}

#[cfg(target_os = "windows")]
fn find_heroic_windows(mounted_drives: &[PathBuf]) -> Option<String> {
    if let Some(local_app_data) = dirs::data_local_dir() {
        let exe = local_app_data.join("Programs\\heroic\\Heroic.exe");
        if exe.is_file() {
            return Some(exe.display().to_string());
        }
    }
    for root in mounted_drives {
        for candidate in [
            root.join("Program Files\\Heroic\\Heroic.exe"),
            root.join("Program Files (x86)\\Heroic\\Heroic.exe"),
        ] {
            if candidate.is_file() {
                return Some(candidate.display().to_string());
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn find_legendary_windows(mounted_drives: &[PathBuf]) -> Option<String> {
    let built_in = legendary_binary_path();
    if built_in.is_file() {
        return Some(built_in.display().to_string());
    }

    // Check user profile and Python scripts
    if let Some(home) = dirs::home_dir() {
        for candidate in [
            home.join(".local\\bin\\legendary.exe"),
            home.join(".config\\legendary\\legendary.exe"),
        ] {
            if candidate.is_file() {
                return Some(candidate.display().to_string());
            }
        }
    }
    if let Some(local_app_data) = dirs::data_local_dir() {
        for candidate in [
            local_app_data.join("Programs\\legendary\\legendary.exe"),
            local_app_data.join("Programs\\Python\\Scripts\\legendary.exe"),
        ] {
            if candidate.is_file() {
                return Some(candidate.display().to_string());
            }
        }
    }
    find_binary_on_drives("legendary.exe", mounted_drives)
}

#[cfg(target_os = "windows")]
fn find_binary_on_drives(filename: &str, mounted_drives: &[PathBuf]) -> Option<String> {
    for root in mounted_drives {
        for sub in ["tools", "bin", "Program Files", "ProgramData\\chocolatey\\bin"] {
            let candidate = root.join(sub).join(filename);
            if candidate.is_file() {
                return Some(candidate.display().to_string());
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// macOS Detection Logic
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
fn detect_platform_launcher_installed(id: &str, _mounted_drives: &[PathBuf]) -> (bool, Option<String>, Option<String>) {
    let app_dirs = [
        PathBuf::from("/Applications"),
        dirs::home_dir().map(|h| h.join("Applications")).unwrap_or_default(),
        PathBuf::from("/System/Applications"),
    ];

    let bundle_names: &[&str] = match id {
        "gog" => &["GOG Galaxy.app"],
        "epic" => &["Epic Games Launcher.app"],
        "itch" => &["itch.app"],
        "battlenet" => &["Battle.net.app"],
        "ea" => &["EA App.app", "Origin.app"],
        "ubisoft" => &["Ubisoft Connect.app"],
        "amazon" => &["Amazon Games.app"],
        "rockstar" => &["Rockstar Games Launcher.app"],
        "xbox" => &[],
        _ => &[],
    };

    for dir in &app_dirs {
        for bundle in bundle_names {
            let path = dir.join(bundle);
            if path.is_dir() {
                return (true, Some(path.display().to_string()), None);
            }
        }
    }

    // Alternative macOS detection: Legendary CLI or Heroic.app
    if id == "epic" || id == "gog" {
        let built_in = legendary_binary_path();
        if id == "epic" && built_in.is_file() {
            return (true, Some(built_in.display().to_string()), Some("Legendary CLI & EOS Service (Built-in)".to_string()));
        }
        for dir in &app_dirs {
            let heroic = dir.join("Heroic.app");
            if heroic.is_dir() {
                return (true, Some(heroic.display().to_string()), Some("Heroic Games Launcher".to_string()));
            }
        }
        if id == "epic" {
            for bin in ["/usr/local/bin/legendary", "/opt/homebrew/bin/legendary"] {
                if Path::new(bin).is_file() {
                    return (true, Some(bin.to_string()), Some("Legendary CLI (Homebrew / Native)".to_string()));
                }
            }
        }
    }

    (false, None, None)
}

// ---------------------------------------------------------------------------
// Linux Detection Logic
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn detect_platform_launcher_installed(id: &str, _mounted_drives: &[PathBuf]) -> (bool, Option<String>, Option<String>) {
    // Check standard executable binaries in PATH or known directories
    let bins: &[(&str, Option<&str>)] = match id {
        "gog" => &[
            ("gog-galaxy", None),
            ("heroic", Some("Heroic Games Launcher")),
            ("minigalaxy", Some("Minigalaxy Client")),
            ("lutris", Some("Lutris Gaming Platform")),
        ],
        "epic" => &[
            ("legendary", Some("Legendary CLI (Native EOS Service)")),
            ("heroic", Some("Heroic Games Launcher")),
        ],
        "itch" => &[("itch", None)],
        "battlenet" => &[("lutris", Some("Lutris Runner")), ("bottles", Some("Bottles Gaming Prefix"))],
        "ea" => &[("lutris", Some("Lutris Runner")), ("bottles", Some("Bottles Gaming Prefix"))],
        "ubisoft" => &[("lutris", Some("Lutris Runner")), ("bottles", Some("Bottles Gaming Prefix"))],
        "amazon" => &[("nile", Some("Nile CLI")), ("heroic", Some("Heroic Games Launcher")), ("lutris", Some("Lutris Runner"))],
        "rockstar" => &[("lutris", Some("Lutris Runner")), ("bottles", Some("Bottles Gaming Prefix"))],
        "xbox" => &[],
        _ => &[],
    };

    if id == "epic" {
        let built_in = legendary_binary_path();
        if built_in.is_file() {
            return (true, Some(built_in.display().to_string()), Some("Legendary CLI & EOS Service (Built-in)".to_string()));
        }
    }

    for &(bin, alt) in bins {
        for prefix in ["/usr/bin", "/usr/local/bin"] {
            let p = Path::new(prefix).join(bin);
            if p.is_file() {
                return (true, Some(p.display().to_string()), alt.map(str::to_string));
            }
        }
        if let Some(home) = dirs::home_dir() {
            let local_bin = home.join(".local/bin").join(bin);
            if local_bin.is_file() {
                return (true, Some(local_bin.display().to_string()), alt.map(str::to_string));
            }
            // Check Flatpak exports
            let flatpak_bin = home.join(".local/share/flatpak/exports/bin").join(bin);
            if flatpak_bin.is_file() {
                return (true, Some(flatpak_bin.display().to_string()), alt.map(str::to_string));
            }
        }
    }

    // Check Flatpak apps
    let flatpak_ids: &[(&str, Option<&str>)] = match id {
        "gog" => &[
            ("com.heroicgameslauncher.hgl", Some("Heroic Games Launcher (Flatpak)")),
            ("net.lutris.Lutris", Some("Lutris (Flatpak)")),
        ],
        "epic" => &[
            ("com.heroicgameslauncher.hgl", Some("Heroic / Legendary (Flatpak)")),
        ],
        "itch" => &[("io.itch.itch", None)],
        "battlenet" | "ea" | "ubisoft" | "rockstar" => &[
            ("com.usebottles.bottles", Some("Bottles (Flatpak)")),
            ("net.lutris.Lutris", Some("Lutris (Flatpak)")),
        ],
        _ => &[],
    };

    if let Some(home) = dirs::home_dir() {
        for &(flatpak, alt) in flatpak_ids {
            let p = home.join(".var/app").join(flatpak);
            if p.is_dir() {
                return (true, Some(p.display().to_string()), alt.map(str::to_string));
            }
        }
    }

    (false, None, None)
}

// Fallback for non-supported OS targets (compile guard)
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn detect_platform_launcher_installed(_id: &str, _mounted_drives: &[PathBuf]) -> (bool, Option<String>, Option<String>) {
    (false, None, None)
}

// ---------------------------------------------------------------------------
// Emulator Detection Logic (Universal with OS adaptations)
// ---------------------------------------------------------------------------

fn detect_emulator_installed(
    emulator_configs: &[EmulatorConfig],
    mounted_drives: &[PathBuf],
) -> (bool, Option<String>) {
    // 1. Check if user configured any valid emulator executable
    for config in emulator_configs {
        let exe = config.executable_path.trim();
        if !exe.is_empty() && Path::new(exe).is_file() {
            return (true, Some(exe.to_string()));
        }
    }

    // 2. Scan standard directories & mounted drives for popular emulators
    #[cfg(target_os = "windows")]
    {
        let popular_exes = [
            "ES-DE.exe",
            "retroarch.exe",
            "Dolphin.exe",
            "pcsx2-qtx64.exe",
            "pcsx2.exe",
            "duckstation-qt-x64-ReleaseLTCG.exe",
            "PPSSPPWindows64.exe",
            "citra-qt.exe",
            "Cemu.exe",
            "Ryujinx.exe",
            "mGBA.exe",
        ];

        for root in mounted_drives {
            for sub in [
                "Program Files\\EmulationStation-DE",
                "Program Files\\RetroArch",
                "Program Files (x86)\\RetroArch",
                "Program Files\\Dolphin",
                "Program Files\\PCSX2",
                "Emulators",
                "Games\\Emulators",
            ] {
                let dir = root.join(sub);
                if dir.is_dir() {
                    for exe in popular_exes {
                        let candidate = dir.join(exe);
                        if candidate.is_file() {
                            return (true, Some(candidate.display().to_string()));
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        for app in [
            "/Applications/ES-DE.app",
            "/Applications/RetroArch.app",
            "/Applications/OpenEmu.app",
            "/Applications/Dolphin.app",
            "/Applications/PCSX2.app",
        ] {
            if Path::new(app).is_dir() {
                return (true, Some(app.to_string()));
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        for bin in ["es-de", "retroarch", "dolphin-emu", "pcsx2", "duckstation"] {
            for prefix in ["/usr/bin", "/usr/local/bin"] {
                let p = Path::new(prefix).join(bin);
                if p.is_file() {
                    return (true, Some(p.display().to_string()));
                }
            }
        }
    }

    (false, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launcher_urls_and_names() {
        assert_eq!(launcher_name("gog"), "GOG Galaxy");
        assert_eq!(launcher_download_url("gog"), "https://www.gog.com/galaxy");
        assert_eq!(launcher_name("epic"), "Epic Games Launcher");
        assert_eq!(launcher_name("itch"), "itch.io");
        assert_eq!(launcher_name("ubisoft"), "Ubisoft Connect");
        assert_eq!(launcher_name("ea"), "EA App");
        assert_eq!(launcher_name("battlenet"), "Battle.net");
    }

    #[test]
    fn test_detect_all_launchers_contains_all_platforms() {
        let statuses = detect_all_launchers(&[]);
        assert!(statuses.iter().any(|s| s.id == "gog"));
        assert!(statuses.iter().any(|s| s.id == "epic"));
        assert!(statuses.iter().any(|s| s.id == "itch"));
        assert!(statuses.iter().any(|s| s.id == "ubisoft"));
        assert!(statuses.iter().any(|s| s.id == "ea"));
        assert!(statuses.iter().any(|s| s.id == "battlenet"));
        assert!(statuses.iter().any(|s| s.id == "amazon"));
        assert!(statuses.iter().any(|s| s.id == "rockstar"));
        assert!(statuses.iter().any(|s| s.id == "xbox"));
        assert!(statuses.iter().any(|s| s.id == "emulator"));
        assert!(statuses.iter().any(|s| s.id == "other"));
    }
}
