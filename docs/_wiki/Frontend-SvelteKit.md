---
layout: wiki
title: Frontend (SvelteKit)
summary: The static SPA, stores, and typed API layer
parent: Steam Vault Technical Wiki
nav_order: 5
---

# Frontend (SvelteKit)

The frontend is a **SvelteKit** application in static SPA mode, built with **Svelte 5**, strict
**TypeScript**, and **Vite**. It is compiled to static assets that Tauri embeds in the binary.

## 🏗️ SPA configuration

- `@sveltejs/adapter-static` with a fallback page, so the app runs as a single-page application.
- Server-side rendering is disabled; the UI is a client-only shell that talks to Rust over IPC.

## 📁 Layout

| Path | Role |
| :--- | :--- |
| `src/app.css` | The global theme: dark surfaces, glass panels, buttons, pills, and form styles. |
| `src/lib/types.ts` | TypeScript mirrors of the Rust serde models. |
| `src/lib/api.ts` | Typed `invoke` wrappers, one per command, plus an error-message helper. |
| `src/lib/stores.ts` | Writable stores: settings, providers, install, games, collections, Steam-running, owned library, platform scan, loading, and error. |
| `src/lib/platform.ts` | Platform detection for window-control rendering. |
| `src/lib/components/WindowControls.svelte` | Custom title-bar controls (macOS traffic lights / Windows buttons). |
| `src/routes/+layout.svelte` | The app shell: custom title bar, sidebar navigation, and status. |
| `src/routes/+page.svelte` | Dashboard. |
| `src/routes/library/+page.svelte` | Library. |
| `src/routes/storage/+page.svelte` | Storage: libraries, drives, and moving games between them. |
| `src/routes/artwork/+page.svelte` | Artwork: custom grid, hero, logo, and icon art for any game. |
| `src/routes/collections/+page.svelte` | Collections. |
| `src/routes/platforms/+page.svelte` | External-platform detection and import. |
| `src/routes/organize/+page.svelte` | Create collections from a sorting option, with the Allowed / Required / Excluded filter builder. |
| `src/routes/settings/+page.svelte` | Settings. |

## 🔌 Talking to the backend

The API layer wraps each command in a typed function, for example:

```ts
export const api = {
  loadLibrary: (accountId3?: string) =>
    invoke<Game[]>("load_library", { accountId3: accountId3 ?? null }),
  // ...
};
```

Because Tauri maps camelCase JavaScript keys to snake_case Rust parameters, the TypeScript surface
stays idiomatic while matching the Rust signatures.

## 🧩 Reactivity

Components use Svelte 5 in **legacy (non-runes) mode** — `let` declarations, props, and reactive
`$:` statements — while shared `writable` stores carry cross-view state (the loaded library,
collections, and settings). Pages read stores with the `$` prefix and update them after each command.

## 🎨 Theming

The interface uses a modern **dark** theme with **glassmorphism**: translucent panels with
backdrop blur over a layered obsidian background, accented in cyan and purple. Styling is driven by
CSS custom properties in `app.css` plus utility classes, so the whole look can be retuned from one
place. The window is **frameless** with a custom, draggable title bar and platform-aware window
controls. A full design pass is on the roadmap.

## ➡️ Next

- [Backend (Rust)](Backend-Rust.md)
- [Development & Contributing](Development-and-Contributing.md)
