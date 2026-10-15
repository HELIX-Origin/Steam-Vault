//! Local game and engine metadata inspector.
//!
//! Extracts developer, publisher, title, engine, and tags from local game
//! installations without network access:
//! - Itch.io `receipt.json` manifests
//! - Ren'Py engine signatures (`renpy/`, `game/*.rpa`, `*.rpyc`)
//! - RPG Maker projects (MV/MZ `System.json`, VX Ace `System.rvdata2`, XP `Scripts.rxdata`, 2000/2003 `RPG_RT.ini`)
//! - Windows PE executable resource strings (`CompanyName`, `ProductName`)

use std::path::Path;

use serde_json::Value;

use crate::steam::models::AppMeta;

/// Sniff metadata from a game's install directory and executable.
pub fn inspect_local_game(dir: Option<&Path>, exe: Option<&Path>) -> Option<AppMeta> {
    let mut meta = AppMeta::default();
    let mut detected = false;

    if let Some(dir) = dir {
        // 1. Itch.io receipt inspection
        if let Some(itch_meta) = inspect_itch_receipt(dir) {
            merge_meta(&mut meta, itch_meta);
            detected = true;
        }

        // 2. Ren'Py engine detection
        if is_renpy_game(dir) {
            detected = true;
            if !meta.genres.contains(&"Visual Novel".to_string()) {
                meta.genres.push("Visual Novel".to_string());
            }
            if !meta.tags.contains(&"Ren'Py".to_string()) {
                meta.tags.push("Ren'Py".to_string());
            }
        }

        // 3. RPG Maker project inspection
        if let Some(rpg_meta) = inspect_rpg_maker(dir) {
            merge_meta(&mut meta, rpg_meta);
            detected = true;
        }
    }

    // 4. Windows PE file header / version info string scanning
    if let Some(exe) = exe {
        if let Some(pe_meta) = inspect_pe_exe(exe) {
            merge_meta(&mut meta, pe_meta);
            detected = true;
        }
    }

    if detected {
        Some(meta)
    } else {
        None
    }
}

use flate2::read::GzDecoder;
use std::io::Read;

/// Inspect itch.io receipt files (`.itch/receipt.json.gz`, `.itch/receipt.json`, or `receipt.json`).
fn inspect_itch_receipt(dir: &Path) -> Option<AppMeta> {
    let gz_path = dir.join(".itch").join("receipt.json.gz");
    let plain_candidates = [
        dir.join(".itch").join("receipt.json"),
        dir.join("receipt.json"),
    ];

    let mut parsed_value: Option<Value> = None;

    if gz_path.is_file() {
        if let Ok(file) = std::fs::File::open(&gz_path) {
            let mut decoder = GzDecoder::new(file);
            let mut text = String::new();
            if decoder.read_to_string(&mut text).is_ok() {
                parsed_value = serde_json::from_str(&text).ok();
            }
        }
    }

    if parsed_value.is_none() {
        for path in &plain_candidates {
            if !path.is_file() {
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(path) {
                if let Ok(v) = serde_json::from_str(&text) {
                    parsed_value = Some(v);
                    break;
                }
            }
        }
    }

    let v = parsed_value?;
    let game = v.get("game")?;

    let title = game
        .get("title")
        .and_then(Value::as_str)
        .map(str::to_string);
    let user = game.get("user");
    let developer = user
        .and_then(|u| u.get("displayName").or_else(|| u.get("username")))
        .and_then(Value::as_str)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let cover_url = game
        .get("coverUrl")
        .and_then(Value::as_str)
        .map(str::to_string);
    let published_at = game
        .get("publishedAt")
        .and_then(Value::as_str)
        .map(str::to_string);
    let release_year = published_at
        .as_deref()
        .and_then(|s| s.split('-').next())
        .and_then(|s| s.parse::<i32>().ok());

    let developers = developer.clone().map(|d| vec![d]).unwrap_or_default();
    let publishers = developers.clone();

    Some(AppMeta {
        name: title,
        developers,
        publishers,
        genres: vec!["Indie".to_string()],
        tags: vec!["itch.io".to_string()],
        release_date: published_at,
        release_year,
        header_image: cover_url.clone(),
        capsule_image: cover_url,
        ..Default::default()
    })
}

/// Check if a folder houses a Ren'Py visual novel.
fn is_renpy_game(dir: &Path) -> bool {
    if dir.join("renpy").is_dir() {
        return true;
    }
    let game_dir = dir.join("game");
    if game_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&game_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
                if name.ends_with(".rpa") || name.ends_with(".rpyc") || name.ends_with(".rpy") {
                    return true;
                }
            }
        }
    }
    false
}

