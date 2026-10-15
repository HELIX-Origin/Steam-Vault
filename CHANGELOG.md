# Changelog

All notable changes to Steam Vault are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial project scaffold: Tauri v2 + SvelteKit (Svelte 5 / TypeScript / Vite).
- Steam installation discovery (Windows registry + cross-platform fallbacks) with
  `libraryfolders.vdf` library-folder and `loginusers.vdf` account parsing.
- Text KeyValues (VDF) parser/serializer and binary KeyValues reader/writer for
  `shortcuts.vdf`.
- Library aggregation from `appmanifest_*.acf`, `libraryfolders.vdf`, `localconfig.vdf`
  play stats, and non-Steam shortcuts.
- Modern Steam collection read/write against `cloud-storage-namespace-1.json`
  (favorites, hidden, create, rename, delete) with timestamped backups and a
  Steam-must-be-closed guard.
- Metadata provider layer: Steam Store (`appdetails`), Steam Web API, SteamGridDB,
  IGDB (Twitch), PCGamingWiki, and Wikipedia.
- AutoCats engine (genre, developer, publisher, flag, tag, release year, user score)
  and an Allowed / Required / Excluded filter model.
- JSON-backed metadata cache and local settings store.
- Tauri command surface and a glassmorphism-themed Svelte app shell
  (Dashboard, Library, Collections, Settings).
- **Custom title bar** (`decorations: false`) with platform-aware window controls
  (macOS traffic lights, Windows-style buttons elsewhere) and a draggable app header.
- **Whole owned Steam library** via `IPlayerService/GetOwnedGames`, cached to
  `owned-games.json` and merged into the Library view so owned-but-not-installed games
  can be organized (`refresh_owned_games`, `get_owned_library`).
- **External platform detection** for GOG, Epic Games, itch.io, Amazon Games, Ubisoft
  Connect, EA, Battle.net, Rockstar, and other publishers (launcher manifests, the GOG
  registry, the itch app library, and the Windows Uninstall registry) — never scraping.
- **Import platform games as non-Steam shortcuts** (`import_platform_games`) with
  Depressurizer-compatible appids (`crc32(exe + name) | 0x80000000`), quoted
  `Exe` / `StartDir` fields, and de-duplication; a persisted appid → platform map keeps
  imported shortcuts attributable.
- **Create collections from any sorting option** (`compute_categorization` /
  `apply_categorization`): genre, developer, publisher, Steam feature, tag, release
  year, user score, **platform**, **installed state**, **playtime**, and **name**, with
  an optional prefix, an installed-only filter, and a replace-membership mode.
- New **Platforms** and **Organize** views plus platform and installed-state badges in the Library.
- **On-demand metadata cache warming** for the AutoCats / collection engine: the Steam Store
  provider now fetches store **tags** (`actions/ajaxgetstoretags`) alongside `appdetails`;
  `ensure_steam_metadata` de-duplicates ids, reuses entries within the cache TTL, and fetches
  `appdetails` per app id with bounded concurrency; a new `sync_library_metadata` command warms the cache for the
  whole owned library, and categorization can fetch on demand (`fetchMetadata`, on by default).
- **Logged command scripts** (`scripts/*.mjs`): `tauri:dev` / `tauri:build` and a generic
  `run.mjs` runner stream output live to the console while writing a timestamped transcript to
  `logs/` (`<type>-<YYYY-MM-DD_HH-MM-SS>.log`, so runs are never overwritten).
- **Agent-friendly test & verification runner** (`scripts/test.mjs` + `scripts/lib/suites.mjs`):
  named suites (`fmt`, `clippy`, `check`, `rust`, `frontend`, `build`, `all`) run the toolchain
  directly — no nested `npm` — with **stdin closed** (no step can block on input), a **per-step
  timeout** (default 15 min, `124` on timeout), a timestamped transcript per step, and a compact
  PASS/FAIL summary with a reliable exit code; wired to `npm test`, `test:rust`, `test:frontend`,
  `test:all`, and `verify`.
- **Filter builder in Organize**: Depressurizer-style Allowed / Required / Excluded
  collections, name search, and kind filter with live match count; filters are
  passed to categorization.
- **Non-Steam sources in Settings**: per-platform enable/disable and extra folders
  (semicolon-separated), plus all-drive enumeration so games on external drives
  are discovered.
- **Branding**: wired the Steam Vault logo into the title bar and replaced the
  favicon with the brand icon.
- **Storage management** (`get_storage_report`, `preview_move_library`, `move_library`,
  `preview_move_game`, `move_game`, `add_library_folder`, `remove_library_folder`): inspect
  library folders and mounted drives, register/unregister libraries, and move a game or a whole
  library between drives, updating every `libraryfolders.vdf` (with backups and a Steam-closed
  guard). New **Storage** view (seven-item navigation) with a drive overview, per-library cards, a
  searchable game list, move previews, and live `storage-progress` events.

### Fixed

- `shortcuts.vdf` now reads canonical field casing (`AppName` / `Exe`) and strips the
  literal quotes Steam stores around `Exe` / `StartDir`; the writer emits the canonical
  `Exe` / `StartDir` keys instead of lowercase ones.
- **Steam metadata fetch**: fixed HTTP 400 when `appids` were comma-joined (batching)
  — now fetches per-appid with bounded concurrency; `appdetails` and store tags are
  fetched robustly.
- **Platform serde**: `Platform` serializes with `lowercase` ids (e.g.
  `battlenet`, `ubisoft`, `ea`) to match the frontend union.
- **Platform scan**: honours per-platform settings, includes extra/custom folders,
  and enumerates mounted drives to detect games on external drives.
