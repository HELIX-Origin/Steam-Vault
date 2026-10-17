//! User-manageable tools, binaries discovery, and generic package installation.
//!
//! Provides OS-native and user-configured directory management for helper binaries
//! and downloaded platform launchers, binary autodetection via PATH and system
//! stores, and generic silent package installation via winget.

#[allow(unused_imports)]
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::plugins::engine::drive_roots;

/// User-accessible directories for managed helper binaries and launchers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedDirectories {
    pub tools_dir: String,
    pub launchers_dir: String,
    pub winget_available: bool,
}

/// User-accessible directory for managed helper binaries and tools.
///
/// Respects user-configured `custom_tools_dir` from settings if provided;
/// otherwise uses standard application directories.
pub fn managed_tools_dir() -> PathBuf {
    if let Some(ref custom) = crate::settings::load().custom_tools_dir {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            let p = PathBuf::from(trimmed);
            let _ = std::fs::create_dir_all(&p);
            if p.is_dir() {
                return p;
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // 1. Check if the app's installation directory has a `tools` folder or is writable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(install_dir) = exe_path.parent() {
                let local_tools = install_dir.join("tools");
                if local_tools.is_dir() {
                    return local_tools;
                }
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
        let base = dirs::data_local_dir()
            .unwrap_or_else(|| dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")));
        let new_tools = base.join("universal-game-launcher").join("tools");
        if new_tools.is_dir() {
            return new_tools;
        }
        let legacy_tools = base.join("Steam-Vault").join("tools");
        if legacy_tools.is_dir() {
            return legacy_tools;
        }
        new_tools
    }
    #[cfg(target_os = "macos")]
    {
        let base = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let new_tools = base.join("Library/Application Support/universal-game-launcher/tools");
        if new_tools.is_dir() {
            return new_tools;
        }
        let legacy_tools = base.join("Library/Application Support/Steam-Vault/tools");
        if legacy_tools.is_dir() {
            return legacy_tools;
        }
        new_tools
    }
    #[cfg(target_os = "linux")]
    {
        let base = dirs::data_local_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".local/share"));
        let new_tools = base.join("universal-game-launcher").join("tools");
        if new_tools.is_dir() {
            return new_tools;
        }
        let legacy_tools = base.join("steam-vault").join("tools");
        if legacy_tools.is_dir() {
            return legacy_tools;
        }
        new_tools
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        PathBuf::from("./tools")
    }
}

/// User-accessible directory for downloaded and installed gaming platform launchers.
///
/// Respects user-configured `custom_launchers_dir` from settings if provided;
/// otherwise falls back cleanly to `<managed_tools_dir>/launchers`.
pub fn managed_launchers_dir() -> PathBuf {
    if let Some(ref custom) = crate::settings::load().custom_launchers_dir {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            let p = PathBuf::from(trimmed);
            let _ = std::fs::create_dir_all(&p);
            if p.is_dir() {
                return p;
            }
        }
    }

    let dir = managed_tools_dir().join("launchers");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Finds an executable binary on the system PATH.
pub fn find_binary_on_path(binary_name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(binary_name);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(target_os = "windows")]
        if !binary_name.ends_with(".exe") {
            let candidate_exe = dir.join(format!("{binary_name}.exe"));
            if candidate_exe.is_file() {
                return Some(candidate_exe);
            }
        }
    }
    None
}

