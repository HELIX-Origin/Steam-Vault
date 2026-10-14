---
layout: docs
title: Documentation Home
summary: Overview and introduction to Steam Vault
permalink: /docs/
nav_order: 1
---

# What is Steam Vault?

**Steam Vault** is a lightweight, cross-platform desktop application for organizing your real
Steam library. It is a clean-room revival of **Depressurizer**: instead of maintaining a separate
list of games, it reads the data Steam already stores on your machine, lets you reorganize it, and
writes your changes back into Steam itself.

It is built as a **Tauri v2** application — a Rust backend with a SvelteKit frontend — and it
prefers **stable, documented data sources** over screen-scraping, so ordinary Steam updates are
far less likely to break it.

- 🗂️ **Real collections** — read and write the collections Steam shows in your library.
- 🧩 **Manual organizing** — multi-select games and add or remove them from collections at once.
- 🧠 **AutoCats** — create categories automatically from genre, developer, publisher, tags, flags,
  release year, and user score.
- 🎛️ **Advanced filters** — an Allowed / Required / Excluded rule builder over your whole library.
- 👾 **Non-Steam games** — import and manage non-Steam shortcuts, with artwork from SteamGridDB.
- 🔌 **Metadata providers** — Steam Store, Steam Web API, SteamGridDB, IGDB, PCGamingWiki, and
  Wikipedia. Only Steam Web API, SteamGridDB, and IGDB need keys. **RAWG is not supported.**
- 🛡️ **Safe writes** — timestamped backups before every write, and changes are refused while Steam
  is running.

> **Early development.** Steam Vault is at milestone **M1 (Foundation)**. The data layer, metadata
> providers, AutoCats engine, and app shell exist; the polished interface and packaged installers
> are on the roadmap. See
> [Getting Started](Getting-Started.md) to build it from source.

## 🧭 Getting around

- **[Getting Started](Getting-Started.md)** — prerequisites, build, and first launch.
- **[Organizing Your Library](Organizing-Your-Library.md)** — the Library view and selections.
- **[Collections](Collections.md)** — creating, editing, and safely writing collections.
- **[AutoCategorization](AutoCategorization.md)** — the AutoCats engine.
- **[Metadata Providers](Metadata-Providers.md)** — every provider and what it contributes.
- **[Non-Steam Games](Non-Steam-Games.md)** — shortcuts and artwork.
- **[Settings & API Keys](Settings-and-API-Keys.md)** — configuration reference.
- **[Troubleshooting](Troubleshooting.md)** — common issues and fixes.
- **[FAQ](FAQ.md)** — quick answers.

## 💡 Quick facts

- Stack: **Rust / Tauri v2** backend + **SvelteKit static SPA** (Svelte 5, strict TypeScript).
- Data sources: Steam's own local files (`libraryfolders.vdf`, `appmanifest_*.acf`,
  `localconfig.vdf`, `shortcuts.vdf`, and the collection cloud-storage namespace).
- Privacy: everything is processed locally. No telemetry, no accounts, no servers.

## 🔗 Links

- [Repository](https://github.com/HELIX-Origin/Steam-Vault)
- [Changelog](https://github.com/HELIX-Origin/Steam-Vault/blob/main/CHANGELOG.md)
- [Security](https://github.com/HELIX-Origin/Steam-Vault/blob/main/SECURITY.md)
- [License](https://github.com/HELIX-Origin/Steam-Vault/blob/main/LICENSE)

---

### 📚 Documentation Index
- **Core**: [Home]({{ '/' | relative_url }}) · [Getting Started](Getting-Started.md) · [Organizing Your Library](Organizing-Your-Library.md)
- **Features**: [Collections](Collections.md) · [AutoCategorization](AutoCategorization.md) · [Non-Steam Games](Non-Steam-Games.md)
- **Reference**: [Metadata Providers](Metadata-Providers.md) · [Settings & API Keys](Settings-and-API-Keys.md) · [Troubleshooting](Troubleshooting.md) · [FAQ](FAQ.md)

---

*[Steam Vault](https://github.com/HELIX-Origin/Steam-Vault) — a cross-platform Steam library organizer.*
