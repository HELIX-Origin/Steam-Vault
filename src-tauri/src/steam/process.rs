//! Steam process detection.
//!
//! Collection writes must happen while Steam is closed, otherwise Steam
//! overwrites our changes on shutdown. We detect it cross-platform with
//! [`sysinfo`] and surface a clear error to the UI.

use sysinfo::System;

/// Returns `true` if any Steam process is currently running.
pub fn is_steam_running() -> bool {
    let system = System::new_all();
    system
        .processes()
        .values()
        .any(|process| is_steam_process(&process.name().to_string_lossy()))
}

fn is_steam_process(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower == "steam"
        || lower == "steam.exe"
        || lower == "steam_osx"
        || lower.starts_with("steamwebhelper")
}
