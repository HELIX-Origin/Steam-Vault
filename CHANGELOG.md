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
