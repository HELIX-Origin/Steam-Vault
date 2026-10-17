---
layout: docs
title: FAQ
summary: Frequently asked questions about Universal Game Launcher
parent: Documentation Home
nav_order: 11
---

# FAQ

## What is Universal Game Launcher?

Universal Game Launcher is a fast, standalone, privacy-first cross-platform universal game library organizer and launcher built with Tauri v2 (Rust backend and SvelteKit frontend). It aggregates your game libraries across Steam, Epic Games Store, GOG Galaxy, Xbox, EA App, Ubisoft Connect, Battle.net, itch.io, Amazon Games, Standalone local executables, and Emulation / ROM collections.

## Does Universal Game Launcher modify my launcher files or require Steam to be closed?

No. Universal Game Launcher is completely **non-invasive**. It **never** writes to Steam's `shortcuts.vdf`, `cloud-storage-namespace-1.json`, or external client files. Official launchers do not need to be closed while using Universal Game Launcher. All collections, custom names, and artwork mappings are persisted locally in our generic embedded SQLite database (`database.sqlite`) backed by an in-memory Redis KV tier (`mem_kv`).

## How does the app launch games?

Universal Game Launcher uses **native client protocol delegation**:
- Storefront games launch via their official client URI schemes (`steam://`, `com.epicgames.launcher://`, `goggalaxy://`, etc.).
- Standalone titles and emulator plugins execute their respective local binaries directly.
This preserves official game overlays, cloud synchronization, playtime tracking, friend lists, and achievement unlocks without ever touching or storing your credentials.

## How does metadata resolution work between game providers and metadata providers?

Universal Game Launcher enforces an authoritative role hierarchy:
1. **Game Providers (Default Authoritative Source):** When games are detected, the platform provider plugin (Steam, Epic, GOG, itch, EA, EmulationStation, Standalone) fetches its own authoritative game details and artwork directly from its own source.
2. **Metadata Providers (Manual Enhancement):** Third-party metadata providers (TheGamesDB, ScreenScraper, IGDB, SteamGridDB, HowLongToBeat, PCGamingWiki, ProtonDB) are run on demand by the user via **✨ Enhance Metadata** in the showcase view to enrich games and fill in missing fields.
3. **Cache Reset Invariant:** Game providers are strictly prohibited from overwriting metadata contributed by metadata providers unless the user explicitly chooses to reset the metadata cache via **↺ Reset Cache to Provider Defaults**.

## Does it upload my data or collect telemetry?

No. Universal Game Launcher is **100% on-device and privacy-first**. There is no account requirement, no user tracking, and zero telemetry. Network communication is strictly limited to user-requested metadata and artwork lookups to providers you explicitly enable.

## Do I need API keys to use it?

No. All primary platform integrations and core storefront metadata operate completely keyless out-of-the-box. Optional API keys (SteamGridDB for community artwork, IGDB for Twitch taxonomy) can be configured per-plugin in the **Plugin Settings** sub-window to enrich high-resolution artwork and community metadata.

## How does the plugin architecture work?

The core application provides the UI, storage statistics, persistence, and launcher engine. All external features — platform integrations, emulation/ROM systems, metadata scrapers, free game alerts, deals, and patch notes — are provided via native in-app modular plugins under `src/lib/plugins/`.

## Can I organize emulation and ROM libraries?

Yes. The `platform-emulationstation` plugin automatically scans multi-console ROM directories conforming to the ES-DE directory standard, supports automatic folder scaffolding, and features an intelligent title normalization engine that strips dump tags `(USA)`, `[!]`, `(Rev 1)`, and NKit extensions.

## Where do I report bugs or request features?

Open an issue on the [Universal Game Launcher GitHub Issues](https://github.com/HELIX-Origin/Universal-Game-Launcher/issues). For security reports, see [SECURITY.md](https://github.com/HELIX-Origin/Universal-Game-Launcher/blob/main/SECURITY.md).
