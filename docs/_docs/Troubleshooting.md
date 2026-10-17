---
layout: docs
title: Troubleshooting
summary: Common issues and how to resolve them in Universal Game Launcher
parent: Documentation Home
nav_order: 10
---

# Troubleshooting

This guide covers common issues, platform detection quirks, and steps to resolve them.

---

## 🧭 Platform or Launcher Not Detected

If games from an installed launcher (Steam, Epic, GOG, Xbox, EA, Ubisoft, Battle.net, etc.) do not appear:

1. **Verify the Platform Plugin is Enabled**: Go to **Settings → Plugins** and ensure the platform integration plugin is enabled.
2. **Check Custom Install Directories**: If you installed a client or games to a non-standard drive, open **Settings → Plugin Settings**, select the platform's tab, and specify custom known directory paths.
3. **Trigger a Background Scan**: Click the `↻ Scan` button at the top of the left games list sidebar.
4. **Client Installation State**: Ensure the launcher client has been run at least once so it generates its local manifest databases and registry keys.

---

## 🛑 "Client is Running" During File Operations

When performing physical file relocations (such as moving Steam library games between drives), Universal Game Launcher requires the client to be closed to prevent cloud save sync conflicts and manifest file locking.

- **Quit the Client Completely**: Check your system tray (notification area) and exit the client (Steam, Epic, etc.) before initiating disk moves.
- **Normal Operations Do Not Require Closing**: Note that browsing, launching games, organizing collections, and applying custom artwork can all be performed while clients are running freely.

---

## 🔑 Plugin API Key Verification Fails

If SteamGridDB, IGDB, or itch.io lookups fail or return authorization errors:

1. Open **Settings → Plugin Settings**.
2. Select the provider's tab in the left rail.
3. Verify your API credentials:
   - **SteamGridDB**: Ensure the API key is active and has no leading/trailing spaces.
   - **IGDB**: Ensure both the **Client ID** and **Client Secret** are provided (both are required by Twitch OAuth).
4. Click **⚡ Test Connection** in the dialog to verify credentials directly against the provider's API.

---

## 🧩 Community Drop-In Plugin Issues

If a plugin dropped into `%APPDATA%\universal-game-launcher\plugins\<plugin_id>\` does not load:

1. **Check Manifest Filename**: The folder must contain a valid `plugin.json` (or `index.json` / `manifest.json`).
2. **Validate JSON Syntax**: Ensure there are no syntax errors (such as trailing commas) in your manifest file.
3. **Required Fields**: Ensure `id`, `name`, `version`, and `type` (`platform`, `metadata`, `media`, or `utility`) are specified.
4. **Reload Plugins**: Click **Settings → Plugins → Reload Plugins** or restart the application to trigger a directory re-scan.

---

## ⚙️ Missing System Runtimes (`VCRUNTIME140.dll` or MSVC)

If launching the application displays a missing runtime error:
- Install the official [Microsoft Visual C++ 2015–2022 Redistributable (x64)](https://aka.ms/vs/17/release/vc_redist.x64.exe).
- If using companion external tools that require .NET, install the [Microsoft .NET Desktop Runtime (x64)](https://dotnet.microsoft.com/download/dotnet/8.0).

---

## 🧱 Build Fails on Fresh Clone (`cargo check`)

The Tauri build embeds the compiled SvelteKit frontend into the Rust binary. On a fresh repository clone, you must build the frontend before compiling Rust:

```bash
# Correct build sequence:
npm install
npm run build
npm run check
cargo test --manifest-path src-tauri/Cargo.toml
```

Alternatively, use `npm run tauri:dev` or `npm run tauri:build`, which orchestrate the frontend and backend build pipelines automatically.

---

## 🧾 Where are My Settings and Backups?

All data is stored locally in your operating system's standard application data directory:
- **Windows**: `%APPDATA%\universal-game-launcher\`
- **Linux**: `~/.config/universal-game-launcher/`
- **macOS**: `~/Library/Application Support/universal-game-launcher/`

Inside this folder:
- `database.sqlite` — SQLite database storing collections, tags, favorites, and cached metadata.
- `plugins/` — Drop-in community plugins folder (accessible via **Settings → 📂 Open Plugins Folder**).
- `backups/` — Timestamped backups made prior to physical file modifications.

---

## 🆘 Still Stuck?

Open an issue on the [Universal Game Launcher Issue Tracker](https://github.com/HELIX-Origin/Universal-Game-Launcher/issues) with:
1. Your operating system and version.
2. Universal Game Launcher version.
3. Steps to reproduce the issue and any relevant log output from the `logs/` directory.

> **Caution:** Never paste private API keys, passwords, or personal credentials into public issue reports.
