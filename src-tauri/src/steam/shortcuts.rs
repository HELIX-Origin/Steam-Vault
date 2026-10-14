//! Non-Steam shortcut read/write via Steam's binary KeyValues `shortcuts.vdf`.
//!
//! Non-Steam games are stored as an ordered object of `"0"`, `"1"`, … entries.
//! Each entry carries the executable path, name, start directory, icon, and an
//! appid derived by Steam (crc32 of the executable + name, stored signed).

use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};
use crate::steam::binvdf::{self, Bvdf};
use crate::steam::models::{Game, GameKind};

pub fn shortcuts_path(root: &Path, id3: &str) -> PathBuf {
    root.join("userdata")
        .join(id3)
        .join("config")
        .join("shortcuts.vdf")
}

/// Read non-Steam shortcuts for an account.
pub fn load_shortcuts(root: &Path, id3: &str) -> AppResult<Vec<Game>> {
    let path = shortcuts_path(root, id3);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let bytes = std::fs::read(&path)?;
    let doc = binvdf::parse(&bytes)?;
    let Some(shortcuts) = doc.get("shortcuts") else {
        return Ok(Vec::new());
    };

    let mut games = Vec::new();
    for (_, entry) in shortcuts.as_object().into_iter().flatten() {
        let app_id = entry.get_i32("appid").unwrap_or_default() as u32;
        let name = entry.get_str("appname").unwrap_or_default().to_string();
        let exe = entry.get_str("exe").map(str::to_string);
        let start_dir = entry.get_str("StartDir").map(str::to_string);
        let icon = entry.get_str("icon").map(str::to_string);
        let hidden = entry.get_i32("IsHidden").unwrap_or(0) != 0;
        let last_played = entry.get_i32("LastPlayTime").map(|v| v as i64);

        games.push(Game {
            app_id,
            name,
            kind: GameKind::Shortcut,
            installed: true,
            hidden,
            last_played,
            shortcut_id: Some(app_id),
            exe,
            start_dir,
            icon,
            ..Default::default()
        });
    }
    Ok(games)
}

/// Persist non-Steam shortcuts, preserving on-disk order where possible.
pub fn save_shortcuts(root: &Path, id3: &str, games: &[Game]) -> AppResult<()> {
    if crate::steam::process::is_steam_running() {
        return Err(AppError::SteamRunning(
            "close Steam before saving non-Steam shortcuts".into(),
        ));
    }

    let path = shortcuts_path(root, id3);
    crate::backup::backup_file(&path)?;

    let mut entries: Vec<(String, Bvdf)> = Vec::new();
    for (index, game) in games.iter().enumerate() {
        let mut fields: Vec<(String, Bvdf)> = Vec::new();
        fields.push(("appid".into(), Bvdf::Int(game.app_id as i32)));
        fields.push(("AppName".into(), Bvdf::String(game.name.clone())));
        fields.push((
            "exe".into(),
            Bvdf::String(game.exe.clone().unwrap_or_default()),
        ));
        fields.push((
            "StartDir".into(),
            Bvdf::String(game.start_dir.clone().unwrap_or_default()),
        ));
        fields.push((
            "icon".into(),
            Bvdf::String(game.icon.clone().unwrap_or_default()),
        ));
        fields.push(("ShortcutPath".into(), Bvdf::String(String::new())));
        fields.push(("LaunchOptions".into(), Bvdf::String(String::new())));
        fields.push(("IsHidden".into(), Bvdf::Int(i32::from(game.hidden))));
        fields.push(("AllowDesktopConfig".into(), Bvdf::Int(1)));
        fields.push(("AllowOverlay".into(), Bvdf::Int(1)));
        fields.push(("OpenVR".into(), Bvdf::Int(0)));
        fields.push(("Devkit".into(), Bvdf::Int(0)));
        fields.push(("DevkitGameID".into(), Bvdf::String(String::new())));
        fields.push((
            "LastPlayTime".into(),
            Bvdf::Int(game.last_played.unwrap_or(0) as i32),
        ));
        fields.push(("tags".into(), Bvdf::Object(Vec::new())));
        entries.push((index.to_string(), Bvdf::Object(fields)));
    }

    let root_node = Bvdf::Object(vec![("shortcuts".into(), Bvdf::Object(entries))]);
    let bytes = binvdf::write(&root_node);
    std::fs::write(&path, bytes)?;
    Ok(())
}
