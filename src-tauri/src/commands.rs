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
use crate::steam::artwork::{ArtworkOutcome, ArtworkSlot, GameArtwork};
use crate::steam::models::{AppMeta, Collection, Game, GameKind, SteamInstall};
use crate::steam::owned::{self, OwnedGameRecord, OwnedLibrary};
use crate::steam::storage::{self, MoveOutcome, MovePreview, StorageReport};
use crate::steam::{artwork, collections, library, paths, process, shortcuts};

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
        self.settings
            .lock()
            .expect("settings lock poisoned")
            .clone()
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
pub fn load_library(
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<Vec<Game>> {
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

    let mut file = collections::load_collections_file(&root, &id3)?.ok_or_else(|| {
        AppError::NotFound("No Steam collection store found for this account.".into())
    })?;

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
            filter_spec: None,
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

    let mut file = collections::load_collections_file(&root, &id3)?.ok_or_else(|| {
        AppError::NotFound("No Steam collection store found for this account.".into())
    })?;

    let nanos = chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default();
    let collection = Collection {
        id: format!("uc-{nanos:x}"),
        name,
        favorite: false,
        hidden: false,
        dynamic: false,
        added: Vec::new(),
        removed: Vec::new(),
        filter_spec: None,
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

    let mut file = collections::load_collections_file(&root, &id3)?.ok_or_else(|| {
        AppError::NotFound("No Steam collection store found for this account.".into())
    })?;
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
                Some(meta) => result.push(meta),
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
            .filter(|id| {
                cache
                    .get("steam-store", *id, Some(CACHE_MAX_AGE_SECS))
                    .is_none()
            })
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
                .filter(|id| {
                    cache
                        .get("steam-tags", *id, Some(CACHE_MAX_AGE_SECS))
                        .is_none()
                })
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

/// Ensure metadata is populated for a list of games (both Steam and Non-Steam shortcuts).
pub async fn ensure_game_metadata(
    state: &AppState,
    games: &[&Game],
    want_tags: bool,
) -> MetadataSyncResult {
    let settings = state.settings_snapshot();
    let client = metadata::http_client();

    let steam_ids: Vec<u32> = games
        .iter()
        .filter(|game| game.kind == GameKind::Steam && game.app_id != 0)
        .map(|game| game.app_id)
        .collect();

    let mut result = ensure_steam_metadata(state, &steam_ids, want_tags).await;

    // For any Steam games where store details were unavailable, try PCGamingWiki fallback
    let missing_steam: Vec<(String, u32)> = {
        let cache = state.cache.lock().expect("cache lock poisoned");
        games
            .iter()
            .filter(|g| {
                g.kind == GameKind::Steam
                    && cache
                        .get("steam-store", g.app_id, Some(CACHE_MAX_AGE_SECS))
                        .is_none()
            })
            .map(|g| (g.name.clone(), g.app_id))
            .collect()
    };

    for (name, app_id) in missing_steam {
        if let Ok(Some(meta)) = metadata::extra::pcgamingwiki_lookup(&client, &name).await {
            let mut cache = state.cache.lock().expect("cache lock poisoned");
            cache.put("steam-store", app_id, &meta);
            let _ = cache.save();
            result.fetched += 1;
        }
    }

    // Resolve non-Steam shortcuts across multiple providers
    for game in games {
        if game.kind != GameKind::Shortcut {
            continue;
        }

        let is_cached = {
            let cache = state.cache.lock().expect("cache lock poisoned");
            cache
                .get("shortcut-meta", game.app_id, Some(CACHE_MAX_AGE_SECS))
                .is_some()
        };

        if is_cached {
            result.cached += 1;
            continue;
        }

        result.requested += 1;
        let mut resolved: Option<AppMeta> = None;

        // 1. Local engine inspection (itch receipt, RPG Maker, Ren'Py, PE VersionInfo)
        let dir = game.install_dir.as_deref().map(Path::new);
        let exe = game.exe.as_deref().map(Path::new);
        if let Some(local_meta) = metadata::local::inspect_local_game(dir, exe) {
            resolved = Some(local_meta);
        }

        // 2. Steam Store Search (check if title exists on Steam Store)
        if resolved
            .as_ref()
            .map_or(true, |m| m.developers.is_empty() || m.publishers.is_empty())
        {
            if let Ok(Some(steam_id)) = metadata::steam_search::find_steam_app_id(
                &client,
                &game.name,
                &settings.country_code,
                &settings.language,
            )
            .await
            {
                if let Ok(map) = metadata::steam_store::fetch_app_details(
                    &client,
                    &[steam_id],
                    &settings.country_code,
                    &settings.language,
                )
                .await
                {
                    if let Some(store_meta) = map.get(&steam_id) {
                        resolved = Some(store_meta.clone());
                    }
                }
            }
        }

        // 3. VNDB lookup (visual novels, Ren'Py, anime narrative)
        if resolved
            .as_ref()
            .map_or(true, |m| m.developers.is_empty() || m.genres.is_empty())
        {
            if let Ok(Some(vndb_meta)) = metadata::vndb::vndb_lookup(&client, &game.name).await {
                if let Some(ref mut cur) = resolved {
                    if cur.developers.is_empty() {
                        cur.developers = vndb_meta.developers;
                    }
                    if cur.publishers.is_empty() {
                        cur.publishers = vndb_meta.publishers;
                    }
                    if cur.release_year.is_none() {
                        cur.release_year = vndb_meta.release_year;
                    }
                    for g in vndb_meta.genres {
                        if !cur.genres.contains(&g) {
                            cur.genres.push(g);
                        }
                    }
                } else {
                    resolved = Some(vndb_meta);
                }
            }
        }

        // 4. PCGamingWiki lookup
        if resolved
            .as_ref()
            .map_or(true, |m| m.developers.is_empty() || m.publishers.is_empty())
        {
            if let Ok(Some(wiki_meta)) =
                metadata::extra::pcgamingwiki_lookup(&client, &game.name).await
            {
                if let Some(ref mut cur) = resolved {
                    if cur.developers.is_empty() {
                        cur.developers = wiki_meta.developers;
                    }
                    if cur.publishers.is_empty() {
                        cur.publishers = wiki_meta.publishers;
                    }
                    if cur.release_year.is_none() {
                        cur.release_year = wiki_meta.release_year;
                    }
                    for g in wiki_meta.genres {
                        if !cur.genres.contains(&g) {
                            cur.genres.push(g);
                        }
                    }
                } else {
                    resolved = Some(wiki_meta);
                }
            }
        }

        // 5. Fallback publisher attribution from platform if still empty
        let mut final_meta = resolved.unwrap_or_else(|| AppMeta {
            name: Some(game.name.clone()),
            ..Default::default()
        });

        if final_meta.publishers.is_empty()
            && game.platform != Platform::Steam
            && game.platform != Platform::Other
        {
            final_meta.publishers = vec![game.platform.label().to_string()];
        }
        if final_meta.developers.is_empty() && !final_meta.publishers.is_empty() {
            final_meta.developers = final_meta.publishers.clone();
        }

        final_meta.app_id = game.app_id;
        final_meta.fetched_at = Some(chrono::Utc::now().timestamp());

        let mut cache = state.cache.lock().expect("cache lock poisoned");
        cache.put("shortcut-meta", game.app_id, &final_meta);
        cache.put("steam-store", game.app_id, &final_meta);
        let _ = cache.save();
        result.fetched += 1;
    }

    result
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
    let refs: Vec<&Game> = games.iter().collect();
    Ok(ensure_game_metadata(&state, &refs, false).await)
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

#[tauri::command]
pub async fn vndb_lookup(title: String) -> AppResult<Option<AppMeta>> {
    let client = metadata::http_client();
    metadata::vndb::vndb_lookup(&client, &title).await
}

#[tauri::command]
pub async fn steam_store_search(
    term: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<metadata::steam_search::SteamSearchResult>> {
    let client = metadata::http_client();
    let settings = state.settings_snapshot();
    metadata::steam_search::search_steam_store(
        &client,
        &term,
        &settings.country_code,
        &settings.language,
    )
    .await
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

    // Only reach out to metadata providers when at least one enabled rule needs it.
    let wants_metadata = autos
        .iter()
        .any(|autocat| autocat.enabled && autocat.kind.needs_metadata());
    let wants_tags = autos
        .iter()
        .any(|autocat| autocat.enabled && autocat.kind.needs_tags());
    if wants_metadata || wants_tags {
        let refs: Vec<&Game> = games.iter().collect();
        ensure_game_metadata(&state, &refs, wants_tags).await;
    }

    let cache = state.cache.lock().expect("cache lock poisoned");
    let mut buckets: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    let mut missing_metadata = Vec::new();

    for game in &games {
        let meta = cache
            .get("shortcut-meta", game.app_id, None)
            .or_else(|| cache.get("steam-store", game.app_id, None));
        if meta.is_none() && wants_metadata {
            missing_metadata.push(game.app_id);
        }
        let mut meta = meta.unwrap_or_default();
        if wants_tags {
            if let Some(tags) = cache.get("steam-tags", game.app_id, None) {
                meta.tags = tags.tags;
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
pub fn detect_platforms(
    include_other_drives: Option<bool>,
    state: State<'_, AppState>,
) -> PlatformScan {
    let settings = state.settings_snapshot();
    platforms::scan_all(&platforms::ScanConfig {
        sources: settings.platform_sources,
        include_other_drives: include_other_drives.unwrap_or(false),
    })
}

/// Existing library folders for one platform, used to autofill Settings.
///
/// Mirrors the "Auto-detect" behaviour of the Steam folder field: the frontend
/// asks for a platform's known folders and offers them as extra scan roots.
#[tauri::command]
pub fn find_platform_folders(platform: String) -> AppResult<Vec<String>> {
    let platform = Platform::from_id(&platform)
        .ok_or_else(|| AppError::Invalid(format!("unknown platform '{platform}'")))?;
    Ok(platforms::library_folders(platform))
}

/// Automatically fetch and apply missing artwork for a game (via SteamGridDB, Steam Store, VNDB, etc.).
pub async fn fetch_and_apply_artwork_for_game(
    state: &AppState,
    root: &Path,
    id3: &str,
    app_id: u32,
    name: &str,
) -> usize {
    let settings = state.settings_snapshot();
    let client = metadata::http_client();
    let grid_dir = artwork::grid_dir(root, id3);
    let mut applied_count = 0;

    let portrait_missing =
        !artwork::inspect_slot(&grid_dir, app_id, ArtworkSlot::GridPortrait).present;
    let hero_missing = !artwork::inspect_slot(&grid_dir, app_id, ArtworkSlot::Hero).present;
    let logo_missing = !artwork::inspect_slot(&grid_dir, app_id, ArtworkSlot::Logo).present;
    let icon_missing = !artwork::inspect_slot(&grid_dir, app_id, ArtworkSlot::Icon).present;

    if !portrait_missing && !hero_missing && !logo_missing && !icon_missing {
        return 0;
    }

    // 1. SteamGridDB resolution if API key is configured
    if let Some(ref key) = settings.steamgriddb_api_key.filter(|k| !k.is_empty()) {
        let slots_to_fetch = [
            (portrait_missing, ArtworkSlot::GridPortrait, "grids"),
            (hero_missing, ArtworkSlot::Hero, "heroes"),
            (logo_missing, ArtworkSlot::Logo, "logos"),
            (icon_missing, ArtworkSlot::Icon, "icons"),
        ];

        for (missing, slot, kind) in slots_to_fetch {
            if !missing {
                continue;
            }
            if let Ok(assets) =
                metadata::steamgriddb::for_game(&client, key, app_id, name, kind).await
            {
                if let Some(top_asset) = assets.first() {
                    if let Ok(resp) = client.get(&top_asset.url).send().await {
                        if resp.status().is_success() {
                            if let Ok(bytes) = resp.bytes().await {
                                if artwork::apply_bytes(
                                    &grid_dir,
                                    app_id,
                                    slot,
                                    &bytes,
                                    Some(top_asset.url.clone()),
                                )
                                .is_ok()
                                {
                                    applied_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Fallback: if portrait grid is still missing, check cached AppMeta (from Steam Store, VNDB, or itch)
    if !artwork::inspect_slot(&grid_dir, app_id, ArtworkSlot::GridPortrait).present {
        let img_url = {
            let cache = state.cache.lock().expect("cache lock poisoned");
            cache
                .get("shortcut-meta", app_id, None)
                .or_else(|| cache.get("steam-store", app_id, None))
                .and_then(|m| m.capsule_image.or(m.header_image))
        };

        if let Some(url) = img_url {
            if let Ok(resp) = client.get(&url).send().await {
                if resp.status().is_success() {
                    if let Ok(bytes) = resp.bytes().await {
                        if artwork::apply_bytes(
                            &grid_dir,
                            app_id,
                            ArtworkSlot::GridPortrait,
                            &bytes,
                            Some(url),
                        )
                        .is_ok()
                        {
                            applied_count += 1;
                        }
                    }
                }
            }
        }
    }

    applied_count
}

/// Import selected platform games into Steam as non-Steam shortcuts.
#[tauri::command]
pub async fn import_platform_games(
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
            launch_options: game.launch_uri.clone().unwrap_or_default(),
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

        // Automatically fetch and apply artwork for newly imported shortcuts
        for added_game in &outcome.added {
            let _ = fetch_and_apply_artwork_for_game(
                &state,
                &root,
                &id3,
                added_game.app_id,
                &added_game.name,
            )
            .await;
        }
    }
    skipped.extend(outcome.skipped);

    Ok(ImportResult {
        added: outcome.added,
        skipped,
    })
}

/// Set or clear the platform attribution for a game (usually a non-Steam shortcut).
#[tauri::command]
pub fn set_game_platform(app_id: u32, platform: Option<String>) -> AppResult<()> {
    let target = match platform {
        Some(ref id) if !id.is_empty() && id != "none" => {
            let p = Platform::from_id(id)
                .ok_or_else(|| AppError::Invalid(format!("unknown platform '{id}'")))?;
            Some(p)
        }
        _ => None,
    };
    platforms::set_override(app_id, target)
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

    // Pull in any missing store/provider metadata before computing, so metadata-based
    // kinds (genre, developer, publisher, release year, …) have data to work with,
    // and community tags when the Tag kind was chosen.
    let wants_metadata = request.kind.needs_metadata();
    let wants_tags = request.kind.needs_tags();
    if request.fetch_metadata && (wants_metadata || wants_tags) {
        ensure_game_metadata(&state, &scoped, wants_tags).await;
    }

    let autocat = build_autocat(&request);
    let cache = state.cache.lock().expect("cache lock poisoned");

    let mut buckets: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    let mut missing_metadata = Vec::new();
    let mut categorized = 0usize;

    for game in &scoped {
        let stored = cache
            .get("shortcut-meta", game.app_id, None)
            .or_else(|| cache.get("steam-store", game.app_id, None));
        if stored.is_none() && wants_metadata {
            missing_metadata.push(game.app_id);
        }
        let mut meta = stored.unwrap_or_default();
        if wants_tags {
            if let Some(tags) = cache.get("steam-tags", game.app_id, None) {
                meta.tags = tags.tags;
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
                        filter_spec: None,
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
            collection.removed.retain(|app_id| !target.contains(app_id));
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

// ---------------------------------------------------------------------------
// Custom artwork
// ---------------------------------------------------------------------------

/// Resolve the Steam root and the active account.
fn artwork_context(state: &AppState, account_id3: Option<String>) -> AppResult<(PathBuf, String)> {
    let root = steam_root(state)?;
    let account = resolve_account(&root, state, account_id3).ok_or_else(|| {
        AppError::NotFound("No Steam account found. Sign in to Steam at least once.".into())
    })?;
    Ok((root, account))
}

/// Inspect the custom artwork of every library entry (optionally limited to a
/// set of app ids) in one round-trip.
#[tauri::command]
pub fn get_artwork(
    app_ids: Option<Vec<u32>>,
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<Vec<GameArtwork>> {
    let (root, account) = artwork_context(&state, account_id3)?;
    let mut games = collect_library(&root, Some(&account))?;
    if let Some(filter) = app_ids {
        let wanted: BTreeSet<u32> = filter.into_iter().collect();
        games.retain(|game| wanted.contains(&game.app_id));
    }
    let entries: Vec<(u32, String)> = games
        .into_iter()
        .map(|game| (game.app_id, game.name))
        .collect();
    Ok(artwork::inspect(
        &artwork::grid_dir(&root, &account),
        &entries,
    ))
}

/// Read one slot's applied image as a `data:` URL (for previews).
#[tauri::command]
pub fn read_artwork(
    app_id: u32,
    slot: String,
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<Option<String>> {
    let (root, account) = artwork_context(&state, account_id3)?;
    let slot = ArtworkSlot::parse(&slot)?;
    artwork::read_data_url(&artwork::grid_dir(&root, &account), app_id, slot)
}

/// Download an image from a URL and apply it as a slot's custom artwork.
#[tauri::command]
pub async fn apply_artwork_from_url(
    app_id: u32,
    slot: String,
    url: String,
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<ArtworkOutcome> {
    let (root, account) = artwork_context(&state, account_id3)?;
    let slot = ArtworkSlot::parse(&slot)?;

    let client = metadata::http_client();
    let response = client.get(&url).send().await?;
    if !response.status().is_success() {
        return Err(AppError::msg(format!(
            "failed to download artwork (HTTP {})",
            response.status()
        )));
    }
    let bytes = response.bytes().await?;
    artwork::apply_bytes(
        &artwork::grid_dir(&root, &account),
        app_id,
        slot,
        &bytes,
        Some(url),
    )
}

/// Apply a local image file as a slot's custom artwork.
#[tauri::command]
pub fn apply_artwork_from_file(
    app_id: u32,
    slot: String,
    path: String,
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<ArtworkOutcome> {
    let (root, account) = artwork_context(&state, account_id3)?;
    let slot = ArtworkSlot::parse(&slot)?;

    let source = PathBuf::from(path.trim().trim_matches('"'));
    if !source.is_file() {
        return Err(AppError::NotFound(format!(
            "{} is not a file",
            source.display()
        )));
    }
    let bytes = std::fs::read(&source)?;
    artwork::apply_bytes(
        &artwork::grid_dir(&root, &account),
        app_id,
        slot,
        &bytes,
        Some(source.to_string_lossy().to_string()),
    )
}

/// Remove a slot's custom artwork (after archiving it).
#[tauri::command]
pub fn remove_artwork(
    app_id: u32,
    slot: String,
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<ArtworkOutcome> {
    let (root, account) = artwork_context(&state, account_id3)?;
    let slot = ArtworkSlot::parse(&slot)?;
    artwork::remove(&artwork::grid_dir(&root, &account), app_id, slot)
}

#[tauri::command]
pub async fn steamgriddb_for_game(
    app_id: u32,
    name: String,
    kind: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<metadata::steamgriddb::SgdbAsset>> {
    let key = require_key(&state, "steamgriddb")?;
    let client = metadata::http_client();
    metadata::steamgriddb::for_game(&client, &key, app_id, &name, &kind).await
}

/// Automatically fetch missing custom artwork for shortcuts or specified games.
#[tauri::command]
pub async fn auto_fetch_artwork_for_games(
    app_ids: Option<Vec<u32>>,
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<usize> {
    let (root, account) = artwork_context(&state, account_id3)?;
    let games = collect_library(&root, Some(&account))?;

    let targets: Vec<&Game> = if let Some(ref ids) = app_ids {
        games.iter().filter(|g| ids.contains(&g.app_id)).collect()
    } else {
        // By default target all non-Steam shortcuts
        games
            .iter()
            .filter(|g| g.kind == GameKind::Shortcut)
            .collect()
    };

    let mut applied_total = 0;
    for game in targets {
        applied_total +=
            fetch_and_apply_artwork_for_game(&state, &root, &account, game.app_id, &game.name)
                .await;
    }

    Ok(applied_total)
}

// ---------------------------------------------------------------------------
// Dynamic (Filter-Spec) & Smart Collections
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_filter_presets() -> Vec<crate::steam::filters::FilterPreset> {
    crate::steam::filters::load_presets()
}

#[tauri::command]
pub fn save_filter_preset(
    preset: crate::steam::filters::FilterPreset,
) -> AppResult<crate::steam::filters::FilterPreset> {
    crate::steam::filters::save_preset(preset)
}

#[tauri::command]
pub fn delete_filter_preset(id: String) -> AppResult<()> {
    crate::steam::filters::delete_preset(&id)
}

#[tauri::command]
pub fn evaluate_filter(
    filter: Filter,
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<Vec<u32>> {
    let root = steam_root(&state)?;
    let account = resolve_account(&root, &state, account_id3);
    let games = collect_library(&root, account.as_deref())?;
    Ok(crate::steam::filters::evaluate_filter(&filter, &games))
}

#[tauri::command]
pub fn save_smart_collection(
    name: String,
    filter: Filter,
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<Collection> {
    ensure_steam_closed()?;
    let root = steam_root(&state)?;
    let id3 = resolve_account(&root, &state, account_id3)
        .ok_or_else(|| AppError::NotFound("No Steam account selected.".into()))?;

    let mut file = collections::load_collections_file(&root, &id3)?.ok_or_else(|| {
        AppError::NotFound("No Steam collection store found for this account.".into())
    })?;

    let games = collect_library(&root, Some(&id3))?;
    crate::steam::filters::save_smart_collection(&mut file, &name, &filter, &games)
}

#[tauri::command]
pub fn refresh_smart_collection(
    collection_id: String,
    account_id3: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<Collection> {
    ensure_steam_closed()?;
    let root = steam_root(&state)?;
    let id3 = resolve_account(&root, &state, account_id3)
        .ok_or_else(|| AppError::NotFound("No Steam account selected.".into()))?;

    let mut file = collections::load_collections_file(&root, &id3)?.ok_or_else(|| {
        AppError::NotFound("No Steam collection store found for this account.".into())
    })?;

    let games = collect_library(&root, Some(&id3))?;
    crate::steam::filters::refresh_smart_collection(&mut file, &collection_id, &games)
}
