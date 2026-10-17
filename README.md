<div align="center">

<img src="./static/universal-game-launcher.svg" alt="Universal Game Launcher Banner" width="100%" />

# Universal Game Launcher

<p align="center">
  <a href="https://docs.helix-origin.club/universal-game-launcher/"><img src="https://img.shields.io/badge/Docs-docs.helix--origin.club-brightgreen" alt="Documentation" height="20" /></a>
  <a href="https://wiki.helix-origin.club/universal-game-launcher/"><img src="https://img.shields.io/badge/Wiki-wiki.helix--origin.club-brightgreen" alt="Technical Wiki" height="20" /></a>
  <a href="https://opensource.org/licenses/BSD-3-Clause"><img src="https://img.shields.io/badge/License-BSD--3--Clause-blue.svg" alt="License: BSD-3-Clause" height="20" /></a>
  <a href="https://tauri.app"><img src="https://img.shields.io/badge/Tauri-v2-blue?logo=tauri" alt="Tauri v2" height="20" /></a>
  <a href="https://svelte.dev"><img src="https://img.shields.io/badge/Svelte-5-orange?logo=svelte" alt="Svelte 5" height="20" /></a>
</p>

**A fast, lightweight, privacy-first universal game launcher and library organizer — built with Tauri v2.**

Organize, categorize, filter, and launch your entire multi-platform game library from a single unified desktop application without scraping or credentials.

</div>


---

## ✨ Overview

**Universal Game Launcher** aggregates your installed games and owned storefront libraries into a beautiful, high-performance desktop hub. It connects directly with your installed official launchers — including **Steam**, **Epic Games Store**, **GOG Galaxy**, **Ubisoft Connect**, **EA App**, **Xbox / Microsoft Store**, **Battle.net**, **itch.io**, and **Amazon Games**.

