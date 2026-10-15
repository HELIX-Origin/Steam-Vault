//! Tauri command handlers: the typed IPC surface for the Svelte frontend.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::autocat::AutoCatKind;
use crate::cache::MetadataCache;
use crate::error::{AppError, AppResult};
use crate::filter::Filter;
use crate::metadata::{self, providers::ProviderInfo, steam_web, steam_web::OwnedGame};
use crate::platforms::{self, Platform, PlatformGame, PlatformScan};
use crate::settings::{self, Settings};
use crate::steam::models::{AppMeta, Collection, Game, GameKind, SteamInstall};
use crate::steam::owned::{self, OwnedGameRecord, OwnedLibrary};
use crate::steam::storage::{self, MoveOutcome, MovePreview, StorageReport};
use crate::steam::{collections, library, paths, process, shortcuts};

/// Shared application state managed by Tauri.
pub struct AppState {
    pub settings: Mutex<Settings>,
    pub cache: Mutex<MetadataCache>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            settings: Mutex::new(settings::load()),
            cache: Mutex::new(MetadataCache::open()),
        }
    }

    pub fn settings_snapshot(&self) -> Settings {
        self.settings.lock().expect("settings lock poisoned").clone()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

fn steam_root(state: &AppState) -> AppResult<PathBuf> {
    let settings = state.settings_snapshot();
    if let Some(root) = settings.steam_root.filter(|root| !root.is_empty()) {
        return Ok(PathBuf::from(root));
    }
    paths::discover_steam_root().ok_or_else(|| {
        AppError::NotFound("Steam installation not found. Set the Steam folder in Settings.".into())
    })
}

fn resolve_account(root: &Path, state: &AppState, account: Option<String>) -> Option<String> {
    let settings = state.settings_snapshot();
    account
        .filter(|value| !value.is_empty())
        .or(settings.account_id3)
        .or_else(|| paths::read_accounts(root).first().map(|a| a.id3.clone()))
}

/// Result of computing AutoCat assignments (dry-run, review before applying).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoCatAssignment {
    pub collection: String,
    pub app_ids: Vec<u32>,
}

/// A proposed category rename/addition computed from AutoCats.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoCatPlan {
    pub assignments: Vec<AutoCatAssignment>,
    pub games_without_metadata: Vec<u32>,
}

// ---------------------------------------------------------------------------
// Discovery & settings
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_providers() -> Vec<ProviderInfo> {
    metadata::providers::all()
}

#[tauri::command]
pub fn detect_steam() -> Option<SteamInstall> {
    let root = paths::discover_steam_root()?;
    paths::load_install(&root).ok()
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings_snapshot()
}

#[tauri::command]
pub fn save_settings(settings: Settings, state: State<'_, AppState>) -> AppResult<Settings> {
    settings::save(&settings)?;
    *state.settings.lock().expect("settings lock poisoned") = settings.clone();
    Ok(settings)
}

#[tauri::command]
pub fn is_steam_running() -> bool {
    process::is_steam_running()
}

// ---------------------------------------------------------------------------
// Library & collections
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn load_library(account_id3: Option<String>, state: State<'_, AppState>) -> AppResult<Vec<Game>> {
    let root = steam_root(&state)?;
    let account = resolve_account(&root, &state, account_id3);
    collect_library(&root, account.as_deref())
}

/// Assemble the full library for organization.
///
/// This is intentionally **not** limited to installed games: the entire owned
/// Steam library (cached from the Steam Web API) is merged in, so most of a
/// large library can still be organized even when it is not installed locally.
/// Non-Steam shortcuts and resolved collection membership are included too.
fn collect_library(root: &Path, account: Option<&str>) -> AppResult<Vec<Game>> {
    let mut games = library::load_games(root)?;

    if let Some(id3) = account {
        let stats = library::load_app_stats(root, id3);
        for game in games.iter_mut() {
            if let Some((last_played, playtime)) = stats.get(&game.app_id) {
                game.last_played = *last_played;
                game.playtime_minutes = *playtime;
            }
        }

        if let Ok(non_steam) = shortcuts::load_shortcuts(root, id3) {
            games.extend(non_steam);
        }
    }

    merge_owned(&mut games, &owned::load());

    if let Some(id3) = account {
        if let Ok(Some(file)) = collections::load_collections_file(root, id3) {
            apply_membership(&mut games, &file.collections());
        }
    }

    Ok(games)
}

