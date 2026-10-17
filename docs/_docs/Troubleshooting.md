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

1. **Verify the Platform Plugin is Enabled**: Go to **Settings → Plugins** and ensure the corresponding platform integration plugin is enabled.
2. **Specify Custom Executable or Install Paths**: If you installed a client or games to a non-standard drive, open **Settings → Plugin Settings**, select the platform's tab, and specify custom executable or installation folder paths.
3. **Trigger a Background Scan**: Open the **Library** view and click the **↻ Scan** button at the top of the left games list sidebar.
4. **Client State**: Ensure the launcher client has been opened at least once so it generates its local manifest databases and registry keys.

---

## 🕹️ EmulationStation & ROM Scans Not Finding Games

If console ROMs are not appearing in your library:

1. **Verify the EmulationStation Plugin is Enabled**: Go to **Settings → Plugins** and check that `platform-emulationstation` is enabled.
2. **Configure ROMs Root Directory**: Open **Settings → Plugin Settings → EmulationStation Integration** and ensure `romsPath` points to the folder containing your console subdirectories (e.g. `D:\Emulation\ROMs`).
3. **Verify Folder Scaffolding**: Ensure console folders follow standard lowercase naming (`nes`, `snes`, `n64`, `gc`, `wii`, `psx`, `ps2`, `gba`, `nds`, etc.). Universal Game Launcher can scaffold these automatically when `autoScaffold: true` is enabled.
4. **Trigger Scan**: Click **↻ Scan** in the Library sidebar to normalize titles and import ROMs.

---

## 🔑 Plugin API Key Verification Fails

If SteamGridDB or IGDB lookups fail or return authorization errors:

1. Open **Settings → Plugin Settings**.
2. Select the provider's tab in the left rail:
   - **SteamGridDB**: Ensure your API key from `steamgriddb.com/profile/preferences/api` is pasted without leading or trailing whitespace.
   - **IGDB**: Ensure both the **Client ID** and **Client Secret** are provided (both are required by Twitch OAuth).
3. Click **⚡ Test Connection** in the dialog to verify credentials directly against the provider's API endpoint.

---

## 🔄 Reverting Metadata to Official Storefront Defaults

If metadata was customized or enhanced by a third-party metadata provider and you want to restore official storefront details:

1. Select the game in the **Library** showcase view.
2. Click **Manage → Manage Metadata** (or click the three-dot action menu).
3. Click **↺ Reset Cache to Provider Defaults**.
4. Universal Game Launcher will clear the cached enhancement record and re-query the authoritative game provider plugin.

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
