---
layout: wiki
title: Backend (Rust)
summary: The Tauri command surface, modules, and shared state
parent: Steam Vault Technical Wiki
nav_order: 4
---

# Backend (Rust)

The backend is a Rust crate at `src-tauri/` exposing a typed command surface to the frontend. All
Steam access, metadata fetching, categorization, and file writing happen here.

## 🧩 Modules

| Path | Role |
| :--- | :--- |
| `error.rs` | `AppError` / `AppResult` — a single serializable error type for IPC. |
| `steam/mod.rs` | The Steam data-layer facade. |
| `steam/vdf.rs`, `steam/binvdf.rs` | Text and binary KeyValues parsers (with tests). |
| `steam/paths.rs` | Steam discovery: root, libraries, accounts. |
| `steam/process.rs` | Detects whether Steam is running. |
| `steam/library.rs` | Aggregates games from manifests, library folders, and play stats. |
| `steam/collections.rs` | Reads/writes the cloud-storage collection namespace. |
| `steam/shortcuts.rs` | Reads/writes non-Steam shortcuts, derives Steam-compatible appids, and appends imports. |
| `steam/owned.rs` | Caches the full owned Steam library (Web API `GetOwnedGames`). |
| `steam/models.rs` | Shared serde models (serialized as camelCase). |
| `metadata/` | Provider clients + a shared HTTP client and registry. |
| `platforms/` | External-launcher detection (GOG, Epic, itch.io, …) and the appid → platform map. |
| `platforms/registry.rs` | Windows Uninstall-registry and GOG registry reads (Windows only). |
| `autocat.rs` | The AutoCats rule engine. |
| `filter.rs` | The Allowed / Required / Excluded model. |
| `cache.rs` | JSON metadata cache with a freshness window. |
| `settings.rs` | Local settings and API keys. |
| `backup.rs` | Timestamped file backups. |
| `commands.rs` | Tauri commands and the shared `AppState`. |
| `lib.rs` | Crate root; registers plugins, state, and the command handler. |

## 🖥️ Command surface

The frontend calls these through `invoke`. Arguments are camelCase in JavaScript and map to
snake_case Rust parameters.

| Command | Purpose |
| :--- | :--- |
| `get_providers` | List metadata providers and whether they need a key. |
| `detect_steam` | Locate Steam and read its libraries and accounts. |
| `get_settings` / `save_settings` | Read and persist settings. |
| `is_steam_running` | Report whether Steam is running (write guard). |
| `load_library` | Aggregate the full library with collection membership. |
| `load_collections` | Read collections for an account. |
| `update_collection` | Add/remove games and/or rename a collection. |
| `create_collection` / `delete_collection` | Manage collections. |
| `fetch_steam_store_metadata` | Fetch/cache Steam Store metadata for app ids. |
| `fetch_owned_games` | Steam Web API owned games (keyed). |
| `steamgriddb_search` / `steamgriddb_assets` | SteamGridDB search and artwork (keyed). |
| `igdb_search` | IGDB search (keyed). |
| `pcgamingwiki_lookup` / `wikipedia_summary` | Keyless supplemental providers. |
| `compute_autocats` | Build an AutoCat plan without writing. |
| `detect_platforms` | Scans the machine for games installed by other launchers. |
| `import_platform_games` | Writes selected platform games to `shortcuts.vdf` as shortcuts. |
| `get_owned_library` / `refresh_owned_games` | Returns / fetches the full owned Steam library. |
| `compute_categorization` / `apply_categorization` | Previews and writes collections from a sorting option. |

## 🎮 External platforms & the owned library

`platforms/` discovers games installed by other launchers from local sources only — launcher
manifests, the GOG registry, the itch app library folder, and the Windows Uninstall registry.
Selected games are written to `shortcuts.vdf` with Steam-compatible appids
(`crc32(exe + name) | 0x80000000`), and the appid → platform map is persisted so imported shortcuts
stay attributable to their storefront.

Because most of a large Steam library is not installed locally, `steam/owned.rs` caches the Web API's
`GetOwnedGames` result and `load_library` merges it into the game list. Owned-but-not-installed games
are first-class candidates for categorization and collections.

## 🗃️ Shared state

`AppState` holds the loaded `Settings` and the `MetadataCache`, each behind a mutex. Long-running
commands snapshot what they need and drop the lock before awaiting network calls, so IPC stays
responsive.

## 🧯 Errors

Fallible commands return `Result<T, AppError>`. `AppError` serializes to a readable message the
frontend shows in its error banner, and it distinguishes cases such as "Steam is running" and
"not found" so the UI can respond appropriately.

## ➡️ Next

- [Steam Data Layer](Steam-Data-Layer.md)
- [Frontend (SvelteKit)](Frontend-SvelteKit.md)
