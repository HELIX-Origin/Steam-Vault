---
layout: docs
title: Getting Started
summary: Prerequisites, building Steam Vault, and your first launch
parent: Documentation Home
nav_order: 2
---

# Getting Started

This page walks you through building **Steam Vault** and running it for the first time.

## 📦 1. Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Node.js](https://nodejs.org/) 18 or newer
- The [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform
  (on Windows this is the WebView2 runtime, which ships with Windows 10/11)

## 🛠️ 2. Build and run

From the repository root:

```bash
npm install
npm run tauri:dev      # development
npm run tauri:build    # production bundle
```

> **Note:** The Tauri build embeds the compiled frontend, so a fresh clone must run the frontend
> build before a bare `cargo check`. Use the npm scripts (which handle ordering) or run
> `npm run build` first.

## 🏁 3. First launch

When the app opens, it shows the **Dashboard** and tries to detect your Steam installation
automatically.

1. Confirm the **Steam folder** on the Dashboard or in **Settings**. If detection fails, point it
   at your Steam directory manually (for example `C:\Program Files (x86)\Steam`).
2. Choose the **account** you want to organize.
3. Optionally add **API keys** (Steam Web API, SteamGridDB, IGDB) in **Settings** to unlock the
   matching metadata providers.
4. Open the **Library** view to load your games.

## 🔒 4. Writing changes safely

Steam Vault only writes when it is safe to do so:

- **Close Steam completely** before saving changes. Writes are refused while Steam is running to
  avoid sync conflicts.
- Every write is preceded by a **timestamped backup**, so changes can be undone from the app data
  directory.

See [Collections](Collections.md) for the details of what gets written and where.

## 🧪 5. Verify your setup

Both the frontend and backend have type checks and tests you can run locally. The quickest way is the
project's test runner, which runs each step directly, logs it to `logs/`, and applies a timeout so a
stuck command can never hang your session:

```bash
npm test                                                 # Rust + Svelte/TypeScript checks & tests
npm run verify                                           # the above, plus a production build
```

The raw toolchain commands still work if you prefer them:

```bash
npm run check                                            # Svelte/TypeScript
cargo check --manifest-path src-tauri/Cargo.toml         # Rust
cargo test  --manifest-path src-tauri/Cargo.toml         # Rust tests
```

## ➡️ Next steps

- [Organizing Your Library](Organizing-Your-Library.md)
- [Collections](Collections.md)
- [Metadata Providers](Metadata-Providers.md)
