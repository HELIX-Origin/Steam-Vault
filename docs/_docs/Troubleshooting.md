---
layout: docs
title: Troubleshooting
summary: Common issues and how to resolve them in Universal Game Launcher
parent: Documentation Home
nav_order: 10
---

# Troubleshooting

This guide covers common issues, platform detection quirks, and steps to resolve them in Universal Game Launcher.

---

## 🧭 Platform or Launcher Not Detected

If games from an installed launcher (Steam, Epic, GOG, Xbox, EA, Ubisoft, Battle.net, itch, Amazon, or Rockstar) do not appear in your library:

1. **Verify the Platform is Enabled**: Go to **Settings → Platforms** and ensure the corresponding platform is toggled on.
2. **Specify Custom Executable or Install Paths**: If you installed a client or games to a non-standard drive, open **Settings → Plugin Settings**, select the platform's tab, and specify custom executable or installation folder paths.
3. **Trigger a Background Scan**: Open the **Library** view and click the **↻ Scan** button at the top of the left games list sidebar.
4. **Client State**: Ensure the launcher client has been opened at least once so it generates its local manifest databases and registry keys.

---

## 🕹️ EmulationStation & ROM Scans Not Finding Games

If console ROMs are not appearing in your library:

1. **Verify Emulation is Enabled**: Go to **Settings → Platforms** and check that EmulationStation is enabled.
2. **Configure ROMs Root Directory**: Open **Settings → Plugin Settings → EmulationStation Integration** and ensure `romsPath` points to the folder containing your console subdirectories (e.g. `D:\Emulation\ROMs`).
3. **Verify Folder Scaffolding**: Ensure console folders follow standard lowercase naming (`nes`, `snes`, `n64`, `gc`, `wii`, `psx`, `ps2`, `gba`, `nds`, etc.). Universal Game Launcher scaffolds standard folders automatically along with `systems.txt` and `systeminfo.txt`.
4. **Trigger Scan**: Click **↻ Scan** in the Library sidebar to normalize titles, extract archives, and import ROMs.

---

## 🔤 Title Normalization & Standalone Slugs

If a standalone game or indie title displays an awkward folder slug or version tag:

1. **Automatic Resolution**: The built-in **Universal Title Normalizer** automatically parses RPG Maker `System.json` `gameTitle` properties and splits lowercase directory slugs.
2. **Test via CLI**: You can test title normalization directly in your terminal:
   ```powershell
   ./universal-game-launcher.exe --cli normalize-title "relicts-auf-aeson"
   # Output: Relicts of Aeson
   ```
3. **Manual Override**: If you prefer a custom name, click the **⚙ Manage** button in the showcase header and enter your preferred Title.

---

## 🔄 Refreshing or Purging Metadata Cache

If you want to re-run the automatic multi-source metadata cascade or clear cached images:

1. Navigate to **Settings → Storage & Cache**.
2. To re-fetch store details and modern MP4 trailers without touching your custom game overrides, click **↻ Refresh Metadata Cache**.
3. To clear all temporary images and cached in-memory KV records, click **🗑 Purge Data Cache**.

---

## 🔑 Plugin API Key Verification Fails

If SteamGridDB or IGDB lookups fail or return authorization errors:

1. Open **Settings → Plugin Settings**.
2. Select the provider's tab in the left rail:
   - **SteamGridDB**: Ensure your API key from `steamgriddb.com/profile/preferences/api` is pasted without leading or trailing whitespace.
   - **IGDB**: Ensure both the **Client ID** and **Client Secret** are provided (both are required by Twitch OAuth).
3. Click **⚡ Test Connection** in the dialog to verify credentials directly against the provider's API endpoint.

---

## ⚙️ Missing System Runtimes (`VCRUNTIME140.dll` or WebView2)

On fresh Windows installations:
- If the application fails to launch with a missing DLL error, install the [Microsoft Visual C++ 2015–2022 Redistributable (x64)](https://aka.ms/vs/17/release/vc_redist.x64.exe).
- If the window fails to render on Windows 10, ensure the [Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) is installed (preinstalled on Windows 11).

---

## ➡️ Next Steps

- [Settings & Plugin Configuration](Settings-and-API-Keys.md) — configuring plugin settings.
- [Available Plugins Catalog](Available-Plugins.md) — inspecting available plugins.
- [FAQ](FAQ.md) — frequently asked questions.
