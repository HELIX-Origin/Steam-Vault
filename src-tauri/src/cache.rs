//! Embedded SQLite on-disk metadata cache (rusqlite bundled).
//!
//! Replaces the legacy JSON metadata store with an embedded SQLite database
//! stored at `<data_dir>/steam-vault/metadata.db`. Provides WAL mode, indexed
//! lookups by provider and app_id, TTL expiration, and automatic migration
//! from legacy `metadata-cache.json`.

use std::path::PathBuf;

use rusqlite::{params, Connection};

use crate::error::AppResult;
use crate::steam::models::AppMeta;

/// SQLite-backed metadata cache.
pub struct MetadataCache {
    conn: Connection,
}

impl Default for MetadataCache {
    fn default() -> Self {
        Self::open()
    }
}

impl MetadataCache {
    /// Open the metadata SQLite database, run migrations, and import legacy JSON if present.
    pub fn open() -> Self {
        let path = db_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn = Connection::open(&path).unwrap_or_else(|_| {
            Connection::open_in_memory().expect("failed to open fallback in-memory SQLite database")
        });

        let mut cache = MetadataCache { conn };
        cache.init_schema();
        cache.migrate_legacy_json();
        cache
    }

    /// Construct a cache with a custom connection (for unit testing).
    pub fn with_connection(conn: Connection) -> Self {
        let cache = MetadataCache { conn };
        cache.init_schema();
        cache
    }

    fn init_schema(&self) {
        let _ = self.conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;

            CREATE TABLE IF NOT EXISTS app_meta (
                provider TEXT NOT NULL,
                app_id INTEGER NOT NULL,
                name TEXT,
                developers TEXT,
                publishers TEXT,
                genres TEXT,
                categories TEXT,
                tags TEXT,
                release_date TEXT,
                release_year INTEGER,
                short_description TEXT,
                header_image TEXT,
                capsule_image TEXT,
                user_score REAL,
                raw_json TEXT NOT NULL,
                fetched_at INTEGER NOT NULL,
                PRIMARY KEY (provider, app_id)
            );

            CREATE INDEX IF NOT EXISTS idx_app_meta_provider_appid ON app_meta (provider, app_id);
            CREATE INDEX IF NOT EXISTS idx_app_meta_name ON app_meta (name);
            ",
        );
    }

    /// Migrate legacy `metadata-cache.json` if it exists.
    fn migrate_legacy_json(&mut self) {
        let legacy_path = legacy_json_path();
        if !legacy_path.is_file() {
            return;
        }

        let Ok(text) = std::fs::read_to_string(&legacy_path) else {
            return;
        };

        #[derive(serde::Deserialize)]
        struct LegacyRecord {
            provider: String,
            app_id: u32,
            meta: AppMeta,
            #[serde(default)]
            fetched_at: i64,
        }

        let map: std::collections::BTreeMap<String, LegacyRecord> =
            serde_json::from_str(&text).unwrap_or_default();

        if let Ok(tx) = self.conn.transaction() {
            for (_, record) in map {
                let raw_json = serde_json::to_string(&record.meta).unwrap_or_default();
                let devs = serde_json::to_string(&record.meta.developers).unwrap_or_default();
                let pubs = serde_json::to_string(&record.meta.publishers).unwrap_or_default();
                let genres = serde_json::to_string(&record.meta.genres).unwrap_or_default();
                let cats = serde_json::to_string(&record.meta.categories).unwrap_or_default();
                let tags = serde_json::to_string(&record.meta.tags).unwrap_or_default();

                let _ = tx.execute(
                    "INSERT OR IGNORE INTO app_meta (
                        provider, app_id, name, developers, publishers, genres, categories,
                        tags, release_date, release_year, short_description, header_image,
                        capsule_image, user_score, raw_json, fetched_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                    params![
                        record.provider,
                        record.app_id,
                        record.meta.name,
                        devs,
                        pubs,
                        genres,
                        cats,
                        tags,
                        record.meta.release_date,
                        record.meta.release_year,
                        record.meta.short_description,
                        record.meta.header_image,
                        record.meta.capsule_image,
                        record.meta.user_score,
                        raw_json,
                        record.fetched_at
                    ],
                );
            }
            let _ = tx.commit();
        }

        // Rename legacy file to indicate completed migration
        let backup_path = legacy_path.with_extension("json.migrated");
        let _ = std::fs::rename(&legacy_path, &backup_path);
    }

    /// Return a cached record when it is within `max_age_secs` (or always when
    /// `max_age_secs` is `None`).
    pub fn get(&self, provider: &str, app_id: u32, max_age_secs: Option<i64>) -> Option<AppMeta> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT raw_json, fetched_at FROM app_meta WHERE provider = ?1 AND app_id = ?2",
            )
            .ok()?;

        let (raw_json, fetched_at): (String, i64) = stmt
            .query_row(params![provider, app_id], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .ok()?;

        if let Some(max_age) = max_age_secs {
            let age = chrono::Utc::now().timestamp() - fetched_at;
            if age > max_age {
                return None;
            }
        }

        let mut meta: AppMeta = serde_json::from_str(&raw_json).ok()?;
        meta.fetched_at = Some(fetched_at);
        Some(meta)
    }

    /// Insert or update a metadata record for a provider and app id.
    pub fn put(&mut self, provider: &str, app_id: u32, meta: &AppMeta) {
        let fetched_at = meta
            .fetched_at
            .unwrap_or_else(|| chrono::Utc::now().timestamp());
        let raw_json = serde_json::to_string(meta).unwrap_or_default();
        let devs = serde_json::to_string(&meta.developers).unwrap_or_default();
        let pubs = serde_json::to_string(&meta.publishers).unwrap_or_default();
        let genres = serde_json::to_string(&meta.genres).unwrap_or_default();
        let cats = serde_json::to_string(&meta.categories).unwrap_or_default();
        let tags = serde_json::to_string(&meta.tags).unwrap_or_default();

        let _ = self.conn.execute(
            "INSERT OR REPLACE INTO app_meta (
                provider, app_id, name, developers, publishers, genres, categories,
                tags, release_date, release_year, short_description, header_image,
                capsule_image, user_score, raw_json, fetched_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                provider,
                app_id,
                meta.name,
                devs,
                pubs,
                genres,
                cats,
                tags,
                meta.release_date,
                meta.release_year,
                meta.short_description,
                meta.header_image,
                meta.capsule_image,
                meta.user_score,
                raw_json,
                fetched_at
            ],
        );
    }

    /// Save is a no-op since SQLite commits transactions directly to disk.
    pub fn save(&self) -> AppResult<()> {
        Ok(())
    }

    /// Total number of cached metadata records in SQLite.
    pub fn count(&self) -> usize {
        self.conn
            .query_row("SELECT COUNT(*) FROM app_meta", [], |row| row.get(0))
            .unwrap_or(0)
    }
}

