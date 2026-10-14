---
layout: docs
title: Troubleshooting
summary: Common issues and how to resolve them
parent: Documentation Home
nav_order: 9
---

# Troubleshooting

## 🧭 Steam is not detected

If the Dashboard shows no installation, open **Settings** and either press **Auto-detect** or type
the path to your Steam folder manually (for example `C:\Program Files (x86)\Steam`).

Steam Vault looks for Steam in the usual places, including the Windows registry and common drive
locations, but unusual installs may need the path set by hand.

## 🛑 "Steam is running" when saving

Steam Vault refuses to write while Steam is running, to avoid racing Steam's cloud sync and
corrupting your collections. **Quit Steam completely**, then try again. Check the system tray, as
Steam often keeps running in the background after you close its window.

## 🔒 Owned games return `403`

The Steam Web API's `GetOwnedGames` only returns data when your profile's **game details** are
public. Set your Steam privacy settings so game details are visible, or rely on the local library
data instead, which does not require the API at all.

## 🎨 SteamGridDB or IGDB lookups fail

- **SteamGridDB** — confirm the API key is set in Settings and has not expired.
- **IGDB** — confirm both the **client id** and **client secret** are present; Steam Vault fetches
  the access token for you, so a bad secret will fail the token request.

## 🐢 Metadata looks stale

Metadata is cached locally for speed. Steam Store results are considered fresh for about **7 days**.
Refetch a game after that window to pick up changes.

## 🧱 A bare `cargo check` fails after a fresh clone

The Tauri build embeds the compiled frontend, so the frontend must exist before the Rust side builds.
Run `npm install && npm run build` first, or use the npm scripts (`npm run tauri:dev` / `:build`)
which handle ordering for you.

## 🧾 Where are my backups?

Backups are timestamped copies of the Steam files that Steam Vault modified, stored in the app data
directory. See [Collections](Collections.md) for the files involved.

## 🆘 Still stuck?

Open an issue on the [tracker](https://github.com/HELIX-Origin/Steam-Vault/issues) with your OS, app
version, and the exact message shown. Never paste API keys or account identifiers into an issue.
