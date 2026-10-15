<div align="center">

<img src="./assets/steam_banner.jpg" alt="Steam Vault Banner" width="100%" />

# Steam Vault

**A clean-room, cross-platform revival of [Depressurizer](https://github.com/rallion/depressurizer) — built with Tauri v2.**

Organize, categorize, filter, and auto-manage your Steam library without the brittleness of scraping.

</div>

---

## ✨ Overview

Steam Vault reads your **local Steam data** and lets you organize your real Steam library directly.
It is a spiritual successor to Depressurizer: it imports your installed games and non-Steam
shortcuts, lets you build and edit collections (categories), filter with a Depressurizer-style
Allowed / Required / Excluded builder, and auto-categorize by genre, developer, publisher, tags,
and feature flags.

Unlike Depressurizer, Steam Vault is a modern, cross-platform **Tauri v2** application (Rust backend,
SvelteKit frontend) that targets **stable, documented data sources** rather than fragile HTML scraping.

## 🧩 Features

- **Redesigned Library Grid & Launcher** — Vertical game capsule grid view as the central launcher hub featuring responsive artwork covers, install status badges, platform icons, and one-click launch controls.
- **Collapsible Right Sidebar** — Instant sort options (Name A-Z/Z-A, Playtime high/low, Last played, Size on disk) and granular filters (installation status, platform counts across Steam, GOG, Epic, Itch, Ubisoft, EA, Battle.net, Amazon, Xbox, Rockstar, Emulators, Other, collection dropdown, and favorites/hidden toggles).
- **Extensive Per-Game Management Sub-Page** — Interactive 10-image media gallery with full-screen lightbox modal, meta links card (Store, PCGamingWiki, SteamDB, ProtonDB, Wikipedia, Community Catalogue), comprehensive details cards (developer, publisher, release date, genres, tags, metacritic/user scores, platform ID, file paths), playtime tracking, Play/Install button (via `steam://run/{app_id}`, `steam://install/{app_id}`, and native fallbacks), and Manage button launching a per-game management sub-window modal (folder explorer, platform tagging, launch URI inspection, uninstaller trigger).
- **Streamlined Storage Statistics** — Pure storage analytics dashboard with drive capacity/usage gauges and Steam Library Folder management (add/remove library folders, view registered paths and disk footprint).
- **Streamlined Organize Scopes List** — Multi-select scope list (Genre, Developer, Publisher, Release Year, Platform) with custom Depressurizer-style rule builder to organize the Steam library without clutter.
- **Read + write Steam collections** — modern Steam collections via the cloud namespace store, plus legacy tag import.
- **Whole owned library** — sync your entire Steam library (not just installed games) through the Steam Web API and local GOG/Epic manifests so everything you own can be organized and launched.
- **External platform & emulator import** — detect games installed by GOG (including full read-only SQLite galaxy database querying), Epic Games, itch.io, Amazon Games, Ubisoft Connect, EA, Battle.net, Xbox/Windows Store, and Rockstar, as well as ROMs across 11 popular emulators with tailored launch parameters.
- **Console platform detection & shortcut tagging** — automatically resolves specific console hardware from ROM extensions and emulator paths, displays console badges, and writes console platforms directly into Steam's binary `shortcuts.vdf` `tags` for native category grouping.
- **Embedded Community Game Catalogue** — precompiled SQLite seed database of curated multi-platform games (`catalog_seed.bin`) for instant offline title and metadata lookup, with online sync from GitHub releases (`Steam-Vault-Catalogue`) and archive export/import.
- **Advanced filter builder & dynamic smart collections** — Allowed (OR), Required (AND), Excluded (NOT), text search, named preset saving/loading, and Valve `filterSpec` smart collections with one-click live re-evaluation.
- **AutoCats** — automatic categories from genre, developer, publisher, store flags, tags, release year, and user score, with multi-entity splitting and platform fallbacks.
- **Embedded SQLite metadata cache** — high-performance, zero-dependency on-disk metadata cache powered by `rusqlite` bundled from source in WAL mode with automatic legacy JSON migration.
- **Hand-rolled i18n localization** — native reactive UI translations across all 30 official Steam languages with zero third-party dependencies and RTL support.
- **Artwork management** — replace or auto-fetch the grid, hero, logo, and icon art for any game, pulling from SteamGridDB (by app id, or by name for shortcuts) or from local metadata and file caches.

## 🔌 Metadata providers

| Provider | Key required | Purpose |
| --- | --- | --- |
| **Community Game Catalog** | ❌ No | Curated offline database with multi-platform OS & console hardware |
| **Steam Store** (`appdetails`) | ❌ No | Genres, developers, publishers, release dates, feature flags |
| **Steam Store Search** | ❌ No | Match non-Steam & indie titles to official Steam app IDs |
| **Steam Web API** | ✅ Yes | Owned games, playtime, player summaries |
| **SteamGridDB** | ✅ Yes | Grids, heroes, logos, icons (especially for non-Steam games) |
| **IGDB (Twitch)** | ✅ Yes | Rich metadata: summary, genres, companies, cover |
| **VNDB** | ❌ No | Visual novel developers, publishers, release dates, tags, covers |
| **PCGamingWiki** | ❌ No | Engine, developers, publishers, release dates |
| **Local Game Sniffer** | ❌ No | itch.io manifests, Ren'Py, RPG Maker, and PE VersionInfo headers |
| **Wikipedia** | ❌ No | Infobox wikitext parser for console games, developers, publishers, genres, release year, and summaries |

> **Note:** RAWG support is intentionally **not** included (the service is defunct).
>
> 💡 **Architectural Direction:** Steam Vault is progressively migrating metadata resolution to rely **solely on official sources** (local launcher manifests, GOG Galaxy DB, Epic manifests, Xbox packages, and official store APIs) and our open **Community Game Catalogue** (`Steam-Vault-Catalogue`). Furthermore, even the **Steam Web API key is planned to be completely optional**, as Steam Vault is designed to read the user's authenticated local Steam installation directly (`appinfo.vdf`, `packageinfo.vdf`, `localconfig.vdf`) to discover all owned games from their authenticated account—mirroring the zero-friction, keyless architecture established across GOG, Epic, Xbox, and itch.io.

## 🔒 Privacy

- 100% on-device. No telemetry, no accounts, no forced network calls.
- API keys are stored only in your local config directory and are never committed.

## 🛠️ Tech stack

- **Backend:** Rust / Tauri v2 (`src-tauri/`)
- **Frontend:** SvelteKit + Svelte 5 + TypeScript + Vite (`src/`)
- **IPC:** strongly-typed Tauri commands (camelCase JSON)

## 🚀 Building & running

Prerequisites: **Rust** (stable), **Node.js 18+**, and the [Tauri system dependencies](https://tauri.app/start/prerequisites/).

```bash
npm install
npm run tauri:dev     # development
npm run tauri:build   # production bundle
```

Both commands run through the logging wrappers in [`scripts/`](./scripts): output streams live to
the console **and** is written to a timestamped transcript in [`logs/`](./logs)
(`<type>-<YYYY-MM-DD_HH-MM-SS>.log`, never overwritten). Use `npm run tauri:dev:raw` /
`npm run tauri:build:raw` to bypass logging, or `npm run log -- <type> <command> [args…]` to log
anything else, e.g.:

```bash
npm run log -- check npm run check
npm run log -- rust  cargo test --manifest-path src-tauri/Cargo.toml
```

### Tests & verification

Every check runs through an agent-friendly runner that streams output live, writes a timestamped
transcript per step to [`logs/`](./logs), **closes stdin** (so no step can block waiting for
input), enforces a **per-step timeout** (default 15 min), and prints a compact PASS/FAIL summary
with a reliable exit code. Run it instead of invoking the tools by hand:

```bash
npm test                 # default: cargo check + cargo test + svelte-check
npm run test:rust        # cargo test only
npm run test:frontend    # svelte-kit sync + svelte-check only
npm run verify           # check + tests + svelte-check + vite build
npm run test:all         # fmt + clippy + check + tests + svelte-check + build
```

Under the hood it is `node scripts/test.mjs [suite …]`; add `--list` to see the suites, `-t 300`
for a different timeout in seconds, and `-- <args>` to forward extra arguments to the `cargo`
steps (for example `npm run test:rust -- some_test_name`).

## 🎨 UI direction

The application UI is a **modern dark interface with a glassmorphism style** — frosted
translucent panels, layered depth, and the HELIX Origin obsidian palette (cyan `#48d8f2` /
purple `#b06cff` accents). It ships with a **custom title bar** (frameless window with
platform-aware window controls) for a cleaner, more cohesive look. The full visual design pass
remains tracked as deferred work.

## 🗺️ Status
 
- **v1.0.0 (Production Launch)** delivered Steam discovery, library/collection reads and writes, non-Steam shortcuts, whole owned-library sync, custom titlebar, external platform detection, storage manager, custom artwork manager, 30-language hand-rolled i18n localization, dynamic smart collections (`filterSpec`), embedded SQLite cache, and modern obsidian dark glassmorphism design.
- **v1.1.0 (Catalog & Console Expansion)** adds the embedded Community Game Catalog seed (`catalog_seed.bin`) with online synchronization, high-precision console platform determination with UI badges, automated `shortcuts.vdf` binary console tagging, and the keyless Wikipedia Video Game & Console metadata infobox parser.

## 📖 Documentation

Full guides and technical reference live on the documentation site:

- **Site:** <https://helix-origin.github.io/Steam-Vault/>
- **User guides:** <https://helix-origin.github.io/Steam-Vault/docs/>
- **Technical wiki:** <https://helix-origin.github.io/Steam-Vault/wiki/>

The site is built from the [`docs/`](./docs) folder in this repository using the
[`helix-docs`](https://github.com/HELIX-Origin/helix-docs) Jekyll theme.

## 📄 License

BSD-3-Clause. See [`LICENSE`](./LICENSE).

## 🙏 Credits

Inspired by **Depressurizer** by Andrew Martin (rallion) and contributors. Steam Vault is an
independent clean-room implementation and is not affiliated with Valve or Depressurizer.