/// Fold the cached owned library into the local game list. Games already known
/// from disk keep their install data; owned-only games are added as
/// not-installed Steam entries.
fn merge_owned(games: &mut Vec<Game>, owned: &OwnedLibrary) {
    let mut index: BTreeMap<u32, usize> = games
        .iter()
        .enumerate()
        .map(|(position, game)| (game.app_id, position))
        .collect();

    for record in &owned.games {
        match index.get(&record.app_id) {
            Some(&position) => {
                let game = &mut games[position];
                if game.name.is_empty() {
                    game.name = record.name.clone();
                }
                if game.playtime_minutes.is_none() {
                    game.playtime_minutes = Some(record.playtime_minutes);
                }
            }
            None => {
                index.insert(record.app_id, games.len());
                games.push(Game {
                    app_id: record.app_id,
                    name: record.name.clone(),
                    kind: GameKind::Steam,
                    installed: false,
                    playtime_minutes: Some(record.playtime_minutes),
                    ..Default::default()
                });
            }
        }
    }
}

fn apply_membership(games: &mut [Game], collections_list: &[Collection]) {
    let mut membership: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    let mut favorites: BTreeSet<u32> = BTreeSet::new();
    let mut hidden: BTreeSet<u32> = BTreeSet::new();

    for collection in collections_list {
        for app_id in &collection.added {
            membership
                .entry(*app_id)
                .or_default()
                .push(collection.name.clone());
            if collection.favorite {
                favorites.insert(*app_id);
            }
            if collection.hidden {
                hidden.insert(*app_id);
            }
        }
    }

    for game in games.iter_mut() {
        if let Some(names) = membership.get(&game.app_id) {
            game.collections = names.clone();
        }
        game.favorite = favorites.contains(&game.app_id);
        game.hidden = hidden.contains(&game.app_id);
    }
}

