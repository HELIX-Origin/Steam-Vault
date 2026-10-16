---
layout: docs
title: Documentation Home
summary: Overview and introduction to Universal Game Launcher
permalink: /docs/
nav_order: 1
---

# What is Universal Game Launcher?

**Universal Game Launcher** is a fast, lightweight, and privacy-first desktop application that unifies your entire multi-platform PC gaming library. It brings together games across **Steam**, **Epic Games Store**, **GOG Galaxy**, **Ubisoft Connect**, **EA App**, **Xbox / Microsoft Store**, **Battle.net**, **itch.io**, and **Amazon Games**, alongside full support for **manually adding local games and standalone executables**.

Inspired by the clean organization philosophies of **Depressurizer**, Universal Game Launcher lets you build custom collections, auto-categorize by genre, developer, and tags, manage multi-drive storage, and track free game giveaways and store discounts.

It is built on **Tauri v2** (Rust backend with a SvelteKit frontend) and relies on **official launcher protocols and local client records** rather than screen-scraping or credential storage.

## 🚀 Key Highlights

- 🎮 **All Popular Game Launchers** — Discover and launch games across Steam, Epic Games, GOG Galaxy, Xbox, EA, Ubisoft, Battle.net, itch.io, and Amazon Games.
- 📁 **Manually Added Local Games** — Easily import and organize standalone DRM-free executables, custom launch arguments, and indie releases.
- 🔒 **Native Launcher Delegation** — Launches, updates, and uninstalls delegate directly to official clients, preserving overlays, cloud saves, friend lists, playtime, and achievements.
- 🗂️ **Custom & Smart Collections** — Create custom categories and evaluate native dynamic filter rules with live previews.
- 🧠 **AutoCats** — Generate categories automatically from genre, developer, publisher, and tags.
- 🎛️ **Advanced Filters** — An Allowed (OR) / Required (AND) / Excluded (NOT) rule builder over your entire collection.
- 🖼️ **Artwork Management** — Inspect and set custom portrait grids, heroes, logos, and icons using SteamGridDB and local image files.
- 🎁 **Free Games & Store Deals** — Track 100% free game giveaways and major store promotions with zero payment processing in-app.
- 💽 **Storage Management** — Inspect drive capacity and game footprints across library folders, with safe cross-drive moves.
- ⚡ **Generic SQLite Persistence** — Zero-dependency embedded database (`database.sqlite`) in WAL mode with transactional backups.
- 🌍 **28+ Languages** — Hand-rolled reactive i18n localization engine supporting all major languages with Arabic RTL support.

## 🧭 Getting Around

- **[Getting Started](Getting-Started.md)** — prerequisites, build, and first launch.
- **[Organizing Your Library](Organizing-Your-Library.md)** — the Library view, filters, and selections.
- **[Supported Launchers & Platforms](External-Platforms.md)** — multi-launcher detection and client delegation.
- **[Manually Added Local Games](Non-Steam-Games.md)** — standalone executables, custom artwork, and tagging.
- **[Collections](Collections.md)** — creating, editing, and safely writing collections.
- **[AutoCategorization](AutoCategorization.md)** — the AutoCats categorization engine.
- **[Artwork Management](Artwork-Management.md)** — custom art for any game.
- **[Storage Management](Storage-Management.md)** — libraries, drives, and moving games.
- **[Metadata Providers](Metadata-Providers.md)** — provider capabilities and keyless architecture.
- **[Settings & API Keys](Settings-and-API-Keys.md)** — configuration reference.
- **[Troubleshooting](Troubleshooting.md)** — common issues and fixes.
- **[FAQ](FAQ.md)** — quick answers.

## 💡 Quick Facts

- Stack: **Rust / Tauri v2** backend + **SvelteKit static SPA** (Svelte 5 Runes, strict TypeScript).
- Architecture: **Local & Keyless** — reads official client manifests on disk; external API keys are optional.
- Privacy: **100% on-device**. No telemetry, no user accounts, and no credential collection.

## 🔗 Links

- [Repository](https://github.com/HELIX-Origin/Steam-Vault)
- [Changelog](https://github.com/HELIX-Origin/Steam-Vault/blob/main/CHANGELOG.md)
- [Security](https://github.com/HELIX-Origin/Steam-Vault/blob/main/SECURITY.md)
- [License](https://github.com/HELIX-Origin/Steam-Vault/blob/main/LICENSE)

---

### 📚 Documentation Index
- **Core**: [Home]({{ '/' | relative_url }}) · [Getting Started](Getting-Started.md) · [Organizing Your Library](Organizing-Your-Library.md)
- **Features**: [Supported Launchers](External-Platforms.md) · [Manually Added Local Games](Non-Steam-Games.md) · [Collections](Collections.md) · [AutoCategorization](AutoCategorization.md) · [Artwork Management](Artwork-Management.md) · [Storage Management](Storage-Management.md)
- **Reference**: [Metadata Providers](Metadata-Providers.md) · [Settings & API Keys](Settings-and-API-Keys.md) · [Troubleshooting](Troubleshooting.md) · [FAQ](FAQ.md)

---

*[Universal Game Launcher](https://github.com/HELIX-Origin/Steam-Vault) — a fast, lightweight universal game launcher and library organizer.*
