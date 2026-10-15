//! Non-Steam shortcut read/write via Steam's binary KeyValues `shortcuts.vdf`.
//!
//! Non-Steam games are stored as an ordered object of `"0"`, `"1"`, … entries.
//! Each entry carries the executable path, name, start directory, icon, and an
//! appid derived by Steam (`crc32(exe + name) | 0x80000000`, stored signed).
//!
//! Steam wraps the stored `Exe`/`StartDir` values in literal double quotes;
//! readers here strip them so the domain model always holds clean paths.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};
use crate::platforms::Platform;
use crate::steam::binvdf::{self, Bvdf};
use crate::steam::models::{Game, GameKind};
use crate::steam::process;

/// A non-Steam shortcut to append to `shortcuts.vdf`.
#[derive(Debug, Clone)]
pub struct NewShortcut {
    pub name: String,
    pub exe: String,
    pub start_dir: String,
    pub icon: String,
    pub launch_options: String,
    pub platform: Platform,
    pub tags: Vec<String>,
}

/// The result of importing one or more platform games as shortcuts.
#[derive(Debug, Clone)]
pub struct ImportOutcome {
    pub added: Vec<Game>,
    pub skipped: Vec<String>,
}

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

    let platform_map = crate::platforms::load_map();
    let mut games = Vec::new();
    for (_, entry) in shortcuts.as_object().into_iter().flatten() {
        let app_id = entry.get_i32("appid").unwrap_or_default() as u32;
        let name = get_str_any(entry, &["AppName", "appname"])
            .unwrap_or_default()
            .to_string();
        let exe = clean_path(get_str_any(entry, &["Exe", "exe"]));
        let start_dir = clean_path(get_str_any(entry, &["StartDir", "startdir"]));
        let icon = entry.get_str("icon").map(str::to_string);
        let hidden = entry.get_i32("IsHidden").unwrap_or(0) != 0;
        let last_played = entry.get_i32("LastPlayTime").map(|value| value as i64);

        let mut collections = Vec::new();
        if let Some(tags_node) = entry.get("tags") {
            for (_, val) in tags_node.as_object().into_iter().flatten() {
                if let Some(tag_str) = val.as_str() {
                    if !tag_str.is_empty() {
                        collections.push(tag_str.to_string());
                    }
                }
            }
        }

        games.push(Game {
            app_id,
            name,
            kind: GameKind::Shortcut,
            installed: true,
            collections,
            hidden,
            last_played,
            shortcut_id: Some(app_id),
            exe,
            start_dir,
            icon,
            platform: platform_map
                .get(&app_id)
                .copied()
                .unwrap_or(Platform::Other),
            ..Default::default()
        });
    }
    Ok(games)
}

/// Append new non-Steam shortcuts, preserving every existing entry untouched.
pub fn add_shortcuts(
    root: &Path,
    id3: &str,
    new_entries: &[NewShortcut],
) -> AppResult<ImportOutcome> {
    if process::is_steam_running() {
        return Err(AppError::SteamRunning(
            "close Steam before importing non-Steam games".into(),
        ));
    }

    let path = shortcuts_path(root, id3);
    let root_node = if path.is_file() {
        let bytes = std::fs::read(&path)?;
        binvdf::parse(&bytes)?
    } else {
        Bvdf::Object(Vec::new())
    };

    let mut top: Vec<(String, Bvdf)> = match root_node {
        Bvdf::Object(entries) => entries,
        _ => Vec::new(),
    };
    let position = top.iter().position(|(key, _)| key == "shortcuts");
    let mut shortcuts: Vec<(String, Bvdf)> = match position {
        Some(index) => match std::mem::replace(&mut top[index].1, Bvdf::Object(Vec::new())) {
            Bvdf::Object(entries) => entries,
            _ => Vec::new(),
        },
        None => Vec::new(),
    };

    let mut next_index = shortcuts
        .iter()
        .filter_map(|(key, _)| key.parse::<usize>().ok())
        .max()
        .map(|max| max + 1)
        .unwrap_or(0);
    let mut existing: BTreeSet<u32> = shortcuts
        .iter()
        .filter_map(|(_, entry)| entry.get_i32("appid").map(|value| value as u32))
        .collect();

    let mut added = Vec::new();
    let mut skipped = Vec::new();

    for candidate in new_entries {
        let app_id = shortcut_app_id(&candidate.exe, &candidate.name);
        if !existing.insert(app_id) {
            skipped.push(candidate.name.clone());
            continue;
        }
        let fields = build_fields(
            app_id,
            &candidate.name,
            &candidate.exe,
            &candidate.start_dir,
            &candidate.icon,
            &candidate.launch_options,
            false,
            0,
            &candidate.tags,
        );
        shortcuts.push((next_index.to_string(), Bvdf::Object(fields)));
        next_index += 1;
        added.push(Game {
            app_id,
            name: candidate.name.clone(),
            kind: GameKind::Shortcut,
            installed: true,
            collections: candidate.tags.clone(),
            shortcut_id: Some(app_id),
            exe: Some(candidate.exe.clone()),
            start_dir: Some(candidate.start_dir.clone()),
            icon: (!candidate.icon.is_empty()).then(|| candidate.icon.clone()),
            platform: candidate.platform,
            ..Default::default()
        });
    }

    if added.is_empty() {
        return Ok(ImportOutcome { added, skipped });
    }

    let shortcuts_node = Bvdf::Object(shortcuts);
    match position {
        Some(index) => top[index].1 = shortcuts_node,
        None => top.push(("shortcuts".into(), shortcuts_node)),
    }
    let bytes = binvdf::write(&Bvdf::Object(top));

    crate::backup::backup_file(&path)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, bytes)?;

    Ok(ImportOutcome { added, skipped })
}

