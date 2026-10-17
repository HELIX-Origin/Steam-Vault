---
layout: docs
title: Organizing Your Library
summary: The Steam & GOG-style sidebar games list, authentic SVG icons, unified media tabs, and per-game customization
parent: Documentation Home
nav_order: 3
---

# Organizing Your Library

Universal Game Launcher pairs a proven **Steam and GOG Galaxy-style games list sidebar** with a dedicated **Main Game Showcase View**, giving you instant access and rich customization over your entire multi-platform library.

---

## 🎮 Left Sidebar — The Games List

The left sidebar is dedicated strictly to the **Library view** (`/library`), serving as your interactive game selector while other application views (Dashboard, Store, Social, Storage, Settings) render full-width without clutter:

- **Instant Search:** Type any title to filter the games list in real time with sub-millisecond responsiveness.
- **Filter Pills:**
  - **All:** Displays all imported titles across all platforms.
  - **Installed:** Filters exclusively to titles with verified local installations on disk.
  - **★ Favorites:** Shows your pinned favorite games across all storefronts.
- **Background Scan (`↻ Scan`):** Trigger an asynchronous background scan across all platform integrations and EmulationStation ROM folders without interrupting your gameplay.
- **+ Add Game:** Opens the Standalone Game addition modal to import arbitrary local executables, DRM-free games, or custom shortcuts.
- **Authentic Vector SVG Platform & Console Icons:** Every platform and emulation console is represented by an authentic vector SVG icon (`PlatformIcon.svelte`) instead of generic Unicode symbols.
- **Collapsible Platform & Console Accordions:** Expand or collapse storefronts (Steam, Epic Games, GOG Galaxy, Xbox, EA Desktop, Ubisoft Connect, Battle.net, itch.io, Amazon Games, Standalone) and retro console categories with live game counts.
- **Collections Tree:** Expand custom and dynamic collections to focus on specific user-curated groups.
- **Instant Selection:** Clicking any game immediately loads that title into the Main Showcase View with zero page reload latency.

---

## 🖼️ Main Game Showcase View (`/library`)

When a game is selected, the showcase view presents the title in rich detail with optimized layout density:

- **16:9 Hero Wallpaper Banner:** High-resolution backdrop with a dark vignette overlay.
- **Title & Primary Action Header:**
  - 2:3 Portrait capsule artwork.
  - Authentic Vector SVG Platform Badge indicating the source client.
  - Transparent game logo or crisp title typography.
  - Big prominent **PLAY NOW** launch button that delegates to the native platform client or executes the local binary.
  - Favorite star toggle (`★`).
  - **⚙ Manage Button**: Opens the full Per-Game Customization Modal.
  - Playtime counter and Last Played timestamp.
- **Consolidated Unified Media Showcase:**
  - Replaces disjointed carousels with a streamlined, tabbed media section:
    - **🖼️ Screenshots & Art**: High-resolution screenshot gallery with an interactive fullscreen lightbox viewer.
    - **🎬 Trailers**: Modern Steam Akamai CDN MP4 video trailer streaming (`https://cdn.akamai.steamstatic.com/steam/apps/{movie_id}/movie_max.mp4`) with native HTML5 player controls.
    - **🎵 Soundtracks**: Dynamic ambient soundtrack player playing theme audio when available.
- **Tightened Layout Ergonomics**: Card margins, `.details-grid`, and `.details-sidebar` are tuned to eliminate dead vertical space and maximize screen real estate.
- **Overview & Metadata**: Developer, Publisher, Release Date, Store links, and Community Tag pills.

---

## ⚙️ Per-Game Customization Modal (`⚙ Manage`)

Clicking **⚙ Manage** on any title in the showcase opens the comprehensive Per-Game Customization modal:

- **Title (Display Name)**: Edit the game's displayed name in the sidebar and showcase.
- **Short Description**: Custom overview text.
- **Developers & Publishers**: Authoritative creator and publishing credits.
- **Release Date**: Custom release date string.
- **Genres**: Comma-separated genre tags.
- **Custom Artwork Slots**:
  - **Portrait Capsule Cover**: 600×900 vertical box art via direct URL or local file picker (`Browse...`).
  - **Hero Wallpaper Banner**: 16:9 wide backdrop art via direct URL or local file picker (`Browse...`).
  - **Transparent Logo**: Transparent PNG title logo via direct URL or local file picker (`Browse...`).
- **Safe Persistence**: All customizations save directly to local SQLite (`game_meta:<appId>`) and update the UI reactively with 0ms reload latency. Background platform scans will never overwrite your custom entries.

---

## 🧹 Settings-Only Cache Controls

To keep the Library interface focused strictly on gaming and customization, manual cache management buttons have been moved out of `/library` and into **Settings → Storage & Cache**:
- **↻ Refresh Metadata Cache**: Safely re-runs the multi-source enrichment cascade for all games without touching user customizations.
- **🗑 Purge Data Cache**: Clears temporary cached images and in-memory KV records to rebuild from scratch.

---

## 🗂️ Adding to Collections

You can assign games to custom collections directly from the game's Manage menu or the Collections inspector, safely persisted in local SQLite (`database.sqlite`) with 0ms in-memory cache synchronization (`mem_kv`).

---

## ➡️ Next Steps

- [Artwork Management](Artwork-Management.md) — customizing covers, heroes, and logos.
- [Metadata Providers](Metadata-Providers.md) — automatic multi-source metadata engine.
- [Collections](Collections.md) — managing custom and dynamic collections.
- [Available Plugins Catalog](Available-Plugins.md) — Universal Core plugins reference.
