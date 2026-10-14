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
- New **Platforms** and **Organize** views (six-item navigation) plus platform and
  installed-state badges in the Library.

### Fixed

- `shortcuts.vdf` now reads canonical field casing (`AppName` / `Exe`) and strips the
  literal quotes Steam stores around `Exe` / `StartDir`; the writer emits the canonical
  `Exe` / `StartDir` keys instead of lowercase ones.
