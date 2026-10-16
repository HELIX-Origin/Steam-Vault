//! Embedded SQLite on-disk metadata cache (rusqlite bundled).
//!
//! Replaces the legacy JSON metadata store with an embedded SQLite database
//! stored at `<data_dir>/universal-game-launcher/database.sqlite` (generic database.sqlite).
//! Provides WAL mode, aggressive memory PRAGMAs, indexed lookups by provider and app_id,
//! TTL expiration, and automatic migration from legacy paths.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::steam::models::{AppMeta, Collection};

/// In-memory report generated after a complete cache reset.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheResetReport {
    pub total_metadata_records: usize,
    pub message: String,
}

/// Universal SQLite-backed metadata and app cache with an embedded in-memory fast-path.
pub struct MetadataCache {
    conn: Connection,
    mem_meta: RwLock<HashMap<String, (AppMeta, i64)>>,
    mem_kv: RwLock<HashMap<String, (String, Option<i64>)>>,
    mem_art: RwLock<HashMap<String, String>>,
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

        let mut cache = MetadataCache {
            conn,
            mem_meta: RwLock::new(HashMap::new()),
            mem_kv: RwLock::new(HashMap::new()),
            mem_art: RwLock::new(HashMap::new()),
        };
        cache.init_schema();
        cache.migrate_legacy_json();
        cache.preload_collections();
        cache
    }

    /// Construct a cache with a custom connection (for unit testing).
    pub fn with_connection(conn: Connection) -> Self {
        let cache = MetadataCache {
            conn,
            mem_meta: RwLock::new(HashMap::new()),
            mem_kv: RwLock::new(HashMap::new()),
            mem_art: RwLock::new(HashMap::new()),
        };
        cache.init_schema();
        cache.preload_collections();
        cache
    }

    fn init_schema(&self) {
        let _ = self.conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = -64000;
            PRAGMA temp_store = MEMORY;
            PRAGMA mmap_size = 268435456;

            -- Purge deprecated offline catalogue tables
            DROP TABLE IF EXISTS catalog_lookups;
            DROP TABLE IF EXISTS game_catalog;

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
                hero_image TEXT,
                logo_image TEXT,
                icon_image TEXT,
                user_score REAL,
                raw_json TEXT NOT NULL,
                fetched_at INTEGER NOT NULL,
                PRIMARY KEY (provider, app_id)
            );

            CREATE INDEX IF NOT EXISTS idx_app_meta_provider_appid ON app_meta (provider, app_id);
            CREATE INDEX IF NOT EXISTS idx_app_meta_name ON app_meta (name);

            CREATE TABLE IF NOT EXISTS app_kv (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL,
                expires_at INTEGER
            );

            CREATE INDEX IF NOT EXISTS idx_app_kv_expires ON app_kv (expires_at);

            CREATE TABLE IF NOT EXISTS artwork_cache (
                app_id INTEGER NOT NULL,
                slot TEXT NOT NULL,
                file_path TEXT NOT NULL,
                source_url TEXT,
                updated_at INTEGER NOT NULL,
                PRIMARY KEY (app_id, slot)
            );

            CREATE INDEX IF NOT EXISTS idx_artwork_cache_appid ON artwork_cache (app_id);

            -- Purge deprecated games cache table if present
            DROP TABLE IF EXISTS games;

            CREATE TABLE IF NOT EXISTS collections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                favorite INTEGER NOT NULL DEFAULT 0,
                hidden INTEGER NOT NULL DEFAULT 0,
                dynamic INTEGER NOT NULL DEFAULT 0,
                raw_json TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );
            ",
        );

        // Safe column migrations for existing databases
        let _ = self.conn.execute("ALTER TABLE app_meta ADD COLUMN hero_image TEXT", []);
        let _ = self.conn.execute("ALTER TABLE app_meta ADD COLUMN logo_image TEXT", []);
        let _ = self.conn.execute("ALTER TABLE app_meta ADD COLUMN icon_image TEXT", []);
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
    /// `max_age_secs` is `None`). Prioritizes embedded in-memory Redis-mock cache.
    pub fn get(&self, provider: &str, app_id: u32, max_age_secs: Option<i64>) -> Option<AppMeta> {
        let cache_key = format!("{provider}:{app_id}");
        let now = chrono::Utc::now().timestamp();

        // 1. In-memory fast-path lookup (<0.1ms)
        if let Ok(mem) = self.mem_meta.read() {
            if let Some((meta, fetched_at)) = mem.get(&cache_key) {
                if max_age_secs.map_or(true, |max_age| (now - fetched_at) <= max_age) {
                    return Some(meta.clone());
                }
            }
        }

        // 2. Persistent SQLite lookup
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
            let age = now - fetched_at;
            if age > max_age {
                return None;
            }
        }

        let mut meta: AppMeta = serde_json::from_str(&raw_json).ok()?;
        meta.fetched_at = Some(fetched_at);
        meta.enrich_from_providers();

        // Populate in-memory fast-path
        if let Ok(mut mem) = self.mem_meta.write() {
            mem.insert(cache_key, (meta.clone(), fetched_at));
        }

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
                capsule_image, hero_image, logo_image, icon_image, user_score, raw_json, fetched_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
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
                meta.hero_image,
                meta.logo_image,
                meta.icon_image,
                meta.user_score,
                raw_json,
                fetched_at
            ],
        );

        // Update in-memory fast path
        let cache_key = format!("{provider}:{app_id}");
        if let Ok(mut mem) = self.mem_meta.write() {
            mem.insert(cache_key, (meta.clone(), fetched_at));
        }
    }

    /// Fast key-value retrieval with in-memory caching and TTL expiration.
    pub fn get_kv(&self, key: &str) -> Option<String> {
        let now = chrono::Utc::now().timestamp();
        if let Ok(mem) = self.mem_kv.read() {
            if let Some((val, expires_at)) = mem.get(key) {
                if expires_at.map_or(true, |exp| exp > now) {
                    return Some(val.clone());
                } else {
                    drop(mem);
                    self.delete_kv(key);
                    return None;
                }
            }
        }

        let mut stmt = self
            .conn
            .prepare("SELECT value, expires_at FROM app_kv WHERE key = ?1")
            .ok()?;
        let (val, expires_at): (String, Option<i64>) = stmt
            .query_row(params![key], |row| Ok((row.get(0)?, row.get(1)?)))
            .ok()?;

        if expires_at.map_or(true, |exp| exp > now) {
            if let Ok(mut mem) = self.mem_kv.write() {
                mem.insert(key.to_string(), (val.clone(), expires_at));
            }
            Some(val)
        } else {
            let _ = self.conn.execute("DELETE FROM app_kv WHERE key = ?1", params![key]);
            None
        }
    }

    /// Store a key-value pair with optional TTL in seconds.
    pub fn set_kv(&self, key: &str, value: &str, ttl_secs: Option<i64>) {
        let now = chrono::Utc::now().timestamp();
        let expires_at = ttl_secs.map(|ttl| now + ttl);
        if let Ok(mut mem) = self.mem_kv.write() {
            mem.insert(key.to_string(), (value.to_string(), expires_at));
        }
        let _ = self.conn.execute(
            "INSERT OR REPLACE INTO app_kv (key, value, updated_at, expires_at) VALUES (?1, ?2, ?3, ?4)",
            params![key, value, now, expires_at],
        );
    }

    /// Delete a key-value entry from cache.
    pub fn delete_kv(&self, key: &str) {
        if let Ok(mut mem) = self.mem_kv.write() {
            mem.remove(key);
        }
        let _ = self.conn.execute("DELETE FROM app_kv WHERE key = ?1", params![key]);
    }

    /// Check if a key exists in cache and has not expired.
    pub fn has_kv(&self, key: &str) -> bool {
        self.get_kv(key).is_some()
    }

    /// Query keys matching an optional wildcard/prefix pattern.
    pub fn keys_kv(&self, pattern: Option<&str>) -> Vec<String> {
        let now = chrono::Utc::now().timestamp();
        let sql_pattern = pattern
            .map(|p| p.replace('*', "%"))
            .unwrap_or_else(|| "%".to_string());
        let mut stmt = match self.conn.prepare(
            "SELECT key FROM app_kv WHERE key LIKE ?1 AND (expires_at IS NULL OR expires_at > ?2)",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = stmt.query_map(params![sql_pattern, now], |row| row.get(0));
        rows.map(|iter| iter.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    }

    /// Multi-get: retrieve multiple key-value pairs in a single operation.
    pub fn mget_kv(&self, keys: &[String]) -> HashMap<String, String> {
        let mut results = HashMap::new();
        for k in keys {
            if let Some(val) = self.get_kv(k) {
                results.insert(k.clone(), val);
            }
        }
        results
    }

    /// Multi-set: store multiple key-value pairs in a single transaction.
    pub fn mset_kv(&mut self, pairs: &[(String, String)], ttl_secs: Option<i64>) -> AppResult<()> {
        let now = chrono::Utc::now().timestamp();
        let expires_at = ttl_secs.map(|ttl| now + ttl);
        if let Ok(mut mem) = self.mem_kv.write() {
            for (k, v) in pairs {
                mem.insert(k.clone(), (v.clone(), expires_at));
            }
        }
        let tx = self.conn.transaction()?;
        for (k, v) in pairs {
            let _ = tx.execute(
                "INSERT OR REPLACE INTO app_kv (key, value, updated_at, expires_at) VALUES (?1, ?2, ?3, ?4)",
                params![k, v, now, expires_at],
            );
        }
        tx.commit()?;
        Ok(())
    }

    /// Purge all expired keys from SQLite and in-memory cache.
    pub fn cleanup_expired_kv(&self) -> usize {
        let now = chrono::Utc::now().timestamp();
        if let Ok(mut mem) = self.mem_kv.write() {
            mem.retain(|_, (_, exp)| exp.map_or(true, |e| e > now));
        }
        self.conn
            .execute(
                "DELETE FROM app_kv WHERE expires_at IS NOT NULL AND expires_at <= ?1",
                params![now],
            )
            .unwrap_or(0)
    }

    /// Retrieve and deserialize a JSON value by key.
    pub fn get_kv_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        let s = self.get_kv(key)?;
        serde_json::from_str(&s).ok()
    }

    /// Serialize and store a JSON value by key with optional TTL.
    pub fn set_kv_json<T: Serialize + ?Sized>(&self, key: &str, value: &T, ttl_secs: Option<i64>) -> AppResult<()> {
        let s = serde_json::to_string(value)?;
        self.set_kv(key, &s, ttl_secs);
        Ok(())
    }

    /// Directory where downloaded artwork assets are persisted locally on disk.
    pub fn art_cache_dir() -> PathBuf {
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        let primary = base.join("universal-game-launcher").join("art_cache");
        if primary.exists() {
            return primary;
        }
        let legacy = base.join("steam-vault").join("art_cache");
        if legacy.exists() {
            return legacy;
        }
        primary
    }

    /// Retrieve a cached artwork as a base64 `data:` URL if present on disk or in memory.
    pub fn get_cached_artwork(&self, app_id: u32, slot: &str) -> Option<String> {
        let key = format!("{app_id}:{slot}");
        if let Ok(mem) = self.mem_art.read() {
            if let Some(val) = mem.get(&key) {
                return Some(val.clone());
            }
        }

        // 1. Check SQLite artwork_cache table
        let mut stmt = self
            .conn
            .prepare("SELECT file_path FROM artwork_cache WHERE app_id = ?1 AND slot = ?2")
            .ok()?;
        if let Ok(path_str) = stmt.query_row(params![app_id, slot], |row| row.get::<_, String>(0)) {
            let path = PathBuf::from(&path_str);
            if path.is_file() {
                if let Ok(bytes) = std::fs::read(&path) {
                    let mime = if path_str.ends_with(".png") {
                        "image/png"
                    } else if path_str.ends_with(".webp") {
                        "image/webp"
                    } else if path_str.ends_with(".ico") {
                        "image/x-icon"
                    } else {
                        "image/jpeg"
                    };
                    let data_url = format!("data:{mime};base64,{}", crate::steam::artwork::base64(&bytes));
                    if let Ok(mut mem) = self.mem_art.write() {
                        mem.insert(key, data_url.clone());
                    }
                    return Some(data_url);
                }
            }
        }

        // 2. Direct fallback file scan in art_cache_dir
        let dir = Self::art_cache_dir();
        for ext in &["jpg", "jpeg", "png", "webp"] {
            let candidate = dir.join(format!("{app_id}_{slot}.{ext}"));
            if candidate.is_file() {
                if let Ok(bytes) = std::fs::read(&candidate) {
                    let mime = match *ext {
                        "png" => "image/png",
                        "webp" => "image/webp",
                        _ => "image/jpeg",
                    };
                    let data_url = format!("data:{mime};base64,{}", crate::steam::artwork::base64(&bytes));
                    let cand_str = candidate.to_string_lossy().to_string();
                    let now = chrono::Utc::now().timestamp();
                    let _ = self.conn.execute(
                        "INSERT OR REPLACE INTO artwork_cache (app_id, slot, file_path, updated_at) VALUES (?1, ?2, ?3, ?4)",
                        params![app_id, slot, cand_str, now],
                    );
                    if let Ok(mut mem) = self.mem_art.write() {
                        mem.insert(key, data_url.clone());
                    }
                    return Some(data_url);
                }
            }
        }

        None
    }

    /// Save image bytes to the local disk artwork cache and record in SQLite and memory.
    pub fn cache_artwork_bytes(
        &self,
        app_id: u32,
        slot: &str,
        bytes: &[u8],
        ext: &str,
        source_url: Option<&str>,
    ) -> AppResult<String> {
        let dir = Self::art_cache_dir();
        std::fs::create_dir_all(&dir)?;
        let file_path = dir.join(format!("{app_id}_{slot}.{ext}"));
        std::fs::write(&file_path, bytes)?;

        let mime = match ext {
            "png" => "image/png",
            "webp" => "image/webp",
            _ => "image/jpeg",
        };
        let data_url = format!("data:{mime};base64,{}", crate::steam::artwork::base64(bytes));
        let path_str = file_path.to_string_lossy().to_string();
        let now = chrono::Utc::now().timestamp();

        let _ = self.conn.execute(
            "INSERT OR REPLACE INTO artwork_cache (app_id, slot, file_path, source_url, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![app_id, slot, path_str, source_url, now],
        );

        let key = format!("{app_id}:{slot}");
        if let Ok(mut mem) = self.mem_art.write() {
            mem.insert(key, data_url.clone());
        }

        Ok(data_url)
    }

    /// Purge all cached artwork files and clear database entries.
    pub fn purge_artwork_cache(&self) {
        if let Ok(mut mem) = self.mem_art.write() {
            mem.clear();
        }
        let _ = self.conn.execute("DELETE FROM artwork_cache", []);
        let dir = Self::art_cache_dir();
        if dir.is_dir() {
            let _ = std::fs::remove_dir_all(&dir);
            let _ = std::fs::create_dir_all(&dir);
        }
    }

    /// Total number of cached artwork entries.
    pub fn artwork_count(&self) -> usize {
        self.conn
            .query_row("SELECT COUNT(*) FROM artwork_cache", [], |row| row.get(0))
            .unwrap_or(0)
    }

    /// Clear all caches (metadata, artwork, key-value) and compact SQLite.
    pub fn clear_all_caches(&mut self) -> AppResult<CacheResetReport> {
        // 1. Clear in-memory caches
        if let Ok(mut mem) = self.mem_meta.write() {
            mem.clear();
        }
        if let Ok(mut mem) = self.mem_kv.write() {
            mem.clear();
        }
        if let Ok(mut mem) = self.mem_art.write() {
            mem.clear();
        }

        // 2. Clear disk artwork cache
        self.purge_artwork_cache();

        // 3. Purge SQLite tables
        let _ = self.conn.execute_batch(
            "
            DELETE FROM app_meta;
            DELETE FROM artwork_cache;
            DELETE FROM app_kv;
            ",
        )?;

        // 4. Compact SQLite database
        let _ = self.conn.execute("VACUUM", []);

        Ok(CacheResetReport {
            total_metadata_records: 0,
            message: "Successfully cleared all metadata, artwork, and key-value caches.".to_string(),
        })
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

    /// Preload collections from SQLite directly into the in-memory KV tier with permanent TTL.
    pub fn preload_collections(&self) {
        let collections = self.load_collections_from_db();
        if !collections.is_empty() {
            let _ = self.set_kv_json("collections:default", &collections, None);
        }
    }



    /// Load all persisted collections from SQLite.
    pub fn load_collections_from_db(&self) -> Vec<Collection> {
        let mut stmt = match self.conn.prepare("SELECT raw_json FROM collections ORDER BY name ASC") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = match stmt.query_map([], |row| {
            let raw: String = row.get(0)?;
            Ok(raw)
        }) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        let mut list = Vec::new();
        for raw in rows.flatten() {
            if let Ok(c) = serde_json::from_str::<Collection>(&raw) {
                list.push(c);
            }
        }
        list
    }

    /// Bulk save collections into SQLite and synchronize in-memory KV tier.
    pub fn save_collections_to_db(&self, collections: &[Collection]) -> AppResult<()> {
        let now = chrono::Utc::now().timestamp();
        let mut stmt = self.conn.prepare(
            "INSERT INTO collections (id, name, favorite, hidden, dynamic, raw_json, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                favorite = excluded.favorite,
                hidden = excluded.hidden,
                dynamic = excluded.dynamic,
                raw_json = excluded.raw_json,
                updated_at = excluded.updated_at",
        )?;

        for col in collections {
            let raw = serde_json::to_string(col).unwrap_or_default();
            let _ = stmt.execute(params![
                col.id,
                col.name,
                col.favorite as i32,
                col.hidden as i32,
                col.dynamic as i32,
                raw,
                now
            ]);
        }
        let _ = self.set_kv_json("collections:default", collections, None);
        Ok(())
    }

    /// Upsert a single collection into SQLite and update in-memory KV tier.
    pub fn upsert_collection_in_db(&self, col: &Collection) -> AppResult<()> {
        let now = chrono::Utc::now().timestamp();
        let raw = serde_json::to_string(col).unwrap_or_default();
        self.conn.execute(
            "INSERT INTO collections (id, name, favorite, hidden, dynamic, raw_json, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                favorite = excluded.favorite,
                hidden = excluded.hidden,
                dynamic = excluded.dynamic,
                raw_json = excluded.raw_json,
                updated_at = excluded.updated_at",
            params![
                col.id,
                col.name,
                col.favorite as i32,
                col.hidden as i32,
                col.dynamic as i32,
                raw,
                now
            ],
        )?;

        let mut cols = self.load_collections_from_db();
        if let Some(pos) = cols.iter().position(|c| c.id == col.id) {
            cols[pos] = col.clone();
        } else {
            cols.push(col.clone());
        }
        let _ = self.set_kv_json("collections:default", &cols, None);
        Ok(())
    }

    /// Delete a collection from SQLite and update in-memory KV tier.
    pub fn delete_collection_from_db(&self, id: &str) -> AppResult<()> {
        self.conn.execute("DELETE FROM collections WHERE id = ?1", params![id])?;
        let cols = self.load_collections_from_db();
        let _ = self.set_kv_json("collections:default", &cols, None);
        Ok(())
    }
}

fn db_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let new_dir = base.join("universal-game-launcher");
    let legacy_dir = base.join("steam-vault");

    let primary_sqlite = new_dir.join("database.sqlite");
    if primary_sqlite.exists() {
        return primary_sqlite;
    }
    let primary_db = new_dir.join("database.db");
    if primary_db.exists() {
        return primary_db;
    }

    // Fallbacks in legacy directories
    let legacy_sqlite = legacy_dir.join("database.sqlite");
    if legacy_sqlite.exists() {
        return legacy_sqlite;
    }
    let legacy_db = legacy_dir.join("database.db");
    if legacy_db.exists() {
        return legacy_db;
    }
    let legacy_meta = legacy_dir.join("metadata.db");
    if legacy_meta.exists() {
        // Transparently migrate existing metadata.db to new database.sqlite if feasible
        let _ = std::fs::create_dir_all(&new_dir);
        if std::fs::copy(&legacy_meta, &primary_sqlite).is_ok() {
            return primary_sqlite;
        }
        return legacy_meta;
    }

    // Default generic path
    primary_sqlite
}

