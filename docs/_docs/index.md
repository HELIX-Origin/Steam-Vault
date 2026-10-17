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
- **Frameless Title Bar Navigation:** Quick switching between Dashboard, Library, Store, Social, Storage, and Settings with 0ms tab latency.
- **Steam & GOG-Style Left Sidebar:** Dedicated strictly to the Library view with instant search, quick filter pills (All, Installed, ★ Favorites), background platform scan trigger (`↻ Scan`), and collapsible platform/console accordions.
- **Authentic Vector SVG Platform & Console Icons:** Custom vector SVGs (`PlatformIcon.svelte`) representing all PC storefronts and retro emulation consoles.
- **Dedicated Game Showcase View:** Rich selected game view with 16:9 hero banner, capsule cover, title typography, prominent **PLAY NOW** launch button, and playtime counters.
- **Consolidated Unified Media Showcase:** Dynamic tabbed showcase providing **🖼️ Screenshots & Art** (with fullscreen lightbox), **🎬 Trailers** (modern Steam Akamai MP4 video streaming), and **🎵 Soundtracks** (ambient game music).
- **Per-Game Customization Modal (`⚙ Manage`):** Full editing of game Title, Short Description, Developers, Publishers, Release Date, Genres, and Custom Artwork (Capsule, Hero, Logo via URL or local file picker).
- **Universal Core Plugins Ecosystem:** High-performance in-app plugins (`platform-universal`, `metadata-universal`, `utility-title-normalizer`, `media-universal`, `utility-deals`, `utility-patchnotes`, `utility-storage`) paired with global helper functions.

Built on **Tauri v2** (Rust backend + SvelteKit frontend), Universal Game Launcher is 100% non-invasive. It never tampers with external store files, never harvests credentials, and stores library data locally in generic SQLite (`database.sqlite`) backed by an in-memory serverless KV tier for sub-millisecond responsiveness.

## 🚀 Key Highlights

- 🎮 **Steam & GOG Left Games List Sidebar** — Instant search, filter pills (All, Installed, Favorites), platform/console accordions with game counts, and collections tree.
- 🧭 **Frameless Title Bar Navigation** — Seamless header navigation: Dashboard, Library, Store, Social, Storage, and Settings.
- 🎯 **Authentic Vector SVG Platform Icons** — Pixel-perfect vector SVGs for all PC storefronts and retro consoles, replacing generic Unicode glyphs.
- 🖼️ **Dedicated Game Showcase View** — 16:9 hero wallpaper banner with dark vignette, title logo, prominent PLAY NOW launch button, and game overview.
- 🎬 **Consolidated Unified Media Showcase** — Tabbed media presentation: Screenshots & Art, Trailers (modern Steam CDN MP4 streaming), and Soundtracks.
- ⚙️ **Per-Game Customization Modal** — Edit Title, Short Description, Developers, Publishers, Release Date, Genres, and Artwork via `⚙ Manage`.
- 🤖 **Automatic Multi-Source Metadata Engine** — Background native Rust cascades across Local Inspector, Libretro, Steam Store, GOG, Wikipedia/Wikidata, and PCGamingWiki.
- 🔤 **Dedicated Title Normalizer** — Automatic clean title resolution for RPG Maker (`System.json`), itch.io slugs, and ROM dump tags.
- 🔒 **Native Launcher Delegation** — Launches, updates, and uninstalls delegate directly to official clients, preserving overlays, cloud saves, friend lists, and achievements.
- 🎁 **Free Games, Deals & Patch Notes** — Plugin-powered feeds for verified 100% free giveaways, major store deals, and official game update notes.
- 🧹 **Settings-Only Cache Controls** — Cache refresh and purge actions live cleanly in **Settings → Storage & Cache**.
- 🗂️ **Custom & Smart Collections** — Create custom categories and evaluate dynamic filter rules with instant preview.
- 💾 **Storage Footprint Explorer** — Inspect drive capacity and game footprints across library volumes with non-blocking workers.
- ⚡ **Zero-Lag Obsidian Purple Theme** — High-contrast dark purple visual theme (#080511 / #0f0920 / #a855f7) with zero `backdrop-filter: blur` performance overhead.
- 💻 **Headless CLI Foundation** — Scriptable `--cli scan`, `scan-roms`, `scaffold-roms`, and `normalize-title` commands.

## 🧭 Getting Around

- **[Getting Started](Getting-Started.md)** — prerequisites, build, and first launch.
- **[Organizing Your Library](Organizing-Your-Library.md)** — the Library showcase view, sidebar filters, unified media showcase, and game launcher.
- **[Supported Launchers & Platforms](External-Platforms.md)** — multi-launcher detection, plugins, and client delegation.
- **[Manually Added Local Games](Non-Steam-Games.md)** — standalone executables, title normalization, custom artwork, and tagging.
- **[Collections](Collections.md)** — creating, editing, and managing collections.
- **[Artwork Management](Artwork-Management.md)** — custom art studio for capsules, heroes, logos, and icons.
- **[Storage Management](Storage-Management.md)** — drive utilization and game storage exploration.
- **[Plugin Development](Plugin-Development.md)** — authoring Universal Core plugins and global helpers.
- **[Available Plugins Catalog](Available-Plugins.md)** — complete directory of Universal Core platform, metadata, utility, and media plugins.
- **[Metadata Providers](Metadata-Providers.md)** — automatic multi-source metadata engine and provider reference.
- **[Settings & API Keys](Settings-and-API-Keys.md)** — configuration reference and storage & cache management.
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
- **Features**: [Supported Launchers](External-Platforms.md) · [Manually Added Local Games](Non-Steam-Games.md) · [Collections](Collections.md) · [Artwork Management](Artwork-Management.md) · [Storage Management](Storage-Management.md) · [Available Plugins](Available-Plugins.md) · [Plugin Development](Plugin-Development.md)
- **Reference**: [Metadata Providers](Metadata-Providers.md) · [Settings & API Keys](Settings-and-API-Keys.md) · [Troubleshooting](Troubleshooting.md) · [FAQ](FAQ.md)

---

*[Universal Game Launcher](https://github.com/HELIX-Origin/Universal-Game-Launcher) — a fast, lightweight universal game launcher and library organizer.*
