//! Lightweight on-disk metadata cache.
//!
//! Metadata requests are rate-limited and expensive, so normalized [`AppMeta`]
//! records are cached as JSON keyed by `provider:appid`. SQLite is a planned
//! upgrade; the interface below is deliberately storage-agnostic.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::steam::models::AppMeta;

/// One cached metadata record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedMeta {
    pub provider: String,
    pub app_id: u32,
    pub meta: AppMeta,
    pub fetched_at: i64,
}

/// JSON-backed metadata cache.
#[derive(Debug, Default)]
pub struct MetadataCache {
    path: PathBuf,
    entries: BTreeMap<String, CachedMeta>,
}

impl MetadataCache {
    pub fn open() -> Self {
        let path = cache_path();
        let entries = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        MetadataCache { path, entries }
    }

    fn key(provider: &str, app_id: u32) -> String {
        format!("{provider}:{app_id}")
    }

    /// Return a cached record when it is within `max_age_secs` (or always when
    /// `max_age_secs` is `None`).
    pub fn get(&self, provider: &str, app_id: u32, max_age_secs: Option<i64>) -> Option<&AppMeta> {
        let entry = self.entries.get(&Self::key(provider, app_id))?;
        if let Some(max_age) = max_age_secs {
            let age = chrono::Utc::now().timestamp() - entry.fetched_at;
            if age > max_age {
                return None;
            }
        }
        Some(&entry.meta)
    }

    pub fn put(&mut self, provider: &str, app_id: u32, meta: &AppMeta) {
        self.entries.insert(
            Self::key(provider, app_id),
            CachedMeta {
                provider: provider.to_string(),
                app_id,
                meta: meta.clone(),
                fetched_at: chrono::Utc::now().timestamp(),
            },
        );
    }

    pub fn save(&self) -> AppResult<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string(&self.entries)?;
        std::fs::write(&self.path, text)?;
        Ok(())
    }
}

fn cache_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("steam-vault")
        .join("metadata-cache.json")
}
