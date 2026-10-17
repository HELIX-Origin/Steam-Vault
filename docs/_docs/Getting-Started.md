---
layout: docs
title: Getting Started
summary: Prerequisites, building Universal Game Launcher, and your first launch
parent: Documentation Home
nav_order: 2
---

# Getting Started

This guide walks you through building **Universal Game Launcher** from source and running it for the first time.

---

## 📦 1. Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Node.js](https://nodejs.org/) 18 or newer
- The [Tauri v2 prerequisites](https://tauri.app/start/prerequisites/) for your operating system (on Windows this is the WebView2 runtime, which comes preinstalled on Windows 10/11)
- **Windows System Runtimes:**
  - [Microsoft Visual C++ 2015–2022 Redistributable (x64)](https://aka.ms/vs/17/release/vc_redist.x64.exe) — ensures standard C runtime DLLs (`vcruntime140.dll`, `msvcp140.dll`) are present for native bundled SQLite and MSVC dependencies.
  - [Microsoft .NET Desktop Runtime 8.0+ (x64)](https://dotnet.microsoft.com/download/dotnet/8.0) — optional; required if configuring third-party companion tools or external utilities that rely on .NET.

---

## 🛠️ 2. Build and Run

From the repository root:

```bash
# Install frontend dependencies
npm install

# Start local development server with Tauri desktop window
npm run tauri:dev

# Build production bundle and desktop installer
npm run tauri:build
```

> **Note:** The Tauri build embeds the compiled SvelteKit frontend, so a fresh clone must compile the frontend before a bare `cargo check`. Use the project npm scripts (e.g. `npm run check`) or execute `npm run build` first.

---

## 🏁 3. First Launch & Discovery

When the application launches, it displays the **Dashboard** and automatically discovers games installed across all supported storefronts:

1. **Automatic Multi-Platform Scan:** On startup, the launcher inspects standard installation paths and registry manifests for all installed clients — **Steam**, **Epic Games Store**, **GOG Galaxy**, **Ubisoft Connect**, **EA App**, **Xbox / Microsoft Store**, **Battle.net**, **itch.io**, and **Amazon Games**.
2. **Library View:** Navigate to **Library** from the top title bar menu. The left sidebar presents your full aggregated games list grouped into collapsible platform accordions with quick filters (`All`, `Installed`, `★ Favorites`).
3. **Dedicated Showcase:** Click any title in the sidebar to view its high-resolution hero banner, portrait capsule, 10-image screenshot media gallery, storage footprint, and prominent **Play Now** launch button.
4. **Plugin Settings & Credentials:** Optional credentials (such as **SteamGridDB** for custom community artwork or **IGDB** for extended taxonomy) can be configured by clicking **Plugin Settings** in **Settings**. Each enabled plugin presents its own dedicated tab with live connection verification.

---

## 🔒 4. Safe Non-Invasive Architecture

Universal Game Launcher operates completely non-invasively:

- **Zero External File Writes:** The launcher never modifies Steam's internal files (`shortcuts.vdf`, `cloud-storage-namespace-1.json`, or `appmanifest.acf`) and never edits other client databases.
- **No Client Lockout:** You never need to close Steam or any other game launcher. Universal Game Launcher runs concurrently with all official clients safely.
- **Local SQLite Persistence:** All user overrides, custom tags, favorite flags, and artwork assignments persist locally in `database.sqlite` (backed by an in-memory Redis KV tier for sub-millisecond retrieval).

---

## 🧪 5. Verification & Testing

Both the frontend and backend have strict type checks and automated tests:

```bash
# Verify TypeScript types and Svelte diagnostics
npm run check

# Run Rust unit tests
cargo test --manifest-path src-tauri/Cargo.toml

# Full end-to-end verification suite (check, rust, frontend, build)
node scripts/test.mjs check rust frontend build
```

---

## ➡️ Next Steps

- [Organizing Your Library](Organizing-Your-Library.md)
- [Managing Collections](Collections.md)
- [Supported Launchers & Platforms](External-Platforms.md)
- [Manually Added Local Games](Non-Steam-Games.md)
- [Custom Artwork Studio](Artwork-Management.md)
- [Plugin Development Guide](Plugin-Development.md)
