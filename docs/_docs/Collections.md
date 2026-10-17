---
layout: docs
title: Collections
summary: Organizing multi-platform games into custom and dynamic smart collections
parent: Documentation Home
nav_order: 4
---

# Collections

In Universal Game Launcher, **collections** allow you to group, organize, and filter games across all your platforms — including Steam, Epic Games, GOG Galaxy, Xbox, EA App, Ubisoft Connect, Battle.net, itch.io, Amazon Games, Standalone executables, and Emulation / ROM titles.

## 🗃️ Generic Local Persistence

Unlike legacy tools that altered storefront cloud files, Universal Game Launcher is completely **non-invasive**:

- Collections are persisted locally and safely in the generic embedded SQLite database (`database.sqlite`).
- Backed by an in-memory fast Key-Value tier (`mem_kv`) for sub-millisecond retrieval with zero UI latency.
- Universal Game Launcher **never modifies external client files** and never requires Steam or other launchers to be closed.

## ➕ Creating and Managing Collections

From the **Steam & GOG-style Left Sidebar** or the **Library** view, you can:

- **Create a New Collection:** Group games by franchise, mood, completion priority, or multiplayer status.
- **Add / Remove Games:** Assign any title from any platform to one or multiple collections.
- **Favorites & Hidden:** Tag any game as a Favorite (`★`) for instant top-tier filtering or hide finished/unwanted titles.
- **Direct Sidebar Filtering:** Expand or collapse collections in the sidebar to filter the games list instantly.

## 🧩 Smart Collections

Universal Game Launcher supports dynamic smart collections backed by rule filters (genre, install state, tags, or keywords). As new games are imported or detected across plugins, smart collections update their membership automatically.

## ➡️ Next steps

- [Manually Added Local Games](Non-Steam-Games.md) — organizing standalone games and custom executables.
- [Artwork Management](Artwork-Management.md) — customize covers, heroes, logos, and icons.
- [Plugin Development](Plugin-Development.md) — creating platform and metadata plugins.
