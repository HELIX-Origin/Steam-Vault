---
layout: docs
title: Available Plugins Catalog
summary: Exhaustive reference of Universal Core platform, metadata, utility, and media plugins with global helpers and configurations
parent: Documentation Home
nav_order: 5
---

# Available Plugins Catalog

Universal Game Launcher implements a high-performance **Universal Core Plugins architecture** (`src/lib/plugins/`). Rather than fragmenting platform and metadata features into dozen of fragile, uncoordinated modules, Universal Game Launcher consolidates core functionality into unified, domain-specific **Universal Core Plugins** paired with **Global Helper Modules**.

All plugins operate as native TypeScript modules compiled directly into the application, guaranteeing zero network CDN vulnerabilities, 100% offline capability, instant loading, and clean architectural decoupling.

---

## 🏛️ Universal Core Plugins Overview

```
src/lib/plugins/
├── framework.ts                  # Core UniversalPlugin TypeScript interfaces and contracts
├── engine.ts                     # In-memory registry, KV persistence, and metadata helpers
├── catalog.ts                    # Master registry exporting all active Universal Core plugins
├── index.ts                      # Public barrel exports
├── platforms/
│   └── universal.ts              # platform-universal (12 platform specs + global helpers)
├── metadata/
│   └── universal.ts              # metadata-universal (automatic multi-source cascade)
├── utility/
│   ├── title-normalizer.ts       # utility-title-normalizer (clean titles, RPG Maker & ROM normalizer)
│   ├── deals.ts                  # utility-deals (100% free giveaways & store deals)
│   ├── patchnotes.ts             # utility-patchnotes (developer game update feeds)
│   └── storage.ts                # utility-storage (non-blocking drive footprint analyzer)
└── media/
    └── universal.ts              # media-universal (Steam CDN MP4 trailer streaming & soundtracks)
```

---

## 🕹️ 1. Universal Platform Discovery & Launcher Engine (`platform-universal`)

- **Plugin ID**: `platform-universal`
- **Version**: `1.4.0` · **Author**: `HELIX Origin`
- **Description**: Unified platform engine managing discovery, launch protocol delegation, social rosters, and achievement hooks across all 12 supported PC storefronts and console emulation systems.
- **Declared Features**: `discovery`, `launch`, `client_delegation`, `legendary_cli`, `cloud_saves`, `social`, `achievements`, `store_cdn`, `multi_drive_scaffolding`

### Supported Platforms & Target Specifications

| Platform ID | Display Label | Detection Mechanism | Launch Delegation Protocol | Social & Achievements |
| :--- | :--- | :--- | :--- | :---: |
| `steam` | **Steam** | Parses `libraryfolders.vdf` and `appmanifest_*.acf` across mounted drives | `steam://rungameid/{appId}` | Yes |
| `epic` | **Epic Games** | Local `.item` manifests in `%ProgramData%\Epic\EpicGamesLauncher\Data\Manifests\` | `com.epicgames.launcher://apps/{appId}?action=launch&silent=true` (Official) or `legendary launch <appName>` (CLI) | Yes |
| `gog` | **GOG Galaxy** | Local Galaxy SQLite database (`galaxy-2.0.db`) & Windows Registry | `goggalaxy://openGameView/{gameId}` or direct executable launch | Yes |
| `xbox` | **Xbox / Microsoft Store** | `XboxGames` library volumes, `MicrosoftGame.config`, and `appxmanifest.xml` | `shell:AppsFolder\{PackageFamilyName}!App` | Yes |
| `ea` | **EA Desktop** | Local EA installer registry entries and directory structures | `origin2://game/launch?offerIds={appId}` | Yes |
| `ubisoft` | **Ubisoft Connect** | Local Ubisoft registry and launcher configurations | `uplay://launch/{gameId}/0` | Yes |
| `battlenet` | **Battle.net** | Blizzard installation product databases and system registry | `battlenet://{appId}` | Yes |
| `itch` | **itch.io** | The itch desktop app library (`%APPDATA%\itch\apps`) and receipt manifests | `itch://games/{gameId}` or direct executable launch | No |
| `amazon` | **Amazon Games** | Amazon Games library SQLite manifests and install folders | `amazon-games://play/{appId}` | No |
| `rockstar` | **Rockstar Games** | Rockstar Games Launcher registry and manifests | `rockstar://launch/{appId}` | No |
| `standalone` | **Standalone Executables** | Manually added local executables, custom arguments, and working directories | Native process execution | No |
| `emulation` | **EmulationStation** | Multi-console ROM directories organized via the ES-DE standard | Direct emulator binary delegation or launcher | No |

