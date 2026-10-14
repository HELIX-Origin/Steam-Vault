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

## 🎮 Supported platforms

| Platform | How it is detected |
| :--- | :--- |
| **GOG** | GOG Galaxy registry keys (`SOFTWARE\GOG.com\Games`) |
| **Epic Games** | Epic Games Launcher manifests, plus legendary / Heroic on Linux and macOS |
| **itch.io** | The itch app library (`%APPDATA%\itch\apps`) |
| **Amazon Games** | The Amazon Games library folder |
| **Ubisoft Connect** | Windows Uninstall registry (publisher match) |
| **EA** | Windows Uninstall registry (publisher match) |
| **Battle.net** | Windows Uninstall registry (publisher match) |
| **Rockstar** | Windows Uninstall registry (publisher match) |
| **Other studios** | Windows Uninstall registry (publisher heuristic) |

> **Note:** Microsoft Store / Xbox Game Pass titles are not registered in the classic Uninstall
> registry and are not detected yet.

## 🗂️ Importing games

1. Open the **Platforms** view and press **Scan for games**.
2. Each platform card reports how many games were found and which source was consulted.
3. Select the games you want and press **Import selected**.

Imported games are written to Steam's `shortcuts.vdf` as **non-Steam shortcuts**, exactly as if you
had added them through Steam. Steam Vault:

- derives each shortcut's appid the same way Steam does (`crc32(exe + name)` with the high bit set),
  so re-importing the same game is a no-op;
- records which platform each shortcut came from, so the **Platform** sorting option can group them;
- refuses to write while Steam is running, and backs up `shortcuts.vdf` first.

## 🧭 After importing

Imported shortcuts appear in the **Library** with a pill naming their platform. From there you can:

- place them into collections by hand; or
- use **Organize → Platform** to create one collection per storefront in a single step.

## ➡️ Next steps

- [Non-Steam Games](Non-Steam-Games.md)
- [Organizing Your Library](Organizing-Your-Library.md)
