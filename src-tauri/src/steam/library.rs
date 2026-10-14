//! Library aggregation: combine `appmanifest_*.acf`, `libraryfolders.vdf`,
//! non-Steam `shortcuts.vdf`, and per-account play stats into [`Game`] records.

use std::collections::BTreeMap;
use std::path::Path;

use crate::error::AppResult;
use crate::steam::models::{Game, GameKind};
use crate::steam::{paths, vdf};

/// `StateFlags` bit indicating a fully installed app.
const STATE_FLAG_FULLY_INSTALLED: u32 = 4;

/// Read installed Steam games from every registered library folder.
pub fn load_games(root: &Path) -> AppResult<Vec<Game>> {
    let libraries = paths::read_library_folders(root);
    let mut games: BTreeMap<u32, Game> = BTreeMap::new();

    for library in &libraries {
        let steamapps = Path::new(&library.path).join("steamapps");
        let Ok(entries) = std::fs::read_dir(&steamapps) else {
            continue;
        };
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let Some(app_id) = parse_appmanifest_id(&file_name) else {
                continue;
            };
            let Ok(text) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            let Ok(doc) = vdf::parse(&text) else {
                continue;
            };
            let Some(state) = doc.get("AppState") else {
                continue;
            };

            let item = games.entry(app_id).or_insert_with(|| Game {
                app_id,
                ..Default::default()
            });
            item.name = state.get_str("name").unwrap_or_default().to_string();
            item.kind = GameKind::Steam;
            item.library_path = Some(library.path.clone());
            item.install_dir = state.get_str("installdir").map(str::to_string);
            item.size_on_disk = state.get_u64("SizeOnDisk").unwrap_or(0);
            item.installed =
                state.get_u32("StateFlags").unwrap_or(0) & STATE_FLAG_FULLY_INSTALLED != 0;
        }
    }

    // The libraryfolders `apps` map lists installed appids even when the
    // appmanifest has been removed or is mid-update.
    for library in &libraries {
        for app_id in &library.app_ids {
            let item = games.entry(*app_id).or_insert_with(|| Game {
                app_id: *app_id,
                ..Default::default()
            });
            if item.library_path.is_none() {
                item.library_path = Some(library.path.clone());
            }
            item.installed = true;
        }
    }

    Ok(games.into_values().collect())
}

/// Read per-app play stats (`LastPlayed` seconds, `Playtime` minutes) from a
/// user's `localconfig.vdf`.
pub fn load_app_stats(root: &Path, id3: &str) -> BTreeMap<u32, (Option<i64>, Option<u64>)> {
    let mut out = BTreeMap::new();
    let path = root
        .join("userdata")
        .join(id3)
        .join("config")
        .join("localconfig.vdf");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return out;
    };
    let Ok(doc) = vdf::parse(&text) else {
        return out;
    };
    let Some(apps) = doc.get_path("UserLocalConfigStore/Software/Valve/Steam/apps") else {
        return out;
    };
    for (key, node) in apps.iter() {
        if let Ok(app_id) = key.parse::<u32>() {
            out.insert(
                app_id,
                (node.get_i64("LastPlayed"), node.get_u64("Playtime")),
            );
        }
    }
    out
}

fn parse_appmanifest_id(file_name: &str) -> Option<u32> {
    let rest = file_name.strip_prefix("appmanifest_")?;
    let id = rest.strip_suffix(".acf")?;
    id.parse().ok()
}
