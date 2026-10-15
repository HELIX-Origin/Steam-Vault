//! Embedded SQLite on-disk metadata cache (rusqlite bundled).
//!
//! Replaces the legacy JSON metadata store with an embedded SQLite database
//! stored at `<data_dir>/steam-vault/metadata.db`. Provides WAL mode, indexed
//! lookups by provider and app_id, TTL expiration, and automatic migration
//! from legacy `metadata-cache.json`.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::steam::models::AppMeta;

/// Represents either a raw JSON string or an array of strings in catalog export records.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrSeq {
    String(String),
    Seq(Vec<String>),
}

impl StringOrSeq {
    pub fn into_json_string(self) -> String {
        match self {
            StringOrSeq::String(s) => s,
            StringOrSeq::Seq(vec) => {
                serde_json::to_string(&vec).unwrap_or_else(|_| "[]".to_string())
            }
        }
    }
}

/// A compressed serializable catalog export record supporting both stringified and structured formats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogExportRecord {
    pub title: String,
    pub normalized_title: String,
    #[serde(default)]
    pub platforms: Option<StringOrSeq>,
    #[serde(default)]
    pub known_exes: Option<StringOrSeq>,
    #[serde(default)]
    pub meta_json: Option<serde_json::Value>,
    #[serde(default)]
    pub meta: Option<serde_json::Value>,
}

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
        cache.seed_initial_catalog();
        cache
    }

    /// Construct a cache with a custom connection (for unit testing).
    pub fn with_connection(conn: Connection) -> Self {
        let cache = MetadataCache { conn };
        cache.init_schema();
        cache
    }

    /// Seed baseline community catalog if the database is newly initialized.
    pub fn seed_initial_catalog(&mut self) {
        if self.catalog_count() == 0 {
            const SEED_BYTES: &[u8] = include_bytes!("catalog_seed.bin");
            if !SEED_BYTES.is_empty() {
                let _ = self.import_catalog_compressed(SEED_BYTES);
            }
        }
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

            CREATE TABLE IF NOT EXISTS game_catalog (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                normalized_title TEXT NOT NULL UNIQUE,
                platforms TEXT NOT NULL,
                known_exes TEXT NOT NULL,
                meta_json TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_game_catalog_norm_title ON game_catalog (normalized_title);

            CREATE TABLE IF NOT EXISTS catalog_lookups (
                lookup_key TEXT NOT NULL,
                key_type TEXT NOT NULL,
                catalog_id INTEGER NOT NULL REFERENCES game_catalog(id) ON DELETE CASCADE,
                PRIMARY KEY (lookup_key, key_type, catalog_id)
            );

            CREATE INDEX IF NOT EXISTS idx_catalog_lookups_key ON catalog_lookups (lookup_key, key_type);
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

    /// Total number of games registered in the local catalog.
    pub fn catalog_count(&self) -> usize {
        self.conn
            .query_row("SELECT COUNT(*) FROM game_catalog", [], |row| row.get(0))
            .unwrap_or(0)
    }

    /// Look up a canonical game metadata record by its executable file name.
    pub fn find_catalog_by_exe(&self, exe: &str) -> Option<AppMeta> {
        let exe_name = Path::new(exe)
            .file_name()
            .and_then(|s| s.to_str())?
            .to_lowercase();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT g.meta_json FROM game_catalog g
                 JOIN catalog_lookups l ON g.id = l.catalog_id
                 WHERE l.key_type = 'exe' AND l.lookup_key = ?1
                 LIMIT 1",
            )
            .ok()?;
        let json: String = stmt.query_row(params![exe_name], |row| row.get(0)).ok()?;
        serde_json::from_str(&json).ok()
    }

    /// Look up a canonical game metadata record by title or alias.
    pub fn find_catalog_by_title(&self, title: &str) -> Option<AppMeta> {
        let norm = normalize_catalog_key(title);
        let raw_norm: String = title
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();
        if norm.is_empty() && raw_norm.is_empty() {
            return None;
        }
        let mut stmt = self
            .conn
            .prepare(
                "SELECT meta_json FROM game_catalog
                 WHERE normalized_title = ?1 OR normalized_title = ?2
                 UNION
                 SELECT g.meta_json FROM game_catalog g
                 JOIN catalog_lookups l ON g.id = l.catalog_id
                 WHERE l.key_type = 'alias' AND (l.lookup_key = ?1 OR l.lookup_key = ?2)
                 LIMIT 1",
            )
            .ok()?;
        let json: String = stmt
            .query_row(params![norm, raw_norm], |row| row.get(0))
            .ok()?;
        serde_json::from_str(&json).ok()
    }

    /// Search the local game catalog by keyword query.
    pub fn search_catalog(&self, query: &str, limit: usize) -> Vec<AppMeta> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }
        let norm = normalize_catalog_key(trimmed);
        let pattern = format!("%{}%", trimmed.to_lowercase());
        let norm_pattern = format!("%{norm}%");

        let mut stmt = match self.conn.prepare(
            "SELECT meta_json FROM (
                SELECT meta_json, 1 as rank FROM game_catalog
                WHERE normalized_title LIKE ?1 OR LOWER(title) LIKE ?2
                UNION
                SELECT g.meta_json, 2 as rank FROM game_catalog g
                JOIN catalog_lookups l ON g.id = l.catalog_id
                WHERE l.lookup_key LIKE ?1 OR LOWER(l.lookup_key) LIKE ?2
             ) ORDER BY rank LIMIT ?3",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map(params![norm_pattern, pattern, limit as i64], |row| {
            row.get::<_, String>(0)
        });
        let Ok(rows) = rows else {
            return Vec::new();
        };

        rows.filter_map(|r| r.ok())
            .filter_map(|json| serde_json::from_str(&json).ok())
            .collect()
    }

    /// Insert or update a game entry in the local catalog and index its lookups.
    pub fn upsert_catalog_entry(
        &mut self,
        title: &str,
        meta: &AppMeta,
        platform: Option<&str>,
        exe: Option<&str>,
    ) {
        let norm = normalize_catalog_key(title);
        let raw_norm: String = title
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();
        if norm.is_empty() && raw_norm.is_empty() {
            return;
        }
        let now = chrono::Utc::now().timestamp();
        let meta_json = serde_json::to_string(meta).unwrap_or_default();

        let tx = match self.conn.transaction() {
            Ok(t) => t,
            Err(_) => return,
        };

        // 1. Check existing record
        let existing: Option<(i64, String, String)> = tx
            .query_row(
                "SELECT id, platforms, known_exes FROM game_catalog WHERE normalized_title = ?1",
                params![norm],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .ok();

        let catalog_id: i64 = match existing {
            Some((id, plat_str, exe_str)) => {
                let mut platforms: Vec<String> =
                    serde_json::from_str(&plat_str).unwrap_or_default();
                if let Some(p) = platform {
                    if !platforms.iter().any(|existing| existing == p) {
                        platforms.push(p.to_string());
                    }
                }
                let mut exes: Vec<String> = serde_json::from_str(&exe_str).unwrap_or_default();
                if let Some(e) = exe {
                    let exe_file = Path::new(e)
                        .file_name()
                        .and_then(|s| s.to_str())
                        .map(str::to_lowercase)
                        .unwrap_or_else(|| e.to_lowercase());
                    if !exes.iter().any(|existing| existing == &exe_file) {
                        exes.push(exe_file);
                    }
                }
                let p_json = serde_json::to_string(&platforms).unwrap_or_default();
                let e_json = serde_json::to_string(&exes).unwrap_or_default();
                let _ = tx.execute(
                    "UPDATE game_catalog SET title = ?1, platforms = ?2, known_exes = ?3, meta_json = ?4, updated_at = ?5 WHERE id = ?6",
                    params![title, p_json, e_json, meta_json, now, id],
                );
                id
            }
            None => {
                let platforms = platform.map(|p| vec![p.to_string()]).unwrap_or_default();
                let exes = exe
                    .and_then(|e| {
                        Path::new(e)
                            .file_name()
                            .and_then(|s| s.to_str())
                            .map(|s| vec![s.to_lowercase()])
                    })
                    .unwrap_or_default();
                let p_json = serde_json::to_string(&platforms).unwrap_or_default();
                let e_json = serde_json::to_string(&exes).unwrap_or_default();
                let _ = tx.execute(
                    "INSERT INTO game_catalog (title, normalized_title, platforms, known_exes, meta_json, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![title, norm, p_json, e_json, meta_json, now],
                );
                tx.last_insert_rowid()
            }
        };

        // Add alias lookups
        let _ = tx.execute(
            "INSERT OR IGNORE INTO catalog_lookups (lookup_key, key_type, catalog_id) VALUES (?1, 'alias', ?2)",
            params![norm, catalog_id],
        );
        if !raw_norm.is_empty() && raw_norm != norm {
            let _ = tx.execute(
                "INSERT OR IGNORE INTO catalog_lookups (lookup_key, key_type, catalog_id) VALUES (?1, 'alias', ?2)",
                params![raw_norm, catalog_id],
            );
        }
        if let Some(e) = exe {
            if let Some(file_name) = Path::new(e).file_name().and_then(|s| s.to_str()) {
                let _ = tx.execute(
                    "INSERT OR IGNORE INTO catalog_lookups (lookup_key, key_type, catalog_id) VALUES (?1, 'exe', ?2)",
                    params![file_name.to_lowercase(), catalog_id],
                );
            }
        }

        let _ = tx.commit();
    }

    /// Export the entire catalog as compressed gzip bytes (`.db.gz` snapshot).
    pub fn export_catalog_compressed(&self) -> AppResult<Vec<u8>> {
        let mut stmt = self.conn.prepare(
            "SELECT title, normalized_title, platforms, known_exes, meta_json FROM game_catalog",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(CatalogExportRecord {
                title: row.get(0)?,
                normalized_title: row.get(1)?,
                platforms: Some(StringOrSeq::String(row.get(2)?)),
                known_exes: Some(StringOrSeq::String(row.get(3)?)),
                meta_json: Some(serde_json::Value::String(row.get(4)?)),
                meta: None,
            })
        })?;

        let mut records = Vec::new();
        for r in rows.flatten() {
            records.push(r);
        }
        let json_bytes = serde_json::to_vec(&records)?;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&json_bytes)?;
        Ok(encoder.finish()?)
    }

    /// Import catalog records from compressed gzip bytes into SQLite.
    pub fn import_catalog_compressed(&mut self, gz_bytes: &[u8]) -> AppResult<usize> {
        let mut decoder = GzDecoder::new(gz_bytes);
        let mut json_str = String::new();
        decoder.read_to_string(&mut json_str)?;

        let records: Vec<CatalogExportRecord> = serde_json::from_str(&json_str)?;
        let count = records.len();
        let now = chrono::Utc::now().timestamp();

        let tx = self.conn.transaction()?;
        for rec in records {
            let platforms_str = rec
                .platforms
                .map(|p| p.into_json_string())
                .unwrap_or_else(|| "[]".to_string());
            let known_exes_str = rec
                .known_exes
                .map(|e| e.into_json_string())
                .unwrap_or_else(|| "[]".to_string());
            let meta_json_str = if let Some(m) = rec.meta_json {
                match m {
                    serde_json::Value::String(s) => s,
                    other => serde_json::to_string(&other).unwrap_or_default(),
                }
            } else if let Some(m) = rec.meta {
                match m {
                    serde_json::Value::String(s) => s,
                    other => serde_json::to_string(&other).unwrap_or_default(),
                }
            } else {
                "{}".to_string()
            };

            let norm = if !rec.normalized_title.trim().is_empty() {
                normalize_catalog_key(&rec.normalized_title)
            } else {
                normalize_catalog_key(&rec.title)
            };
            let raw_norm: String = rec
                .title
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect();

            let _ = tx.execute(
                "INSERT OR REPLACE INTO game_catalog (title, normalized_title, platforms, known_exes, meta_json, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![rec.title, norm, platforms_str, known_exes_str, meta_json_str, now],
            );
            let catalog_id = tx.last_insert_rowid();
            let _ = tx.execute(
                "INSERT OR IGNORE INTO catalog_lookups (lookup_key, key_type, catalog_id) VALUES (?1, 'alias', ?2)",
                params![norm, catalog_id],
            );
            if !raw_norm.is_empty() && raw_norm != norm {
                let _ = tx.execute(
                    "INSERT OR IGNORE INTO catalog_lookups (lookup_key, key_type, catalog_id) VALUES (?1, 'alias', ?2)",
                    params![raw_norm, catalog_id],
                );
            }
            if !rec.normalized_title.is_empty()
                && rec.normalized_title != norm
                && rec.normalized_title != raw_norm
            {
                let _ = tx.execute(
                    "INSERT OR IGNORE INTO catalog_lookups (lookup_key, key_type, catalog_id) VALUES (?1, 'alias', ?2)",
                    params![rec.normalized_title, catalog_id],
                );
            }
            let exes: Vec<String> = serde_json::from_str(&known_exes_str).unwrap_or_default();
            for e in exes {
                let _ = tx.execute(
                    "INSERT OR IGNORE INTO catalog_lookups (lookup_key, key_type, catalog_id) VALUES (?1, 'exe', ?2)",
                    params![e.to_lowercase(), catalog_id],
                );
            }
        }
        tx.commit()?;
        Ok(count)
    }

    /// Ingest game metadata from the Windows uninstall registry (Windows only).
    ///
    /// This uses the registry purely for canonical title and publisher metadata;
    /// it NEVER sets or links uninstaller executables.
    #[cfg(windows)]
    pub fn ingest_uninstall_metadata(&mut self) {
        use crate::platforms::registry;

        for entry in registry::uninstall_entries() {
            let Some(publisher) = entry.publisher.as_deref().filter(|s| !s.trim().is_empty())
            else {
                continue;
            };
            if !crate::platforms::detectors::is_game_candidate(&entry.display_name, None, None) {
                continue;
            }
            let meta = AppMeta {
                name: Some(entry.display_name.clone()),
                developers: vec![publisher.to_string()],
                publishers: vec![publisher.to_string()],
                ..Default::default()
            };
            self.upsert_catalog_entry(&entry.display_name, &meta, None, None);
        }
    }

    #[cfg(not(windows))]
    pub fn ingest_uninstall_metadata(&mut self) {}
}

