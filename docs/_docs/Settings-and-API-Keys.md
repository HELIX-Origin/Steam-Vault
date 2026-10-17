---
layout: docs
title: Settings & Plugin Configuration
summary: Configuration reference for Universal Game Launcher, Storage & Cache controls, and the Plugin Settings dialog
parent: Documentation Home
nav_order: 9
---

# Settings & Plugin Configuration

Universal Game Launcher cleanly separates core application preferences, platform detection options, and storage cache maintenance from third-party API configurations. All settings are stored locally on your machine with zero cloud telemetry.

---

## ⚙️ Settings View Layout (`/settings`)

The **Settings** view is structured into clear, functional sections:

### 1. General & Localization
- **System Language**: Display and metadata language. Select from 30 officially supported languages or auto-detect from system locale. See [Localization](Localization.md).
- **Store Country**: Two-letter country/region code (e.g. `US`, `GB`, `DE`) used for regional store pricing, currency, and age ratings.

### 2. Platforms & Clients
- Renders authentic vector SVG platform icons (`PlatformIcon.svelte`) for all 12 supported game providers.
- Toggle individual platforms (Steam, Epic Games, GOG Galaxy, Xbox, EA Desktop, Ubisoft Connect, Battle.net, itch.io, Amazon Games, Rockstar Games, Standalone, EmulationStation) to control which clients are scanned on startup.
- Dedicated button to launch the **Plugin Settings Dialog** for advanced client paths and alternate backends.

### 3. Storage & Cache
- Dedicated hub for cache maintenance and local database integrity:
  - **↻ Refresh Metadata Cache**: Re-executes the automatic multi-source metadata engine across all library titles to fetch updated descriptions, reviews, and artwork without touching your custom per-game overrides.
  - **🗑 Purge Data Cache**: Clears in-memory KV records (`mem_kv`) and cached disk artwork, allowing the application to rebuild assets cleanly from scratch.

---

## 🧩 Dedicated Plugin Settings Dialog

To maintain a clean and modular architecture, third-party API keys and platform-specific configurations are **decoupled from general settings**. Each enabled plugin that declares configurable fields renders inside the **Plugin Settings** sub-window dialog.

You can open this dialog by clicking **Open Plugin Settings** or the gear icon (`⚙`) next to any configurable plugin in the Platforms or Plugins list.

### Features of the Plugin Settings Dialog

- **Tabbed Plugin Navigation**: A clean left-hand rail lists every enabled plugin with configurable settings (Steam, GOG, Epic, itch.io, EmulationStation, SteamGridDB, IGDB, GamerPower, Storage Analyzer).
- **Dynamic Field Schemas**: Renders inputs based on each plugin's `PluginSettingsConfig`:
  - `text` / `string`: General configuration values (paths, directory locations, countries).
  - `password`: API keys and secrets with a visibility eye toggle (`👁`).
  - `boolean`: Feature toggles (e.g. `allowNsfw`, `useLegendary`, `autoScaffold`).
  - `number`: Numeric parameters (check intervals, discount thresholds, volume).
  - `select`: Dropdown options (regional box art, preferred language).
- **Test Connection (`⚡ Test Connection`)**: Verify your API key or client path in real time before saving. The dialog reports instant success or detailed error feedback.
- **Safe Persistence**: Settings are persisted to the in-memory KV store (`plugin_cfg:<id>`) backed by `database.sqlite`.

---

## 🔑 Common Plugin API Keys

Universal Game Launcher works **100% keyless** out of the box. API keys are entirely optional and only needed if you wish to query specific third-party APIs:

| Plugin | Key Type | Where to Obtain | Key Required |
| :--- | :--- | :--- | :---: |
| **SteamGridDB** (`metadata-universal`) | API Key | `steamgriddb.com` → Preferences → API | Optional |
| **IGDB** (`metadata-universal`) | Client ID & Client Secret | Twitch Developer Console (`dev.twitch.tv/console`) | Optional |
| **itch.io** (`platform-universal`) | API Key | `itch.io` → Account Settings → API Keys | Optional |

All primary built-in data sources — including Steam Store (`appdetails` and modern MP4 trailer streams), GOG Catalog v1, Libretro CDN, Wikipedia/Wikidata MediaWiki API, PCGamingWiki Cargo API, ProtonDB, GamerPower, and Storage Analyzer — require **zero API keys or credentials**.

---

## 🔐 Storage & Credential Security

- **Local Storage**: Plugin configurations and API keys are stored locally on your machine in generic SQLite (`database.sqlite`) via the in-memory KV tier (`plugin_cfg:<plugin_id>`).
- **No Cloud Telemetry**: Keys and personal settings are **never** committed, never uploaded to HELIX Origin servers, and never sent to any third party other than the official provider API endpoint.
- **Threat Model**: Keys are stored as plain text locally on your disk, consistent with standard desktop application threat models (e.g. Steam's `loginusers.vdf`). Keep your user machine account secure.

---

## ➡️ Next Steps

- [Available Plugins Catalog](Available-Plugins.md) — complete catalog of Universal Core plugins.
- [Organizing Your Library](Organizing-Your-Library.md) — library showcase and per-game customization.
- [Metadata Providers](Metadata-Providers.md) — multi-source metadata engine.
- [Storage Management](Storage-Management.md) — drive utilization and game storage exploration.
