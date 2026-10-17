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

## How does metadata resolution work between game platforms and metadata providers?

Universal Game Launcher features an **Automatic Multi-Source Metadata Engine** (`metadata-universal`):
1. **Automatic Cascading:** When games are detected or imported, the engine cascades across Local Inspector, Libretro Thumbnails CDN, Steam Store, GOG Catalog v1, Wikipedia/Wikidata MediaWiki API, and PCGamingWiki Cargo API to automatically populate descriptions, release dates, reviews, and artwork.
2. **Modern Video Trailers:** For Steam games, modern Akamai CDN MP4 trailer streams (`movie_max.mp4`) are automatically extracted and playable in the **🎬 Trailers** tab.
3. **Per-Game Customization:** Users can edit any game's Title, Short Description, Developers, Publishers, Release Date, Genres, and Custom Artwork via the **⚙ Manage** modal in `/library`.
4. **Settings-Only Cache Maintenance:** Manual cache refresh and purge actions live cleanly in **Settings → Storage & Cache** (`↻ Refresh Metadata Cache`, `🗑 Purge Data Cache`).

## Where do platform icons come from?

Universal Game Launcher uses **Authentic Vector SVG Platform & Console Icons** (`PlatformIcon.svelte`) rather than generic Unicode shapes or low-resolution bitmaps. Every PC storefront and retro console is rendered with its authentic vector brand geometry.

## Does it upload my data or collect telemetry?

No. Universal Game Launcher is **100% on-device and privacy-first**. There is no account requirement, no user tracking, and zero telemetry. Network communication is strictly limited to metadata lookups to providers you have enabled.

## Do I need API keys to use it?

No. All primary platform integrations and core storefront metadata operate completely keyless out-of-the-box. Optional API keys (SteamGridDB for community artwork, IGDB for Twitch taxonomy) can be configured per-plugin in the **Plugin Settings** dialog to enrich high-resolution artwork and community metadata.

## Can I organize emulation and ROM libraries?

Yes. The Universal Platform Engine automatically scans multi-console ROM directories conforming to the ES-DE directory standard, supports automatic folder scaffolding (`systems.txt` + `systeminfo.txt`), extracts compressed ROM archives, creates isolated `rom.ini` per-game manifests, and features an intelligent title normalization engine that strips dump tags `(USA)`, `[!]`, `(Rev 1)`, and NKit extensions.

## Is there a command-line interface for automation?

Yes. Universal Game Launcher includes a headless CLI (`--cli scan`, `scan-roms`, `scaffold-roms`, `normalize-title`, and `launchers`) enabling automated library scanning, verification, and scriptable headless operation.

## Where do I report bugs or request features?

Open an issue on the [Universal Game Launcher GitHub Issues](https://github.com/HELIX-Origin/Universal-Game-Launcher/issues). For security reports, see [SECURITY.md](https://github.com/HELIX-Origin/Universal-Game-Launcher/blob/main/SECURITY.md).