/// Convert a title to a clean lowercase alphanumeric key for indexing and lookups,
/// normalizing common roman numeral tokens (e.g. "ii" -> "2", "iii" -> "3").
pub fn normalize_catalog_key(s: &str) -> String {
    let lower = s.to_lowercase();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    let mut result = String::new();
    for w in words {
        match w {
            "i" => result.push('1'),
            "ii" => result.push('2'),
            "iii" => result.push('3'),
            "iv" => result.push('4'),
            "v" => result.push('5'),
            "vi" => result.push('6'),
            "vii" => result.push('7'),
            "viii" => result.push('8'),
            "ix" => result.push('9'),
            "x" => result.push_str("10"),
            _ => result.push_str(w),
        }
    }
    result
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
    fn catalog_lookups_and_compressed_export_import() {
        let conn = Connection::open_in_memory().unwrap();
        let mut cache = MetadataCache::with_connection(conn);

        let meta = AppMeta {
            app_id: 100,
            name: Some("Dragon Age II".into()),
            developers: vec!["BioWare".into()],
            publishers: vec!["Electronic Arts".into()],
            genres: vec!["RPG".into()],
            categories: vec!["Single-player".into()],
            tags: vec!["Story Rich".into(), "Fantasy".into()],
            release_date: Some("Mar 8, 2011".into()),
            release_year: Some(2011),
            short_description: Some("Experience the epic sequel.".into()),
            header_image: None,
            capsule_image: None,
            user_score: Some(8.0),
            fetched_at: Some(chrono::Utc::now().timestamp()),
            ..Default::default()
        };

        cache.upsert_catalog_entry(
            "Dragon Age II",
            &meta,
            Some("ea"),
            Some("E:\\Games\\Dragon Age II\\bin_ship\\DragonAge2.exe"),
        );

        assert_eq!(cache.catalog_count(), 1);

        // Lookup by exe
        let by_exe = cache
            .find_catalog_by_exe("DragonAge2.exe")
            .expect("should find by exe filename");
        assert_eq!(by_exe.name.as_deref(), Some("Dragon Age II"));
        assert_eq!(by_exe.release_year, Some(2011));

        // Lookup by title
        let by_title = cache
            .find_catalog_by_title("Dragon Age 2")
            .expect("should find by normalized title");
        assert_eq!(by_title.name.as_deref(), Some("Dragon Age II"));

        // Compressed export & import
        let gz_bytes = cache
            .export_catalog_compressed()
            .expect("should export compressed catalog");
        assert!(!gz_bytes.is_empty());

        let conn2 = Connection::open_in_memory().unwrap();
        let mut cache2 = MetadataCache::with_connection(conn2);
        let imported = cache2
            .import_catalog_compressed(&gz_bytes)
            .expect("should import compressed catalog");
        assert_eq!(imported, 1);
        assert_eq!(cache2.catalog_count(), 1);

        let imported_by_exe = cache2
            .find_catalog_by_exe("DragonAge2.exe")
            .expect("should find by exe in restored catalog");
        assert_eq!(imported_by_exe.name.as_deref(), Some("Dragon Age II"));

        // Test importing structured format (arrays for platforms & exes, object for meta)
        let structured_json = r#"[
            {
                "title": "Super Mario 64",
                "normalized_title": "super mario 64",
                "platforms": ["Nintendo 64"],
                "known_exes": ["sm64.z64", "sm64.n64"],
                "meta": {
                    "app_id": 0,
                    "name": "Super Mario 64",
                    "developers": ["Nintendo EAD"],
                    "publishers": ["Nintendo"],
                    "release_year": 1996
                }
            }
        ]"#;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(structured_json.as_bytes()).unwrap();
        let structured_gz = encoder.finish().unwrap();

        let conn3 = Connection::open_in_memory().unwrap();
        let mut cache3 = MetadataCache::with_connection(conn3);
        let imported_structured = cache3
            .import_catalog_compressed(&structured_gz)
            .expect("should import structured compressed catalog");
        assert_eq!(imported_structured, 1);
        assert_eq!(cache3.catalog_count(), 1);

        let n64_game = cache3
            .find_catalog_by_title("Super Mario 64")
            .expect("should find Super Mario 64 by title");
        assert_eq!(n64_game.name.as_deref(), Some("Super Mario 64"));
        assert_eq!(n64_game.release_year, Some(1996));

        let by_rom = cache3
            .find_catalog_by_exe("sm64.z64")
            .expect("should find by ROM filename");
        assert_eq!(by_rom.name.as_deref(), Some("Super Mario 64"));
    }
}
