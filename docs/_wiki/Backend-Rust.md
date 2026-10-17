---
layout: wiki
title: Backend (Rust)
summary: The Tauri v2 command surface, internal plugin engine, metadata modules, and persistence architecture
parent: Technical Wiki
nav_order: 4
---

# Backend (Rust)

The Universal Game Launcher backend is a Rust codebase at `src-tauri/` exposing a strongly typed command surface to the SvelteKit frontend via Tauri v2 IPC handlers and providing a headless CLI interface (`--cli`).

The core backend owns the application lifecycle, local SQLite persistence (`database.sqlite`), in-memory KV caching (`mem_kv`), storage footprint analytics, client launch delegation protocols, title normalization, and the automatic multi-source metadata cascade.

---

## 🧩 Backend Modules

| Path | Role |
| :--- | :--- |
| `src-tauri/src/metadata/universal.rs` | Automatic multi-source metadata cascade querying Local Inspector, Libretro CDN, Steam Store, GOG Catalog v1, Wikipedia/Wikidata, and PCGamingWiki. |
| `src-tauri/src/metadata/local.rs` | Local folder inspector parsing RPG Maker `System.json` (`"gameTitle"`), itch.io receipt manifests, `rom.ini` configurations, and binary headers. |
| `src-tauri/src/metadata/steam_store.rs` | Steam Store API client parsing store details and modern Akamai CDN MP4 video trailers (`movie_max.mp4`). |
| `src-tauri/src/plugins/title_normalizer.rs` | Dedicated Rust title normalization engine (`normalize_game_title`, `normalize_rom_title`, `canonical_title_key`) handling article inversion, unified subtitle separators, and slug separation. |
| `src-tauri/src/plugins/engine.rs` | Platform discovery and launch specification engine. |
| `src-tauri/src/cli.rs` | Headless CLI interface (`--cli scan`, `scan-roms`, `scaffold-roms`, `normalize-title`, `inspect-local`, `launchers`). |
| `src-tauri/src/cache.rs` | Embedded SQLite persistence layer (`database.sqlite`) with WAL mode, prepared statements, and the high-speed in-memory KV tier (`mem_kv`). |
| `src-tauri/src/settings.rs` | Application preferences, discovery roots, and plugin settings persistence. |
| `src-tauri/src/commands.rs` | Tauri IPC commands, asynchronous scan triggers, and shared `AppState`. |
| `src-tauri/src/lib.rs` | Tauri application builder, state mounting, and command registration. |
| `src-tauri/src/platforms/` | Platform detectors, launch protocol builders (`steam://`, `com.epicgames.launcher://`, `goggalaxy://`, native executables), and storage footprint discovery. |
| `src-tauri/src/platforms/windows.rs` | Windows-specific APIs (registry inspection, known folders, shortcut resolution). |
| `src-tauri/src/platforms/linux.rs` | Linux-specific APIs (XDG desktop entries, Flatpak manifests). |
| `src-tauri/src/platforms/macos.rs` | macOS-specific APIs (Application Support discovery, NSWorkspace helpers). |

---

## 🖥️ Primary Command Surface

| Command | Purpose |
| :--- | :--- |
| `load_library` | Loads all games with collection memberships and artwork paths via the sub-millisecond in-memory KV tier. |
| `get_cached_game_metadata` | Retrieves cached multi-source metadata (`game_meta:<appId>`) including developer, publisher, release date, genres, and modern video trailers. |
| `fetch_external_json` | Native Rust HTTP proxy executing secure external GET requests with custom User-Agent headers, avoiding CORS blocks in the frontend. |
| `load_collections` | Reads all custom and dynamic smart collections from local SQLite. |
| `update_collection` | Add/remove games or rename a collection, persisting directly to local SQLite. |
| `create_collection` / `delete_collection` | Creates or removes user collections safely. |
| `launch_game` | Executes official client launch protocols (`steam://`, `goggalaxy://`, etc.) or spawns standalone binaries. |
| `install_game` / `uninstall_game` | Delegates installation or uninstallation intent directly to official clients. |
| `open_game_folder` | Opens the local installation directory in the host system file manager. |
| `get_storage_report` | Calculates drive capacity and game disk footprints across all detected volumes via background workers. |
| `get_cache_stats` / `clear_metadata_cache` | Inspects and resets cached artwork mappings and temporary keys. |
| `detect_platforms` | Discovers games installed across official platform storefront clients. |
| `get_installed_plugins` | Returns all discovered plugins with active states and capabilities. |
| `toggle_plugin` | Enables or disables a plugin dynamically. |

---

## 🔒 Non-Invasive Safety & Performance
- **Zero Client Modification:** Never alters external client files (`shortcuts.vdf`, `cloud-storage-namespace-1.json`). Official clients can remain open and running freely.
- **Serverless In-Memory KV Tier (`mem_kv`):** Library queries and collection filters complete in under 1 millisecond.
- **Zero Credentials:** Launching and installation delegate entirely to the operating system protocol handlers or native binaries.

---

## ➡️ Next Steps

- [Frontend (SvelteKit)](Frontend-SvelteKit.md) — SvelteKit SPA and Svelte 5 Runes.
- [Architecture](Architecture.md) — overall system architecture.
- [Plugins Architecture](Plugins-Architecture.md) — Universal Core plugins framework.