/// Inspect RPG Maker projects across MV, MZ, VX Ace, XP, and 2000/2003.
fn inspect_rpg_maker(dir: &Path) -> Option<AppMeta> {
    // RPG Maker MV / MZ: System.json
    let mv_system = [
        dir.join("data").join("System.json"),
        dir.join("www").join("data").join("System.json"),
    ];

    for path in &mv_system {
        if path.is_file() {
            if let Ok(text) = std::fs::read_to_string(path) {
                if let Ok(v) = serde_json::from_str::<Value>(&text) {
                    let title = v
                        .get("gameTitle")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    return Some(AppMeta {
                        name: title,
                        genres: vec!["RPG".to_string()],
                        tags: vec!["RPG Maker".to_string()],
                        ..Default::default()
                    });
                }
            }
        }
    }

    // RPG Maker 2000/2003: RPG_RT.ini
    let ini_path = dir.join("RPG_RT.ini");
    if ini_path.is_file() {
        if let Ok(text) = std::fs::read_to_string(&ini_path) {
            let mut title = None;
            for line in text.lines() {
                if let Some(val) = line.strip_prefix("GameTitle=") {
                    title = Some(val.trim().to_string());
                    break;
                }
            }
            return Some(AppMeta {
                name: title,
                genres: vec!["RPG".to_string()],
                tags: vec!["RPG Maker".to_string()],
                ..Default::default()
            });
        }
    }

    // RPG Maker VX / Ace / XP signatures
    if dir.join("data").join("System.rvdata2").is_file()
        || dir.join("data").join("Scripts.rxdata").is_file()
    {
        return Some(AppMeta {
            genres: vec!["RPG".to_string()],
            tags: vec!["RPG Maker".to_string()],
            ..Default::default()
        });
    }

    None
}

/// Extract company and product names from Windows PE version info headers.
fn inspect_pe_exe(exe: &Path) -> Option<AppMeta> {
    if !exe.is_file() {
        return None;
    }

    let Ok(bytes) = std::fs::read(exe) else {
        return None;
    };

    // Fast check for MZ header
    if bytes.len() < 1024 || bytes[0] != b'M' || bytes[1] != b'Z' {
        return None;
    }

    // Scan for CompanyName and ProductName in UTF-16LE / ASCII strings
    let company = extract_version_string(&bytes, "CompanyName");
    let product = extract_version_string(&bytes, "ProductName");

    if company.is_none() && product.is_none() {
        return None;
    }

    let developers = company.clone().map(|c| vec![c]).unwrap_or_default();
    let publishers = developers.clone();

    Some(AppMeta {
        name: product,
        developers,
        publishers,
        ..Default::default()
    })
}

/// Scan bytes for a Windows PE Version Resource key and extract its string value.
fn extract_version_string(bytes: &[u8], key: &str) -> Option<String> {
    // Keys in PE StringFileInfo are usually UTF-16LE
    let key_u16: Vec<u8> = key.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();

    // Look for key in bytes
    let key_pos = bytes
        .windows(key_u16.len())
        .position(|w| w == key_u16.as_slice())?;
    let after_key = key_pos + key_u16.len();

    // Skip null terminators and padding up to 8 bytes
    let mut val_start = after_key;
    while val_start < bytes.len() && bytes[val_start] == 0 && (val_start - after_key) < 8 {
        val_start += 1;
    }

    if val_start + 2 > bytes.len() {
        return None;
    }

    // Read UTF-16LE characters until double null
    let mut u16_chars = Vec::new();
    let mut idx = val_start;
    while idx + 1 < bytes.len() && idx - val_start < 256 {
        let code = u16::from_le_bytes([bytes[idx], bytes[idx + 1]]);
        if code == 0 {
            break;
        }
        u16_chars.push(code);
        idx += 2;
    }

    if u16_chars.is_empty() {
        return None;
    }

    let s = String::from_utf16_lossy(&u16_chars);
    let trimmed = s.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("todo")
        || trimmed.eq_ignore_ascii_case("default")
    {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Helper to merge non-empty fields from `source` into `target`.
fn merge_meta(target: &mut AppMeta, source: AppMeta) {
    if target.name.is_none() && source.name.is_some() {
        target.name = source.name;
    }
    for dev in source.developers {
        if !target.developers.contains(&dev) {
            target.developers.push(dev);
        }
    }
    for publ in source.publishers {
        if !target.publishers.contains(&publ) {
            target.publishers.push(publ);
        }
    }
    for genre in source.genres {
        if !target.genres.contains(&genre) {
            target.genres.push(genre);
        }
    }
    for tag in source.tags {
        if !target.tags.contains(&tag) {
            target.tags.push(tag);
        }
    }
    if target.release_date.is_none() && source.release_date.is_some() {
        target.release_date = source.release_date;
    }
    if target.release_year.is_none() && source.release_year.is_some() {
        target.release_year = source.release_year;
    }
    if target.header_image.is_none() && source.header_image.is_some() {
        target.header_image = source.header_image;
    }
    if target.capsule_image.is_none() && source.capsule_image.is_some() {
        target.capsule_image = source.capsule_image;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_utf16_version_string() {
        let key = "CompanyName";
        let key_u16: Vec<u8> = key.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
        let val = "Valve Corporation";
        let val_u16: Vec<u8> = val.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();

        let mut dummy = vec![0u8; 128];
        dummy[10..10 + key_u16.len()].copy_from_slice(&key_u16);
        dummy[10 + key_u16.len() + 2..10 + key_u16.len() + 2 + val_u16.len()]
            .copy_from_slice(&val_u16);

        let extracted = extract_version_string(&dummy, "CompanyName");
        assert_eq!(extracted.as_deref(), Some("Valve Corporation"));
    }
}
