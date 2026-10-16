<div align="center">

<img src="./static/universal-game-launcher.svg" alt="Universal Game Launcher Banner" width="100%" />

# Universal Game Launcher

**A fast, lightweight, privacy-first universal game launcher and library organizer — built with Tauri v2.**

Organize, categorize, filter, and launch your entire multi-platform game library from a single unified desktop application without scraping or credentials.

</div>

---

## ✨ Overview

**Universal Game Launcher** aggregates your installed games and owned storefront libraries into a beautiful, high-performance desktop hub. It connects directly with your installed official launchers — including **Steam**, **Epic Games Store**, **GOG Galaxy**, **Ubisoft Connect**, **EA App**, **Xbox / Microsoft Store**, **Battle.net**, **itch.io**, and **Amazon Games**.

It serves as a clean-room, modernized revival of the library organization philosophies of [Depressurizer](https://github.com/rallion/depressurizer): it automatically discovers games from all installed launchers with full support for manually adding local games, lets you build and edit custom collections, filter with an advanced Allowed / Required / Excluded builder, auto-categorize by genre, developer, publisher, and tags, and manage local storage across drives.

Built on **Tauri v2** (Rust backend, SvelteKit + Svelte 5 frontend), Universal Game Launcher delivers instant launch performance, minimal memory footprint, and full offline functionality with zero scraping.

---

## 🧩 Features

- **Unified Multi-Store Library Grid** — Responsive capsule cover grid serving as your primary gaming cockpit with artwork covers, install badges, platform badges, playtime, and one-click launch controls.
- **Native Launcher Delegation Architecture** — Installation, launching, updating, and uninstallation delegate directly to installed official client protocols (`steam://`, `com.epicgames.launcher://`, `goggalaxy://`, `uplay://`). Never handles user credentials while preserving official client overlays, cloud save sync, friend lists, playtime tracking, and achievement unlocks.
- **Collapsible Filter & Sort Drawer** — Instant sort options (Name A-Z/Z-A, Playtime high/low, Last played, Size on disk) and granular filters across installation status, official platforms, custom collections, and favorites/hidden toggles.
- **Per-Game Management & Media Gallery** — Interactive 10-image media gallery with full-screen lightbox modal, quick-access links (Store, PCGamingWiki, SteamDB, ProtonDB, Wikipedia), detailed metadata cards (developer, publisher, release date, genres, tags, review scores), and per-game management tools (file explorer, platform tagging, launch URI inspection, uninstaller trigger).
- **Cross-Platform Free Games, Deals & Patch Notes Feed** — Real-time dashboard sub-panel tracking verified 100% free game giveaways, store specials/discounts across Steam, Epic, and GOG, and official Steam News patch notes for installed titles.
- **Multi-Drive Storage Manager** — Storage analytics dashboard with drive capacity/usage gauges and Steam Library Folder management (add/remove library folders, view registered paths and disk footprint, move games between drives with safe transactional backups).
- **Advanced Collection & AutoCats Engine** — Allowed (OR), Required (AND), Excluded (NOT) term builder with name search, named preset saving/loading, and native Valve `filterSpec` smart collections with live re-evaluation.
- **Custom Artwork Slot Manager** — Inspect, replace, and auto-fetch custom grid covers, hero banners, logos, and icons across all your games and manually added local titles, with SteamGridDB integration and local file import.
- **Modular Drop-In Plugin Engine** — Extensible architecture where built-in platforms, metadata providers, and community plugins are loaded dynamically from self-contained packages (`index.json`) without hardcoding them into the core launcher. Support for GitHub-hosted repository entrypoints.
- **Hand-Rolled Reactive i18n Localization** — Native reactive UI translations supporting 28+ languages with zero runtime localization dependencies and Arabic RTL layout support.
- **Animated Splash Screen & Obsidian Glassmorphism UI** — Dark futuristic purple vector aesthetic with smooth startup splash transition, frosted glass panels, atmospheric glow accents, and custom frameless titlebar.

---

## 🔌 Metadata Providers

| Provider | Key Required | Purpose |
| --- | --- | --- |
| **Steam Store** (`appdetails`) | ❌ No | Genres, developers, publishers, release dates, feature flags, covers, screenshots |
| **Steam Store Search** | ❌ No | Match standalone & indie titles to official Steam app IDs |
| **GOG Store & Galaxy DB** | ❌ No | Official GOG covers, heroes, and metadata via local SQLite galaxy DB |
| **Epic Games Manifests** | ❌ No | Title, install locations, app IDs, and official launch options |
| **Xbox / Windows Store AppModel** | ❌ No | Registry and package manifest discovery with zero scraping |
| **PCGamingWiki** | ❌ No | Engine, developers, publishers, release dates, taxonomy genres, box art |
| **Wikipedia** | ❌ No | Infobox wikitext parser & high-resolution `pageimages` box art |
| **VNDB** | ❌ No | Visual novel developers, publishers, release dates, tags, covers, Japanese titles |
| **Local Game Sniffer** | ❌ No | itch.io manifests, Ren'Py, RPG Maker, and PE VersionInfo headers |
| **SteamGridDB** | ✅ Yes (Optional) | Grids, heroes, logos, icons (especially for manually added local games) |
| **IGDB (Twitch)** | ✅ Yes (Optional) | Rich supplementary metadata: summaries, genres, companies, cover |
| **Steam Web API** | ✅ Yes (Optional) | Owned games list, playtime, player summaries |

> 💡 **Keyless Architecture**: Universal Game Launcher works fully out of the box without requiring API keys or user logins. External providers are used primarily for manually added non-store titles and supplementary high-resolution artwork.

---

## 🔒 Privacy & Security

- **100% On-Device**: No telemetry, no usage analytics, no user accounts, and no background tracking.
- **Zero Credential Handling**: The app never prompts for, stores, transmits, or handles user passwords, payment information, or account tokens. All authentication and transactions remain strictly delegated to official storefront clients.
- **Local Persistence**: All settings, caches, and custom tags remain stored locally on your machine in `database.sqlite`.

---

## 🛠️ Tech Stack

- **Backend:** Rust 2021 / Tauri v2 (`src-tauri/`)
- **Frontend:** SvelteKit + Svelte 5 (Runes) + TypeScript + Vite (`src/`)
- **Database:** SQLite 3 in WAL mode via `rusqlite`
- **Cache Tier:** In-memory key-value cache
- **IPC:** Strongly-typed Tauri commands (camelCase JSON)

---

## 🚀 Building & Running

### Prerequisites

- **Rust** (stable toolchain)
- **Node.js 18+**
- Platform [Tauri system dependencies](https://tauri.app/start/prerequisites/) (WebView2 on Windows, WebKitGTK on Linux)
- **Windows System Runtimes:**
  - [Microsoft Visual C++ 2015–2022 Redistributable (x64)](https://aka.ms/vs/17/release/vc_redist.x64.exe) — required for native MSVC dependencies and bundled SQLite operations.

```bash
npm install
npm run tauri:dev     # development
npm run tauri:build   # production bundle
```

Both commands run through the logging wrappers in [`scripts/`](./scripts): output streams live to the console **and** is written to a timestamped transcript in [`logs/`](./logs) (`<type>-<YYYY-MM-DD_HH-MM-SS>.log`, never overwritten).

---

### Tests & Verification

Every check runs through an agent-friendly test runner that streams output live, writes timestamped transcripts to `logs/`, enforces per-step timeouts, and prints a compact PASS/FAIL summary:

```bash
npm test                 # cargo check + cargo test + svelte-check
npm run test:rust        # cargo test only
npm run test:frontend    # svelte-kit sync + svelte-check only
npm run verify           # check + tests + svelte-check + vite build
npm run test:all         # fmt + clippy + check + tests + svelte-check + build
```


---

### ✍️ Code Signing

Both local development builds and release packages support opt-in code signing:

```bash
npm run sign             # Signs discovered platform artifacts (.exe/.msi, .app/.dmg, .deb/.AppImage)
npm run sign:local       # Alias for local signing
npm run sign -- --dry-run
```

---

## 📄 License

BSD-3-Clause. See [`LICENSE`](./LICENSE).

## 🙏 Credits

Inspired by **Depressurizer** by Andrew Martin (rallion) and contributors. Universal Game Launcher is an independent clean-room implementation and is not affiliated with Valve, Microsoft, Epic Games, or Depressurizer.