### Global Platform Helper Modules

Exported alongside `platformUniversalPlugin` in `src/lib/plugins/platforms/universal.ts`:

- `getAllPlatformSpecs(): PlatformSpec[]` — Returns specifications, colors, protocols, and detection metadata for all 12 platforms.
- `detectPlatformLauncher(platformId, config): Promise<LauncherDetectionResult>` — Detects whether the official platform client is installed locally.
- `installPlatformLauncher(platformId): Promise<boolean>` — Opens the official client installation page or installer package.
- `getPlatformLaunchSpec(platformId, game, config): PlatformLaunchSpec | null` — Derives the exact URI protocol or binary launch arguments for a game.
- `getPlatformUris(platformId, game): { installUri?, uninstallUri?, storePageUri? }` — Formulates official client install, uninstall, and store URLs.

---

## 📚 2. Universal Multi-Source Metadata Engine (`metadata-universal`)

- **Plugin ID**: `metadata-universal`
- **Version**: `1.4.0` · **Author**: `HELIX Origin`
- **Description**: Automatic multi-source metadata engine cascading across native and community data sources to enrich descriptions, release dates, reviews, and high-resolution artwork without requiring manual intervention.
- **Declared Features**: `automatic_enrichment`, `multi_source_cascade`, `steam_store`, `gog_catalog`, `wikipedia_wikidata`, `libretro_cdn`, `pcgamingwiki`, `protondb`, `steamgriddb`, `igdb`

### Automatic Enrichment Cascade

When a game is imported or indexed, the Universal Metadata Engine queries sources in priority order:

1. **Local Inspector**: Parses local game folders, RPG Maker `System.json` (`"gameTitle"`), itch.io receipt manifests, `rom.ini` overrides, and PE binary headers.
2. **Libretro Thumbnails CDN**: Downloads authentic console box art (`Named_Boxarts`), title screens (`Named_Titles`), snaps, and logos for retro console ROMs.
3. **Steam Store API**: Official Valve store details, genres, developer credits, publisher credits, user reviews, 1080p screenshots, and modern Akamai CDN MP4 video trailers.
4. **GOG Catalog v1 API**: Official DRM-free store covers, hero wallpapers, and product descriptions.
5. **Wikipedia & Wikidata MediaWiki API**: Encyclopedic game synopses and Wikidata taxonomy properties (`P178` Developer, `P123` Publisher, `P136` Genre, `P577` Release Date) for games without official store pages.
6. **PCGamingWiki Cargo API**: Game engine detection, architecture flags, save game directory locations, and widescreen/FOV fixes.
7. **ProtonDB API**: Linux and Steam Deck compatibility ratings (`Native`, `Platinum`, `Gold`, `Silver`, `Bronze`, `Borked`) and recommended launch arguments.
8. **SteamGridDB (Optional)**: Community capsule covers (600×900), hero banners (1920×620), transparent logos, and square icons (requires free user API key).
9. **IGDB / Twitch (Optional)**: Comprehensive global video game taxonomy and critic ratings (requires Twitch Developer credentials).

---

## 🔤 3. Universal Title Normalizer (`utility-title-normalizer`)

