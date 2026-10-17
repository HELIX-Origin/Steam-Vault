# Changelog

All notable changes to Universal Game Launcher are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.4.1] - 2026-10-17

### Added

- **Dynamic Federation Shard Discovery (`getActiveShards()`)**:
  - Implemented dynamic volume shard discovery in `src/lib/plugins/metadata/helix-db.ts` querying the `/api/v2/federation/shards.json` manifest.
  - Added multi-tier fallback architecture: in-memory 1-hour cache, resilient 7-day SQLite KV persistent storage (`helix_db_dynamic_shards`), and static baseline fallback (`00`, `01`, `02`, `03`).
  - Seamlessly discovers newly provisioned sequential metadata shards (`04`, `05`, etc.) in real time without requiring application updates.

### Changed

- **Multi-Shard Query Resolution**:
  - `getHelixDbSearchIndex` and `getHelixDbGame` now resolve shard lists dynamically via `getActiveShards()` instead of relying on a hardcoded array.

## [1.4.0] - 2026-10-17

### Added

- **Universal Game Launcher Rebranding & Multi-Platform Library**:
  - Official rebranding to **Universal Game Launcher** with dark obsidian aesthetic (`#07111f` / `#0d1b2d`) and cyan/purple accents.
  - Unified library management for Steam, GOG, Epic Games (via native protocol or Legendary CLI), itch.io, standalone executables, and retro emulator ROMs.
  - Standalone game addition modal and enhanced manual metadata editor.
- **Modular Universal Core Plugins Framework (`src/lib/plugins/`)**:
  - Re-architected native in-app modular framework with `platform-universal`, `metadata-universal`, `metadata-helix`, `utility-title-normalizer`, `media-universal`, `utility-deals`, `utility-patchnotes`, and `utility-storage`.
  - Non-blocking background worker execution for storage space scanning and library aggregation.
- **Authoritative HELIX Gaming DB Metadata Integration (`metadata-helix`)**:
  - Multi-shard volume federation support querying across `helix-gaming-db-00`, `01`, `02`, and `03`.
  - Consolidated multi-shard search index aggregation and deduplication with SQLite KV persistence.
  - Cloudflare Edge direct shard proxy support (`https://api.helix-origin.club/{shard}/api/...`) with automated failover to GitHub CDN.
  - Three-tier catalog segregation: `v1` legacy, `v2` family/general indie & retro, and `v3` adult 18+ titles.
- **Dynamic `i18next` Localization Engine**:
  - Replaced hand-rolled hardcoded dictionaries with dynamic `i18next` client library.
  - Remote edge bundle loading (`/api/v2/i18n/{code}/bundle.json`) for 12 primary languages (`en`, `ja`, `zh-CN`, `zh-TW`, `es`, `fr`, `de`, `it`, `pt-BR`, `ru`, `ko`, `pl`).
  - Dual UI (`app.json`) and metadata taxonomy (`genres`, `categories`, `platforms`, `common`) support in a single atomic payload.
  - Persistent SQLite KV caching (`i18n_bundle_{code}`) providing instant 0ms offline startup.
  - Dynamic RTL (Right-to-Left) layout support for Arabic and robust English fallback chains.
- **Automated Continuous Integration CI Workflow**:
  - Deployed `.github/workflows/ci.yml` verifying frontend type safety (`svelte-check`) and Rust backend compilation and unit tests (`cargo check`, `cargo test --lib`) on `windows-latest` and `ubuntu-22.04`.

### Changed

- **Centralized Documentation Decoupling**:
  - Migrated detailed technical guides and wikis to organization hubs (`https://docs.helix-origin.club/universal-game-launcher/` and `https://wiki.helix-origin.club/universal-game-launcher/`).
  - Streamlined repository `README.md` into a minimal, clean entry point.

### Security & Compliance

- **Organization Standards Compliance**:
  - Canonical BSD-3-Clause license harmonization across manifests and source headers.
  - Standardized all repository shields and badges to raw HTML `<a><img ... height="20" /></a>`.

## [1.3.3] - 2026-10-16

### Changed

- **Project Script Standards & Command Hierarchy Compliance**:
  - Aligned package automation commands in `package.json` to strictly follow the organizational `main:sub:option` hierarchy (`winget:manifest:generate`, `winget:manifest:verify`, `winget:manifest:install`).
  - Upgraded WinGet Package Manager manifest generator (`scripts/generate-winget-manifest.mjs`) to Microsoft Schema **1.12.0** (`ManifestVersion: 1.12.0`).
  - Added native manifest verification and silent installation testing CLI options to `scripts/generate-winget-manifest.mjs` (`--verify`, `--install`).

### Fixed

- **Repository Hygiene & IDE Untracking**:
  - Removed local `.vscode/` IDE directory from git tracking and added comprehensive editor ignore rules (`.vscode/`, `.idea/`, etc.) to `.gitignore`.

