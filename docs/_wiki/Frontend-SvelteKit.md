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

---

## 📁 Layout & Route Structure

| Path | Role |
| :--- | :--- |
| `src/routes/+layout.svelte` | App shell: Custom frameless title bar navigation (Dashboard, Library, Store, Social, Storage, Plugins, Settings) and the Library-dedicated **Steam & GOG Galaxy-style left games list sidebar**. |
| `src/routes/+page.svelte` | **Dashboard**: Quick launch carousel, library statistics, volume utilization snapshot, and plugin-powered free games and deals feeds. |
| `src/routes/library/+page.svelte` | **Library Showcase**: Dedicated selected game showcase featuring 16:9 hero wallpaper, capsule, title logo, big prominent PLAY NOW button, 10-screenshot gallery with lightbox, metadata enhancement workflows, and custom artwork studio. |
| `src/routes/store/+page.svelte` | **Store**: Multi-storefront game discovery and deals browser delegating securely to official store clients. |
| `src/routes/social/+page.svelte` | **Social**: Unified friends list and activity tracker aggregating status across connected platform plugins. |
| `src/routes/storage/+page.svelte` | **Storage**: Disk capacity usage gauges and game storage footprint explorer across connected drives. |
| `src/routes/settings/+page.svelte` | **Settings & Plugins Manager**: Core preferences, system locale, cache reset, and installed plugin toggles. |
| `src/lib/components/settings/PluginSettingsDialog.svelte` | Tabbed per-plugin settings configuration dialog for API keys (with password toggle), executable paths, options, and live connection testing. |
| `src/lib/components/WindowControls.svelte` | Frameless window controls (Minimize, Maximize, Close). |
| `src/lib/api.ts` | Strongly typed wrappers around Tauri IPC commands. |
| `src/lib/types.ts` | TypeScript interfaces mirroring backend Rust structs. |

---

## 🎮 Navigation & Interaction Model

```
+-------------------------------------------------------------------------------------------------------------------------+
| [LOGO] Universal Game Launcher   DASHBOARD   LIBRARY   STORE   SOCIAL   STORAGE   PLUGINS   SETTINGS            [_][#][X]   |  <- Custom Title Bar
+------------------------------------+------------------------------------------------------------------------------------+
|  [🔍 Filter games...             ] |  HERO WALLPAPER BANNER (16:9 / 21:9 with dark vignette)            |
|  [All (142)] [Installed] [★ Favs]  |  +---------------------------------------------------------------+ |
|                                    |  | [Capsule]  TITLE LOGO TYPOGRAPHY                              | |
|  ▼ STEAM (84)                      |  |            By Developer · Published by Publisher · 2024       | |
|    • Cyberpunk 2077                |  |            [▶ PLAY NOW]  [★] [⚙ Manage]   Playtime: 48.2 hrs  | |
|    • Elden Ring                    |  +---------------------------------------------------------------+ |
|  ▼ GOG GALAXY (28)                 |  MEDIA GALLERY (10 Screenshots with Fullscreen Lightbox)           |
|    • The Witcher 3: Wild Hunt      |  [Img 1] [Img 2] [Img 3] [Img 4] [Img 5]                           |
|  ▼ EPIC GAMES (18)                 |  GAME DETAILS & OVERVIEW                                           |
|    • Alan Wake 2                   |  Full description, community tags, store links.                    |
|  ▼ STANDALONE / INDIE (12)         |  CUSTOM ARTWORK STUDIO                                             |
|    • Custom Game                   |  Portrait Capsule · Hero Wallpaper · Transparent Logo · Icon       |
+------------------------------------+--------------------------------------------------------------------+
```

---

## 🔌 Talking to the Backend

All IPC calls are wrapped in strongly typed functions in `src/lib/api.ts`:
```ts
export const api = {
  loadLibrary: () => invoke<Game[]>("load_library"),
  launchGame: (appId: number, launchUri?: string | null, exe?: string | null, platform?: string | null) =>
    invoke<boolean>("launch_game", { appId, launchUri: launchUri ?? null, exe: exe ?? null, platform: platform ?? null }),
  // ...
};
```
Tauri automatically maps camelCase parameters in TypeScript to snake_case parameters in Rust.

---

## ➡️ Next Steps

- [Backend Rust Architecture](Backend-Rust.md) — Rust core and IPC command handlers.
- [Plugins Architecture](Plugins-Architecture.md) — in-app modular plugin system.
- [Security Model](Security.md) — non-invasive security and permissions.
