//! Epic Games / Legendary CLI backend integration.
//!
//! Provides CLI resolution, OAuth authentication, and Epic Online Services overlay
//! management specifically for the Epic Games platform plugin.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::plugins::engine::drive_roots;
use crate::platforms::tools::{find_binary_on_path, managed_tools_dir};

/// Status of Legendary Epic Games authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegendaryAuthStatus {
    pub is_installed: bool,
    pub is_logged_in: bool,
    pub account_name: Option<String>,
    pub account_id: Option<String>,
    pub auth_url: String,
    pub direct_auth_url: String,
    pub executable_path: Option<String>,
}

/// Status of Epic Online Services (EOS) overlay.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EosOverlayStatus {
    pub is_installed: bool,
    pub is_enabled: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

pub const EPIC_AUTH_URL: &str = "https://legendary.gl/epiclogin";
pub const EPIC_DIRECT_AUTH_URL: &str =
    "https://www.epicgames.com/id/login?redirectUrl=https%3A%2F%2Fwww.epicgames.com%2Fid%2Fapi%2Fredirect%3FclientId%3D34a02cf8f4414e29b15921876da36f9a%26responseType%3Dcode";
pub const LEGENDARY_DOWNLOAD_URL: &str = "https://github.com/legendary-gl/legendary/releases";

/// Helper to resolve a potential file or directory into a valid legendary executable path.
pub fn resolve_legendary_executable(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        return Some(path.to_path_buf());
    }
    if path.is_dir() {
        for name in [
            "legendary.exe",
            "legendary_windows_x64.exe",
            "legendary_windows.exe",
            "legendary",
            "legendary_macos",
            "legendary_linux_arm64",
        ] {
            let candidate = path.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Target path for the Legendary CLI binary (respecting user settings, auto-detection, and built-in tools).
pub fn legendary_binary_path() -> PathBuf {
    // 1. User-configured custom path from settings (optional fallback override)
    if let Some(ref custom) = crate::settings::load().legendary_path {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            let p = PathBuf::from(trimmed);
            if let Some(resolved) = resolve_legendary_executable(&p) {
                return resolved;
            }
            if let Some(resolved) = find_binary_on_path(trimmed) {
                return resolved;
            }
        }
    }

    // 2. Autodetect binary from system PATH (most efficient, avoids needing custom paths)
    for name in ["legendary", "legendary.exe"] {
        if let Some(path) = find_binary_on_path(name) {
            return path;
        }
    }

    #[cfg(target_os = "windows")]
    {
        // 3. Scan mounted drives for standard installation directories
        let drives = drive_roots();
        for root in &drives {
            for sub in [
                "Games\\Epic Games\\Legendary",
                "Epic Games\\Legendary",
                "Games\\Legendary",
                "Legendary",
                "tools",
                "bin",
                "Program Files\\Legendary",
                "Program Files (x86)\\Legendary",
            ] {
                let dir = root.join(sub);
                if let Some(resolved) = resolve_legendary_executable(&dir) {
                    return resolved;
                }
            }
        }

        // 4. User profile & local AppData
        if let Some(home) = dirs::home_dir() {
            for sub in [".local\\bin", ".config\\legendary"] {
                if let Some(resolved) = resolve_legendary_executable(&home.join(sub)) {
                    return resolved;
                }
            }
        }
        if let Some(local_app_data) = dirs::data_local_dir() {
            for sub in [
                "universal-game-launcher\\tools",
                "Steam-Vault\\tools",
                "Programs\\legendary",
                "Programs\\Python\\Scripts",
            ] {
                if let Some(resolved) = resolve_legendary_executable(&local_app_data.join(sub)) {
                    return resolved;
                }
            }
        }

        // 5. Executable relative tools directory
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(install_dir) = exe_path.parent() {
                if let Some(resolved) = resolve_legendary_executable(&install_dir.join("tools")) {
                    return resolved;
                }
            }
        }

        managed_tools_dir().join("legendary.exe")
    }

    #[cfg(target_os = "macos")]
    {
        for p in [
            "/usr/local/bin/legendary",
            "/opt/homebrew/bin/legendary",
        ] {
            let path = PathBuf::from(p);
            if path.is_file() {
                return path;
            }
        }
        if let Some(home) = dirs::home_dir() {
            let candidate = home.join("Library/Application Support/Steam-Vault/tools/legendary");
            if candidate.is_file() {
                return candidate;
            }
        }
        managed_tools_dir().join("legendary")
    }

    #[cfg(target_os = "linux")]
    {
        for p in [
            "/usr/bin/legendary",
            "/usr/local/bin/legendary",
        ] {
            let path = PathBuf::from(p);
            if path.is_file() {
                return path;
            }
        }
        if let Some(home) = dirs::home_dir() {
            let candidate = home.join(".local/bin/legendary");
            if candidate.is_file() {
                return candidate;
            }
        }
        managed_tools_dir().join("legendary")
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        PathBuf::from("./tools/legendary")
    }
}

