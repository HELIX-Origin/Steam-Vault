---
layout: docs
title: Supported Launchers & Platforms
summary: Detecting and aggregating games across Steam, Epic, GOG, Xbox, EA, Ubisoft, and local libraries
parent: Documentation Home
nav_order: 8
---

# Supported Launchers & Platforms

Universal Game Launcher connects with all popular PC gaming storefronts and launcher clients installed on your system. Rather than managing fragmented, isolated libraries across different launcher interfaces, Universal Game Launcher automatically aggregates all your installed titles into a single, cohesive hub.

## 🔍 Local, Authoritative Detection

Universal Game Launcher inspects **local, authoritative client records** — official launcher manifests, system registry entries, SQLite databases, and package descriptors written by the official stores:

- **Zero Credential Handling**: The app never prompts for passwords, executes web scraping, or accesses user tokens.
- **Offline Reliability**: Installed games are discovered locally on disk without requiring active network connectivity.
- **Configurable Platform Sources**: In **Settings → Platforms**, you can enable or disable specific storefronts, add custom search folders, and configure multi-drive scanning.

## 🎮 Supported Storefronts & Services

| Platform | Detection Mechanism | Launch Delegation Protocol |
| :--- | :--- | :--- |
| **Steam** | Local `libraryfolders.vdf` and `appmanifest_*.acf` descriptors | `steam://run/{app_id}` |
| **GOG Galaxy** | Local SQLite Galaxy database (`galaxy-2.0.db`) & Windows Registry | `goggalaxy://openGameView/{game_id}` |
| **Epic Games** | Official manifests (`.item` files in `Manifests/`) | `com.epicgames.launcher://apps/{app_name}?action=launch` |
| **Xbox / Microsoft Store** | `XboxGames` library volumes, `MicrosoftGame.config`, and `appxmanifest.xml` | Native AppModel execution protocol |
| **EA App** | Local EA installer registry entries and directory structures | Native client launch protocol |
| **Ubisoft Connect** | Local Ubisoft registry and launcher configurations | `uplay://launch/{game_id}/0` |
| **Battle.net** | Blizzard installation product databases and system registry | Battle.net client protocol |
| **itch.io** | The itch desktop app library (`%APPDATA%\itch\apps`) | `itch://games/{game_id}` |
| **Amazon Games** | Amazon Games library SQLite manifests and install folders | Amazon Games client protocol |
| **Manually Added Local Games** | User-specified executable paths and custom working directories | Native process execution |

## 🚀 Native Launcher Delegation

When launching, updating, or managing games from different stores, Universal Game Launcher strictly delegates actions to the official client:
- **Preserved Overlays & Cloud Sync:** Official client overlays, cloud save synchronization, playtime tracking, friend lists, and achievement unlocks function identically to launching from the official store.
- **Safe State Transitions:** If an update is required, the official launcher handles the secure package download without third-party intervention.

## 🗂️ Scanning & Managing Games

1. Open the **Platforms** view and click **Scan for games**.
2. Each platform card displays the number of games discovered, the install path consulted, and status indicators.
3. Select any discovered titles to add them to your unified library view.
4. Filter by platform instantly using the sidebar filters in the **Library** view.

## 🛡️ Write Safety & Backups

All custom tags, platform attributions, and collections are saved in the generic SQLite database (`database.sqlite`) with automatic transactional backups before any configuration changes.

## ➡️ Next steps

- [Manually Added Local Games](Non-Steam-Games.md)
- [Artwork Management](Artwork-Management.md)
- [Collections](Collections.md)
- [Organizing Your Library](Organizing-Your-Library.md)