- **Plugin ID**: `utility-title-normalizer`
- **Version**: `1.4.0` · **Author**: `HELIX Origin`
- **Description**: Dedicated title normalization engine operating across frontend TypeScript and backend Rust (`src-tauri/src/plugins/title_normalizer.rs`) to produce clean, authentic game titles across PC and console libraries.
- **Capabilities**:
  - **RPG Maker Inspection**: Inspects 1-level child folders for `www/data/System.json` or `data/System.json` to extract authentic `gameTitle` (e.g. resolving `relicts-auf-aeson` to **`Relicts of Aeson`**).
  - **Slug Separation**: Intelligently decomposes concatenated lowercase directory slugs (e.g. `myfutafamily` to **`My Futa Family`**).
  - **Article Inversion**: Converts inverted library titles to standard reading format (e.g. `Legend of Zelda, The - The Wind Waker` to **`The Legend of Zelda: The Wind Waker`**).
  - **ROM Dump Tag Stripping**: Strips dump tags `(USA)`, `[!]`, `(Rev 1)`, `(Track 01)`, and `.nkit` extensions.
  - **Deduplication**: Generates canonical title keys to prevent duplicate entries in custom game listings.

---

## 🎬 4. Universal Ambient Audio & Media Engine (`media-universal`)

- **Plugin ID**: `media-universal`
- **Version**: `1.4.0` · **Author**: `HELIX Origin`
- **Description**: Powers the consolidated media showcase in `/library` with streaming video trailers and ambient soundtrack playback.
- **Capabilities**:
  - **Modern Steam CDN Trailer Streaming**: Parses Steam Store `movies` arrays to extract high-definition Akamai MP4 video streams (`https://cdn.akamai.steamstatic.com/steam/apps/{movie_id}/movie_max.mp4`) rendered via native HTML5 video with full player controls.
  - **Ambient Game Audio**: Plays official game soundtrack themes and ambient music in the background while viewing titles in the showcase.
  - **Dynamic Tab Rendering**: Media tabs dynamically adapt: `🖼️ Screenshots & Art`, `🎬 Trailers` (displayed only when trailers are available), and `🎵 Soundtracks` (rendered dynamically when audio is present).

---

## 🎁 5. Free Games & Store Deals (`utility-deals`)

- **Plugin ID**: `utility-deals`
- **Version**: `1.4.0` · **Author**: `HELIX Origin`
- **Description**: Delivers real-time notifications and browser feeds for 100% free game promotions and major store sales across Steam, Epic Games, GOG, and Humble Bundle.
- **Capabilities**:
  - Live giveaway tracking powered by GamerPower feeds.
  - Highlights limited-time 100% discount promotions.
  - Direct store delegation to claim giveaways safely in your browser or official client.

---

## 📰 6. Patch Notes & News Feed (`utility-patchnotes`)

- **Plugin ID**: `utility-patchnotes`
- **Version**: `1.4.0` · **Author**: `HELIX Origin`
- **Description**: Aggregates developer patch notes, update announcements, and community news for installed games in your library.
- **Capabilities**:
  - Automatically queries official client news feeds for installed titles.
  - Displays formatted update changelogs and version release notes directly on the Dashboard.

---

## 💾 7. Storage & Drive Footprint Analyzer (`utility-storage`)

- **Plugin ID**: `utility-storage`
- **Version**: `1.4.0` · **Author**: `HELIX Origin`
- **Description**: Non-blocking disk storage analytics engine inspecting drive capacities and installed game footprints.
- **Capabilities**:
  - Offloaded to native background thread workers (`spawn_blocking`) for 0ms UI freezing.
  - Computes per-drive capacity gauges and per-platform game storage consumption.
  - Powers the **Storage** view (`/storage`) and the **Settings → Storage & Cache** controls.

---

## ➡️ Next Steps

- [Metadata Providers Architecture](Metadata-Providers.md) — multi-source automatic cascade details.
- [Organizing Your Library](Organizing-Your-Library.md) — the Library showcase, unified media tabs, and game management.
- [Artwork Management](Artwork-Management.md) — custom artwork studio and slots.
- [Settings & Plugin Configuration](Settings-and-API-Keys.md) — configuring optional API keys.
- [Plugin Development Guide]({{ '/wiki/Plugin-Developer-Guide/' | relative_url }}) — authoring new plugins.
