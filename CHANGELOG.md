# Changelog

All notable changes to Steam Vault are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.1] - 2026-10-15

### Fixed

- **Online Catalog Sync JSON Sequence Deserialization**:
  - Resolved `JSON error: invalid type: sequence, expected a string at line 1 column 79` when synchronizing online community catalog archives (`sync_online_catalog`).
  - Added untagged `StringOrSeq` deserializer in `cache.rs` supporting both raw stringified JSON and JSON sequence arrays for `platforms` and `known_exes`.
  - Added `Option<serde_json::Value>` support for `meta_json` and nested `meta` objects in `CatalogExportRecord`.
  - Recomputed normalized titles and lookups (`alias` and `exe`) during catalog ingestion to ensure exact title and alias matching in `find_catalog_by_title`.
- **`AppMeta` Resilient Metadata Deserialization**:
  - Added serde aliases (`app_id`, `release_date`, `release_year`, `user_score`, `short_description`, `header_image`, `capsule_image`, `fetched_at`) and defaults to `AppMeta` in `models.rs` ensuring seamless deserialization across both `camelCase` and `snake_case` formats.
- **Embedded Catalog Seed Synchronization**:
  - Synchronized precompiled offline catalog seed (`catalog_seed.bin`) with the latest catalog build.

## [1.1.0] - 2026-10-15

### Added

- **Integrated Community Game Catalogue (`Steam-Vault-Catalogue`)**:
  - Precompiled embedded SQLite catalog seed (`src-tauri/src/catalog_seed.bin`) with curated multi-platform games for offline title and metadata lookup.
  - Online catalog synchronization (`sync_online_catalog`) downloading compressed catalog databases (`catalog.db.gz`) from GitHub Releases or custom URLs.
  - Catalog backup and restoration (`export_catalog_archive`, `import_catalog_archive`) enabling offline portability and custom database overrides.
  - Live catalog entry lookup (`fetch_online_game_entry`) and catalog statistics reporting (`get_catalog_stats`).
- **Console Platform Determination & Categorization**:
  - High-precision console detection (`determine_console_platform`) identifying specific console hardware (GameCube, Wii, Wii U, Nintendo Switch, N64, SNES, NES, GBA, GBC, GB, NDS, 3DS, PS1, PS2, PS3, PSP, Genesis, Saturn, Dreamcast, Atari 2600) from ROM extensions and emulator paths.
  - Console badges displayed alongside emulator names in the Platforms view.
  - Automatic shortcut tagging: console platforms written directly into binary `shortcuts.vdf` `tags` so Steam client natively categorizes emulated games under their console name, and Steam Vault ingests them into collections on scan.
- **Wikipedia Video Game & Console Metadata Provider**:
  - Keyless MediaWiki wikitext infobox parser (`wikipedia_game_lookup`) extracting developers, publishers, genres, release year, console platforms, and lead summaries.
  - Fallback metadata resolver in `ensure_game_metadata` for console ROM shortcuts lacking PC storefront listings.
  - Strongly-typed Tauri command `wikipedia_game_lookup` exposed to frontend.
- **Multi-Platform OS & Hardware Taxonomy**:
  - Catalog records and platform metadata support multi-platform operating systems (`windows`, `macos`, `linux`) and distinct console hardware, strictly separating software categories from platforms.

### Changed

- Updated binary VDF parser (`Bvdf`) with string accessor `as_str()` for tag dictionary inspection.
- Improved non-Steam shortcut collection assignment during import and scan.

## [1.0.0] - 2026-10-15

### Added