/// Download and install the Legendary CLI out of the box and initialize Epic Online Services.
pub async fn install_legendary_cli() -> AppResult<crate::platforms::tools::LauncherStatus> {
    let bin_dir = managed_tools_dir();
    let _ = std::fs::create_dir_all(&bin_dir);
    let target_path = bin_dir.join(if cfg!(target_os = "windows") { "legendary.exe" } else { "legendary" });

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
        .header("User-Agent", "Universal-Game-Launcher/1.3.3")
        .send()
        .await
        .map_err(|e| AppError::msg(format!("Failed to connect to Legendary release server: {e}")))?;

    if !res.status().is_success() {
        return Err(AppError::msg(format!(
            "Download failed with HTTP status {}",
            res.status()
        )));
    }

    let bytes = res
        .bytes()
        .await
        .map_err(|e| AppError::msg(format!("Failed to download Legendary binary bytes: {e}")))?;

    if bytes.len() < 100_000 {
        return Err(AppError::msg(
            "Downloaded file is suspiciously small or corrupted.",
        ));
    }

    std::fs::write(&target_path, &bytes)
        .map_err(|e| AppError::msg(format!("Failed to write binary to {}: {e}", target_path.display())))?;

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&target_path) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            let _ = std::fs::set_permissions(&target_path, permissions);
        }
    }

    let mut current_settings = crate::settings::load();
    current_settings.legendary_path = Some(target_path.display().to_string());
    let _ = crate::settings::save(&current_settings);

    Ok(crate::platforms::tools::LauncherStatus {
        id: "epic".to_string(),
        name: "Epic Games".to_string(),
        installed: true,
        executable_path: Some(target_path.display().to_string()),
        alternative_method: Some("Legendary CLI & EOS Service (Built-in)".to_string()),
        download_url: "https://store.epicgames.com/download".to_string(),
        alternative_url: Some(LEGENDARY_DOWNLOAD_URL.to_string()),
        alternative_label: Some("Download Legendary CLI ↗".to_string()),
        supports_silent_install: true,
        winget_package_id: Some("EpicGames.EpicGamesLauncher".to_string()),
        launch_args: None,
    })
}

pub fn get_legendary_auth_status() -> LegendaryAuthStatus {
    let bin = legendary_binary_path();
    let is_installed = bin.is_file();
    let executable_path = if is_installed {
        Some(bin.display().to_string())
    } else {
        None
    };

    if !is_installed {
        return LegendaryAuthStatus {
            is_installed: false,
            is_logged_in: false,
            account_name: None,
            account_id: None,
            auth_url: EPIC_AUTH_URL.to_string(),
            direct_auth_url: EPIC_DIRECT_AUTH_URL.to_string(),
            executable_path: None,
        };
    }

    let output = std::process::Command::new(&bin)
        .arg("status")
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        let full = format!("{stdout}\n{stderr}");

        let is_logged_in = !full.contains("not logged in")
            && (full.contains("Logged in as")
                || full.contains("Account:")
                || full.contains("Epic account:"));
        let mut account_name = None;
        let mut account_id = None;

        for line in full.lines() {
            let trimmed = line.trim();
            if trimmed.contains("Logged in as") {
                if let Some(pos) = trimmed.find("Logged in as") {
                    let rest = trimmed[pos + 12..].trim();
                    if let Some(open_paren) = rest.find('(') {
                        account_name = Some(rest[..open_paren].trim().to_string());
                        if let Some(close_paren) = rest.find(')') {
                            account_id = Some(rest[open_paren + 1..close_paren].trim().to_string());
                        }
                    } else {
                        account_name = Some(rest.to_string());
                    }
                }
            } else if trimmed.starts_with("Account:") {
                let rest = trimmed["Account:".len()..].trim();
                if let Some(open_paren) = rest.find('(') {
                    account_name = Some(rest[..open_paren].trim().to_string());
                    if let Some(close_paren) = rest.find(')') {
                        account_id = Some(rest[open_paren + 1..close_paren].trim().to_string());
                    }
                } else {
                    account_name = Some(rest.to_string());
                }
            } else if trimmed.starts_with("Epic account:") {
                let rest = trimmed["Epic account:".len()..].trim();
                if let Some(open_paren) = rest.find('(') {
                    account_name = Some(rest[..open_paren].trim().to_string());
                    if let Some(close_paren) = rest.find(')') {
                        account_id = Some(rest[open_paren + 1..close_paren].trim().to_string());
                    }
                } else {
                    account_name = Some(rest.to_string());
                }
            } else if trimmed.starts_with("Account ID:") {
                let rest = trimmed["Account ID:".len()..].trim();
                account_id = Some(rest.to_string());
            }
        }

        LegendaryAuthStatus {
            is_installed: true,
            is_logged_in,
            account_name,
            account_id,
            auth_url: EPIC_AUTH_URL.to_string(),
            direct_auth_url: EPIC_DIRECT_AUTH_URL.to_string(),
            executable_path,
        }
    } else {
        LegendaryAuthStatus {
            is_installed: true,
            is_logged_in: false,
            account_name: None,
            account_id: None,
            auth_url: EPIC_AUTH_URL.to_string(),
            direct_auth_url: EPIC_DIRECT_AUTH_URL.to_string(),
            executable_path,
        }
    }
}

