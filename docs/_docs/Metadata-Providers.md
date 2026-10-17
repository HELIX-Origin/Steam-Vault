---
layout: docs
title: Metadata Providers
summary: Authoritative metadata role hierarchy, manual enhancement plugins, and provider reference
parent: Documentation Home
nav_order: 6
---

# Metadata Providers & Architecture

Universal Game Launcher implements an intelligent, multi-tier metadata resolution framework. Rather than forcing third-party web scrapers onto every title, the architecture establishes a clean, authoritative role hierarchy between **Native Game Providers** and **Metadata Providers**.

---

## 🏛️ Authoritative Role Hierarchy & Invariants

```
┌────────────────────────────────────────────────────────┐
│              Authoritative Game Providers              │
│  Steam, Epic Games, GOG, itch.io, EA, EmulationStation │
│  (Default source for game titles, installs & artwork)  │
└───────────────────────────┬────────────────────────────┘
                            │ Authoritative Initial Data
                            ▼
┌────────────────────────────────────────────────────────┐
│           Local Library & In-Memory KV Cache           │
│         database.sqlite  ·  mem_kv (AppMeta)           │
└───────────────────────────┬────────────────────────────┘
                            │ Manual User Enhancement
                            ▼
┌────────────────────────────────────────────────────────┐
│             Third-Party Metadata Providers             │
│   TheGamesDB, ScreenScraper, IGDB, SteamGridDB, etc.   │
│   (On-demand: fills missing fields or overrides)       │
│              [userEnhanced: true]                      │
└────────────────────────────────────────────────────────┘
```

### 1. Game Providers are the Authoritative Default
When your library is scanned or games are imported, the native platform provider plugin (such as `platform-steam`, `platform-epic`, `platform-gog`, `platform-itch`, `platform-ea`, `platform-emulationstation`, or `platform-standalone`) queries its own authoritative storefront or local files:
- **Steam**: Valve store details API and Cloudflare CDN artwork.
- **GOG Galaxy**: Official GOG product endpoints and Galaxy client database.
- **Epic Games**: Official `.item` catalog manifests and promotional keyart.
- **itch.io**: itch catalog metadata and cover images.
- **EmulationStation**: Normalized ROM titles and system taxonomy.
- **Standalone**: Local PE version headers and executable inspection.

This ensures zero external dependencies and fast, accurate defaults for all official games.

### 2. Metadata Providers are Manual Enhancement Tools
Third-party metadata providers (such as **TheGamesDB**, **ScreenScraper**, **IGDB**, **SteamGridDB**, **HowLongToBeat**, **PCGamingWiki**, and **ProtonDB**) are intended to **enhance** your library on demand. They are run manually when you choose:
- Click **✨ Enhance Metadata** in the game showcase view or open **Manage Game → Metadata & Provider Management**.
- Select specific providers to query, or query all enabled metadata plugins simultaneously.
- Choose whether to **Fill Missing Fields Only** (the safe default, which leaves existing information intact and fills in blank descriptions, missing genres, or absent release years) or **Override Existing Metadata**.

### 3. The Cache Reset Invariant
To prevent external sync tasks or platform re-scans from blowing away your custom enhancements:
- Whenever a game is enhanced by a metadata provider, it is tagged with `userEnhanced: true` and recorded with `enhancedBy: [...]` in the persistent cache.
- **Native game providers are strictly prohibited from overwriting metadata contributed by metadata providers during normal background scans.**
- A game provider may only overwrite user-enhanced metadata if the user explicitly triggers **↺ Reset Cache to Provider Defaults** in the Manage Game modal.

---

## 🔌 Built-In Metadata Provider Reference

Universal Game Launcher includes 11 dedicated, native metadata plugins under `src/lib/plugins/metadata/`:

| Provider Plugin | Key Required | Source Endpoint | Primary Information & Assets Provided |
| :--- | :---: | :--- | :--- |
| **Steam Store** (`metadata-steam`) | No | `store.steampowered.com/api/appdetails` | Genres, developers, publishers, release dates, reviews, 1080p screenshots. |
| **GOG Catalog** (`metadata-gog`) | No | `embed.gog.com` | Official GOG vertical covers, hero wallpapers, summaries, DRM-free releases. |
| **Epic Games Store** (`metadata-epic`) | No | `store-content.ak.epicgames.com` | Catalog details, promotional keyart covers, release dates, publisher info. |
| **itch.io Store** (`metadata-itch`) | No | `itch.io/api/1` | Indie titles, developer profiles, screenshots, library sync. |
| **SteamGridDB** (`metadata-steamgriddb`) | User Key | `www.steamgriddb.com/api/v2` | Community capsule covers (600×900), hero banners (1920×620), transparent logos, square icons. |
| **IGDB (Twitch)** (`metadata-igdb`) | User Key | `api.igdb.com/v4/games` | Global taxonomy, developer studios, franchise links, aggregated critic scores. |
| **TheGamesDB** (`metadata-thegamesdb`) | No | `api.thegamesdb.net/v1/` | Retro console metadata, authentic box art, fanart heroes, clear logos, release taxonomy. |
| **ScreenScraper** (`metadata-screenscraper`) | No | `screenscraper.fr` | Multi-console ROM metadata, 2D/3D box art, wheel logos, video snaps, screenshot titles. |
| **HowLongToBeat** (`metadata-howlongtobeat`) | No | `howlongtobeat.com/api/search` | Main Story, Main + Extra, and Completionist playtime averages. |
| **PCGamingWiki** (`metadata-pcgamingwiki`) | No | `www.pcgamingwiki.com/w/api.php` | Game engine, FOV/ultrawide fixes, controller flags, save and config locations. |
| **ProtonDB** (`metadata-protondb`) | No | `www.protondb.com/api/v1` | Linux / Steam Deck compatibility tiers (Native, Platinum, Gold) and launch tweaks. |

---

## 🔑 Authentication & API Key Configuration

Only two metadata providers require user-supplied credentials:
- **SteamGridDB**: Enter your free API key from your SteamGridDB account preferences (`steamgriddb.com/profile/preferences/api`).
- **IGDB**: Enter your Twitch Application Client ID and Client Secret from the Twitch Developer Console (`dev.twitch.tv/console`). Universal Game Launcher automatically manages OAuth token acquisition and renewal.

All credentials are configured in the dedicated **Plugin Settings Dialog** (**Settings → Plugin Settings**), stored locally in `database.sqlite` via `mem_kv`, and verified in real-time with **⚡ Test Connection**.

---

## 🗄️ Local Caching & Performance

All fetched metadata is stored locally in `database.sqlite` and indexed in the in-memory Redis KV tier (`game_meta:<appId>`). Subsequent library loads retrieve cached metadata with **0ms blocking time**.

---

## ➡️ Next Steps

- [Available Plugins Catalog](Available-Plugins.md) — complete catalog of all 27 plugins.
- [Plugin Development Guide](Plugin-Development.md) — build custom metadata plugins.
- [Artwork Management](Artwork-Management.md) — customize covers, heroes, logos, and icons.
