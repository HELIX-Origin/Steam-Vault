---
layout: docs
title: External Platforms
summary: Detecting and importing games from GOG, Epic, itch.io, and more
parent: Documentation Home
nav_order: 8
---

# External Platforms

Most players own games outside Steam. Steam Vault can find those games on your machine and import
them into Steam as non-Steam shortcuts, so they appear in your library and can be organized into
collections alongside everything else.

## 🔍 What "detection" means

Steam Vault reads **local, authoritative sources** — launcher manifests, registry keys, and library
folders that the launchers themselves write. It never scrapes a storefront, never signs in on your
behalf, and never launches another launcher.

Which sources are consulted is configurable. Under **Settings → Non-Steam sources** you enable or
disable each launcher, add **extra locations** to search, and choose whether to scan other mounted
drives. Disabled sources are skipped entirely.

Like Steam's `libraryfolders.vdf`, each launcher records its installed games in its own config, so
the location Steam Vault asks for is usually that **config / app path** — loading it is enough to
find games. You can also add plain **game install folders**, which are the fallback when a
launcher's config is missing or wrong, and the only option for local games that have no launcher.
**Find folders** autofills the config and library locations it can see.

## 🎮 Supported platforms

Each platform is read from the config or library that its launcher writes:

| Platform | How it is detected |
| :--- | :--- |
| **GOG** | GOG Galaxy registry keys (`SOFTWARE\GOG.com\Games`) |
| **Epic Games** | Epic Games Launcher manifests, plus legendary / Heroic on Linux and macOS |
| **itch.io** | The itch app library (`%APPDATA%\itch\apps`) |
| **Amazon Games** | The Amazon Games library folder |
| **Xbox / Microsoft Store** | `XboxGames` library folders, `MicrosoftGame.config`, and `appxmanifest.xml` |
| **Ubisoft Connect** | Windows Uninstall registry (publisher match) |
| **EA** | Windows Uninstall registry (publisher match) |
| **Battle.net** | Windows Uninstall registry (publisher match) |
| **Rockstar** | Windows Uninstall registry (publisher match) |
| **Emulators & ROMs** | Detects installed emulators (Dolphin, PCSX2, DuckStation, Project64, PPSSPP, RPCS3, Citra, Cemu, Ryujinx, mGBA, Snes9x, RetroArch) and scans their ROM folders |
| **Other studios** | Windows Uninstall registry (publisher heuristic) |

## 🕹️ Emulators and ROMs

Steam Vault can discover your installed emulators and recursively scan configured ROM libraries. Supported emulators include:

- **Nintendo GameCube / Wii:** Dolphin Emulator
- **Nintendo 64:** Project64, Mupen64Plus
- **Sony PlayStation 1 & 2:** PCSX2, DuckStation
- **Sony PlayStation 3 & Portable:** RPCS3, PPSSPP
- **Nintendo 3DS, Wii U & Switch:** Citra, Cemu, Ryujinx
- **Retro Handhelds & Consoles:** mGBA, Snes9x, RetroArch

When imported, ROM shortcuts are configured with the emulator executable and appropriate command-line flags (e.g., `-b -e "<rom>"` for Dolphin) so they boot directly into full gameplay from Steam's Big Picture mode or desktop client.

## 🗂️ Importing games

1. Open the **Platforms** view and press **Scan for games**.
2. Each platform card reports how many games were found and which source was consulted.
3. Select the games you want and press **Import selected**.

Imported games are written to Steam's `shortcuts.vdf` as **non-Steam shortcuts**, exactly as if you
had added them through Steam. Steam Vault:

- derives each shortcut's appid the same way Steam does (`crc32(exe + name)` with the high bit set),
  so re-importing the same game is a no-op;
- writes storefront launch URIs (Epic `com.epicgames.launcher://apps/…`, itch `itch://games/…`, Xbox
  AppX package protocols) into the shortcut's launch options;
- records which platform each shortcut came from, so the **Platform** sorting option can group them;
- refuses to write while Steam is running, and backs up `shortcuts.vdf` first.

## 🧭 After importing

Imported shortcuts appear in the **Library** with a pill naming their platform. From there you can:

- re-tag a shortcut's source storefront individually via its platform dropdown, or in bulk by selecting multiple shortcuts in the toolbar;
- place them into collections by hand; or
- use **Organize → Platform** to create one collection per storefront in a single step.

## ➡️ Next steps

- [Non-Steam Games](Non-Steam-Games.md)
- [Organizing Your Library](Organizing-Your-Library.md)

