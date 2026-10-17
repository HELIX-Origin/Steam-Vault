---
layout: docs
title: Organizing Your Library
summary: The Steam & GOG-style sidebar games list, showcase view, and metadata enhancement workflows
parent: Documentation Home
nav_order: 3
---

# Organizing Your Library

Universal Game Launcher pairs a proven **Steam and GOG Galaxy-style games list sidebar** with a dedicated **Main Game Showcase View**, giving you instant access to your entire multi-platform library.

---

## 🎮 Left Sidebar — The Games List

The left sidebar is dedicated strictly to the **Library view** (`/library`), serving as your interactive game selector while other application views (Dashboard, Store, Social, Storage, Plugins, Settings) render full-width without clutter:

- **Instant Search:** Type any title to filter the games list in real time with sub-millisecond responsiveness.
- **Filter Pills:**
  - **All:** Displays all imported titles across all platforms.
  - **Installed:** Filters exclusively to titles with verified local installations on disk.
  - **★ Favorites:** Shows your pinned favorite games across all storefronts.
- **Background Scan (`↻ Scan`):** Trigger an asynchronous background scan across all platform integrations and EmulationStation ROM folders without interrupting your gameplay.
- **+ Add Game:** Opens the Standalone Game addition modal to import arbitrary local executables, DRM-free games, or custom shortcuts.
- **Platform Accordions:** Collapsible platform headers with live game counts (Steam, Epic Games, GOG Galaxy, Xbox, EA Desktop, Ubisoft Connect, Battle.net, itch.io, Amazon Games, Standalone, and EmulationStation). Clicking an accordion header toggles visibility for that storefront.
- **Collections Tree:** Expand custom and dynamic collections to focus on specific user-curated groups.
- **Instant Selection:** Clicking any game immediately loads that title into the Main Showcase View with zero page reload latency.

---

## 🖼️ Main Game Showcase View (`/library`)

When a game is selected, the showcase view presents the title in rich detail:

- **16:9 Hero Wallpaper Banner:** High-resolution backdrop with a dark vignette overlay.
- **Title & Primary Action Header:**
  - 2:3 Portrait capsule artwork.
  - Transparent game logo or crisp title typography.
  - Big prominent **PLAY NOW** launch button that delegates to the native platform client or executes the local binary.
  - Favorite star toggle (`★`) and Manage dropdown (open install folder, rename, custom arguments, manage metadata).
  - Playtime counter and Last Played timestamp.
- **✨ Enhance Metadata Action:** Showcase button allowing you to run third-party metadata providers (TheGamesDB, ScreenScraper, IGDB, SteamGridDB, HowLongToBeat, PCGamingWiki, ProtonDB) on demand to enrich descriptions, reviews, and missing assets.
- **10-Screenshot Media Gallery:** Interactive screenshot carousel supporting up to 10 images with a fullscreen lightbox viewer.
- **Overview & Metadata:** Developer, Publisher, Release Date, Store links, and Community Tag pills.
- **Custom Artwork Studio:** Dedicated artwork slot inspector allowing you to customize portrait capsule, hero banner, header grid, transparent logo, and square icon via community providers or local file imports.

---

## 🔄 Metadata & Provider Management Modal

Clicking **Manage → Manage Metadata** on any title opens the dedicated Metadata Management modal:
- **Provider Source Selection**: Choose to query all enabled metadata providers or select a specific provider (e.g. TheGamesDB or ScreenScraper for retro ROMs, SteamGridDB for custom community art, IGDB for taxonomy).
- **Override Existing Metadata Checkbox**: Toggle whether to fill missing fields only (default) or overwrite existing data.
- **↺ Reset Cache to Provider Defaults**: Clears the user-enhanced cache record and restores official game provider defaults.

---

## 🗂️ Adding to Collections

You can assign games to custom collections directly from the game's Manage menu or the Collections inspector, safely persisted in local SQLite (`database.sqlite`) with 0ms in-memory cache synchronization (`mem_kv`).

---

## ➡️ Next Steps

- [Collections](Collections.md) — managing custom and dynamic collections.
- [Artwork Management](Artwork-Management.md) — customize covers, heroes, logos, and icons.
- [Metadata Providers](Metadata-Providers.md) — role hierarchy and provider capabilities.
- [Available Plugins Catalog](Available-Plugins.md) — complete catalog of all 27 plugins.