/// Autodetect binary from PATH, custom path override, or managed directories.
///
/// Detecting the binary instead of fixed paths avoids requiring users to know custom paths.
/// An optional custom path is checked first if provided as a fallback override.
pub fn check_binary_installed(binaries: &[String], custom_path: Option<&str>) -> Option<String> {
    // 1. Optional user-specified custom path / binary override
    if let Some(custom) = custom_path {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            let p = PathBuf::from(trimmed);
            if p.is_file() {
                return Some(p.display().to_string());
            }
            if p.is_dir() {
                for bin in binaries {
                    let candidate = p.join(bin);
                    if candidate.is_file() {
                        return Some(candidate.display().to_string());
                    }
                    #[cfg(target_os = "windows")]
                    if !bin.ends_with(".exe") {
                        let candidate_exe = p.join(format!("{bin}.exe"));
                        if candidate_exe.is_file() {
                            return Some(candidate_exe.display().to_string());
                        }
                    }
                }
            }
            if let Some(found) = find_binary_on_path(trimmed) {
                return Some(found.display().to_string());
            }
        }
    }

    // 2. Autodetect binary directly from system PATH
    for bin in binaries {
        if let Some(found) = find_binary_on_path(bin) {
            return Some(found.display().to_string());
        }
    }

    // 3. Check user-manageable tools & launchers directories
    let tools_dir = managed_tools_dir();
    let launchers_dir = managed_launchers_dir();
    for dir in [&tools_dir, &launchers_dir] {
        if dir.is_dir() {
            for bin in binaries {
                let candidate = dir.join(bin);
                if candidate.is_file() {
                    return Some(candidate.display().to_string());
                }
                #[cfg(target_os = "windows")]
                if !bin.ends_with(".exe") {
                    let candidate_exe = dir.join(format!("{bin}.exe"));
                    if candidate_exe.is_file() {
                        return Some(candidate_exe.display().to_string());
                    }
                }
            }
        }
    }

    // 4. Check mounted drives and standard OS locations
    #[cfg(target_os = "windows")]
    {
        let drives = drive_roots();
        for drive in &drives {
            // 4a. Check top-level candidates and subdirectories in Program Files, Games, etc.
            for sub in [
                "Program Files",
                "Program Files (x86)",
                "Games",
                "tools",
                "bin",
            ] {
                let check_dir = drive.join(sub);
                if check_dir.is_dir() {
                    for bin in binaries {
                        let candidate = check_dir.join(bin);
                        if candidate.is_file() {
                            return Some(candidate.display().to_string());
                        }
                        if !bin.ends_with(".exe") {
                            let candidate_exe = check_dir.join(format!("{bin}.exe"));
                            if candidate_exe.is_file() {
                                return Some(candidate_exe.display().to_string());
                            }
                        }
                    }

                    // Check 1 level deep inside subdirectories
                    if let Ok(entries) = std::fs::read_dir(&check_dir) {
                        for entry in entries.flatten() {
                            let sub_folder = entry.path();
                            if sub_folder.is_dir() {
                                for bin in binaries {
                                    let cand = sub_folder.join(bin);
                                    if cand.is_file() {
                                        return Some(cand.display().to_string());
                                    }
                                    if !bin.ends_with(".exe") {
                                        let cand_exe = sub_folder.join(format!("{bin}.exe"));
                                        if cand_exe.is_file() {
                                            return Some(cand_exe.display().to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 4b. Known nested launcher paths across any mounted drive
            let known_nested = [
                "Games/Epic Games/Launcher/Portal/Binaries/Win64/EpicGamesLauncher.exe",
                "Program Files/Epic Games/Launcher/Portal/Binaries/Win64/EpicGamesLauncher.exe",
                "Program Files (x86)/Epic Games/Launcher/Portal/Binaries/Win64/EpicGamesLauncher.exe",
                "Games/Epic Games/Legendary/legendary.exe",
                "Games/GOG Galaxy/GalaxyClient.exe",
                "Program Files (x86)/GOG Galaxy/GalaxyClient.exe",
                "Program Files/GOG Galaxy/GalaxyClient.exe",
                "Games/ES-DE/ES-DE.exe",
                "ES-DE/ES-DE.exe",
                "Games/Ubisoft/Ubisoft Game Launcher/UbisoftConnect.exe",
                "Program Files (x86)/Ubisoft/Ubisoft Game Launcher/UbisoftConnect.exe",
                "Program Files/Ubisoft/Ubisoft Game Launcher/UbisoftConnect.exe",
                "Program Files/Electronic Arts/EA Desktop/EA Desktop/EALauncher.exe",
                "Program Files/Electronic Arts/EA Desktop/EA Desktop/EADesktop.exe",
                "Program Files (x86)/Electronic Arts/EA Desktop/EA Desktop/EALauncher.exe",
                "Program Files (x86)/Origin/Origin.exe",
                "Games/EA Games/EA Desktop/EALauncher.exe",
                "Program Files (x86)/Battle.net/Battle.net.exe",
                "Program Files/Battle.net/Battle.net.exe",
                "Games/itch/itch-setup.exe",
                "Games/itch/itch.exe",
                "Games/Dolphin/Dolphin.exe",
            ];
            for rel in known_nested {
                let p = drive.join(rel);
                if p.is_file() {
                    let file_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    for bin in binaries {
                        if file_name.eq_ignore_ascii_case(bin)
                            || (!bin.ends_with(".exe") && format!("{bin}.exe").eq_ignore_ascii_case(file_name))
                        {
                            return Some(p.display().to_string());
                        }
                    }
                }
            }
        }

        // 4c. Check User AppData directories (e.g. itch, Amazon Games)
        if let Some(local_app_data) = dirs::data_local_dir() {
            let user_candidates = [
                local_app_data.join("itch").join("itch-setup.exe"),
                local_app_data.join("itch").join("itch.exe"),
                local_app_data.join("Programs").join("itch").join("itch.exe"),
                local_app_data.join("Amazon Games").join("App").join("Amazon Games.exe"),
            ];
            for cand in user_candidates {
                if cand.is_file() {
                    let file_name = cand.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    for bin in binaries {
                        if file_name.eq_ignore_ascii_case(bin)
                            || (!bin.ends_with(".exe") && format!("{bin}.exe").eq_ignore_ascii_case(file_name))
                        {
                            return Some(cand.display().to_string());
                        }
                    }
                }
            }

            for bin in binaries {
                let candidate = local_app_data.join("Programs").join(bin);
                if candidate.is_file() {
                    return Some(candidate.display().to_string());
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        for prefix in ["/usr/local/bin", "/opt/homebrew/bin", "/Applications"] {
            for bin in binaries {
                let candidate = Path::new(prefix).join(bin);
                if candidate.is_file() {
                    return Some(candidate.display().to_string());
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        for prefix in ["/usr/bin", "/usr/local/bin", "/var/lib/flatpak/exports/bin"] {
            for bin in binaries {
                let candidate = Path::new(prefix).join(bin);
                if candidate.is_file() {
                    return Some(candidate.display().to_string());
                }
            }
        }
        if let Some(home) = dirs::home_dir() {
            for bin in binaries {
                let candidate = home.join(".local/bin").join(bin);
                if candidate.is_file() {
                    return Some(candidate.display().to_string());
                }
            }
        }
    }

    None
}

/// Checks if winget is installed and available on the host PATH.
pub fn is_winget_available() -> bool {
    find_binary_on_path("winget.exe").is_some() || find_binary_on_path("winget").is_some()
}

/// Silently install a package using winget into a user-manageable directory.
pub fn install_winget_package(package_id: &str, location: Option<&str>) -> AppResult<bool> {
    #[cfg(target_os = "windows")]
    {
        if !is_winget_available() {
            return Err(AppError::msg(
                "WinGet package manager is not available on this system.",
            ));
        }

        let target = location
            .map(PathBuf::from)
            .unwrap_or_else(managed_launchers_dir);
        let _ = std::fs::create_dir_all(&target);

        let mut cmd = std::process::Command::new("winget");
        cmd.args([
            "install",
            "--id",
            package_id,
            "--silent",
            "--accept-package-agreements",
            "--accept-source-agreements",
            "--disable-interactivity",
        ]);

        // Provide user-manageable location
        cmd.arg("--location");
        cmd.arg(&target);

        let output = cmd.output().map_err(|e| {
            AppError::msg(format!("Failed to execute winget for {package_id}: {e}"))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(AppError::msg(format!(
                "WinGet installation failed for {package_id}: {stdout} {stderr}"
            )));
        }

        Ok(true)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = package_id;
        let _ = location;
        Err(AppError::msg(
            "WinGet package installation is only supported on Windows.",
        ))
    }
}

/// Launcher status model for plugins and settings.
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
    pub supports_silent_install: bool,
    pub winget_package_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub launch_args: Option<String>,
}

/// Fallback detection for all platform launchers using binary discovery.
pub fn detect_all_launchers() -> Vec<LauncherStatus> {
    let winget_ready = is_winget_available();
    let entries = [
        ("steam", "Steam", vec!["steam.exe", "steam"], "https://store.steampowered.com/about/", Some("Valve.Steam")),
        ("epic", "Epic Games", vec!["EpicGamesLauncher.exe", "legendary.exe", "legendary"], "https://store.epicgames.com/download", Some("EpicGames.EpicGamesLauncher")),
        ("gog", "GOG Galaxy", vec!["GalaxyClient.exe", "GalaxyCommunication.exe"], "https://www.gog.com/galaxy", Some("GOG.Galaxy")),
        ("ea", "EA App", vec!["EALauncher.exe", "EADesktop.exe", "Origin.exe"], "https://www.ea.com/ea-app", Some("ElectronicArts.EADesktop")),
        ("ubisoft", "Ubisoft Connect", vec!["UbisoftConnect.exe", "upc.exe"], "https://ubisoftconnect.com/", Some("Ubisoft.Connect")),
        ("battlenet", "Battle.net", vec!["Battle.net.exe"], "https://www.battle.net/download/getInstaller", Some("Blizzard.BattleNet")),
        ("itch", "itch.io", vec!["itch-setup.exe", "itch.exe", "itch", "butler.exe", "butler"], "https://itch.io/app", Some("itchio.itch")),
        ("amazon", "Amazon Games", vec!["Amazon Games.exe", "AmazonGames.exe", "nile.exe", "nile"], "https://gaming.amazon.com/amazon-games-app", Some("Amazon.Games")),
        ("rockstar", "Rockstar Games Launcher", vec!["Launcher.exe", "RockstarService.exe"], "https://socialclub.rockstargames.com/rockstar-games-launcher", Some("RockstarGames.RockstarGamesLauncher")),
        ("xbox", "Xbox App", vec!["XboxApp.exe", "XboxPcApp.exe"], "https://www.xbox.com/apps/xbox-app-for-pc", Some("Microsoft.GamingApp")),
        ("emulationstation", "EmulationStation (ES-DE)", vec!["ES-DE.exe", "emulationstation.exe", "EmulationStation.exe"], "https://es-de.org", None),
        ("other", "Other Studios", vec![], "", None),
    ];

    entries.into_iter().map(|(id, name, bins, url, winget_id)| {
        if id == "other" {
            return LauncherStatus {
                id: id.to_string(),
                name: name.to_string(),
                installed: true,
                executable_path: None,
                alternative_method: Some("Direct Standalone Executable".to_string()),
                download_url: String::new(),
                alternative_url: None,
                alternative_label: None,
                supports_silent_install: false,
                winget_package_id: None,
                launch_args: None,
            };
        }
        let string_bins: Vec<String> = bins.into_iter().map(String::from).collect();
        let exe = check_binary_installed(&string_bins, None);
        let installed = exe.is_some();
        let alt = if id == "epic" && check_binary_installed(&["legendary.exe".into(), "legendary".into()], None).is_some() {
            Some("Legendary CLI & EOS Service".to_string())
        } else {
            None
        };
        let launch_args = if id == "itch" && exe.as_deref().map_or(false, |p| p.to_lowercase().ends_with("itch-setup.exe")) {
            Some("--prefer-launch --appname itch".to_string())
        } else {
            None
        };
        LauncherStatus {
            id: id.to_string(),
            name: name.to_string(),
            installed,
            executable_path: exe,
            alternative_method: alt,
            download_url: url.to_string(),
            alternative_url: if id == "epic" { Some("https://github.com/legendary-gl/legendary/releases".to_string()) } else { None },
            alternative_label: if id == "epic" { Some("Download Legendary CLI ↗".to_string()) } else { None },
            supports_silent_install: winget_ready && winget_id.is_some(),
            winget_package_id: winget_id.map(String::from),
            launch_args,
        }
    }).collect()
}
