<div align="center">

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

## 🧩 Features (v1 scope)

- **Read + write Steam collections** — modern Steam collections via the cloud namespace store, plus
  legacy tag import.
- **Manual organizing UI** — multi-select games and add/remove collections, favorites, and hidden.
- **Whole owned library** — sync your entire Steam library (not just installed games) through the
  Steam Web API so everything you own can be organized.
- **External platform import** — detect games installed by GOG, Epic Games, itch.io, Amazon Games,
  Ubisoft Connect, EA, Battle.net, and Rockstar, then import them as non-Steam shortcuts.
- **Advanced filter builder** — Allowed (OR), Required (AND), Excluded (NOT), and text search.
- **AutoCats** — automatic categories from genre, developer, publisher, store flags, tags, release
  year, and user score.
- **Create collections from any sorting option** — one collection per value of the chosen key
  (genre, developer, platform, installed state, playtime, name, …), with a preview before writing.
- **Non-Steam game support** — import and manage `shortcuts.vdf` entries, with artwork from
  SteamGridDB.
- **Metadata providers** — see the table below.

## 🔌 Metadata providers

| Provider | Key required | Purpose |
| --- | --- | --- |
| **Steam Store** (`appdetails`) | ❌ No | Genres, developers, publishers, release dates, feature flags |
| **Steam Web API** | ✅ Yes | Owned games, playtime, player summaries |
| **SteamGridDB** | ✅ Yes | Grids, heroes, logos, icons (especially for non-Steam games) |
| **IGDB (Twitch)** | ✅ Yes | Rich metadata: summary, genres, companies, cover |
| **PCGamingWiki** | ❌ No | Engine, developers, publishers, release dates |
| **Wikipedia** | ❌ No | Article summaries used as a fallback description |

> **Note:** RAWG support is intentionally **not** included (the service is defunct). Steam Web API,
> SteamGridDB, and IGDB are the **only** providers that require an API key.

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

Static checks:

```bash
npm run check                                              # svelte-check
cargo check --manifest-path src-tauri/Cargo.toml           # Rust
cargo test  --manifest-path src-tauri/Cargo.toml           # Rust tests
```

## 🎨 UI direction

The application UI is a **modern dark interface with a glassmorphism style** — frosted
translucent panels, layered depth, and the HELIX Origin obsidian palette (cyan `#48d8f2` /
purple `#b06cff` accents). It ships with a **custom title bar** (frameless window with
platform-aware window controls) for a cleaner, more cohesive look. The full visual design pass
remains tracked as deferred work.

## 🗺️ Status

Early development. **M1 (Foundation)** delivered Steam discovery, library/collection reads and
writes, non-Steam shortcuts, the metadata provider layer, AutoCats, and the app shell.
**M2 (Whole-Library Organizing)** adds the custom title bar, external platform detection and
import, the full owned-library sync, and collection creation from any sorting option.

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
