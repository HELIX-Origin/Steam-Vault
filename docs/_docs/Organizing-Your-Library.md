---
layout: docs
title: Organizing Your Library
summary: The Steam & GOG-style sidebar games list and main game showcase view
parent: Documentation Home
nav_order: 3
---

# Organizing Your Library

Universal Game Launcher pairs a proven **Steam and GOG Galaxy-style games list sidebar** with a dedicated **Main Game Showcase View**, giving you instant access to your entire multi-platform library.

## 🎮 Left Sidebar — The Games List

The left sidebar is permanently accessible across the application and serves as your interactive game selector:

- **Instant Search:** Type any title to filter the games list instantly in real-time.
- **Filter Pills:**
  - **All:** Displays all imported titles across all platforms.
  - **Installed:** Filters exclusively to titles with verified local installations on disk.
  - **★ Favorites:** Shows your pinned favorite games across all storefronts.
- **Background Scan (`↻ Scan`):** Trigger an asynchronous background scan of all platform integrations without interrupting your gameplay or browsing.
- **Platform Accordions:** Collapsible platform headers with badge and live game counts (Steam, Epic Games, GOG Galaxy, Xbox, EA App, Ubisoft Connect, Battle.net, itch.io, Amazon Games, Standalone, and Emulation / ROMs). Clicking a platform expands or collapses its games.
- **Collections Tree:** Expand custom and dynamic collections to focus on specific groups.
- **Instant Selection:** Clicking any game instantly loads that title in the Main Showcase View with zero page reload latency.

## 🖼️ Main Game Showcase View (`/library`)

When a game is selected, the main view showcases the title in rich detail:

- **16:9 Hero Wallpaper Banner:** High-resolution backdrop with a dark vignette overlay.
- **Title & Primary Action Header:**
  - 2:3 Portrait capsule artwork.
  - Transparent game logo or crisp title typography.
  - Big prominent **PLAY NOW** launch button that delegates to the native platform client or executes the local binary.
  - Favorite star toggle and Manage dropdown (open install folder, rename, custom arguments).
  - Playtime counter and Last Played timestamp.
- **10-Screenshot Media Gallery:** Interactive screenshot carousel supporting up to 10 images with a fullscreen lightbox viewer.
- **Overview & Metadata:** Developer, Publisher, Release Date, Store links, and Community Tag pills.
- **Custom Artwork Studio:** Dedicated artwork slot inspector allowing you to customize portrait capsule, hero banner, transparent logo, and square icon via SteamGridDB or local file imports.

## 🗂️ Adding to Collections

You can assign games to custom collections directly from the game's Manage menu or the Collections inspector, safely persisted in local SQLite with 0ms in-memory cache synchronization.

## ➡️ Next steps

- [Collections](Collections.md) — managing custom and dynamic collections.
- [Artwork Management](Artwork-Management.md) — customize covers, heroes, logos, and icons.
- [Plugin Development](Plugin-Development.md) — build custom platform and metadata plugins.
