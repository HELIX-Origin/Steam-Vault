---
layout: docs
title: FAQ
summary: Frequently asked questions about Universal Game Launcher
parent: Documentation Home
nav_order: 11
---

# FAQ

## What is Universal Game Launcher?

Universal Game Launcher is a fast, standalone, privacy-first cross-platform universal game library organizer and launcher built with Tauri v2 (Rust backend and SvelteKit frontend). It aggregates your game libraries across Steam, Epic Games Store, GOG Galaxy, Xbox, EA App, Ubisoft Connect, Battle.net, itch.io, Amazon Games, Standalone executables, and Emulation / ROM collections.

## Does Universal Game Launcher modify my launcher files or require Steam to be closed?

No. Universal Game Launcher is completely **non-invasive**. It **never** writes to Steam's `shortcuts.vdf`, `cloud-storage-namespace-1.json`, or external client files. Official launchers do not need to be closed while using Universal Game Launcher. All collections, custom names, and artwork mappings are persisted locally in our generic embedded SQLite database (`database.sqlite`).

## How does the app launch games?

Universal Game Launcher uses **native client protocol delegation**:
- Storefront games launch via their official client URI schemes (`steam://`, `com.epicgames.launcher://`, `goggalaxy://`, etc.).
- Standalone titles and emulator plugins execute their respective local binaries directly.
This preserves official game overlays, cloud synchronization, playtime tracking, friend lists, and achievement unlocks without ever touching or storing your credentials.

## Does it upload my data or collect telemetry?

No. Universal Game Launcher is **100% on-device and privacy-first**. There is no account requirement, no user tracking, and zero telemetry. Network communication is strictly limited to user-requested metadata and artwork lookups to providers you explicitly enable.

## Do I need API keys to use it?

No. All primary platform integrations and core storefront metadata operate completely keyless out-of-the-box. Optional API keys (e.g. SteamGridDB, IGDB, MobyGames) can be configured per-plugin in the **Plugin Settings** sub-window to enrich high-resolution artwork and community metadata.

## How does the plugin architecture work?

The core application provides the UI, storage statistics, persistence, and launcher engine. All external features — platform integrations, emulation/ROM systems, metadata scrapers, free game alerts, deals, and patch notes — are provided via native in-app modular plugins under `src/lib/plugins/`.

## Can I organize emulation and ROM libraries?

Yes. Emulation platforms and ROM scrapers are fully supported through the drop-in plugin ecosystem. Because Universal Game Launcher distributes direct installers rather than being restricted by app store policies, users can freely install and configure emulator and ROM plugins.

## Where do I report bugs or request features?

Open an issue on the [Universal Game Launcher GitHub Issues](https://github.com/HELIX-Origin/Universal-Game-Launcher/issues). For security reports, see [SECURITY.md](https://github.com/HELIX-Origin/Universal-Game-Launcher/blob/main/SECURITY.md).