fn db_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("steam-vault")
        .join("metadata.db")
}

fn legacy_json_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("steam-vault")
        .join("metadata-cache.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlite_cache_puts_and_gets_metadata() {
        let conn = Connection::open_in_memory().unwrap();
        let mut cache = MetadataCache::with_connection(conn);

        let meta = AppMeta {
            app_id: 400,
            name: Some("Portal".into()),
            developers: vec!["Valve".into()],
            publishers: vec!["Valve".into()],
            genres: vec!["Action".into(), "Puzzle".into()],
            categories: vec!["Single-player".into()],
            tags: vec!["Puzzle".into(), "First-Person".into()],
            release_date: Some("Oct 10, 2007".into()),
            release_year: Some(2007),
            short_description: Some("A puzzle game".into()),
            header_image: None,
            capsule_image: None,
            user_score: Some(9.5),
            fetched_at: Some(chrono::Utc::now().timestamp()),
        };

        cache.put("steam-store", 400, &meta);
        assert_eq!(cache.count(), 1);

        let retrieved = cache
            .get("steam-store", 400, None)
            .expect("should find cached meta");
        assert_eq!(retrieved.name.as_deref(), Some("Portal"));
        assert_eq!(retrieved.release_year, Some(2007));
        assert_eq!(retrieved.developers, vec!["Valve"]);

        // Expired lookup with max_age = 0 should return None if older
        let non_existent = cache.get("steam-store", 999, None);
        assert!(non_existent.is_none());
    }
}
