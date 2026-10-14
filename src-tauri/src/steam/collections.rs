//! Steam collection (category) reads and writes.
//!
//! Modern Steam (2022+) stores collections in the cloud-synced
//! `config/cloudstorage/cloud-storage-namespace-1.json` file. The file is a JSON
//! array of `[key, entry]` pairs where each entry holds a `value` that is an
//! **escaped JSON string**. Legacy categories live in `localconfig.vdf` under
//! `.../apps/<appid>/tags` and are read for import/back-compat only.
//!
//! Writes must occur while Steam is closed; callers check
//! [`crate::steam::process::is_steam_running`] first and back up the file via
//! [`crate::backup`].

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::error::AppResult;
use crate::steam::models::Collection;
use crate::steam::vdf;

const COLLECTION_PREFIX: &str = "user-collections.";

/// Path to the modern cloud collection namespace file.
pub fn cloud_storage_path(root: &Path, id3: &str) -> PathBuf {
    root.join("userdata")
        .join(id3)
        .join("config")
        .join("cloudstorage")
        .join("cloud-storage-namespace-1.json")
}

/// An in-memory view of the cloud namespace file, preserving entry order.
#[derive(Debug)]
pub struct CollectionsFile {
    pub path: PathBuf,
    pub entries: Vec<Value>,
}

impl CollectionsFile {
    /// Parse every non-deleted collection entry.
    pub fn collections(&self) -> Vec<Collection> {
        self.entries.iter().filter_map(parse_entry).collect()
    }

    /// Insert or replace a collection, returning `true` if an existing entry
    /// was modified.
    pub fn upsert(&mut self, collection: &Collection) -> AppResult<bool> {
        let key = format!("{COLLECTION_PREFIX}{}", collection.id);
        let inner = serde_json::json!({
            "id": collection.id,
            "name": collection.name,
            "added": collection.added,
            "removed": collection.removed,
        });
        let value = serde_json::to_string(&inner)?;
        let timestamp = chrono::Utc::now().timestamp();
        let deleted_key = format!("{COLLECTION_PREFIX}{}", collection.id);

        for entry in self.entries.iter_mut() {
            let Some(arr) = entry.as_array_mut() else {
                continue;
            };
            if arr.len() != 2 || arr[0].as_str() != Some(key.as_str()) {
                continue;
            }
            let Some(obj) = arr[1].as_object_mut() else {
                continue;
            };
            obj.insert("is_deleted".into(), Value::Bool(false));
            obj.insert("value".into(), Value::String(value));
            obj.insert("timestamp".into(), Value::from(timestamp));
            return Ok(true);
        }

        self.entries.push(serde_json::json!([
            deleted_key,
            {
                "key": key,
                "timestamp": timestamp,
                "is_deleted": false,
                "value": value,
                "version": "1",
            }
        ]));
        Ok(false)
    }

    /// Mark a collection deleted by writing a tombstone (Steam keeps history).
    pub fn delete(&mut self, id: &str) -> bool {
        let target = format!("{COLLECTION_PREFIX}{id}");
        for entry in self.entries.iter_mut() {
            let Some(arr) = entry.as_array_mut() else {
                continue;
            };
            if arr.len() != 2 || arr[0].as_str() != Some(target.as_str()) {
                continue;
            }
            if let Some(obj) = arr[1].as_object_mut() {
                obj.insert("is_deleted".into(), Value::Bool(true));
                return true;
            }
        }
        false
    }

    /// Serialize the namespace back to disk.
    pub fn save(&self) -> AppResult<()> {
        let text = serde_json::to_string(&self.entries)?;
        std::fs::write(&self.path, text)?;
        Ok(())
    }
}

/// Load the cloud collection file if it exists for this account.
pub fn load_collections_file(root: &Path, id3: &str) -> AppResult<Option<CollectionsFile>> {
    let path = cloud_storage_path(root, id3);
    if !path.is_file() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path)?;
    let entries: Vec<Value> = serde_json::from_str(&text).unwrap_or_default();
    Ok(Some(CollectionsFile { path, entries }))
}

fn parse_entry(entry: &Value) -> Option<Collection> {
    let arr = entry.as_array()?;
    if arr.len() != 2 {
        return None;
    }
    let key = arr[0].as_str()?;
    let obj = arr[1].as_object()?;
    if obj
        .get("is_deleted")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return None;
    }
    let id = key.strip_prefix(COLLECTION_PREFIX)?;
    let value_str = obj.get("value").and_then(Value::as_str)?;
    let inner: Value = serde_json::from_str(value_str).ok()?;

    Some(Collection {
        id: id.to_string(),
        name: inner
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(id)
            .to_string(),
        favorite: id == "favorite",
        hidden: id == "hidden",
        dynamic: inner
            .get("filterSpec")
            .map(|v| !v.is_null())
            .unwrap_or(false),
        added: json_u32_array(inner.get("added")),
        removed: json_u32_array(inner.get("removed")),
    })
}

fn json_u32_array(value: Option<&Value>) -> Vec<u32> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_u64().map(|n| n as u32))
                .collect()
        })
        .unwrap_or_default()
}

/// Read legacy `.../apps/<appid>/tags` categories from `localconfig.vdf`.
pub fn load_legacy_tags(root: &Path, id3: &str) -> BTreeMap<u32, Vec<String>> {
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
        let Ok(app_id) = key.parse::<u32>() else {
            continue;
        };
        if let Some(tags) = node.get("tags") {
            let names: Vec<String> = tags
                .iter()
                .filter_map(|(_, value)| value.as_str().map(str::to_string))
                .collect();
            if !names.is_empty() {
                out.insert(app_id, names);
            }
        }
    }
    out
}