#[tauri::command]
pub fn load_collections(
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<Vec<Collection>> {
    let root = steam_root(&state)?;
    let account = resolve_account(&root, &state, account_id3);
    let Some(id3) = account else {
        return Ok(Vec::new());
    };
    match collections::load_collections_file(&root, &id3)? {
        Some(file) => Ok(file.collections()),
        None => Ok(Vec::new()),
    }
}

fn ensure_steam_closed() -> AppResult<()> {
    if process::is_steam_running() {
        return Err(AppError::SteamRunning(
            "Steam is running. Close it completely, then try again.".into(),
        ));
    }
    Ok(())
}

#[tauri::command]
pub fn update_collection(
    collection_id: String,
    name: Option<String>,
    add_app_ids: Option<Vec<u32>>,
    remove_app_ids: Option<Vec<u32>>,
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<Collection> {
    ensure_steam_closed()?;
    let root = steam_root(&state)?;
    let id3 = resolve_account(&root, &state, account_id3)
        .ok_or_else(|| AppError::NotFound("No Steam account selected.".into()))?;

    let mut file = collections::load_collections_file(&root, &id3)?
        .ok_or_else(|| AppError::NotFound("No Steam collection store found for this account.".into()))?;

    let mut collection = file
        .collections()
        .into_iter()
        .find(|c| c.id == collection_id)
        .unwrap_or(Collection {
            id: collection_id.clone(),
            name: collection_id.clone(),
            favorite: collection_id == "favorite",
            hidden: collection_id == "hidden",
            dynamic: false,
            added: Vec::new(),
            removed: Vec::new(),
        });

    if let Some(name) = name {
        collection.name = name;
    }
    if let Some(add) = add_app_ids {
        for app_id in add {
            if !collection.added.contains(&app_id) {
                collection.added.push(app_id);
            }
            collection.removed.retain(|id| *id != app_id);
        }
    }
    if let Some(remove) = remove_app_ids {
        for app_id in remove {
            collection.added.retain(|id| *id != app_id);
            if !collection.removed.contains(&app_id) {
                collection.removed.push(app_id);
            }
        }
    }

    crate::backup::backup_file(&file.path)?;
    file.upsert(&collection)?;
    file.save()?;
    Ok(collection)
}

#[tauri::command]
pub fn create_collection(
    name: String,
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<Collection> {
    ensure_steam_closed()?;
    let root = steam_root(&state)?;
    let id3 = resolve_account(&root, &state, account_id3)
        .ok_or_else(|| AppError::NotFound("No Steam account selected.".into()))?;

    let mut file = collections::load_collections_file(&root, &id3)?
        .ok_or_else(|| AppError::NotFound("No Steam collection store found for this account.".into()))?;

    let nanos = chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default();
    let collection = Collection {
        id: format!("uc-{nanos:x}"),
        name,
        favorite: false,
        hidden: false,
        dynamic: false,
        added: Vec::new(),
        removed: Vec::new(),
    };

    crate::backup::backup_file(&file.path)?;
    file.upsert(&collection)?;
    file.save()?;
    Ok(collection)
}

#[tauri::command]
pub fn delete_collection(
    collection_id: String,
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<()> {
    ensure_steam_closed()?;
    let root = steam_root(&state)?;
    let id3 = resolve_account(&root, &state, account_id3)
        .ok_or_else(|| AppError::NotFound("No Steam account selected.".into()))?;

    let mut file = collections::load_collections_file(&root, &id3)?
        .ok_or_else(|| AppError::NotFound("No Steam collection store found for this account.".into()))?;
    crate::backup::backup_file(&file.path)?;
    file.delete(&collection_id);
    file.save()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Metadata
// ---------------------------------------------------------------------------

const CACHE_MAX_AGE_SECS: i64 = 60 * 60 * 24 * 7;

#[tauri::command]
pub async fn fetch_steam_store_metadata(
    app_ids: Vec<u32>,
    state: State<'_, AppState>,
) -> AppResult<Vec<AppMeta>> {
    let client = metadata::http_client();
    let settings = state.settings_snapshot();

    let mut result = Vec::new();
    let mut missing = Vec::new();
    {
        let cache = state.cache.lock().expect("cache lock poisoned");
        for app_id in &app_ids {
            match cache.get("steam-store", *app_id, Some(CACHE_MAX_AGE_SECS)) {
                Some(meta) => result.push(meta.clone()),
                None => missing.push(*app_id),
            }
        }
    }

    if !missing.is_empty() {
        let fetched = metadata::steam_store::fetch_app_details(
            &client,
            &missing,
            &settings.country_code,
            &settings.language,
        )
        .await?;
        let mut cache = state.cache.lock().expect("cache lock poisoned");
        for (app_id, meta) in fetched {
            cache.put("steam-store", app_id, &meta);
            result.push(meta);
        }
        let _ = cache.save();
    }

    Ok(result)
}

/// Maximum app ids per `appdetails` request (the endpoint accepts a CSV list).
const METADATA_BATCH_SIZE: usize = 50;

/// Summary of a metadata sync pass, surfaced to the UI.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataSyncResult {
    /// Distinct app ids considered.
    pub requested: usize,
    /// Entries newly downloaded from the Steam Store.
    pub fetched: usize,
    /// Community tag lists newly downloaded.
    pub tags_fetched: usize,
    /// Entries already fresh in the local cache.
    pub cached: usize,
    /// Ids the store could not answer for (not a store app, delisted, or a
    /// transient network failure).
    pub unavailable: usize,
}

/// Ensure the metadata cache holds fresh entries for every id in `app_ids`.
///
/// Steam's `appdetails` endpoint only returns data for real store apps, so
/// callers pass the ids they actually intend to categorize; unknown ids simply
/// count as [`MetadataSyncResult::unavailable`]. Entries younger than
/// [`CACHE_MAX_AGE_SECS`] are reused. Network failures are tolerated so that
/// categorization still works offline for kinds that do not need metadata.
///
/// When `want_tags` is set, Steam community tags are fetched too (one request
/// per app) and cached under the `steam-tags` provider key.
pub async fn ensure_steam_metadata(
    state: &AppState,
    app_ids: &[u32],
    want_tags: bool,
) -> MetadataSyncResult {
    let mut unique: Vec<u32> = app_ids.iter().copied().filter(|id| *id != 0).collect();
    unique.sort_unstable();
    unique.dedup();

    // Split into entries we can reuse and entries we must fetch.
    let missing: Vec<u32> = {
        let cache = state.cache.lock().expect("cache lock poisoned");
        unique
            .iter()
            .copied()
            .filter(|id| cache.get("steam-store", *id, Some(CACHE_MAX_AGE_SECS)).is_none())
            .collect()
    };

    let requested = unique.len();
    let cached = requested - missing.len();
    let mut fetched = 0usize;

    if !missing.is_empty() {
        let settings = state.settings_snapshot();
        let client = metadata::http_client();
        for chunk in missing.chunks(METADATA_BATCH_SIZE) {
            if let Ok(map) = metadata::steam_store::fetch_app_details(
                &client,
                chunk,
                &settings.country_code,
                &settings.language,
            )
            .await
            {
                let mut cache = state.cache.lock().expect("cache lock poisoned");
                for (app_id, meta) in &map {
                    cache.put("steam-store", *app_id, meta);
                }
                let _ = cache.save();
                fetched += map.len();
            }
        }
    }

    // Community tags come from a separate (per-app) endpoint, so they are only
    // fetched when a caller actually needs them.
    let mut tags_fetched = 0usize;
    if want_tags {
        let needing: Vec<u32> = {
            let cache = state.cache.lock().expect("cache lock poisoned");
            unique
                .iter()
                .copied()
                .filter(|id| cache.get("steam-tags", *id, Some(CACHE_MAX_AGE_SECS)).is_none())
                .collect()
        };
        if !needing.is_empty() {
            let settings = state.settings_snapshot();
            let client = metadata::http_client();
            let tags = metadata::steam_store::fetch_tags(
                &client,
                &needing,
                &settings.country_code,
                &settings.language,
            )
            .await;
            let mut cache = state.cache.lock().expect("cache lock poisoned");
            for (app_id, names) in &tags {
                let meta = AppMeta {
                    app_id: *app_id,
                    tags: names.clone(),
                    ..Default::default()
                };
                cache.put("steam-tags", *app_id, &meta);
            }
            let _ = cache.save();
            tags_fetched = tags.len();
        }
    }

    MetadataSyncResult {
        requested,
        fetched,
        tags_fetched,
        cached,
        unavailable: requested.saturating_sub(cached + fetched),
    }
}

/// Warm (or refresh) store metadata for the whole local library.
#[tauri::command]
pub async fn sync_library_metadata(
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<MetadataSyncResult> {
    let root = steam_root(&state)?;
    let account = resolve_account(&root, &state, account_id3);
    let games = collect_library(&root, account.as_deref())?;
    let ids: Vec<u32> = games
        .iter()
        .filter(|game| game.kind == GameKind::Steam && game.app_id != 0)
        .map(|game| game.app_id)
        .collect();
    Ok(ensure_steam_metadata(&state, &ids, false).await)
}

#[tauri::command]
pub async fn fetch_owned_games(state: State<'_, AppState>) -> AppResult<Vec<OwnedGame>> {
    let settings = state.settings_snapshot();
    let key = settings
        .steam_api_key
        .filter(|key| !key.is_empty())
        .ok_or_else(|| AppError::msg("No Steam Web API key configured."))?;
    let steamid64 = settings
        .account_id3
        .as_deref()
        .and_then(paths::id3_to_steamid64)
        .map(|v| v.to_string())
        .ok_or_else(|| AppError::msg("No SteamID64 available for the selected account."))?;
    let client = metadata::http_client();
    steam_web::get_owned_games(&client, &key, &steamid64, true).await
}

#[tauri::command]
pub async fn steamgriddb_search(
    term: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<metadata::steamgriddb::SgdbGame>> {
    let key = require_key(&state, "steamgriddb")?;
    let client = metadata::http_client();
    metadata::steamgriddb::search_games(&client, &key, &term).await
}

#[tauri::command]
pub async fn steamgriddb_assets(
    game_id: u32,
    kind: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<metadata::steamgriddb::SgdbAsset>> {
    let key = require_key(&state, "steamgriddb")?;
    let client = metadata::http_client();
    match kind.as_str() {
        "grids" => metadata::steamgriddb::grids(&client, &key, game_id, Some("600x900")).await,
        "heroes" => metadata::steamgriddb::heroes(&client, &key, game_id).await,
        "logos" => metadata::steamgriddb::logos(&client, &key, game_id).await,
        "icons" => metadata::steamgriddb::icons(&client, &key, game_id).await,
        other => Err(AppError::Invalid(format!("unknown artwork kind '{other}'"))),
    }
}

#[tauri::command]
pub async fn igdb_search(
    query: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<metadata::igdb::IgdbGame>> {
    let settings = state.settings_snapshot();
    let client_id = settings
        .igdb_client_id
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::msg("No IGDB client id configured."))?;
    let client_secret = settings
        .igdb_client_secret
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::msg("No IGDB client secret configured."))?;

    let client = metadata::http_client();
    let token = metadata::igdb::request_token(&client, &client_id, &client_secret).await?;
    metadata::igdb::search_games(&client, &client_id, &token, &query, 20).await
}

#[tauri::command]
pub async fn pcgamingwiki_lookup(name: String) -> AppResult<Option<AppMeta>> {
    let client = metadata::http_client();
    metadata::extra::pcgamingwiki_lookup(&client, &name).await
}

#[tauri::command]
pub async fn wikipedia_summary(title: String) -> AppResult<Option<String>> {
    let client = metadata::http_client();
    metadata::extra::wikipedia_summary(&client, &title).await
}

fn require_key(state: &AppState, provider: &str) -> AppResult<String> {
    let settings = state.settings_snapshot();
    let key = match provider {
        "steamgriddb" => settings.steamgriddb_api_key,
        "steam" => settings.steam_api_key,
        _ => None,
    };
    key.filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::msg(format!("No API key configured for {provider}.")))
}

// ---------------------------------------------------------------------------
// AutoCats
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn compute_autocats(
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<AutoCatPlan> {
    let root = steam_root(&state)?;
    let account = resolve_account(&root, &state, account_id3);
    let games = collect_library(&root, account.as_deref())?;

    let autos = state.settings_snapshot().auto_cats;

    // Only reach out to the store when at least one enabled rule needs it.
    let wants_metadata = autos
        .iter()
        .any(|autocat| autocat.enabled && autocat.kind.needs_metadata());
    let wants_tags = autos
        .iter()
        .any(|autocat| autocat.enabled && autocat.kind.needs_tags());
    if wants_metadata || wants_tags {
        let ids: Vec<u32> = games
            .iter()
            .filter(|game| game.kind == GameKind::Steam)
            .map(|game| game.app_id)
            .collect();
        ensure_steam_metadata(&state, &ids, wants_tags).await;
    }

    let cache = state.cache.lock().expect("cache lock poisoned");
    let mut buckets: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    let mut missing_metadata = Vec::new();

    for game in &games {
        let meta = cache.get("steam-store", game.app_id, None).cloned();
        if meta.is_none() && game.kind == GameKind::Steam && wants_metadata {
            missing_metadata.push(game.app_id);
        }
        let mut meta = meta.unwrap_or_default();
        if wants_tags {
            if let Some(tags) = cache.get("steam-tags", game.app_id, None) {
                meta.tags = tags.tags.clone();
            }
        }
        for autocat in &autos {
            for name in crate::autocat::apply(autocat, game, &meta) {
                buckets.entry(name).or_default().insert(game.app_id);
            }
        }
    }

    let assignments = buckets
        .into_iter()
        .map(|(collection, ids)| AutoCatAssignment {
            collection,
            app_ids: ids.into_iter().collect(),
        })
        .collect();

    Ok(AutoCatPlan {
        assignments,
        games_without_metadata: missing_metadata,
    })
}

// ---------------------------------------------------------------------------
// External platforms & the full owned library
// ---------------------------------------------------------------------------

/// The outcome of importing platform games as non-Steam shortcuts.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub added: Vec<Game>,
    pub skipped: Vec<String>,
}

/// Scan the machine for games installed by GOG, Epic, itch.io, and others.
///
/// Honours the per-platform preferences saved in Settings and looks on every
/// mounted drive, so games installed on an external drive are discovered too.
#[tauri::command]
pub fn detect_platforms(state: State<'_, AppState>) -> PlatformScan {
    let settings = state.settings_snapshot();
    platforms::scan_all(&platforms::ScanConfig {
        sources: settings.platform_sources,
        include_other_drives: true,
    })
}

/// Import selected platform games into Steam as non-Steam shortcuts.
#[tauri::command]
pub fn import_platform_games(
    games: Vec<PlatformGame>,
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<ImportResult> {
    ensure_steam_closed()?;
    let root = steam_root(&state)?;
    let id3 = resolve_account(&root, &state, account_id3)
        .ok_or_else(|| AppError::NotFound("No Steam account selected.".into()))?;

    let mut skipped = Vec::new();
    let mut candidates = Vec::new();
    for game in games {
        let Some(exe) = game.exe.clone().filter(|value| !value.is_empty()) else {
            skipped.push(format!("{} (no executable found)", game.name));
            continue;
        };
        let start_dir = game
            .install_dir
            .clone()
            .or_else(|| {
                Path::new(&exe)
                    .parent()
                    .map(|parent| parent.display().to_string())
            })
            .unwrap_or_default();
        candidates.push(shortcuts::NewShortcut {
            name: game.name.clone(),
            exe,
            start_dir,
            icon: String::new(),
            launch_options: String::new(),
            platform: game.platform,
        });
    }

    let outcome = shortcuts::add_shortcuts(&root, &id3, &candidates)?;
    if !outcome.added.is_empty() {
        let pairs: Vec<(u32, Platform)> = outcome
            .added
            .iter()
            .map(|game| (game.app_id, game.platform))
            .collect();
        let _ = platforms::record(&pairs);
    }
    skipped.extend(outcome.skipped);

    Ok(ImportResult {
        added: outcome.added,
        skipped,
    })
}

/// Return the cached owned Steam library (may be empty before a refresh).
#[tauri::command]
pub fn get_owned_library() -> OwnedLibrary {
    owned::load()
}

/// Fetch the user's **entire** owned Steam library via the Steam Web API and
/// cache it, so uninstalled games can be organized too.
#[tauri::command]
pub async fn refresh_owned_games(state: State<'_, AppState>) -> AppResult<OwnedLibrary> {
    let settings = state.settings_snapshot();
    let key = settings
        .steam_api_key
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::msg("No Steam Web API key configured."))?;
    let steamid64 = settings
        .account_id3
        .as_deref()
        .and_then(paths::id3_to_steamid64)
        .map(|value| value.to_string())
        .ok_or_else(|| AppError::msg("No SteamID64 available for the selected account."))?;

    let client = metadata::http_client();
    let owned_games = steam_web::get_owned_games(&client, &key, &steamid64, true).await?;

    let library = OwnedLibrary {
        fetched_at: Some(chrono::Utc::now().timestamp()),
        steam_id64: Some(steamid64),
        games: owned_games
            .into_iter()
            .map(|game| OwnedGameRecord {
                app_id: game.appid,
                name: game.name.unwrap_or_else(|| format!("App {}", game.appid)),
                playtime_minutes: game.playtime_forever.unwrap_or(0),
                playtime_two_weeks: game.playtime_2weeks.unwrap_or(0),
                icon_url: game
                    .img_icon_url
                    .map(|hash| owned::icon_url(game.appid, &hash)),
            })
            .collect(),
    };
    owned::save(&library)?;
    Ok(library)
}

// ---------------------------------------------------------------------------
// Create collections from a chosen sorting option
// ---------------------------------------------------------------------------

/// One sort key plus options, used to build collections in bulk.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategorizeRequest {
    pub kind: AutoCatKind,
    pub prefix: Option<String>,
    pub max_categories: Option<usize>,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Replace each collection's membership with the computed set.
    #[serde(default)]
    pub remove_existing: bool,
    /// Only include games installed locally.
    #[serde(default)]
    pub only_installed: bool,
    /// Download missing store metadata before computing (default on).
    #[serde(default = "default_true")]
    pub fetch_metadata: bool,
    /// Restrict to games matching this filter (e.g. only installed games).
    pub filter: Option<Filter>,
}

/// Whether a game passes the request's scope filters (installed-only / filter).
fn in_scope(game: &Game, request: &CategorizeRequest) -> bool {
    if request.only_installed && !game.installed {
        return false;
    }
    if let Some(filter) = &request.filter {
        if !crate::filter::matches(filter, game) {
            return false;
        }
    }
    true
}

fn default_true() -> bool {
    true
}

/// A dry-run preview of the collections a request would create.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategorizationPlan {
    pub kind: AutoCatKind,
    pub assignments: Vec<AutoCatAssignment>,
    pub categorized_games: usize,
    pub total_games: usize,
    pub games_without_metadata: Vec<u32>,
    #[serde(default)]
    pub remove_existing: bool,
}

/// The outcome of applying a categorization plan.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
    pub collections_created: usize,
    pub collections_updated: usize,
    pub games_assigned: usize,
}

fn build_autocat(request: &CategorizeRequest) -> crate::autocat::AutoCat {
    crate::autocat::AutoCat {
        id: "create-collections".into(),
        name: "Create collections".into(),
        kind: request.kind,
        enabled: true,
        prefix: request.prefix.clone(),
        max_categories: request.max_categories,
        remove_existing: request.remove_existing,
        include: request.include.clone(),
        exclude: request.exclude.clone(),
    }
}

/// Compute (without writing) the collections a sorting option would produce.
#[tauri::command]
pub async fn compute_categorization(
    request: CategorizeRequest,
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<CategorizationPlan> {
    let root = steam_root(&state)?;
    let account = resolve_account(&root, &state, account_id3);
    let games = collect_library(&root, account.as_deref())?;

    // The games this request actually considers.
    let scoped: Vec<&Game> = games
        .iter()
        .filter(|game| in_scope(game, &request))
        .collect();

    // Pull in any missing store metadata before computing, so metadata-based
    // kinds (genre, developer, release year, …) have data to work with, and
    // community tags when the Tag kind was chosen. Shortcuts are never in the
    // store, so only Steam apps are requested.
    let wants_metadata = request.kind.needs_metadata();
    let wants_tags = request.kind.needs_tags();
    if request.fetch_metadata && (wants_metadata || wants_tags) {
        let ids: Vec<u32> = scoped
            .iter()
            .filter(|game| game.kind == GameKind::Steam)
            .map(|game| game.app_id)
            .collect();
        ensure_steam_metadata(&state, &ids, wants_tags).await;
    }

    let autocat = build_autocat(&request);
    let cache = state.cache.lock().expect("cache lock poisoned");

    let mut buckets: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    let mut missing_metadata = Vec::new();
    let mut categorized = 0usize;

    for game in &scoped {
        let stored = cache.get("steam-store", game.app_id, None).cloned();
        if stored.is_none() && game.kind == GameKind::Steam && wants_metadata {
            missing_metadata.push(game.app_id);
        }
        let mut meta = stored.unwrap_or_default();
        if wants_tags {
            if let Some(tags) = cache.get("steam-tags", game.app_id, None) {
                meta.tags = tags.tags.clone();
            }
        }
        let names = crate::autocat::apply(&autocat, game, &meta);
        if names.is_empty() {
            continue;
        }
        categorized += 1;
        for name in names {
            buckets.entry(name).or_default().insert(game.app_id);
        }
    }

    let assignments = buckets
        .into_iter()
        .map(|(collection, ids)| AutoCatAssignment {
            collection,
            app_ids: ids.into_iter().collect(),
        })
        .collect();

    Ok(CategorizationPlan {
        kind: request.kind,
        assignments,
        categorized_games: categorized,
        total_games: scoped.len(),
        games_without_metadata: missing_metadata,
        remove_existing: request.remove_existing,
    })
}

/// Apply a categorization plan: create/update collections in one atomic write.
#[tauri::command]
pub fn apply_categorization(
    plan: CategorizationPlan,
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<ApplyResult> {
    ensure_steam_closed()?;
    let root = steam_root(&state)?;
    let id3 = resolve_account(&root, &state, account_id3)
        .ok_or_else(|| AppError::NotFound("No Steam account selected.".into()))?;

    let mut file = collections::load_collections_file(&root, &id3)?.ok_or_else(|| {
        AppError::NotFound("No Steam collection store found for this account.".into())
    })?;
    crate::backup::backup_file(&file.path)?;

    let existing = file.collections();
    let mut created = 0usize;
    let mut updated = 0usize;
    let mut assigned = 0usize;

    for assignment in &plan.assignments {
        let found = existing
            .iter()
            .position(|collection| collection.name.eq_ignore_ascii_case(&assignment.collection));
        if let Some(index) = found {
            // Never overwrite Steam's reserved Favorites / Hidden collections.
            if existing[index].id == "favorite" || existing[index].id == "hidden" {
                continue;
            }
        }

        let (mut collection, was_new) = match found {
            Some(index) => (existing[index].clone(), false),
            None => {
                created += 1;
                let nanos = chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default();
                (
                    Collection {
                        id: format!("uc-{nanos:x}-{created}"),
                        name: assignment.collection.clone(),
                        favorite: false,
                        hidden: false,
                        dynamic: false,
                        added: Vec::new(),
                        removed: Vec::new(),
                    },
                    true,
                )
            }
        };

        if plan.remove_existing {
            let target: BTreeSet<u32> = assignment.app_ids.iter().copied().collect();
            for app_id in &collection.added {
                if !target.contains(app_id) && !collection.removed.contains(app_id) {
                    collection.removed.push(*app_id);
                }
            }
            collection
                .removed
                .retain(|app_id| !target.contains(app_id));
            collection.added = target.into_iter().collect();
        } else {
            for app_id in &assignment.app_ids {
                if !collection.added.contains(app_id) {
                    collection.added.push(*app_id);
                }
                collection.removed.retain(|existing| existing != app_id);
            }
        }

        assigned += assignment.app_ids.len();
        file.upsert(&collection)?;
        if !was_new {
            updated += 1;
        }
    }

    file.save()?;
    Ok(ApplyResult {
        collections_created: created,
        collections_updated: updated,
        games_assigned: assigned,
    })
}

// ---------------------------------------------------------------------------
// Storage management
// ---------------------------------------------------------------------------

/// Inspect every registered Steam library folder, its games, and the mounted
/// disks — everything the Storage view needs in one round-trip.
#[tauri::command]
pub fn get_storage_report(state: State<'_, AppState>) -> AppResult<StorageReport> {
    let root = steam_root(&state)?;
    storage::report(&root)
}

/// Dry-run a move of a whole library folder: sizes, free space, warnings.
#[tauri::command]
pub fn preview_move_library(
    from: String,
    to: String,
    state: State<'_, AppState>,
) -> AppResult<MovePreview> {
    let root = steam_root(&state)?;
    storage::preview_move_library(&root, &from, &to)
}

/// Move a whole library folder to a new path and update every
/// `libraryfolders.vdf`. Runs off the main thread so progress events flow.
#[tauri::command]
pub async fn move_library(
    from: String,
    to: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> AppResult<MoveOutcome> {
    ensure_steam_closed()?;
    let root = steam_root(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        storage::move_library(&root, &from, &to, Some(&app))
    })
    .await
    .map_err(|error| AppError::Invalid(format!("The move task failed: {error}")))?
}

/// Dry-run a move of one installed game to another library folder.
#[tauri::command]
pub fn preview_move_game(
    app_id: u32,
    target: String,
    state: State<'_, AppState>,
) -> AppResult<MovePreview> {
    let root = steam_root(&state)?;
    storage::preview_move_game(&root, app_id, &target)
}

/// Move one installed game to another library folder, updating every
/// `libraryfolders.vdf`. Runs off the main thread so progress events flow.
#[tauri::command]
pub async fn move_game(
    app_id: u32,
    target: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> AppResult<MoveOutcome> {
    ensure_steam_closed()?;
    let root = steam_root(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        storage::move_game(&root, app_id, &target, Some(&app))
    })
    .await
    .map_err(|error| AppError::Invalid(format!("The move task failed: {error}")))?
}

/// Register a new library folder (its `steamapps` directory is created).
#[tauri::command]
pub fn add_library_folder(path: String, state: State<'_, AppState>) -> AppResult<Vec<String>> {
    ensure_steam_closed()?;
    let root = steam_root(&state)?;
    storage::add_library_folder(&root, &path)
}

/// Unregister an empty library folder (the files on disk are left untouched).
#[tauri::command]
pub fn remove_library_folder(path: String, state: State<'_, AppState>) -> AppResult<Vec<String>> {
    ensure_steam_closed()?;
    let root = steam_root(&state)?;
    storage::remove_library_folder(&root, &path)
}
