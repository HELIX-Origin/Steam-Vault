---
layout: docs
title: Documentation Home
summary: Overview and introduction to Universal Game Launcher
permalink: /docs/
nav_order: 1
---

# What is Universal Game Launcher?

**Universal Game Launcher** is a fast, lightweight, and privacy-first desktop application that unifies your entire multi-platform PC gaming library. It brings together games across **Steam**, **Epic Games Store**, **GOG Galaxy**, **Ubisoft Connect**, **EA App**, **Xbox / Microsoft Store**, **Battle.net**, **itch.io**, and **Amazon Games**, alongside full support for **manually added standalone executables** and **emulation / ROM libraries via drop-in plugins**.

Universal Game Launcher provides an ergonomic, high-performance user experience inspired by the proven layouts of **Steam and GOG Galaxy**:
- **Frameless Title Bar Navigation:** Quick switching between Dashboard, Library, Storage, Plugins, and Settings.
- **Steam & GOG-Style Left Sidebar:** Interactive game list with instant search, quick filter pills (All, Installed, ★ Favorites), background platform scan trigger, and collapsible platform and collection accordions.
- **Dedicated Game Showcase View:** Rich selected game view with 16:9 hero banner, capsule cover, title typography, prominent **PLAY NOW** launch button, 10-screenshot media gallery with fullscreen lightbox, details, and custom artwork studio.
- **Modular Drop-In Plugin Ecosystem:** All game platform integrations, emulation/ROM systems, metadata providers, and feeds (free games, store deals, patch notes) are decoupled into modular drop-in plugins.

Built on **Tauri v2** (Rust backend + SvelteKit frontend), Universal Game Launcher is 100% non-invasive. It never tampers with external store files, never harvests credentials, and stores library data locally in generic SQLite (`database.sqlite`) backed by an in-memory serverless KV tier for sub-millisecond responsiveness.

## 🚀 Key Highlights

- 🎮 **Steam & GOG Left Games List Sidebar** — Instant search, filter pills (All, Installed, Favorites), platform accordions with game counts, and collections tree.
- 🧭 **Frameless Title Bar Navigation** — Seamless header navigation: Dashboard, Library, Storage, Plugins, and Settings.
- 🖼️ **Dedicated Game Showcase View** — 16:9 hero wallpaper banner with dark vignette, title logo, prominent PLAY NOW launch button, and game overview.
- 📸 **10-Screenshot Media Gallery** — Interactive screenshot gallery with full-resolution lightbox viewer.
- 🧩 **Drop-In Plugin Ecosystem** — All game platforms, emulation/ROM systems, metadata providers, and notification feeds are modular plugins with schema validation.
- ⚙️ **Universal Per-Plugin Settings** — Dedicated configuration modal for API keys (with password toggle), custom executable paths, and options.
- 🔒 **Native Launcher Delegation** — Launches, updates, and uninstalls delegate directly to official clients, preserving overlays, cloud saves, friend lists, and achievements.
- 🎁 **Free Games, Deals & Patch Notes** — Plugin-powered feeds for verified 100% free giveaways, major store deals, and official game update notes.
- 🎨 **Custom Artwork Studio** — Inspect and set custom portrait capsules, heroes, logos, and icons using SteamGridDB and local files.
- 🗂️ **Custom & Smart Collections** — Create custom categories and evaluate dynamic filter rules with instant preview.
- 💾 **Storage Footprint Explorer** — Inspect drive capacity and game footprints across library volumes.
- ⚡ **Zero-Lag Obsidian Purple Theme** — High-contrast dark purple visual theme (#080511 / #0f0920 / #a855f7) with zero `backdrop-filter: blur` performance overhead.

## 🧭 Getting Around

- **[Getting Started](Getting-Started.md)** — prerequisites, build, and first launch.
- **[Organizing Your Library](Organizing-Your-Library.md)** — the Library showcase view, sidebar filters, and game launcher.
- **[Supported Launchers & Platforms](External-Platforms.md)** — multi-launcher detection, plugins, and client delegation.
- **[Manually Added Local Games](Non-Steam-Games.md)** — standalone executables, custom artwork, and tagging.
- **[Collections](Collections.md)** — creating, editing, and managing collections.
- **[Artwork Management](Artwork-Management.md)** — custom art studio for capsules, heroes, logos, and icons.
- **[Storage Management](Storage-Management.md)** — drive utilization and game storage exploration.
- **[Plugin Development](Plugin-Development.md)** — creating platform and metadata plugins.
- **[Metadata Providers](Metadata-Providers.md)** — provider capabilities and keyless architecture.
- **[Settings & API Keys](Settings-and-API-Keys.md)** — configuration reference.
- **[Troubleshooting](Troubleshooting.md)** — common issues and fixes.
- **[FAQ](FAQ.md)** — quick answers.

## 💡 Quick Facts

- Stack: **Rust / Tauri v2** backend + **SvelteKit static SPA** (Svelte 5 Runes, strict TypeScript).
- Architecture: **Local & Keyless** — reads official client manifests on disk; external API keys are optional.
- Privacy: **100% on-device**. No telemetry, no user accounts, and no credential collection.

## 🔗 Links

- [Repository](https://github.com/HELIX-Origin/Universal-Game-Launcher)
- [Security](https://github.com/HELIX-Origin/Universal-Game-Launcher/blob/main/SECURITY.md)
- [License](https://github.com/HELIX-Origin/Universal-Game-Launcher/blob/main/LICENSE)

---

### 📚 Documentation Index
- **Core**: [Home]({{ '/' | relative_url }}) · [Getting Started](Getting-Started.md) · [Organizing Your Library](Organizing-Your-Library.md)
- **Features**: [Supported Launchers](External-Platforms.md) · [Manually Added Local Games](Non-Steam-Games.md) · [Collections](Collections.md) · [Artwork Management](Artwork-Management.md) · [Storage Management](Storage-Management.md) · [Plugin Development](Plugin-Development.md)
- **Reference**: [Metadata Providers](Metadata-Providers.md) · [Settings & API Keys](Settings-and-API-Keys.md) · [Troubleshooting](Troubleshooting.md) · [FAQ](FAQ.md)

---

*[Universal Game Launcher](https://github.com/HELIX-Origin/Universal-Game-Launcher) — a fast, lightweight universal game launcher and library organizer.*
