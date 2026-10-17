---
layout: wiki
title: Frontend (SvelteKit)
summary: The static SPA, Svelte 5 Runes architecture, SCSS design system, and views
parent: Technical Wiki
nav_order: 5
---

# Frontend (SvelteKit)

The Universal Game Launcher frontend is a high-performance **SvelteKit static SPA** built with **Svelte 5 Runes**, strict **TypeScript**, and **Vite**.

It compiles to static HTML/CSS/JS assets bundled directly into the Tauri desktop application.

---

## 🏗️ SPA Configuration & Architecture

- **Adapter:** `@sveltejs/adapter-static` with client-side routing fallback.
- **Rendering:** Pure client-side SPA (no Node.js SSR). Communicates with Rust backend exclusively through typed Tauri IPC (`invoke`).
- **State Management:** Svelte 5 modern runes (`$state`, `$derived`, `$effect`, `$props`) coupled with cross-view reactive stores (`$games`, `$collections`, `$settings`).
- **Styling Architecture:** Modular SCSS design system (`src/styles/`):
  - `_tokens.scss`: Obsidian dark purple design tokens (`#080511`, `#0f0920`, `#160e2e`, accent `#a855f7` neon violet, secondary `#38bdf8` cyan).
  - High-performance GPU rendering: Zero `backdrop-filter: blur` overhead; hardware-accelerated transforms (`translateZ(0)`).
  - Tightened `.details-grid` and `.details-sidebar` layouts eliminating dead screen space.

---

## 📁 Layout & Route Structure

| Path | Role |
| :--- | :--- |
| `src/routes/+layout.svelte` | App shell: Custom frameless title bar navigation (Dashboard, Library, Store, Social, Storage, Settings) and the Library-dedicated **Steam & GOG Galaxy-style left games list sidebar**. |
| `src/routes/+page.svelte` | **Dashboard**: Quick launch carousel, library statistics, volume utilization snapshot, and plugin-powered free games and deals feeds. |
| `src/routes/library/+page.svelte` | **Library Showcase**: Dedicated selected game showcase featuring 16:9 hero wallpaper, capsule, title logo, prominent PLAY NOW button, consolidated tabbed media showcase, and the `⚙ Manage` per-game customization modal. |
| `src/routes/store/+page.svelte` | **Store**: Multi-storefront game discovery and deals browser delegating securely to official store clients. |
| `src/routes/social/+page.svelte` | **Social**: Unified friends list and activity tracker aggregating status across connected platform plugins. |
| `src/routes/storage/+page.svelte` | **Storage**: Disk capacity usage gauges and game storage footprint explorer across connected drives. |
| `src/routes/settings/+page.svelte` | **Settings & Plugins Manager**: Core preferences, system locale, cache reset, installed platform toggles, and Storage & Cache tab. |
| `src/lib/components/PlatformIcon.svelte` | Authentic vector SVG platform & console icons replacing generic Unicode symbols across all views. |
| `src/lib/components/settings/PluginSettingsDialog.svelte` | Tabbed per-plugin settings configuration dialog for API keys (with password toggle), executable paths, options, and live connection testing. |
| `src/lib/components/WindowControls.svelte` | Frameless window controls (Minimize, Maximize, Close). |
| `src/lib/api.ts` | Strongly typed wrappers around Tauri IPC commands. |
| `src/lib/types.ts` | TypeScript interfaces mirroring backend Rust structs. |

---

## 🎮 Navigation & Interaction Model

```
+-------------------------------------------------------------------------------------------------------------------------+
| [LOGO] Universal Game Launcher   DASHBOARD   LIBRARY   STORE   SOCIAL   STORAGE   SETTINGS                    [_][#][X]   |  <- Custom Title Bar
+------------------------------------+------------------------------------------------------------------------------------+
|  [🔍 Filter games...             ] |  HERO WALLPAPER BANNER (16:9 / 21:9 with dark vignette)                            |
|  [All (142)] [Installed] [★ Favs]  |  +-------------------------------------------------------------------------------+ |
|                                    |  | [Capsule]  [SVG-Icon] TITLE LOGO TYPOGRAPHY                                   | |
|  ▼ STEAM (84)                      |  |            By Developer · Published by Publisher · 2024                      | |
|    • Cyberpunk 2077                |  |            [▶ PLAY NOW]  [★]  [⚙ Manage]   Playtime: 48.2 hrs                 | |
|    • Elden Ring                    |  +-------------------------------------------------------------------------------+ |
|  ▼ GOG GALAXY (28)                 |  UNIFIED MEDIA SHOWCASE                                                            |
|    • The Witcher 3: Wild Hunt      |  [🖼️ Screenshots & Art]   [🎬 Trailers (MP4 Stream)]   [🎵 Soundtracks]           |
|  ▼ EPIC GAMES (18)                 |  +-------------------------------------------------------------------------------+ |
|    • Alan Wake 2                   |  | Media viewer / HTML5 video player / ambient audio player                      | |
|  ▼ STANDALONE / INDIE (12)         |  +-------------------------------------------------------------------------------+ |
|    • Relicts of Aeson              |  GAME DETAILS & OVERVIEW                                                           |
|    • My Futa Family                |  Full description, community tags, store links.                                    |
+------------------------------------+------------------------------------------------------------------------------------+
```

---

## 🔌 Talking to the Backend

All IPC calls are wrapped in strongly typed functions in `src/lib/api.ts`:
```ts
export const api = {
  loadLibrary: () => invoke<Game[]>("load_library"),
  getCachedGameMetadata: (appId: number) => invoke<GameMetadata | null>("get_cached_game_metadata", { appId }),
  fetchExternalJson: (url: string) => invoke<any>("fetch_external_json", { url }),
  launchGame: (appId: number, launchUri?: string | null, exe?: string | null, platform?: string | null) =>
    invoke<boolean>("launch_game", { appId, launchUri: launchUri ?? null, exe: exe ?? null, platform: platform ?? null }),
};
```
Tauri automatically maps camelCase parameters in TypeScript to snake_case parameters in Rust.

---

## ➡️ Next Steps

- [Backend Rust Architecture](Backend-Rust.md) — Rust core and IPC command handlers.
- [Plugins Architecture](Plugins-Architecture.md) — Universal Core plugins framework.
- [Security Model](Security.md) — non-invasive security and permissions.