/// Persist non-Steam shortcuts, preserving on-disk order where possible.
pub fn save_shortcuts(root: &Path, id3: &str, games: &[Game]) -> AppResult<()> {
    if process::is_steam_running() {
        return Err(AppError::SteamRunning(
            "close Steam before saving non-Steam shortcuts".into(),
        ));
    }

    let path = shortcuts_path(root, id3);
    crate::backup::backup_file(&path)?;

    let entries: Vec<(String, Bvdf)> = games
        .iter()
        .enumerate()
        .map(|(index, game)| {
            let fields = build_fields(
                game.app_id,
                &game.name,
                game.exe.as_deref().unwrap_or_default(),
                game.start_dir.as_deref().unwrap_or_default(),
                game.icon.as_deref().unwrap_or_default(),
                "",
                game.hidden,
                game.last_played.unwrap_or(0),
                &game.collections,
            );
            (index.to_string(), Bvdf::Object(fields))
        })
        .collect();

    let root_node = Bvdf::Object(vec![("shortcuts".into(), Bvdf::Object(entries))]);
    let bytes = binvdf::write(&root_node);
    std::fs::write(&path, bytes)?;
    Ok(())
}

/// The Steam shortcut appid: `crc32(exe + name)` with the high bit set.
pub fn shortcut_app_id(exe: &str, name: &str) -> u32 {
    crc32(format!("{exe}{name}").as_bytes()) | 0x8000_0000
}

/// Build the binary-VDF fields for one shortcut entry, matching Steam's schema.
#[allow(clippy::too_many_arguments)]
fn build_fields(
    app_id: u32,
    name: &str,
    exe: &str,
    start_dir: &str,
    icon: &str,
    launch_options: &str,
    hidden: bool,
    last_played: i64,
    tags: &[String],
) -> Vec<(String, Bvdf)> {
    let tag_entries: Vec<(String, Bvdf)> = tags
        .iter()
        .enumerate()
        .map(|(i, t)| (i.to_string(), Bvdf::String(t.clone())))
        .collect();

    vec![
        ("appid".into(), Bvdf::Int(app_id as i32)),
        ("AppName".into(), Bvdf::String(name.to_string())),
        ("Exe".into(), Bvdf::String(quoted_path(exe))),
        ("StartDir".into(), Bvdf::String(quoted_start_dir(start_dir))),
        ("icon".into(), Bvdf::String(icon.to_string())),
        ("ShortcutPath".into(), Bvdf::String(String::new())),
        (
            "LaunchOptions".into(),
            Bvdf::String(launch_options.to_string()),
        ),
        ("IsHidden".into(), Bvdf::Int(i32::from(hidden))),
        ("AllowDesktopConfig".into(), Bvdf::Int(1)),
        ("AllowOverlay".into(), Bvdf::Int(1)),
        ("OpenVR".into(), Bvdf::Int(0)),
        ("Devkit".into(), Bvdf::Int(0)),
        ("DevkitGameID".into(), Bvdf::String(String::new())),
        ("LastPlayTime".into(), Bvdf::Int(last_played as i32)),
        ("tags".into(), Bvdf::Object(tag_entries)),
    ]
}

/// Steam stores a shortcut's exe wrapped in literal double quotes.
fn quoted_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("\"{trimmed}\"")
    }
}

/// `StartDir` is stored quoted and terminated with a backslash.
fn quoted_start_dir(path: &str) -> String {
    let trimmed = path.trim().trim_end_matches('\\');
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("\"{trimmed}\\\"")
    }
}

fn get_str_any<'a>(node: &'a Bvdf, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|key| node.get_str(key))
}

/// Strip the surrounding quotes Steam writes around stored paths.
fn clean_path(value: Option<&str>) -> Option<String> {
    let raw = value?.trim();
    if raw.is_empty() {
        return None;
    }
    Some(raw.trim_matches('"').to_string())
}

/// CRC-32 (IEEE 802.3), used to derive non-Steam shortcut appids.
fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for byte in data {
        crc ^= *byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_matches_ieee_reference() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn shortcut_app_id_sets_the_high_bit() {
        let app_id = shortcut_app_id("C:\\Games\\Foo\\foo.exe", "Foo");
        assert_ne!(app_id & 0x8000_0000, 0);
    }

    #[test]
    fn quoted_paths_round_trip() {
        assert_eq!(
            clean_path(Some("\"C:\\Games\\Foo\"")).as_deref(),
            Some("C:\\Games\\Foo")
        );
        assert_eq!(quoted_start_dir("C:\\Games\\Foo\\"), "\"C:\\Games\\Foo\\\"");
        assert_eq!(clean_path(Some("")), None);
    }
}