fn legacy_json_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let new_path = base.join("universal-game-launcher").join("metadata-cache.json");
    if new_path.exists() {
        return new_path;
    }
    base.join("steam-vault").join("metadata-cache.json")
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
            ..Default::default()
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

    #[test]
    fn test_serverless_redis_kv_store_and_ttl() {
        let conn = Connection::open_in_memory().unwrap();
        let mut cache = MetadataCache::with_connection(conn);

        // Basic GET / SET / EXISTS / DEL
        assert_eq!(cache.get_kv("session:user"), None);
        assert!(!cache.has_kv("session:user"));

        cache.set_kv("session:user", "alice", None);
        assert_eq!(cache.get_kv("session:user").as_deref(), Some("alice"));
        assert!(cache.has_kv("session:user"));

        // JSON serialization helper
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Preferences {
            theme: String,
            volume: i32,
        }
        let prefs = Preferences {
            theme: "dark".into(),
            volume: 85,
        };
        cache.set_kv_json("user:prefs", &prefs, None).unwrap();
        let loaded: Option<Preferences> = cache.get_kv_json("user:prefs");
        assert_eq!(loaded, Some(prefs));

        // MSET and MGET
        let pairs = vec![
            ("config:a".to_string(), "1".to_string()),
            ("config:b".to_string(), "2".to_string()),
            ("config:c".to_string(), "3".to_string()),
        ];
        cache.mset_kv(&pairs, None).unwrap();

        let keys = vec!["config:a".to_string(), "config:b".to_string(), "nonexistent".to_string()];
        let mget_res = cache.mget_kv(&keys);
        assert_eq!(mget_res.get("config:a").map(|s| s.as_str()), Some("1"));
        assert_eq!(mget_res.get("config:b").map(|s| s.as_str()), Some("2"));
        assert_eq!(mget_res.get("nonexistent"), None);

        // KEYS with wildcard pattern
        let matching = cache.keys_kv(Some("config:*"));
        assert_eq!(matching.len(), 3);

        // Delete
        cache.delete_kv("config:a");
        assert_eq!(cache.get_kv("config:a"), None);

        // TTL expiration
        cache.set_kv("temp:token", "xyz", Some(-10)); // expired 10s ago
        assert_eq!(cache.get_kv("temp:token"), None); // triggers auto-eviction on read
        assert!(!cache.has_kv("temp:token"));

        cache.set_kv("temp:stale", "abc", Some(-5));
        let cleaned = cache.cleanup_expired_kv();
        assert!(cleaned >= 1);
    }

    #[test]
    fn test_database_purge_and_cache_reset() {
        let conn = Connection::open_in_memory().unwrap();
        let mut cache = MetadataCache::with_connection(conn);

        // Verify catalog tables do NOT exist
        let check_catalog: Result<i64, _> = cache.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='game_catalog'",
            [],
            |r| r.get(0),
        );
        assert_eq!(check_catalog.unwrap(), 0);

        let check_lookups: Result<i64, _> = cache.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='catalog_lookups'",
            [],
            |r| r.get(0),
        );
        assert_eq!(check_lookups.unwrap(), 0);

        // Put metadata and kv, then clear all
        let meta = AppMeta {
            app_id: 200,
            name: Some("Half-Life 2".into()),
            ..Default::default()
        };
        cache.put("steam-store", 200, &meta);
        cache.set_kv("test_key", "test_val", None);
        assert_eq!(cache.count(), 1);
        assert!(cache.has_kv("test_key"));

        let report = cache.clear_all_caches().expect("clear all caches should succeed");
        assert_eq!(report.total_metadata_records, 0);
        assert_eq!(cache.count(), 0);
        assert!(!cache.has_kv("test_key"));
    }
}

