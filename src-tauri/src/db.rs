use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRecord {
    pub id: String,
    pub title: String,
    pub platform: String,
    pub platform_id: Option<String>,
    pub install_dir: Option<String>,
    pub exe: Option<String>,
    pub launch_uri: Option<String>,
    pub install_uri: Option<String>,
    pub uninstall_uri: Option<String>,
    pub installed: bool,
    pub favorite: bool,
    pub hidden: bool,
    pub last_played: Option<i64>,
    pub playtime_minutes: i64,
    pub custom_title: Option<String>,
    pub tags: Vec<String>,
    pub icon_url: Option<String>,
    pub capsule_url: Option<String>,
    pub hero_url: Option<String>,
    pub logo_url: Option<String>,
    pub description: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub release_date: Option<String>,
    pub collections: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionRecord {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub favorite: bool,
    pub hidden: bool,
    pub filter_spec: Option<String>,
    pub game_count: usize,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginRepositoryRecord {
    pub name: String,
    pub url: String,
    pub enabled: bool,
}

pub struct Database {
    conn: Mutex<Connection>,
    pub path: PathBuf,
}

impl Database {
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self, rusqlite::Error> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn = Connection::open(p)?;

        // Performance pragmas
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA temp_store = MEMORY;
            PRAGMA foreign_keys = ON;
            "#,
        )?;

        // Tables schema
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS games (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                platform TEXT NOT NULL,
                platform_id TEXT,
                install_dir TEXT,
                exe TEXT,
                launch_uri TEXT,
                install_uri TEXT,
                uninstall_uri TEXT,
                installed INTEGER NOT NULL DEFAULT 1,
                favorite INTEGER NOT NULL DEFAULT 0,
                hidden INTEGER NOT NULL DEFAULT 0,
                last_played INTEGER,
                playtime_minutes INTEGER NOT NULL DEFAULT 0,
                custom_title TEXT,
                tags TEXT,
                icon_url TEXT,
                capsule_url TEXT,
                hero_url TEXT,
                logo_url TEXT,
                description TEXT,
                developer TEXT,
                publisher TEXT,
                release_date TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS collections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                icon TEXT,
                favorite INTEGER NOT NULL DEFAULT 0,
                hidden INTEGER NOT NULL DEFAULT 0,
                filter_spec TEXT,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS game_collections (
                game_id TEXT NOT NULL,
                collection_id TEXT NOT NULL,
                PRIMARY KEY (game_id, collection_id),
                FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE,
                FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS plugin_settings (
                plugin_id TEXT PRIMARY KEY,
                settings_json TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS plugin_repositories (
                url TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS kv_store (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                expires_at INTEGER
            );

            CREATE TABLE IF NOT EXISTS artwork_cache (
                key TEXT PRIMARY KEY,
                local_path TEXT NOT NULL,
                source_url TEXT,
                updated_at INTEGER NOT NULL
            );
            "#,
        )?;

        // Ensure default official repository exists
        conn.execute(
            r#"
            INSERT OR IGNORE INTO plugin_repositories (url, name, enabled)
            VALUES (?1, ?2, 1)
            "#,
            params![
                "https://raw.githubusercontent.com/HELIX-Origin/Universal-Game-Launcher-Plugins/main/index.json",
                "HELIX Origin Official Plugins"
            ],
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
            path: p.to_pathBuf(),
        })
    }

    // --- Games CRUD ---

    pub fn get_games(&self) -> Result<Vec<GameRecord>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, title, platform, platform_id, install_dir, exe, launch_uri, install_uri, uninstall_uri,
                installed, favorite, hidden, last_played, playtime_minutes, custom_title, tags,
                icon_url, capsule_url, hero_url, logo_url, description, developer, publisher, release_date,
                created_at, updated_at
            FROM games
            ORDER BY favorite DESC, title ASC
            "#,
        )?;

        let mut coll_stmt = conn.prepare("SELECT collection_id FROM game_collections WHERE game_id = ?1")?;

        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let tags_raw: Option<String> = row.get(15)?;
            let tags: Vec<String> = tags_raw
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            Ok((
                id,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
                row.get::<_, i64>(9)? != 0,
                row.get::<_, i64>(10)? != 0,
                row.get::<_, i64>(11)? != 0,
                row.get(12)?,
                row.get(13)?,
                row.get(14)?,
                tags,
                row.get(16)?,
                row.get(17)?,
                row.get(18)?,
                row.get(19)?,
                row.get(20)?,
                row.get(21)?,
                row.get(22)?,
                row.get(23)?,
                row.get(24)?,
                row.get(25)?,
            ))
        })?;

        let mut games = Vec::new();
        for r in rows {
            let (
                id, title, platform, platform_id, install_dir, exe, launch_uri, install_uri, uninstall_uri,
                installed, favorite, hidden, last_played, playtime_minutes, custom_title, tags,
                icon_url, capsule_url, hero_url, logo_url, description, developer, publisher, release_date,
                created_at, updated_at
            ) = r?;

            let coll_rows = coll_stmt.query_map([&id], |crow| crow.get::<_, String>(0))?;
            let collections: Vec<String> = coll_rows.filter_map(|c| c.ok()).collect();

            games.push(GameRecord {
                id,
                title,
                platform,
                platform_id,
                install_dir,
                exe,
                launch_uri,
                install_uri,
                uninstall_uri,
                installed,
                favorite,
                hidden,
                last_played,
                playtime_minutes,
                custom_title,
                tags,
                icon_url,
                capsule_url,
                hero_url,
                logo_url,
                description,
                developer,
                publisher,
                release_date,
                collections,
                created_at,
                updated_at,
            });
        }

        Ok(games)
    }

    pub fn upsert_game(&self, game: &GameRecord) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let tags_json = serde_json::to_string(&game.tags).unwrap_or_else(|_| "[]".into());
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            r#"
            INSERT INTO games (
                id, title, platform, platform_id, install_dir, exe, launch_uri, install_uri, uninstall_uri,
                installed, favorite, hidden, last_played, playtime_minutes, custom_title, tags,
                icon_url, capsule_url, hero_url, logo_url, description, developer, publisher, release_date,
                created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24,
                ?25, ?26
            )
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                platform = excluded.platform,
                platform_id = excluded.platform_id,
                install_dir = COALESCE(excluded.install_dir, games.install_dir),
                exe = COALESCE(excluded.exe, games.exe),
                launch_uri = COALESCE(excluded.launch_uri, games.launch_uri),
                install_uri = COALESCE(excluded.install_uri, games.install_uri),
                uninstall_uri = COALESCE(excluded.uninstall_uri, games.uninstall_uri),
                installed = excluded.installed,
                capsule_url = COALESCE(excluded.capsule_url, games.capsule_url),
                hero_url = COALESCE(excluded.hero_url, games.hero_url),
                logo_url = COALESCE(excluded.logo_url, games.logo_url),
                icon_url = COALESCE(excluded.icon_url, games.icon_url),
                description = COALESCE(excluded.description, games.description),
                developer = COALESCE(excluded.developer, games.developer),
                publisher = COALESCE(excluded.publisher, games.publisher),
                release_date = COALESCE(excluded.release_date, games.release_date),
                updated_at = excluded.updated_at
            "#,
            params![
                game.id,
                game.title,
                game.platform,
                game.platform_id,
                game.install_dir,
                game.exe,
                game.launch_uri,
                game.install_uri,
                game.uninstall_uri,
                if game.installed { 1 } else { 0 },
                if game.favorite { 1 } else { 0 },
                if game.hidden { 1 } else { 0 },
                game.last_played,
                game.playtime_minutes,
                game.custom_title,
                tags_json,
                game.icon_url,
                game.capsule_url,
                game.hero_url,
                game.logo_url,
                game.description,
                game.developer,
                game.publisher,
                game.release_date,
                if game.created_at > 0 { game.created_at } else { now },
                now,
            ],
        )?;

        // Update collections
        conn.execute("DELETE FROM game_collections WHERE game_id = ?1", params![game.id])?;
        for cid in &game.collections {
            let _ = conn.execute(
                "INSERT OR IGNORE INTO game_collections (game_id, collection_id) VALUES (?1, ?2)",
                params![game.id, cid],
            );
        }

        Ok(())
    }

    pub fn delete_game(&self, id: &str) -> Result<bool, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let changed = conn.execute("DELETE FROM games WHERE id = ?1", params![id])?;
        Ok(changed > 0)
    }

    pub fn set_favorite(&self, id: &str, favorite: bool) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE games SET favorite = ?1 WHERE id = ?2",
            params![if favorite { 1 } else { 0 }, id],
        )?;
        Ok(())
    }

    pub fn set_hidden(&self, id: &str, hidden: bool) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE games SET hidden = ?1 WHERE id = ?2",
            params![if hidden { 1 } else { 0 }, id],
        )?;
        Ok(())
    }

    pub fn update_playtime(&self, id: &str, delta_minutes: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            r#"
            UPDATE games
            SET playtime_minutes = playtime_minutes + ?1,
                last_played = ?2,
                updated_at = ?2
            WHERE id = ?3
            "#,
            params![delta_minutes, now, id],
        )?;
        Ok(())
    }

    // --- Collections CRUD ---

    pub fn get_collections(&self) -> Result<Vec<CollectionRecord>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT c.id, c.name, c.icon, c.favorite, c.hidden, c.filter_spec, c.created_at,
                   COUNT(gc.game_id) AS game_count
            FROM collections c
            LEFT JOIN game_collections gc ON c.id = gc.collection_id
            GROUP BY c.id
            ORDER BY c.favorite DESC, c.name ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(CollectionRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                icon: row.get(2)?,
                favorite: row.get::<_, i64>(3)? != 0,
                hidden: row.get::<_, i64>(4)? != 0,
                filter_spec: row.get(5)?,
                created_at: row.get(6)?,
                game_count: row.get::<_, i64>(7)? as usize,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn create_collection(&self, id: &str, name: &str, icon: Option<&str>) -> Result<CollectionRecord, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO collections (id, name, icon, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, icon, now],
        )?;
        Ok(CollectionRecord {
            id: id.to_string(),
            name: name.to_string(),
            icon: icon.map(|s| s.to_string()),
            favorite: false,
            hidden: false,
            filter_spec: None,
            game_count: 0,
            created_at: now,
        })
    }

    pub fn delete_collection(&self, id: &str) -> Result<bool, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let changed = conn.execute("DELETE FROM collections WHERE id = ?1", params![id])?;
        Ok(changed > 0)
    }

    // --- Repositories ---

    pub fn get_repositories(&self) -> Result<Vec<PluginRepositoryRecord>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT name, url, enabled FROM plugin_repositories ORDER BY name ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok(PluginRepositoryRecord {
                name: row.get(0)?,
                url: row.get(1)?,
                enabled: row.get::<_, i64>(2)? != 0,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn add_repository(&self, name: &str, url: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO plugin_repositories (name, url, enabled) VALUES (?1, ?2, 1)",
            params![name, url],
        )?;
        Ok(())
    }

    pub fn remove_repository(&self, url: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM plugin_repositories WHERE url = ?1", params![url])?;
        Ok(())
    }

    pub fn toggle_repository(&self, url: &str, enabled: bool) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE plugin_repositories SET enabled = ?1 WHERE url = ?2",
            params![if enabled { 1 } else { 0 }, url],
        )?;
        Ok(())
    }

    // --- Plugin Settings ---

    pub fn get_plugin_settings(&self, plugin_id: &str) -> Result<serde_json::Value, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT settings_json FROM plugin_settings WHERE plugin_id = ?1")?;
        let mut rows = stmt.query([plugin_id])?;
        if let Some(row) = rows.next()? {
            let json_str: String = row.get(0)?;
            let parsed = serde_json::from_str(&json_str).unwrap_or(serde_json::json!({}));
            Ok(parsed)
        } else {
            Ok(serde_json::json!({}))
        }
    }

    pub fn save_plugin_settings(&self, plugin_id: &str, settings: &serde_json::Value) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        let json_str = serde_json::to_string(settings).unwrap_or_else(|_| "{}".into());
        conn.execute(
            r#"
            INSERT INTO plugin_settings (plugin_id, settings_json, updated_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(plugin_id) DO UPDATE SET
                settings_json = excluded.settings_json,
                updated_at = excluded.updated_at
            "#,
            params![plugin_id, json_str, now],
        )?;
        Ok(())
    }

    // --- Key-Value Cache Store ---

    pub fn kv_get(&self, key: &str) -> Option<String> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        let mut stmt = conn
            .prepare("SELECT value, expires_at FROM kv_store WHERE key = ?1")
            .ok()?;
        let mut rows = stmt.query([key]).ok()?;
        if let Some(row) = rows.next().ok()? {
            let expires_at: Option<i64> = row.get(1).ok()?;
            if let Some(exp) = expires_at {
                if exp <= now {
                    return None;
                }
            }
            row.get(0).ok()
        } else {
            None
        }
    }

    pub fn kv_set(&self, key: &str, value: &str, ttl_secs: Option<i64>) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let expires_at = ttl_secs.map(|ttl| chrono::Utc::now().timestamp() + ttl);
        conn.execute(
            r#"
            INSERT INTO kv_store (key, value, expires_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                expires_at = excluded.expires_at
            "#,
            params![key, value, expires_at],
        )?;
        Ok(())
    }

    pub fn kv_delete(&self, key: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM kv_store WHERE key = ?1", params![key])?;
        Ok(())
    }
}