## [1.3.2] - 2026-10-16

### Fixed

- **Cross-Platform Compiler Diagnostics & Clean Multi-Platform Builds**:
  - Eliminated unused `imp::*` wildcard import warning in `src/platforms/windows.rs` on non-Windows targets by defining fallback stubs directly.
  - Resolved `unused_mut` compiler warnings in `src/platforms/detectors.rs` (`detect_ubisoft` and `detect_ea`) on Linux and macOS targets where registry detection logic only runs under Windows.
  - Resolved `unused_variables` warning in `src/platforms/launchers.rs` (`detect_emulator_installed`) for `_mounted_drives` on POSIX platforms.

### Added

- **Standalone WinGet Publishing & Dry-Run Validation Workflow**:
  - Added `.github/workflows/publish-winget.yml` allowing manual `workflow_dispatch` testing with dry-run schema validation, authentic SHA256 checksum generation, and Microsoft `wingetcreate` verification.

## [1.3.1] - 2026-10-15

### Added

- **Multi-Platform Free Game Promotions & Giveaways**:
  - Structured modular per-platform promotion fetchers (`platforms::promotions::epic`, `platforms::promotions::steam`, `platforms::promotions::gog`, and `platforms::promotions::gamerpower`) supporting verified 100% free-to-keep promotions across all enabled storefronts.
- **Store Deals & Major Discounts**:
  - Added real-time store deals integration checking featured Steam specials, Epic Games promotional discounts, and GOG catalog sales.
- **Library Patch Notes & Update Announcements**:
  - Integrated official game patch notes feed for installed library titles via Steam's News API with BBCode sanitization and direct announcement links.
- **Unified 3-Tab Notifications Subpage on Dashboard**:
  - Added dedicated tabbed notifications sub-panel accessible from the dashboard header with "🎁 Free Games" (with active vs. upcoming giveaways cleanly separated), "🏷️ Deals & Promotions", and "📰 Patch Notes & Updates".
- **Downloads & Component Status Subpage**:
  - Added downloads sub-page on dashboard providing real-time status visibility for tools (Legendary CLI) and gaming components.
- **Epic Online Services (EOS) Overlay Integration**:
  - Implemented EOS overlay status detection and one-click installation via Legendary CLI backend.

### Changed

- **Settings Navigation & Global Sidebar**:
  - Relocated the Settings access button to a dedicated gear cog button pinned at the bottom-left corner of the sidebar, freeing up the primary navigation bar.
- **Epic Games Platform Card Cleanliness**:
  - Standardized the Epic Games platform card to match uniform launcher styling without nested or redundant panels.
- **Custom Path Configuration**:
  - Added `⚙ Legendary & EOS Settings` drawer for configuring custom binary paths and EOS overlay management.

### Fixed

- **Free Game Promotion False Positives**:
  - Fixed promotion filter bug where non-free on-sale games leaked into free game notifications by strictly validating `price.totalPrice.discountPrice == 0` and active 0% promotional offers.
- **Legendary Account Status Detection**:
  - Fixed auth status parser to check `line.starts_with("Epic account:")` and `Account ID:`, correctly identifying active authenticated sessions.

## [1.3.0] - 2026-10-15

### Added

- **Executive Gaming Dashboard**:
  - Transformed the home view into a real-time gaming dashboard featuring an Executive KPI strip (Total Library, Ready to Play, Total Playtime, Starred Favorites, Free Storage capacity).
  - Added "Jump Back In" hero banner and recently-played/favorite games carousel with instantaneous "▶" game launch action and live playtime stats.
  - Interactive storefront ecosystem capacity distribution chart with multi-platform proportional progress meter and category filters.
  - Top Played Titles leaderboard tracking user engagement and total hours invested across the gaming collection.
  - Quick Actions hub for one-click access to Library views, Storage Explorer, Manual Game Addition, and Platform Integration settings.
- **Multi-Platform Storage Analytics & Storage Explorer**:
  - Re-engineered the Storage tab into a multi-platform storage analytics console tracking disk usage across all 12 supported platforms (Steam, GOG, Epic Games, Xbox Game Pass, EA App, Ubisoft Connect, Battle.net, itch.io, Amazon Games, Rockstar Games, Emulators, and Other).
  - Real-time connected storage drives overview charting aggregate volume utilization, total capacities, and dedicated gaming footprints per mount point.
  - Added the **Installed Games Storage Explorer** featuring live search, multi-platform filter pills, visual disk consumption meters, and deep links to game details.
