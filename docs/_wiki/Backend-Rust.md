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
| `steam/shortcuts.rs` | Reads/writes non-Steam shortcuts. |
| `steam/models.rs` | Shared serde models (serialized as camelCase). |
| `metadata/` | Provider clients + a shared HTTP client and registry. |
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
