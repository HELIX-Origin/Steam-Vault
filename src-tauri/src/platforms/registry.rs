//! Windows registry access for external-platform discovery (Windows only).
//!
//! Two authoritative sources are read:
//! * the **Uninstall** registry (`…\CurrentVersion\Uninstall`), whose entries
//!   expose `DisplayName`, `Publisher`, `InstallLocation`, and `DisplayIcon`;
//! * GOG Galaxy's `SOFTWARE\GOG.com\Games\<id>` keys, written when a game is
//!   installed.

#![cfg(windows)]

use winreg::enums::*;
use winreg::RegKey;

/// A row from the Windows "Programs and Features" uninstall registry.
#[derive(Debug, Clone)]
pub struct UninstallEntry {
    pub display_name: String,
    pub publisher: Option<String>,
    pub install_location: Option<String>,
    pub display_icon: Option<String>,
}

/// An installed GOG game as recorded under `…\GOG.com\Games\<id>`.
#[derive(Debug, Clone)]
pub struct GogInstall {
    pub name: String,
    pub id: String,
    pub path: Option<String>,
    pub exe: Option<String>,
}

fn read_str(key: &RegKey, name: &str) -> Option<String> {
    key.get_value::<String, _>(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Enumerate uninstall entries from both registry views of HKLM and from HKCU.
pub fn uninstall_entries() -> Vec<UninstallEntry> {
    let mut out = Vec::new();
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    collect_uninstall(&hklm, KEY_WOW64_64KEY, &mut out);
    collect_uninstall(&hklm, KEY_WOW64_32KEY, &mut out);
    collect_uninstall(&hkcu, 0, &mut out);
    out
}

fn collect_uninstall(hive: &RegKey, view: u32, out: &mut Vec<UninstallEntry>) {
    let path = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall";
    let Ok(key) = hive.open_subkey_with_flags(path, KEY_READ | view) else {
        return;
    };
    for name in key.enum_keys().flatten() {
        let Ok(sub) = key.open_subkey(&name) else {
            continue;
        };
        // Updates and redistributables are listed under a parent; skip them.
        if read_str(&sub, "ParentKeyName").is_some() {
            continue;
        }
        let Some(display_name) = read_str(&sub, "DisplayName") else {
            continue;
        };
        out.push(UninstallEntry {
            display_name,
            publisher: read_str(&sub, "Publisher"),
            install_location: read_str(&sub, "InstallLocation"),
            display_icon: read_str(&sub, "DisplayIcon"),
        });
    }
}

/// Enumerate GOG Galaxy installs from both registry views and HKCU.
pub fn gog_installs() -> Vec<GogInstall> {
    let mut out: Vec<GogInstall> = Vec::new();
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let sources = [
        (&hklm, KEY_WOW64_32KEY),
        (&hklm, KEY_WOW64_64KEY),
        (&hkcu, 0),
    ];
    for (hive, view) in sources {
        let Ok(key) = hive.open_subkey_with_flags("SOFTWARE\\GOG.com\\Games", KEY_READ | view)
        else {
            continue;
        };
        for id in key.enum_keys().flatten() {
            if !seen.insert(id.clone()) {
                continue;
            }
            let Ok(sub) = key.open_subkey(&id) else {
                continue;
            };
            let Some(name) = read_str(&sub, "gameName") else {
                continue;
            };
            out.push(GogInstall {
                name,
                id,
                path: read_str(&sub, "path"),
                exe: read_str(&sub, "exe"),
            });
        }
    }
    out
}