It serves as a clean-room, modernized revival of the library organization philosophies of [Depressurizer](https://github.com/rallion/depressurizer): it automatically discovers games from all installed launchers with full support for manually adding local games, lets you build and edit custom collections, filter with an advanced Allowed / Required / Excluded builder, auto-categorize by genre, developer, publisher, and tags, and manage local storage across drives.

Built on **Tauri v2** (Rust backend, SvelteKit + Svelte 5 frontend), Universal Game Launcher delivers instant launch performance, minimal memory footprint, and full offline functionality with zero scraping.

---

## 📖 Documentation & Technical Wiki

Comprehensive guides, plugin development manuals, and technical architecture specifications are hosted at:
- **Documentation**: [docs.helix-origin.club/universal-game-launcher](https://docs.helix-origin.club/universal-game-launcher/)
- **Technical Wiki**: [wiki.helix-origin.club/universal-game-launcher](https://wiki.helix-origin.club/universal-game-launcher/)

---

## 🧩 Features

- **Steam & GOG Left Games List Sidebar** — Dedicated to the Library view, featuring instant real-time search, quick filter pills (All, Installed, ★ Favorites), platform scan trigger (`↻ Scan`), and collapsible platform and console accordions with live game counts.
- **Authentic Vector SVG Platform & Console Icons** — Pixel-perfect vector SVGs for all PC storefronts and retro emulation consoles (`PlatformIcon.svelte`), eliminating generic glyphs.
- **Dedicated Game Showcase View** — Rich selected game cockpit featuring high-resolution 16:9 hero wallpaper banner with dark vignette, portrait capsule, title typography, prominent PLAY NOW launch button, playtime metrics, and game details.
- **Consolidated Unified Media Showcase** — Streamlined media section with dynamic tabs: **🖼️ Screenshots & Art** (with fullscreen lightbox), **🎬 Trailers** (streaming modern Steam Akamai MP4 videos), and **🎵 Soundtracks** (ambient theme playback).
- **Per-Game Customization Modal (`⚙ Manage`)** — Full in-app customization of game Title, Short Description, Developers, Publishers, Release Date, Genres, and Custom Artwork (Capsule, Hero, Logo via URL or local file picker).
- **Automatic Multi-Source Metadata Engine** — Background native Rust cascades across Local Inspector, Libretro Thumbnails CDN, Steam Store, GOG Catalog v1, Wikipedia/Wikidata MediaWiki API, PCGamingWiki Cargo API, and ProtonDB with zero dummy strings.
- **Dedicated Universal Title Normalizer** — Resolves RPG Maker `System.json` titles (e.g. `relicts-auf-aeson` → `Relicts of Aeson`), concatenated lowercase slugs (`myfutafamily` → `My Futa Family`), and ROM dump tags.
- **Native Launcher Delegation Architecture** — Installation, launching, updating, and uninstallation delegate directly to installed official client protocols (`steam://`, `com.epicgames.launcher://`, `goggalaxy://`, `uplay://`). Never handles user credentials while preserving official client overlays, cloud save sync, friend lists, playtime tracking, and achievement unlocks.
- **Settings-Only Cache Controls** — Keeps the Library view clutter-free; cache refresh (`↻ Refresh Metadata Cache`) and cache purge (`🗑 Purge Data Cache`) live strictly in **Settings → Storage & Cache**.
- **Cross-Platform Free Games, Deals & Patch Notes Feed** — Real-time dashboard panel tracking verified 100% free game giveaways, store specials/discounts across major storefronts, and official game patch notes.
- **Multi-Drive Storage Footprint Analyzer** — Storage analytics cockpit with drive capacity/usage gauges and non-blocking background disk space calculation.
- **Custom Collections & Smart Categories** — Build custom collections, tag favorites, and evaluate dynamic filter rules evaluated locally with 0ms in-memory cache synchronization.
- **Universal Core Plugins Framework** — Decoupled architecture where platform integrations (`platform-universal`), metadata providers (`metadata-universal`), title normalizer (`utility-title-normalizer`), and media engines (`media-universal`) operate as unified core plugins paired with global helper modules.
- **Hand-Rolled Reactive i18n Localization** — Native reactive UI translations supporting 30 languages with zero runtime localization dependencies and Arabic RTL layout support.
- **Headless CLI Interface (`--cli`)** — Built-in command-line interface supporting `--cli scan`, `scan-roms`, `scaffold-roms`, and `normalize-title`.
- **Animated Splash Screen & Obsidian Purple Theme** — High-contrast dark purple visual identity (`#080511` / `#140d28` / `#a855f7`) with hardware-accelerated transforms and zero `backdrop-filter: blur` performance overhead.

---

## 🔌 Data Providers & Metadata Cascade

Universal Game Launcher automatically enriches metadata via the **Universal Metadata Engine** (`metadata-universal`):
1. **Local Inspector:** Inspects local game directories, RPG Maker `System.json` (`"gameTitle"`), itch.io receipt manifests, and `rom.ini` files.
2. **Libretro CDN:** Downloads authentic retro console box art (`Named_Boxarts`), title screens (`Named_Titles`), snaps, and logos.
3. **Steam Store API:** Official store details, reviews, 1080p screenshots, and modern Akamai CDN MP4 video trailers (`movie_max.mp4`).
4. **GOG Catalog v1:** Official GOG covers, heroes, and DRM-free summaries.
5. **Wikipedia & Wikidata API:** Encyclopedic synopses and metadata taxonomy properties (`P178`, `P123`, `P136`, `P577`).
6. **PCGamingWiki Cargo API:** Engine info, architecture flags, FOV fixes, and save directory locations.
7. **ProtonDB API:** Linux & Steam Deck compatibility ratings (`Native`, `Platinum`, `Gold`) and launch flags.
8. **SteamGridDB (Optional):** Community vertical capsules (600×900), heroes, logos, and square icons.
9. **IGDB (Optional):** Twitch IGDB game taxonomy, franchise links, developer studios, and critic ratings.

> 💡 **Keyless Architecture**: Universal Game Launcher works fully out of the box without requiring API keys or user logins. Background cascades run automatically, and custom overrides persist locally in `database.sqlite`.

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
npm run sign:windows     # Signs Windows artifacts (SignTool)
npm run sign:linux       # Signs Linux packages (GPG)
npm run sign:macos       # Signs macOS .app and .dmg bundles (codesign + notarize)
npm run sign -- --dry-run
```

---

## 📄 License

BSD-3-Clause. See [`LICENSE`](./LICENSE).

## 🙏 Credits

Inspired by **Depressurizer** by Andrew Martin (rallion) and contributors. Universal Game Launcher is an independent clean-room implementation and is not affiliated with Valve, Microsoft, Epic Games, or Depressurizer.