pub fn legendary_authenticate(code: Option<String>, import_from_launcher: bool) -> AppResult<LegendaryAuthStatus> {
    let bin = legendary_binary_path();
    if !bin.is_file() {
        return Err(AppError::msg(
            "Legendary CLI is not detected or installed. Please configure your Legendary folder or click '⚡ Install Legendary CLI Automatically'."
        ));
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
                "Failed to import Epic Games credentials from official launcher: {stderr} {stdout}"
            )));
        }
        return Ok(get_legendary_auth_status());
    }

    if let Some(auth_code) = code {
        let trimmed = auth_code.trim();
        if trimmed.is_empty() {
            return Err(AppError::msg("Authorization code cannot be empty."));
        }

        let output = std::process::Command::new(&bin)
            .args(["auth", "--code", trimmed])
            .output()
            .map_err(|e| AppError::msg(format!("Failed to run legendary auth --code: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(AppError::msg(format!(
                "Failed to authenticate with Epic Games: {stderr} {stdout}"
            )));
        }
        return Ok(get_legendary_auth_status());
    }

    Err(AppError::msg("No authentication method provided."))
}

pub fn legendary_logout() -> AppResult<LegendaryAuthStatus> {
    let bin = legendary_binary_path();
    if !bin.is_file() {
        return Ok(get_legendary_auth_status());
    }

    let _ = std::process::Command::new(&bin)
        .args(["auth", "--delete"])
        .output();

    Ok(get_legendary_auth_status())
}

pub fn get_eos_overlay_status() -> EosOverlayStatus {
    let bin = legendary_binary_path();
    if !bin.is_file() {
        return EosOverlayStatus {
            is_installed: false,
            is_enabled: false,
            version: None,
            path: None,
        };
    }

    let output = std::process::Command::new(&bin)
        .args(["eos-overlay", "info"])
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        let full = format!("{stdout}\n{stderr}");

        let mut is_installed = false;
        let mut is_enabled = false;
        let mut version = None;
        let mut path = None;

        for line in full.lines() {
            let trimmed = line.trim();
            if trimmed.contains("Installed version:") {
                let v = trimmed.split("Installed version:").nth(1).unwrap_or("").trim();
                if !v.is_empty() && v != "None" {
                    is_installed = true;
                    version = Some(v.to_string());
                }
            } else if trimmed.contains("Installed path:") {
                let p = trimmed.split("Installed path:").nth(1).unwrap_or("").trim();
                if !p.is_empty() && p != "None" {
                    path = Some(p.to_string());
                }
            } else if trimmed.contains("Overlay enabled:") {
                let en = trimmed.split("Overlay enabled:").nth(1).unwrap_or("").trim().to_lowercase();
                if en.starts_with("yes") || en.starts_with("true") {
                    is_enabled = true;
                }
            }
        }

        EosOverlayStatus {
            is_installed,
            is_enabled,
            version,
            path,
        }
    } else {
        EosOverlayStatus {
            is_installed: false,
            is_enabled: false,
            version: None,
            path: None,
        }
    }
}

pub fn install_eos_overlay() -> AppResult<EosOverlayStatus> {
    let bin = legendary_binary_path();
    if !bin.is_file() {
        return Err(AppError::msg(
            "Legendary CLI is not installed or detected. Please configure or install Legendary first.",
        ));
    }

    let auth = get_legendary_auth_status();
    if !auth.is_logged_in {
        return Err(AppError::msg(
            "Authentication required: Please log into Epic Games via Legendary before downloading the Epic Online Services overlay.",
        ));
    }

    let install_output = std::process::Command::new(&bin)
        .args(["eos-overlay", "install", "-y"])
        .output()
        .map_err(|e| AppError::msg(format!("Failed to download/install EOS overlay: {e}")))?;

    if !install_output.status.success() {
        let stderr = String::from_utf8_lossy(&install_output.stderr);
        let stdout = String::from_utf8_lossy(&install_output.stdout);
        return Err(AppError::msg(format!(
            "Failed to install Epic Games overlay service: {stderr} {stdout}"
        )));
    }

    let _ = std::process::Command::new(&bin)
        .args(["eos-overlay", "enable"])
        .output();

    Ok(get_eos_overlay_status())
}
