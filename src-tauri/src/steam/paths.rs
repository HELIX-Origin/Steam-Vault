//! Steam installation discovery: root folder, library folders, and accounts.
//!
//! Discovery is intentionally tolerant. It prefers the authoritative Windows
//! registry value, then falls back to well-known filesystem locations, and
//! finally validates a candidate by checking for a `steamapps`/`config` folder.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::AppResult;
use crate::steam::models::{SteamAccount, SteamInstall, SteamLibrary};
use crate::steam::vdf;

/// Offset between a SteamID64 and a SteamID3 (account) number.
pub const STEAMID64_BASE: u64 = 76_561_197_960_265_728;

pub fn steamid64_to_id3(id64: u64) -> String {
    id64.saturating_sub(STEAMID64_BASE).to_string()
}

pub fn id3_to_steamid64(id3: &str) -> Option<u64> {
    id3.parse::<u64>().ok().map(|n| n + STEAMID64_BASE)
}

fn id3_from_id64_str(id64: &str) -> Option<String> {
    id64.parse::<u64>().ok().map(steamid64_to_id3)
}

/// Locate the root Steam installation folder, if present.
pub fn discover_steam_root() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        if let Some(path) = windows_registry_path() {
            return Some(path);
        }
    }

    candidate_paths()
        .into_iter()
        .find(|candidate| is_steam_root(candidate))
}

fn is_steam_root(path: &Path) -> bool {
    path.join("steamapps").is_dir() || path.join("config").is_dir()
}

#[cfg(windows)]
fn windows_registry_path() -> Option<PathBuf> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey("Software\\Valve\\Steam") {
        if let Ok(raw) = key.get_value::<String, _>("SteamPath") {
            let path = PathBuf::from(raw.replace('/', "\\"));
            if is_steam_root(&path) {
                return Some(path);
            }
        }
    }

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    for subkey in [
        "SOFTWARE\\WOW6432Node\\Valve\\Steam",
        "SOFTWARE\\Valve\\Steam",
    ] {
        if let Ok(key) = hklm.open_subkey(subkey) {
            if let Ok(raw) = key.get_value::<String, _>("InstallPath") {
                let path = PathBuf::from(raw.replace('/', "\\"));
                if is_steam_root(&path) {
                    return Some(path);
                }
            }
        }
    }
    None
}

fn candidate_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(windows)]
    {
        for base in ["C:\\Program Files (x86)", "C:\\Program Files", "C:\\Steam"] {
            paths.push(PathBuf::from(base).join("Steam"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join("Library/Application Support/Steam"));
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".steam/steam"));
            paths.push(home.join(".local/share/Steam"));
            paths.push(home.join(".steam/root"));
            paths.push(home.join(".var/app/com.valvesoftware.Steam/data/Steam"));
        }
    }

    paths
}

/// The `libraryfolders.vdf` file location for a Steam root.
pub fn library_folders_file(root: &Path) -> Option<PathBuf> {
    let primary = root.join("steamapps").join("libraryfolders.vdf");
    if primary.is_file() {
        return Some(primary);
    }
    let secondary = root.join("config").join("libraryfolders.vdf");
    if secondary.is_file() {
        return Some(secondary);
    }
    None
}

/// Read all registered library folders (supports both modern and legacy shapes).
pub fn read_library_folders(root: &Path) -> Vec<SteamLibrary> {
    let mut libraries = Vec::new();
    let Some(file) = library_folders_file(root) else {
        return libraries;
    };
    let Ok(text) = std::fs::read_to_string(&file) else {
        return libraries;
    };
    let Ok(doc) = vdf::parse(&text) else {
        return libraries;
    };
    let Some(obj) = doc.get("libraryfolders") else {
        return libraries;
    };

    for (key, value) in obj.iter() {
        let Ok(index) = key.parse::<u32>() else {
            continue;
        };
        match value {
            vdf::Vdf::String(path) => libraries.push(SteamLibrary {
                index,
                path: path.clone(),
                label: None,
                app_ids: Vec::new(),
            }),
            vdf::Vdf::Object(_) => {
                let path = value.get_str("path").unwrap_or_default().to_string();
                let label = value.get_str("label").map(str::to_string);
                let mut app_ids = Vec::new();
                if let Some(vdf::Vdf::Object(apps)) = value.get("apps") {
                    for app_key in apps.keys() {
                        if let Ok(id) = app_key.parse::<u32>() {
                            app_ids.push(id);
                        }
                    }
                }
                app_ids.sort_unstable();
                libraries.push(SteamLibrary {
                    index,
                    path,
                    label,
                    app_ids,
                });
            }
        }
    }

    libraries.sort_by_key(|library| library.index);
    libraries
}

/// Read accounts from `loginusers.vdf` merged with numeric `userdata/` folders.
pub fn read_accounts(root: &Path) -> Vec<SteamAccount> {
    let mut accounts: BTreeMap<String, SteamAccount> = BTreeMap::new();

    let login_users = root.join("config").join("loginusers.vdf");
    if let Ok(text) = std::fs::read_to_string(&login_users) {
        if let Ok(doc) = vdf::parse(&text) {
            if let Some(users) = doc.get("users") {
                for (id64, node) in users.iter() {
                    let id3 = id3_from_id64_str(id64).unwrap_or_else(|| id64.clone());
                    accounts.insert(
                        id3.clone(),
                        SteamAccount {
                            id3,
                            id64: Some(id64.clone()),
                            account_name: node.get_str("AccountName").map(str::to_string),
                            persona_name: node.get_str("PersonaName").map(str::to_string),
                        },
                    );
                }
            }
        }
    }

    let userdata = root.join("userdata");
    if let Ok(entries) = std::fs::read_dir(&userdata) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.chars().all(|c| c.is_ascii_digit()) && !accounts.contains_key(&name) {
                accounts.insert(
                    name.clone(),
                    SteamAccount {
                        id3: name.clone(),
                        id64: id3_to_steamid64(&name).map(|v| v.to_string()),
                        account_name: None,
                        persona_name: None,
                    },
                );
            }
        }
    }

    accounts.into_values().collect()
}

/// Build a full [`SteamInstall`] description for a Steam root.
pub fn load_install(root: &Path) -> AppResult<SteamInstall> {
    Ok(SteamInstall {
        root: root.to_string_lossy().to_string(),
        libraries: read_library_folders(root),
        accounts: read_accounts(root),
    })
}
