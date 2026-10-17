---
layout: wiki
title: Backend (Rust)
summary: The Tauri v2 command surface, internal plugin engine, modules, and persistence architecture
parent: Technical Wiki
nav_order: 4
---

# Backend (Rust)

The Universal Game Launcher backend is a Rust codebase at `src-tauri/` exposing a strongly typed command surface to the SvelteKit frontend via Tauri v2 IPC handlers.

The core backend owns the application lifecycle, local SQLite persistence (`database.sqlite`), in-memory KV caching (`mem_kv`), storage footprint analytics, client launch delegation protocols, and the internal plugin engine module (`src-tauri/src/plugins/`).

---

## 🧩 Backend Modules

| Path | Role |
| :--- | :--- |
| `src-tauri/src/plugins/` | Internal plugin engine module (`engine.rs`, `manifest.rs`, `defaults.rs`) managing platform scanning, launch protocol generation, and store CDN artwork resolution. |
| `src-tauri/src/cache.rs` | Embedded SQLite persistence layer (`database.sqlite`) with WAL mode, prepared statements, and the high-speed in-memory KV tier (`mem_kv`). |
| `src-tauri/src/settings.rs` | Application preferences, discovery roots, and plugin settings persistence. |
| `src-tauri/src/commands.rs` | Tauri IPC commands, asynchronous scan triggers, and shared `AppState`. |
| `src-tauri/src/lib.rs` | Tauri application builder, plugin engine initialization, state mounting, and command registration. |
| `src-tauri/src/platforms/` | Platform detectors, launch protocol builders (`steam://`, `com.epicgames.launcher://`, `goggalaxy://`, native executables), and storage footprint discovery. |
| `src-tauri/src/platforms/windows.rs` | Windows-specific APIs (registry inspection, known folders, shortcut resolution). |
| `src-tauri/src/platforms/linux.rs` | Linux-specific APIs (XDG desktop entries, Flatpak manifests). |
| `src-tauri/src/platforms/macos.rs` | macOS-specific APIs (Application Support discovery, NSWorkspace helpers). |

---

## 🖥️ Primary Command Surface

| Command | Purpose |
| :--- | :--- |
| `load_library` | Loads all games with collection memberships and artwork paths via the sub-millisecond in-memory KV tier. |
| `load_collections` | Reads all custom and dynamic smart collections from local SQLite. |
| `update_collection` | Add/remove games or rename a collection, persisting directly to local SQLite. |
| `create_collection` / `delete_collection` | Creates or removes user collections safely. |
| `launch_game` | Executes official client launch protocols (`steam://`, `goggalaxy://`, etc.) or spawns standalone binaries. |
| `install_game` / `uninstall_game` | Delegates installation or uninstallation intent directly to official clients. |
| `open_game_folder` | Opens the local installation directory in the host system file manager. |
| `get_storage_report` | Calculates drive capacity and game disk footprints across all detected volumes. |
| `get_cache_stats` / `clear_metadata_cache` | Inspects and resets cached artwork mappings and temporary keys. |
| `detect_platforms` | Discovers games installed across official platform storefront clients. |
| `get_installed_plugins` | Returns all discovered plugins with active states and capabilities. |
| `toggle_plugin` | Enables or disables a plugin dynamically. |

---

## 🔒 Non-Invasive Safety & Performance
- **Zero Client Modification:** Never alters external client files (`shortcuts.vdf`, `cloud-storage-namespace-1.json`). Official clients can remain open and running freely.
- **Serverless In-Memory KV Tier (`mem_kv`):** Library queries and collection filters complete in under 1 millisecond.
- **Zero Credentials:** Launching and installation delegate entirely to the operating system protocol handlers or native binaries.