- **Per-Game Installation Directory Management & Cross-Drive Relocation**:
  - Integrated an "Installation & Storage" card on the per-game details sub-page displaying exact disk footprint, resolved filesystem paths, main executable paths, and a direct "Open Folder" action.
  - Added game relocation capabilities in the Manage modal allowing users to move installed Steam games between registered library folders with pre-flight storage verification (`preview_move_game`) and atomic directory movement with automatic `appmanifest_<appid>.acf` rewriting (`move_game`).
  - Added deep linking support via `?game=<appId>` URL queries allowing immediate navigation from Storage Explorer to any game's management interface.
- **Serverless In-Memory & Backend Redis-Compatible KV Tier**:
  - Implemented high-performance Redis-compatible key-value storage engine (`src-tauri/src/cache.rs` and `src/lib/redis.ts`) for ultra-fast transient caching of game stats, metadata lookups, and session state.
  - Exposed Redis command primitives (`redis_get`, `redis_set`, `redis_del`, `redis_keys`, `redis_flush`) via strongly typed Tauri IPC commands.
- **Epic Games Launcher & Legendary CLI Integration**:
  - Added configurable binary/folder path support (`legendary_path`) allowing users to designate custom standalone installations on non-PATH locations.
  - Implemented multi-platform auto-detection scanning installation directories, standard tools folders, and mounted drives (e.g. `E:\Games\Epic Games\Legendary`).
  - Added direct "Download Legendary ↗" button linking to official GitHub releases alongside the 1-click built-in downloader.
  - Seamless fallback execution for Epic Games store titles when the official Epic Games Launcher is absent.
- **Clean Uninstallation NSIS Hooks**:
  - Embedded NSIS uninstaller hooks (`src-tauri/windows/hooks.nsh`) for Windows installations ensuring complete removal of WebView2 runtime caches, local app data, and temporary files when uninstalled.

### Changed

- **Sidebar & Status Cleanliness**:
  - Removed persistent "Steam is closed / running" process notice from the global sidebar.
  - Relocated live Steam process status monitor exclusively inside the Depressurizer Mode operation card in `/settings`.
- **Database Schema Upgrade**:
  - Upgraded offline SQLite cache to schema v4 with dedicated user review score caching.

### Fixed

- **Epic Games Account Portal OAuth `client_not_found` Error**:
  - Resolved `{"errorCode":"errors.com.epicgames.accountportal.client_not_found"}` by replacing outdated/deprecated client tokens with the active Legendary portal helper (`https://legendary.gl/epiclogin`) and valid client ID (`34a02cf8f4414e29b15921876da36f9a`).
  - Added resilient credential extraction handling pure authorization codes, JSON payloads, redirect URLs, and session ID (`--sid`) cookies.
- **Chromium WebView2 Win32 Window Class Unregistration**:
  - Resolved `UnregisterClassW failed: 1412 (ERROR_CLASS_ALREADY_EXISTS)` during rapid window teardown and reload in debug mode.
- **Relative Path Resolution for Installed Games**:
  - Resolved relative `installDir` manifests from Steam libraries against their respective `libraryPath` parent directory.

## [1.2.0] - 2026-10-15

### Added

- **Multi-Provider Metadata & Artwork Pipeline Parity**:
  - Full metadata & artwork engine expansion across Community Catalogue, Steam Store, GOG, PCGamingWiki, Wikipedia (with high-res box art pageimages), Libretro Emulation Thumbnails, VNDB, SteamGridDB, and IGDB.
  - Implemented `refresh_game_metadata` command cascading across providers and merging results into offline SQLite cache.
  - Added "Fetch & Refresh Library Metadata" UI block in Settings under Preferences & Offline Cache with live status indicators, auto-artwork toggle, and online catalogue sync trigger.
  - Added per-game "Refresh Metadata" button on game details topbar and in Manage modal with live spin feedback toast.
- **Steam-Parity Keyword Artwork Search & Broken Link Filtering**:
  - Artwork manager search is keyword-driven across all 5 visual slots (grid portrait, grid landscape, hero banner, logo, icon).
  - Expanded visual provider filter dropdown (All, Catalogue, GOG, Wikipedia, PCGamingWiki, VNDB, Emulation, SteamGridDB, Steam, IGDB, Metadata).
  - Client-side error removal (`onerror`) so dead/blocked CDN URLs are automatically culled, preventing broken cards.
- **Custom Display Name Override & Management Modal**:
  - Modal sub-window allowing users to specify custom display names for any game (persisted locally and seamlessly displayed across topbar, cards, and heroes).
- **Cross-Repo i18n & Multilingual Metadata Integration**:
  - Multi-language game metadata support (`locales`) for localized titles and descriptions matching active user language.
  - Unified 30-language Steam platform i18n support with reactive translations for all new actions.
- **Settings Navigation & Layout Consolidation**:
  - Consolidated Collections, Platforms & Launchers, and Library Organization into modular, dedicated section tabs within `/settings`.
  - Streamlined top navigation focusing on core library workflows.

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