- **M5 Dark Glassmorphism Design System**: Complete modern obsidian glass visual overhaul across every view (`app.css`, Titlebar, Dashboard, Library, Organize, Collections, Platforms, Storage, Artwork, Settings) featuring multi-layer frosted surfaces, refined glow borders, tactile buttons, and interactive cards.
- **Embedded SQLite metadata cache**: High-performance metadata storage powered by `rusqlite` bundled from source (zero system dependencies) in WAL mode, with indexing on `(provider, app_id)`, TTL expiration checking, and automatic zero-touch migration from the legacy JSON metadata cache.
- **Dynamic (filter-spec) smart collections**: Persist named filter presets (`filter-presets.json`), evaluate dynamic library rules, and save native Valve `filterSpec` smart collections directly into `cloud-storage-namespace-1.json` with live "Smart Collection" badges and one-click re-evaluation.
- **Hand-rolled i18n localization engine**: Zero-dependency reactive internationalization engine supporting all 30 officially supported Valve/Steam platform languages with parameter interpolation, English fallback safety, RTL layout support for Arabic, and a 30-language selector in Settings.
- **Keyless VNDB metadata provider** (`metadata::vndb`): Kana API client fetching visual novel metadata (developers, publishers, release year, tags, and cover art) for visual novels and Ren'Py games.
- **Steam Store Search provider** (`metadata::steam_search`): Keyless search mapping non-Steam and indie titles to official Steam App IDs to fetch store metadata.
- **Local game & engine sniffer** (`metadata::local`): Offline metadata extraction from itch.io `receipt.json`, Ren'Py signatures, RPG Maker manifests (`System.json`, `System.rvdata2`, `Scripts.rxdata`), and Windows PE binary VersionInfo headers (`CompanyName`, `ProductName`).
- **Emulators and ROMs support** (`Platform::Emulator`): Automatic detection of 11 popular emulators (Dolphin, PCSX2, DuckStation, Project64, PPSSPP, RPCS3, Citra, Cemu, Ryujinx, mGBA, Snes9x, RetroArch), recursive ROM scanning, and automated CLI launch arguments.
- **Automatic artwork resolution**: Non-Steam imports automatically download portrait, hero, logo, and icon art via SteamGridDB and metadata caches; new **Auto-fetch missing art** button in Artwork view.
- **Multi-entity string splitting & platform fallbacks**: AutoCats splits comma/semicolon/slash-separated developer and publisher lists into discrete collections while preserving legal entity names (`Co., Ltd.`), with fallback to platform storefront for unlisted shortcuts.
- **High-performance platform scanner & ROM matching**: Replaced slow drive letter probing with instantaneous `sysinfo::Disks` mounted volume enumeration; pre-located emulator executables with $O(1)$ in-memory ROM extension resolution; added top-level executable fast paths and opt-in "Scan all drives" toggle in Platforms view.

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
  guard). New **Storage** view (navigation grows to seven items) with a drive overview, per-library cards, a
  searchable game list, move previews, and live `storage-progress` events.
- **Artwork management** (`get_artwork`, `read_artwork`, `apply_artwork_from_url`,
  `apply_artwork_from_file`, `remove_artwork`, `steamgriddb_for_game`): inspect and edit Steam's five
  custom-artwork slots (portrait grid, landscape grid, hero, logo, icon) in
  `userdata/<id3>/config/grid/`, for Steam games and non-Steam shortcuts alike. SteamGridDB assets
  are resolved by **exact app id** for real games and by **name** for shortcuts; local PNG/JPEG files
  can be applied directly, and the image format is sniffed so the correct extension is written.
  Replacing or removing art backs up the previous image first. New **Artwork** view (eight-item
  navigation). Unlike collection and shortcut writes, artwork writes do **not** require Steam to be
  closed.
- **Settings non-Steam paths refinement**: replaced the semicolon-joined text field
  with individual folder inputs with **+** add and **×** remove buttons, plus a per-platform
  **Find folders** button (`find_platform_folders`) that auto-detects launcher config and
  library locations mirroring Steam's auto-detect UX.
- **Storefront launch URIs**: `PlatformGame.launch_uri` populated for Epic Games Launcher
  (`com.epicgames.launcher://apps/<id>?action=launch`), Legendary / Heroic (`legendary://<id>`),
  and itch.io (`itch://games/<id>`), and written into the non-Steam shortcut's launch options
  on import.
- **Per-game platform override & re-tagging**: new `set_game_platform` command persisting
  custom platform attributions for shortcuts, with interactive dropdown pills on each shortcut
  row and bulk re-tagging in the Library toolbar.
- **Microsoft Store / Xbox Game Pass detection**: manifest-based detector scanning `XboxGames`
  directories across mounted drives, reading `MicrosoftGame.config` and `appxmanifest.xml`
  for game titles, executables, package identities, and `shell:AppsFolder` launch URIs without scraping.
- **Automated log file cleanup**: `scripts/lib/logger.mjs` prunes stale log files across runs,
  maintaining a configurable retention cap (default 25 most recent `.log` files) to protect disk storage.

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
