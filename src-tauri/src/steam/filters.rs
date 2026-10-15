//! Dynamic (filter-spec) and smart collections.
//!
//! Evaluates Allowed / Required / Excluded filters against the user's library
//! and manages named filter presets as well as cloud-synced dynamic collections
//! in Steam's `cloud-storage-namespace-1.json`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::filter::{self, Filter};
use crate::steam::collections::CollectionsFile;
use crate::steam::models::{Collection, Game};

/// A named filter preset saved locally in the application configuration directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterPreset {
    pub id: String,
    pub name: String,
    pub filter: Filter,
    #[serde(default)]
    pub created_at: i64,
}

/// Path to `filter-presets.json` in the application configuration folder.
pub fn presets_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("steam-vault")
        .join("filter-presets.json")
}

/// Load all saved named filter presets.
pub fn load_presets() -> Vec<FilterPreset> {
    let path = presets_path();
    if !path.is_file() {
        return Vec::new();
    }
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

/// Save a named filter preset.
pub fn save_preset(preset: FilterPreset) -> AppResult<FilterPreset> {
    let mut presets = load_presets();
    if let Some(existing) = presets
        .iter_mut()
        .find(|p| p.id == preset.id || p.name.eq_ignore_ascii_case(&preset.name))
    {
        *existing = preset.clone();
    } else {
        presets.push(preset.clone());
    }
    presets.sort_by_key(|a| a.name.to_lowercase());
    let path = presets_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(&presets)?;
    std::fs::write(&path, json)?;
    Ok(preset)
}

/// Delete a named filter preset by id.
pub fn delete_preset(id: &str) -> AppResult<()> {
    let mut presets = load_presets();
    presets.retain(|p| p.id != id);
    let path = presets_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(&presets)?;
    std::fs::write(&path, json)?;
    Ok(())
}

/// Evaluate a filter against a list of games, returning the matching app IDs.
pub fn evaluate_filter(filter: &Filter, games: &[Game]) -> Vec<u32> {
    games
        .iter()
        .filter(|game| filter::matches(filter, game))
        .map(|game| game.app_id)
        .collect()
}

/// Create or update a dynamic (smart) collection in Steam's collection store.
pub fn save_smart_collection(
    file: &mut CollectionsFile,
    name: &str,
    filter: &Filter,
    games: &[Game],
) -> AppResult<Collection> {
    let matching_app_ids = evaluate_filter(filter, games);

    // Look for existing collection by name or generate new slug id
    let existing = file
        .collections()
        .into_iter()
        .find(|c| c.name.eq_ignore_ascii_case(name));

    let id = existing.map(|c| c.id).unwrap_or_else(|| {
        let nanos = chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default();
        let slug: String = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect();
        let clean_slug = slug.trim_matches('-');
        format!("smart-{clean_slug}-{nanos:x}")
    });

    let spec_json = serde_json::json!({
        "vaultFilter": filter,
        "version": 1
    });

    let collection = Collection {
        id,
        name: name.to_string(),
        favorite: false,
        hidden: false,
        dynamic: true,
        added: matching_app_ids,
        removed: Vec::new(),
        filter_spec: Some(spec_json),
    };

    crate::backup::backup_file(&file.path)?;
    file.upsert(&collection)?;
    file.save()?;
    Ok(collection)
}

/// Re-evaluate an existing dynamic collection from its filter spec against the current library.
pub fn refresh_smart_collection(
    file: &mut CollectionsFile,
    collection_id: &str,
    games: &[Game],
) -> AppResult<Collection> {
    let existing = file
        .collections()
        .into_iter()
        .find(|c| c.id == collection_id)
        .ok_or_else(|| AppError::NotFound(format!("Collection '{collection_id}' not found")))?;

    let spec = existing.filter_spec.as_ref().ok_or_else(|| {
        AppError::Invalid("Collection is not a dynamic / smart collection".into())
    })?;

    // Try extracting vaultFilter
    let filter: Filter = if let Some(vf) = spec.get("vaultFilter") {
        serde_json::from_value(vf.clone()).unwrap_or_default()
    } else {
        // Fallback: parse basic search text if native Steam dynamic collection
        let search = spec
            .get("strSearchText")
            .and_then(Value::as_str)
            .map(str::to_string);
        Filter {
            allowed: Vec::new(),
            required: Vec::new(),
            excluded: Vec::new(),
            search,
            kind: None,
        }
    };

    let matching_app_ids = evaluate_filter(&filter, games);

    let updated = Collection {
        added: matching_app_ids,
        ..existing
    };

    crate::backup::backup_file(&file.path)?;
    file.upsert(&updated)?;
    file.save()?;
    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::steam::models::GameKind;

    fn test_game(app_id: u32, name: &str, collections: Vec<&str>, kind: GameKind) -> Game {
        Game {
            app_id,
            name: name.to_string(),
            kind,
            collections: collections.into_iter().map(String::from).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn evaluates_filter_with_and_or_not_rules() {
        let games = vec![
            test_game(
                10,
                "Counter-Strike",
                vec!["Action", "Multiplayer"],
                GameKind::Steam,
            ),
            test_game(
                20,
                "Team Fortress 2",
                vec!["Action", "Shooter"],
                GameKind::Steam,
            ),
            test_game(30, "Portal", vec!["Puzzle"], GameKind::Steam),
            test_game(
                40,
                "Dolphin (GameCube)",
                vec!["Emulator"],
                GameKind::Shortcut,
            ),
        ];

        let filter = Filter {
            allowed: vec!["Action".into(), "Puzzle".into()],
            required: vec!["Action".into()],
            excluded: vec!["Shooter".into()],
            search: None,
            kind: Some(GameKind::Steam),
        };

        let matches = evaluate_filter(&filter, &games);
        assert_eq!(matches, vec![10]);
    }
}
