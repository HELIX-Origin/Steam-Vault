---
layout: docs
title: Supported Launchers & Platforms
summary: Detecting and aggregating games across Steam, Epic, GOG, Xbox, EA, Ubisoft, and local libraries
parent: Documentation Home
nav_order: 8
---

# Supported Launchers & Platforms

Universal Game Launcher connects with all popular PC gaming storefronts and launcher clients installed on your system. Rather than managing fragmented, isolated libraries across different launcher interfaces, Universal Game Launcher automatically aggregates all your installed titles into a single, cohesive hub.

---

## 🔍 Local, Authoritative Detection

Universal Game Launcher inspects **local, authoritative client records** — official launcher manifests, system registry entries, SQLite databases, and package descriptors written by the official stores:

- **Zero Credential Handling**: The app never prompts for passwords, executes web scraping, or accesses user tokens.
- **Offline Reliability**: Installed games are discovered locally on disk without requiring active network connectivity.
- **Authentic Vector SVG Platform Icons**: In both the Library sidebar and game showcase, platforms are badged with pixel-perfect vector SVGs (`PlatformIcon.svelte`) instead of generic glyphs.
- **Configurable Platform Sources**: In **Settings → Platforms** and the **Plugin Settings** dialog, you can enable or disable specific storefronts, customize executable paths, and configure multi-drive scanning.

---

## 🎮 Supported Storefronts & Services (12 Platforms)

Managed via the consolidated **Universal Platform Engine** (`platform-universal`):

| Platform Specification | Detection Mechanism | Launch Delegation Protocol | Social & Achievements |
| :--- | :--- | :--- | :---: |
| **Steam** | Parses `libraryfolders.vdf` and `appmanifest_*.acf` files across all mounted drives | `steam://rungameid/{appId}` | Yes |
| **Epic Games** | Official `.item` manifests in `%ProgramData%\Epic\EpicGamesLauncher\Data\Manifests\` | `com.epicgames.launcher://apps/{appId}?action=launch&silent=true` (Default) or `legendary launch <appName>` (CLI) | Yes |
| **GOG Galaxy** | Local Galaxy SQLite database (`galaxy-2.0.db`) & Windows Registry | `goggalaxy://openGameView/{gameId}` or direct executable launch | Yes |
| **Xbox / Microsoft Store** | `XboxGames` library volumes, `MicrosoftGame.config`, and `appxmanifest.xml` | `shell:AppsFolder\{PackageFamilyName}!App` | Yes |
| **EA Desktop** | Local EA installer registry entries and directory structures | `origin2://game/launch?offerIds={appId}` | Yes |
| **Ubisoft Connect** | Local Ubisoft registry and launcher configurations | `uplay://launch/{gameId}/0` | Yes |
| **Battle.net** | Blizzard installation product databases and system registry | `battlenet://{appId}` | Yes |
| **itch.io** | The itch desktop app library (`%APPDATA%\itch\apps`) and receipt files | `itch://games/{gameId}` or direct executable launch | No |
| **Amazon Games** | Amazon Games library SQLite manifests and install folders | `amazon-games://play/{appId}` | No |
| **Rockstar Games** | Rockstar Games Launcher registry and manifests | `rockstar://launch/{appId}` | No |
| **Standalone Executables** | User-added local executables, custom arguments, and working directories | Native process execution | No |
| **EmulationStation** | Multi-console ROM directories organized via the ES-DE standard | Direct emulator binary delegation or launcher | No |

---

## 🚀 Native Launcher Delegation & Alternate Backends

When launching, updating, or managing games from different stores, Universal Game Launcher strictly delegates actions to the official client or configured alternate providers:
- **Preserved Overlays & Cloud Sync:** Official client overlays, cloud save synchronization, playtime tracking, friend lists, and achievement unlocks function identically to launching from the official store.
- **Epic Games Alternate Provider (Legendary CLI):** The Epic Games integration defaults to the official Epic Games Launcher. If you prefer headless execution, faster launching, and independent cloud saves, you can enable Legendary integration in **Settings → Plugin Settings** and specify the path to your `legendary.exe` binary.
- **Safe State Transitions:** If an update is required, the official launcher handles the secure package download without third-party intervention.

---

## 🕹️ EmulationStation Scaffolding & ROM Manifests

For retro and console libraries:
- **Directory Scaffolding**: Automatically generates standard ES-DE folder layouts with `systems.txt` and `systeminfo.txt`.
- **Zip Extraction**: Compressed ROM archives (`.zip`, `.7z`, `.rar`) are safely unpacked into dedicated `<system>/<CleanTitle>/` cache directories with isolated `saves/` folders.
- **Per-Game Manifests**: Generates isolated `rom.ini` configuration files per game to preserve clean title mappings and custom emulator flags.

---

## 🗂️ Scanning & Managing Games

1. Open the **Library** view (`/library`).
2. Click the **↻ Scan** button at the top of the left games list sidebar.
3. Universal Game Launcher asynchronously inspects all enabled platform integrations and EmulationStation ROM folders in the background.
4. Games are automatically organized into collapsible platform and console accordions with live game counts.
5. Filter by platform, status (`All`, `Installed`, `★ Favorites`), or search query in real time.

---

## 🛡️ Write Safety Invariant

Universal Game Launcher is completely non-invasive:
- **Zero External File Writes:** Never writes to, modifies, or backs up external client files (such as Steam's `shortcuts.vdf` or `libraryfolders.vdf`).
- All custom tags, display names, platform attributions, and collections are saved strictly in our generic SQLite database (`database.sqlite`) backed by the in-memory Redis KV tier (`mem_kv`).

---

## ➡️ Next Steps

- [Available Plugins Catalog](Available-Plugins.md) — exhaustive details on Universal Core plugins.
- [Manually Added Local Games](Non-Steam-Games.md) — standalone executables and custom shortcuts.
- [Artwork Management](Artwork-Management.md) — custom covers, heroes, and logos.
- [Collections](Collections.md) — grouping titles across platforms.
