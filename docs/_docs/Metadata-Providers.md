---
layout: docs
title: Metadata Providers
summary: Automatic multi-source metadata resolution engine, provider cascade hierarchy, and per-game customization
parent: Documentation Home
nav_order: 6
---

# Metadata Providers & Architecture

Universal Game Launcher implements an intelligent, automatic multi-source metadata resolution framework (`metadata-universal`). Rather than requiring tedious manual scraping or relying on fragile single-source APIs, the architecture automatically cascades across native platform endpoints, encyclopedic APIs, and community repositories in the background.

---

## 🏛️ Automatic Resolution Architecture

```
┌────────────────────────────────────────────────────────┐
│               Game Discovered or Imported              │
│       Steam · Epic · GOG · itch · Local · Consoles     │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│       Universal Metadata Engine (metadata-universal)   │
│                                                        │
│  1. Local Inspector  (System.json, itch, rom.ini)     │
│  2. Libretro CDN     (Retro console box art & snaps)  │
│  3. Steam Store API  (Details, reviews, MP4 trailers) │
│  4. GOG Catalog v1   (Vertical covers, summaries)     │
│  5. Wikipedia/Wikidata (Synopses, devs, pubs, dates)  │
│  6. PCGamingWiki     (Engines, save locations, fixes) │
│  7. ProtonDB         (Linux/Steam Deck compatibility) │
│  8. SteamGridDB/IGDB (Optional user-authenticated)    │
└───────────────────────────┬────────────────────────────┘
                            │ Field-by-field merge
                            ▼
┌────────────────────────────────────────────────────────┐
│            Local SQLite & In-Memory KV Cache           │
│        database.sqlite  ·  mem_kv (game_meta:<appId>)  │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│        Per-Game Customization Modal (⚙ Manage)         │
│   User edits Title, Description, Devs, Pubs, Art       │
│               [Saved locally with 0ms lag]             │
└────────────────────────────────────────────────────────┘
```

---

## ⚡ Key Architectural Invariants

### 1. Automatic Background Resolution
When games are imported or detected, the metadata engine automatically resolves missing fields across all supported data sources in the background. Games populate with rich descriptions, release dates, developer credits, high-resolution capsules, hero banners, and video trailers without requiring user intervention.

### 2. Modern Steam CDN Video Trailers
For Steam titles, the engine extracts high-definition Akamai MP4 trailer URLs directly from the Steam Store `movies` array:
```plaintext
https://cdn.akamai.steamstatic.com/steam/apps/{movie_id}/movie_max.mp4
```
Trailers are streamed on demand via native HTML5 video inside the **🎬 Trailers** tab of the consolidated media showcase.

### 3. Per-Game Customization Modal (`⚙ Manage` in `/library`)
Every game provides a dedicated **⚙ Manage** modal in the Library showcase view where users can directly customize:
- **Title (Display Name)**: Override auto-derived or platform titles.
- **Short Description**: Custom overview text displayed on the showcase.
- **Developers & Publishers**: Studio credits and publishing labels.
- **Release Date**: Formatted release year or date string.
- **Genres**: Custom category tag pills.
- **Custom Artwork**: Set Portrait Capsule Cover, Hero Wallpaper Banner, and Transparent Logo via either direct image URL or local file picker (`Browse...`).

All overrides persist locally in SQLite KV (`game_meta:<appId>`) and are protected from background scan overwrites.

### 4. Settings-Only Cache Controls
To maintain an ergonomic, clutter-free Library view, manual cache actions are isolated exclusively in **Settings → Storage & Cache**:
- **↻ Refresh Metadata Cache**: Re-runs the multi-source enrichment cascade for all games without discarding custom user overrides.
- **🗑 Purge Data Cache**: Clears temporary cached images and in-memory KV records to rebuild from scratch.

---

## 🔌 Built-In Data Sources Reference

| Provider Source | Category | Key Required | Primary Information & Assets Provided |
| :--- | :---: | :---: | :--- |
| **Local Inspector** | Local Manifest | No | RPG Maker `System.json` `gameTitle`, itch.io receipt manifests, `rom.ini` configs, binary PE headers. |
| **Libretro CDN** | Metadata & Art | No | Authentic console box art (`Named_Boxarts`), title screens (`Named_Titles`), snaps, and logos for retro console ROMs. |
| **Steam Store** | Metadata & Media | No | Genres, developers, publishers, release dates, reviews, 1080p screenshots, and modern Akamai CDN MP4 video trailers. |
| **GOG Catalog v1** | Metadata | No | Official vertical covers, hero wallpapers, summaries, DRM-free releases. |
| **Wikipedia & Wikidata** | Metadata | No | Encyclopedic game synopses and Wikidata taxonomy properties (`P178` Developer, `P123` Publisher, `P136` Genre, `P577` Release Date). |
| **PCGamingWiki Cargo** | Metadata | No | Game engine, architecture, save file locations, and widescreen/FOV fixes. |
| **ProtonDB** | Metadata | No | Linux and Steam Deck compatibility ratings (`Native`, `Platinum`, `Gold`, `Silver`, `Bronze`, `Borked`) and launch flags. |
| **SteamGridDB** | Metadata & Art | Yes (Optional) | Community capsule covers (600×900), hero banners (1920×620), transparent logos, and square icons. |
| **IGDB (Twitch)** | Metadata | Yes (Optional) | Global video game taxonomy, studio links, and aggregated critic scores. |

---

## 🔑 Authentication & API Key Configuration

Only two metadata sources require user-supplied credentials:
- **SteamGridDB**: Enter your free API key from your SteamGridDB account preferences (`steamgriddb.com/profile/preferences/api`).
- **IGDB**: Enter your Twitch Application Client ID and Client Secret from the Twitch Developer Console (`dev.twitch.tv/console`). Universal Game Launcher automatically manages OAuth token acquisition and renewal.

All credentials are configured in the dedicated **Plugin Settings Dialog** (**Settings → Plugin Settings**), stored locally in `database.sqlite` via `mem_kv`, and verified in real-time with **⚡ Test Connection**.

---

## 🗄️ Local Caching & Performance

All fetched metadata is stored locally in `database.sqlite` and indexed in the in-memory Redis KV tier (`game_meta:<appId>`). Subsequent library loads retrieve cached metadata with **0ms blocking time**.

---

## ➡️ Next Steps

- [Organizing Your Library](Organizing-Your-Library.md) — the Library showcase, unified media tabs, and per-game management.
- [Artwork Management](Artwork-Management.md) — customizing artwork slots across games.
- [Settings & Plugin Configuration](Settings-and-API-Keys.md) — configuring optional API keys and cache settings.
